# Single-file Open Routing

**Status:** Approved · **Date:** 2026-05-19 · **Author:** Chivier + Claude

## Problem

Opening a single file (CLI, macOS Finder "Open With", drag-drop, pending-file
drain) always lands the file as a new tab in whichever window happens to have
focus, even when that file does not belong to the open project. This pollutes
the project's tab strip with strangers and makes the sidebar/Cmd+P search feel
misleading.

## Goal

Route each incoming single-file open to the window whose context *owns* the
file, and spawn a fresh single-file window when no window owns it.

## Routing rule

For each window we define `canClaim(filePath, win)`:

```
if win has project P:
  canClaim = pathStartsWithChild(filePath, P) || filePath inside P
if win is in single-file mode with ≥ 1 tab:
  canClaim = parent(filePath) ∈ {parent(t.path) for t in tabs}
if win is empty (no project, no tabs):
  canClaim = true  // ultimate fallback to avoid spawning when an idle window exists
```

Resolution order when multiple windows bid `canClaim = true`:
1. The window that initiated the open (focus stays put).
2. The first window with `win.project != null` that bids true (project owner beats single-file).
3. Any single-file window that bids true.
4. Any empty window.

If no window bids true → spawn a new single-file window via the existing
`openFileInNewWindow(path, line, col)` helper.

## Architecture

A new chokepoint, `routeSingleFileOpen(path, line, col)`, replaces direct
`openFileByPath` calls at every single-file entry point. Routing logic lives in
Rust (the CLI single-instance callback also needs it, and Rust already owns
window enumeration via `app.webview_windows()`).

### Rust side (`core/src/services/file_routing.rs` — new)

- `route_single_file_open(app, source_label, path, line, col)` async fn:
  1. Generate a `bid_id` (UUID-ish u64).
  2. Insert `(bid_id, mpsc::Sender<Bid>)` into a `Mutex<HashMap>` state.
  3. `app.emit("file-open-bid-request", { id, path })` to every webview window.
  4. Spawn a tokio task that collects bids until either:
     - all known window labels at request-time have replied, or
     - 250 ms elapses.
  5. Choose winner by the priority list above.
  6. Either `app.emit_to(winner_label, "open-file-deliver", { path, line, col })`
     and `set_focus` on that window, **or** spawn a new window with the same URL-
     hash seed that `openFileInNewWindow` uses today.

- `submit_file_open_bid(state, bid_id, window_label, can_claim, has_project)`
  Tauri command — frontends call this in response to `file-open-bid-request`.

State: `FileRoutingState` (Mutex over `HashMap<u64, BidCollector>`), `.manage()`
in `lib.rs` alongside the other state types.

### Frontend side

- **New service** `app/lib/services/file-route.ts`:
  - `computeCanClaim(filePath: string): { canClaim: boolean; hasProject: boolean }`
    — implements the rule above against `projectStore` + `tabsStore`.
  - `wireFileRoutingBids()` — listens to `file-open-bid-request`, computes,
    calls `commands.submitFileOpenBid(...)`.
  - `routeSingleFileOpen(path, line?, col?)` — thin wrapper around
    `commands.routeSingleFileOpen(...)`. Used at every entry point.

- **`app/lib/composables/app-events.svelte.ts` changes**:
  - Replace direct `openFileByPath` calls in 3 places (pending-file drain,
    `open-file` event, drag-drop) with `routeSingleFileOpen(path, line)`.
  - Add a listener for `open-file-deliver` that calls the existing
    `openFileByPath` (now repurposed as the *local* "actually open" path —
    no routing).
  - Wire `wireFileRoutingBids()` once per window.

- **`app/lib/services/cli-open.ts` changes**:
  - `handleCliOpen` no longer needs the file routing branch — it just iterates
    `payload.files` and calls `routeSingleFileOpen(...)`. (Folder routing
    unchanged.) `force_new_window` flag is passed through so Rust can skip the
    bid round and spawn directly.
  - Existing `openFileInNewWindow` helper stays — Rust calls back into it via
    the new-window spawn step.

- **`App.svelte` `consumeWindowSeed.openFile` callback** is unchanged — that's
  the "I was *spawned* with this file" path, not an incoming open.

## Event/IPC summary

| Direction | Channel | Payload | When |
|-----------|---------|---------|------|
| FE → BE | `route_single_file_open` (cmd) | `{ source: label, path, line, col, force_new: bool }` | Each entry point |
| BE → FE (all) | `file-open-bid-request` (event) | `{ bid_id, path }` | Routing kickoff |
| FE → BE | `submit_file_open_bid` (cmd) | `{ bid_id, label, can_claim, has_project }` | Bid response |
| BE → FE (winner) | `open-file-deliver` (event) | `{ path, line, col }` | After resolution |

## Edge cases

- **Cold start, no windows ready**: Pending-file drain runs in the first
  window's onMount. By that time the window exists, so bid round works
  normally. The first window itself bids `canClaim` (empty → true) and wins.
- **`-n / --new-window` flag**: Rust skips the bid round and spawns
  immediately, preserving today's behavior.
- **Bid timeout exceeded**: Use the bids collected so far. If still none,
  spawn new window.
- **Window closes mid-bid**: Its label drops out of `app.webview_windows()`;
  the collector treats it as not responding. Timeout handles cleanup.
- **Non-text extension**: Rust does *not* filter — frontends already filter
  via `TEXT_EXTENSIONS`. A bid responder with `canClaim=true` will later
  filter inside `open-file-deliver` handler; if the file is unsupported, the
  handler returns silently. This is unchanged from today.
- **Multiple identical bids race**: Rust collects in arrival order; first
  matching priority wins. Stable enough.

## Out of scope (deferred)

- Cross-window routing for *folders* — `openProjectInThisWindow` already
  always spawns new, which is correct.
- A user-visible toggle "always open in new window". The `-n` CLI flag
  already covers power users; UI toggle is YAGNI for v1.
- Migrating already-open foreign tabs out of a project window — purely
  historical state.

## Test plan

- Unit: `computeCanClaim` covers project/single-file/empty modes.
- Rust unit: bid resolver picks correct winner across priorities.
- E2E (browser, mocked IPC): drag-drop external file when project open →
  spawn called; drag-drop internal file → opens as tab.
- Manual: `novelist /some/external/file.md` from terminal with a project
  window already open → new window appears, project window unaffected.

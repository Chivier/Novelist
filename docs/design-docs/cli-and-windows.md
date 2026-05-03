# CLI Entry Point & Window Routing

> Status: shipped in v0.2.3 · primary code: `core/src/services/cli.rs`,
> `app/lib/services/cli-open.ts`, `app/lib/composables/app-events.svelte.ts`

## Goals

1. `novelist file.md` and `novelist /folder` should feel like VS Code's
   `code` command — one process, sensible window routing, no duplicate
   startups.
2. The same parser handles cold start (`std::env::args()` in `setup()`)
   and hot path (second invocation forwarded by
   `tauri-plugin-single-instance`).
3. The user can install the shim from inside the app
   (no manual `ln -s` for typical cases).

## Routing rules

| Argument | Cold start | Hot path |
|---|---|---|
| Folder | Open in main window | **Always** new window |
| File   | Open in main window | Open in this window if it's in single-file mode AND user did not pass `-n`; else new window |
| `-n` / `--new-window` | (ignored — only one window exists) | Force new window for files |
| `-g FILE:LINE[:COL]` | Open + jump to line | Same |
| `-h` / `-v` | Print + `exit(0)` before Tauri init | Same |

## Components

```
┌─────────────────────────────────────────────────────────────────┐
│ Cold start (first process)                                      │
│   main.rs → run() → handle_early_exit_flags()  ← -h/-v exit     │
│                  → parse_argv(argv, cwd)                        │
│                  → push files → PendingOpenFiles                │
│                  → push folders → PendingOpenProjects           │
│                  → tauri::Builder...setup()                     │
│                                                                 │
│   App.svelte mount → wireAppEvents:                             │
│     get_pending_open_projects → open first as project           │
│                                  spawn windows for the rest     │
│     get_pending_open_files     → open all as tabs               │
└─────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│ Hot path (second process, same user)                            │
│   tauri-plugin-single-instance callback                         │
│     ↳ parse_argv(argv, cwd)  (same parser as above)             │
│     ↳ emit("cli-open", CliOpenPayload)                          │
│     ↳ exits the second process                                  │
│                                                                 │
│   Frontend listener (app-events.ts):                            │
│     ↳ handleCliOpen(payload, openFileHere)                      │
│         folders → openProjectInThisWindow(spawnNew=true)        │
│         files   → openFileHere if isSingleFileMode && !-n       │
│                   else openFileInNewWindow                      │
└─────────────────────────────────────────────────────────────────┘
```

## Inter-window seeding

Spawned windows can't read shared Rust state safely (they all drain the
same `PendingOpenFiles`), so the routing helper passes the seed in the
URL hash:

```
index.html#project=<encoded-path>
index.html#file=<encoded-path>&line=<n>&col=<n>
```

App.svelte's mount calls `consumeWindowSeed()` (in
`app/lib/services/cli-open.ts`), which reads the hash, opens the
indicated target, then clears the hash via `history.replaceState` so a
reload doesn't re-fire.

## CLI shim

The bundled shim ships under `core/bundled-cli/`:

- `novelist` — POSIX shell script for macOS + Linux. Detached background
  launch (`& exit 0`) keeps the shell prompt responsive; the
  single-instance plugin handles dedup.
- `novelist.cmd` — Windows launcher using `start ""`.

The in-app installer (`commands::cli_shim::install_cli_shim`) symlinks
the macOS/Linux shim to `/usr/local/bin/novelist`; on Windows it copies
to `%LOCALAPPDATA%\Novelist\bin\novelist.cmd` and the dialog instructs
the user to add that directory to `PATH`. We deliberately do **not**
escalate to root or edit the registry from the app — surface the error
and let the user decide.

## Out of scope (for now)

- `--wait` / `$EDITOR` integration. Needs backend tracking of tab close
  events plus IPC back to the shim shell process (typically via a
  sentinel file). Reasonable v0.3.x candidate.
- `--diff` two-file diff view. Needs an editor pane mode that doesn't
  exist yet.
- `--add` to add a folder to an existing project. Multi-folder workspaces
  not supported in the project model yet.
- Windows auto-PATH editing. PowerShell vs cmd vs WSL nuances make
  silent registry edits risky.

## Tests

- `core/src/services/cli.rs` — 14 unit tests covering files, folders,
  flags, relative paths, `--`, `--goto=` form, Windows drive letters,
  unknown flag forward-compat, and help text shape.
- `tests/unit/composables/app-events.test.ts` — drains
  `get_pending_open_projects` / `get_pending_open_files`, asserts
  `cli-open` listener registration and teardown bookkeeping.

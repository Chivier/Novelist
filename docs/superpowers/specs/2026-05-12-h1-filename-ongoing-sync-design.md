# H1 ↔ Filename Ongoing Sync — Design

**Status:** Approved 2026-05-12
**Owner:** Chivier
**Scope:** `app/lib/stores/tabs.svelte.ts`, `app/lib/utils/placeholder.ts`, `app/lib/i18n/locales/{en,zh-CN}.ts` (+ unit tests)
**Supersedes (partial):** behavior described in `docs/exec-plans/completed/2026-05-07-v0.2.4-rename-and-macros.md` §3.5 ("rename fires once, then stops")

## Problem

The template system substitutes `{title}` with `"Untitled"` at create time, then v0.2.4's H1 auto-rename fires **once** when the first H1 is saved into a placeholder file. After that single rename, `isPlaceholder()` no longer matches, so subsequent H1 edits never touch the filename.

Conceptually `{title}` *is* the file's H1 — they're meant to be the same thing — but in practice the link is cut after the first save. Users who rename a chapter ("开篇" → "序幕") expect the filename to follow. Today it doesn't.

## Decision

Extend H1 auto-rename from a one-shot to **ongoing one-way sync (H1 → filename)** keyed on a per-tab `lastSyncedH1` and structural matching of the previous H1 inside the current filename. Trigger only on save (manual `Cmd+S` or auto-save) — the same `tryRenameAfterSave` entry point already in use.

The existing `auto_rename_from_h1` setting remains the single gate; its semantics widen from "first rename" to "filename follows H1".

## Non-goals

- **Not bidirectional.** Renaming the file does not edit the H1.
- **Not live.** No rename on every keystroke; only at save boundaries.
- **No new persistent metadata.** `lastSyncedH1` lives on the in-memory `TabState`; nothing is written to frontmatter, sidecar files, or project index.
- **No external-edit propagation.** If a file watcher reload brings in a new H1 (Git checkout, another editor), we update `lastSyncedH1` silently but do not rename — avoids fighting external tooling.

## State

Add one field to `TabState`:

```ts
interface TabState {
  // ... existing fields ...
  lastSyncedH1: string | null;
}
```

**Initialization points:**

| Event | `lastSyncedH1` set to |
|-------|------------------------|
| `openTab` / `openTabInPane` | `extractFirstH1(content)` (or `''` if none) |
| Watcher-triggered content reload | `extractFirstH1(newContent)` |
| Successful rename (path A or B below) | the H1 that was just synced out |
| Scratch files (no real path) | `null`, never participates |

The field is not persisted across app restarts — re-derived from disk content on `openTab`, which is correct because the filename is also already in sync at that moment.

## Algorithm (in `tryRenameAfterSave`)

Called after each successful `writeFile` (which is what auto-save and manual save both funnel through today).

```
if (!autoRenameFromH1) return filePath
if (isScratch(filePath)) return filePath

oldH1 = tab.lastSyncedH1
newH1 = extractFirstH1(content)

if (newH1 === oldH1) return filePath        // no change

fileName = basename(filePath)

// Path A — first-time entry into sync (existing v0.2.4 behavior)
if (isPlaceholder(fileName)) {
  if (newH1 && newH1.trim().length > 0) {
    newName = renameFromH1(fileName, newH1, siblings)
    if (newName && newName !== fileName) {
      await commands.renameItem(filePath, newName, true)
      // ...update tab path, broadcast...
    }
  }
  tab.lastSyncedH1 = newH1
  return newPathOrFilePath
}

// Path B — ongoing structural sync
if (!oldH1 || oldH1.length === 0) {       // no previous anchor → not in sync
  tab.lastSyncedH1 = newH1                 // adopt silently for next time
  return filePath
}
if (!newH1 || newH1.trim().length === 0) {// user emptied H1 → keep filename
  // Intentionally do NOT update lastSyncedH1: if they retype the same H1
  // later we still want to recognize "no change".
  return filePath
}

sanitizedOld = sanitizeFilenameStem(oldH1)
sanitizedNew = sanitizeFilenameStem(newH1)
if (sanitizedOld === sanitizedNew) {       // edits below sanitize granularity
  tab.lastSyncedH1 = newH1
  return filePath
}

stem = fileName.replace(/\.md$/, '')
idx = stem.lastIndexOf(sanitizedOld)       // rightmost — title slot is at end
if (idx === -1) {                          // user manually renamed → detach
  tab.lastSyncedH1 = newH1                 // future edits also won't match → stay detached
  return filePath
}

newStem = stem.slice(0, idx) + sanitizedNew + stem.slice(idx + sanitizedOld.length)
newName = bumpStemUntilFree(newStem + '.md', siblings, fileName)
result = await commands.renameItem(filePath, newName, true)
if (result.status !== 'ok') return filePath

// update tab path, broadcast cross-window, return new path
tab.lastSyncedH1 = newH1
return newPath
```

## Properties (intentional)

1. **Manual rename detaches.** User renames `第1章-开篇.md` → `chapter1.md` in the sidebar. Next save: `oldH1="开篇"` is not in `"chapter1"` → no rename. `lastSyncedH1` updates to current H1 so the detach is sticky.
2. **Empty H1 is safe.** Deleting the H1 does **not** revert the filename to `Untitled`. The filename stays put.
3. **Prefix preserved.** `第1章-开篇.md` + new H1 "序幕" → `第1章-序幕.md`. The `第1章-` prefix is structural, not part of the title slot, and survives the substring replace.
4. **`lastIndexOf` over `indexOf`.** Robust to cases where the H1 string overlaps with prefix literals (e.g. `第开篇章-开篇.md`).
5. **External edits don't rename.** Watcher reloads update `lastSyncedH1` but don't trigger rename — Git checkouts, external editors, AI tools writing the file behind our back won't get their files renamed underneath them.
6. **No new IPC, no new persistence.** Reuses existing `renameItem` + `broadcastFileRenamed` + BLAKE3 self-write suppression. Watcher rename-ignore already covers our own renames.

## Edge cases

| Case | Behavior |
|------|----------|
| Collision after substitution (sibling already named `第1章-序幕.md`) | `bumpStemUntilFree` appends ` 2`, ` 3`, … to the whole stem |
| H1 sanitizes to empty (e.g. `# ///`) | Treated as empty H1 → keep filename |
| H1 unchanged after sanitize (e.g. `开篇 ` → `开篇`) | Short-circuit; `lastSyncedH1` updated |
| First save of a non-placeholder file with no prior H1 | Adopts current H1 as `lastSyncedH1`, does not rename |
| File renamed in another window | Existing `broadcastFileRenamed` listener updates `tab.filePath`; `lastSyncedH1` is unchanged, so next save uses the up-to-date filename and old H1 |
| Scratch buffer | Never participates (early return) |
| Auto-save fires during external write | BLAKE3 self-write suppression + the existing `tryRenameAfterSave` gating already handle this |

## I18N copy update

Setting key `settings.editor.newFile.autoRenameHint`:

- **en (was):** `"Only affects auto-generated names. Stops once you manually rename."`
- **en (new):** `"Filename follows the H1 heading on each save. Manually renaming the file detaches the sync."`
- **zh-CN (was):** corresponding zh string
- **zh-CN (new):** `"保存时让文件名跟随 H1 标题。手动重命名文件即可脱离同步。"`

Label key `settings.editor.newFile.autoRename` also gets a tightened
phrasing — the v0.2.4 wording ("Auto-rename placeholder files from H1")
implies one-shot behavior and becomes misleading under Path B:

- **en:** `"Sync filename with H1 on save"`
- **zh-CN:** `"保存时让文件名跟随 H1 标题"`

## Testing

Unit tests in `app/lib/utils/placeholder.test.ts` (existing) — add cases for:

- Structural sync replacing rightmost match (`第1章-开篇.md` + 开篇→序幕 → `第1章-序幕.md`)
- `lastIndexOf` correctness with overlapping prefix
- Manual-rename detach (filename without old H1 → no rename)
- Empty new H1 → no rename
- Sanitize equality short-circuit

Unit tests in `app/lib/stores/tabs.test.ts` (or equivalent — confirm location during plan) — add cases for:

- `lastSyncedH1` initialization on `openTab`
- Watcher reload updates `lastSyncedH1` without rename
- Successful path-B rename updates `lastSyncedH1` and `tab.filePath`

Existing v0.2.4 placeholder→rename tests must continue to pass unchanged (path A).

## Out of scope (future)

- Project-level "track H1 sync" toggle (per-file metadata)
- Renaming the file → editing the H1 (bidirectional)
- Bulk re-sync across an entire project folder
- Sync indicator in the UI ("this file is tracking H1")

## Changelog & spec breadcrumb

- `CHANGELOG.md`: one line under the next version, e.g. `feat(editor): filename follows H1 on every save (was first-save only)`
- Cross-link from `docs/exec-plans/completed/2026-05-07-v0.2.4-rename-and-macros.md` to this design so future readers find the updated semantics.

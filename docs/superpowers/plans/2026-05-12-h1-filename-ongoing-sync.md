# H1 ↔ Filename Ongoing Sync — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend v0.2.4's one-shot H1 auto-rename into an ongoing one-way sync (H1 → filename) that fires at every save, detaches on manual rename, and stays silent on external edits.

**Architecture:** Add a per-tab `lastSyncedH1` field (in-memory only). Refactor `tabsStore.tryRenameAfterSave` into two paths — Path A (existing placeholder→title behavior, unchanged) and Path B (structural substring match of `lastSyncedH1` inside the current filename, replace with sanitized new H1). Initialize `lastSyncedH1` on every entry point that brings content into a tab (openTab, openTabInPane, reloadContent). Update i18n hint text. Tests TDD first.

**Tech Stack:** Svelte 5 runes, Vitest, existing `commands.renameItem` / `commands.broadcastFileRenamed` IPC, existing `extractFirstH1` / `sanitizeFilenameStem` / `isPlaceholder` / `renameFromH1` utilities.

**Spec:** `docs/superpowers/specs/2026-05-12-h1-filename-ongoing-sync-design.md`

---

## File Structure

**Modify:**
- `app/lib/stores/tabs.svelte.ts` — add `lastSyncedH1` field; refactor `tryRenameAfterSave`; initialize `lastSyncedH1` in `openTab`, `openTabInPane`, `reloadContent`, `updateFilePath`
- `app/lib/utils/placeholder.ts` — add small pure helper `applyH1Substitution(fileName, oldH1, newH1, siblings)` used by Path B
- `app/lib/i18n/locales/en.ts` — replace `autoRenameHint` text
- `app/lib/i18n/locales/zh-CN.ts` — replace `autoRenameHint` text
- `docs/exec-plans/completed/2026-05-07-v0.2.4-rename-and-macros.md` — add a top breadcrumb pointing to the new spec
- `CHANGELOG.md` — add one line under the next unreleased section

**Create:**
- `tests/unit/utils/placeholder-h1-sub.test.ts` — unit tests for `applyH1Substitution`
- `tests/unit/stores/tabs-ongoing-rename.test.ts` — integration-flavored tests for Path B in `tryRenameAfterSave`

Existing test files left in place; the v0.2.4 behavior in `tests/unit/stores/tabs-auto-rename.test.ts` must keep passing unchanged (Path A regression coverage).

---

## Task 1: Pure helper `applyH1Substitution` (TDD)

**Files:**
- Test: `tests/unit/utils/placeholder-h1-sub.test.ts` (create)
- Modify: `app/lib/utils/placeholder.ts`

The helper is the meat of Path B. Keeping it in `placeholder.ts` makes it pure / unit-testable in isolation, mirrors `renameFromH1`'s structure, and avoids dragging tab state into the math.

- [ ] **Step 1.1: Write the failing test file**

Create `tests/unit/utils/placeholder-h1-sub.test.ts`:

```ts
import { describe, it, expect } from 'vitest';
import { applyH1Substitution } from '$lib/utils/placeholder';

describe('applyH1Substitution', () => {
  it('replaces rightmost occurrence of old H1 in stem (prefix preserved)', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '序幕', [])).toBe('第1章-序幕.md');
  });

  it('uses lastIndexOf when old H1 string appears in the prefix too', () => {
    // contrived: prefix literal happens to contain the same characters as old H1
    expect(applyH1Substitution('第开篇章-开篇.md', '开篇', '终结', [])).toBe('第开篇章-终结.md');
  });

  it('sanitizes the new H1 before substitution (filesystem-forbidden chars)', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', 'a/b:c', [])).toBe('第1章-a-b-c.md');
  });

  it('returns null when old H1 (sanitized) is not in the current filename — manual rename detach', () => {
    expect(applyH1Substitution('chapter1.md', '开篇', '序幕', [])).toBeNull();
  });

  it('returns null when sanitized new H1 is empty (nothing to substitute with)', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '   ', [])).toBeNull();
  });

  it('returns null when sanitized old H1 is empty', () => {
    expect(applyH1Substitution('第1章-开篇.md', '   ', '序幕', [])).toBeNull();
  });

  it('returns null when old and new H1 sanitize to the same stem', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '开篇 ', [])).toBeNull();
  });

  it('bumps with " 2" suffix on sibling collision', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '序幕', ['第1章-序幕.md'])).toBe(
      '第1章-序幕 2.md',
    );
  });

  it('does not collide with itself', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '序幕', ['第1章-开篇.md'])).toBe(
      '第1章-序幕.md',
    );
  });

  it('preserves the .md extension casing and does not touch directories', () => {
    expect(applyH1Substitution('开篇.md', '开篇', '序幕', [])).toBe('序幕.md');
  });
});
```

- [ ] **Step 1.2: Run test, verify it fails**

Run: `pnpm vitest run tests/unit/utils/placeholder-h1-sub.test.ts`
Expected: FAIL — `applyH1Substitution is not a function` (or import error).

- [ ] **Step 1.3: Implement `applyH1Substitution` in `app/lib/utils/placeholder.ts`**

Append below the existing `renameFromH1` / `bumpStemUntilFree` block (after line ~415):

```ts
/**
 * Path B of ongoing H1→filename sync. Compute the new filename when a tab's
 * H1 has changed from `oldH1` to `newH1` and the file is already past the
 * placeholder→title transition (Path A in tabsStore.tryRenameAfterSave).
 *
 * Returns null when:
 * - either side sanitizes to empty (no anchor to act on)
 * - old and new sanitize to the same stem (nothing to do)
 * - the sanitized old H1 is not present in the current filename stem
 *   (i.e. the user manually renamed the file — sync auto-detaches)
 *
 * On a hit, the rightmost occurrence of sanitized old H1 inside the stem is
 * replaced with sanitized new H1 (lastIndexOf — title slot is conventionally
 * at the end of the stem). Resulting collisions with `siblings` get bumped
 * via the existing `bumpStemUntilFree` ` 2`/` 3`/… scheme.
 */
export function applyH1Substitution(
  currentName: string,
  oldH1: string,
  newH1: string,
  siblings: string[],
): string | null {
  const sanitizedOld = sanitizeFilenameStem(oldH1);
  const sanitizedNew = sanitizeFilenameStem(newH1);
  if (sanitizedOld.length === 0) return null;
  if (sanitizedNew.length === 0) return null;
  if (sanitizedOld === sanitizedNew) return null;

  const stem = currentName.replace(/\.md$/, '');
  const idx = stem.lastIndexOf(sanitizedOld);
  if (idx === -1) return null;

  const newStem = stem.slice(0, idx) + sanitizedNew + stem.slice(idx + sanitizedOld.length);
  const newName = `${newStem}.md`;
  if (newName === currentName) return null;

  return bumpStemUntilFree(newName, siblings, currentName);
}
```

Note: `sanitizeFilenameStem` is already imported at the top of the file; `bumpStemUntilFree` is defined in the same file.

- [ ] **Step 1.4: Run test, verify it passes**

Run: `pnpm vitest run tests/unit/utils/placeholder-h1-sub.test.ts`
Expected: PASS — all 10 cases green.

- [ ] **Step 1.5: Commit**

```bash
git add app/lib/utils/placeholder.ts tests/unit/utils/placeholder-h1-sub.test.ts
git commit -m "feat(placeholder): add applyH1Substitution for ongoing H1 sync"
```

---

## Task 2: Track `lastSyncedH1` on `TabState`

**Files:**
- Modify: `app/lib/stores/tabs.svelte.ts` (lines 1-30, 240-275, 444-456, 423-433)

This task adds the field and initializes it at every content-ingress point. It does NOT yet wire it into `tryRenameAfterSave` — that's Task 3 — so the field is set but unread. This keeps the diff small and verifiable.

- [ ] **Step 2.1: Add field to `TabState` interface**

Edit `app/lib/stores/tabs.svelte.ts:12-28`. Replace the interface with:

```ts
interface TabState {
  id: string;
  filePath: string;
  fileName: string;
  content: string;
  isDirty: boolean;
  scrollPosition: number;
  cursorPosition: number;
  version: number;
  /**
   * Deprecated since v0.2.4 — see spec 2026-05-07-v0.2.4-rename-and-macros.md.
   * Field retained so existing call sites and IPC mocks compile; no longer
   * read by `tryRenameAfterSave`. Slated for removal after v0.2.5 once
   * we confirm no external consumers depend on it.
   */
  justCreated: boolean;
  /**
   * Last H1 we successfully synchronized into the filename (or seeded from
   * disk content at open time). Drives the ongoing H1→filename sync in
   * `tryRenameAfterSave` — see spec 2026-05-12-h1-filename-ongoing-sync-design.md.
   *
   * Empty string means "the file currently has no H1, and that is also what
   * the filename reflects". `null` means "not initialized yet" (treated the
   * same as empty by the sync algorithm).
   */
  lastSyncedH1: string | null;
}
```

- [ ] **Step 2.2: Seed `lastSyncedH1` in `openTab`**

Edit `app/lib/stores/tabs.svelte.ts:231-251` (the `openTab` method). Replace its body so the new field is set:

```ts
openTab(filePath: string, content: string, options: OpenTabOptions = {}) {
  const pane = this.activePane;
  const existing = pane.tabs.find(t => t.filePath === filePath);
  if (existing) { pane.activeTabId = existing.id; return; }

  const rawName = pathBasename(filePath) || filePath;
  const fileName = isScratchFile(filePath) ? nextScratchDisplayName() : rawName;
  const id = crypto.randomUUID();
  pane.tabs.push({
    id,
    filePath,
    fileName,
    content,
    isDirty: false,
    scrollPosition: 0,
    cursorPosition: 0,
    version: 0,
    justCreated: options.justCreated === true,
    lastSyncedH1: extractFirstH1(content) ?? '',
  });
  pane.activeTabId = id;
}
```

Note: `extractFirstH1` is already imported at the top.

- [ ] **Step 2.3: Seed `lastSyncedH1` in `openTabInPane`**

Edit `app/lib/stores/tabs.svelte.ts:253-274` (the `openTabInPane` method). Replace its body:

```ts
openTabInPane(paneId: string, filePath: string, content: string, options: OpenTabOptions = {}) {
  const pane = this.panes.find(p => p.id === paneId);
  if (!pane) return;
  const existing = pane.tabs.find(t => t.filePath === filePath);
  if (existing) { pane.activeTabId = existing.id; return; }

  const fileName = pathBasename(filePath) || filePath;
  const id = crypto.randomUUID();
  pane.tabs.push({
    id,
    filePath,
    fileName,
    content,
    isDirty: false,
    scrollPosition: 0,
    cursorPosition: 0,
    version: 0,
    justCreated: options.justCreated === true,
    lastSyncedH1: extractFirstH1(content) ?? '',
  });
  pane.activeTabId = id;
}
```

- [ ] **Step 2.4: Refresh `lastSyncedH1` on watcher reload**

Edit `app/lib/stores/tabs.svelte.ts:444-456` (the `reloadContent` method). Replace its body:

```ts
reloadContent(id: string, newContent: string) {
  for (const pane of this.panes) {
    const tab = pane.tabs.find(t => t.id === id);
    if (tab) {
      tab.content = newContent;
      tab.isDirty = false;
      tab.version += 1;
      // External-edit policy: refresh anchor silently so the next save
      // doesn't see a stale `lastSyncedH1` and try to rename based on
      // someone else's edit. See spec §"Edge cases".
      tab.lastSyncedH1 = extractFirstH1(newContent) ?? '';
      // Clear saved editor state so the tab gets a fresh state with new content
      savedEditorStates.delete(id);
      return;
    }
  }
}
```

- [ ] **Step 2.5: Run existing tests to ensure no regression**

Run: `pnpm vitest run tests/unit/stores/`
Expected: all existing tabs tests still pass (the field is added with a sensible default; nothing reads it yet).

If a test fails because mocked `TabState`-shaped fixtures are missing `lastSyncedH1`: TypeScript's structural typing usually lets this slide, but if any test constructs a literal `TabState`, add `lastSyncedH1: ''`. Search with: `grep -rn "lastSyncedH1\|TabState" tests/`.

- [ ] **Step 2.6: Commit**

```bash
git add app/lib/stores/tabs.svelte.ts
git commit -m "feat(tabs): add lastSyncedH1 field, initialize at every content ingress"
```

---

## Task 3: Failing test for Path B in `tryRenameAfterSave`

**Files:**
- Test: `tests/unit/stores/tabs-ongoing-rename.test.ts` (create)

Write the integration-flavored tests before changing the algorithm. They will fail because today's `tryRenameAfterSave` early-returns on non-placeholder filenames.

- [ ] **Step 3.1: Create the failing test file**

Create `tests/unit/stores/tabs-ongoing-rename.test.ts`:

```ts
import { describe, it, expect, beforeEach, vi } from 'vitest';

/**
 * Path B coverage for `tabsStore.tryRenameAfterSave` — ongoing H1→filename
 * sync after a file has left the placeholder state. See spec
 * `docs/superpowers/specs/2026-05-12-h1-filename-ongoing-sync-design.md`.
 *
 * Path A (placeholder first-time rename) is covered by `tabs-auto-rename.test.ts`.
 */

vi.mock('$lib/ipc/commands', () => ({
  commands: {
    listDirectory: vi.fn(),
    renameItem: vi.fn(),
    broadcastFileRenamed: vi.fn(),
  },
}));

vi.mock('$lib/stores/new-file-settings.svelte', () => ({
  newFileSettings: { autoRenameFromH1: true },
}));

vi.mock('$lib/i18n', () => ({
  t: (k: string) => k,
}));

import { tabsStore } from '$lib/stores/tabs.svelte';
import { commands } from '$lib/ipc/commands';

const PROJECT = '/proj';

describe('tabsStore.tryRenameAfterSave — Path B (ongoing H1 sync)', () => {
  beforeEach(() => {
    localStorage.clear();
    tabsStore.closeAll();
    vi.clearAllMocks();
    (commands.listDirectory as any).mockResolvedValue({ status: 'ok', data: [] });
    (commands.renameItem as any).mockImplementation((_old: string, name: string) =>
      Promise.resolve({ status: 'ok', data: `${PROJECT}/${name}` }),
    );
    (commands.broadcastFileRenamed as any).mockResolvedValue({ status: 'ok', data: null });
  });

  it('renames when H1 changes on a non-placeholder file whose name still contains the old H1', async () => {
    // File was already renamed once via Path A: name reflects H1 "开篇".
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇\n\nbody');

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      '# 序幕\n\nbody',
    );

    expect(newPath).toBe(`${PROJECT}/第1章-序幕.md`);
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/第1章-开篇.md`,
      '第1章-序幕.md',
      true,
    );
  });

  it('updates lastSyncedH1 after a successful Path B rename so the next change has the right anchor', async () => {
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇\n\nbody');

    await tabsStore.tryRenameAfterSave(`${PROJECT}/第1章-开篇.md`, '# 序幕');
    (commands.renameItem as any).mockClear();

    // Second change "序幕" → "终章" should chain off the just-renamed file.
    const finalPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-序幕.md`,
      '# 终章',
    );
    expect(finalPath).toBe(`${PROJECT}/第1章-终章.md`);
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/第1章-序幕.md`,
      '第1章-终章.md',
      true,
    );
  });

  it('detaches sync when the user manually renamed the file (old H1 no longer in filename)', async () => {
    // Open a file whose name does NOT contain the H1 — simulates a manual rename
    // performed earlier (or a file opened from disk with mismatched H1).
    tabsStore.openTab(`${PROJECT}/chapter1.md`, '# 开篇\n\nbody');

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/chapter1.md`,
      '# 序幕',
    );

    expect(newPath).toBe(`${PROJECT}/chapter1.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('keeps the filename when the H1 is emptied (does not revert to Untitled)', async () => {
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇\n\nbody');

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      'just body, no heading anymore',
    );

    expect(newPath).toBe(`${PROJECT}/第1章-开篇.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('after clearing the H1 then retyping the SAME H1, no rename fires', async () => {
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇\n\nbody');

    await tabsStore.tryRenameAfterSave(`${PROJECT}/第1章-开篇.md`, 'no heading');
    (commands.renameItem as any).mockClear();

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      '# 开篇',
    );
    expect(newPath).toBe(`${PROJECT}/第1章-开篇.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('no-ops when H1 is unchanged (extra Cmd+S on a stable file)', async () => {
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇\n\nbody');

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      '# 开篇\n\nbody edited',
    );
    expect(newPath).toBe(`${PROJECT}/第1章-开篇.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('does not rename when auto_rename_from_h1 setting is off (gate honored)', async () => {
    // The default mock has it on; flip it for this case.
    const settingsMod = await import('$lib/stores/new-file-settings.svelte');
    (settingsMod.newFileSettings as any).autoRenameFromH1 = false;
    try {
      tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇');
      const newPath = await tabsStore.tryRenameAfterSave(
        `${PROJECT}/第1章-开篇.md`,
        '# 序幕',
      );
      expect(newPath).toBe(`${PROJECT}/第1章-开篇.md`);
      expect(commands.renameItem).not.toHaveBeenCalled();
    } finally {
      (settingsMod.newFileSettings as any).autoRenameFromH1 = true;
    }
  });

  it('bumps with " 2" when the new H1 collides with a sibling', async () => {
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇');
    (commands.listDirectory as any).mockResolvedValue({
      status: 'ok',
      data: [
        { name: '第1章-开篇.md' },
        { name: '第1章-序幕.md' }, // collision
      ],
    });

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      '# 序幕',
    );
    expect(newPath).toBe(`${PROJECT}/第1章-序幕 2.md`);
  });

  it('watcher reload silently refreshes lastSyncedH1 (no rename on external edit)', async () => {
    tabsStore.openTab(`${PROJECT}/第1章-开篇.md`, '# 开篇\n\nbody');
    const id = tabsStore.activeTabId!;

    // Simulate external edit landing via the file watcher.
    tabsStore.reloadContent(id, '# 终结\n\nbody from another editor');
    expect(commands.renameItem).not.toHaveBeenCalled();

    // A subsequent in-app save with the SAME H1 should also be a no-op.
    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      '# 终结',
    );
    expect(newPath).toBe(`${PROJECT}/第1章-开篇.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();

    // But if the user now changes the H1 again, sync wakes up using "终结"
    // as the anchor — which is NOT in `第1章-开篇.md`, so it stays detached.
    const next = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/第1章-开篇.md`,
      '# 别的标题',
    );
    expect(next).toBe(`${PROJECT}/第1章-开篇.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 3.2: Run, verify failures**

Run: `pnpm vitest run tests/unit/stores/tabs-ongoing-rename.test.ts`
Expected: FAIL — every Path B case fails because `tryRenameAfterSave` currently early-returns when `isPlaceholder()` is false.

- [ ] **Step 3.3: Commit (failing tests)**

```bash
git add tests/unit/stores/tabs-ongoing-rename.test.ts
git commit -m "test(tabs): failing Path B tests for ongoing H1 sync"
```

---

## Task 4: Implement Path B in `tryRenameAfterSave`

**Files:**
- Modify: `app/lib/stores/tabs.svelte.ts:189-229`

- [ ] **Step 4.1: Update imports**

Edit `app/lib/stores/tabs.svelte.ts:5`. Add `applyH1Substitution` to the placeholder import:

```ts
import { isPlaceholder, renameFromH1, applyH1Substitution } from '$lib/utils/placeholder';
```

- [ ] **Step 4.2: Replace the body of `tryRenameAfterSave`**

Edit `app/lib/stores/tabs.svelte.ts:189-229`. Replace the entire method (the docblock too) with:

```ts
/**
 * Post-write hook: keep the filename in sync with the file's H1 heading.
 * Returns the new path (== old path if no rename). Safe to call after every
 * successful writeFile — both manual Cmd+S and auto-save funnel here.
 *
 * Two paths, gated by `auto_rename_from_h1`:
 *
 *  Path A — placeholder first-time rename (v0.2.4 behavior, unchanged):
 *    File still matches `isPlaceholder()` and has a non-empty H1.
 *    Uses `renameFromH1`.
 *
 *  Path B — ongoing sync (v0.2.5+):
 *    File is past the placeholder stage; we compare the just-saved H1
 *    against the per-tab `lastSyncedH1`. If changed and the old H1 still
 *    appears in the current filename, swap that substring for the new H1.
 *    Manually renamed files have no anchor to find, so sync auto-detaches.
 *
 * See spec `docs/superpowers/specs/2026-05-12-h1-filename-ongoing-sync-design.md`.
 */
async tryRenameAfterSave(filePath: string, content: string): Promise<string> {
  if (!newFileSettings.autoRenameFromH1) return filePath;
  if (isScratchFile(filePath)) return filePath;

  // Find the tab so we can read/update `lastSyncedH1`. Save-from-another-pane
  // is possible; we update every matching tab via `updatePath` after rename.
  const tab = this.findByPath(filePath);
  const oldH1 = tab?.lastSyncedH1 ?? '';
  const newH1Raw = extractFirstH1(content);
  const newH1 = newH1Raw ?? '';

  if (newH1 === oldH1) return filePath; // no change

  const fileName = pathBasename(filePath) || filePath;
  const parentDir = pathDirname(filePath) || filePath;

  // -------- Path A: placeholder → titled (existing behavior) --------
  if (isPlaceholder(fileName)) {
    if (newH1.trim().length === 0) {
      // No H1 yet; nothing to do. Don't update anchor either — let next
      // save with a real H1 fall through here again.
      return filePath;
    }
    const list = await commands.listDirectory(parentDir, null);
    const siblings = list.status === 'ok' ? list.data.map(e => e.name) : [];
    const newName = renameFromH1(fileName, newH1, siblings);
    if (!newName || newName === fileName) {
      // Couldn't compute a rename, but we DID observe an H1; record it so
      // Path B can pick up future changes.
      this.setLastSyncedH1ByPath(filePath, newH1);
      return filePath;
    }
    const result = await commands.renameItem(filePath, newName, true);
    if (result.status !== 'ok') {
      console.warn('Auto-rename failed:', result.error);
      return filePath;
    }
    const newPath = result.data;
    this.updatePath(filePath, newPath);
    this.setLastSyncedH1ByPath(newPath, newH1);
    await commands.broadcastFileRenamed(filePath, newPath).catch(() => {});
    return newPath;
  }

  // -------- Path B: ongoing sync --------
  // If we have no anchor (e.g. tab opened before this feature, or file
  // opened from disk with no H1), adopt the current H1 silently so the
  // next *change* has something to compare against.
  if (oldH1.length === 0) {
    this.setLastSyncedH1ByPath(filePath, newH1);
    return filePath;
  }
  // User emptied the H1 — keep filename, do NOT update anchor (so retyping
  // the same H1 short-circuits via `newH1 === oldH1`).
  if (newH1.trim().length === 0) {
    return filePath;
  }

  const list = await commands.listDirectory(parentDir, null);
  const siblings = list.status === 'ok' ? list.data.map(e => e.name) : [];
  const newName = applyH1Substitution(fileName, oldH1, newH1, siblings);
  if (!newName) {
    // Manual-rename detach OR sanitize-equal. Update anchor so we don't
    // keep retrying the same comparison every save.
    this.setLastSyncedH1ByPath(filePath, newH1);
    return filePath;
  }

  const result = await commands.renameItem(filePath, newName, true);
  if (result.status !== 'ok') {
    console.warn('Auto-rename failed:', result.error);
    return filePath;
  }
  const newPath = result.data;
  this.updatePath(filePath, newPath);
  this.setLastSyncedH1ByPath(newPath, newH1);
  await commands.broadcastFileRenamed(filePath, newPath).catch(() => {});
  return newPath;
}

/** Internal helper: update `lastSyncedH1` on every tab matching `filePath`. */
private setLastSyncedH1ByPath(filePath: string, h1: string) {
  for (const pane of this.panes) {
    for (const tab of pane.tabs) {
      if (tab.filePath === filePath) tab.lastSyncedH1 = h1;
    }
  }
}
```

Note: `findByPath` already exists at line ~459. `pathDirname` is already imported. The new helper `setLastSyncedH1ByPath` is private and only used within this method.

- [ ] **Step 4.3: Run the Path B tests, verify they pass**

Run: `pnpm vitest run tests/unit/stores/tabs-ongoing-rename.test.ts`
Expected: PASS — all 9 cases green.

- [ ] **Step 4.4: Run the existing Path A tests, verify they still pass**

Run: `pnpm vitest run tests/unit/stores/tabs-auto-rename.test.ts`
Expected: PASS — all 6 cases green. (The "does not double-rename" case still passes because after the first Path A rename, the file is no longer a placeholder, AND `lastSyncedH1` now equals the H1 — so a subsequent save with a *different* H1 would actually trigger Path B. The existing test uses "Different Title" which is NOT a substring of "Title A.md", so Path B detaches.)

- [ ] **Step 4.5: Run all unit tests + svelte-check**

Run: `pnpm test && pnpm check`
Expected: both pass cleanly.

- [ ] **Step 4.6: Commit**

```bash
git add app/lib/stores/tabs.svelte.ts
git commit -m "feat(tabs): ongoing H1→filename sync via lastSyncedH1 anchor"
```

---

## Task 5: Update i18n hint copy

**Files:**
- Modify: `app/lib/i18n/locales/en.ts:430`
- Modify: `app/lib/i18n/locales/zh-CN.ts:430`

- [ ] **Step 5.1: Update English hint**

Edit `app/lib/i18n/locales/en.ts:430`:

```ts
  'settings.editor.newFile.autoRenameHint': 'Filename follows the H1 heading on each save. Manually renaming the file detaches the sync.',
```

Also update the label on line 429 from "Auto-rename placeholder files from H1" to "Sync filename with H1 on save":

```ts
  'settings.editor.newFile.autoRename': 'Sync filename with H1 on save',
```

- [ ] **Step 5.2: Update Chinese hint**

Edit `app/lib/i18n/locales/zh-CN.ts:430`:

```ts
  'settings.editor.newFile.autoRenameHint': '保存时让文件名跟随 H1 标题。手动重命名文件即可脱离同步。',
```

And the label on line 429:

```ts
  'settings.editor.newFile.autoRename': '保存时让文件名跟随 H1 标题',
```

- [ ] **Step 5.3: Run svelte-check to catch any key mismatches**

Run: `pnpm check`
Expected: PASS (the keys are unchanged; only values shifted).

- [ ] **Step 5.4: Commit**

```bash
git add app/lib/i18n/locales/en.ts app/lib/i18n/locales/zh-CN.ts
git commit -m "i18n(settings): rewrite autoRename label+hint for ongoing sync"
```

---

## Task 6: Cross-link old spec + CHANGELOG entry

**Files:**
- Modify: `docs/exec-plans/completed/2026-05-07-v0.2.4-rename-and-macros.md` (top of file)
- Modify: `CHANGELOG.md`

- [ ] **Step 6.1: Add breadcrumb to the v0.2.4 spec**

Open `docs/exec-plans/completed/2026-05-07-v0.2.4-rename-and-macros.md`. Find the title (`# ...`) on the first line. Immediately after the title, insert:

```markdown
> **Updated 2026-05-12:** The "first-rename-then-stop" behavior described in §3.5 has been superseded by ongoing H1→filename sync. See `docs/superpowers/specs/2026-05-12-h1-filename-ongoing-sync-design.md`. The Path A logic in this spec is preserved as the placeholder→titled transition; Path B handles subsequent edits.
```

If the file does not have an existing top-level title (`# `), insert the breadcrumb as a fresh top block before the first existing content.

- [ ] **Step 6.2: Add CHANGELOG entry**

Open `CHANGELOG.md`. Find the topmost unreleased / next version section (look for `## [Unreleased]` or the highest version header). Under its "Changed" or "Features" subsection (create one if needed), add:

```markdown
- Filename now follows the H1 heading on every save (was first-save only). Manually renaming a file detaches the sync.
```

If unsure which section, run `git log --oneline -20 CHANGELOG.md` to see how prior entries were structured.

- [ ] **Step 6.3: Commit**

```bash
git add CHANGELOG.md docs/exec-plans/completed/2026-05-07-v0.2.4-rename-and-macros.md
git commit -m "docs: cross-link ongoing H1 sync spec + CHANGELOG entry"
```

---

## Task 7: Pre-push verification (mirror CI)

**Files:** none modified

- [ ] **Step 7.1: Run cargo fmt --check**

Run: `cargo fmt --check --manifest-path core/Cargo.toml`
Expected: clean exit code. (Not strictly necessary for this frontend-only change but matches user's pre-push CI mirror rule.)

- [ ] **Step 7.2: Run clippy**

Run: `cargo clippy --manifest-path core/Cargo.toml -- -D warnings`
Expected: no warnings/errors. (Frontend-only change; this is a sanity check that we didn't touch Rust by accident.)

- [ ] **Step 7.3: Run svelte-check**

Run: `pnpm check`
Expected: 0 errors.

- [ ] **Step 7.4: Run full unit suite**

Run: `pnpm test`
Expected: all tests pass, including the v0.2.4 placeholder regression tests and the new Path B tests.

- [ ] **Step 7.5: Run Rust tests**

Run: `pnpm test:rust`
Expected: PASS (untouched by this change but a sanity check).

- [ ] **Step 7.6: Manual smoke test in `pnpm tauri dev`**

Boot the app and verify:
1. New file via Cmd+N in a project with template `第{N}章-{title}` → creates `第1章-Untitled.md`
2. Type `# 开篇` and save → filename becomes `第1章-开篇.md` (Path A)
3. Change H1 to `# 序幕` and save → filename becomes `第1章-序幕.md` (Path B)
4. Manually rename file in sidebar to `chapter1.md`
5. Change H1 to `# 终结` and save → filename remains `chapter1.md` (detached)
6. Delete the `#` heading entirely and save → filename remains `chapter1.md` (empty H1 is safe)

If any of these diverge from the spec, file an issue and pause before merging.

---

## Self-Review

**1. Spec coverage:**
- §State (lastSyncedH1 + init points + reload refresh): Task 2 ✓
- §Algorithm Path A (placeholder): preserved in Task 4 ✓
- §Algorithm Path B (structural match + lastIndexOf + detach + bump): Tasks 1 + 4 ✓
- §Properties 1–6: covered by Path B tests in Task 3 ✓
- §Edge cases (collision / empty-H1 / scratch / external edit): covered in Tasks 1 & 3 ✓
- §I18N copy update: Task 5 ✓
- §Testing: Tasks 1 & 3 ✓
- §Changelog & spec breadcrumb: Task 6 ✓

**2. Placeholder scan:** No "TBD" / "TODO" / "implement later" markers. Every code block contains the full content to paste.

**3. Type consistency:** `lastSyncedH1: string | null` used uniformly (interface, openTab, openTabInPane, reloadContent, setLastSyncedH1ByPath, Path A/B writes). `applyH1Substitution(currentName, oldH1, newH1, siblings) → string | null` signature matches between test cases (Task 1.1) and implementation (Task 1.3) and call site (Task 4.2). The `findByPath` method referenced in Task 4 exists at `app/lib/stores/tabs.svelte.ts:459` (already verified).

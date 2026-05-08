import { describe, it, expect, beforeEach, vi } from 'vitest';

/**
 * Regression coverage for `tabsStore.tryRenameAfterSave`.
 *
 * Gating (v0.2.4+):
 *  - `auto_rename_from_h1` setting must be on
 *  - Filename must match `isPlaceholder()` (Untitled N / 第N章 / Chapter N / …)
 *  - The save content must contain a non-empty H1
 *
 * Notably, the legacy `tab.justCreated` gate was removed in v0.2.4 — users
 * who reopen a placeholder file in a later session and finally type an H1
 * should still get the auto-rename. See spec
 * `docs/product-specs/2026-05-07-v0.2.4-rename-and-macros.md`.
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

describe('tabsStore.tryRenameAfterSave — placeholder + H1 gating', () => {
  beforeEach(() => {
    tabsStore.closeAll();
    vi.clearAllMocks();
    (commands.listDirectory as any).mockResolvedValue({ status: 'ok', data: [] });
    (commands.renameItem as any).mockImplementation((_old: string, name: string) =>
      Promise.resolve({ status: 'ok', data: `${PROJECT}/${name}` }),
    );
    (commands.broadcastFileRenamed as any).mockResolvedValue({ status: 'ok', data: null });
  });

  it('renames a placeholder-named tab that was just created (Cmd+N flow)', async () => {
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '', { justCreated: true });
    const body = '# 开篇\n\nsome body';

    const newPath = await tabsStore.tryRenameAfterSave(`${PROJECT}/Untitled 1.md`, body);

    expect(newPath).toBe(`${PROJECT}/开篇.md`);
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/Untitled 1.md`,
      '开篇.md',
      true,
    );
  });

  it('renames a placeholder-named tab opened from disk (no justCreated flag)', async () => {
    // User created `第1章.md` manually (e.g. via Finder) or in a previous
    // session. Opening it from the sidebar yields justCreated=false. Once
    // the user finally types an H1 and saves, we still want the rename.
    tabsStore.openTab(`${PROJECT}/第1章.md`, 'existing body');
    const body = '# 开篇\n\nmore body';

    const newPath = await tabsStore.tryRenameAfterSave(`${PROJECT}/第1章.md`, body);

    expect(newPath).toBe(`${PROJECT}/第1章 开篇.md`);
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/第1章.md`,
      '第1章 开篇.md',
      true,
    );
  });

  it('does not double-rename: once the file is no longer a placeholder, future saves are no-ops', async () => {
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '', { justCreated: true });
    const id = tabsStore.activeTabId!;

    await tabsStore.tryRenameAfterSave(`${PROJECT}/Untitled 1.md`, '# Title A');

    const renamedTab = tabsStore.tabs.find(t => t.id === id);
    expect(renamedTab?.filePath).toBe(`${PROJECT}/Title A.md`);

    // A subsequent save must NOT auto-rename again — `Title A.md` no longer
    // matches `isPlaceholder()`, so the gate naturally closes.
    (commands.renameItem as any).mockClear();
    await tabsStore.tryRenameAfterSave(`${PROJECT}/Title A.md`, '# Different Title');
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('no-ops when the save contains no H1 yet (rename deferred until the user titles it)', async () => {
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '', { justCreated: true });

    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/Untitled 1.md`,
      'body with no heading',
    );
    expect(newPath).toBe(`${PROJECT}/Untitled 1.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();

    // Later save WITH an H1 still triggers the rename.
    await tabsStore.tryRenameAfterSave(`${PROJECT}/Untitled 1.md`, '# 终于有标题了');
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/Untitled 1.md`,
      '终于有标题了.md',
      true,
    );
  });

  it('does nothing for non-placeholder filenames regardless of H1 content', async () => {
    tabsStore.openTab(`${PROJECT}/my-novel.md`, '');
    const newPath = await tabsStore.tryRenameAfterSave(`${PROJECT}/my-novel.md`, '# A Heading');
    expect(newPath).toBe(`${PROJECT}/my-novel.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('runs even when the tab is clean — Editor.saveCurrentFile relies on this for Cmd+S on clean tabs', async () => {
    // Open a placeholder tab and immediately mark it saved (no writeFile). This
    // simulates a file that's been autosaved already, where the user then
    // presses Cmd+S to confirm — the filename should still update.
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '# 开篇');
    const id = tabsStore.activeTabId!;
    tabsStore.markSaved(id);
    const tab = tabsStore.tabs.find(t => t.id === id);
    expect(tab?.isDirty).toBe(false);

    const newPath = await tabsStore.tryRenameAfterSave(`${PROJECT}/Untitled 1.md`, '# 开篇');
    expect(newPath).toBe(`${PROJECT}/开篇.md`);
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/Untitled 1.md`,
      '开篇.md',
      true,
    );
  });
});

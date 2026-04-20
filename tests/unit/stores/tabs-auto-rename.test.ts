import { describe, it, expect, beforeEach, vi } from 'vitest';

/**
 * Regression coverage for `tabsStore.tryRenameAfterSave`.
 *
 * The H1-driven auto-rename pipeline must ONLY fire for tabs that were
 * created this session via "New File" / scratch / sidebar new-file. Opening
 * an existing file whose name happens to match a placeholder regex (e.g.
 * `第1章.md` the user created manually in Finder) must NOT be renamed on
 * Cmd+S — the user explicitly chose that name.
 *
 * The `justCreated` flag on TabState gates this. These tests pin both
 * branches so we don't regress to the old "rename every placeholder-named
 * file" behavior.
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

describe('tabsStore.tryRenameAfterSave — justCreated gating', () => {
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

  it('does NOT rename an existing file whose name matches the placeholder regex', async () => {
    // User created `第1章.md` manually (e.g. via Finder). Opening it via the
    // sidebar produces a tab with justCreated=false — the default.
    tabsStore.openTab(`${PROJECT}/第1章.md`, 'existing body');
    const body = '# 开篇\n\nmore body';

    const newPath = await tabsStore.tryRenameAfterSave(`${PROJECT}/第1章.md`, body);

    expect(newPath).toBe(`${PROJECT}/第1章.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();
    expect(commands.listDirectory).not.toHaveBeenCalled();
  });

  it('clears justCreated after a successful rename (one-shot)', async () => {
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '', { justCreated: true });
    const id = tabsStore.activeTabId!;
    const firstBody = '# Title A';

    await tabsStore.tryRenameAfterSave(`${PROJECT}/Untitled 1.md`, firstBody);

    const renamedTab = tabsStore.tabs.find(t => t.id === id);
    expect(renamedTab?.filePath).toBe(`${PROJECT}/Title A.md`);
    expect(renamedTab?.justCreated).toBe(false);

    // A subsequent save must NOT auto-rename again, even if the file happens
    // to still match a placeholder pattern (it doesn't here, but the gate is
    // on the flag regardless).
    (commands.renameItem as any).mockClear();
    await tabsStore.tryRenameAfterSave(`${PROJECT}/Title A.md`, '# Different Title');
    expect(commands.renameItem).not.toHaveBeenCalled();
  });

  it('keeps justCreated set when the save contains no H1 (rename deferred)', async () => {
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '', { justCreated: true });
    const id = tabsStore.activeTabId!;

    // First save: no H1 yet. tryRenameAfterSave should no-op *without*
    // burning the one-shot — the user hasn't had a chance to title the file.
    const newPath = await tabsStore.tryRenameAfterSave(
      `${PROJECT}/Untitled 1.md`,
      'body with no heading',
    );
    expect(newPath).toBe(`${PROJECT}/Untitled 1.md`);
    expect(commands.renameItem).not.toHaveBeenCalled();

    const tab = tabsStore.tabs.find(t => t.id === id);
    expect(tab?.justCreated).toBe(true);

    // Later save WITH an H1 still triggers the rename.
    await tabsStore.tryRenameAfterSave(`${PROJECT}/Untitled 1.md`, '# 终于有标题了');
    expect(commands.renameItem).toHaveBeenCalledWith(
      `${PROJECT}/Untitled 1.md`,
      '终于有标题了.md',
      true,
    );
  });

  it('defaults justCreated to false when openTab is called without options', () => {
    tabsStore.openTab(`${PROJECT}/Untitled 1.md`, '');
    const tab = tabsStore.tabs[0];
    expect(tab.justCreated).toBe(false);
  });
});

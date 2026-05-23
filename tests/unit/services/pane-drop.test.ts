import { beforeEach, describe, expect, it, vi } from 'vitest';

const { h } = vi.hoisted(() => {
  const tabsState = {
    splitActive: false,
    openTabInPane: vi.fn(),
    setActivePane: vi.fn(),
    toggleSplit: vi.fn(() => {
      tabsState.splitActive = !tabsState.splitActive;
    }),
  };
  const commandsState = {
    readFile: vi.fn(),
    registerOpenFile: vi.fn(),
  };
  return { h: { tabsState, commandsState } };
});

vi.mock('$lib/ipc/commands', () => ({
  commands: h.commandsState,
}));

vi.mock('$lib/stores/tabs.svelte', () => ({
  tabsStore: {
    get splitActive() { return h.tabsState.splitActive; },
    openTabInPane: h.tabsState.openTabInPane,
    setActivePane: h.tabsState.setActivePane,
    toggleSplit: h.tabsState.toggleSplit,
  },
}));

import { commands } from '$lib/ipc/commands';
import { tabsStore } from '$lib/stores/tabs.svelte';
import {
  hasSidebarPath,
  isOpenablePath,
  openPathInPane,
  openPathSplitRight,
  SIDEBAR_PATH_MIME,
} from '$lib/services/pane-drop';

describe('[contract] pane-drop service', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    h.tabsState.splitActive = false;
    h.commandsState.readFile.mockResolvedValue({ status: 'ok', data: '# One' });
    h.commandsState.registerOpenFile.mockResolvedValue(undefined);
  });

  it('recognizes supported file extensions case-insensitively', () => {
    expect(isOpenablePath('/novel/Chapter.MD')).toBe(true);
    expect(isOpenablePath('/board/plot.CANVAS')).toBe(true);
    expect(isOpenablePath('/data/events.jsonl')).toBe(true);
    expect(isOpenablePath('/image/cover.png')).toBe(false);
  });

  it('detects sidebar drag payloads in type lists', () => {
    expect(hasSidebarPath(undefined)).toBe(false);
    expect(hasSidebarPath(['text/plain'])).toBe(false);
    expect(hasSidebarPath(['text/plain', SIDEBAR_PATH_MIME.toUpperCase()])).toBe(true);
  });

  it('opens a readable file in the requested pane', async () => {
    await expect(openPathInPane('pane-1', '/novel/ch01.md')).resolves.toBe(true);

    expect(commands.readFile).toHaveBeenCalledWith('/novel/ch01.md');
    expect(tabsStore.openTabInPane).toHaveBeenCalledWith('pane-1', '/novel/ch01.md', '# One');
    expect(tabsStore.setActivePane).toHaveBeenCalledWith('pane-1');
    expect(commands.registerOpenFile).toHaveBeenCalledWith('/novel/ch01.md');
  });

  it('refuses unsupported paths without touching IPC', async () => {
    await expect(openPathInPane('pane-1', '/novel/cover.png')).resolves.toBe(false);

    expect(commands.readFile).not.toHaveBeenCalled();
    expect(tabsStore.openTabInPane).not.toHaveBeenCalled();
  });

  it('returns false when the file read fails', async () => {
    h.commandsState.readFile.mockResolvedValueOnce({ status: 'error', error: 'missing' });

    await expect(openPathInPane('pane-1', '/novel/missing.md')).resolves.toBe(false);
    expect(tabsStore.openTabInPane).not.toHaveBeenCalled();
  });

  it('ignores register-open-file failures after opening the pane', async () => {
    h.commandsState.registerOpenFile.mockRejectedValueOnce(new Error('window registry unavailable'));

    await expect(openPathInPane('pane-1', '/novel/ch01.md')).resolves.toBe(true);
    expect(tabsStore.openTabInPane).toHaveBeenCalled();
  });

  it('enables split mode before opening a sidebar file to the right pane', async () => {
    await expect(openPathSplitRight('/novel/ch02.md')).resolves.toBe(true);

    expect(tabsStore.toggleSplit).toHaveBeenCalledTimes(1);
    expect(tabsStore.openTabInPane).toHaveBeenCalledWith('pane-2', '/novel/ch02.md', '# One');
  });

  it('refuses unsupported split-right drops before toggling split mode', async () => {
    await expect(openPathSplitRight('/novel/cover.png')).resolves.toBe(false);

    expect(tabsStore.toggleSplit).not.toHaveBeenCalled();
    expect(commands.readFile).not.toHaveBeenCalled();
  });

  it('reuses an existing split without toggling it off', async () => {
    h.tabsState.splitActive = true;

    await expect(openPathSplitRight('/novel/ch03.md')).resolves.toBe(true);
    expect(tabsStore.toggleSplit).not.toHaveBeenCalled();
    expect(tabsStore.openTabInPane).toHaveBeenCalledWith('pane-2', '/novel/ch03.md', '# One');
  });
});

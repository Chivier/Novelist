import { describe, it, expect, vi, beforeEach } from 'vitest';

/**
 * [contract] services/file-route — claim rule for the cross-window single-
 * file open router. See docs/superpowers/specs/2026-05-19-single-file-open-
 * routing-design.md.
 *
 * Rules (per window):
 *   - Window has a project P → claim iff filePath lives under P.
 *   - Single-file mode with ≥ 1 tab → claim iff filePath shares parent dir
 *     with at least one existing tab.
 *   - Empty window → always claim (fallback).
 */

const { h } = vi.hoisted(() => {
  const projectState = { dirPath: null as string | null };
  const tabsState = { panes: [] as Array<{ tabs: Array<{ filePath: string }> }> };
  return { h: { projectState, tabsState } };
});

vi.mock('$lib/stores/project.svelte', () => ({
  projectStore: {
    get dirPath() { return h.projectState.dirPath; },
  },
}));

vi.mock('$lib/stores/tabs.svelte', () => ({
  tabsStore: {
    get panes() { return h.tabsState.panes; },
  },
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ label: 'main' }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('$lib/ipc/commands', () => ({
  commands: {
    submitFileOpenBid: vi.fn(),
    routeSingleFileOpenCmd: vi.fn(),
  },
}));

vi.mock('$lib/services/cli-open', () => ({
  openFileInNewWindow: vi.fn(),
}));

import { computeCanClaim } from '$lib/services/file-route';

beforeEach(() => {
  h.projectState.dirPath = null;
  h.tabsState.panes = [];
});

describe('computeCanClaim — project-window rule', () => {
  it('claims a file inside the project root', () => {
    h.projectState.dirPath = '/work/novel';
    expect(computeCanClaim('/work/novel/ch01.md')).toEqual({
      canClaim: true,
      hasProject: true,
    });
  });

  it('refuses a file outside the project root', () => {
    h.projectState.dirPath = '/work/novel';
    expect(computeCanClaim('/tmp/scratch.md')).toEqual({
      canClaim: false,
      hasProject: true,
    });
  });

  it('refuses the project root itself (no trailing slash means not inside)', () => {
    h.projectState.dirPath = '/work/novel';
    expect(computeCanClaim('/work/novel')).toEqual({
      canClaim: false,
      hasProject: true,
    });
  });
});

describe('computeCanClaim — single-file rule', () => {
  it('claims a file that shares parent dir with an existing tab', () => {
    h.tabsState.panes = [{ tabs: [{ filePath: '/tmp/a.md' }] }];
    expect(computeCanClaim('/tmp/b.md')).toEqual({
      canClaim: true,
      hasProject: false,
    });
  });

  it('refuses a file with a different parent dir', () => {
    h.tabsState.panes = [{ tabs: [{ filePath: '/tmp/a.md' }] }];
    expect(computeCanClaim('/elsewhere/b.md')).toEqual({
      canClaim: false,
      hasProject: false,
    });
  });

  it('looks at all panes, not just the active one', () => {
    h.tabsState.panes = [
      { tabs: [{ filePath: '/tmp/a.md' }] },
      { tabs: [{ filePath: '/other/c.md' }] },
    ];
    expect(computeCanClaim('/other/d.md').canClaim).toBe(true);
  });
});

describe('computeCanClaim — empty-window fallback', () => {
  it('claims any file when no project and no tabs', () => {
    expect(computeCanClaim('/any/where.md')).toEqual({
      canClaim: true,
      hasProject: false,
    });
  });
});

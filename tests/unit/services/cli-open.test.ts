import { describe, it, expect, vi, beforeEach } from 'vitest';

/**
 * [contract] services/cli-open — routing for the `cli-open` event from
 * tauri-plugin-single-instance.
 *
 * Rules under test (mirrors `cli-and-windows.md`):
 *   1. Folders always spawn a new WebviewWindow with `#project=…` hash.
 *   2. Files reuse the current window IFF projectStore.isOpen===false AND
 *      `force_new_window===false`. Otherwise spawn a new window with
 *      `#file=…&line=…&col=…`.
 *   3. consumeWindowSeed parses the URL hash and dispatches at most one
 *      project + one file, then clears the hash.
 */

const { h } = vi.hoisted(() => {
  const projectState = {
    isOpen: false,
  };
  const ctorCalls: Array<{ label: string; opts: Record<string, unknown> }> = [];
  class FakeWebviewWindow {
    constructor(label: string, opts: Record<string, unknown>) {
      ctorCalls.push({ label, opts });
    }
    once(_evt: string, _cb: (e: unknown) => void) { /* no-op */ }
  }
  return { h: { projectState, ctorCalls, FakeWebviewWindow } };
});

vi.mock('$lib/stores/project.svelte', () => ({
  projectStore: {
    get isOpen() { return h.projectState.isOpen; },
  },
}));

vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: h.FakeWebviewWindow,
}));

beforeEach(() => {
  h.ctorCalls.length = 0;
  h.projectState.isOpen = false;
});

import { handleCliOpen, consumeWindowSeed, openProjectInThisWindow, openFileInNewWindow } from '$lib/services/cli-open';
import type { CliOpenPayload } from '$lib/ipc/commands';

function payload(overrides: Partial<CliOpenPayload> = {}): CliOpenPayload {
  return {
    files: overrides.files ?? [],
    folders: overrides.folders ?? [],
    force_new_window: overrides.force_new_window ?? false,
  };
}

function lastWindowHash(): string | null {
  const last = h.ctorCalls[h.ctorCalls.length - 1];
  if (!last) return null;
  const url = String(last.opts.url ?? '');
  const i = url.indexOf('#');
  return i < 0 ? null : url.slice(i + 1);
}

describe('handleCliOpen — folder routing', () => {
  it('spawns one window per folder, encoding the project path in the URL hash', async () => {
    const opener = vi.fn(async () => true);
    await handleCliOpen(
      payload({ folders: ['/work/novel-a', '/work/novel-b'] }),
      opener,
    );
    expect(h.ctorCalls).toHaveLength(2);
    const hashes = h.ctorCalls.map(c => {
      const url = String(c.opts.url);
      return url.slice(url.indexOf('#') + 1);
    });
    expect(hashes[0]).toContain('project=%2Fwork%2Fnovel-a');
    expect(hashes[1]).toContain('project=%2Fwork%2Fnovel-b');
    // Folders never go through the in-window opener.
    expect(opener).not.toHaveBeenCalled();
  });

  it('spawns folder windows even when no project is currently open', async () => {
    h.projectState.isOpen = false;
    await handleCliOpen(payload({ folders: ['/p'] }), vi.fn());
    expect(h.ctorCalls).toHaveLength(1);
  });

  it('spawns folder windows even with force_new_window=false (folders ignore the flag)', async () => {
    await handleCliOpen(
      payload({ folders: ['/p'], force_new_window: false }),
      vi.fn(),
    );
    expect(h.ctorCalls).toHaveLength(1);
  });
});

describe('handleCliOpen — file routing', () => {
  it('reuses this window for a file when no project is open and -n was not passed', async () => {
    h.projectState.isOpen = false;
    const opener = vi.fn(async () => true);
    await handleCliOpen(
      payload({ files: [{ path: '/x/note.md', line: null, col: null }] }),
      opener,
    );
    expect(opener).toHaveBeenCalledWith('/x/note.md', null);
    expect(h.ctorCalls).toHaveLength(0);
  });

  it('spawns a new window for a file when a project IS open (any file would pollute project view)', async () => {
    h.projectState.isOpen = true;
    const opener = vi.fn(async () => true);
    await handleCliOpen(
      payload({ files: [{ path: '/x/note.md', line: 12, col: 3 }] }),
      opener,
    );
    expect(opener).not.toHaveBeenCalled();
    expect(h.ctorCalls).toHaveLength(1);
    const hash = lastWindowHash() ?? '';
    expect(hash).toContain('file=%2Fx%2Fnote.md');
    expect(hash).toContain('line=12');
    expect(hash).toContain('col=3');
  });

  it('spawns a new window for a file when force_new_window=true, even if no project is open', async () => {
    h.projectState.isOpen = false;
    const opener = vi.fn(async () => true);
    await handleCliOpen(
      payload({
        files: [{ path: '/x/note.md', line: null, col: null }],
        force_new_window: true,
      }),
      opener,
    );
    expect(opener).not.toHaveBeenCalled();
    expect(h.ctorCalls).toHaveLength(1);
  });

  it('falls back to a new window if the in-window opener returns false', async () => {
    h.projectState.isOpen = false;
    const opener = vi.fn(async () => false); // e.g. file unreadable
    await handleCliOpen(
      payload({ files: [{ path: '/x/note.md', line: null, col: null }] }),
      opener,
    );
    expect(opener).toHaveBeenCalled();
    expect(h.ctorCalls).toHaveLength(1);
  });

  it('routes multiple files independently in order', async () => {
    h.projectState.isOpen = false;
    const opener = vi.fn(async () => true);
    await handleCliOpen(
      payload({
        files: [
          { path: '/a.md', line: null, col: null },
          { path: '/b.md', line: 5, col: null },
        ],
      }),
      opener,
    );
    expect(opener).toHaveBeenCalledTimes(2);
    expect(opener).toHaveBeenNthCalledWith(1, '/a.md', null);
    expect(opener).toHaveBeenNthCalledWith(2, '/b.md', 5);
    expect(h.ctorCalls).toHaveLength(0);
  });

  it('combined: folder + file with project open → folder window + file window', async () => {
    h.projectState.isOpen = true;
    const opener = vi.fn(async () => true);
    await handleCliOpen(
      payload({
        folders: ['/proj'],
        files: [{ path: '/x/note.md', line: null, col: null }],
      }),
      opener,
    );
    expect(opener).not.toHaveBeenCalled();
    expect(h.ctorCalls).toHaveLength(2);
  });
});

describe('openProjectInThisWindow', () => {
  it('spawns a new window seeded with the project path when spawnNew=true (default)', async () => {
    await openProjectInThisWindow('/the/proj');
    expect(h.ctorCalls).toHaveLength(1);
    const hash = lastWindowHash() ?? '';
    expect(hash).toContain('project=%2Fthe%2Fproj');
  });

  it('logs a warning and is a no-op when spawnNew=false (caller should bypass)', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    await openProjectInThisWindow('/x', false);
    expect(h.ctorCalls).toHaveLength(0);
    expect(warn).toHaveBeenCalled();
    warn.mockRestore();
  });
});

describe('openFileInNewWindow', () => {
  it('encodes file + line + col into the hash', async () => {
    await openFileInNewWindow('/x/n.md', 7, 2);
    const hash = lastWindowHash() ?? '';
    expect(hash).toContain('file=%2Fx%2Fn.md');
    expect(hash).toContain('line=7');
    expect(hash).toContain('col=2');
  });

  it('omits line/col when null', async () => {
    await openFileInNewWindow('/x/n.md', null, null);
    const hash = lastWindowHash() ?? '';
    expect(hash).toContain('file=');
    expect(hash).not.toContain('line=');
    expect(hash).not.toContain('col=');
  });
});

describe('consumeWindowSeed — URL hash parsing', () => {
  function withHash(hash: string, fn: () => Promise<void>): Promise<void> {
    const original = window.location.hash;
    Object.defineProperty(window.location, 'hash', { value: hash, writable: true, configurable: true });
    const replaceSpy = vi.spyOn(history, 'replaceState').mockImplementation(() => {});
    return fn().finally(() => {
      Object.defineProperty(window.location, 'hash', { value: original, writable: true, configurable: true });
      replaceSpy.mockRestore();
    });
  }

  it('does nothing when hash is empty', async () => {
    await withHash('', async () => {
      const openProject = vi.fn();
      const openFile = vi.fn();
      await consumeWindowSeed({ openProject, openFile });
      expect(openProject).not.toHaveBeenCalled();
      expect(openFile).not.toHaveBeenCalled();
    });
  });

  it('opens project when #project= is present', async () => {
    await withHash('#project=%2Fwork%2Fp', async () => {
      const openProject = vi.fn();
      const openFile = vi.fn(async () => true);
      await consumeWindowSeed({ openProject, openFile });
      expect(openProject).toHaveBeenCalledWith('/work/p');
      expect(openFile).not.toHaveBeenCalled();
    });
  });

  it('opens file when #file= is present, with line if provided', async () => {
    await withHash('#file=%2Fa.md&line=42', async () => {
      const openProject = vi.fn();
      const openFile = vi.fn(async () => true);
      await consumeWindowSeed({ openProject, openFile });
      expect(openProject).not.toHaveBeenCalled();
      expect(openFile).toHaveBeenCalledWith('/a.md', 42);
    });
  });

  it('clears the hash after dispatching', async () => {
    await withHash('#file=%2Fa.md', async () => {
      const replaceSpy = vi.spyOn(history, 'replaceState');
      await consumeWindowSeed({
        openProject: vi.fn(),
        openFile: vi.fn(async () => true),
      });
      expect(replaceSpy).toHaveBeenCalled();
    });
  });

  it('treats non-numeric or zero line/col as null', async () => {
    await withHash('#file=%2Fa.md&line=abc&col=0', async () => {
      const openFile = vi.fn(async () => true);
      await consumeWindowSeed({
        openProject: vi.fn(),
        openFile,
      });
      // line=abc → null; col=0 ignored (positive only)
      expect(openFile).toHaveBeenCalledWith('/a.md', null);
    });
  });

  it('handles hash without leading #', async () => {
    await withHash('project=%2Fp', async () => {
      const openProject = vi.fn();
      await consumeWindowSeed({
        openProject,
        openFile: vi.fn(async () => true),
      });
      expect(openProject).toHaveBeenCalledWith('/p');
    });
  });
});

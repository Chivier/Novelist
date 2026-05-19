/**
 * Routing logic for the `cli-open` event emitted by the Rust single-instance
 * callback.
 *
 * - **Folders** always spawn new windows.
 * - **Files** go through the cross-window router (`routeSingleFileOpen`),
 *   which either picks an existing window that owns the file or spawns a
 *   fresh single-file window. The `-n / --new-window` flag is forwarded so
 *   the router skips the bid round and spawns immediately.
 *
 * URL-hash payload format (read by App.svelte mount in the new window):
 *   #project=<encoded-path>
 *   #file=<encoded-path>
 *   #file=<encoded-path>&line=<n>&col=<n>
 */

import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { CliOpenPayload } from '$lib/ipc/commands';
import { routeSingleFileOpen } from '$lib/services/file-route';

export type FileOpener = (filePath: string, line: number | null) => Promise<boolean>;

/**
 * Spawn a new window seeded with a project path. The new window's
 * `App.svelte` reads `window.location.hash` on mount and opens the project.
 *
 * `spawnNew=false` is reserved for the cold-start path where the *current*
 * window is the freshly-opened one — that path bypasses this helper and
 * calls the project-open store directly.
 */
export async function openProjectInThisWindow(dirPath: string, spawnNew = true): Promise<void> {
  if (!spawnNew) {
    // Defer to caller; this branch is only hit by mistake.
    console.warn('[cli-open] openProjectInThisWindow called with spawnNew=false');
    return;
  }
  await spawnWindow({ project: dirPath });
}

export async function openFileInNewWindow(
  filePath: string,
  line: number | null,
  col: number | null,
): Promise<void> {
  await spawnWindow({ file: filePath, line, col });
}

interface WindowSeed {
  project?: string;
  file?: string;
  line?: number | null;
  col?: number | null;
}

async function spawnWindow(seed: WindowSeed): Promise<void> {
  const params = new URLSearchParams();
  if (seed.project) params.set('project', seed.project);
  if (seed.file) params.set('file', seed.file);
  if (seed.line) params.set('line', String(seed.line));
  if (seed.col) params.set('col', String(seed.col));
  const hash = params.toString();
  const label = `novelist-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
  const url = hash ? `index.html#${hash}` : 'index.html';

  const win = new WebviewWindow(label, {
    url,
    title: 'Novelist',
    width: 1200,
    height: 800,
    titleBarStyle: 'overlay',
    hiddenTitle: true,
  });
  win.once('tauri://error', (e) => {
    console.error('[cli-open] failed to spawn window:', e);
  });
}

/**
 * Apply the routing rules above. Called from the `cli-open` event listener.
 *
 * Folders spawn new windows directly. Files are routed by the cross-window
 * router, which talks to Rust + every existing window before deciding.
 */
export async function handleCliOpen(payload: CliOpenPayload): Promise<void> {
  for (const folder of payload.folders) {
    await openProjectInThisWindow(folder, /*spawnNew*/ true);
  }
  for (const f of payload.files) {
    await routeSingleFileOpen(f.path, f.line ?? null, f.col ?? null, payload.force_new_window);
  }
}

/**
 * Read the URL hash on window mount and dispatch any seeded file/project.
 * Called once from App.svelte's onMount.
 */
export async function consumeWindowSeed(opts: {
  openProject: (path: string) => Promise<void> | void;
  openFile: FileOpener;
}): Promise<void> {
  const hash = window.location.hash.startsWith('#')
    ? window.location.hash.slice(1)
    : window.location.hash;
  if (!hash) return;
  const params = new URLSearchParams(hash);
  const project = params.get('project');
  const file = params.get('file');
  const line = parseIntOrNull(params.get('line'));
  const col = parseIntOrNull(params.get('col'));

  if (project) {
    await opts.openProject(project);
  }
  if (file) {
    await opts.openFile(file, line);
  }

  // Clear the hash so a reload doesn't re-trigger.
  if (project || file) {
    history.replaceState(null, '', window.location.pathname + window.location.search);
  }

  // col is parsed but currently unused — line-only goto matches the
  // existing `novelist-goto-line` event. Kept in the URL for forward compat.
  void col;
}

function parseIntOrNull(s: string | null): number | null {
  if (!s) return null;
  const n = parseInt(s, 10);
  return Number.isFinite(n) && n > 0 ? n : null;
}

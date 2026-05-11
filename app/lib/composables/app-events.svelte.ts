import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { commands, type CliOpenPayload, type PendingFile, type RecentProject } from '$lib/ipc/commands';
import { projectStore } from '$lib/stores/project.svelte';
import { tabsStore } from '$lib/stores/tabs.svelte';
import { uiStore } from '$lib/stores/ui.svelte';
import { handleCliOpen, openProjectInThisWindow } from '$lib/services/cli-open';
import { pathDirname } from '$lib/utils/path';

export type AppEventContext = {
  /** Called when file-changed arrives for a dirty open tab. */
  onConflict: (filePath: string) => void;
  /** Called with the new list after backend finishes its recent-projects cleanup. */
  onRecentProjectsUpdated: (list: RecentProject[]) => void;
  /** Called by novelist-goto-line CustomEvent from ProjectSearch. */
  onGotoLine: (line: number) => void;
  /** Called when a folder must be opened in *this* window (cold start path). */
  onOpenProjectInThisWindow: (dirPath: string) => Promise<void> | void;
};

const TEXT_EXTENSIONS = ['.md', '.markdown', '.txt', '.canvas', '.kanban', '.json', '.jsonl', '.csv'];

/**
 * Open a text file by absolute path. Used by pending-files drain, the
 * open-file Tauri event (macOS Finder "Open With"), and drag-drop.
 *
 * Kicks the app into single-file mode if no project is open.
 *
 * Returns true on success so the caller can know whether to advance focus
 * (e.g. scroll to a goto line).
 */
async function openFileByPath(filePath: string, line: number | null = null): Promise<boolean> {
  const lower = filePath.toLowerCase();
  if (!TEXT_EXTENSIONS.some(ext => lower.endsWith(ext))) return false;

  const result = await commands.readFile(filePath);
  if (result.status !== 'ok') return false;

  if (!projectStore.isOpen) {
    projectStore.enterSingleFileMode();
    uiStore.sidebarVisible = false;
  }
  tabsStore.openTab(filePath, result.data);
  await commands.registerOpenFile(filePath);

  if (line && line > 0) {
    // Defer to the next frame so the editor view has time to mount before
    // we ask it to jump.
    requestAnimationFrame(() => {
      window.dispatchEvent(new CustomEvent('novelist-goto-line', { detail: { line } }));
    });
  }
  return true;
}

/**
 * Subscribes to all window/IPC events App.svelte cares about. Returns a
 * teardown function to be called from the onMount cleanup block.
 */
export function wireAppEvents(ctx: AppEventContext): () => void {
  let unlistenFileChanged: (() => void) | null = null;
  let unlistenDragDrop: (() => void) | null = null;
  let unlistenOpenFile: (() => void) | null = null;
  let unlistenFileRenamed: (() => void) | null = null;
  let unlistenDirectoryChanged: (() => void) | null = null;
  let unlistenRecentProjectsUpdated: (() => void) | null = null;

  let unlistenCliOpen: (() => void) | null = null;

  // Drain any files + folders queued before the frontend was ready
  // (CLI args, macOS "Open With" on cold start). Folders take precedence:
  // if both are pending, the first folder loads as the project here and
  // any other folders spawn additional windows; files then load as tabs.
  (async () => {
    try {
      const pendingProjects = await invoke<string[]>('get_pending_open_projects');
      const pendingFiles = await invoke<PendingFile[]>('get_pending_open_files');
      if (pendingProjects.length > 0) {
        await ctx.onOpenProjectInThisWindow(pendingProjects[0]);
        // Extra folders go to fresh windows.
        for (const extra of pendingProjects.slice(1)) {
          await openProjectInThisWindow(extra, /*spawnNew*/ true);
        }
      }
      for (const f of pendingFiles) {
        await openFileByPath(f.path, f.line ?? null);
      }
    } catch (_) {
      // ignore — command may not exist on older builds
    }
  })();

  // Listen for open-file events from Rust (macOS Finder "Open With" while running)
  listen<string>('open-file', async (event) => {
    await openFileByPath(event.payload);
  }).then(fn => { unlistenOpenFile = fn; });

  // Hot-path CLI invocations: a second `novelist ...` process forwarded its
  // args via tauri-plugin-single-instance. Route folders to new windows and
  // files to either this window (if in single-file mode) or a fresh window.
  listen<CliOpenPayload>('cli-open', async (event) => {
    await handleCliOpen(event.payload, openFileByPath);
  }).then(fn => { unlistenCliOpen = fn; });

  listen<{ path: string }>('file-changed', async (event) => {
    const { path } = event.payload;

    // Refresh tab content if a currently-open file changed on disk.
    const tab = tabsStore.findByPath(path);
    if (tab) {
      if (!tab.isDirty) {
        const result = await commands.readFile(path);
        if (result.status === 'ok') {
          tabsStore.reloadContent(tab.id, result.data);
        }
      } else {
        ctx.onConflict(path);
      }
    }

    // Refresh the sidebar folder containing the changed path, IF it's
    // been loaded (expanded at least once). refreshFolder is a no-op for
    // folders whose children are still undefined.
    const parent = pathDirname(path);
    if (parent) {
      await projectStore.refreshFolder(parent);
    }
  }).then(fn => { unlistenFileChanged = fn; });

  listen<{ path: string }>('directory-changed', async (event) => {
    await projectStore.refreshFolder(event.payload.path);
  }).then(fn => { unlistenDirectoryChanged = fn; });

  const refreshInterval = window.setInterval(() => {
    if (projectStore.dirPath) {
      projectStore.refreshLoadedFolders().catch(() => {});
    }
  }, 15_000);

  // Cross-window file rename broadcast: another window auto-renamed a file we
  // may have open. Update our tab paths and refresh the affected sidebar folder.
  listen<{ old_path: string; new_path: string }>('file-renamed', (event) => {
    const { old_path, new_path } = event.payload;
    tabsStore.updatePath(old_path, new_path);
    const parent = pathDirname(new_path);
    if (parent) {
      projectStore.refreshFolder(parent).catch(() => {});
    }
  }).then(fn => { unlistenFileRenamed = fn; });

  // Backend background cleanup of recent projects completed — refresh our
  // in-memory list. The event payload is the filtered list.
  listen<RecentProject[]>('recent-projects-updated', (event) => {
    ctx.onRecentProjectsUpdated(event.payload);
  }).then(fn => { unlistenRecentProjectsUpdated = fn; });

  // Drag-and-drop: open text files dropped onto the window
  getCurrentWindow().onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop') {
      for (const filePath of event.payload.paths) {
        await openFileByPath(filePath);
      }
    }
  }).then(fn => { unlistenDragDrop = fn; });

  // Listen for goto-line events from ProjectSearch
  const handleGotoLine = (e: Event) => {
    const detail = (e as CustomEvent).detail;
    if (detail?.line) ctx.onGotoLine(detail.line);
  };
  window.addEventListener('novelist-goto-line', handleGotoLine);

  return () => {
    unlistenFileChanged?.();
    unlistenDirectoryChanged?.();
    unlistenDragDrop?.();
    unlistenOpenFile?.();
    unlistenFileRenamed?.();
    unlistenRecentProjectsUpdated?.();
    unlistenCliOpen?.();
    window.clearInterval(refreshInterval);
    window.removeEventListener('novelist-goto-line', handleGotoLine);
  };
}

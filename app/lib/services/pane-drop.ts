import { commands } from '$lib/ipc/commands';
import { tabsStore } from '$lib/stores/tabs.svelte';

const OPENABLE_EXTENSIONS = ['.md', '.markdown', '.txt', '.canvas', '.kanban', '.json', '.jsonl', '.csv'];

export const SIDEBAR_PATH_MIME = 'application/x-novelist-path';

export function isOpenablePath(path: string): boolean {
  const lower = path.toLowerCase();
  return OPENABLE_EXTENSIONS.some(ext => lower.endsWith(ext));
}

export function hasSidebarPath(types: ReadonlyArray<string> | DOMStringList | undefined): boolean {
  if (!types) return false;
  for (let i = 0; i < types.length; i++) {
    if ((types as ArrayLike<string>)[i].toLowerCase() === SIDEBAR_PATH_MIME) return true;
  }
  return false;
}

/** Open a sidebar-dragged file inside an existing pane. */
export async function openPathInPane(paneId: string, filePath: string): Promise<boolean> {
  if (!isOpenablePath(filePath)) return false;
  const result = await commands.readFile(filePath);
  if (result.status !== 'ok') return false;
  tabsStore.openTabInPane(paneId, filePath, result.data);
  tabsStore.setActivePane(paneId);
  await commands.registerOpenFile(filePath).catch(() => {});
  return true;
}

/**
 * Drop a sidebar-dragged file into a "split right" zone:
 * enable split if it isn't already, then open the file in pane-2.
 */
export async function openPathSplitRight(filePath: string): Promise<boolean> {
  if (!isOpenablePath(filePath)) return false;
  if (!tabsStore.splitActive) tabsStore.toggleSplit();
  return openPathInPane('pane-2', filePath);
}

/**
 * Single-file open routing. When opening a file outside the current project,
 * we hand it off to whichever window actually owns the file's directory
 * instead of polluting the focused project's tabs. If nobody claims, we spawn
 * a fresh single-file window.
 *
 * See: docs/superpowers/specs/2026-05-19-single-file-open-routing-design.md
 *
 * Flow:
 *   1. Entry point calls `routeSingleFileOpen(path, line?, col?)`.
 *   2. The wrapper invokes Rust `route_single_file_open_cmd`. Rust emits a
 *      `file-open-bid-request` to every webview window.
 *   3. Each window's `wireFileRoutingBids()` listener computes a bid via
 *      `computeCanClaim()` and replies via `submit_file_open_bid`.
 *   4. Rust waits ~250ms, picks a winner, and either emits `open-file-deliver`
 *      to the winner or returns `winner_label: null` so we spawn a new window.
 */

import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { commands } from '$lib/ipc/commands';
import { projectStore } from '$lib/stores/project.svelte';
import { tabsStore } from '$lib/stores/tabs.svelte';
import { pathStartsWithChild, pathDirname } from '$lib/utils/path';
import { openFileInNewWindow } from '$lib/services/cli-open';

interface BidRequestPayload {
  bid_id: number;
  path: string;
}

/**
 * Decide whether this window should host `filePath`.
 *
 * Rules:
 *   - Window has a project P → claim iff filePath lives under P.
 *   - Single-file mode with tabs → claim iff filePath shares a parent dir
 *     with at least one existing tab.
 *   - Empty window (no project, no tabs) → always claim (fallback).
 */
export function computeCanClaim(filePath: string): { canClaim: boolean; hasProject: boolean } {
  const projectDir = projectStore.dirPath;
  if (projectDir) {
    return {
      canClaim: pathStartsWithChild(filePath, projectDir),
      hasProject: true,
    };
  }

  const allTabs = tabsStore.panes.flatMap(p => p.tabs);
  if (allTabs.length > 0) {
    const incomingParent = pathDirname(filePath);
    const canClaim = allTabs.some(t => pathDirname(t.filePath) === incomingParent);
    return { canClaim, hasProject: false };
  }

  // Empty scratch window — accept anything.
  return { canClaim: true, hasProject: false };
}

/**
 * Listen for `file-open-bid-request` events and reply with this window's bid.
 * Returns an unlisten function for cleanup.
 */
export async function wireFileRoutingBids(): Promise<() => void> {
  const label = getCurrentWindow().label;
  return listen<BidRequestPayload>('file-open-bid-request', async (event) => {
    const { bid_id, path } = event.payload;
    const { canClaim, hasProject } = computeCanClaim(path);
    try {
      await commands.submitFileOpenBid(bid_id, label, canClaim, hasProject);
    } catch (e) {
      console.warn('[file-route] submitFileOpenBid failed:', e);
    }
  });
}

/**
 * Entry point used by every single-file open path (CLI hot-path, Finder
 * Open-With, drag-drop, pending-file drain). Hands off to Rust, then either
 * trusts Rust's delivery to the winner or spawns a new window locally.
 */
export async function routeSingleFileOpen(
  filePath: string,
  line: number | null = null,
  col: number | null = null,
  forceNew = false,
): Promise<void> {
  const sourceLabel = getCurrentWindow().label;
  const result = await commands.routeSingleFileOpenCmd(
    sourceLabel,
    filePath,
    line ?? null,
    col ?? null,
    forceNew,
  );
  if (!result.winner_label) {
    await openFileInNewWindow(filePath, line, col);
  }
}

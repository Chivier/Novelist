import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { t } from '$lib/i18n';
import { updaterState } from '$lib/stores/updater-state.svelte';

const SKIPPED_VERSION_KEY = 'novelist-skipped-update-version';

function getSkippedVersion(): string | null {
  return localStorage.getItem(SKIPPED_VERSION_KEY);
}

function setSkippedVersion(version: string) {
  localStorage.setItem(SKIPPED_VERSION_KEY, version);
}

export function clearSkippedVersion() {
  localStorage.removeItem(SKIPPED_VERSION_KEY);
}

let _cachedUpdate: Update | null = null;

/**
 * Check for updates.
 *
 * - `silent=true` (startup): if a non-skipped update is found, surface it via
 *   the in-app "Update Available" banner. Errors swallowed.
 * - `silent=false` (manual command): always show a result — either the
 *   confirm dialog, an "already latest" message, or the failure message.
 */
export async function checkForUpdates(silent = true): Promise<void> {
  try {
    const update = await check({ timeout: 10000 });
    if (update) {
      _cachedUpdate = update;

      const skipped = getSkippedVersion();
      if (silent && skipped === update.version) {
        // User chose "Skip This Version" for this exact version — stay quiet
        return;
      }

      updaterState.setAvailable(update.version, update.body ?? null);

      if (!silent) {
        await promptAndInstall();
      }
    } else {
      if (!silent) {
        await message(t('updater.alreadyLatest'), {
          title: t('updater.noUpdates'),
          kind: 'info',
        });
      }
      updaterState.reset();
    }
  } catch (e) {
    console.warn('[updater] Check failed:', e);
    if (!silent) {
      await message(t('updater.checkFailedMessage'), {
        title: t('updater.checkFailed'),
        kind: 'error',
      });
    }
  }
}

/**
 * Entry point used by the in-app banner / "Update available" affordance.
 * Also used by the command palette `installUpdate` after a manual check.
 */
export async function startUpdateFlow(): Promise<void> {
  if (!_cachedUpdate) {
    await checkForUpdates(false);
    return;
  }
  await promptAndInstall();
}

/** Legacy alias kept for `app-commands.ts` callers. */
export const installUpdate = startUpdateFlow;

/**
 * Skip the currently-pending version. Called from the banner.
 */
export function skipPendingVersion(): void {
  const v = updaterState.version;
  if (v) setSkippedVersion(v);
  updaterState.reset();
  _cachedUpdate = null;
}

/** Dismiss the banner without skipping (next startup will re-prompt). */
export function dismissPendingVersion(): void {
  updaterState.reset();
}

/**
 * Native two-step prompt → progress modal → restart prompt.
 */
async function promptAndInstall(): Promise<void> {
  const update = _cachedUpdate;
  if (!update) return;

  const wantUpdate = await ask(
    t('updater.availableMessage', { version: update.version, notes: update.body || '' }),
    { title: t('updater.available'), kind: 'info', okLabel: t('updater.updateNow'), cancelLabel: t('updater.notNow') }
  );

  if (!wantUpdate) {
    const skip = await ask(
      t('updater.skipMessage', { version: update.version }),
      { title: t('updater.skipTitle'), kind: 'info', okLabel: t('updater.skipVersion'), cancelLabel: t('updater.remindLater') }
    );
    if (skip) {
      skipPendingVersion();
    }
    return;
  }

  await downloadAndInstall(update);
}

async function downloadAndInstall(update: Update): Promise<void> {
  updaterState.startDownload(0);

  try {
    await update.downloadAndInstall((event: DownloadEvent) => {
      switch (event.event) {
        case 'Started':
          updaterState.startDownload(event.data.contentLength ?? 0);
          break;
        case 'Progress':
          updaterState.recordChunk(event.data.chunkLength);
          break;
        case 'Finished':
          updaterState.setInstalling();
          break;
      }
    });
  } catch (e) {
    console.warn('[updater] Download/install failed:', e);
    const detail = e instanceof Error ? e.message : String(e);
    updaterState.setError(detail);
    return;
  }

  // Install succeeded; ask the user whether to relaunch now.
  updaterState.setReady();
}

/**
 * Called from the modal's "Restart now" button.
 *
 * `relaunch()` exits the current process and starts the freshly installed
 * binary. We try to flush via the lifecycle handler first so unsaved state
 * isn't lost, but the user has already been asked to restart so we don't
 * second-guess them.
 */
export async function restartForUpdate(): Promise<void> {
  try {
    await relaunch();
  } catch (e) {
    console.error('[updater] relaunch failed:', e);
    const detail = e instanceof Error ? e.message : String(e);
    await message(t('updater.relaunchFailedMessage', { detail }), {
      title: t('updater.relaunchFailed'),
      kind: 'error',
    });
  }
}

/** Called from the modal "Later" button — keeps the new version pending. */
export function deferRestart(): void {
  updaterState.reset();
  _cachedUpdate = null;
}

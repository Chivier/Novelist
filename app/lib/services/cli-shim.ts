/**
 * Frontend wrapper for the install_cli_shim backend command.
 *
 * Surfaces a small dialog flow:
 *   - On macOS/Linux, asks for confirmation before symlinking to
 *     /usr/local/bin/novelist; on EACCES, instructs the user to retry from
 *     a terminal with sudo (we never escalate from the GUI).
 *   - On Windows, copies the .cmd to %LOCALAPPDATA%\Novelist\bin\novelist.cmd
 *     and tells the user to add that directory to PATH (auto-PATH editing
 *     across PowerShell/cmd is risky enough that we leave it manual for v1).
 */

import { ask, message } from '@tauri-apps/plugin-dialog';
import { commands } from '$lib/ipc/commands';

type Translator = (key: string, params?: Record<string, string>) => string;

export async function runInstallCliShim(t: Translator): Promise<void> {
  const statusRes = await commands.cliShimStatus();
  if (statusRes.status !== 'ok') {
    await message(t('cliShim.statusFailed', { detail: statusRes.error }), {
      title: t('cliShim.title'),
      kind: 'error',
    });
    return;
  }

  const status = statusRes.data;
  if (status.installed && status.up_to_date) {
    const reinstall = await ask(
      t('cliShim.alreadyInstalled', { path: status.install_path }),
      { title: t('cliShim.title'), kind: 'info', okLabel: t('cliShim.reinstall'), cancelLabel: t('cliShim.cancel') },
    );
    if (!reinstall) return;
  } else {
    const proceed = await ask(
      t('cliShim.confirmInstall', { path: status.install_path }),
      { title: t('cliShim.title'), kind: 'info', okLabel: t('cliShim.install'), cancelLabel: t('cliShim.cancel') },
    );
    if (!proceed) return;
  }

  const result = await commands.installCliShim();
  if (result.status !== 'ok') {
    await message(t('cliShim.installFailed', { detail: result.error }), {
      title: t('cliShim.title'),
      kind: 'error',
    });
    return;
  }

  // Windows: dir we install to is rarely on PATH out of the box. Prompt the
  // user to add it.
  const isWindows = result.data.install_path.toLowerCase().endsWith('.cmd');
  if (isWindows) {
    const dir = result.data.install_path.replace(/[\\/][^\\/]+$/, '');
    await message(t('cliShim.installedWindows', { path: result.data.install_path, dir }), {
      title: t('cliShim.title'),
      kind: 'info',
    });
  } else {
    await message(t('cliShim.installedUnix', { path: result.data.install_path }), {
      title: t('cliShim.title'),
      kind: 'info',
    });
  }
}

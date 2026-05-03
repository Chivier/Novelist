<script lang="ts">
  import { t } from '$lib/i18n';
  import { updaterState } from '$lib/stores/updater-state.svelte';
  import { restartForUpdate, deferRestart } from '$lib/updater';

  function formatBytes(n: number): string {
    if (n <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    let v = n;
    let i = 0;
    while (v >= 1024 && i < units.length - 1) {
      v /= 1024;
      i++;
    }
    return `${v.toFixed(v >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
  }

  function handleKeydown(e: KeyboardEvent) {
    // Allow Esc only when terminal phases (ready/error) — never during download.
    if (e.key !== 'Escape' || e.isComposing) return;
    if (updaterState.phase === 'ready') {
      e.preventDefault();
      deferRestart();
    } else if (updaterState.phase === 'error') {
      e.preventDefault();
      updaterState.reset();
    }
  }

  function focusPrimary(el: HTMLButtonElement) {
    requestAnimationFrame(() => el.focus());
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center"
  style="background: rgba(0, 0, 0, 0.5);"
  data-testid="update-progress-modal"
>
  <div
    class="rounded-lg shadow-xl p-6 max-w-md w-full mx-4"
    style="background: var(--novelist-bg); color: var(--novelist-text); border: 1px solid var(--novelist-border);"
    role="dialog"
    aria-modal="true"
    aria-labelledby="update-progress-title"
    tabindex="-1"
  >
    {#if updaterState.phase === 'downloading'}
      <h2 id="update-progress-title" class="text-base font-semibold mb-3">
        {t('updater.downloadingTitle', { version: updaterState.version ?? '' })}
      </h2>
      <div
        class="w-full h-2 rounded overflow-hidden mb-2"
        style="background: var(--novelist-bg-secondary);"
      >
        <div
          class="h-full transition-[width] duration-150"
          style="background: var(--novelist-accent); width: {updaterState.percent}%;"
          data-testid="update-progress-bar"
        ></div>
      </div>
      <p class="text-xs" style="color: var(--novelist-text-secondary);">
        {#if updaterState.total > 0}
          {formatBytes(updaterState.downloaded)} / {formatBytes(updaterState.total)} ({updaterState.percent}%)
        {:else}
          {formatBytes(updaterState.downloaded)}
        {/if}
      </p>

    {:else if updaterState.phase === 'installing'}
      <h2 id="update-progress-title" class="text-base font-semibold mb-3">
        {t('updater.installingTitle')}
      </h2>
      <p class="text-sm" style="color: var(--novelist-text-secondary);">
        {t('updater.installingMessage')}
      </p>

    {:else if updaterState.phase === 'ready'}
      <h2 id="update-progress-title" class="text-base font-semibold mb-3">
        {t('updater.restartTitle')}
      </h2>
      <p class="text-sm mb-5 whitespace-pre-line" style="color: var(--novelist-text-secondary);">
        {t('updater.restartMessage', { version: updaterState.version ?? '' })}
      </p>
      <div class="flex gap-3 justify-end">
        <button
          type="button"
          class="px-4 py-2 text-sm rounded cursor-pointer hover:opacity-80"
          style="background: transparent; color: var(--novelist-text-secondary);"
          onclick={deferRestart}
          data-testid="update-restart-later"
        >
          {t('updater.restartLater')}
        </button>
        <button
          type="button"
          class="px-4 py-2 text-sm rounded cursor-pointer hover:opacity-80"
          style="background: var(--novelist-accent); color: #fff;"
          onclick={restartForUpdate}
          use:focusPrimary
          data-testid="update-restart-now"
        >
          {t('updater.restartNow')}
        </button>
      </div>

    {:else if updaterState.phase === 'error'}
      <h2 id="update-progress-title" class="text-base font-semibold mb-3">
        {t('updater.downloadFailed')}
      </h2>
      <p class="text-sm mb-5 whitespace-pre-line" style="color: var(--novelist-text-secondary);">
        {t('updater.downloadFailedMessage', { detail: updaterState.error ?? '' })}
      </p>
      <div class="flex justify-end">
        <button
          type="button"
          class="px-4 py-2 text-sm rounded cursor-pointer hover:opacity-80"
          style="background: var(--novelist-accent); color: #fff;"
          onclick={() => updaterState.reset()}
          use:focusPrimary
          data-testid="update-error-dismiss"
        >
          {t('updater.dismiss')}
        </button>
      </div>
    {/if}
  </div>
</div>

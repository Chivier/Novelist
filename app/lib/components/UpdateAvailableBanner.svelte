<script lang="ts">
  import { t } from '$lib/i18n';
  import { updaterState } from '$lib/stores/updater-state.svelte';
  import { startUpdateFlow, dismissPendingVersion } from '$lib/updater';
</script>

<div
  class="fixed bottom-4 right-4 z-40 rounded-lg shadow-xl px-4 py-3 max-w-sm flex items-center gap-3"
  style="background: var(--novelist-bg); color: var(--novelist-text); border: 1px solid var(--novelist-border);"
  role="status"
  data-testid="update-available-banner"
>
  <div class="flex-1 min-w-0">
    <div class="text-sm font-medium">{t('updater.bannerTitle', { version: updaterState.version ?? '' })}</div>
    <div class="text-xs mt-0.5" style="color: var(--novelist-text-secondary);">
      {t('updater.bannerHint')}
    </div>
  </div>
  <button
    type="button"
    class="px-3 py-1.5 text-xs rounded cursor-pointer hover:opacity-80 shrink-0"
    style="background: var(--novelist-accent); color: #fff;"
    onclick={() => { void startUpdateFlow(); }}
    data-testid="update-banner-install"
  >
    {t('updater.bannerInstall')}
  </button>
  <button
    type="button"
    class="text-xs cursor-pointer hover:opacity-80 shrink-0"
    style="color: var(--novelist-text-secondary);"
    onclick={dismissPendingVersion}
    data-testid="update-banner-dismiss"
    aria-label={t('updater.bannerDismiss')}
  >
    ✕
  </button>
</div>

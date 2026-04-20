<script lang="ts">
  // Thin async shim for the kanban editor. The actual implementation (which
  // pulls in `svelte-dnd-action`) is dynamic-imported so it ships in its own
  // chunk instead of bloating the main bundle.

  import { onMount } from 'svelte';
  import type { Component } from 'svelte';
  import { t } from '$lib/i18n';

  let { paneId = 'pane-1' }: { paneId?: string } = $props();

  let Impl = $state<Component<{ paneId: string }> | null>(null);
  let loadError = $state<string | null>(null);

  onMount(async () => {
    try {
      const mod = await import('./kanban/KanbanImpl.svelte');
      Impl = mod.default as unknown as Component<{ paneId: string }>;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
      console.error('[kanban] failed to load:', e);
    }
  });
</script>

<div class="kanban-wrapper" data-testid="kanban-file-editor">
  {#if Impl}
    <Impl {paneId} />
  {:else if loadError}
    <div class="state">
      <p class="state-title">{t('kanban.loadFailed')}</p>
      <p class="state-hint">{loadError}</p>
    </div>
  {:else}
    <div class="state">
      <span class="spinner" aria-hidden="true"></span>
      <p class="state-hint">{t('kanban.loading')}</p>
    </div>
  {/if}
</div>

<style>
  .kanban-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--novelist-bg);
    color: var(--novelist-text);
  }
  .state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--novelist-text-secondary);
  }
  .state-title { font-size: 0.95rem; font-weight: 500; margin: 0; }
  .state-hint { font-size: 0.8rem; margin: 0; opacity: 0.8; }
  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid color-mix(in srgb, var(--novelist-text) 18%, transparent);
    border-top-color: var(--novelist-accent);
    border-radius: 50%;
    animation: spin 720ms linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>

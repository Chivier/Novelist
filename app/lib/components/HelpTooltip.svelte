<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    /** Accessible label for the "?" trigger. */
    label: string;
    /** Card content. */
    children: Snippet;
  }
  let { label, children }: Props = $props();

  let open = $state(false);
  let closeTimer: ReturnType<typeof setTimeout> | null = null;
  let openTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleOpen() {
    if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
    if (open) return;
    openTimer = setTimeout(() => { open = true; openTimer = null; }, 300);
  }

  function scheduleClose() {
    if (openTimer) { clearTimeout(openTimer); openTimer = null; }
    closeTimer = setTimeout(() => { open = false; closeTimer = null; }, 150);
  }

  function toggle(e: Event) {
    e.stopPropagation();
    if (openTimer) { clearTimeout(openTimer); openTimer = null; }
    if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
    open = !open;
  }

  function closeFromWindow() { open = false; }

  onDestroy(() => {
    if (openTimer) clearTimeout(openTimer);
    if (closeTimer) clearTimeout(closeTimer);
  });
</script>

<svelte:window onclick={closeFromWindow} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span class="help-wrap" onmouseenter={scheduleOpen} onmouseleave={scheduleClose}>
  <button
    type="button"
    class="help-trigger"
    aria-label={label}
    data-testid="help-trigger"
    onclick={toggle}
  >?</button>
  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="help-card"
      role="tooltip"
      data-testid="help-card"
      onclick={(e) => e.stopPropagation()}
      onmouseenter={scheduleOpen}
      onmouseleave={scheduleClose}
    >
      {@render children()}
    </div>
  {/if}
</span>

<style>
  .help-wrap { position: relative; display: inline-flex; align-items: center; }
  .help-trigger {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    margin-left: 6px;
    border-radius: 50%;
    border: 1px solid var(--novelist-border);
    background: transparent;
    color: var(--novelist-text-secondary);
    font-size: 0.68rem;
    font-weight: 600;
    cursor: help;
  }
  .help-trigger:hover { border-color: var(--novelist-accent); color: var(--novelist-accent); }
  .help-card {
    position: absolute;
    top: 22px;
    left: -4px;
    z-index: 40;
    min-width: 280px;
    max-width: 360px;
    padding: 12px 14px;
    border-radius: 8px;
    background: var(--novelist-bg);
    border: 1px solid var(--novelist-border);
    box-shadow: 0 6px 20px rgba(0,0,0,0.12);
    color: var(--novelist-text);
    font-size: 0.78rem;
    line-height: 1.5;
  }
</style>

<script lang="ts">
  import { buildWordDiff } from './diff';

  type Props = {
    original: string;
    revised: string;
    onAccept: () => void;
    onReject: () => void;
    onRegenerate?: () => void;
  };

  let { original, revised, onAccept, onReject, onRegenerate }: Props = $props();
  let parts = $derived(buildWordDiff(original, revised));

  async function copyResult() {
    await navigator.clipboard?.writeText(revised).catch(() => {});
  }
</script>

<section class="review" data-testid="inline-edit-review">
  <div class="diff">
    {#each parts as part}
      <span class={part.kind}>{part.text}</span>
    {/each}
  </div>
  <div class="actions">
    <button class="novelist-btn novelist-btn-primary" type="button" onclick={onAccept}>Accept all</button>
    <button class="novelist-btn novelist-btn-ghost" type="button" onclick={onReject}>Reject</button>
    <button class="novelist-btn novelist-btn-ghost" type="button" onclick={copyResult}>Copy result</button>
    {#if onRegenerate}
      <button class="novelist-btn novelist-btn-ghost" type="button" onclick={onRegenerate}>Regenerate</button>
    {/if}
  </div>
</section>

<style>
  .review {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .diff {
    max-height: 220px;
    overflow: auto;
    padding: 8px;
    border: 1px solid var(--novelist-border);
    border-radius: 4px;
    background: var(--novelist-bg);
    white-space: pre-wrap;
    line-height: 1.6;
  }
  .same { color: var(--novelist-text); }
  .added {
    color: #166534;
    background: rgba(34, 197, 94, 0.18);
  }
  .removed {
    color: #991b1b;
    background: rgba(239, 68, 68, 0.16);
    text-decoration: line-through;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
</style>

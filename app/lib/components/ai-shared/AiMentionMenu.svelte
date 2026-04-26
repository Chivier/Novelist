<script lang="ts">
  type Mention = { token: string; label: string; hint: string };
  const MENTIONS: Mention[] = [
    { token: '@selection', label: '@selection', hint: 'selected editor text' },
    { token: '@current', label: '@current', hint: 'current file contents' },
    { token: '@outline', label: '@outline', hint: 'current file heading outline' },
    { token: '@file:', label: '@file:path', hint: 'attach first matching file' },
    { token: '@folder:', label: '@folder:path', hint: 'attach folder summary' },
  ];

  type Props = {
    visible: boolean;
    query: string;
    onPick: (token: string) => void;
  };

  let { visible, query, onPick }: Props = $props();
  let filtered = $derived(MENTIONS.filter((m) => m.label.toLowerCase().includes(query.toLowerCase())));
</script>

{#if visible && filtered.length > 0}
  <div class="menu" data-testid="ai-mention-menu">
    {#each filtered as mention}
      <button type="button" onclick={() => onPick(mention.token)}>
        <strong>{mention.label}</strong>
        <span>{mention.hint}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .menu {
    border: 1px solid var(--novelist-border);
    background: var(--novelist-bg);
    border-radius: 4px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
    overflow: hidden;
  }
  button {
    display: flex;
    width: 100%;
    gap: 8px;
    align-items: baseline;
    border: 0;
    background: transparent;
    color: var(--novelist-text);
    padding: 5px 8px;
    font: inherit;
    font-size: 11px;
    cursor: pointer;
    text-align: left;
  }
  button:hover { background: var(--novelist-bg-secondary); }
  span { color: var(--novelist-text-secondary); }
</style>

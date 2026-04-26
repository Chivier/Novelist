<script lang="ts">
  import type { SlashCommandId } from './context';

  type Command = { id: SlashCommandId; label: string; hint: string };
  const COMMANDS: Command[] = [
    { id: 'rewrite', label: '/rewrite', hint: 'rewrite selected or attached prose' },
    { id: 'summarize', label: '/summarize', hint: 'summarize attached context' },
    { id: 'continue', label: '/continue', hint: 'continue in the same voice' },
    { id: 'translate', label: '/translate', hint: 'translate literary prose' },
    { id: 'line-edit', label: '/line-edit', hint: 'line edit with concrete changes' },
    { id: 'brainstorm', label: '/brainstorm', hint: 'generate story options' },
    { id: 'compact', label: '/compact', hint: 'compress conversation' },
    { id: 'clear', label: '/clear', hint: 'reset current session' },
    { id: 'save', label: '/save', hint: 'save transcript to project' },
    { id: 'plan', label: '/plan', hint: 'switch agent to plan mode' },
    { id: 'act', label: '/act', hint: 'switch agent to act mode' },
  ];

  type Props = {
    visible: boolean;
    query: string;
    onPick: (id: SlashCommandId) => void;
  };

  let { visible, query, onPick }: Props = $props();
  let filtered = $derived(COMMANDS.filter((c) => c.label.includes(query.toLowerCase())));
</script>

{#if visible && filtered.length > 0}
  <div class="menu" data-testid="ai-command-menu">
    {#each filtered as cmd}
      <button type="button" onclick={() => onPick(cmd.id)}>
        <strong>{cmd.label}</strong>
        <span>{cmd.hint}</span>
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

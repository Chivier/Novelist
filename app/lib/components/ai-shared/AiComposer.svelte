<script lang="ts">
  import type { Snippet } from 'svelte';
  import AiContextBar from './AiContextBar.svelte';
  import AiMentionMenu from './AiMentionMenu.svelte';
  import AiCommandMenu from './AiCommandMenu.svelte';
  import type { SlashCommandId } from './context';
  import type { AiContextAttachment } from './attachments';
  import { attachmentToContextItem } from './attachments';
  import { IconClose, IconDocument } from '../icons';

  type SuggestedSelection = {
    attachment: AiContextAttachment;
    status: 'suggested' | 'attached' | 'dismissed';
  } | null;

  type Props = {
    value: string;
    placeholder: string;
    inputTestId?: string;
    attachments: readonly AiContextAttachment[];
    mentionVisible: boolean;
    mentionQuery: string;
    commandVisible: boolean;
    commandQuery: string;
    suggestedSelection?: SuggestedSelection;
    busy?: boolean;
    canSend: boolean;
    sendLabel?: string;
    sendTestId?: string;
    stopTestId?: string;
    onInput: (value: string) => void;
    onSend: () => void;
    onStop?: () => void;
    mentionCandidates?: readonly AiContextAttachment[];
    onPickMention: (token: string, attachment?: AiContextAttachment) => void | Promise<void>;
    onPickCommand: (id: SlashCommandId) => void;
    onRemoveAttachment: (id: string) => void;
    onClearAttachments: () => void;
    onAttachSelection?: () => void;
    onDismissSelection?: () => void;
    onDropPaths?: (paths: string[]) => void;
    actions?: Snippet;
  };

  let {
    value,
    placeholder,
    inputTestId,
    attachments,
    mentionVisible,
    mentionQuery,
    commandVisible,
    commandQuery,
    suggestedSelection = null,
    busy = false,
    canSend,
    sendLabel = 'Send',
    sendTestId,
    stopTestId,
    onInput,
    onSend,
    onStop,
    mentionCandidates = [],
    onPickMention,
    onPickCommand,
    onRemoveAttachment,
    onClearAttachments,
    onAttachSelection,
    onDismissSelection,
    onDropPaths,
    actions,
  }: Props = $props();

  let contextItems = $derived(attachments.map(attachmentToContextItem));

  function keydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      if (busy) onStop?.();
      else if (canSend) onSend();
    }
  }

  function drop(e: DragEvent) {
    const raw = e.dataTransfer?.getData('text/plain') || '';
    const paths = raw.split('\n').map((p) => p.trim()).filter(Boolean);
    if (paths.length > 0) {
      e.preventDefault();
      onDropPaths?.(paths);
    }
  }
</script>

<div
  class="ai-composer"
  data-testid="ai-composer"
  role="group"
  aria-label="AI composer"
  ondrop={drop}
  ondragover={(e) => e.preventDefault()}
>
  {#if suggestedSelection && suggestedSelection.status === 'suggested'}
    <div class="selection-suggestion" data-testid="ai-selection-suggestion">
      <button type="button" class="suggestion-main" onclick={onAttachSelection}>
        <IconDocument size={12} />
        <span>{suggestedSelection.attachment.label}</span>
      </button>
      <button type="button" class="suggestion-close" aria-label="Dismiss selection" onclick={onDismissSelection}>
        <IconClose size={12} />
      </button>
    </div>
  {/if}
  <AiContextBar
    items={contextItems}
    onRemove={onRemoveAttachment}
    onClear={onClearAttachments}
  />
  <AiCommandMenu visible={commandVisible} query={commandQuery} onPick={onPickCommand} />
  <AiMentionMenu
    visible={mentionVisible}
    query={mentionQuery}
    candidates={mentionCandidates}
    onPick={onPickMention}
  />
  <textarea
    data-testid={inputTestId}
    rows="3"
    {placeholder}
    value={value}
    oninput={(e) => onInput(e.currentTarget.value)}
    onkeydown={keydown}
  ></textarea>
  <div class="composer-actions">
    {#if actions}
      {@render actions()}
    {/if}
    {#if busy}
      <button class="novelist-btn novelist-btn-primary" data-testid={stopTestId} type="button" onclick={() => onStop?.()}>Stop</button>
    {:else}
      <button class="novelist-btn novelist-btn-primary" data-testid={sendTestId} type="button" onclick={onSend} disabled={!canSend}>{sendLabel}</button>
    {/if}
  </div>
</div>

<style>
  .ai-composer {
    border-top: 1px solid var(--novelist-border);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--novelist-bg-secondary);
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--novelist-bg);
    border: 1px solid var(--novelist-border);
    color: var(--novelist-text);
    border-radius: 4px;
    padding: 6px 8px;
    font: inherit;
    resize: vertical;
  }
  .selection-suggestion {
    display: flex;
    align-items: center;
    border: 1px dashed color-mix(in srgb, var(--novelist-accent) 45%, var(--novelist-border));
    border-radius: 4px;
    background: color-mix(in srgb, var(--novelist-accent) 8%, var(--novelist-bg));
    overflow: hidden;
  }
  .suggestion-main {
    flex: 1;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    border: 0;
    background: transparent;
    color: var(--novelist-text);
    padding: 4px 6px;
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  .suggestion-main span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .suggestion-close {
    display: inline-flex;
    align-items: center;
    border: 0;
    background: transparent;
    color: var(--novelist-text-secondary);
    padding: 4px 6px;
    cursor: pointer;
  }
  .composer-actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
</style>

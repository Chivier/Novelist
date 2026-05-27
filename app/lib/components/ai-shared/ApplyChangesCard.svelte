<script lang="ts">
  import type { AiChangeSet, AiFileChange } from './apply-change-set';

  type Props = {
    changeSet: AiChangeSet;
    status?: 'pending' | 'accepted' | 'rejected' | 'conflict';
    onAcceptFile: (file: AiFileChange) => void | Promise<void>;
    onRejectFile: (file: AiFileChange) => void | Promise<void>;
    onAcceptAll: () => void | Promise<void>;
    onRejectAll: () => void | Promise<void>;
  };

  let { changeSet, status = 'pending', onAcceptFile, onRejectFile, onAcceptAll, onRejectAll }: Props = $props();
</script>

<section class="apply-card" data-testid="ai-apply-changes-card">
  <header>
    <div>
      <strong>Apply Changes</strong>
      <span>{changeSet.summary}</span>
    </div>
    <em>{status}</em>
  </header>
  <div class="files">
    {#each changeSet.files as file (file.path)}
      <details open>
        <summary>
          <span>{file.status}</span>
          <code>{file.path}</code>
        </summary>
        {#each file.hunks as hunk}
          <pre>{#each hunk.lines as line}{line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' '}{line.text}
{/each}</pre>
        {/each}
        {#if file.conflict}
          <p class="conflict">{file.conflict}</p>
        {/if}
        <div class="file-actions">
          <button class="novelist-btn novelist-btn-primary novelist-btn-sm" type="button" onclick={() => onAcceptFile(file)}>Accept file</button>
          <button class="novelist-btn novelist-btn-ghost novelist-btn-sm" type="button" onclick={() => onRejectFile(file)}>Reject file</button>
        </div>
      </details>
    {/each}
  </div>
  <footer>
    <button class="novelist-btn novelist-btn-primary novelist-btn-sm" type="button" onclick={onAcceptAll}>Accept all</button>
    <button class="novelist-btn novelist-btn-ghost novelist-btn-sm" type="button" onclick={onRejectAll}>Reject all</button>
  </footer>
</section>

<style>
  .apply-card {
    border: 1px solid var(--novelist-border);
    border-radius: 4px;
    background: var(--novelist-bg-secondary);
    overflow: hidden;
  }
  header,
  footer,
  .file-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
  }
  header {
    justify-content: space-between;
    border-bottom: 1px solid var(--novelist-border);
  }
  header div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  header span,
  header em {
    color: var(--novelist-text-secondary);
    font-size: 11px;
    font-style: normal;
  }
  details {
    border-bottom: 1px solid var(--novelist-border);
  }
  summary {
    cursor: pointer;
    padding: 5px 8px;
    font-size: 11px;
  }
  code {
    color: var(--novelist-text);
  }
  pre {
    margin: 0;
    padding: 6px 8px;
    max-height: 220px;
    overflow: auto;
    white-space: pre-wrap;
    background: var(--novelist-bg);
    font-size: 11px;
  }
  .conflict {
    margin: 6px 8px;
    color: #b45309;
    font-size: 11px;
  }
  footer {
    justify-content: flex-end;
  }
</style>

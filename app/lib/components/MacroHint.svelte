<script lang="ts">
  import { t } from '$lib/i18n';
</script>

<span class="macro-hint" data-testid="macro-hint">
  <button
    type="button"
    class="macro-hint-trigger"
    data-testid="macro-hint-trigger"
    aria-label={t('settings.editor.newFile.macroHint.label')}
    aria-describedby="macro-hint-popover"
    tabindex="0"
  >?</button>
  <span
    id="macro-hint-popover"
    class="macro-hint-popover"
    data-testid="macro-hint-popover"
    role="tooltip"
  >
    <strong>{t('settings.editor.newFile.macroHint.headingCounter')}</strong>
    <pre>{t('settings.editor.newFile.macroHint.body')}</pre>
    <strong>{t('settings.editor.newFile.macroHint.headingExamples')}</strong>
    <code>{t('settings.editor.newFile.macroHint.example1')}</code>
    <code>{t('settings.editor.newFile.macroHint.example2')}</code>
    <em>{t('settings.editor.newFile.macroHint.note')}</em>
  </span>
</span>

<style>
  .macro-hint {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .macro-hint-trigger {
    width: 18px;
    height: 18px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    border: 1px solid var(--novelist-border);
    background: transparent;
    color: var(--novelist-text-secondary);
    font-size: 0.7rem;
    font-weight: 600;
    line-height: 1;
    cursor: help;
    padding: 0;
    transition: color 100ms, border-color 100ms;
  }
  .macro-hint-trigger:hover,
  .macro-hint-trigger:focus-visible {
    color: var(--novelist-accent);
    border-color: var(--novelist-accent);
    outline: none;
  }

  .macro-hint-popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 320px;
    max-width: 460px;
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid var(--novelist-border);
    background: var(--novelist-bg, #fff);
    color: var(--novelist-text);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.12);
    font-size: 0.78rem;
    line-height: 1.45;
    z-index: 60;
    visibility: hidden;
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity 120ms, transform 120ms, visibility 120ms;
    pointer-events: none;
    text-align: left;
  }
  .macro-hint:hover .macro-hint-popover,
  .macro-hint-trigger:focus-visible + .macro-hint-popover {
    visibility: visible;
    opacity: 1;
    transform: translateY(0);
    pointer-events: auto;
  }
  .macro-hint-popover strong {
    display: block;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--novelist-text-secondary);
    margin: 8px 0 4px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .macro-hint-popover strong:first-child { margin-top: 0; }
  .macro-hint-popover pre {
    font-family: var(--novelist-font-mono, ui-monospace, SFMono-Regular, monospace);
    font-size: 0.74rem;
    line-height: 1.4;
    white-space: pre;
    margin: 0 0 4px;
    color: var(--novelist-text);
    background: var(--novelist-bg-secondary);
    padding: 6px 8px;
    border-radius: 4px;
    overflow-x: auto;
  }
  .macro-hint-popover code {
    display: block;
    font-family: var(--novelist-font-mono, ui-monospace, SFMono-Regular, monospace);
    font-size: 0.72rem;
    color: var(--novelist-text);
    background: var(--novelist-bg-secondary);
    padding: 4px 6px;
    border-radius: 3px;
    margin: 2px 0;
  }
  .macro-hint-popover em {
    display: block;
    margin-top: 6px;
    font-style: normal;
    font-size: 0.7rem;
    color: var(--novelist-text-tertiary, var(--novelist-text-secondary));
  }
</style>

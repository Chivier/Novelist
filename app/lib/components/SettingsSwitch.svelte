<script lang="ts">
  interface Props {
    checked: boolean;
    disabled?: boolean;
    label?: string;
    description?: string;
    id?: string;
    testId?: string;
    title?: string;
    ariaLabel?: string;
    onCheckedChange?: (checked: boolean) => void | Promise<void>;
  }

  let {
    checked,
    disabled = false,
    label,
    description,
    id,
    testId,
    title,
    ariaLabel,
    onCheckedChange,
  }: Props = $props();

  function toggle() {
    if (disabled) return;
    void onCheckedChange?.(!checked);
  }
</script>

<button
  {id}
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={ariaLabel ?? label}
  data-testid={testId}
  {title}
  {disabled}
  class="settings-switch"
  class:settings-switch-on={checked}
  class:settings-switch-with-copy={!!(label || description)}
  onclick={toggle}
>
  <span class="settings-switch-track" aria-hidden="true">
    <span class="settings-switch-thumb"></span>
  </span>
  {#if label || description}
    <span class="settings-switch-copy">
      {#if label}
        <span class="settings-switch-label">{label}</span>
      {/if}
      {#if description}
        <span class="settings-switch-description">{description}</span>
      {/if}
    </span>
  {/if}
</button>

<style>
  .settings-switch {
    display: inline-flex;
    align-items: center;
    justify-content: flex-start;
    gap: 10px;
    min-width: 42px;
    min-height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--novelist-text);
    text-align: left;
    cursor: pointer;
  }
  .settings-switch:disabled {
    cursor: default;
    opacity: 0.55;
  }
  .settings-switch-track {
    position: relative;
    flex: 0 0 auto;
    width: 42px;
    height: 24px;
    border-radius: 999px;
    background: var(--novelist-bg-tertiary, var(--novelist-bg-secondary));
    border: 1px solid var(--novelist-border);
    transition: background 120ms ease, border-color 120ms ease;
  }
  .settings-switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 999px;
    background: var(--novelist-bg);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.26);
    transition: transform 120ms ease, background 120ms ease;
  }
  .settings-switch-on .settings-switch-track {
    background: var(--novelist-accent);
    border-color: var(--novelist-accent);
  }
  .settings-switch-on .settings-switch-thumb {
    transform: translateX(18px);
    background: #fff;
  }
  .settings-switch-with-copy {
    width: 100%;
    align-items: flex-start;
  }
  .settings-switch-copy {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding-top: 1px;
  }
  .settings-switch-label {
    font-size: 0.875rem;
    line-height: 1.35;
    color: var(--novelist-text);
  }
  .settings-switch-description {
    font-size: 0.75rem;
    line-height: 1.35;
    color: var(--novelist-text-secondary);
  }
</style>

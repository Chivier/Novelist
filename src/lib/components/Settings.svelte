<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  const fontOptions = [
    { label: 'LXGW WenKai', value: '"LXGW WenKai", "Noto Serif SC", Georgia, serif' },
    { label: 'Noto Serif SC', value: '"Noto Serif SC", Georgia, serif' },
    { label: 'System Serif', value: 'Georgia, "Times New Roman", serif' },
    { label: 'System Sans', value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif' },
    { label: 'Source Han Serif', value: '"Source Han Serif SC", "Noto Serif SC", Georgia, serif' },
    { label: 'PingFang SC', value: '"PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif' },
  ];

  const fontSizeOptions = [13, 14, 15, 16, 17, 18, 20, 22, 24];
  const lineHeightOptions = [1.4, 1.5, 1.6, 1.8, 2.0, 2.2];
  const maxWidthOptions = [
    { label: '600px (Narrow)', value: 600 },
    { label: '680px', value: 680 },
    { label: '720px (Default)', value: 720 },
    { label: '800px', value: 800 },
    { label: '900px (Wide)', value: 900 },
    { label: '100% (Full)', value: 9999 },
  ];

  let settings = $derived(uiStore.editorSettings);

  function handleFontChange(e: Event) {
    uiStore.updateEditorSettings({ fontFamily: (e.target as HTMLSelectElement).value });
  }

  function handleFontSizeChange(e: Event) {
    uiStore.updateEditorSettings({ fontSize: Number((e.target as HTMLSelectElement).value) });
  }

  function handleLineHeightChange(e: Event) {
    uiStore.updateEditorSettings({ lineHeight: Number((e.target as HTMLSelectElement).value) });
  }

  function handleMaxWidthChange(e: Event) {
    uiStore.updateEditorSettings({ maxWidth: Number((e.target as HTMLSelectElement).value) });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center"
  style="background: rgba(0,0,0,0.4);"
  onclick={onClose}
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="rounded-lg shadow-xl w-full max-w-md"
    style="background: var(--novelist-bg); color: var(--novelist-text); border: 1px solid var(--novelist-border);"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4" style="border-bottom: 1px solid var(--novelist-border);">
      <h2 class="text-base font-semibold">Settings</h2>
      <button
        class="text-sm px-2 py-1 rounded cursor-pointer hover:opacity-80"
        style="color: var(--novelist-text-secondary);"
        onclick={onClose}
      >Esc</button>
    </div>

    <!-- Body -->
    <div class="px-5 py-4 space-y-5">
      <!-- Section: Editor -->
      <div>
        <h3 class="text-xs font-semibold uppercase tracking-wide mb-3" style="color: var(--novelist-text-secondary);">Editor</h3>

        <!-- Font Family -->
        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Font Family</label>
          <select
            class="text-sm px-2 py-1 rounded cursor-pointer"
            style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border); max-width: 200px;"
            value={settings.fontFamily}
            onchange={handleFontChange}
          >
            {#each fontOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Font Size -->
        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Font Size</label>
          <select
            class="text-sm px-2 py-1 rounded cursor-pointer"
            style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border);"
            value={settings.fontSize}
            onchange={handleFontSizeChange}
          >
            {#each fontSizeOptions as size}
              <option value={size}>{size}px</option>
            {/each}
          </select>
        </div>

        <!-- Line Height -->
        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Line Height</label>
          <select
            class="text-sm px-2 py-1 rounded cursor-pointer"
            style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border);"
            value={settings.lineHeight}
            onchange={handleLineHeightChange}
          >
            {#each lineHeightOptions as lh}
              <option value={lh}>{lh}</option>
            {/each}
          </select>
        </div>

        <!-- Max Width -->
        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Editor Width</label>
          <select
            class="text-sm px-2 py-1 rounded cursor-pointer"
            style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border);"
            value={settings.maxWidth}
            onchange={handleMaxWidthChange}
          >
            {#each maxWidthOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Preview -->
      <div>
        <h3 class="text-xs font-semibold uppercase tracking-wide mb-2" style="color: var(--novelist-text-secondary);">Preview</h3>
        <div
          class="rounded p-3 text-sm"
          style="background: var(--novelist-bg-secondary); font-family: {settings.fontFamily}; font-size: {settings.fontSize}px; line-height: {settings.lineHeight}; border: 1px solid var(--novelist-border);"
        >
          The quick brown fox jumps over the lazy dog.<br/>
          落霞与孤鹜齐飞，秋水共长天一色。
        </div>
      </div>
    </div>
  </div>
</div>

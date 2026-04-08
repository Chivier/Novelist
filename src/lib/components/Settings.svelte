<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { builtinThemes } from '$lib/themes';
  import { commands } from '$lib/ipc/commands';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let activeSection = $state<'editor' | 'theme' | 'plugins'>('editor');

  // Editor settings
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

  // Theme options: system + all builtins
  const themeOptions = [
    { id: 'system', name: 'System (Auto)', dark: false },
    ...builtinThemes,
  ];

  // Plugins
  type PluginInfo = { id: string; name: string; version: string; permissions: string[]; active: boolean };
  let plugins = $state<PluginInfo[]>([]);
  let pluginsLoaded = $state(false);

  async function loadPlugins() {
    const result = await commands.listPlugins();
    if (result.status === 'ok') {
      plugins = result.data;
    }
    pluginsLoaded = true;
  }

  async function togglePlugin(plugin: PluginInfo) {
    if (plugin.active) {
      await commands.unloadPlugin(plugin.id);
    } else {
      await commands.loadPlugin(plugin.id);
    }
    await loadPlugins();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.preventDefault(); onClose(); }
  }

  $effect(() => {
    if (activeSection === 'plugins' && !pluginsLoaded) {
      loadPlugins();
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.4);" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="rounded-lg shadow-xl w-full max-w-lg flex"
    style="background: var(--novelist-bg); color: var(--novelist-text); border: 1px solid var(--novelist-border); height: 480px;"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Left nav -->
    <div class="shrink-0 flex flex-col py-3" style="width: 140px; border-right: 1px solid var(--novelist-border); background: var(--novelist-bg-secondary); border-radius: 8px 0 0 8px;">
      {#each [
        { id: 'editor', label: 'Editor' },
        { id: 'theme', label: 'Theme' },
        { id: 'plugins', label: 'Plugins' },
      ] as section}
        <button
          class="text-left px-4 py-2 text-sm cursor-pointer"
          style="background: {activeSection === section.id ? 'var(--novelist-sidebar-active)' : 'transparent'}; color: {activeSection === section.id ? 'var(--novelist-accent)' : 'var(--novelist-text)'}; border: none; font-weight: {activeSection === section.id ? '600' : '400'};"
          onclick={() => activeSection = section.id as any}
        >{section.label}</button>
      {/each}

      <div class="flex-1"></div>
      <button
        class="text-xs px-4 py-2 cursor-pointer"
        style="color: var(--novelist-text-secondary); background: none; border: none;"
        onclick={onClose}
      >Close (Esc)</button>
    </div>

    <!-- Right content -->
    <div class="flex-1 overflow-y-auto px-5 py-4">

      {#if activeSection === 'editor'}
        <h3 class="text-xs font-semibold uppercase tracking-wide mb-4" style="color: var(--novelist-text-secondary);">Editor</h3>

        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Font</label>
          <select class="text-sm px-2 py-1 rounded cursor-pointer" style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border); max-width: 180px;" value={settings.fontFamily} onchange={(e) => uiStore.updateEditorSettings({ fontFamily: (e.target as HTMLSelectElement).value })}>
            {#each fontOptions as opt}<option value={opt.value}>{opt.label}</option>{/each}
          </select>
        </div>

        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Size</label>
          <select class="text-sm px-2 py-1 rounded cursor-pointer" style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border);" value={settings.fontSize} onchange={(e) => uiStore.updateEditorSettings({ fontSize: Number((e.target as HTMLSelectElement).value) })}>
            {#each fontSizeOptions as size}<option value={size}>{size}px</option>{/each}
          </select>
        </div>

        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Line Height</label>
          <select class="text-sm px-2 py-1 rounded cursor-pointer" style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border);" value={settings.lineHeight} onchange={(e) => uiStore.updateEditorSettings({ lineHeight: Number((e.target as HTMLSelectElement).value) })}>
            {#each lineHeightOptions as lh}<option value={lh}>{lh}</option>{/each}
          </select>
        </div>

        <div class="flex items-center justify-between mb-3">
          <label class="text-sm">Width</label>
          <select class="text-sm px-2 py-1 rounded cursor-pointer" style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border: 1px solid var(--novelist-border);" value={settings.maxWidth} onchange={(e) => uiStore.updateEditorSettings({ maxWidth: Number((e.target as HTMLSelectElement).value) })}>
            {#each maxWidthOptions as opt}<option value={opt.value}>{opt.label}</option>{/each}
          </select>
        </div>

        <div class="mt-4 rounded p-3 text-sm" style="background: var(--novelist-bg-secondary); font-family: {settings.fontFamily}; font-size: {settings.fontSize}px; line-height: {settings.lineHeight}; border: 1px solid var(--novelist-border);">
          The quick brown fox jumps over the lazy dog.<br/>
          落霞与孤鹜齐飞，秋水共长天一色。
        </div>

      {:else if activeSection === 'theme'}
        <h3 class="text-xs font-semibold uppercase tracking-wide mb-4" style="color: var(--novelist-text-secondary);">Theme</h3>

        <div class="grid grid-cols-2 gap-2">
          {#each themeOptions as theme}
            <button
              class="rounded-lg p-3 text-left cursor-pointer"
              style="
                border: 2px solid {uiStore.themeId === theme.id ? 'var(--novelist-accent)' : 'var(--novelist-border)'};
                background: {theme.id === 'system' ? 'var(--novelist-bg-secondary)' : theme.dark ? '#1e1e1e' : '#f8f8f8'};
                color: {theme.id === 'system' ? 'var(--novelist-text)' : theme.dark ? '#d4d4d4' : '#2c2c2c'};
              "
              onclick={() => uiStore.setTheme(theme.id)}
            >
              <div class="text-sm font-medium mb-1">{theme.name}</div>
              <div class="text-xs" style="opacity: 0.6">{theme.id === 'system' ? 'Follow OS' : theme.dark ? 'Dark' : 'Light'}</div>
            </button>
          {/each}
        </div>

        <div class="mt-4 text-xs" style="color: var(--novelist-text-secondary);">
          Custom themes can be created by editing <code style="background: var(--novelist-code-bg); padding: 1px 4px; border-radius: 3px;">src/lib/themes.ts</code> — use Claude Code to design your own.
        </div>

      {:else if activeSection === 'plugins'}
        <h3 class="text-xs font-semibold uppercase tracking-wide mb-4" style="color: var(--novelist-text-secondary);">Plugins</h3>

        {#if !pluginsLoaded}
          <p class="text-sm" style="color: var(--novelist-text-secondary);">Loading...</p>
        {:else if plugins.length === 0}
          <div class="text-sm" style="color: var(--novelist-text-secondary);">
            <p class="mb-3">No plugins installed.</p>
            <div class="rounded p-3" style="background: var(--novelist-bg-secondary); border: 1px solid var(--novelist-border);">
              <p class="font-medium mb-2">Create a plugin</p>
              <p class="text-xs mb-2">Plugins live in <code style="background: var(--novelist-code-bg); padding: 1px 4px; border-radius: 3px;">~/.novelist/plugins/&lt;id&gt;/</code></p>
              <p class="text-xs mb-1">Each plugin needs:</p>
              <ul class="text-xs list-disc pl-4 space-y-1">
                <li><code style="background: var(--novelist-code-bg); padding: 1px 4px; border-radius: 3px;">manifest.toml</code> — metadata &amp; permissions</li>
                <li><code style="background: var(--novelist-code-bg); padding: 1px 4px; border-radius: 3px;">index.js</code> — plugin code</li>
              </ul>
              <p class="text-xs mt-2">Use Claude Code: <em>"Create a Novelist plugin that counts sentences"</em></p>
            </div>
          </div>
        {:else}
          <div class="space-y-2">
            {#each plugins as plugin}
              <div class="flex items-center justify-between rounded p-3" style="background: var(--novelist-bg-secondary); border: 1px solid var(--novelist-border);">
                <div>
                  <div class="text-sm font-medium">{plugin.name}</div>
                  <div class="text-xs" style="color: var(--novelist-text-secondary);">
                    {plugin.id} v{plugin.version}
                    {#if plugin.permissions.length > 0}
                      &middot; {plugin.permissions.join(', ')}
                    {/if}
                  </div>
                </div>
                <button
                  class="text-xs px-3 py-1 rounded cursor-pointer"
                  style="background: {plugin.active ? 'var(--novelist-accent)' : 'var(--novelist-bg-tertiary)'}; color: {plugin.active ? '#fff' : 'var(--novelist-text)'}; border: none;"
                  onclick={() => togglePlugin(plugin)}
                >{plugin.active ? 'Active' : 'Enable'}</button>
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

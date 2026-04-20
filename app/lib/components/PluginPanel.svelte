<script lang="ts">
  import type { UIExtension } from '$lib/stores/extensions.svelte';
  import { tabsStore, getEditorView } from '$lib/stores/tabs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { createPluginPanelBridge, type PluginPanelBridge } from '$lib/services/plugin-panel-bridge';

  let { extension, onNavigate }: { extension: UIExtension; onNavigate?: (from: number) => void } = $props();

  let iframeEl = $state<HTMLIFrameElement | undefined>(undefined);
  let loaded = $state(false);
  let bridge: PluginPanelBridge | null = null;

  // Send content updates to the plugin iframe
  $effect(() => {
    const tab = tabsStore.activeTab;
    if (!tab || !iframeEl?.contentWindow || !loaded) return;
    const view = getEditorView(tab.id);
    const content = view?.state.doc.toString() ?? tab.content ?? '';
    iframeEl.contentWindow.postMessage({ type: 'content-update', content }, '*');
  });

  // Send theme updates
  $effect(() => {
    // Track theme changes
    const _theme = uiStore.themeId;
    if (!iframeEl?.contentWindow || !loaded) return;
    const styles = getComputedStyle(document.documentElement);
    const vars: Record<string, string> = {};
    for (const prop of ['--novelist-bg', '--novelist-bg-secondary', '--novelist-text', '--novelist-text-secondary', '--novelist-accent', '--novelist-border']) {
      vars[prop] = styles.getPropertyValue(prop);
    }
    iframeEl.contentWindow.postMessage({ type: 'theme-update', theme: vars }, '*');
  });

  async function attachBridge() {
    if (!iframeEl) return;
    if (bridge) {
      await bridge.destroy();
      bridge = null;
    }
    bridge = await createPluginPanelBridge({
      iframe: iframeEl,
      pluginId: extension.pluginId,
      permissions: extension.permissions,
      onNavigate,
    });
  }

  $effect(() => {
    void attachBridge();
    return () => {
      void bridge?.destroy();
      bridge = null;
    };
  });
</script>

<div class="plugin-panel">
  <!--
    No `sandbox` attribute: WKWebView blocks custom-protocol main-resource loads
    from sandboxed iframes, which breaks plugins whose `entry` is served from
    Tauri's asset protocol. Plugins are trusted (local install or marketplace-vetted).
  -->
  <iframe
    bind:this={iframeEl}
    src={extension.entryUrl}
    title={extension.label}
    onload={() => loaded = true}
  ></iframe>
</div>

<style>
  .plugin-panel {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  iframe {
    flex: 1;
    width: 100%;
    border: none;
    background: var(--novelist-bg);
  }
</style>

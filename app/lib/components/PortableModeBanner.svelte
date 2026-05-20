<script lang="ts">
  import { onMount } from "svelte";
  import { getPortableInfo, type PortableInfo } from "$lib/services/portable";

  let info = $state<PortableInfo | null>(null);

  onMount(async () => {
    info = await getPortableInfo();
  });
</script>

{#if info?.enabled}
  <div
    role="status"
    class="px-4 py-2 text-sm"
    style="background: var(--novelist-bg-secondary); color: var(--novelist-text); border-bottom: 1px solid var(--novelist-border);"
    data-testid="portable-mode-banner"
  >
    便携模式 — 数据存储于 <code class="font-mono" style="color: var(--novelist-text-secondary);">{info.dataRoot}</code>
  </div>
{/if}

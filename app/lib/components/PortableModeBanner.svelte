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
    class="bg-amber-50 border-b border-amber-200 px-4 py-2 text-sm text-amber-900"
    data-testid="portable-mode-banner"
  >
    便携模式 — 数据存储于 <code class="font-mono">{info.dataRoot}</code>
  </div>
{/if}

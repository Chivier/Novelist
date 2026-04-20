<script lang="ts">
  import { SvelteFlowProvider } from '@xyflow/svelte';
  import CanvasInner from './CanvasInner.svelte';

  let { paneId = 'pane-1' }: { paneId?: string } = $props();
</script>

<SvelteFlowProvider>
  <CanvasInner {paneId} />
</SvelteFlowProvider>

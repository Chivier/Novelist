<script lang="ts">
  import { projectStore } from '$lib/stores/project.svelte';

  interface Props {
    wordCount?: number;
    cursorLine?: number;
    cursorCol?: number;
  }

  let { wordCount = 0, cursorLine = 1, cursorCol = 1 }: Props = $props();

  let dailyGoal = $derived(projectStore.config?.writing?.daily_goal ?? 2000);
  let goalPercent = $derived(dailyGoal > 0 ? Math.min(100, Math.round((wordCount / dailyGoal) * 100)) : 0);
</script>

<div
  class="h-5 flex items-center px-3 select-none"
  style="background: var(--novelist-bg); border-top: 1px solid var(--novelist-border-subtle); color: var(--novelist-text-tertiary); font-size: 0.65rem; letter-spacing: 0.01em;"
>
  <span>{wordCount} words</span>
  <span class="ml-auto">Ln {cursorLine}, Col {cursorCol}</span>
</div>

<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, keymap } from '@codemirror/view';
  import type { Extension, ChangeSet } from '@codemirror/state';
  import { createEditorExtensions, createEditorState } from '$lib/editor/setup';
  import { tabsStore, registerEditorView, unregisterEditorView } from '$lib/stores/tabs.svelte';
  import { projectStore } from '$lib/stores/project.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { commands } from '$lib/ipc/commands';
  import { countWords } from '$lib/utils/wordcount';
  import { extractHeadings, type HeadingItem } from '$lib/editor/outline';

  interface Props {
    paneId?: string;
    wordCount?: number;
    cursorLine?: number;
    cursorCol?: number;
    headings?: HeadingItem[];
  }

  // --- Three-tier thresholds ---
  const FILE_SIZE_LARGE = 1024 * 1024;          // 1MB — disable WYSIWYG
  const FILE_SIZE_READONLY = 3.5 * 1024 * 1024; // 3.5MB — read-only mode
  const LARGE_DOC_LINES = 5000;

  let {
    paneId,
    wordCount = $bindable(0),
    cursorLine = $bindable(1),
    cursorCol = $bindable(1),
    headings = $bindable<HeadingItem[]>([]),
  }: Props = $props();

  let effectivePaneId = $derived(paneId ?? tabsStore.activePaneId);
  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;
  let currentTabId: string | null = null;
  let currentTabVersion: number = -1;
  let currentZenMode: boolean = false;
  let isReadOnly = $state(false);
  let readOnlyFileSize = $state(0);
  let statsTimer: ReturnType<typeof setTimeout> | null = null;

  function scrollToPosition(from: number) {
    if (!view) return;
    view.dispatch({
      selection: { anchor: Math.min(from, view.state.doc.length) },
      scrollIntoView: true,
    });
    view.focus();
  }

  function jumpToAbsoluteLine(absLine: number) {
    if (!view) return;
    if (absLine >= 1 && absLine <= view.state.doc.lines) {
      const line = view.state.doc.line(absLine);
      view.dispatch({
        selection: { anchor: line.from },
        scrollIntoView: true,
      });
      view.focus();
    }
  }

  export { scrollToPosition, jumpToAbsoluteLine };

  function updateCursorInfo(v: EditorView) {
    const pos = v.state.selection.main.head;
    const line = v.state.doc.lineAt(pos);
    cursorLine = line.number;
    cursorCol = pos - line.from + 1;
  }

  function getActiveTab() {
    return tabsStore.getPaneActiveTab(effectivePaneId);
  }

  function scheduleStatsUpdate(state: import('@codemirror/state').EditorState) {
    if (statsTimer) clearTimeout(statsTimer);
    const isLarge = state.doc.lines > LARGE_DOC_LINES;
    const delay = isLarge ? 2000 : 300;

    statsTimer = setTimeout(() => {
      statsTimer = null;
      if (isLarge) {
        // Large file: skip heading extraction (forces full parse).
        // Only update word count estimate.
        wordCount = Math.round(state.doc.length / 4);
      } else {
        wordCount = countWords(state.doc.toString());
        headings = extractHeadings(state);
      }
    }, delay);
  }

  function buildExtensions(fileSize: number): Extension[] {
    const readOnly = fileSize >= FILE_SIZE_READONLY;
    const largeFile = fileSize >= FILE_SIZE_LARGE;

    return createEditorExtensions({
      wysiwyg: !largeFile && !readOnly,
      zen: uiStore.zenMode && !largeFile && !readOnly,
      largeFile: largeFile && !readOnly,
      readOnly,
    });
  }

  function buildUpdateListener(): Extension {
    return EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const t = getActiveTab();
        if (t) tabsStore.markDirty(t.id);
        scheduleStatsUpdate(update.state);
      }
      if (update.selectionSet || update.docChanged) {
        updateCursorInfo(update.view);
      }
    });
  }

  async function saveCurrentFile() {
    const tab = getActiveTab();
    if (!tab || !tab.isDirty || !view) return;

    const content = view.state.doc.toString();
    console.log(`[Save] ${tab.fileName}: ${view.state.doc.lines} lines, ${content.length} bytes`);
    await commands.registerWriteIgnore(tab.filePath);
    const result = await commands.writeFile(tab.filePath, content);
    if (result.status === 'ok') {
      tabsStore.updateContent(tab.id, content);
      tabsStore.markSaved(tab.id);
    } else {
      console.error('[Save] Failed:', result.error);
    }
  }

  /** Split a large read-only file into chapters based on H1 headings */
  async function splitIntoChunks() {
    if (!view || !projectStore.dirPath) return;
    const doc = view.state.doc;
    const chunks: { name: string; content: string }[] = [];
    let currentChunk = '';
    let chunkIndex = 0;
    let chunkName = 'part-001';

    for (let i = 1; i <= doc.lines; i++) {
      const lineText = doc.line(i).text;
      if (lineText.startsWith('# ') && currentChunk.length > 0) {
        chunks.push({ name: chunkName, content: currentChunk });
        chunkIndex++;
        chunkName = `part-${String(chunkIndex + 1).padStart(3, '0')}`;
        currentChunk = lineText + '\n';
      } else {
        currentChunk += lineText + '\n';
      }
    }
    if (currentChunk.length > 0) {
      chunks.push({ name: chunkName, content: currentChunk });
    }

    if (chunks.length <= 1) {
      // No H1 headings found — split by line count
      const LINES_PER_CHUNK = 5000;
      chunks.length = 0;
      let idx = 0;
      for (let start = 1; start <= doc.lines; start += LINES_PER_CHUNK) {
        const end = Math.min(start + LINES_PER_CHUNK - 1, doc.lines);
        let content = '';
        for (let i = start; i <= end; i++) {
          content += doc.line(i).text + '\n';
        }
        idx++;
        chunks.push({ name: `part-${String(idx).padStart(3, '0')}`, content });
      }
    }

    // Create chunks directory and write files
    const tab = getActiveTab();
    const baseName = tab?.fileName?.replace(/\.md$/, '') ?? 'split';
    const chunksDir = `${projectStore.dirPath}/${baseName}-chunks`;

    const mkdirResult = await commands.createDirectory(projectStore.dirPath, `${baseName}-chunks`);
    if (mkdirResult.status !== 'ok') {
      console.error('Failed to create chunks directory:', mkdirResult.error);
      return;
    }

    for (const chunk of chunks) {
      await commands.createFile(chunksDir, `${chunk.name}.md`);
      await commands.writeFile(`${chunksDir}/${chunk.name}.md`, chunk.content);
    }

    // Refresh file tree
    const filesResult = await commands.listDirectory(projectStore.dirPath);
    if (filesResult.status === 'ok') {
      projectStore.updateFiles(filesResult.data);
    }

    alert(`Split into ${chunks.length} files in ${baseName}-chunks/`);
  }

  export { splitIntoChunks };

  // --- Tab lifecycle ---

  function cleanupCurrentView() {
    if (view && currentTabId) {
      if (!isReadOnly) {
        tabsStore.syncFromView(currentTabId);
      }
      unregisterEditorView(currentTabId);
    }
    if (view) {
      view.destroy();
      view = null;
    }
    isReadOnly = false;
  }

  function loadTab() {
    const tab = getActiveTab();
    if (!tab) {
      cleanupCurrentView();
      currentTabId = null;
      currentTabVersion = -1;
      return;
    }

    if (tab.id === currentTabId && tab.version === currentTabVersion && currentZenMode === uiStore.zenMode && view) return;

    cleanupCurrentView();
    currentTabId = tab.id;
    currentTabVersion = tab.version;
    currentZenMode = uiStore.zenMode;

    const fileEntry = projectStore.files.find(f => f.path === tab.filePath);
    const fileSize = fileEntry?.size ?? tab.content.length;

    isReadOnly = fileSize >= FILE_SIZE_READONLY;
    readOnlyFileSize = fileSize;

    console.log(`[loadTab] ${tab.fileName}: ${fileSize} bytes, readOnly=${isReadOnly}, lines=${tab.content.split('\n').length}`);

    const extensions = [
      ...buildExtensions(fileSize),
      buildUpdateListener(),
      ...(!isReadOnly ? [keymap.of([{ key: 'Mod-s', run: () => { saveCurrentFile(); return true; } }])] : []),
    ];

    const state = createEditorState(tab.content, extensions);
    view = new EditorView({ state, parent: editorContainer });
    registerEditorView(tab.id, view);

    // Expose for automated testing (accessible from DevTools / test harness)
    (window as any).__novelist_view = view;
    (window as any).__novelist_save = saveCurrentFile;

    console.log(`[loadTab] CM6: ${view.state.doc.lines} lines, lastLine="${view.state.doc.line(view.state.doc.lines).text.substring(0, 60)}"`);

    if (view.state.doc.lines > LARGE_DOC_LINES) {
      wordCount = Math.round(view.state.doc.length / 4);
      headings = [];  // Skip full-tree parse for large files
    } else {
      wordCount = countWords(tab.content);
      headings = extractHeadings(view.state);
    }
    updateCursorInfo(view);
  }

  $effect(() => {
    const _tab = tabsStore.getPaneActiveTab(effectivePaneId);
    const _version = _tab?.version;
    const _zen = uiStore.zenMode;
    if (editorContainer) {
      loadTab();
    }
  });

  onMount(() => {
    loadTab();

    const autoSaveInterval = setInterval(async () => {
      for (const tab of tabsStore.allTabs) {
        if (!tab.isDirty) continue;
        tabsStore.syncFromView(tab.id);
        const freshTab = tabsStore.findByPath(tab.filePath);
        if (freshTab?.isDirty && freshTab.content) {
          await commands.registerWriteIgnore(freshTab.filePath);
          const result = await commands.writeFile(freshTab.filePath, freshTab.content);
          if (result.status === 'ok') tabsStore.markSaved(freshTab.id);
        }
      }
    }, 5 * 60 * 1000);

    return () => {
      clearInterval(autoSaveInterval);
      if (statsTimer) clearTimeout(statsTimer);
      cleanupCurrentView();
    };
  });
</script>

<div class="flex flex-col h-full w-full">
  {#if isReadOnly}
    <div class="shrink-0 flex items-center gap-2 px-3 py-1.5 text-xs" style="background: color-mix(in srgb, var(--novelist-accent) 15%, var(--novelist-bg)); color: var(--novelist-accent); border-bottom: 1px solid var(--novelist-border);">
      <span>Read-only — file is {(readOnlyFileSize / 1024 / 1024).toFixed(1)}MB. Use "Split into Chunks" to edit in smaller files.</span>
      <button
        class="px-2 py-0.5 rounded text-xs cursor-pointer"
        style="background: var(--novelist-accent); color: #fff;"
        onclick={splitIntoChunks}
      >Split into Chunks</button>
    </div>
  {/if}
  <div class="flex-1 min-h-0 w-full" bind:this={editorContainer}></div>
</div>

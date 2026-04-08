<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView } from '@codemirror/view';
  import { keymap } from '@codemirror/view';
  import type { Extension } from '@codemirror/state';
  import { createEditorExtensions, createEditorState } from '$lib/editor/setup';
  import { tabsStore } from '$lib/stores/tabs.svelte';
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

  const FILE_SIZE_WYSIWYG_LIMIT = 1024 * 1024; // 1MB

  let {
    paneId,
    wordCount = $bindable(0),
    cursorLine = $bindable(1),
    cursorCol = $bindable(1),
    headings = $bindable<HeadingItem[]>([]),
  }: Props = $props();

  // Effective pane id - falls back to active pane
  let effectivePaneId = $derived(paneId ?? tabsStore.activePaneId);

  function scrollToPosition(from: number) {
    if (!view) return;
    view.dispatch({
      selection: { anchor: from },
      scrollIntoView: true,
    });
    view.focus();
  }

  export { scrollToPosition };

  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;
  let currentTabId: string | null = null;
  let currentTabVersion: number = -1;
  let currentZenMode: boolean = false;

  function updateCursorInfo(v: EditorView) {
    const pos = v.state.selection.main.head;
    const line = v.state.doc.lineAt(pos);
    cursorLine = line.number;
    cursorCol = pos - line.from + 1;
  }

  function getActiveTab() {
    return tabsStore.getPaneActiveTab(effectivePaneId);
  }

  function buildExtensions(): Extension[] {
    const tab = getActiveTab();
    const fileEntry = projectStore.files.find(f => f.path === tab?.filePath);
    const useWysiwyg = !fileEntry || fileEntry.size < FILE_SIZE_WYSIWYG_LIMIT;

    if (!useWysiwyg) {
      console.log(`[Editor] File size ${fileEntry?.size} bytes exceeds ${FILE_SIZE_WYSIWYG_LIMIT} bytes limit; disabling WYSIWYG mode.`);
    }

    const base = createEditorExtensions({ wysiwyg: useWysiwyg, zen: uiStore.zenMode });
    return [
      ...base,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const text = update.state.doc.toString();
          wordCount = countWords(text);
          headings = extractHeadings(update.state);
          const t = getActiveTab();
          if (t) {
            tabsStore.updateContent(t.id, text);
          }
        }
        if (update.selectionSet || update.docChanged) {
          updateCursorInfo(update.view);
        }
      }),
      keymap.of([
        {
          key: 'Mod-s',
          run: () => {
            saveCurrentFile();
            return true;
          },
        },
      ]),
    ];
  }

  async function saveCurrentFile() {
    const tab = getActiveTab();
    if (!tab || !tab.isDirty) return;

    await commands.registerWriteIgnore(tab.filePath);
    const result = await commands.writeFile(tab.filePath, tab.content);
    if (result.status === 'ok') {
      tabsStore.markSaved(tab.id);
    } else {
      console.error('Failed to save file:', result.error);
    }
  }

  function loadTab() {
    const tab = getActiveTab();
    if (!tab) {
      if (view) {
        view.destroy();
        view = null;
        currentTabId = null;
        currentTabVersion = -1;
      }
      return;
    }

    // Rebuild view if tab changed, version changed, or zen mode toggled
    if (tab.id === currentTabId && tab.version === currentTabVersion && currentZenMode === uiStore.zenMode && view) return;

    currentTabId = tab.id;
    currentTabVersion = tab.version;
    currentZenMode = uiStore.zenMode;

    if (view) {
      view.destroy();
      view = null;
    }

    const extensions = buildExtensions();
    const state = createEditorState(tab.content, extensions);
    view = new EditorView({ state, parent: editorContainer });
    wordCount = countWords(tab.content);
    headings = extractHeadings(view.state);
    updateCursorInfo(view);
  }

  $effect(() => {
    // Track pane's active tab, version, and zen mode reactively
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
        if (tab.isDirty) {
          await commands.registerWriteIgnore(tab.filePath);
          const result = await commands.writeFile(tab.filePath, tab.content);
          if (result.status === 'ok') {
            tabsStore.markSaved(tab.id);
          }
        }
      }
    }, 5 * 60 * 1000); // 5 minutes

    return () => {
      clearInterval(autoSaveInterval);
      if (view) {
        view.destroy();
        view = null;
      }
    };
  });
</script>

<div class="h-full w-full overflow-hidden" bind:this={editorContainer}></div>

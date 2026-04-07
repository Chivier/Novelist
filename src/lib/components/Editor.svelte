<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView } from '@codemirror/view';
  import { keymap } from '@codemirror/view';
  import type { Extension } from '@codemirror/state';
  import { createEditorExtensions, createEditorState } from '$lib/editor/setup';
  import { tabsStore } from '$lib/stores/tabs.svelte';
  import { commands } from '$lib/ipc/commands';
  import { countWords } from '$lib/utils/wordcount';

  let wordCount = $state(0);
  let cursorLine = $state(1);
  let cursorCol = $state(1);

  export { wordCount, cursorLine, cursorCol };

  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;
  let currentTabId: string | null = null;

  function updateCursorInfo(v: EditorView) {
    const pos = v.state.selection.main.head;
    const line = v.state.doc.lineAt(pos);
    cursorLine = line.number;
    cursorCol = pos - line.from + 1;
  }

  function buildExtensions(): Extension[] {
    const base = createEditorExtensions();
    return [
      ...base,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const text = update.state.doc.toString();
          wordCount = countWords(text);
          const tab = tabsStore.activeTab;
          if (tab) {
            tabsStore.updateContent(tab.id, text);
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
    const tab = tabsStore.activeTab;
    if (!tab || !tab.isDirty) return;

    const result = await commands.writeFile(tab.filePath, tab.content);
    if (result.status === 'ok') {
      tabsStore.markSaved(tab.id);
    } else {
      console.error('Failed to save file:', result.error);
    }
  }

  function loadTab() {
    const tab = tabsStore.activeTab;
    if (!tab) {
      if (view) {
        view.destroy();
        view = null;
        currentTabId = null;
      }
      return;
    }

    if (tab.id === currentTabId && view) return;

    currentTabId = tab.id;

    if (view) {
      view.destroy();
      view = null;
    }

    const extensions = buildExtensions();
    const state = createEditorState(tab.content, extensions);
    view = new EditorView({ state, parent: editorContainer });
    wordCount = countWords(tab.content);
    updateCursorInfo(view);
  }

  $effect(() => {
    // Track activeTab reactively
    const _tab = tabsStore.activeTab;
    if (editorContainer) {
      loadTab();
    }
  });

  onMount(() => {
    loadTab();
    return () => {
      if (view) {
        view.destroy();
        view = null;
      }
    };
  });
</script>

<div class="h-full w-full overflow-hidden" bind:this={editorContainer}></div>

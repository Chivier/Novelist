import { type Range } from '@codemirror/state';
import { EditorView, ViewPlugin, type ViewUpdate, Decoration, type DecorationSet } from '@codemirror/view';

/**
 * Typewriter scrolling — keeps the cursor line vertically centered.
 * Uses instant scroll to avoid fighting with typing cadence.
 */
export const typewriterPlugin = ViewPlugin.fromClass(
  class {
    private raf: number | null = null;

    update(update: ViewUpdate) {
      if (update.selectionSet || update.docChanged) {
        // Coalesce multiple updates into a single scroll
        if (this.raf !== null) cancelAnimationFrame(this.raf);
        this.raf = requestAnimationFrame(() => {
          this.raf = null;
          const { view } = update;
          const head = view.state.selection.main.head;
          const coords = view.coordsAtPos(head);
          if (coords) {
            const editorRect = view.dom.getBoundingClientRect();
            const centerY = editorRect.top + editorRect.height / 2;
            const offset = coords.top - centerY;
            if (Math.abs(offset) > 10) {
              view.scrollDOM.scrollBy({ top: offset, behavior: 'instant' });
            }
          }
        });
      }
    }

    destroy() {
      if (this.raf !== null) cancelAnimationFrame(this.raf);
    }
  }
);

const dimLine = Decoration.line({ class: 'cm-novelist-zen-dim' });

/**
 * Paragraph focus — dims non-active paragraphs.
 * Only processes visible lines to avoid O(n) iteration on large documents.
 */
export const paragraphFocusPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    private pending: number | null = null;

    constructor(view: EditorView) {
      this.decorations = this.buildDim(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        // Debounce to avoid per-keystroke full rebuild
        if (this.pending !== null) cancelAnimationFrame(this.pending);
        this.pending = requestAnimationFrame(() => {
          this.pending = null;
          this.decorations = this.buildDim(update.view);
          update.view.requestMeasure();
        });
      }
    }

    buildDim(view: EditorView): DecorationSet {
      const { state } = view;
      const cursorLine = state.doc.lineAt(state.selection.main.head).number;
      const decos: Range<Decoration>[] = [];

      // Find current paragraph boundaries
      let paraStart = cursorLine;
      let paraEnd = cursorLine;
      while (paraStart > 1 && state.doc.line(paraStart - 1).text.trim() !== '') paraStart--;
      while (paraEnd < state.doc.lines && state.doc.line(paraEnd + 1).text.trim() !== '') paraEnd++;

      // Only dim VISIBLE lines outside the current paragraph
      for (const { from, to } of view.visibleRanges) {
        const startLine = state.doc.lineAt(from).number;
        const endLine = state.doc.lineAt(to).number;
        for (let i = startLine; i <= endLine; i++) {
          if (i < paraStart || i > paraEnd) {
            const line = state.doc.line(i);
            decos.push(dimLine.range(line.from));
          }
        }
      }

      return Decoration.set(decos, true);
    }

    destroy() {
      if (this.pending !== null) cancelAnimationFrame(this.pending);
    }
  },
  { decorations: (v) => v.decorations }
);

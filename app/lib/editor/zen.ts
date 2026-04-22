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
 * Line focus — dims every logical line except the cursor's current line.
 *
 * Optimizations:
 * - Caches the focused line number; skips rebuild if the cursor is still on it
 * - Does NOT call view.requestMeasure() (let CM6 schedule its own layout)
 * - Only processes visible lines
 */
export const lineFocusPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    private pending: number | null = null;
    private lastFocusedLine = -1;

    constructor(view: EditorView) {
      this.decorations = this.buildDim(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.lastFocusedLine = -1;
        this.scheduleBuild(update.view);
        return;
      }

      if (update.selectionSet) {
        const { state } = update.view;
        const cursorLine = state.doc.lineAt(state.selection.main.head).number;
        if (cursorLine === this.lastFocusedLine) return;
        this.scheduleBuild(update.view);
      }
    }

    private scheduleBuild(view: EditorView) {
      if (this.pending !== null) cancelAnimationFrame(this.pending);
      this.pending = requestAnimationFrame(() => {
        this.pending = null;
        this.decorations = this.buildDim(view);
      });
    }

    buildDim(view: EditorView): DecorationSet {
      const { state } = view;
      const cursorLine = state.doc.lineAt(state.selection.main.head).number;
      this.lastFocusedLine = cursorLine;

      const decos: Range<Decoration>[] = [];
      for (const { from, to } of view.visibleRanges) {
        const startLine = state.doc.lineAt(from).number;
        const endLine = state.doc.lineAt(to).number;
        for (let i = startLine; i <= endLine; i++) {
          if (i !== cursorLine) {
            decos.push(dimLine.range(state.doc.line(i).from));
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

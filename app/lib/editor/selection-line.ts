import { Decoration, type DecorationSet, EditorView, ViewPlugin, type ViewUpdate } from '@codemirror/view';
import { type Range } from '@codemirror/state';

const selectedLineDeco = Decoration.line({ class: 'cm-novelist-selected-line' });

function buildDecorations(view: EditorView): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const doc = view.state.doc;
  const seen = new Set<number>();

  for (const sel of view.state.selection.ranges) {
    if (sel.empty) continue;
    const fromLine = doc.lineAt(sel.from).number;
    const toLine = doc.lineAt(sel.to).number;
    for (let n = fromLine; n <= toLine; n++) {
      if (seen.has(n)) continue;
      seen.add(n);
      ranges.push(selectedLineDeco.range(doc.line(n).from));
    }
  }
  ranges.sort((a, b) => a.from - b.from);
  return Decoration.set(ranges);
}

export const unifiedLineSelectionPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }
    update(update: ViewUpdate) {
      if (update.selectionSet || update.docChanged || update.viewportChanged) {
        this.decorations = buildDecorations(update.view);
      }
    }
  },
  { decorations: (v) => v.decorations },
);

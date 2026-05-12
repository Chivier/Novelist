import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { markdown } from '@codemirror/lang-markdown';
import { computeCursorImageLine, cursorImageLineField } from '$lib/editor/wysiwyg';

/**
 * [precision] cursor-line image toggle.
 *
 * The image block widget collapses when the cursor enters an image line so
 * the user can edit the markdown source (e.g. fix a broken path). To avoid
 * mid-click height-map shifts (an old infinite-oscillation regression), the
 * toggle is suppressed during `select.pointer` transactions.
 *
 * These tests exercise `computeCursorImageLine` (pure derivation from the
 * syntax tree) and `cursorImageLineField` (transaction filtering + initial
 * state).
 */

function stateFrom(doc: string, selection?: { anchor: number; head?: number }): EditorState {
  return EditorState.create({
    doc,
    selection,
    extensions: [markdown(), cursorImageLineField],
  });
}

describe('[precision] computeCursorImageLine', () => {
  it('returns null for an empty document', () => {
    const state = stateFrom('');
    expect(computeCursorImageLine(state)).toBe(null);
  });

  it('returns null when cursor sits on a plain-text line', () => {
    const state = stateFrom('just prose\n\n![alt](a.png)', { anchor: 2 });
    expect(computeCursorImageLine(state)).toBe(null);
  });

  it('returns the line number when cursor sits on a line containing an Image node', () => {
    const doc = 'lead\n![alt](a.png)\ntrail';
    const imgLineStart = doc.indexOf('![');
    const state = stateFrom(doc, { anchor: imgLineStart + 3 }); // inside the alt text
    expect(computeCursorImageLine(state)).toBe(2);
  });

  it('returns null when cursor moves off the image line', () => {
    const doc = 'lead\n![alt](a.png)\ntrail';
    const trailIdx = doc.indexOf('trail');
    const state = stateFrom(doc, { anchor: trailIdx + 1 });
    expect(computeCursorImageLine(state)).toBe(null);
  });

  it('identifies the correct image line when multiple images are present', () => {
    const doc = '![a](a.png)\nmiddle\n![b](b.png)';
    const secondImgStart = doc.lastIndexOf('![');
    const state = stateFrom(doc, { anchor: secondImgStart });
    expect(computeCursorImageLine(state)).toBe(3);
  });

  it('detects inline images embedded in prose', () => {
    const doc = 'a sentence ![alt](broken.png) continues';
    const state = stateFrom(doc, { anchor: 0 });
    // Whole sentence + inline image lives on line 1
    expect(computeCursorImageLine(state)).toBe(1);
  });
});

describe('[precision] cursorImageLineField', () => {
  it('starts as null even when the document opens with cursor on an image line', () => {
    // Doc-open default: cursor at pos 0 of an image line. Initial value
    // MUST be null so the first image still renders on file open.
    const state = stateFrom('![alt](a.png)\nbody');
    expect(state.field(cursorImageLineField)).toBe(null);
  });

  it('updates to the image line number after a keyboard-style selection change', () => {
    const state = stateFrom('![alt](a.png)\nbody');
    const tr = state.update({ selection: { anchor: 3 } });
    expect(tr.state.field(cursorImageLineField)).toBe(1);
  });

  it('returns to null when the selection leaves the image line', () => {
    const doc = '![alt](a.png)\nbody';
    const onImage = stateFrom(doc).update({ selection: { anchor: 3 } }).state;
    expect(onImage.field(cursorImageLineField)).toBe(1);

    const offImage = onImage.update({ selection: { anchor: doc.indexOf('body') } }).state;
    expect(offImage.field(cursorImageLineField)).toBe(null);
  });

  it('does NOT update on a select.pointer transaction (mid-click guard)', () => {
    // Pointer-driven selection must not shift the height map between
    // mousedown and mouseup. The field stays at its prior value.
    const state = stateFrom('![alt](a.png)\nbody');
    expect(state.field(cursorImageLineField)).toBe(null);

    const tr = state.update({
      selection: { anchor: 3 },
      userEvent: 'select.pointer',
    });
    expect(tr.state.field(cursorImageLineField)).toBe(null);
  });

  it('still updates when a non-pointer userEvent moves selection onto an image line', () => {
    const state = stateFrom('![alt](a.png)\nbody');
    const tr = state.update({
      selection: { anchor: 3 },
      userEvent: 'select',
    });
    expect(tr.state.field(cursorImageLineField)).toBe(1);
  });

  it('recomputes after a doc-change transaction (e.g. typing inserts an image)', () => {
    const state = stateFrom('text\nbody', { anchor: 0 });
    expect(state.field(cursorImageLineField)).toBe(null);
    // Type an image at cursor (pos 0): now line 1 contains an Image node.
    const tr = state.update({
      changes: { from: 0, to: 4, insert: '![a](b.png)' },
      selection: { anchor: 0 },
    });
    expect(tr.state.field(cursorImageLineField)).toBe(1);
  });

  it('flips back to null when selection moves to a pointer-driven location after a keyboard move', () => {
    // After a keyboard move sets the field to 1, a subsequent select.pointer
    // off the image line must NOT clear it (still mid-click guard).
    const doc = '![alt](a.png)\nbody';
    const state = stateFrom(doc);
    const keyboardOn = state.update({ selection: { anchor: 3 }, userEvent: 'select' }).state;
    expect(keyboardOn.field(cursorImageLineField)).toBe(1);

    const pointerOff = keyboardOn.update({
      selection: { anchor: doc.indexOf('body') },
      userEvent: 'select.pointer',
    }).state;
    expect(pointerOff.field(cursorImageLineField)).toBe(1); // unchanged — pointer guard
  });
});

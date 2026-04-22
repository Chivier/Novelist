# Active Line Highlight + Single-Line Zen Focus

**Date:** 2026-04-22
**Scope:** Editor visuals.

Two small, independent UX tweaks to the CodeMirror 6 editor:

1. Subtle highlight for the cursor's active line, plus a dot marker in the line-number gutter.
2. Zen-mode paragraph focus becomes **single-line** focus (only the cursor's logical line stays bright; all others dim).

## Motivation

Today the active-line background is overridden to `transparent` (see
`app/lib/editor/setup.ts:289-295`) because the previous implementation's
tint collided with the 18% accent-tinted selection wash. We want the
reader cue back — but distinct from selection.

Zen mode currently keeps the **entire paragraph** (lines between blank
lines) bright and dims the rest. Users asked for the narrower,
typewriter-style *only the line I'm typing on* focus.

## Design

### Active line

| Layer | Style |
|---|---|
| `.cm-activeLine` | `background-color: color-mix(in srgb, var(--novelist-text) 4%, transparent)` |
| `.cm-activeLineGutter` | Line number color unchanged. A 4px accent-colored dot (`::before`) sits flush-left inside the gutter element, vertically centered. |

**Why this reads as "different" from selection:**
- Selection = accent hue (blue-ish), 18% alpha. Active line = neutral grey, 4% alpha. Different hue *and* intensity.
- Selection paints the full line via `.cm-novelist-selected-line` (specificity `0,2,1`) which still wins over `.cm-activeLine` (`0,1,1`) when both apply — no double-tint.
- Gutter dot uses accent color but lives in the gutter, not the text region, so it cannot be confused with a selection rectangle.

### Zen single-line focus

Modify `paragraphFocusPlugin` in `app/lib/editor/zen.ts`:

- Remove the two `while` loops that expand `paraStart`/`paraEnd` to blank-line boundaries.
- Keep `paraStart = paraEnd = cursorLine`. The existing `lastParaStart/lastParaEnd` cache still works — it just becomes a one-line range.
- Rename the export to `lineFocusPlugin` (behavior-accurate). Update the single import site (`app/lib/editor/setup.ts`) and the test import (`tests/unit/editor/zen.test.ts`).

The `cm-novelist-zen-dim` CSS class stays as-is; only the set of dimmed lines changes.

## Non-goals

- No change to typewriter scrolling (`typewriterPlugin`).
- No change to ZenMode overlay chrome (HUD, exit button, dark background).
- No change to selection styling.

## Testing

- Existing `tests/unit/editor/zen.test.ts` imports update to `lineFocusPlugin`; structural assertions (is-a-ViewPlugin, builds decorations, survives updates) still apply.
- No new unit coverage: active-line CSS is pure theme, and the zen behavior change is a one-line edit that the existing structural tests cover.
- Manual check in `pnpm tauri dev`: cursor moves → correct line highlights, gutter dot appears; in zen mode, only the cursor's single line is bright.

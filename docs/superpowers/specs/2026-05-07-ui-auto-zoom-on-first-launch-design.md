# UI Auto-Zoom on First Launch — Design

**Status:** Approved 2026-05-07
**Owner:** Chivier
**Scope:** `app/lib/stores/ui.svelte.ts` (+ unit test)

## Problem

On high-DPI displays the UI chrome (sidebar, settings, dialogs, status bar)
feels too small. The Settings panel only exposes editor body font size — UI
font sizes are hardcoded in `app/app.css` (`html { font-size: 13px }`) and
across components, so users can't reach them without manually pressing
`Cmd +`. Users on 4K and Retina monitors hit this immediately.

## Decision

Auto-pick a sensible initial `zoomLevel` once on first launch based on the
physical pixel width of the primary screen. Reuse the existing `Cmd +/-`
zoom (CSS `transform: scale()`) — that's exactly what makes the hardcoded
`px` literals scale, so it stays as the implementation. No new settings
panel entry, no new keyboard shortcut.

## Non-goals

- No separate "UI font size" setting — `Cmd +/-` remains the single knob.
- No replacement of `transform: scale()` with `html` root font-size +
  rem audit. The blurriness tradeoff is real but out of scope here.
- No display-change listener, no per-launch recompute. First launch only.

## Detection

```
physicalWidth = screen.width * devicePixelRatio
```

Picks:

| physicalWidth | zoomLevel |
|---------------|-----------|
| ≥ 3840 (4K)   | 1.5       |
| ≥ 2560 (2K)   | 1.25      |
| otherwise     | 1.0       |

Why `screen.width` not `window.innerWidth`: the latter shrinks with the
window. Why multiply by DPR: avoids being fooled by OS-level logical
scaling (a 4K monitor at 2× reports `screen.width === 1920`).

A 1920×1080 Retina laptop reports `physicalWidth === 3840` and lands on
1.5×. That is the desired behavior — its OS is already doing 2× scaling
to make UI legible, and our `transform: scale(1.5)` lines up with that.

## First-launch trigger

New localStorage key: `novelist-zoom-auto-inited` (set to `"1"` after the
one-time pick).

Order at module init in `ui.svelte.ts`:

1. If `novelist-zoom-auto-inited` exists → skip.
2. Else if `novelist-zoom` already exists (existing v0.2.4-and-earlier
   user who set zoom manually) → write `auto-inited=1`, do nothing else.
   We don't want to override their explicit choice with our default.
3. Else compute `level` from `physicalWidth`, call `setZoom(level)` (which
   writes `novelist-zoom`), then write `auto-inited=1`.

Once `auto-inited=1` is set, `setZoom`/`Cmd +/-`/`Cmd+0` work exactly as
before. Resetting to default behavior across migrations is not a concern —
clearing localStorage gives users a fresh auto-pick.

## Code shape

```ts
// New, testable, pure:
export function pickAutoZoomFromScreen(physicalWidth: number): number {
  if (physicalWidth >= 3840) return 1.5;
  if (physicalWidth >= 2560) return 1.25;
  return 1.0;
}

// New, side-effecting, runs once at module init:
function autoInitZoomFromScreen() {
  if (localStorage.getItem('novelist-zoom-auto-inited')) return;
  if (localStorage.getItem('novelist-zoom')) {
    localStorage.setItem('novelist-zoom-auto-inited', '1');
    return;
  }
  const physicalWidth = (window.screen?.width ?? 0) * (window.devicePixelRatio || 1);
  const level = pickAutoZoomFromScreen(physicalWidth);
  uiStore.setZoom(level);
  localStorage.setItem('novelist-zoom-auto-inited', '1');
}
```

Called from the existing `if (typeof document !== 'undefined') { … }`
block, **before** the existing `if (uiStore.zoomLevel !== 1) setZoom(...)`
line — so the auto-picked level is what gets applied.

## Tests

Add to `tests/unit/stores/ui.test.ts`:

- `pickAutoZoomFromScreen` returns 1.0 / 1.25 / 1.5 at boundary widths
  (1920, 2559, 2560, 3839, 3840, 5120).
- An `autoInitZoomFromScreen` helper test (export it for testability):
  - Fresh storage + `physicalWidth=3840` → `novelist-zoom='1.5'` and
    `novelist-zoom-auto-inited='1'`.
  - With `novelist-zoom-auto-inited='1'` already set → no change.
  - With `novelist-zoom='0.8'` already set (existing user) → zoom
    untouched, `auto-inited` written.

The helper takes `physicalWidth` as a parameter (or reads `screen` lazily)
so tests don't need to monkey-patch `window.screen`.

## Risks

- A user on a low-DPR ultrawide (e.g. 3440×1440 @ DPR=1, physicalWidth
  3440) lands on 1.25× — likely fine, but worth noting.
- If a user clears localStorage to "fix" something, they'll get the
  auto-pick again on next launch. Acceptable.
- Multi-monitor: `screen.width` is the primary monitor; if the app
  launches on a secondary monitor with different DPR the pick may be
  off. Out of scope — they can `Cmd +/-`.

## Out of scope (follow-up candidates)

- Replace `transform: scale()` with root font-size + rem audit (removes
  blurriness + the `width = 100/zoom%` hack).
- Expose a UI scale dropdown in Settings.
- Listen for display changes and re-pick.

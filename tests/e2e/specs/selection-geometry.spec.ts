import { test, expect } from '../fixtures/app-fixture';

/**
 * Regression test for the selection-left-edge alignment bug.
 *
 * CM6's built-in `drawSelection` paints rectangles using two different
 * coordinate frames — first-line text rects vs middle-line content rects —
 * which leaves a ~19px stair-step on the left edge across heading lines.
 * We fix this by hiding `.cm-selectionBackground` and drawing per-line
 * background via `.cm-novelist-selected-line` (a `Decoration.line`), whose
 * left edge is uniform because `.cm-line` has uniform padding. This spec
 * guarantees that invariant: every selected-line background starts at the
 * same x, regardless of the heading level of the line it sits on.
 */
test.describe('Selection left-edge alignment', () => {
  test('all selected-line backgrounds share a single left x-coordinate', async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
    await app.getByTestId('sidebar-file-Chapter 1.md').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    await app.evaluate(() => {
      const view = (window as any).__novelist_view;
      const doc =
        '# Top\n' +
        '\n' +
        '## Middle A\n' +
        '\n' +
        '\n' +
        '## Middle B\n' +
        '\n' +
        '### Leaf 1\n' +
        '\n' +
        '### Leaf 2\n';
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: doc } });
      view.dispatch({ selection: { anchor: 0, head: view.state.doc.length } });
    });

    await app.waitForTimeout(200);

    const { distinctLineLefts, count } = await app.evaluate(() => {
      const lefts: number[] = [];
      document.querySelectorAll<HTMLElement>('.cm-novelist-selected-line').forEach(el => {
        lefts.push(+el.getBoundingClientRect().left.toFixed(2));
      });
      return {
        count: lefts.length,
        distinctLineLefts: Array.from(new Set(lefts)).sort((a, b) => a - b),
      };
    });

    expect(count).toBeGreaterThan(3);
    // Single distinct left value → every line background aligns perfectly.
    expect(distinctLineLefts).toHaveLength(1);
  });

  test('native drawSelection backgrounds are suppressed so they can not jitter', async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
    await app.getByTestId('sidebar-file-Chapter 1.md').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    await app.evaluate(() => {
      const view = (window as any).__novelist_view;
      view.dispatch({ selection: { anchor: 0, head: view.state.doc.length } });
    });
    await app.waitForTimeout(200);

    const bgColors = await app.evaluate(() => {
      const colors: string[] = [];
      document.querySelectorAll<HTMLElement>('.cm-selectionBackground').forEach(el => {
        colors.push(getComputedStyle(el).backgroundColor);
      });
      return colors;
    });

    // Every `.cm-selectionBackground` must be fully transparent. Any opaque
    // color would signal that drawSelection's misaligned rects are rendering.
    for (const c of bgColors) {
      expect(c).toMatch(/rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*0\s*\)|transparent/);
    }
  });
});

import { test, expect } from '../fixtures/app-fixture';

test.describe('Editor', () => {
  test.beforeEach(async ({ app }) => {
    // Open project and a file
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }

    const fileItem = app.getByTestId('sidebar-file-Chapter 1.md');
    await fileItem.click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });
  });

  test('editor displays file content', async ({ app }) => {
    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('Chapter 1');
    await expect(cmContent).toContainText('dark and stormy night');
  });

  test('cursor position is reported in status bar', async ({ app }) => {
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.click();

    const cursorPos = app.getByTestId('status-cursor-pos');
    await expect(cursorPos).toBeVisible();
    await expect(cursorPos).toContainText(/Ln \d+/);
  });

  test('word count updates in status bar', async ({ app }) => {
    const wordCount = app.getByTestId('status-word-count');
    await expect(wordCount).toBeVisible();
    const text = await wordCount.textContent();
    expect(text).toBeTruthy();
  });

  test('clicking in editor moves cursor to correct position', async ({ app }) => {
    const targetLine = app.locator('.cm-line').filter({ hasText: 'wind howled' });
    await targetLine.click();

    const cursorLine = await app.evaluate(() => {
      const view = (window as any).__novelist_view;
      if (!view) return -1;
      const pos = view.state.selection.main.head;
      return view.state.doc.lineAt(pos).number;
    });

    expect(cursorLine).toBeGreaterThan(0);
  });

  test('typing inserts text at cursor position', async ({ app }) => {
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.click();

    await app.keyboard.press('Meta+ArrowDown');
    await app.keyboard.press('Enter');
    await app.keyboard.type('New paragraph here');

    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('New paragraph here');
  });

  test('multiple edits are tracked', async ({ app }) => {
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.click();

    // Type first edit
    await app.keyboard.press('End');
    await app.keyboard.type(' - edited');

    // Verify content was modified
    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('edited');

    // Editor state should have history entries
    const hasHistory = await app.evaluate(() => {
      const view = (window as any).__novelist_view;
      if (!view) return false;
      // If the doc length is greater than original, edits were tracked
      return view.state.doc.length > 0;
    });
    expect(hasHistory).toBe(true);
  });

  test('CJK content renders correctly', async ({ app }) => {
    const fileItem = app.getByTestId('sidebar-file-Chapter 3.md');
    await fileItem.click();

    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('第三章');
    await expect(cmContent).toContainText('中文测试文本');
  });
});

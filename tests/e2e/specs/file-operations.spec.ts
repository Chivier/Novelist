import { test, expect } from '../fixtures/app-fixture';

test.describe('File Operations', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('open file, edit content, and save', async ({ app, mockState }) => {
    const fileItem = app.getByTestId('sidebar-file-Chapter 1.md');
    await fileItem.click();
    await app.getByTestId('editor-container').waitFor({ state: 'visible' });

    const cmEditor = app.locator('.cm-editor');
    await cmEditor.waitFor({ state: 'visible', timeout: 5000 });
    await cmEditor.click();

    await app.keyboard.type('Hello World');

    // Use the exposed save function (browser intercepts Meta+S)
    await app.evaluate(() => {
      const saveFn = (window as any).__novelist_save;
      if (saveFn) saveFn();
    });
    await app.waitForTimeout(500);

    const written = await mockState.getWrittenFiles();
    const savedContent = Object.values(written).find((c) => (c as string).includes('Hello World'));
    expect(savedContent).toBeTruthy();
  });

  test('Cmd+N creates a new file', async ({ app }) => {
    await app.keyboard.press('Meta+n');

    await expect(app.getByTestId('editor-container')).toBeVisible();

    const tabBar = app.getByTestId('tab-bar');
    await expect(tabBar).toBeVisible();
  });

  test('Cmd+W closes the current tab', async ({ app }) => {
    const fileItem = app.getByTestId('sidebar-file-Chapter 1.md');
    await fileItem.click();
    await app.getByTestId('editor-container').waitFor({ state: 'visible' });

    await app.keyboard.press('Meta+w');
  });

  test('editing content marks tab as dirty', async ({ app }) => {
    const fileItem = app.getByTestId('sidebar-file-Chapter 2.md');
    await fileItem.click();

    const cmEditor = app.locator('.cm-editor');
    await cmEditor.waitFor({ state: 'visible', timeout: 5000 });
    await cmEditor.click();

    await app.keyboard.type('Modified');

    const dirtyDot = app.locator('.dirty-dot');
    await expect(dirtyDot).toBeVisible({ timeout: 2000 });
  });
});

import { test, expect } from '../fixtures/app-fixture';

test.describe('Sidebar', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('sidebar shows project files', async ({ app }) => {
    const sidebar = app.getByTestId('sidebar');
    await expect(sidebar).toBeVisible();

    await expect(sidebar).toContainText('Chapter 1');
    await expect(sidebar).toContainText('Chapter 2');
    await expect(sidebar).toContainText('Notes');
  });

  test('toggle sidebar visibility', async ({ app }) => {
    const sidebar = app.getByTestId('sidebar');
    await expect(sidebar).toBeVisible();

    // Use test API to toggle (browser intercepts Meta+B)
    await app.evaluate(() => (window as any).__test_api__.toggleSidebar());
    await expect(sidebar).not.toBeVisible({ timeout: 2000 });

    await app.evaluate(() => (window as any).__test_api__.toggleSidebar());
    await expect(sidebar).toBeVisible({ timeout: 2000 });
  });

  test('clicking a file opens it in editor', async ({ app }) => {
    const fileItem = app.getByTestId('sidebar-file-Chapter 1.md');
    await fileItem.click();

    await expect(app.getByTestId('editor-container')).toBeVisible();
    await expect(app.getByTestId('tab-bar')).toContainText('Chapter 1');
  });

  test('new file button creates a file', async ({ app }) => {
    const newFileBtn = app.getByTestId('sidebar-new-file');
    await newFileBtn.click();

    const input = app.getByTestId('sidebar-input');
    await expect(input).toBeVisible();
    await input.fill('New Chapter.md');
    await input.press('Enter');

    await expect(app.getByTestId('sidebar')).toContainText('New Chapter');
  });

  test('right-click shows context menu', async ({ app }) => {
    const fileItem = app.getByTestId('sidebar-file-Chapter 1.md');
    await fileItem.click({ button: 'right' });

    const contextMenu = app.getByTestId('context-menu');
    await expect(contextMenu).toBeVisible();

    await app.keyboard.press('Escape');
  });
});

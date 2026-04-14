import { test, expect } from '../fixtures/app-fixture';

test.describe('Split View', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
    await app.getByTestId('sidebar-file-Chapter 1.md').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });
  });

  test('toggling split view shows second pane', async ({ app }) => {
    // Before split: only one tab bar
    const tabBars = app.getByTestId('tab-bar');
    await expect(tabBars).toHaveCount(1);

    // Enable split view
    await app.evaluate(() => (window as any).__test_api__.toggleSplit());
    await app.waitForTimeout(300);

    // After split: two tab bars (one per pane)
    await expect(app.getByTestId('tab-bar')).toHaveCount(2, { timeout: 2000 });

    // Disable split view
    await app.evaluate(() => (window as any).__test_api__.toggleSplit());
    await app.waitForTimeout(300);

    // Back to one tab bar
    await expect(app.getByTestId('tab-bar')).toHaveCount(1, { timeout: 2000 });
  });
});

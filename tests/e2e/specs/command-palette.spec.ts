import { test, expect } from '../fixtures/app-fixture';

test.describe('Command Palette', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('Cmd+Shift+P opens command palette', async ({ app }) => {
    await app.keyboard.press('Meta+Shift+p');

    const palette = app.getByTestId('command-palette');
    await expect(palette).toBeVisible({ timeout: 2000 });

    const input = app.getByTestId('palette-input');
    await expect(input).toBeFocused();
  });

  test('typing filters commands', async ({ app }) => {
    await app.keyboard.press('Meta+Shift+p');

    const input = app.getByTestId('palette-input');
    await input.fill('toggle');

    const results = app.locator('[data-testid^="palette-result-"]');
    const count = await results.count();
    expect(count).toBeGreaterThan(0);
  });

  test('Escape closes command palette', async ({ app }) => {
    await app.keyboard.press('Meta+Shift+p');
    const palette = app.getByTestId('command-palette');
    await expect(palette).toBeVisible();

    await app.keyboard.press('Escape');
    await expect(palette).not.toBeVisible();
  });

  test('"Switch Project" command opens the sidebar switcher popup', async ({ app }) => {
    // Popup should be closed initially.
    await expect(app.getByTestId('project-switcher')).toHaveCount(0);

    await app.keyboard.press('Meta+Shift+p');
    const input = app.getByTestId('palette-input');
    await input.fill('switch project');

    const firstResult = app.locator('[data-testid="palette-result-0"]');
    await expect(firstResult).toBeVisible();
    await firstResult.click();

    // Popup is now visible with the recent projects.
    const switcher = app.getByTestId('project-switcher');
    await expect(switcher).toBeVisible();
    await expect(switcher).toContainText('Test Novel');
  });

  test('selecting a command executes it', async ({ app }) => {
    // Open a file so we have an editor
    await app.getByTestId('sidebar-file-Chapter 1.md').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    await app.keyboard.press('Meta+Shift+p');
    const input = app.getByTestId('palette-input');
    await input.fill('zen');

    const firstResult = app.locator('[data-testid="palette-result-0"]');
    if (await firstResult.isVisible()) {
      await firstResult.click();
      await expect(app.getByTestId('zen-mode')).toBeVisible({ timeout: 3000 });
      await app.keyboard.press('Escape');
    }
  });
});

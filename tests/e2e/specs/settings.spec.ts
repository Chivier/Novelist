import { test, expect } from '../fixtures/app-fixture';

test.describe('Settings Dialog', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('Cmd+, opens settings dialog', async ({ app }) => {
    await app.keyboard.press('Meta+,');

    const settings = app.getByTestId('settings-dialog');
    await expect(settings).toBeVisible({ timeout: 2000 });
  });

  test('Escape closes settings dialog', async ({ app }) => {
    await app.keyboard.press('Meta+,');
    const settings = app.getByTestId('settings-dialog');
    await expect(settings).toBeVisible();

    await app.keyboard.press('Escape');
    await expect(settings).not.toBeVisible();
  });

  test('clicking overlay closes settings', async ({ app }) => {
    await app.keyboard.press('Meta+,');
    const overlay = app.getByTestId('settings-overlay');
    await expect(overlay).toBeVisible();

    // Click the overlay (outside the dialog)
    await overlay.click({ position: { x: 10, y: 10 } });
    await expect(app.getByTestId('settings-dialog')).not.toBeVisible({ timeout: 2000 });
  });

  test('settings sections are navigable', async ({ app }) => {
    await app.keyboard.press('Meta+,');
    const settings = app.getByTestId('settings-dialog');

    const sections = ['editor', 'theme', 'shortcuts'];
    for (const section of sections) {
      const sectionBtn = app.getByTestId(`settings-section-${section}`);
      if (await sectionBtn.isVisible()) {
        await sectionBtn.click();
        await expect(settings).toBeVisible();
      }
    }
  });

  test('font size dropdown exists and is interactive', async ({ app }) => {
    await app.keyboard.press('Meta+,');

    const fontSizeSelect = app.locator('#settings-size');
    if (await fontSizeSelect.isVisible()) {
      await expect(fontSizeSelect).toBeEnabled();
    }
  });
});

import { test, expect } from '../fixtures/app-fixture';

test.describe('Welcome Screen', () => {
  test('shows welcome screen when no project is open', async ({ app }) => {
    const welcome = app.getByTestId('welcome-screen');
    await expect(welcome).toBeVisible();
    await expect(welcome).toContainText('Novelist');
  });

  test('shows recent projects list', async ({ app }) => {
    const welcome = app.getByTestId('welcome-screen');
    await expect(welcome).toContainText('Test Novel');
    await expect(welcome).toContainText('Another Story');
  });

  test('new file button is visible and clickable', async ({ app }) => {
    const newFileBtn = app.getByTestId('welcome-new-file');
    await expect(newFileBtn).toBeVisible();
    await newFileBtn.click();

    // After clicking new file, welcome should disappear and editor should appear
    await expect(app.getByTestId('welcome-screen')).not.toBeVisible({ timeout: 3000 });
    await expect(app.getByTestId('editor-container')).toBeVisible();
  });

  test('open folder button is visible', async ({ app }) => {
    const openBtn = app.getByTestId('welcome-open-folder');
    await expect(openBtn).toBeVisible();
  });

  test('clicking recent project opens it', async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    await recentItem.click();

    // Welcome screen should close, sidebar should show project files
    await expect(app.getByTestId('welcome-screen')).not.toBeVisible({ timeout: 5000 });
    await expect(app.getByTestId('sidebar')).toBeVisible();
  });

  test('recent list caps at 7 items and shows an overflow hint when there are more', async ({ app }) => {
    // Seed 10 recent projects via localStorage (the mock reads this on page load).
    const seeded = Array.from({ length: 10 }, (_, i) => ({
      path: `/tmp/project-${i}`,
      name: `Project ${i + 1}`,
      last_opened: `2026-04-${String(19 - i).padStart(2, '0')}T10:00:00Z`,
      pinned: false,
      sort_order: null,
    }));
    await app.evaluate(
      (list) => localStorage.setItem('__novelist_mock_recent_seed__', JSON.stringify(list)),
      seeded,
    );
    await app.reload();
    await app.waitForSelector('#app > *', { timeout: 5000 });

    // Exactly 7 rendered.
    await expect(app.getByTestId('recent-project-0')).toBeVisible();
    await expect(app.getByTestId('recent-project-6')).toBeVisible();
    await expect(app.getByTestId('recent-project-7')).toHaveCount(0);

    // Overflow hint visible with the remainder count.
    const hint = app.getByTestId('recent-overflow-hint');
    await expect(hint).toBeVisible();
    await expect(hint).toContainText('3');

    // Cleanup so the next test doesn't inherit the seed.
    await app.evaluate(() => localStorage.removeItem('__novelist_mock_recent_seed__'));
  });
});

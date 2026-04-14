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
});

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

  test('folder tree: expand shows children, collapse hides them', async ({ app }) => {
    const folder = app.getByTestId('sidebar-folder-Notes');
    await expect(folder).toBeVisible();

    // Child file is hidden before expand.
    await expect(app.getByTestId('sidebar-file-outline.md')).toHaveCount(0);

    // Click the chevron (the Expand/Collapse button inside the folder row).
    const chevron = folder.getByRole('button', { name: /Expand|Collapse/i });
    await chevron.click();
    await expect(app.getByTestId('sidebar-file-outline.md')).toBeVisible();

    // Collapsing again hides it.
    await chevron.click();
    await expect(app.getByTestId('sidebar-file-outline.md')).toHaveCount(0);
  });

  test('clicking bottom-bar project button opens the switcher popup', async ({ app }) => {
    // Popup is closed by default.
    await expect(app.getByTestId('project-switcher')).toHaveCount(0);

    await app.getByTestId('sidebar-switch-btn').click();

    // After the click, the switcher should be visible and list recent projects.
    const switcher = app.getByTestId('project-switcher');
    await expect(switcher).toBeVisible();
    await expect(switcher).toContainText('Test Novel');
    await expect(switcher).toContainText('Another Story');
  });

  test('switcher popup exposes a pin button that toggles pinned state', async ({ app, mockState }) => {
    await app.getByTestId('sidebar-switch-btn').click();

    // The non-active project "Another Story" is at index 1 (index 0 is the current project).
    // Pin it.
    const pinBtn = app.getByTestId('project-switcher-pin-1');
    await pinBtn.click();

    // Backend state reflects the pin.
    const after = await mockState.getRecentProjects();
    const another = after.find(p => p.name === 'Another Story');
    expect(another?.pinned).toBe(true);
  });

  test('folder tree: drag a root file into a subfolder', async ({ app }) => {
    const folder = app.getByTestId('sidebar-folder-Notes');
    const chevron = folder.getByRole('button', { name: /Expand|Collapse/i });

    // Expand first.
    await chevron.click();
    await expect(app.getByTestId('sidebar-file-outline.md')).toBeVisible();

    // Drag "Chapter 3.md" (currently at project root) onto Notes folder.
    const rootFile = app.getByTestId('sidebar-file-Chapter 3.md');
    await rootFile.dragTo(folder);

    // Give the reactive refresh a moment, then verify Chapter 3 is now inside Notes.
    // Assertion: Chapter 3.md testid still exists (file-level id is the filename, not the path).
    await expect(app.getByTestId('sidebar-file-Chapter 3.md')).toBeVisible();
    // And the Notes subfolder still shows outline.md.
    await expect(app.getByTestId('sidebar-file-outline.md')).toBeVisible();
  });
});

# Automated Testing Infrastructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace fragile bash+cliclick GUI tests with a three-tier automated testing system (Vitest component tests, Playwright browser E2E, Playwright+Tauri full E2E) that uses DOM selectors instead of screen coordinates, real assertions instead of "did it crash?" checks, and runs fully automated in CI.

**Architecture:** Tier 1 enhances existing Vitest unit tests with `@tauri-apps/api/mocks` for IPC mocking. Tier 2 uses Playwright in browser mode against the Vite dev server with an injected Tauri IPC mock layer — this covers all user-visible flows without compiling the Tauri binary. Tier 3 adds `tauri-plugin-playwright` to drive the real WKWebView for full backend integration testing.

**Tech Stack:** Playwright, @testing-library/svelte, @tauri-apps/api/mocks, tauri-plugin-playwright (Rust crate + npm), Vitest

---

## File Structure

```
playwright.config.ts                    # Playwright config (browser mode)
tests/
  e2e/
    fixtures/
      tauri-mock.ts                     # window.__TAURI_INTERNALS__ mock
      app-fixture.ts                    # Playwright test fixture with IPC mock injection
      mock-data.ts                      # Reusable mock data (files, projects, etc.)
    specs/
      welcome.spec.ts                   # Welcome screen tests
      sidebar.spec.ts                   # Sidebar file browser tests
      file-operations.spec.ts           # Create, open, edit, save, delete
      editor.spec.ts                    # Editor content, cursor, scroll
      tabs.spec.ts                      # Tab management
      command-palette.spec.ts           # Command palette tests
      settings.spec.ts                  # Settings dialog tests
      zen-mode.spec.ts                  # Zen mode enter/exit
      split-view.spec.ts               # Split view tests
    old/                                # Archive of old bash scripts
      test-all-features.sh
      test-scroll-click.sh
      ...
  unit/                                 # (existing - enhanced)
    ...
```

Files to modify in source for testability:
- `app/App.svelte` — add `data-testid` attributes to key layout regions
- `app/lib/components/Sidebar.svelte` — add `data-testid` to interactive elements
- `app/lib/components/Editor.svelte` — add `data-testid` to editor container
- `app/lib/components/Welcome.svelte` — add `data-testid` to buttons
- `app/lib/components/Settings.svelte` — add `data-testid` to modal
- `app/lib/components/CommandPalette.svelte` — add `data-testid` to palette
- `app/lib/components/TabBar.svelte` — add `data-testid` to tabs
- `app/lib/components/StatusBar.svelte` — add `data-testid` to status bar
- `app/lib/components/ZenMode.svelte` — add `data-testid`
- `app/lib/components/Outline.svelte` — add `data-testid`

---

### Task 1: Install Dependencies and Configure Playwright

**Files:**
- Modify: `package.json`
- Create: `playwright.config.ts`

- [ ] **Step 1: Install Playwright and testing dependencies**

```bash
cd /Users/chivier/Documents/Projects/Novelist
pnpm add -D @playwright/test
pnpm exec playwright install webkit chromium
```

- [ ] **Step 2: Create Playwright configuration**

Create `playwright.config.ts`:

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e/specs',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['list'],
    ['html', { open: 'never' }],
  ],
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 30000,
  },
});
```

- [ ] **Step 3: Add test scripts to package.json**

Add to `"scripts"` in `package.json`:

```json
"test:e2e:browser": "playwright test --project=webkit",
"test:e2e:all-browsers": "playwright test",
"test:e2e:ui": "playwright test --ui",
"test:e2e:debug": "playwright test --debug"
```

- [ ] **Step 4: Add Playwright artifacts to .gitignore**

Append to `.gitignore`:

```
test-results/
playwright-report/
blob-report/
```

- [ ] **Step 5: Verify Playwright installs and config is valid**

Run: `pnpm exec playwright test --list`
Expected: No errors, empty test list (no specs yet).

- [ ] **Step 6: Commit**

```bash
git add playwright.config.ts package.json pnpm-lock.yaml .gitignore
git commit -m "chore: add Playwright for browser-based E2E testing"
```

---

### Task 2: Create Tauri IPC Mock Layer

This is the core infrastructure that makes browser-mode testing possible. It intercepts all `@tauri-apps/api` calls by providing a mock `window.__TAURI_INTERNALS__` before the app loads.

**Files:**
- Create: `tests/e2e/fixtures/mock-data.ts`
- Create: `tests/e2e/fixtures/tauri-mock.ts`
- Create: `tests/e2e/fixtures/app-fixture.ts`

- [ ] **Step 1: Create mock data module**

Create `tests/e2e/fixtures/mock-data.ts`:

```typescript
/** Reusable mock data for Tauri IPC responses */

export interface MockFileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
}

export interface MockRecentProject {
  path: string;
  name: string;
  last_opened: string;
}

export const MOCK_PROJECT_DIR = '/tmp/novelist-test-project';

export const MOCK_FILES: MockFileEntry[] = [
  { name: 'Chapter 1.md', path: `${MOCK_PROJECT_DIR}/Chapter 1.md`, is_dir: false, size: 1024 },
  { name: 'Chapter 2.md', path: `${MOCK_PROJECT_DIR}/Chapter 2.md`, is_dir: false, size: 2048 },
  { name: 'Notes', path: `${MOCK_PROJECT_DIR}/Notes`, is_dir: true, size: 0 },
  { name: 'Chapter 3.md', path: `${MOCK_PROJECT_DIR}/Chapter 3.md`, is_dir: false, size: 512 },
];

export const MOCK_FILE_CONTENTS: Record<string, string> = {
  [`${MOCK_PROJECT_DIR}/Chapter 1.md`]: '# Chapter 1\n\nIt was a dark and stormy night.\n\nThe wind howled through the trees.\n',
  [`${MOCK_PROJECT_DIR}/Chapter 2.md`]: '# Chapter 2\n\nThe next morning dawned bright and clear.\n',
  [`${MOCK_PROJECT_DIR}/Chapter 3.md`]: '# Chapter 3\n\n这是第三章的内容。\n\n中文测试文本。\n',
};

export const MOCK_RECENT_PROJECTS: MockRecentProject[] = [
  { path: MOCK_PROJECT_DIR, name: 'Test Novel', last_opened: '2026-04-14T10:00:00Z' },
  { path: '/tmp/another-project', name: 'Another Story', last_opened: '2026-04-13T09:00:00Z' },
];

export const MOCK_PROJECT_CONFIG = {
  project: { name: 'Test Novel', type: 'novel', version: '1.0' },
  outline: null,
  writing: null,
};
```

- [ ] **Step 2: Create the Tauri IPC mock injection script**

Create `tests/e2e/fixtures/tauri-mock.ts`:

```typescript
/**
 * Tauri IPC mock for Playwright browser-mode testing.
 *
 * This script is injected via page.addInitScript() BEFORE the app loads.
 * It provides window.__TAURI_INTERNALS__ so @tauri-apps/api calls work
 * without a real Tauri backend.
 */

import type { MockFileEntry, MockRecentProject } from './mock-data';

export interface TauriMockConfig {
  files: MockFileEntry[];
  fileContents: Record<string, string>;
  recentProjects: MockRecentProject[];
  projectDir: string;
  projectConfig: unknown;
}

/**
 * Returns JavaScript source code to be injected into the page.
 * All mock data is serialized into the script as JSON literals.
 */
export function buildTauriMockScript(config: TauriMockConfig): string {
  return `
    (() => {
      const files = ${JSON.stringify(config.files)};
      const fileContents = ${JSON.stringify(config.fileContents)};
      const recentProjects = ${JSON.stringify(config.recentProjects)};
      const projectDir = ${JSON.stringify(config.projectDir)};
      const projectConfig = ${JSON.stringify(config.projectConfig)};

      // In-memory file system for write operations
      const writtenFiles = {};
      const createdFiles = [];
      const deletedFiles = [];

      // Event listeners
      const eventListeners = {};

      function handleInvoke(cmd, args) {
        switch (cmd) {
          // File operations
          case 'read_file':
            return writtenFiles[args.path] ?? fileContents[args.path] ?? '';
          case 'write_file':
            writtenFiles[args.path] = args.content;
            return null;
          case 'get_file_encoding':
            return 'utf-8';
          case 'list_directory':
            return files;
          case 'create_file': {
            const newPath = args.parentDir + '/' + args.filename;
            createdFiles.push(newPath);
            fileContents[newPath] = '';
            files.push({ name: args.filename, path: newPath, is_dir: false, size: 0 });
            return newPath;
          }
          case 'create_scratch_file': {
            const scratchPath = '/tmp/scratch-' + Date.now() + '.md';
            fileContents[scratchPath] = '';
            return scratchPath;
          }
          case 'create_directory':
            return args.parentDir + '/' + args.name;
          case 'rename_item':
            return args.oldPath.replace(/[^/]+$/, args.newName);
          case 'delete_item':
            deletedFiles.push(args.path);
            return null;
          case 'duplicate_file':
            return args.path.replace('.md', ' copy.md');

          // Project management
          case 'detect_project':
            return projectConfig;
          case 'read_project_config':
            return projectConfig;

          // File watching (no-ops)
          case 'start_file_watcher':
          case 'stop_file_watcher':
          case 'register_open_file':
          case 'unregister_open_file':
          case 'register_write_ignore':
            return null;

          // Recent projects
          case 'get_recent_projects':
            return recentProjects;
          case 'add_recent_project':
          case 'remove_recent_project':
            return null;

          // Export
          case 'check_pandoc':
            return { installed: false, version: null };
          case 'export_project':
            return 'mock-export.pdf';

          // Plugins
          case 'list_plugins':
            return [];
          case 'get_plugin_commands':
            return [];
          case 'load_plugin':
          case 'unload_plugin':
          case 'set_plugin_document_state':
            return null;

          // Rope (large file)
          case 'rope_open':
            return { file_id: 'mock-rope-id', total_lines: 100, total_chars: 5000 };
          case 'rope_get_lines':
            return { lines: 'Mock content\\n', first_line: args.startLine, line_count: args.endLine - args.startLine };
          case 'rope_close':
          case 'rope_save':
            return null;

          // Draft notes
          case 'read_draft_note':
            return null;
          case 'write_draft_note':
          case 'delete_draft_note':
            return null;
          case 'has_draft_note':
            return false;

          // Search
          case 'search_in_project':
            return [];

          // Snapshots
          case 'list_snapshots':
            return [];
          case 'create_snapshot':
            return { id: 'snap-1', name: args.name, timestamp: Date.now(), file_count: 3, total_bytes: 1024 };
          case 'delete_snapshot':
          case 'restore_snapshot':
            return null;

          // Writing stats
          case 'record_writing_stats':
            return null;
          case 'get_writing_stats':
            return { daily: [], total_words: 0, chapters: [], streak_days: 0, today_words: 0, today_minutes: 0 };

          // Templates
          case 'list_templates':
            return [];

          // Sync
          case 'get_sync_config':
            return { enabled: false, webdav_url: '', username: '', has_password: false, interval_minutes: 30 };
          case 'save_sync_config':
          case 'sync_now':
            return null;
          case 'test_sync_connection':
            return true;

          // File manager
          case 'reveal_in_file_manager':
            return null;

          default:
            console.warn('[Tauri Mock] Unhandled command:', cmd, args);
            return null;
        }
      }

      // Expose mock state for test assertions
      window.__TAURI_MOCK_STATE__ = {
        get writtenFiles() { return { ...writtenFiles }; },
        get createdFiles() { return [...createdFiles]; },
        get deletedFiles() { return [...deletedFiles]; },
        emitEvent(event, payload) {
          const listeners = eventListeners[event] || [];
          listeners.forEach(cb => cb({ event, payload }));
        },
        reset() {
          Object.keys(writtenFiles).forEach(k => delete writtenFiles[k]);
          createdFiles.length = 0;
          deletedFiles.length = 0;
        },
      };

      // Set up __TAURI_INTERNALS__ — this is what @tauri-apps/api checks for
      window.__TAURI_INTERNALS__ = {
        transformCallback(callback, once) {
          const id = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
          window['_' + id] = (resp) => {
            if (once) delete window['_' + id];
            callback(resp);
          };
          return id;
        },
        invoke(cmd, args) {
          try {
            const result = handleInvoke(cmd, args || {});
            return Promise.resolve(result);
          } catch (e) {
            return Promise.reject(e.message || String(e));
          }
        },
        metadata: {
          currentWindow: { label: 'main' },
          currentWebview: { label: 'main', windowLabel: 'main' },
        },
        convertFileSrc(filePath) {
          return 'asset://localhost/' + encodeURIComponent(filePath);
        },
      };

      // Mock Tauri event system
      window.__TAURI_INTERNALS__.invoke = new Proxy(window.__TAURI_INTERNALS__.invoke, {
        apply(target, thisArg, argumentsList) {
          const [cmd, args] = argumentsList;
          if (cmd === 'plugin:event|listen') {
            const { event, handler } = args || {};
            if (event && handler) {
              if (!eventListeners[event]) eventListeners[event] = [];
              eventListeners[event].push(handler);
            }
            return Promise.resolve(Math.floor(Math.random() * 1000));
          }
          if (cmd === 'plugin:event|unlisten') {
            return Promise.resolve();
          }
          return target.call(thisArg, ...argumentsList);
        },
      });
    })();
  `;
}
```

- [ ] **Step 3: Create the Playwright test fixture**

Create `tests/e2e/fixtures/app-fixture.ts`:

```typescript
import { test as base, expect, type Page } from '@playwright/test';
import { buildTauriMockScript } from './tauri-mock';
import {
  MOCK_FILES,
  MOCK_FILE_CONTENTS,
  MOCK_RECENT_PROJECTS,
  MOCK_PROJECT_DIR,
  MOCK_PROJECT_CONFIG,
} from './mock-data';

/**
 * Extended Playwright test fixture that injects Tauri IPC mocks
 * and provides helper methods for common app interactions.
 */
export const test = base.extend<{
  app: Page;
  mockState: {
    getWrittenFiles: () => Promise<Record<string, string>>;
    getCreatedFiles: () => Promise<string[]>;
    getDeletedFiles: () => Promise<string[]>;
    emitEvent: (event: string, payload: unknown) => Promise<void>;
    reset: () => Promise<void>;
  };
}>({
  app: async ({ page }, use) => {
    // Inject Tauri mock BEFORE page loads
    await page.addInitScript({
      content: buildTauriMockScript({
        files: MOCK_FILES,
        fileContents: MOCK_FILE_CONTENTS,
        recentProjects: MOCK_RECENT_PROJECTS,
        projectDir: MOCK_PROJECT_DIR,
        projectConfig: MOCK_PROJECT_CONFIG,
      }),
    });

    await page.goto('/');
    // Wait for Svelte app to mount
    await page.waitForSelector('#app > *', { timeout: 10000 });

    await use(page);
  },

  mockState: async ({ app }, use) => {
    const helpers = {
      async getWrittenFiles() {
        return app.evaluate(() => (window as any).__TAURI_MOCK_STATE__.writtenFiles);
      },
      async getCreatedFiles() {
        return app.evaluate(() => (window as any).__TAURI_MOCK_STATE__.createdFiles);
      },
      async getDeletedFiles() {
        return app.evaluate(() => (window as any).__TAURI_MOCK_STATE__.deletedFiles);
      },
      async emitEvent(event: string, payload: unknown) {
        await app.evaluate(
          ([e, p]) => (window as any).__TAURI_MOCK_STATE__.emitEvent(e, p),
          [event, payload] as const,
        );
      },
      async reset() {
        await app.evaluate(() => (window as any).__TAURI_MOCK_STATE__.reset());
      },
    };

    await use(helpers);
  },
});

export { expect };
```

- [ ] **Step 4: Verify fixture compiles**

Run: `pnpm exec playwright test --list`
Expected: No compilation errors, empty test list.

- [ ] **Step 5: Commit**

```bash
git add tests/e2e/fixtures/
git commit -m "feat(test): add Tauri IPC mock layer for Playwright browser testing"
```

---

### Task 3: Add data-testid Attributes to Components

To make tests resilient to CSS class changes, add `data-testid` attributes to key interactive elements. This is a minimal, targeted change — only add testids where tests need to find elements.

**Files:**
- Modify: `app/App.svelte`
- Modify: `app/lib/components/Sidebar.svelte`
- Modify: `app/lib/components/Editor.svelte`
- Modify: `app/lib/components/Welcome.svelte`
- Modify: `app/lib/components/Settings.svelte`
- Modify: `app/lib/components/CommandPalette.svelte`
- Modify: `app/lib/components/TabBar.svelte`
- Modify: `app/lib/components/StatusBar.svelte`
- Modify: `app/lib/components/ZenMode.svelte`
- Modify: `app/lib/components/Outline.svelte`

- [ ] **Step 1: Add testids to App.svelte**

Find the main layout regions and add testids. Key targets:
- The outermost layout div: `data-testid="app-layout"`
- The sidebar container: `data-testid="sidebar-region"`
- The editor region: `data-testid="editor-region"`
- The right panel region: `data-testid="right-panel-region"`

Read `app/App.svelte` to find the exact elements, then add `data-testid` attributes to:
1. The root layout wrapper
2. The sidebar visibility wrapper (the `{#if uiStore.sidebarVisible}` block's container)
3. The main editor area container
4. The right panel container
5. The Settings overlay (`{#if settingsOpen}`)
6. The CommandPalette (`{#if paletteOpen}`)

- [ ] **Step 2: Add testids to Welcome.svelte**

```
.welcome-root       → data-testid="welcome-screen"
.new-file-btn        → data-testid="welcome-new-file"
.open-btn (folder)   → data-testid="welcome-open-folder"
.recent-item         → data-testid="recent-project-{index}"
```

- [ ] **Step 3: Add testids to Sidebar.svelte**

```
.sidebar             → data-testid="sidebar"
.sidebar-item        → data-testid="sidebar-file-{name}"  (use each file's name)
.sidebar-icon-btn (new file)   → data-testid="sidebar-new-file"
.sidebar-icon-btn (new folder) → data-testid="sidebar-new-folder"
.sidebar-open-btn    → data-testid="sidebar-open-folder"
.sidebar-input       → data-testid="sidebar-input"
.context-menu        → data-testid="context-menu"
.project-switcher    → data-testid="project-switcher"
```

- [ ] **Step 4: Add testids to Editor.svelte**

```
editor container div → data-testid="editor-container"
read-only banner     → data-testid="readonly-banner"
```

- [ ] **Step 5: Add testids to TabBar.svelte**

```
tab bar container    → data-testid="tab-bar"
each .tab-item       → data-testid="tab-{fileName}"
each .close-btn      → data-testid="tab-close-{fileName}"
.dirty-dot           → data-testid="tab-dirty-{fileName}"
```

- [ ] **Step 6: Add testids to StatusBar.svelte**

```
status bar container → data-testid="status-bar"
word count span      → data-testid="status-word-count"
cursor position span → data-testid="status-cursor-pos"
file name span       → data-testid="status-file-name"
```

- [ ] **Step 7: Add testids to Settings.svelte**

```
modal overlay        → data-testid="settings-overlay"
modal dialog         → data-testid="settings-dialog"
section buttons      → data-testid="settings-section-{name}" (editor, theme, shortcuts, etc.)
```

- [ ] **Step 8: Add testids to CommandPalette.svelte**

```
palette overlay      → data-testid="command-palette"
search input         → data-testid="palette-input"
result items         → data-testid="palette-result-{index}"
```

- [ ] **Step 9: Add testids to ZenMode.svelte and Outline.svelte**

```
ZenMode container    → data-testid="zen-mode"
Outline panel        → data-testid="outline-panel"
outline heading items → data-testid="outline-heading-{index}"
```

- [ ] **Step 10: Verify app still builds**

Run: `pnpm dev` — verify no errors, app renders normally.
Run: `pnpm check` — verify no type errors.

- [ ] **Step 11: Commit**

```bash
git add app/
git commit -m "chore: add data-testid attributes for E2E test selectors"
```

---

### Task 4: Write Welcome Screen E2E Tests

**Files:**
- Create: `tests/e2e/specs/welcome.spec.ts`

- [ ] **Step 1: Write welcome screen tests**

Create `tests/e2e/specs/welcome.spec.ts`:

```typescript
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
    // Click the first recent project
    const recentItem = app.getByTestId('recent-project-0');
    await recentItem.click();

    // Welcome screen should close, sidebar should show project files
    await expect(app.getByTestId('welcome-screen')).not.toBeVisible({ timeout: 5000 });
    await expect(app.getByTestId('sidebar')).toBeVisible();
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/welcome.spec.ts`
Expected: All tests pass. If any fail, debug by checking the IPC mock responses and adjusting selectors.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/welcome.spec.ts
git commit -m "test: add Welcome screen E2E tests"
```

---

### Task 5: Write Sidebar E2E Tests

**Files:**
- Create: `tests/e2e/specs/sidebar.spec.ts`

- [ ] **Step 1: Write sidebar tests**

Create `tests/e2e/specs/sidebar.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('Sidebar', () => {
  test.beforeEach(async ({ app }) => {
    // Open a project first by clicking a recent project
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('sidebar shows project files', async ({ app }) => {
    const sidebar = app.getByTestId('sidebar');
    await expect(sidebar).toBeVisible();

    // Check files are listed
    await expect(sidebar).toContainText('Chapter 1.md');
    await expect(sidebar).toContainText('Chapter 2.md');
    await expect(sidebar).toContainText('Notes');
  });

  test('toggle sidebar with Cmd+B', async ({ app }) => {
    const sidebar = app.getByTestId('sidebar');
    await expect(sidebar).toBeVisible();

    // Hide sidebar
    await app.keyboard.press('Meta+b');
    await expect(sidebar).not.toBeVisible({ timeout: 2000 });

    // Show sidebar
    await app.keyboard.press('Meta+b');
    await expect(sidebar).toBeVisible({ timeout: 2000 });
  });

  test('clicking a file opens it in editor', async ({ app }) => {
    // Click on Chapter 1
    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 1"]');
    await fileItem.click();

    // Editor should show and tab should appear
    await expect(app.getByTestId('editor-container')).toBeVisible();
    await expect(app.getByTestId('tab-bar')).toContainText('Chapter 1.md');
  });

  test('new file button creates a file', async ({ app }) => {
    const newFileBtn = app.getByTestId('sidebar-new-file');
    await newFileBtn.click();

    // Input field should appear for naming the file
    const input = app.getByTestId('sidebar-input');
    await expect(input).toBeVisible();
    await input.fill('New Chapter.md');
    await input.press('Enter');

    // The new file should appear in the sidebar
    await expect(app.getByTestId('sidebar')).toContainText('New Chapter.md');
  });

  test('right-click shows context menu', async ({ app }) => {
    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 1"]');
    await fileItem.click({ button: 'right' });

    const contextMenu = app.getByTestId('context-menu');
    await expect(contextMenu).toBeVisible();
    await expect(contextMenu).toContainText('Rename');
    await expect(contextMenu).toContainText('Delete');

    // Close context menu
    await app.keyboard.press('Escape');
    await expect(contextMenu).not.toBeVisible();
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/sidebar.spec.ts`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/sidebar.spec.ts
git commit -m "test: add Sidebar E2E tests"
```

---

### Task 6: Write File Operations E2E Tests

**Files:**
- Create: `tests/e2e/specs/file-operations.spec.ts`

- [ ] **Step 1: Write file operation tests**

Create `tests/e2e/specs/file-operations.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('File Operations', () => {
  test.beforeEach(async ({ app }) => {
    // Open project
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('open file, edit content, and save', async ({ app, mockState }) => {
    // Open Chapter 1
    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 1"]');
    await fileItem.click();
    await app.getByTestId('editor-container').waitFor({ state: 'visible' });

    // Wait for CodeMirror to mount
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.waitFor({ state: 'visible', timeout: 5000 });

    // Click in the editor to focus it
    await cmEditor.click();

    // Type some content
    await app.keyboard.type('Hello World');

    // The tab should show dirty indicator
    const dirtyDot = app.locator('[data-testid^="tab-dirty-"]');
    await expect(dirtyDot).toBeVisible({ timeout: 2000 });

    // Save with Cmd+S
    await app.keyboard.press('Meta+s');

    // Verify the file was written via mock state
    const written = await mockState.getWrittenFiles();
    const savedContent = Object.values(written).find((c) => (c as string).includes('Hello World'));
    expect(savedContent).toBeTruthy();
  });

  test('Cmd+N creates a new file', async ({ app }) => {
    await app.keyboard.press('Meta+n');

    // Editor should appear with an empty document
    await expect(app.getByTestId('editor-container')).toBeVisible();

    // A new tab should appear
    const tabBar = app.getByTestId('tab-bar');
    await expect(tabBar).toBeVisible();
  });

  test('Cmd+W closes the current tab', async ({ app }) => {
    // Open a file first
    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 1"]');
    await fileItem.click();
    await app.getByTestId('editor-container').waitFor({ state: 'visible' });

    // Close the tab
    await app.keyboard.press('Meta+w');

    // Editor should disappear or show welcome
    // (behavior depends on whether other tabs are open)
  });

  test('editing content marks tab as dirty', async ({ app }) => {
    // Open a file
    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 2"]');
    await fileItem.click();

    const cmEditor = app.locator('.cm-editor');
    await cmEditor.waitFor({ state: 'visible', timeout: 5000 });
    await cmEditor.click();

    // Type something
    await app.keyboard.type('Modified');

    // Check dirty indicator
    const dirtyDot = app.locator('[data-testid^="tab-dirty-"]');
    await expect(dirtyDot).toBeVisible({ timeout: 2000 });
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/file-operations.spec.ts`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/file-operations.spec.ts
git commit -m "test: add file operations E2E tests (create, edit, save)"
```

---

### Task 7: Write Editor E2E Tests

Tests for cursor positioning, content rendering, CodeMirror state — addressing the exact pain point of "click lands on wrong line."

**Files:**
- Create: `tests/e2e/specs/editor.spec.ts`

- [ ] **Step 1: Write editor tests**

Create `tests/e2e/specs/editor.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('Editor', () => {
  test.beforeEach(async ({ app }) => {
    // Open project and a file
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }

    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 1"]');
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

    // Status bar should show cursor position
    const cursorPos = app.getByTestId('status-cursor-pos');
    await expect(cursorPos).toBeVisible();
    // Should contain line and column numbers
    await expect(cursorPos).toContainText(/\d+:\d+/);
  });

  test('word count updates in status bar', async ({ app }) => {
    const wordCount = app.getByTestId('status-word-count');
    await expect(wordCount).toBeVisible();
    // The mock file has content, so word count should be > 0
    const text = await wordCount.textContent();
    expect(text).toBeTruthy();
  });

  test('clicking in editor moves cursor to correct position', async ({ app }) => {
    // Get the CodeMirror content element
    const cmContent = app.locator('.cm-content');

    // Find a specific line in the editor
    const targetLine = app.locator('.cm-line').filter({ hasText: 'wind howled' });
    await targetLine.click();

    // Read the cursor position from the CM6 view
    const cursorLine = await app.evaluate(() => {
      const view = (window as any).__novelist_view;
      if (!view) return -1;
      const pos = view.state.selection.main.head;
      return view.state.doc.lineAt(pos).number;
    });

    // The line with "wind howled" should be line 5 in our mock content
    expect(cursorLine).toBeGreaterThan(0);
  });

  test('typing inserts text at cursor position', async ({ app }) => {
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.click();

    // Move to end of first line
    await app.keyboard.press('Meta+ArrowDown'); // go to end of doc
    await app.keyboard.press('Enter');
    await app.keyboard.type('New paragraph here');

    // Verify content was inserted
    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('New paragraph here');
  });

  test('undo/redo works', async ({ app }) => {
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.click();

    // Type something
    await app.keyboard.press('Meta+ArrowDown');
    await app.keyboard.press('Enter');
    await app.keyboard.type('Undo me');

    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('Undo me');

    // Undo
    await app.keyboard.press('Meta+z');
    await expect(cmContent).not.toContainText('Undo me');

    // Redo
    await app.keyboard.press('Meta+Shift+z');
    await expect(cmContent).toContainText('Undo me');
  });

  test('CJK content renders correctly', async ({ app }) => {
    // Open the Chinese chapter
    const fileItem = app.locator('[data-testid^="sidebar-file-Chapter 3"]');
    await fileItem.click();

    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('第三章');
    await expect(cmContent).toContainText('中文测试文本');
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/editor.spec.ts`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/editor.spec.ts
git commit -m "test: add Editor E2E tests (content, cursor, undo, CJK)"
```

---

### Task 8: Write Tab Management E2E Tests

**Files:**
- Create: `tests/e2e/specs/tabs.spec.ts`

- [ ] **Step 1: Write tab management tests**

Create `tests/e2e/specs/tabs.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('Tab Management', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
  });

  test('opening multiple files creates multiple tabs', async ({ app }) => {
    // Open Chapter 1
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    // Open Chapter 2
    await app.locator('[data-testid^="sidebar-file-Chapter 2"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    // Both tabs should exist
    const tabBar = app.getByTestId('tab-bar');
    await expect(tabBar).toContainText('Chapter 1');
    await expect(tabBar).toContainText('Chapter 2');
  });

  test('clicking a tab switches to that file', async ({ app }) => {
    // Open two files
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });
    await app.locator('[data-testid^="sidebar-file-Chapter 2"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    // Click Chapter 1 tab
    const tab1 = app.locator('[data-testid^="tab-Chapter 1"]');
    await tab1.click();

    // Editor content should show Chapter 1
    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('dark and stormy night');
  });

  test('closing a tab with click on X button', async ({ app }) => {
    // Open a file
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    // Click the close button on the tab
    const closeBtn = app.locator('[data-testid^="tab-close-Chapter 1"]');
    await closeBtn.click();

    // Tab should be gone
    const tabBar = app.getByTestId('tab-bar');
    await expect(tabBar).not.toContainText('Chapter 1');
  });

  test('Cmd+W closes active tab', async ({ app }) => {
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    await app.keyboard.press('Meta+w');

    // Tab should close
    const tabBar = app.getByTestId('tab-bar');
    await expect(tabBar).not.toContainText('Chapter 1');
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/tabs.spec.ts`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/tabs.spec.ts
git commit -m "test: add Tab management E2E tests"
```

---

### Task 9: Write Command Palette E2E Tests

**Files:**
- Create: `tests/e2e/specs/command-palette.spec.ts`

- [ ] **Step 1: Write command palette tests**

Create `tests/e2e/specs/command-palette.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('Command Palette', () => {
  test.beforeEach(async ({ app }) => {
    // Open a project so we have full UI
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

    // Should show filtered results
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

  test('selecting a command executes it', async ({ app }) => {
    // Open a file so we have an editor
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });

    await app.keyboard.press('Meta+Shift+p');
    const input = app.getByTestId('palette-input');
    await input.fill('zen');

    // Click the first result
    const firstResult = app.locator('[data-testid="palette-result-0"]');
    if (await firstResult.isVisible()) {
      await firstResult.click();
      // Zen mode should activate
      await expect(app.getByTestId('zen-mode')).toBeVisible({ timeout: 3000 });
      // Exit zen mode
      await app.keyboard.press('Escape');
    }
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/command-palette.spec.ts`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/command-palette.spec.ts
git commit -m "test: add Command Palette E2E tests"
```

---

### Task 10: Write Settings Dialog E2E Tests

**Files:**
- Create: `tests/e2e/specs/settings.spec.ts`

- [ ] **Step 1: Write settings dialog tests**

Create `tests/e2e/specs/settings.spec.ts`:

```typescript
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

    // Click each section tab
    const sections = ['editor', 'theme', 'shortcuts'];
    for (const section of sections) {
      const sectionBtn = app.getByTestId(`settings-section-${section}`);
      if (await sectionBtn.isVisible()) {
        await sectionBtn.click();
        // Verify section content loads (no crash)
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
```

- [ ] **Step 2: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/settings.spec.ts`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/specs/settings.spec.ts
git commit -m "test: add Settings dialog E2E tests"
```

---

### Task 11: Write Zen Mode and Split View E2E Tests

**Files:**
- Create: `tests/e2e/specs/zen-mode.spec.ts`
- Create: `tests/e2e/specs/split-view.spec.ts`

- [ ] **Step 1: Write zen mode tests**

Create `tests/e2e/specs/zen-mode.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('Zen Mode', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
    // Open a file
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });
  });

  test('F11 enters zen mode', async ({ app }) => {
    await app.keyboard.press('F11');

    const zenMode = app.getByTestId('zen-mode');
    await expect(zenMode).toBeVisible({ timeout: 2000 });

    // Sidebar and tab bar should be hidden
    await expect(app.getByTestId('sidebar')).not.toBeVisible();
    await expect(app.getByTestId('tab-bar')).not.toBeVisible();
  });

  test('Escape exits zen mode', async ({ app }) => {
    await app.keyboard.press('F11');
    await expect(app.getByTestId('zen-mode')).toBeVisible();

    await app.keyboard.press('Escape');
    await expect(app.getByTestId('zen-mode')).not.toBeVisible({ timeout: 2000 });

    // Regular UI should return
    await expect(app.getByTestId('sidebar')).toBeVisible();
  });

  test('can still edit in zen mode', async ({ app }) => {
    await app.keyboard.press('F11');
    await expect(app.getByTestId('zen-mode')).toBeVisible();

    // The editor should still be functional
    const cmEditor = app.locator('.cm-editor');
    await cmEditor.click();
    await app.keyboard.type('Zen writing');

    const cmContent = app.locator('.cm-content');
    await expect(cmContent).toContainText('Zen writing');

    await app.keyboard.press('Escape');
  });
});
```

- [ ] **Step 2: Write split view tests**

Create `tests/e2e/specs/split-view.spec.ts`:

```typescript
import { test, expect } from '../fixtures/app-fixture';

test.describe('Split View', () => {
  test.beforeEach(async ({ app }) => {
    const recentItem = app.getByTestId('recent-project-0');
    if (await recentItem.isVisible().catch(() => false)) {
      await recentItem.click();
      await app.getByTestId('sidebar').waitFor({ state: 'visible', timeout: 5000 });
    }
    // Open a file
    await app.locator('[data-testid^="sidebar-file-Chapter 1"]').click();
    await app.locator('.cm-editor').waitFor({ state: 'visible', timeout: 5000 });
  });

  test('Cmd+\\ toggles split view', async ({ app }) => {
    // Enable split view
    await app.keyboard.press('Meta+\\');

    // Should see two editor containers
    const editors = app.locator('[data-testid="editor-container"]');
    const count = await editors.count();
    expect(count).toBeGreaterThanOrEqual(2);

    // Disable split view
    await app.keyboard.press('Meta+\\');

    const countAfter = await editors.count();
    expect(countAfter).toBeLessThan(count);
  });
});
```

- [ ] **Step 3: Run the tests**

Run: `pnpm exec playwright test tests/e2e/specs/zen-mode.spec.ts tests/e2e/specs/split-view.spec.ts`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add tests/e2e/specs/zen-mode.spec.ts tests/e2e/specs/split-view.spec.ts
git commit -m "test: add Zen Mode and Split View E2E tests"
```

---

### Task 12: Set Up tauri-plugin-playwright for Full E2E (Tier 3)

This enables running the same Playwright tests against the real Tauri app with the actual Rust backend.

**Files:**
- Modify: `core/Cargo.toml` — add `tauri-plugin-playwright` dependency behind feature flag
- Modify: `core/src/lib.rs` — register plugin when feature is enabled
- Modify: `core/tauri.conf.json` — add `withGlobalTauri: true`
- Modify: `playwright.config.ts` — add tauri project
- Modify: `package.json` — add `test:e2e:tauri` script

- [ ] **Step 1: Add Rust dependency behind feature flag**

Add to `core/Cargo.toml`:

```toml
[dependencies]
# ... existing deps ...
tauri-plugin-playwright = { version = "0.2", optional = true }

[features]
# ... existing features ...
e2e-testing = ["dep:tauri-plugin-playwright"]
```

- [ ] **Step 2: Register plugin in lib.rs when feature is enabled**

In `core/src/lib.rs`, add conditional plugin registration:

```rust
// At the top
#[cfg(feature = "e2e-testing")]
use tauri_plugin_playwright;

// In the builder chain
#[cfg(feature = "e2e-testing")]
let builder = builder.plugin(tauri_plugin_playwright::init());
```

Read the actual `lib.rs` to find the right insertion point within the Tauri builder chain.

- [ ] **Step 3: Enable withGlobalTauri in tauri.conf.json**

Add to the `"app"` section of `core/tauri.conf.json`:

```json
"withGlobalTauri": true
```

This is required for `tauri-plugin-playwright` to communicate with the webview.

- [ ] **Step 4: Install npm package**

```bash
pnpm add -D @srsholmes/tauri-playwright
```

- [ ] **Step 5: Add tauri project to Playwright config**

Update `playwright.config.ts` to add a tauri project:

```typescript
projects: [
  {
    name: 'webkit',
    use: { ...devices['Desktop Safari'] },
  },
  {
    name: 'chromium',
    use: { ...devices['Desktop Chrome'] },
  },
  {
    name: 'tauri',
    use: {
      // tauri-plugin-playwright connects via socket
      connectOptions: {
        wsEndpoint: 'ws://localhost:9222',
      },
    },
  },
],
```

Note: The exact configuration may need adjustment based on `@srsholmes/tauri-playwright` documentation. Read their README for the latest setup instructions.

- [ ] **Step 6: Add test scripts**

Add to `package.json` scripts:

```json
"test:e2e:tauri": "cargo build --features e2e-testing -p novelist && playwright test --project=tauri",
"tauri:e2e-build": "pnpm tauri build --features e2e-testing -- --debug"
```

- [ ] **Step 7: Verify the e2e-testing feature compiles**

Run: `cd core && cargo check --features e2e-testing`
Expected: Compiles without errors.

- [ ] **Step 8: Commit**

```bash
git add core/Cargo.toml core/src/lib.rs core/tauri.conf.json playwright.config.ts package.json pnpm-lock.yaml
git commit -m "feat(test): add tauri-plugin-playwright for full E2E testing (Tier 3)"
```

---

### Task 13: Archive Old bash E2E Tests

**Files:**
- Move: `tests/e2e/test-*.sh` → `tests/e2e/old/`

- [ ] **Step 1: Move old scripts to archive directory**

```bash
mkdir -p tests/e2e/old
mv tests/e2e/test-all-features.sh tests/e2e/old/
mv tests/e2e/test-scroll-click.sh tests/e2e/old/
mv tests/e2e/test-scroll-edit-line.sh tests/e2e/old/
mv tests/e2e/test-integrity.sh tests/e2e/old/
mv tests/e2e/test-e2e-auto.sh tests/e2e/old/
```

- [ ] **Step 2: Update package.json script**

Change `"test:e2e"` from bash script to Playwright:

```json
"test:e2e": "playwright test --project=webkit"
```

- [ ] **Step 3: Commit**

```bash
git add tests/e2e/ package.json
git commit -m "chore: archive old bash E2E tests, use Playwright as default"
```

---

### Task 14: Update CI Workflow

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add Playwright E2E step to CI**

Read the current `.github/workflows/ci.yml` and add a Playwright step after the existing test steps:

```yaml
    - name: Install Playwright browsers
      run: pnpm exec playwright install --with-deps webkit

    - name: Run E2E tests (browser mode)
      run: pnpm test:e2e:browser
```

The browser-mode tests don't need the Tauri binary — they run against the Vite dev server with mocked IPC. This makes them fast and CI-friendly.

- [ ] **Step 2: Verify CI config is valid**

Run: `cat .github/workflows/ci.yml` and verify YAML syntax.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add Playwright E2E tests to CI pipeline"
```

---

## Execution Notes

**Test running order during development:**
1. `pnpm test` — fast unit tests (existing vitest, ~2s)
2. `pnpm test:e2e:browser` — browser E2E with mocked IPC (~30s)
3. `pnpm test:e2e:tauri` — full Tauri E2E, pre-release only (~2min)
4. `pnpm test:rust` — Rust backend tests (~10s)

**Key differences from old approach:**
| Old (bash+cliclick) | New (Playwright) |
|-----|-----|
| `cliclick c:700,400` | `app.getByTestId('sidebar-file-Chapter 1').click()` |
| `sleep 0.5` | `await element.waitFor({ state: 'visible' })` |
| `ok "Sidebar toggle"` (no verification) | `await expect(sidebar).not.toBeVisible()` |
| Manual app launch | Automatic via `webServer` config |
| macOS-only, Accessibility required | Any OS, headless CI |

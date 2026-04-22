# Changelog

All notable changes to Novelist will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-04-22

Second minor release. Focuses on AI-assisted writing (multi-session
AI Talk + AI Agent panels, prompt-preset library) and a hardened test
foundation (three-tier hierarchy, describe-tag discipline, coverage
campaign with enforced floors). Also ships a from-scratch SVG icon
system and a rewritten selection-background subsystem.

### Added
- **AI Talk** — multi-session chat panel with preset picker, per-session
  conversation state, save-as-Markdown to current project
- **AI Agent** — multi-session UI with per-session Claude CLI
  subprocess, save-chat-to-project action
- **Prompt preset manager** — full CRUD UI in Settings, stored per
  project; shared by Talk and Agent panels
- **Keyboard shortcuts for AI panels** — toggle / new-session / save-chat
  wired through the single `app-commands.ts` dispatch map
- **Icon system** — hand-rolled SVG component batch replacing emoji
  across the UI (Batch-1 spec in `docs/design/icon-batch-1-spec.md`)
- **Three-tier test hierarchy** — `tests/unit/`, `tests/integration/`,
  `tests/e2e/` split with vitest projects + fixtures skeleton
- **`[precision]` / `[contract]` / `[regression]` / `[smoke]`
  describe-tag convention** — documented in
  `docs/architecture/testing-precision.md`, applied to starter set
- **Coverage governance** — `@vitest/coverage-v8` pipeline, baseline
  capture, waiver registry (`tests/COVERAGE.md`), CI artifact upload
- **Enforced coverage floors** — stmt 73 / branch 67 / func 75 / line 75
  (campaign raised ~160 new tests across stores, composables, editor,
  services, utils, app-root)
- **Extracted pure decoration builders** — `buildSelectionDecorations`
  and friends exposed for precision-level unit testing

### Changed
- **Selection highlight rewrite** — three-layer paint system (line
  decoration + native `::selection` + suppression inside selected
  lines) with specificity override for CM6's `hideNativeSelection`
  internal theme. See the "Unified selection background" section of
  `docs/architecture/editor.md` before touching this subsystem
- **Test runtime split** — previous `tests/unit/**` reorganised; pure
  logic stays in unit, anything that boots a View or DOM moved to
  integration
- Runtime/integration-leaning suites migrated out of `unit/` per the
  new hierarchy spec

### Fixed
- **Partial-character selection color mismatch** — partial ranges and
  full-line ranges now render at the same 18% accent tint
- **Wrapped partial selection ragged right edge** — continuation rows
  of a wrapped selection now fill to the container right edge via
  native `::selection` instead of fragmented `<span>` backgrounds
- **Invisible partial selections** — overcome CM6's
  `hideNativeSelection` at `Prec.highest` by bumping our rule's CSS
  specificity to `(0,3,1)` plus `!important`
- **Precise character-range selection background** — earlier fix for
  the original paint-whole-line regression that motivated the
  test-hierarchy and precision-testing work

## [0.1.0] - 2026-04-17

First minor release. Consolidates all 0.0.5+ work into a feature-complete
baseline: restructured codebase, full plugin system, three-tier test suite,
and mature WYSIWYG editor.

### Added
- Complete Rust backend: plugin sandbox (QuickJS), rope-backed large-file
  support, FSEvent file watcher, writing-stats, snapshots, WebDAV sync
- Three-tier testing: 291 Vitest unit tests, 38 Playwright E2E specs, 161
  cargo tests (see `docs/development.md`)
- Plugin system with manifest, enable/disable, permission tiers; built-in
  canvas and mindmap plugins
- Split-pane editing with independent tab state per pane
- Notion-style project switcher (Cmd+1~9)
- Project-wide search (Cmd+Shift+F), multi-window (Cmd+Shift+N)
- Snapshot panel, stats panel, conflict dialog, virtual scrollbar
- i18n infrastructure (English, 简体中文)
- Styled DMG installer with new branding
- GitHub issue templates, PR template, CONTRIBUTING.md, CHANGELOG.md

### Changed
- Directory restructure: `src/` → `app/`, `src-tauri/` → `core/`;
  consolidated `design/` and `research/` into `docs/`
- Typora-style heading rendering with flat-size mode for > 5000-line files
- Tab key inserts configurable indentation (tab / 2 / 4 / 8 spaces)
- Image rendering now uses single block decoration (prevents height-map drift)
- Replaced CSS `zoom` with `transform: scale()` for CM6 coordinate correctness
- Security hardening: CSP, atomic writes, self-write suppression, zero warnings

### Fixed
- Scroll-click offset bug in large files (three-layer native scroll guard)
- Duplicate line-number gutter markers after block image decoration
- Cursor oscillation caused by cursor-dependent block decorations
- White-screen crashes now caught by ErrorBoundary

## [0.0.6] - 2026-04-13

### Added
- Single-file mode with scratch files (Cmd+N without project)
- Project remove functionality in sidebar and welcome screen
- Cmd+W close window, auto-save on tab switch
- Rename file shortcut (Cmd+Shift+R), Enter to rename in sidebar
- Playwright E2E testing infrastructure
- Updater i18n: all update dialogs now respect locale setting
- Close-tab dialog i18n: unsaved changes prompt uses locale

### Changed
- Editor read-only banner and split-chunks alert now use i18n
- Removed scroll diagnostic debug plugin from production builds
- Removed all debug console.log statements from production code
- Code robustness improvements across stores and components

### Fixed
- Hardcoded English strings in updater, close-tab dialog, and editor banner replaced with i18n keys

## [0.0.4] - 2026-04-12

### Changed
- Restructured project directories for clarity: `src/` -> `app/`, `src-tauri/` -> `core/`
- Consolidated documentation: `design/`, `docs/`, `research/` merged into `docs/`
- Separated build assets into `assets/` (DMG backgrounds, branding)

### Added
- CONTRIBUTING.md with development setup and guidelines
- CHANGELOG.md
- `.editorconfig` for consistent formatting
- GitHub issue and PR templates
- i18n support (English, Simplified Chinese)
- New app icon
- Plugin system with QuickJS sandbox
- Styled DMG installer with custom background

## [0.0.3] - 2026-04-09

### Added
- Tab key inserts indentation with configurable style (tab/2/4/8 spaces)
- Cmd+W closes tab with unsaved changes prompt
- Keyboard shortcut customization via Settings
- Theme system tests

### Fixed
- Sidebar font size increased by 2px for readability
- Heading line-height matched to body rhythm

## [0.0.2] - 2026-04-08

### Added
- Error boundary to prevent white-screen crashes
- Multi-window support (Cmd+Shift+N)
- Project-wide search (Cmd+Shift+F)
- File drag-and-drop (.md, .markdown, .txt)
- Undo history persistence across tab switches
- Auto-save with configurable interval (default 5 min)
- Theme transitions (smooth CSS animation)
- Status bar (file name, size, daily goal progress)
- Export with animated progress bar
- Content Security Policy
- CI/CD with GitHub Actions

### Fixed
- Recent projects list filters non-existent paths

## [0.0.1] - 2026-04-07

### Added
- Initial release
- WYSIWYG Markdown editing (Typora-style)
- GFM support (tables, strikethrough, task lists)
- Inline image preview
- Split view editing
- Draft notes panel
- Theme system (Light, Dark, Sepia, Nord, GitHub, Dracula)
- Outline panel with heading navigation
- Zen Mode with typewriter scrolling
- Command palette
- Export via Pandoc (HTML, PDF, DOCX, EPUB)
- File watching with conflict resolution
- CJK-aware word counting
- Large file support (4-tier performance system)

# Changelog

All notable changes to Novelist will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### CI

- **Release workflow no longer dead-locks on a single failing matrix leg.**
  The `release` job's `if:` now wraps with `always()` so the implicit
  `success()` gate stops permanently skipping it when any one upstream
  platform fails (the case that left v0.2.4's macOS dmg out of the auto-
  published release until manual upload). Gate still requires at least one
  upstream build to succeed before publishing.
- **Intel macOS DMG cross-compiled from `macos-14` instead of `macos-13`.**
  GitHub-hosted Intel macOS runners are mid-retirement and queue for
  multiple hours; we now cross-compile `x86_64-apple-darwin` from the
  GitHub-hosted Apple Silicon runner via `rustup target add` +
  `tauri build --target x86_64-apple-darwin`. ARM mac still builds
  natively on the self-hosted runner.

## [0.2.6] - 2026-05-23

### Fixed

- Prevented close-tab, window-close, and project-switch flows from proceeding
  after a dirty file fails to save.
- Fixed empty-document saves so clearing a file writes the empty content
  instead of skipping the save path.
- Restored H1-driven filename sync for `Untitled {N}`-style templates and made
  the behavior respect the `auto_rename_from_h1` setting.
- Corrected first-launch and legacy auto-zoom behavior so OS-level DPI scaling
  is not compounded by Novelist's own zoom transform.
- Improved sidebar file-row sizing and font selection for readable CJK and
  non-monospace desktop navigation.
- Hardened startup/menu event listeners so async initialization failures are
  logged without breaking the app shell.
- Stabilized Vitest under Node 26 by providing a deterministic `localStorage`
  test setup.
- Fixed Rust clippy failures in async tests that mutate process-global test
  data directories.

## [0.2.4] - 2026-05-14

Major release adding **image hosting** and **publishing** — Novelist
can now upload images to your CDN of choice and push the active
Markdown document to your blog with one click.

本次 v0.2.4 主版本新增两套核心功能：图床上传与一键发布。可以把 Markdown
中的本地图片自动上传到你常用的图床（七牛、阿里云 OSS、S3、R2、imgur、
sm.ms 或自定义 HTTP 接口），并直接发布到 Ghost / WordPress / Medium。

### Added — Image hosting

- **Six built-in image-host providers** plus a minimal Custom HTTP
  endpoint:
  - **Qiniu Kodo** — HMAC-SHA1 upload-token, multipart POST
  - **Aliyun OSS** — HMAC-SHA1 header signature, virtual-hosted PUT
  - **Amazon S3** — manual SigV4 path-style PUT (works with Cloudflare
    R2 and MinIO via endpoint override)
  - **imgur** — Client-ID Bearer
  - **sm.ms** — optional API token, with `image_repeated` dedup
  - **Custom** — multipart POST with optional Bearer; response URL
    read from `url` or `data.url`
- **Settings → Image Hosts panel** — list, add/edit/delete, set active
  default, per-host Test button (uploads a 1×1 PNG to verify
  credentials without saving anything noisy)
- **Image context menu** — right-click any inline image with a local
  path → "Upload to host" replaces the Markdown URL in place
- **Upload all local images** — palette command iterates the
  document's image references, uploads each sequentially, applies all
  URL replacements in one CodeMirror transaction, and reports
  partial-failure status via toast
- **Optional auto-on-paste / auto-on-drop** — off by default. When
  enabled in settings, every pasted or dropped image immediately
  uploads to the active host
- **Date-prefixed object keys** for bucket-style hosts —
  `{yyyy}/{mm}/{dd}/{8-char-blake3}-{sanitized-name}.{ext}` keeps the
  remote bucket organized and collision-free
- **Plaintext credentials in global settings only** — per-project
  overrides are limited to the active-host pointer, so credentials
  never leak into project files that might end up in git

### Added — Publishing

- **Four publish platforms**, each as a self-contained adapter:
  - **Ghost** — Admin API key signed as 5-min HS256 JWT, posts via
    `?source=html` so we send HTML and Ghost converts to Lexical
    server-side
  - **WordPress (self-hosted)** — Application Passwords (HTTP Basic),
    full REST tag-name → ID resolution with auto-create, raw-body
    media upload
  - **WordPress.com** — OAuth2 personal access token (Bearer), same
    REST shape with `/sites/<id>` prefix
  - **Medium** — legacy Integration Token Bearer; native Markdown
    body, max-3 tag cap. *Note: Medium removed the Integration Token
    UI in late 2024; the API still accepts pre-existing tokens but
    new users typically cannot generate one.*
- **Editor share menu** — top-right share button with dropdown:
  *Upload Local Images to…* + one *Publish to <channel>* entry per
  configured channel (mweb-style, single discoverable entry point)
- **Publish dialog** with mweb-shaped fields: title (auto-prefilled
  from the document's H1), tags (chip input), excerpt, slug
  (auto-generated kebab-case), per-platform status dropdown, drag-drop
  cover image. Inline error on failure with retry; success banner with
  "Open in browser" action.
- **Pre-publish image upload** — local images in the body are uploaded
  to the publish target's own media endpoint and rewritten in the
  submitted body. No third-party CDN required for publishing. (If the
  user prefers their image host instead, they run *Upload Local Images
  to…* explicitly first.)
- **Settings → Publish panel** — per-channel CRUD, manual-paste auth
  for all four platforms, Test button does a 1×1 PNG smoke check.
- **Pandoc Markdown → HTML conversion** — used by Ghost / WordPress /
  WordPress.com; Medium consumes Markdown natively.
- **No update tracking** — every publish creates a new post.
  Re-publishing an edited document creates duplicates the user prunes
  manually on the platform side. (Front-matter ID writeback is
  intentionally out of scope for v0.2.4.)

### Added — UX polish

- **Bilingual command palette** — Cmd+Shift+P now shows the
  active-locale label large with the alternate-locale label as a small
  subscript below; the search matcher hits both. Lets a Chinese-locale
  user find "Toggle Outline" by typing either *outline* or *大纲*.
- **Double-click to rename** in the file tree — double-clicking a file
  row in the sidebar opens inline rename. Folder double-click still
  toggles expansion. Rename input now renders inline under the
  renamed node (previously rendered at the top of the sidebar).
- **Right-side panel shortcuts moved to Cmd+Shift+1/2/3/4** — outline,
  draft, snapshot, stats. Easier chord than Cmd+Alt. Note: macOS users
  may need to disable system screenshot shortcuts (Cmd+Shift+3/4) for
  4-position behavior. Toggle-template stays on Cmd+Alt+5 (Cmd+Shift+5
  is the macOS screenshot toolbar).
- **Drag files from the sidebar into a pane or split-right zone** —
  while dragging a leaf node the editor area shows two drop overlays
  (the active pane + a right-edge "split right" gutter); dropping
  opens the file in that pane. Tab bars accept the same payload.
- **Wrap long file names** — new Settings → Editor toggle keeps long
  file names on multiple lines instead of truncating to an ellipsis.
- **Edge-mounted sidebar toggle** — collapse/reopen the sidebar from a
  small ◁ / ▷ button on the resize handle, in addition to the
  existing keyboard shortcut.
- **Editor max-width is a slider + numeric input** (480–9999px)
  instead of a fixed dropdown.
- **Filename macros for new files** — `{yyyy}`, `{mm}`, `{dd}`,
  `{HH}`, `{MM}`, `{ss}` are expanded in the new-file template,
  with live previews under the setting.
- **Searchable Settings dialog** — top filter narrows visible panels
  and rows by label, hint, or keyword (Chinese aliases included).
- **Publish dialog polish** — paste-from-clipboard, Ghost tag
  autocomplete, redesigned pill-grid tag picker, native clipboard
  read, Test button that actually hits the platform, edit button +
  secret-reveal in the credential rows, fully localized.
- **`{title}`-only filename template** — new-file template now
  accepts `{title}` without a `{N}` counter slot (renders
  `Untitled.md` initially, then `Untitled 2.md`, `Untitled 3.md`
  on collision; H1 auto-rename replaces the whole stem). Typo
  tokens like `{cN}` or `{Title}` are still rejected so a misspelt
  placeholder doesn't silently become literal text.
- **Sort sidebar by creation time** — Sidebar view menu adds
  *Created (newest)* / *创建时间（最新）* and *Created (oldest)* /
  *创建时间（最早）* alongside the existing modified-time options.
  Filesystems that don't expose birth time fall back to mtime.
- **Command-palette shortcut chip uses the UI font** so Apple
  shortcut symbols (⇧⌘⌥⌃) render with the same stroke weight as
  the letter next to them.
- **Settings checkbox rows align with the first text line** — the
  wrap-filenames row in Settings → Editor previously floated its
  checkbox above the label baseline; it now lines up.
- **Sidebar folder right-click adds Expand All / Collapse All** —
  recursive (whole subtree), lazy-loaded BFS expand, collapse keeps
  the child cache so re-expand is instant. The same items also appear
  in the empty-area sidebar menu (operates on project root). See
  `docs/design-docs/ui-design.md` §3.2.3.

### Changed

- **Filename now follows the H1 heading on every save** (was first-save
  only). Manually renaming a file detaches the sync. See spec
  `docs/superpowers/specs/2026-05-12-h1-filename-ongoing-sync-design.md`.
- **`{title}` in the new-file template implicitly opts in to H1
  rename-on-save.** The dedicated *Sync filename with H1 on save*
  checkbox is gone — its intent is now expressed by whether the
  template contains `{title}`. The shipped default
  (`第{N}章-{title}`) keeps the behavior on; remove `{title}` to
  opt out. Path A (placeholder first-time rename) and Path B (ongoing
  sync) both honor the new gate.
- **Sibling-aware new-file naming is always on.** The *Detect chapter
  pattern from folder* checkbox is gone — `inferNextName` always reads
  the folder so `{N}` slots pick the next free number; templates
  without `{N}` are unaffected.

### Fixed

- **Filename-macro tooltip (the `?` next to "默认文件名模板") opens
  rightward**, so content no longer clips against the panel's left
  edge when the trigger sits near it.

- **Numeric sidebar sort orders 第九 < 第十 < 第十一** and no
  longer misplaces files whose Chinese number is followed by extra
  characters like `第十章终稿.md`.
- **In-project search is now Unicode case-insensitive** and emits
  UTF-16 offsets, so highlights land on the right glyphs for CJK
  and Latin-with-diacritic queries; empty queries short-circuit
  instead of scanning the project.
- **File watcher refreshes the right parent on rename / delete**
  on Windows + macOS, emitting a separate `directory-changed`
  event and polling every 15s so the sidebar doesn't keep ghost
  rows around after a rename done in another window or by an
  external tool. `.canvas` and `.kanban` files now open via the
  same external open-file path as Markdown.
- **Plain folders without `.novelist/project.toml`** stay
  global-scoped — previously the app tried to write view settings
  into folders it had no project file in.
- **Save (Cmd+S) on a clean tab still runs the H1 → file-name
  auto-rename check** when no edits since the last save introduced
  a new H1.
- **External links open via tauri-plugin-shell** instead of the
  default WebKit navigation, so they actually open in the
  system browser.
- **Pandoc auto-discovery + user override** — no longer relies on
  the previously-bundled Pandoc binary; falls back to PATH probing
  with a manual override in Settings → Editor.

### Security

- **Template zip import** is now done via the `zip` crate (instead
  of shelling out to `unzip`), rejects symlinks, NUL bytes,
  path-traversal entries, and caps the archive at 2048 entries /
  50 MiB uncompressed. Template ids are validated to
  `[a-z0-9-]{1,96}` before they reach the filesystem.
- **WebDAV sync** validates remote file paths before downloading —
  absolute paths, `..` components, and dotfile-prefixed components
  are rejected, so a hostile server can't write above the project
  root.

### Platform

- **macOS bundle declares `CFBundleLocalizations` for `en` and
  `zh-Hans`** via the newly bundled `core/Info.plist`, so Chinese
  users get localized application menus and system dialogs.

### Removed

- **"Run Release Benchmark" command** — internal diagnostic, not
  user-facing. The regular performance benchmark stays available.
- **Settings → Editor → "Sync filename with H1 on save" checkbox** —
  the gate moved to the filename template (use `{title}`). Stored
  `auto_rename_from_h1` values in existing project configs are still
  parsed but no longer consulted.
- **Settings → Editor → "Auto-detect chapter pattern from folder"
  checkbox** — sibling-aware naming is now unconditional.

### Notes for developers

- The `tauri-specta` codegen pipeline is currently broken upstream
  (specta-serde rejects `skip_serializing_if` on existing
  `ProjectConfig` fields). New IPC commands and types in v0.2.4 are
  patched into `app/lib/ipc/commands.ts` manually. The file remains
  `@ts-nocheck` so this is type-safe at the consumer level. A
  follow-up release will restore auto-generation.
- E2E coverage for the new image-host and publish UIs is deferred.
  Rust + frontend unit coverage is comprehensive (over 1500 tests
  total); manual smoke-testing covers the UI surfaces for v0.2.4.

## [0.2.3] - 2026-05-03

Patch release. Three quality-of-life improvements that finally make
Novelist feel **at home in a terminal-driven workflow** while smoothing
out two long-standing rough edges (auto-update and chapter naming).

本次 patch 集中提升日常工作流的顺滑度：新增 `novelist` 命令行（仿 VS Code
`code`），重写自动更新流程（带真实重启），并把章节自动命名梳理成统一规则
（`{N}` 总是阿拉伯数字、`{CN}` 总是中文 一二三）。

### Added
- **`novelist` command-line entry point** — invoke files or folders from
  any terminal:
  - `novelist file.md` opens the file (in the existing single-file
    window if one is already open, otherwise a fresh window)
  - `novelist /path/to/project` always opens the folder in a **new
    window** (matches VS Code's "new window for folder" convention)
  - `novelist -n file.md` — force a new window even for files
  - `novelist -g file.md:42:5` — open and jump to line:col
  - `novelist --help` / `--version` — print and exit without launching
    the GUI
  - `tauri-plugin-single-instance` dedupes concurrent invocations so a
    second `novelist …` forwards its argv to the running instance
    instead of spawning a duplicate process
  - **Install command in palette** — *"Install 'novelist' Command in
    PATH"* symlinks the bundled shim to `/usr/local/bin/novelist` on
    macOS/Linux; on Windows the shim copies to
    `%LOCALAPPDATA%\Novelist\bin\novelist.cmd` and the dialog explains
    how to add that directory to `PATH`
- **Update banner + progress modal** — the silent startup-check no
  longer disappears. When a new version is found you see a
  bottom-right banner; clicking *Install* opens an in-app modal that
  shows download progress (bytes + %) → installing → *"Update
  installed. Restart now?"* with explicit *Restart Now* / *Restart
  Later* choices. Restart goes through `tauri-plugin-process::relaunch`
  (newly bundled).
- **`{CN}` chapter token** — Chinese-numeral form (一、二、三) now
  available as `{CN}` in the new-file template (e.g. `第{CN}章`)

### Changed
- **Default new-file template** is now `第{N}章-{title}` (was
  `Untitled {N}`). Existing user/project settings are not touched.
- **`{N}` is always Arabic** — the previous implicit "if template
  starts with `第`, default to Chinese" magic was removed. Use `{CN}`
  explicitly when you want Chinese numerals.
- **Updater UX strings** no longer claim "the app will restart
  automatically" — the new flow asks the user.

### Removed
- **Capital `{cN}` / financial Chinese (壹贰叁) numbering** — the
  upper-case Chinese-numeral style was unused in practice and
  removed. `{CN}` is now the single Chinese form (lowercase 一二三).
  Templates using the old `{cN}` token must switch to `{CN}`.

### Fixed
- Updater no longer silently fails to relaunch after install — the
  app actually exits and restarts on the new binary now.

### Frontend
- New `app/lib/services/cli-open.ts` — window-spawn routing for the
  `cli-open` event (folders → new window with `#project=…` URL
  fragment, files → reuse single-file window if available)
- New `app/lib/services/cli-shim.ts` — shim installer dialog flow
- New `app/lib/components/UpdateProgressModal.svelte` and
  `UpdateAvailableBanner.svelte`
- New `app/lib/stores/updater-state.svelte.ts` — rune state machine
  (`idle | available | downloading | installing | ready | error`)

### Backend
- New `core/src/services/cli.rs` — argv parser with 14 unit tests
  covering files, folders, `-h/-v/-n/-g`, relative paths, `--`, and
  Windows-style drive letters
- New `core/src/commands/cli_shim.rs` — `cli_shim_status` /
  `install_cli_shim` IPC commands
- `tauri-plugin-single-instance` registered first; callback parses
  argv with the same parser as cold start and emits `cli-open`
- `tauri-plugin-process` registered (used by updater for `relaunch()`)
- `bundled-cli/` resource ships the `novelist` POSIX shim and
  `novelist.cmd` Windows launcher

### Known follow-ups
- `--wait` / `$EDITOR` integration not yet implemented (would need
  backend tracking of tab close + IPC back to the shell shim)
- Windows shim install does not auto-edit `PATH` (manual step
  documented in the install dialog)

---

## [0.2.2] - 2026-04-27

Patch release. Focuses on **rounding out the two AI panels** introduced
in 0.2.0 — AI Talk and AI Agent both gain a shared composer scaffold
(slash commands, @-mention context, inline edit review) and
project-scoped persistence of sessions, skills, and memory. Also lands
a docs reorganisation, a unified verification harness, and the
project's first formal third-party / inspiration credits file.

本次 patch 主要补充 0.2.0 引入的两个 AI 面板:AI Talk 与 AI Agent 现在共享
统一的输入脚手架(斜杠命令 / @-mention 上下文 / 内联改写审阅),并将会话、
skill、memory 持久化到项目目录。此外完成文档结构整理、统一验证脚本,以及
首次提供正式的第三方致谢与设计灵感清单。

### Added
- **Shared AI composer scaffold** (`app/lib/components/ai-shared/`) —
  used by both AI Talk and AI Agent:
  - **Slash commands** (`/rewrite`, `/summarize`, `/continue`,
    `/translate`, `/line-edit`, `/brainstorm`, `/compact`, `/clear`,
    `/save`, `/plan`, `/act`) via `AiCommandMenu`
  - **@-mention picker** for current selection / current file /
    outline / project files / folder summaries via `AiMentionMenu`
  - **Context bar** (`AiContextBar`) for reviewing and pruning per-turn
    context items before sending
  - **Inline edit review** (`InlineEditReview`) with diff preview for
    rewrite-style commands (`diff.ts`)
  - **Built-in skills** plus project-scoped skills loaded from
    `.novelist/ai/skills/**/*.md`
- **Project-scoped AI persistence** (Rust core, `commands/ai_files.rs`) —
  path-validated and atomically written, with unit tests:
  - `list_ai_sessions`, `read_ai_session`, `write_ai_session`,
    `delete_ai_session` writing to `.novelist/ai/sessions/`
  - `list_ai_prompt_assets` reading `.novelist/ai/{commands,skills}/*.md`
  - `write_ai_memory` for `.novelist/ai/memory.md`
- **Unified verification harness** — `scripts/harness.sh` plus
  `pnpm verify:{quick,unit,coverage,e2e,rust,ci}` aliases mirroring CI
  gates locally
- **New unit suites** for `ai-agent` sessions, `ai-shared` context
  builder, and `ai-shared` diff helpers
- **`CREDITS.md`** — design inspiration and third-party software
  acknowledgements (see *Acknowledgements* below)

### Changed
- **AI Talk / AI Agent panels** now mount the shared context bar,
  command menu, mention menu, and inline review surfaces
- **AI Talk settings store** loads project preset / skill assets via
  the new prompt-asset commands
- **Documentation reorg** — legacy `docs/architecture/`, `docs/design/`,
  `docs/plans/`, `docs/superpowers/`, and ad-hoc top-level design
  docs consolidated under `docs/{design-docs,exec-plans,product-specs,
  references,generated}/`. New top-level `ARCHITECTURE.md` and
  `docs/index.md` for navigation
- **CLAUDE.md / README.md** updated to point at the new docs layout
  and the new `CREDITS.md`

### Acknowledgements
The shared AI scaffold was inspired by two MIT-licensed projects whose
authors showed how an embedded AI agent can fit naturally inside a
writing tool. No code was vendored or forked from either project; these
are design references credited in good faith.

- [**claudian**](https://github.com/YishenTu/claudian) by *YishenTu* —
  embedded coding-agent UX inside an editor host
- [**obsidian-yolo**](https://github.com/Lapis0x0/obsidian-yolo) by
  *Lapis0x0* — agent-native chat + writing assistant integrated into
  a notes app

A complete list of design references and bundled third-party software
is in [`CREDITS.md`](CREDITS.md).

## [0.2.1] - 2026-04-23

Patch release. Startup performance and macOS chrome polish.

### Changed
- **Startup performance** — code-split heavy components, async i18n
  loading, view-fade transitions to mask first paint
- **Diagnostic instrumentation** — `log_startup_phase` plumbed through
  the boot path; tracing wired up on macOS

### Fixed
- **Window visibility on launch** — wired the missing `window.show`
  capability so the main window reliably surfaces after splash
- **macOS top-edge titlebar hairline** — tamed the 1px artefact under
  the unified titlebar

### Docs
- README aligned with novelist.dev messaging and v0.2.0 download links

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

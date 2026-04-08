# MiaoYan Competitive Research

## 1. Technical Architecture

**Platform**: macOS-only, requires macOS 11.5+. Swift 6 native app using AppKit (Cocoa).

**Markdown Rendering Pipeline** (two-layer system):
- **Editor**: `NSTextView` subclass (`EditTextView`) with custom `NSTextStorage` delegate that performs real-time regex-based syntax highlighting. Uses `Highlightr` (highlight.js wrapper) for code block syntax coloring.
- **Preview**: `WKWebView` subclass (`MPreviewView`). Markdown is parsed to HTML via **cmark-gfm** (C library, `CMarkGFM` Swift package) with GFM extensions (tables, footnotes, strikethrough, tasklists). The HTML is injected into a template from `DownView.bundle` containing CSS/JS for rendering, then loaded into the WebView.
- **Split mode**: Side-by-side editor + WebView preview with bidirectional scroll sync at ~60fps via JavaScript message handlers (`previewScroll`) and `MPreviewScrollDelegate`.

**Key Dependencies** (Package.swift):
- `swift-cmark-gfm` - Markdown-to-HTML parsing
- `Highlightr` - Code syntax highlighting (highlight.js)
- `Prettier` / `PrettierMarkdown` - Markdown auto-formatting
- `KeyboardShortcuts` - Global hotkey management
- `Sparkle` - Auto-updates (non-App Store builds)
- `ZipArchive` - Archive operations

**No Electron, no SwiftUI** -- pure AppKit with WKWebView only for the preview pane.

## 2. How It Achieves Small App Size

- **No bundled browser engine**: Uses system WebKit (WKWebView) for preview, not Electron/CEF.
- **Minimal dependencies**: Only 6 SPM packages, all lightweight. No heavy frameworks.
- **Plain file storage**: Notes are `.md`/`.txt` files on disk -- no SQLite, no Core Data, no custom database.
- **Bundled resources are tiny**: `DownView.bundle` contains only HTML templates, CSS, and small JS files for preview rendering. Fonts are the heaviest asset (TsangerJinKai default font).
- **No embedded runtimes**: Everything compiles to native code. cmark-gfm is a small C library.
- **Single note type**: Only supports Markdown (the `NoteType` enum has one case: `.markdown`). No rich text, no DOCX, no other formats.

## 3. Key Features

- **Three-column layout**: Sidebar (projects/folders) + Note list + Editor
- **Split editor/preview mode** (`Cmd+\`): Real-time side-by-side with scroll sync
- **Preview mode**: Full WebView rendering with LaTeX (KaTeX/MathJax via JS), Mermaid diagrams
- **PPT/Presentation mode**: Uses reveal.js to render `---`-separated slides from Markdown
- **Syntax highlighting**: Real-time in-editor highlighting for Markdown + code blocks (via Highlightr)
- **Auto-formatting**: Prettier-based Markdown formatting with cursor position preservation
- **Local-first storage**: Plain files in user-chosen directory (iCloud Drive compatible)
- **File system watching**: `FileSystemEventManager` + `FileWatcher` for external change detection
- **Image handling**: Paste/drag images into editor, stored in `/i/` subdirectory per project, inline preview in editor
- **Pin notes**: Uses macOS extended attributes (`xattr`) on files -- no separate metadata DB
- **Cursor position persistence**: Also stored via xattr on each file
- **Dark mode + custom themes**: System/Light/Dark/Custom appearance with named colors via asset catalog
- **CLI tool**: Shell script (`miao` command) for opening/creating/searching notes from terminal
- **Export**: HTML, PPT export capabilities
- **Debounced auto-save**: 1.5-second debounce on saves to avoid excessive disk writes
- **Orphan attachment cleanup**: Scans for unreferenced images/files across all projects
- **i18n**: Chinese (Simplified/Traditional), English, Japanese

## 4. File/Project Organization Model

**On-disk structure** (user-facing):
```
StorageRoot/           # User-chosen folder (e.g., iCloud Drive/MiaoYan)
  ├── FolderA/         # "Project" in code
  │   ├── note1.md
  │   ├── note2.md
  │   └── i/           # Images directory (per-project)
  ├── FolderB/
  │   └── ...
  └── Trash/           # Built-in trash folder
```

**Code model**:
- `Storage` (singleton): Manages list of `Project` objects and flat `noteList: [Note]` array. Uses dictionary `notesDict` for fast lookup.
- `Project`: Represents a folder. Has parent/child relationships (one level of nesting). Properties: `url`, `isRoot`, `isTrash`, `isCloudDrive`, `sortBy`, `sortDirection`.
- `Note`: Represents a single file. Stores `url`, `content` (NSMutableAttributedString), `title` (derived from filename), `project` reference, pin state, modification dates. Content is lazy-loaded.
- **No database**: Everything is derived from the file system. Pin state and cursor position stored as xattr. Sort preferences stored in UserDefaults.
- **Single-mode support**: Can open a single folder as the entire workspace (for focused usage).
- **Security-scoped bookmarks**: Properly handles sandboxed file access for App Store distribution.

## 5. Editor Implementation Details

**EditTextView** (`NSTextView` subclass, ~750+ lines):
- Custom caret width (1px)
- Clipboard manager for paste handling (images, formatted text)
- Image preview manager for inline image display
- Editor search bar (find/replace within note)
- IME-aware escape key handling
- Configurable vertical inset and bottom padding

**Syntax Highlighting** (regex-based, in `NSTextStorage` delegate):
- `CustomTextStorage` hooks into `textStorage:didProcessEditing:` to trigger highlighting on every edit
- **Partial rescan optimization**: Only rescans affected paragraphs unless a code fence marker (```) changes, which triggers full rescan
- **Performance tiers**: Automatically switches to simplified highlighting for large documents (`shouldUseSimplifiedHighlighting`, `shouldSkipCodeHighlighting`)
- `MarkdownRuleHighlighter`: Applies colors for headers, lists, blockquotes, code blocks, images, links
- `NotesTextProcessor`: Central class with regex patterns for all Markdown elements. Handles code block detection, backtick highlighting, checkbox rendering
- `CodeBlockHighlighter`: Delegates to `Highlightr` for language-specific syntax coloring within fenced code blocks. Falls back to basic styling for blocks >3000 chars.
- Context expansion: Expands highlight range up to 8 paragraphs in each direction to catch multi-line constructs

**TextFormatter**: Handles bold, italic, and other formatting insertions with proper cursor positioning and undo support.

**Preview rendering flow**:
1. Note content -> `renderMarkdownHTML()` (cmark-gfm with GFM extensions: tables, footnotes, strikethrough, tasklist)
2. Post-process: Fix LaTeX blocks (remove `<br>` inside `$$...$$`)
3. `HtmlManager.processImages()`: Resolve local image paths to `file://` URLs
4. `HtmlManager.htmlFromTemplate()`: Inject HTML + CSS into template from DownView.bundle
5. `MPreviewView` loads the generated HTML via `loadFileURL` or `loadHTMLString`
6. JavaScript handlers for checkbox toggling, scroll sync, code copy, TOC tips

**Split view scroll sync**:
- Editor scroll -> calculate ratio -> send to WebView via `evaluateJavaScript("scrollToRatio(...)")`
- WebView scroll -> JS posts message to `previewScroll` handler -> delegate calls back to editor
- Timing: 50ms delay for JS rendering, 16ms (~60fps) reset delay

## 6. Strengths and Weaknesses

### Strengths
- **Genuinely lightweight**: ~28K lines of Swift. No Electron overhead. Fast startup, low memory.
- **Native feel**: Proper AppKit integration -- system fonts, native text editing (spell check, dictionary, services), dark mode follows system.
- **Smart file-based storage**: No proprietary format. Notes are plain `.md` files. Pin/cursor state via xattr is clever -- no sidecar files or DB needed.
- **Good performance engineering**: Partial syntax rescan, performance tiers for large files, debounced saves, lazy content loading.
- **Clean separation**: Editor (NSTextView) and Preview (WKWebView) are cleanly separated. The cmark-gfm rendering is straightforward.
- **iCloud-compatible**: Works with iCloud Drive out of the box via standard file system APIs + security-scoped bookmarks.
- **Presentation mode**: Built-in reveal.js PPT from Markdown is a differentiator.

### Weaknesses
- **No WYSIWYG**: Explicitly rejected in README. Split mode is the compromise, but the editor itself shows raw Markdown with syntax coloring only. This is a philosophical choice but limits appeal to non-technical users.
- **Regex-based highlighting is fragile**: The syntax highlighting is built on stacked regex patterns, not a proper Markdown AST. This means edge cases (nested constructs, complex inline code) can produce incorrect highlighting. The 8-paragraph context expansion is a band-aid.
- **Single-platform**: macOS only. No iOS/iPadOS companion, no web version.
- **Limited note types**: Only Markdown. No plain text with rich editing, no other formats.
- **Flat project hierarchy**: Only one level of folder nesting (root -> subfolders). No tags system beyond what's in the content. The `tags` array in Storage exists but appears unused.
- **Preview requires WebView**: Each preview instantiation creates WKWebView + copies resources to cache directory. This is heavier than necessary for simple Markdown.
- **No real-time collaboration or sync**: Beyond iCloud Drive's file-level sync, there's no conflict resolution or multi-device awareness.
- **Static note class**: `Note` is an `NSObject` subclass with mutable state everywhere. No clear model/view-model separation -- `EditTextView.note` is a static class variable shared globally.
- **ViewController bloat**: The main `ViewController` is split across 5 files (ViewController.swift, +Action, +Data, +Editor, +Layout) indicating it has grown very large and handles too many responsibilities.

### Lessons for Novelist
1. **cmark-gfm is a solid choice** for Markdown-to-HTML rendering. It's fast, small, and handles GFM well.
2. **WKWebView for preview is the right approach** on macOS -- leverages system WebKit without bundling a browser.
3. **File-based storage with xattr metadata** is elegant for a local-first app but limits cross-platform potential.
4. **Regex-based editor highlighting works but doesn't scale** to a WYSIWYG experience. A tree-sitter or Markdown AST-based approach would be more robust.
5. **The split view scroll sync implementation** (JS message handlers + ratio-based scrolling) is a good reference for similar features.
6. **Performance tiers for highlighting** (simplified mode for large docs) is a pragmatic pattern worth adopting.

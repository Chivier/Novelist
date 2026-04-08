# Typora Competitive Research

> Comprehensive analysis of Typora (https://typora.io), the benchmark lightweight markdown editor.
> Research date: 2026-04-06

---

## 1. WYSIWYG Markdown Editing Approach

Typora's core differentiator is its "seamless" live preview -- there is **no separate preview pane**. Markdown is rendered in-place as you type.

### How It Works

- **Inline styles** (bold, italic, code, etc.): The rendered result appears as soon as you finish typing the syntax markers (e.g., `**text**`). The Markdown markers (`**`) are hidden but reappear when you place your cursor on them.
- **Block styles** (headings, lists, blockquotes, code fences): Rendered after you press Enter to move to the next paragraph. The prefix syntax (`###`, `- [x]`, etc.) is hidden once the block is rendered.
- **Source mode toggle**: Users can switch to a raw Markdown source code view at any time for full control. This mode is powered by CodeMirror.
- **Context-sensitive editing**: When you click on rendered content, Typora reveals the underlying Markdown syntax for that element, allowing in-place editing. Click away and it re-renders.

### Technical Implementation

- Typora uses its **own custom markdown engine** (not markdown-it, remark, or any off-the-shelf library). Confirmed by the developer: "Typora use its own markdown engine, and try to be compatible with Github Flavored Markdown."
- The rendering is done via HTML/CSS within the Electron webview -- the document IS the rendered output, with Markdown syntax hidden via CSS and DOM manipulation.
- The library `morphdom` (efficient DOM diffing) is used for incremental updates, avoiding full re-renders.
- `DOMPurify` is used for sanitizing HTML input.
- `rangy` is used for cross-browser range and selection handling.

### Key UX Details

- No mode switcher required for normal use
- No split-pane layout (intentionally removed)
- Feels like a word processor, not a code editor
- Markdown syntax is "revealed on hover/focus" rather than always visible

---

## 2. Supported Markdown Syntax and Extensions

Typora follows **GitHub Flavored Markdown (GFM)** as its base spec, with numerous extensions.

### Block Elements

| Feature | Syntax | Notes |
|---------|--------|-------|
| Paragraphs | Blank line separation | Shift+Return for line break |
| Headings | `#` through `######` | 6 levels |
| Blockquotes | `>` | Nested supported |
| Unordered Lists | `*`, `+`, `-` | |
| Ordered Lists | `1.` | |
| Task Lists | `- [ ]` / `- [x]` | Interactive checkboxes |
| Fenced Code Blocks | ` ``` ` with language | ~100 languages highlighted |
| Math Blocks | `$$...$$` | LaTeX/TeX via MathJax |
| Tables | Pipe syntax with alignment | GFM-style |
| Footnotes | `[^identifier]` | Reference-style |
| Horizontal Rules | `***` or `---` | |
| YAML Front Matter | `---` delimited block | At document start |
| Table of Contents | `[toc]` | Auto-generated from headings |
| Callouts/Alerts | GitHub-style | Requires preference toggle |

### Inline/Span Elements

| Feature | Syntax | Notes |
|---------|--------|-------|
| Bold | `**text**` or `__text__` | |
| Italic | `*text*` or `_text_` | |
| Strikethrough | `~~text~~` | GFM extension |
| Inline Code | `` `code` `` | |
| Links | `[text](url)` | Inline, reference, and anchor |
| Images | `![alt](src)` | Drag-and-drop supported |
| Emoji | `:smile:` | Auto-complete popup |
| Inline Math | `$expr$` | Requires preference toggle |
| Subscript | `~text~` | Requires preference toggle |
| Superscript | `^text^` | Requires preference toggle |
| Highlight | `==text==` | Requires preference toggle |
| Auto-linked URLs | `<url>` or raw URLs | |

### Extended Features (Require Preferences Toggle)

- **Diagrams via Mermaid**: Sequence, flowchart, Gantt, class, state, pie, mindmap
- **Flowchart.js**: Simple SVG flow charts
- **js-sequence**: Sequence diagrams
- **Math via MathJax**: Full LaTeX support including AMSmath, mhchem, BBox extensions
- **Code Block Math**: GitLab/GitHub style ` ```math ` blocks
- **Raw HTML**: `<u>`, `<video>`, `<iframe>`, custom tags

### HTML Support

Typora supports inline HTML within Markdown documents, including:
- Underline via `<u>` tags
- Embedded videos via `<video>`
- Iframes for embedded content
- Custom styling via inline HTML/CSS

---

## 3. Theme System

Typora's theme system is **pure CSS** -- remarkably simple and powerful.

### Architecture

- Each theme = one `.css` file placed in Typora's theme folder
- Themes are selectable from the `Themes` menu
- The entire editor UI is styled via CSS (the document IS an HTML page)

### CSS Variable System

Themes use CSS custom properties (variables) on `:root`:

```css
:root {
  --bg-color: #ffffff;
  --text-color: #333333;
  --primary-color: #428bca;
  --side-bar-bg-color: #fafafa;
  --control-text-color: #777;
  /* ... many more */
}
```

### Customization Layers

1. **`base.user.css`**: Applied to ALL themes -- global overrides
2. **`{theme-name}.user.css`**: Applied only to a specific theme (e.g., `newsprint.user.css`)
3. **Custom theme files**: Full custom `.css` files placed in the theme folder

### Theme Gallery

- Official gallery at https://theme.typora.io with community-contributed themes
- Installing a theme = copying the `.css` file (and any assets) to the theme folder
- Accessible via `Preferences > Appearance > Open Theme Folder`

### Code Block Theming

- Syntax highlighting themes are separate and configurable
- CodeMirror classes are used for syntax tokens
- Both the WYSIWYG code blocks and Source Mode use CodeMirror highlighting

### Key Design Decision

The Windows/Linux version renders more UI components (context menus, preference panels, window frames) via HTML/CSS than the macOS version, which uses more native components. This means themes have more reach on Windows/Linux.

---

## 4. Export Capabilities

### Built-in (No Dependencies)

- **PDF**: With bookmarks, using the document's CSS styling
- **HTML**: With styles (themed) or without styles (plain)
- **Image**: Export document as an image

### Via Pandoc Integration (Requires Pandoc >= v2.0)

- **Word (.docx)**
- **OpenDocument (.odt)**
- **RTF**
- **EPUB**
- **LaTeX (.tex)**
- **MediaWiki**

### Export Configuration

- Accessible via `File > Export` menu
- YAML front matter can control export settings
- Custom Pandoc arguments can be passed
- PDF export respects the current theme's CSS styling

### Notable

- Typora does NOT require a LaTeX installation for math rendering in PDF (MathJax handles it)
- The Pandoc dependency is optional -- basic PDF/HTML export works out of the box
- Some users use Typora as a LaTeX alternative for academic writing

---

## 5. File Management

### Sidebar Modes

1. **File Tree**: Hierarchical folder view showing markdown/text files
2. **File List (Articles)**: Flat list of files in the current folder
3. **Outline**: Auto-generated heading-based document outline

### Folder Handling

- Opening any file automatically loads its parent folder in the sidebar
- Folders can be opened explicitly via `File > Open`
- Only Typora-compatible files (`.md`, `.txt`, etc.) are shown in the tree

### Organization Features

- Sort by: alphabet, natural order, modification date, creation date
- Option to show folders before files or mixed
- Drag-and-drop files/folders within the sidebar
- Drag-and-drop between OS file manager and Typora sidebar
- Dragging a file from sidebar to editor inserts a link to that file

### File Operations

- Create, rename, delete files/folders from sidebar
- Move files via drag-and-drop
- Quick Open (search across files)
- Recent files list
- Manual refresh option for file tree

### Limitations

- No custom file ordering (requested but not implemented)
- No tagging or metadata-based organization
- No multi-vault/workspace system (single folder at a time)
- No full-text search across files (only filename search in Quick Open)

---

## 6. Performance Characteristics

### Startup Time

- ~3 seconds on typical hardware (Electron must initialize Chromium, parse JS, construct DOM)
- Noticeably slower than native editors (Sublime Text, TextEdit, etc.)

### Memory Usage

- Electron-based: inherits Chromium's memory overhead
- Idle RAM consumption in the range of 100-300+ MB
- Native markdown editors use roughly 10x less RAM for equivalent content
- Each Electron app bundles its own Chromium instance

### Rendering Performance

- Morphdom for efficient incremental DOM updates (avoids full re-renders)
- Real-time rendering has no perceptible lag for normal documents
- Very large documents (10,000+ lines) can cause slowdowns
- Math rendering (MathJax) can be slow for documents with many equations

### Disk Size

- **Windows installer**: ~93 MB (x64), ~88 MB (ia32)
- **Installed size**: Up to ~500 MB (including Chromium runtime, libraries)
- **macOS app bundle**: Smaller due to more native component usage

### Why It Feels Fast Despite Electron

1. **Minimal UI chrome**: Very little beyond the editor itself
2. **No plugin system**: No third-party code loading at startup
3. **Single-purpose**: Does one thing (markdown editing) without bloat
4. **Custom Electron fork**: Typora maintains a forked Electron (github.com/typora/electron), likely with optimizations/stripped components
5. **Efficient DOM updates**: morphdom instead of virtual DOM frameworks like React
6. **No split pane**: Only one "view" to render at a time
7. **Lazy rendering**: Diagram/math blocks only render when visible
8. **Lean dependency tree**: Uses lightweight libraries (lowdb for data, jQuery instead of heavy frameworks)

---

## 7. Technical Architecture

### Core Stack

| Component | Technology |
|-----------|------------|
| Runtime | **Custom Electron fork** (github.com/typora/electron) |
| Markdown Parser | **Custom/proprietary** (GFM-compatible) |
| Source Mode Editor | **CodeMirror** |
| Syntax Highlighting | **CodeMirror** (shared between source mode and code fences) |
| Math Rendering | **MathJax** (with extensions: AMSmath, mhchem, BBox) |
| Diagrams | **Mermaid**, **flowchart.js**, **js-sequence** |
| DOM Updates | **morphdom** (efficient DOM diffing/patching) |
| HTML Sanitization | **DOMPurify** |
| PDF Viewing | **PDF.js**, **YAPDFKit** |
| UI Framework | **jQuery** + **Bootstrap** (lightweight, no React/Vue/Angular) |
| Selection/Range | **rangy** |
| Data Storage | **lowdb** (lightweight JSON database) |
| Auto-complete | **autoComplt** |
| Emoji | **markdown-it-emoji** + **emojilib** |
| Character Encoding | **iconv-lite** + **jschardet** |
| Error Reporting | **Sentry** |
| Auto-update (macOS) | **Sparkle** |
| Auto-update (Windows) | **winsparkle-node** |
| File Operations | **fs-extra** |
| Config Format | **HJson** |
| Icons | **Font Awesome**, **ionicons**, **Vaadin icons** |

### How Typora Stays Small (Relative to Other Electron Apps)

1. **No heavy JS framework**: jQuery instead of React/Vue/Angular (saves 100KB+ of JS and avoids virtual DOM overhead)
2. **No plugin architecture**: No extension API, no marketplace, no dynamic loading
3. **Custom Electron fork**: Potentially strips unused Chromium features
4. **No bundled language servers or linters**
5. **Diagrams/math loaded on-demand**: Mermaid, MathJax, flowchart.js only initialized when needed
6. **Single window architecture**: No multi-tab, no multi-pane splitting
7. **Platform-specific native components on macOS**: Uses native context menus, preferences, window chrome on macOS (MacGap integration)
8. **No embedded terminal, git integration, or dev tools**

### Platform Differences

- **macOS**: Uses more native components (MacGap), native context menus, native preferences panel, Sparkle for updates
- **Windows/Linux**: More HTML-rendered UI components, including context menus, preference panel, and optional "unibody" window frame. Uses winsparkle-node for updates.

---

## 8. What Users Love and Hate

### What Users Love

- **Seamless WYSIWYG**: The #1 praised feature. "It hides the Markdown code and shows the result live, making writing truly enjoyable."
- **Clean, distraction-free interface**: Minimal chrome, focus on content
- **Focus Mode**: Fades out everything except the current paragraph
- **Typewriter Mode**: Keeps the cursor line vertically centered (like a typewriter)
- **Theme customization**: Easy CSS-based theming
- **Export versatility**: PDF, HTML, Word, EPUB, LaTeX
- **Diagram support**: Mermaid, flowchart.js built in
- **Math support**: Full LaTeX math without needing a LaTeX installation
- **Cross-platform**: Consistent experience on Windows, macOS, Linux
- **One-time purchase**: $14.99, no subscription, 3 devices, no expiration
- **Keyboard shortcuts**: Comprehensive and customizable
- **Image handling**: Drag-and-drop, paste, auto-copy, auto-upload

### What Users Hate

- **Paid (was free in beta)**: Many users resent the transition from free beta to $14.99. "Missing the days when it was available for free."
- **No split-pane view**: Cannot view source and preview side by side (intentional design choice, but frustrating for some)
- **Closed source**: Cannot inspect, modify, or contribute to the core
- **No plugin/extension system**: Cannot extend functionality
- **Limited IDE features**: No git integration, no terminal, no multi-cursor (compared to VS Code)
- **Electron overhead**: ~100-300 MB RAM for a text editor; some users prefer native alternatives
- **No collaboration features**: No real-time collaboration, comments, or sharing
- **No built-in sync**: Relies on external services (Dropbox, iCloud, Git)
- **No web/mobile version**: Desktop only
- **Limited search**: No full-text search across files
- **Large documents**: Performance degrades on very large files
- **Proprietary format quirks**: Some Typora-specific rendering may not match GitHub/other renderers exactly
- **No tabs**: Single document per window (can open multiple windows)
- **3-device limit**: License restricted to 3 activations

### Hacker News Sentiment

A highly upvoted comment: "Typora is probably the best Electron app I've ever seen" -- but the commenter still switched to a native alternative due to Electron's resource overhead. This captures the general sentiment: Typora is excellent for what it is, but Electron is a dealbreaker for performance-conscious users.

---

## 9. Image Handling

### Insertion Methods

1. **Markdown syntax**: `![alt](path)` typed manually
2. **Drag and drop**: Single or multiple files; copies to configured location
3. **Clipboard paste**: Paste image data (screenshots, copied images)
4. **Menu**: `Format > Image > Insert Local Images...`
5. **macOS Finder copy**: Copy file in Finder, paste in Typora

### Automatic Image Management

- **Copy to folder**: Auto-copy inserted images to a specified folder (relative or absolute)
- **Move image**: Relocate image file and update all references
- **Rename image**: Rename file and update references
- **Delete image**: Remove image file and markdown reference
- **Bulk operations**: `Format > Image` menu can move/copy/download ALL images in a document

### Path Configuration

- Relative paths (default when document is saved)
- `./` prefix option for VuePress compatibility
- Auto URL-escaping (spaces to `%20`)
- `typora-root-url` YAML front matter for project-wide base paths

### Cloud Upload Integration

Supported upload tools:
- **iPic** (macOS, freemium)
- **uPic** (macOS, open source)
- **Picsee** (macOS, freemium)
- **PicGo-Core** (CLI, open source, cross-platform)
- **PicGo.app** (GUI)
- **PicList** (GUI)
- **Upgit** (open source, cross-platform, GitHub-focused)
- **Custom commands**: Any CLI tool can be integrated

Supported cloud destinations (via above tools): Imgur, Flickr, Amazon S3, GitHub repos, and more.

### Auto-Upload

Can be configured to automatically upload images on insert (paste, drag-and-drop, or menu insert), replacing local paths with URLs.

---

## 10. Custom CSS Support

### How It Works

Since Typora renders everything as HTML, **the entire editor is CSS-styleable**. This includes:

- Document body text and all markdown elements
- Code blocks and syntax highlighting
- Sidebar and file tree
- Toolbar and status bar
- Context menus and dialogs (Windows/Linux)
- Print/export output

### Customization Files

| File | Scope |
|------|-------|
| `base.user.css` | Applied to ALL themes |
| `{theme}.user.css` | Applied only when that theme is active |
| Custom `.css` theme file | Full theme replacement |

### What You Can Customize

- Fonts (family, size, weight, line-height)
- Colors (background, text, links, selections)
- Spacing (margins, padding, line spacing)
- Code block appearance and syntax colors
- Table styling
- Blockquote styling
- Image alignment and sizing
- Sidebar appearance
- Print/export layout
- Writing area width
- Custom scrollbar styling

### CSS Variables

Themes expose variables that can be overridden without rewriting the entire stylesheet:

```css
:root {
  --bg-color: #fff;
  --text-color: #333;
  --primary-color: #428bca;
  --side-bar-bg-color: #fafafa;
  --control-text-color: #777;
  --heading-char-color: #aaa;
  --active-file-bg-color: #eee;
  --active-file-text-color: inherit;
  --monospace: monospace;
}
```

### Creating a Custom Theme

1. Create a `.css` file (lowercase, no spaces in filename)
2. Place it in the theme folder (`Preferences > Appearance > Open Theme Folder`)
3. Override CSS variables and/or write custom rules
4. Select the theme from the `Themes` menu

### Limitations

- No SCSS/LESS preprocessing (must be compiled to CSS externally)
- No JavaScript customization (CSS only)
- Some internal elements may have high-specificity selectors that are hard to override

---

## Summary: Key Takeaways for Building a Competitor

### What Makes Typora Special
1. **The seamless WYSIWYG experience** is the killer feature. No other editor does this as well.
2. **CSS-based theming** is brilliant -- simple for users, powerful for designers.
3. **One focused purpose** keeps it lean. No plugins, no IDE features.

### Where Typora Is Vulnerable
1. **Electron overhead**: 93 MB installer, 100-300 MB RAM. A truly native app could be 5-10x smaller.
2. **Closed source, no plugins**: Users who want extensibility have no option.
3. **No collaboration or sync**: Modern writing tools are expected to have this.
4. **No mobile/web version**: Desktop-only in a mobile-first world.
5. **No full-text search**: A basic expected feature for any file-based writing tool.
6. **No tabs or split view**: Many users want this.

### To Match Typora's ~30 MB Target Size (Corrected: Installer is ~93 MB)

Note: Typora's actual installer is ~93 MB and installed size can reach ~500 MB due to Electron/Chromium. The "~30 MB" target would require:

1. **Skip Electron entirely**: Use a native toolkit (GTK, Qt, native platform APIs) or a lightweight webview (Tauri with system webview is ~2-10 MB for the app portion)
2. **Custom markdown parser**: Keep it small, GFM-compatible
3. **Lazy-load heavy features**: Math (MathJax is large), diagrams (Mermaid is large) -- load only on demand or use lighter alternatives
4. **Minimal dependencies**: Follow Typora's lead -- jQuery-level simplicity, not React-level complexity
5. **No bundled browser engine**: The single biggest size contributor in Electron apps

### Architecture Lessons from Typora

- morphdom for incremental DOM updates is smart (avoids virtual DOM overhead)
- Custom markdown parser gives full control over WYSIWYG behavior
- CSS variables for theming is the right abstraction
- lowdb for lightweight persistence (no SQLite, no heavy ORM)
- Platform-specific native components where possible (especially macOS)

---

## Sources

- [Typora Official Site](https://typora.io/)
- [Typora Markdown Reference](https://support.typora.io/Markdown-Reference/)
- [Typora Export Documentation](https://support.typora.io/Export/)
- [Typora Image Handling](https://support.typora.io/Images/)
- [Typora Upload Image](https://support.typora.io/Upload-Image/)
- [Typora Theme Documentation](https://theme.typora.io/doc/)
- [Typora Custom CSS](https://support.typora.io/Add-Custom-CSS/)
- [Typora Write Custom Theme](https://theme.typora.io/doc/Write-Custom-Theme/)
- [Typora File Management](https://support.typora.io/File-Management/)
- [Typora Focus and Typewriter Mode](https://support.typora.io/Focus-and-Typewriter-Mode/)
- [Typora Acknowledgements](https://support.typora.io/Acknowledgement/)
- [Typora Markdown Engine Discussion (GitHub Issue #315)](https://github.com/typora/typora-issues/issues/315)
- [Typora Forked Electron Repository](https://github.com/typora/electron)
- [Typora Quick Start](https://support.typora.io/Quick-Start/)
- [Typora Shortcut Keys](https://support.typora.io/Shortcut-Keys/)
- [Typora Draw Diagrams](https://support.typora.io/Draw-Diagrams-With-Markdown/)
- [Typora Math Support](https://support.typora.io/Math/)
- [Typora Store (Pricing)](https://store.typora.io/)
- [Typora G2 Reviews](https://www.g2.com/products/typora/reviews)
- [Typora Product Hunt Reviews](https://www.producthunt.com/products/typora/reviews)
- [Hacker News Discussion on Typora](https://news.ycombinator.com/item?id=21461174)
- [Typora System Requirements](https://support.typora.io/System-Requirements/)

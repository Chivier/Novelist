# Novelist

A lightweight, extensible, WYSIWYG Markdown writing tool for desktop.

**Core idea**: No existing product simultaneously satisfies "lightweight (<15MB) + plugin system + multi-project + plain Markdown + WYSIWYG". Novelist fills this gap.

## Design Philosophy: Prompt as UI, Prompt as Plugin

Novelist is built around a radical idea: **your AI assistant is your customization engine**.

Instead of complex settings panels or learning a plugin API, you describe what you want in natural language, and your AI coding tool (Claude Code, Cursor, etc.) makes it happen directly in the source.

- **Prompt as Theme**: *"Make the editor look like a vintage typewriter — cream paper, dark brown text, monospace font"* → AI edits `src/lib/themes.ts`, adds a new theme.
- **Prompt as Plugin**: *"Create a plugin that counts sentences and shows a readability score"* → AI scaffolds a plugin in `~/.novelist/plugins/`.
- **Prompt as Feature**: *"Add a Pomodoro timer to the status bar"* → AI modifies the Svelte components directly.

This makes Novelist infinitely customizable without a plugin marketplace, theme store, or extension API documentation. The source code IS the API.

## Features

- **WYSIWYG Markdown editing** — Typora-style: syntax markers hidden when cursor is away
- **GFM support** — tables, strikethrough, autolinks, task lists
- **Image preview** — inline image rendering, paste/drag-drop images
- **Link click** — Cmd+Click to open URLs in browser
- **Split view** — two-pane editing (Cmd+\\)
- **Draft notes** — per-file scratchpad in right panel (Cmd+Shift+D)
- **Theme system** — Light, Dark, Sepia, Nord, GitHub, Dracula + System auto
- **Outline panel** — heading navigation (Cmd+Shift+O)
- **Zen Mode** — fullscreen writing with typewriter scrolling (F11)
- **Command palette** — keyboard-driven actions (Cmd+Shift+P)
- **Plugin system** — QuickJS sandbox with read/write permissions
- **Export** — HTML, PDF, DOCX, EPUB via Pandoc with theme CSS
- **Auto-save** — 5-minute interval with atomic writes
- **File watching** — external edit detection with conflict resolution
- **CJK-aware** — proper word counting, IME composition guard
- **Large file support** — performance tiers: normal / stripped WYSIWYG / read-only
- **Project switching** — quick switch via sidebar bottom button

## Tech Stack

| Layer | Technology |
|-------|-----------|
| App Framework | [Tauri v2](https://v2.tauri.app/) |
| Frontend | [Svelte 5](https://svelte.dev/) (Runes) |
| Editor | [CodeMirror 6](https://codemirror.net/) |
| CSS | [Tailwind CSS 4](https://tailwindcss.com/) |
| Build | [Vite 6](https://vite.dev/) |
| Backend | Rust |
| IPC | [tauri-specta](https://github.com/oscartbeaumont/tauri-specta) |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://rustup.rs/) >= 1.77
- System dependencies for Tauri v2:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libcairo2-dev libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev`
  - **Windows**: [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Development

```bash
pnpm install
pnpm tauri dev
```

### Testing

```bash
pnpm test          # Frontend unit tests (vitest)
pnpm test:rust     # Rust backend tests (cargo test)
pnpm test:all      # Both
```

### Build

```bash
pnpm tauri build
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+S | Save file |
| Cmd+B | Toggle sidebar |
| Cmd+Shift+O | Toggle outline |
| Cmd+Shift+D | Toggle draft note |
| Cmd+Shift+P | Command palette |
| Cmd+\\ | Toggle split view |
| Cmd+N | New file |
| Cmd+W | Close tab |
| Cmd+G | Go to line |
| Cmd+, | Settings |
| Cmd+Click | Open link in browser |
| F11 | Toggle Zen Mode |
| Escape | Exit Zen Mode |

## Creating Themes

Edit `src/lib/themes.ts` to add a theme:

```typescript
{
  id: 'my-theme',
  name: 'My Theme',
  dark: false,
  vars: {
    '--novelist-bg': '#ffffff',
    '--novelist-text': '#333333',
    '--novelist-accent': '#0066cc',
    // ... see existing themes for all variables
  },
}
```

Or ask your AI assistant: *"Add a solarized theme to Novelist"*

## Creating Plugins

Plugins live in `~/.novelist/plugins/<id>/`:

```
~/.novelist/plugins/word-frequency/
  manifest.toml
  index.js
```

**manifest.toml**:
```toml
[plugin]
id = "word-frequency"
name = "Word Frequency"
version = "1.0.0"
permissions = ["read"]
```

**index.js**:
```javascript
novelist.registerCommand("word-freq", "Show Word Frequency", function() {
  const doc = novelist.getDocument();
  const words = doc.split(/\s+/).filter(w => w.length > 0);
  // ... your logic here
});
```

Or ask your AI: *"Create a Novelist plugin that highlights overused words"*

## Project Structure

```
src/                          # Frontend (Svelte 5 + TypeScript)
├── lib/
│   ├── components/           # UI components
│   ├── editor/               # CodeMirror 6 setup + WYSIWYG
│   ├── stores/               # Svelte 5 rune stores
│   ├── ipc/                  # Tauri IPC bindings
│   ├── themes.ts             # Theme definitions
│   └── utils/                # Utilities
├── App.svelte                # Root layout
tests/                        # Test files + README
scripts/                      # GUI automation tests
src-tauri/                    # Backend (Rust)
├── src/
│   ├── commands/             # Tauri IPC commands
│   ├── services/             # File watcher, rope, plugins
│   └── models/               # Data models
design/                       # Design docs + logo prompts
```

## License

[MIT](LICENSE) - Copyright (c) 2026 Chivier

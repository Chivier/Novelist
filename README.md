# Novelist

A lightweight, extensible, WYSIWYG Markdown writing tool for desktop.

**Core idea**: No existing product simultaneously satisfies "lightweight (<15MB) + plugin system + multi-project + plain Markdown + WYSIWYG". Novelist fills this gap.

## Features (v0.0.1)

- **WYSIWYG Markdown editing** — Typora-style: syntax markers hidden when cursor is away, revealed on focus
- **CodeMirror 6 editor** — virtual scrolling, syntax highlighting, line wrapping
- **File tree sidebar** — open any directory, browse and open `.md` files
- **Tab bar** — open/close/switch files, dirty indicator for unsaved changes
- **Outline panel** — heading navigation, click to scroll (Cmd/Ctrl+Shift+O)
- **Zen Mode** — fullscreen immersive writing with typewriter scrolling and paragraph focus (F11)
- **Command palette** — keyboard-driven actions (Cmd/Ctrl+Shift+P)
- **Auto-save** — 5-minute interval with atomic writes
- **File watching** — detects external edits (vim, git, AI tools), conflict resolution dialog
- **CJK-aware word count** — proper Chinese/Japanese character counting
- **Dark mode** — follows OS preference via CSS custom properties
- **IME compatible** — decorations freeze during CJK input composition
- **Large file support** — files >1MB automatically disable WYSIWYG for performance

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
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl+S | Save file |
| Cmd/Ctrl+B | Toggle sidebar |
| Cmd/Ctrl+Shift+O | Toggle outline |
| Cmd/Ctrl+Shift+P | Command palette |
| F11 / Cmd/Ctrl+Shift+Z | Toggle Zen Mode |
| Cmd/Ctrl+F | Find in file |
| Cmd/Ctrl+H | Find and replace |

## Project Structure

```
src/                          # Frontend (Svelte 5 + TypeScript)
├── lib/
│   ├── components/           # UI components
│   ├── editor/               # CodeMirror 6 setup + WYSIWYG
│   ├── stores/               # Svelte 5 rune stores
│   ├── ipc/                  # Auto-generated Tauri IPC bindings
│   └── utils/                # Utilities (word count, etc.)
└── App.svelte                # Root layout

src-tauri/                    # Backend (Rust)
├── src/
│   ├── commands/             # Tauri IPC commands
│   ├── services/             # File watcher, etc.
│   └── models/               # Data models
└── Cargo.toml
```

## Roadmap

- [ ] Split view (2-pane editing)
- [ ] Theme plugin system
- [ ] Welcome screen with recent projects
- [ ] Plugin system (QuickJS sandbox)
- [ ] Export via pandoc (HTML, PDF, DOCX, EPUB)
- [ ] Huge file viewport mode (>10MB with ropey)
- [ ] Project-wide search
- [ ] Multi-window support

## License

[MIT](LICENSE) - Copyright (c) 2026 Chivier

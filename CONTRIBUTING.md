# Contributing to Novelist

Thank you for your interest in contributing to Novelist!

## Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://rustup.rs/) >= 1.77
- Platform dependencies for Tauri v2:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libcairo2-dev libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev`
  - **Windows**: [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Development Setup

```bash
git clone https://github.com/aspect-build/novelist.git
cd novelist
pnpm install
pnpm tauri dev
```

## Project Structure

```
app/       # Frontend — Svelte 5, TypeScript, CodeMirror 6
core/      # Backend  — Rust, Tauri v2
docs/      # Design docs, research, plans
assets/    # DMG backgrounds, logos, icons
plugins/   # Plugin templates
tests/     # Unit, e2e, benchmark tests
scripts/   # Build scripts
```

## Development Workflow

### Adding a Rust IPC Command

1. Create or edit a file in `core/src/commands/`
2. Tag the function with `#[tauri::command]` and `#[specta::specta]`
3. Register it in `core/src/lib.rs` via `collect_commands![]`
4. Run `pnpm tauri dev` — TypeScript bindings regenerate automatically at `app/lib/ipc/commands.ts`
5. **Do not** manually edit `app/lib/ipc/commands.ts`

### Frontend Conventions

- Use Svelte 5 Runes (`$state()`, `$derived()`), not legacy stores
- Stores live in `app/lib/stores/`
- Components live in `app/lib/components/`
- Editor extensions live in `app/lib/editor/`

### Rust Conventions

- One file per domain in `core/src/commands/`
- All commands return `Result<T, AppError>`
- Use `thiserror` for error types
- Atomic file writes (write to temp file, then rename)

## Testing

```bash
pnpm test          # Frontend unit tests (vitest)
pnpm test:rust     # Rust tests (cargo test)
pnpm test:all      # Both
pnpm check         # Svelte type checking
```

### Writing Tests

- Frontend tests: `tests/unit/**/*.test.ts`
- Rust tests: inline `#[cfg(test)]` modules
- Name tests by the behavior being verified, not the function name

## Code Style

- **TypeScript/Svelte**: 2-space indent, no semicolons enforced
- **Rust**: `cargo fmt` formatting, `cargo clippy` clean
- **CJK awareness**: Always consider CJK characters in word counting, IME handling, and layout

## Pull Request Process

1. Fork the repository and create a feature branch
2. Make your changes with clear, focused commits
3. Ensure all tests pass (`pnpm test:all`)
4. Ensure type checking passes (`pnpm check`)
5. Ensure Rust lints pass (`cd core && cargo clippy`)
6. Open a PR with a clear description of what and why

## Reporting Issues

- Use the GitHub issue templates (bug report or feature request)
- For bugs: include OS, Novelist version, and steps to reproduce
- For features: describe the use case and proposed solution

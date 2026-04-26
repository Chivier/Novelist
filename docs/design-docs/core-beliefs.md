# Core Beliefs

Last updated: 2026-04-25

Novelist is for writers who want the speed and ownership of plain text
without giving up a polished writing surface.

## Product Beliefs

- Plain Markdown is the source of truth. Project metadata stays beside it,
  not inside a proprietary document format.
- The editor should feel WYSIWYG while preserving Markdown control.
- CJK writing is a first-class workflow. Word count, IME behavior, sorting,
  layout, and typography must account for it.
- Plugins, templates, canvas, kanban, and mindmap are core product surface,
  not optional experiments to hide when the app gets crowded.
- "Prompt as UI" means the source tree is intentionally understandable to
  coding assistants. Prefer a small kernel and clear files over opaque
  configuration machinery.

## Engineering Beliefs

- Keep UI work in Svelte and editing work in CodeMirror.
- Keep filesystem, export, watcher, sync, plugin sandboxing, and large-file
  work in Rust.
- Prefer typed IPC boundaries and generated bindings over hand-written glue.
- Preserve observable behavior when refactoring stores, commands, and editor
  extensions.
- Add tests at the lowest useful layer, then use Playwright for user flows.

# Generated Data Schema

Last updated: 2026-04-25

Novelist currently has no relational database schema.

The app is local-first and file-backed:

- User content lives in normal Markdown and sidecar project files.
- Project metadata lives under `.novelist/`.
- Rust models in `core/src/models/` define serializable project/plugin data.
- IPC bindings are generated into `app/lib/ipc/commands.ts` by tauri-specta.

If a database is added later, this file should be generated from the schema
source of truth rather than edited by hand.

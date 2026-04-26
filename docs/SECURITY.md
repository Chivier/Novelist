# Security

Last updated: 2026-04-25

Novelist is a local desktop app, so the main security concerns are file
safety, plugin isolation, update integrity, sync boundaries, and command
surface discipline.

## Current Posture

- Filesystem access goes through Rust Tauri commands, not direct frontend
  filesystem APIs.
- IPC bindings are typed and generated through tauri-specta.
- Plugins run in a QuickJS sandbox with explicit permission tiers.
- Export is delegated to an external Pandoc binary.
- Updater support exists through Tauri's updater plugin.

## Rules For New Work

- Do not add HTTP API calls or AI model integrations to the desktop kernel.
- Do not grant plugins filesystem or network access without a product and
  security review.
- Validate paths at Rust command boundaries.
- Preserve atomic writes for user content and project metadata.
- Keep secrets out of project files, logs, and generated docs.
- Treat WebDAV and future sync integrations as untrusted network surfaces.

## Review Checklist

- Does the change introduce a new command, file operation, plugin permission,
  external process, updater behavior, or network path?
- Is the command covered by Rust tests or E2E tests?
- Does the frontend rely on generated bindings rather than hand-written IPC?
- Can an untrusted plugin reach files, shell, network, or project settings?
- Are errors explicit and recoverable for the writer?

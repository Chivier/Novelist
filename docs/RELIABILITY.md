# Reliability

Last updated: 2026-04-25

Reliability in Novelist means the writer's files are safe, the editor stays
responsive, and regressions are caught before release.

## Local Harness

```bash
pnpm harness --help
pnpm verify:quick
pnpm verify:ci
```

| Command | Purpose |
|---|---|
| `pnpm harness doctor` | Print local tool versions and project paths. |
| `pnpm verify:quick` | Fast local gate: `pnpm check`, unit tests, integration tests. |
| `pnpm verify:coverage` | Enforced Vitest coverage gate. |
| `pnpm verify:e2e` | Browser Playwright suite with mocked IPC; runs with `CI=1` so a foreign `localhost:1420` server fails fast. |
| `pnpm verify:rust` | Rust format, clippy, and tests. |
| `pnpm verify:ci` | Local mirror of CI's quality gate. |

## CI Gate

The macOS CI job runs:

- `pnpm check`
- `pnpm test:coverage`
- `pnpm test:e2e:browser`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

The Linux CI job runs Rust check and tests with Linux Tauri dependencies.

## Data Safety Rules

- User-data writes must be atomic: temp file then rename.
- Watcher self-write suppression uses BLAKE3 hashes.
- External changes must surface through conflict handling rather than silent
  overwrite.
- Generated IPC bindings must be regenerated after Rust command changes.

## Release Smoke

Before a release, run or verify:

1. `pnpm verify:ci`
2. `pnpm test:e2e:tauri` on the release platform when feasible
3. `pnpm tauri build`
4. Manual smoke: open project, type CJK text, save, rename, reopen, export

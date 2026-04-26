# Novelist Test Suite

Last updated: 2026-04-25

Novelist uses a three-tier test strategy: fast Vitest suites, browser E2E
with mocked Tauri IPC, and Rust backend tests. Use the root harness when you
want a repeatable local gate.

## Current Shape

| Layer | Current count | Command |
|---|---:|---|
| Frontend unit tests | 68 files / 978 tests | `pnpm test:unit` |
| Frontend integration tests | 3 files / 74 tests | `pnpm test:integration` |
| Browser E2E specs | 20 spec files | `pnpm test:e2e:browser` |
| Rust backend tests | 248 tests | `pnpm test:rust` |

## Harness Commands

```bash
pnpm verify:quick     # Svelte check + Vitest unit/integration
pnpm verify:coverage  # Coverage gate
pnpm verify:e2e       # Playwright browser E2E
pnpm verify:rust      # Rust fmt + clippy + tests
pnpm verify:ci        # Local CI mirror
```

## When Adding Tests

- Pure helpers, stores, and command registry behavior go in `tests/unit/`.
- CodeMirror runtime behavior that needs a DOM goes in `tests/integration/`.
- User workflows go in `tests/e2e/specs/` and should use `data-testid`.
- New IPC calls used by browser E2E need handlers in
  `tests/e2e/fixtures/tauri-mock.ts`.
- Browser-intercepted shortcuts should use the app's `window.__test_api__`
  bridge.

See [../docs/design-docs/testing.md](../docs/design-docs/testing.md),
[../docs/design-docs/testing-precision.md](../docs/design-docs/testing-precision.md),
and [COVERAGE.md](COVERAGE.md).

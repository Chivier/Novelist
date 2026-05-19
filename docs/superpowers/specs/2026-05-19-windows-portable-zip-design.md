# Windows Portable Zip Release — Design

**Date**: 2026-05-19
**Status**: Approved, awaiting implementation plan
**Scope**: Add Windows免安装 (portable) distribution alongside existing MSI/NSIS installers, supporting two variants: a standard zip that still writes user data to `%APPDATA%`, and a truly portable zip that keeps all data alongside the exe.

## Motivation

Some users want to run Novelist without an installer — for evaluation, sandboxed/locked-down corporate environments, U-disk usage, or simply preferring zip distributions. The current Windows release ships only MSI + NSIS, both of which require installation and admin elevation in many situations.

This spec adds two new Windows release artifacts. The standard zip is a trivial CI repackage; the truly portable variant requires a small Rust refactor to centralize user-data path resolution.

## Artifacts

Two new zips per Windows release, alongside the existing MSI / NSIS installers:

| Artifact | Data location | Marker file | Updater |
|----------|---------------|-------------|---------|
| `Novelist_<ver>_x64_windows.zip` | `%APPDATA%\com.novelist.desktop` | none | enabled (same as MSI/NSIS — updater downloads & runs NSIS installer; standard-zip users effectively migrate to installed builds when they accept an update) |
| `Novelist_<ver>_x64_windows-portable.zip` | `<exe_dir>\data\` | `portable.dat` next to exe | disabled — both registration in Rust and UI affordance in frontend |

Both zips contain the same `novelist.exe` binary — the only difference is the presence of the `portable.dat` marker file. Updater behavior diverges based on portable-mode detection at startup.

## Architecture

### Mode detection

A single binary supports both modes. At startup, `core/src/lib.rs` calls `services::portable::init()` very early (before any path-using code runs). The function:

1. Resolves `current_exe().canonicalize().parent()` → `exe_dir`.
2. Checks for `exe_dir/portable.dat`.
3. If absent → `PortableConfig { enabled: false, data_root: dirs::home_dir().join(".novelist") }`.
4. If present:
   - Sets `data_root = exe_dir/data`.
   - Creates `data_root` if missing.
   - Writes a probe file to confirm write access. **On failure → panic with a clear message that surfaces in a Tauri error dialog ("Portable mode: cannot write to <data_root>. Move Novelist out of Program Files or a read-only location.")**. We do not silently fall back to `%APPDATA%` — that would deceive users who explicitly chose portable mode.

Result is stored in a `OnceLock<PortableConfig>`. The detection is synchronous and runs once.

### Marker file rationale

`portable.dat` (not `.portable`):
- Visible to users in Windows Explorer (dotfiles are hidden by default).
- "DAT" reads as an opaque data file — discourages users from deleting it.

### Path API

```rust
// core/src/services/portable.rs
pub struct PortableConfig {
    pub enabled: bool,
    pub data_root: PathBuf,
}

pub fn init();                       // called once from lib.rs setup
pub fn config() -> &'static PortableConfig;
pub fn novelist_home() -> PathBuf;   // shortcut for config().data_root.clone()
pub fn is_portable() -> bool;
```

All ~8 existing `dirs::home_dir()...join(".novelist")` sites are replaced with `portable::novelist_home()`.

## Rust changes

### New file

- `core/src/services/portable.rs` — module described above, plus unit tests using `tempfile::TempDir` to inject a fake exe directory.

### Modified files

| File | Site | Change |
|------|------|--------|
| `core/src/lib.rs` | `setup()` | Call `portable::init()` first. Conditionally skip updater plugin registration when `portable::is_portable()`. Conditionally add `app.asset_protocol_scope().allow_directory(<portable_plugins>, true)` so the existing `assetProtocol.scope = $HOME/.novelist/plugins/**` in `tauri.conf.json` is augmented with the portable plugins directory. |
| `core/src/commands/recent.rs:23` | path resolution | `dirs::home_dir()...join(".novelist")` → `portable::novelist_home()` |
| `core/src/commands/template.rs:24` | path resolution | same |
| `core/src/commands/plugin.rs:14` | path resolution | same |
| `core/src/commands/settings.rs:20` | path resolution | same |
| `core/src/services/writing_stats.rs:97` | path resolution | same |
| `core/src/services/sync.rs:51` | path resolution | same |
| `core/src/services/snapshots.rs:19` | path resolution | same |
| `core/src/commands/claude_bridge.rs:103` | path resolution | same |

### New IPC command

`is_portable_mode() -> Result<bool, AppError>` — frontend reads this once to toggle UI affordances.

## Frontend changes

- `app/lib/services/updater.ts` (or wherever the manual-update-check trigger lives) — early-return when `is_portable_mode()` is true. Hide the "Check for updates" menu item in Settings.
- Settings UI — render a small info banner at the top when portable: "便携模式 — 数据存储于 `<data_root>`". Helps users confirm which mode they're in.

## Explicitly out of scope (YAGNI)

- Mac / Linux portable variants — user only requested Windows.
- Migration tooling (standard → portable). Users can manually copy `%APPDATA%\com.novelist.desktop\*` → `<exe>\data\` if they want to switch.
- Separate CI matrix leg for portable — same exe binary, only the zip packaging differs.
- Project directory `<project>/.novelist/` path unchanged — it's relative to the user-selected workspace, not user-data.

## CI changes

### Modify `.github/workflows/release.yml`

In the `build-other` job's Windows matrix leg, after the existing `tauri-action` "Build Tauri app" step, add a staging step:

```yaml
- name: Stage portable artifacts (Windows)
  if: matrix.platform == 'windows-latest'
  shell: pwsh
  run: |
    $ver = "${{ github.ref_name }}" -replace '^v',''
    $src = "core/target/release"
    $outDir = "$src/bundle/zip"
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null

    # ---- standard zip (APPDATA mode) ----
    $std = "Novelist_${ver}_x64_windows"
    New-Item -ItemType Directory -Path $std -Force | Out-Null
    Copy-Item "$src/novelist.exe" "$std/"
    Copy-Item "$src/resources"     "$std/" -Recurse -ErrorAction SilentlyContinue
    Copy-Item "$src/WebView2Loader.dll" "$std/" -ErrorAction SilentlyContinue
    Compress-Archive -Path "$std/*" -DestinationPath "$outDir/$std.zip" -Force

    # ---- portable zip (same-dir mode) ----
    $pt = "Novelist_${ver}_x64_windows-portable"
    Copy-Item $std $pt -Recurse
    New-Item -ItemType File -Path "$pt/portable.dat" -Force | Out-Null
    Compress-Archive -Path "$pt/*" -DestinationPath "$outDir/$pt.zip" -Force

- name: Upload portable zips to release
  if: matrix.platform == 'windows-latest'
  uses: softprops/action-gh-release@v3
  with:
    files: core/target/release/bundle/zip/*.zip
    tag_name: ${{ github.ref_name }}
```

### CI design notes

- **Why not use Tauri's bundle target system**: Tauri v2 has no `zip` bundler for Windows. Manual zip keeps full control over layout.
- **WebView2Loader.dll**: MSVC builds typically static-link the bootstrapper, but we copy defensively with `-ErrorAction SilentlyContinue`. If missing on the user's machine and not present in the zip, the app fails to launch — README must list "Windows 10/11 with WebView2 Runtime" as a prerequisite (preinstalled on modern Windows).
- **No CI smoke test in v1**: deferred. Optional follow-up: unzip portable, run `novelist.exe --version` with a 5-second timeout. Requires Rust-side `--version` support, which we don't currently have.
- **Updater (`latest.json`) and Homebrew tap unaffected** — Mac/Linux installer flow is unchanged.

## Testing

### Unit tests (Rust)

`core/src/services/portable.rs` `#[cfg(test)] mod tests`:

1. `detect_no_marker` → `enabled == false`, `data_root` points at home `.novelist`.
2. `detect_with_marker_writable` → `enabled == true`, `data_root == <exe_dir>/data`, directory created.
3. `detect_with_marker_readonly` → panics with the expected error message (use `std::panic::catch_unwind`).

Use a `detect_with_exe_dir(exe_dir: &Path) -> PortableConfig` helper so tests can inject a `TempDir`-based fake. `init()` is a thin wrapper that calls `current_exe()` and delegates.

### Frontend tests

`app/lib/services/__tests__/portable.test.ts` (or co-located): mock the `is_portable_mode` IPC, assert the updater entry is hidden and the Settings banner renders with the expected path.

### Manual verification checklist

1. ☐ Standard zip extracts and runs → data lands in `%APPDATA%\com.novelist.desktop`.
2. ☐ Portable zip extracts and runs → data lands in `<exe>\data\`; no new directory appears under `%APPDATA%`.
3. ☐ Portable mode shows the Settings banner with the active `data_root`.
4. ☐ Portable mode hides the "Check for updates" menu item.
5. ☐ Portable zip copied to a USB drive → runs on another machine, data follows.
6. ☐ Portable zip extracted into `C:\Program Files\Novelist-portable\` → startup shows an error dialog (no silent fallback).
7. ☐ Bundled plugins (canvas / mindmap / kanban) load correctly in portable mode — verifies the dynamic asset-scope extension works.

## Documentation updates

- `docs/design-docs/file-lifecycle.md` — append a "Portable mode path resolution" section.
- README — download table gains a "Windows Portable (no install)" row.
- `docs/exec-plans/` release notes template — add a portable-version bullet.

## Acceptance criteria

A tagged release `v0.x.y` produces the following on GitHub Releases, all from a single Windows CI leg:

- `Novelist_<ver>_x64_en-US.msi` (existing)
- `Novelist_<ver>_x64-setup.exe` (NSIS, existing)
- `Novelist_<ver>_x64_windows.zip` (new — standard portable, APPDATA data)
- `Novelist_<ver>_x64_windows-portable.zip` (new — true portable, same-dir data)

All manual checklist items above pass on a clean Windows 11 VM.

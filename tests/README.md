# Novelist Test Suite

## Overview

| Category | Count | Runner |
|----------|-------|--------|
| Rust Backend Unit Tests | 85 | `cargo test` (in `src-tauri/`) |
| Frontend Unit Tests | 34 | `pnpm test` (vitest) |
| GUI Automation Tests | 4 scripts | bash + cliclick + osascript |
| **Total** | **119+ automated** | |

Run everything: `pnpm test:all`

---

## 1. Rust Backend Tests (`cargo test`)

### commands/file.rs (18 tests)
| Test | What it verifies |
|------|-----------------|
| `test_read_file` | Read existing .md file |
| `test_read_file_not_found` | Error on missing file |
| `test_read_file_utf8_cjk` | CJK content roundtrip |
| `test_write_file_atomic` | Atomic write (temp file → rename), no leftover .novelist-tmp |
| `test_write_file_creates_new` | Write creates new file |
| `test_write_file_overwrites` | Write overwrites existing content |
| `test_list_directory` | Sort: dirs first, then alpha; hidden files excluded |
| `test_list_directory_shows_novelist_dir` | `.novelist` dir is shown (exception to hidden rule) |
| `test_list_directory_not_found` | Error on missing directory |
| `test_list_directory_not_a_dir` | Error when path is a file |
| `test_create_file` | Create empty .md, returns path |
| `test_create_file_already_exists` | Error on duplicate |
| `test_create_directory` | Create new directory |
| `test_create_directory_already_exists` | Error on duplicate |
| `test_rename_item` | Rename preserves content |
| `test_rename_item_not_found` | Error on missing source |
| `test_rename_item_target_exists` | Error on name conflict |
| `test_delete_file` | Delete single file |
| `test_delete_directory` | Recursive delete (dir + contents) |
| `test_delete_item_not_found` | Error on missing path |
| `test_read_large_file_150k_lines` | 150K-line file read completeness |

### commands/draft.rs (7 tests)
| Test | What it verifies |
|------|-----------------|
| `test_draft_path` | Draft path generation from project + file |
| `test_draft_path_nested_file` | Nested file path → flat draft name |
| `test_read_draft_not_found` | Returns None for missing draft |
| `test_write_and_read_draft` | Write → read roundtrip |
| `test_has_draft_note` | Existence check before/after write |
| `test_delete_draft_note` | Delete removes draft file |
| `test_delete_draft_nonexistent` | Delete non-existent is not an error |
| `test_write_draft_creates_dirs` | Auto-creates .novelist/drafts/ |

### commands/project.rs (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_detect_project_found` | Detect .novelist/project.toml |
| `test_detect_project_not_found` | Returns None when no config |
| `test_config_defaults` | Minimal config gets default values |

### commands/recent.rs (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_recent_project_serialize` | JSON roundtrip |
| `test_recent_projects_list_serialize` | List serialization |
| `test_dedup_and_truncate_logic` | Dedup + max 20 limit |

### error.rs (4 tests)
| Test | What it verifies |
|------|-----------------|
| `test_error_display` | Error message formatting |
| `test_error_serialize` | JSON serialization as string |
| `test_io_error_conversion` | IO error → AppError |
| `test_toml_parse_error_conversion` | TOML error → AppError |

### models/project.rs (6 tests)
| Test | What it verifies |
|------|-----------------|
| `test_full_config_deserialize` | All fields from TOML |
| `test_minimal_config_defaults` | Default values for optional fields |
| `test_serialize_roundtrip` | TOML serialize → deserialize |
| `test_writing_config_default` | WritingConfig::default() values |
| `test_partial_writing_config` | Partial config fills defaults |
| `test_custom_project_type` | Non-default project type |

### models/plugin.rs (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_manifest_deserialize` | Plugin manifest TOML parsing |
| `test_manifest_no_permissions` | Empty permissions default |
| `test_plugin_info_serialize` | JSON output structure |

### services/file_watcher.rs (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_hash_file` | BLAKE3 hash consistency and change detection |
| `test_ignore_set` | Self-write suppression within 2s window |
| `test_file_watcher_state_new` | Initial state is clean |

### services/rope_document.rs (13 tests)
| Test | What it verifies |
|------|-----------------|
| `test_state_new_is_empty` | Empty initial state |
| `test_open_and_metadata` | File → Rope, correct line/byte counts |
| `test_get_lines_basic` | Line range extraction |
| `test_get_lines_clamped` | Out-of-bounds clamping |
| `test_apply_edit_insert` | Insert text, dirty flag |
| `test_apply_edit_delete` | Delete range |
| `test_apply_edit_replace` | Replace range |
| `test_close_removes_document` | Close frees memory |
| `test_line_to_char_basic` | Line number → char offset |
| `test_cjk_content` | CJK line counting |
| `test_multiple_documents` | Multi-doc state isolation |
| `test_save_roundtrip` | Modify → save → reload matches |
| `test_viewport_reads_dont_mutate` | Read-only viewport ops are pure |

### services/rope_benchmark.rs (5 tests)
| Test | What it verifies |
|------|-----------------|
| `rope_benchmark_150k` | Performance profiling (150K lines) |
| `rope_scenario_begin_mid_end_save` | Edit at begin/mid/end → save → verify |
| `rope_webview_integration_correctness` | Multi-viewport edits + hash integrity |
| `rope_file_integrity_mandatory` | No-edit save preserves hash; edit grow exact |
| `rope_e2e_automated_integrity` | Full open→edit→jump→save→verify pipeline |
| `rope_benchmark_200k_cjk` | CJK content performance (200K lines) |

### services/plugin_host/permissions.rs (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_has_read_permission` | Read permission check |
| `test_write_implies_read` | Write grants read |
| `test_check_tier` | Permission tier hierarchy |

### services/plugin_host/sandbox.rs (13 tests)
| Test | What it verifies |
|------|-----------------|
| `test_plugin_host_new` | Empty initial state |
| `test_set_document_state` | Document state accessible to plugins |
| `test_load_plugin_basic` | Load + register command |
| `test_load_plugin_multiple_commands` | Multiple commands per plugin |
| `test_load_plugin_eval_error` | Invalid JS → error |
| `test_unload_plugin` | Unload removes plugin + commands |
| `test_invoke_command_read_only` | Read-only command returns no replacements |
| `test_invoke_command_with_write` | Write command returns replaceSelection |
| `test_invoke_command_replace_range` | replaceRange returns correct offsets |
| `test_invoke_nonexistent_plugin` | Error on missing plugin |
| `test_multiple_plugins` | Multi-plugin isolation |
| `test_reload_plugin_replaces_commands` | Reload replaces old commands |

---

## 2. Frontend Unit Tests (`pnpm test`)

### utils/wordcount.test.ts (20 tests)
| Test | What it verifies |
|------|-----------------|
| Empty string → 0 | Whitespace-only → 0 |
| Single word | Multiple words |
| Extra whitespace handling | Newlines as separators |
| Tabs as separators | CJK characters (each = 1 word) |
| Japanese characters | Fullwidth punctuation |
| Mixed CJK + Latin | CJK adjacent to Latin |
| Markdown heading | Markdown bold/emphasis |
| Multi-line markdown | Chinese novel paragraph |
| Single character | Single CJK character |
| Numbers as words | Hyphenated words |

### editor/large-file.test.ts (4 tests)
| Test | What it verifies |
|------|-----------------|
| Small files → Normal | Boundary at 1MB |
| 1-10MB → Large | >10MB → Huge |

### editor/outline.test.ts (10 tests)
| Test | What it verifies |
|------|-----------------|
| H1-H6 text extraction | Extra spaces after # |
| No space after # | Inline formatting preserved |
| CJK headings | Empty heading (# only) |
| ATXHeading node name matching | Level parsing |

---

## 3. GUI Automation Tests (bash scripts)

### scripts/test-all-features.sh
Comprehensive feature test covering:
- **Panel toggles**: Sidebar (Cmd+B), Outline (Cmd+Shift+O), Draft (Cmd+Shift+D), Settings (Cmd+,)
- **File operations**: New file (Cmd+N), type content, save (Cmd+S), close tab (Cmd+W)
- **Navigation**: Multi-line content creation, Go to Line (Cmd+G)
- **Command Palette**: Open (Cmd+Shift+P), search, close (Escape)
- **Zen Mode**: Enter (F11), exit (Escape)
- **Split View**: Toggle (Cmd+\\), re-toggle
- **Edit stability**: Edit → save → verify roundtrip
- **Draft Note**: Open/close draft panel

### scripts/test-scroll-click.sh
Large file scroll + click stability:
- Jump to lines 30K, 80K, 50K, 5K, 145K (via Cmd+G)
- Click + type markers at each position
- Scroll back to verify all markers
- Save + verify markers on disk + line count preserved

### scripts/test-scroll-edit-line.sh
Heading-aware scroll + edit test:
- Generate 5000-line file with headings at known positions
- Rapid scroll-edit cycles across 6 waypoints
- Save + verify all markers present

### scripts/test-integrity.sh + test-e2e-auto.sh
Rust rope backend integrity:
- 150K-line file integrity (BLAKE3 hash)
- Edit begin/mid/end → save → verify
- Webview+Rust integration correctness
- Full E2E pipeline with jump + viewport reads

---

## 4. In-Editor Tests (Command Palette)

| Command | What it tests |
|---------|--------------|
| Run Scroll+Edit Stability Test | 5-round scroll→click→edit→verify with markers |
| Run Performance Benchmark (150K) | EditorState creation, typing latency, scroll, word count timing |

---

## Running Tests

```bash
# Frontend unit tests
pnpm test

# Rust backend tests
pnpm test:rust

# All unit tests
pnpm test:all

# GUI tests (requires Novelist running)
./scripts/test-all-features.sh
./scripts/test-scroll-click.sh
./scripts/test-scroll-edit-line.sh

# Rust integrity tests
./scripts/test-integrity.sh
./scripts/test-e2e-auto.sh
```

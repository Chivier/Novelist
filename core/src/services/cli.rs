//! CLI argument parsing.
//!
//! Used in two places:
//!   1. `setup()` on cold start — parses `std::env::args()` and stages
//!      pending file/project paths so the first-paint frontend can drain them.
//!   2. The `tauri-plugin-single-instance` callback on hot path — receives
//!      a second invocation's argv and emits a `cli-open` event so the
//!      already-running frontend can route the request.
//!
//! The parser is deliberately simple — no clap dep, no subcommands, no shell
//! parsing tricks. We accept positional file/dir paths plus a small set of
//! flags. Anything we don't recognize is ignored (forward-compat for future
//! flags written by older shims).

use std::path::{Path, PathBuf};

const TEXT_EXTENSIONS: &[&str] = &[".md", ".markdown", ".txt", ".json", ".jsonl", ".csv"];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CliRequest {
    /// Absolute file paths the user asked to open.
    pub files: Vec<FileTarget>,
    /// Absolute directory paths the user asked to open as projects.
    pub folders: Vec<PathBuf>,
    /// `-n` / `--new-window` — force a new window even for files that
    /// would otherwise reuse an existing single-file-mode window.
    pub force_new_window: bool,
    /// `-h` / `--help` was passed. Caller should print help and exit.
    pub want_help: bool,
    /// `-v` / `--version` was passed. Caller should print version and exit.
    pub want_version: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileTarget {
    pub path: PathBuf,
    /// 1-indexed line to scroll to after open, if `-g file:LINE[:COL]` was used.
    pub line: Option<u32>,
    /// 1-indexed column. Only meaningful with `line`.
    pub col: Option<u32>,
}

/// Parse `argv[1..]`. The first element of `argv` (program name) is ignored.
/// `cwd` resolves relative paths.
pub fn parse_argv(argv: &[String], cwd: &Path) -> CliRequest {
    let mut req = CliRequest::default();
    let mut i = 0;
    let args = &argv[argv.len().min(1)..]; // skip program name if present

    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "-h" | "--help" => {
                req.want_help = true;
            }
            "-v" | "--version" => {
                req.want_version = true;
            }
            "-n" | "--new-window" => {
                req.force_new_window = true;
            }
            "-g" | "--goto" => {
                // Next arg is the target.
                if let Some(target) = args.get(i + 1) {
                    if let Some(file) = parse_goto_target(target, cwd) {
                        req.files.push(file);
                    }
                    i += 1;
                }
            }
            // `--` ends flag parsing — everything after is positional.
            "--" => {
                for rest in &args[i + 1..] {
                    classify_positional(rest, cwd, &mut req);
                }
                break;
            }
            // Allow combined `--goto=file:line` form.
            other if other.starts_with("--goto=") => {
                if let Some(file) = parse_goto_target(&other[7..], cwd) {
                    req.files.push(file);
                }
            }
            // Unknown flags: ignore (forward-compat). Anything that doesn't
            // start with '-' is positional.
            other if other.starts_with('-') && other.len() > 1 => {
                // skip
            }
            other => {
                classify_positional(other, cwd, &mut req);
            }
        }
        i += 1;
    }

    req
}

fn classify_positional(raw: &str, cwd: &Path, req: &mut CliRequest) {
    let path = resolve_path(raw, cwd);
    if !path.exists() {
        // Non-existent path: treat as a file open request anyway IF it has a
        // text extension (lets `novelist new-chapter.md` create a stub via
        // the existing file-open flow). Otherwise drop silently.
        let lower = raw.to_lowercase();
        if TEXT_EXTENSIONS.iter().any(|e| lower.ends_with(e)) {
            req.files.push(FileTarget {
                path,
                line: None,
                col: None,
            });
        }
        return;
    }
    if path.is_dir() {
        req.folders.push(canonical(&path));
    } else {
        let lower = raw.to_lowercase();
        if TEXT_EXTENSIONS.iter().any(|e| lower.ends_with(e)) {
            req.files.push(FileTarget {
                path: canonical(&path),
                line: None,
                col: None,
            });
        }
    }
}

fn parse_goto_target(raw: &str, cwd: &Path) -> Option<FileTarget> {
    // Supports `file`, `file:LINE`, `file:LINE:COL`. The `file` portion may
    // itself contain colons on Windows (`C:\foo`), so we split from the right
    // and only consume trailing :NUM components.
    let mut line: Option<u32> = None;
    let mut col: Option<u32> = None;
    let mut s = raw;

    // Try to peel a trailing :col
    if let Some((head, tail)) = rsplit_once_colon(s) {
        if let Ok(n) = tail.parse::<u32>() {
            // If head also ends with :NUM treat current tail as col.
            if let Some((head2, tail2)) = rsplit_once_colon(head) {
                if let Ok(l) = tail2.parse::<u32>() {
                    line = Some(l);
                    col = Some(n);
                    s = head2;
                } else {
                    line = Some(n);
                    s = head;
                }
            } else {
                line = Some(n);
                s = head;
            }
        }
    }

    let path = resolve_path(s, cwd);
    Some(FileTarget { path, line, col })
}

fn rsplit_once_colon(s: &str) -> Option<(&str, &str)> {
    s.rfind(':').map(|i| (&s[..i], &s[i + 1..]))
}

fn resolve_path(raw: &str, cwd: &Path) -> PathBuf {
    let p = Path::new(raw);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    }
}

fn canonical(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

/// Plain-text help blurb printed by `-h`. Kept here so the unit tests can
/// snapshot it.
pub fn help_text(program: &str, version: &str) -> String {
    format!(
        "Novelist v{version}\n\
\n\
Usage: {program} [options] [path ...]\n\
\n\
Open files or folders in Novelist. Folders always open in a new window.\n\
Files open in the current single-file window if one exists, otherwise in\n\
a new window.\n\
\n\
Options:\n\
  -h, --help              Show this help and exit\n\
  -v, --version           Show version and exit\n\
  -n, --new-window        Force a new window even for files\n\
  -g, --goto FILE:LINE[:COL]\n\
                          Open FILE and jump to LINE (and optionally COL)\n\
\n\
Examples:\n\
  {program} chapter1.md\n\
  {program} ~/novels/my-book\n\
  {program} -g chapter1.md:42:5\n\
  {program} -n draft.md\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn argv(items: &[&str]) -> Vec<String> {
        std::iter::once("novelist".to_string())
            .chain(items.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn no_args_is_empty_request() {
        let cwd = std::env::current_dir().unwrap();
        let req = parse_argv(&argv(&[]), &cwd);
        assert_eq!(req, CliRequest::default());
    }

    #[test]
    fn help_and_version_flags() {
        let cwd = std::env::current_dir().unwrap();
        assert!(parse_argv(&argv(&["-h"]), &cwd).want_help);
        assert!(parse_argv(&argv(&["--help"]), &cwd).want_help);
        assert!(parse_argv(&argv(&["-v"]), &cwd).want_version);
        assert!(parse_argv(&argv(&["--version"]), &cwd).want_version);
    }

    #[test]
    fn new_window_flag() {
        let cwd = std::env::current_dir().unwrap();
        assert!(parse_argv(&argv(&["-n"]), &cwd).force_new_window);
        assert!(parse_argv(&argv(&["--new-window"]), &cwd).force_new_window);
    }

    #[test]
    fn classifies_existing_file_and_folder() {
        let dir = TempDir::new().unwrap();
        let f = dir.path().join("note.md");
        fs::write(&f, "x").unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();

        let req = parse_argv(
            &argv(&[f.to_str().unwrap(), sub.to_str().unwrap()]),
            dir.path(),
        );
        assert_eq!(req.files.len(), 1);
        assert_eq!(req.folders.len(), 1);
        assert!(req.folders[0].ends_with("sub"));
    }

    #[test]
    fn nonexistent_text_file_is_kept_for_creation_flow() {
        let dir = TempDir::new().unwrap();
        let req = parse_argv(&argv(&["new-thing.md"]), dir.path());
        assert_eq!(req.files.len(), 1);
        assert!(req.folders.is_empty());
        assert!(req.files[0].path.ends_with("new-thing.md"));
    }

    #[test]
    fn nonexistent_unknown_extension_is_dropped() {
        let dir = TempDir::new().unwrap();
        let req = parse_argv(&argv(&["mystery.bin"]), dir.path());
        assert!(req.files.is_empty());
    }

    #[test]
    fn relative_paths_resolve_against_cwd() {
        let dir = TempDir::new().unwrap();
        let f = dir.path().join("rel.md");
        fs::write(&f, "x").unwrap();
        let req = parse_argv(&argv(&["rel.md"]), dir.path());
        assert_eq!(req.files.len(), 1);
        assert!(req.files[0].path.is_absolute());
    }

    #[test]
    fn goto_short_form_with_line() {
        let dir = TempDir::new().unwrap();
        let f = dir.path().join("c.md");
        fs::write(&f, "").unwrap();
        let req = parse_argv(&argv(&["-g", "c.md:42"]), dir.path());
        assert_eq!(req.files.len(), 1);
        assert_eq!(req.files[0].line, Some(42));
        assert_eq!(req.files[0].col, None);
    }

    #[test]
    fn goto_with_line_and_col() {
        let dir = TempDir::new().unwrap();
        let req = parse_argv(&argv(&["-g", "c.md:42:5"]), dir.path());
        assert_eq!(req.files[0].line, Some(42));
        assert_eq!(req.files[0].col, Some(5));
    }

    #[test]
    fn goto_equals_form() {
        let dir = TempDir::new().unwrap();
        let req = parse_argv(&argv(&["--goto=c.md:7"]), dir.path());
        assert_eq!(req.files.len(), 1);
        assert_eq!(req.files[0].line, Some(7));
    }

    #[test]
    fn double_dash_stops_flag_parsing() {
        let dir = TempDir::new().unwrap();
        let f = dir.path().join("-weird.md");
        fs::write(&f, "").unwrap();
        let req = parse_argv(&argv(&["--", f.to_str().unwrap()]), dir.path());
        assert_eq!(req.files.len(), 1);
    }

    #[test]
    fn unknown_flag_is_ignored() {
        let dir = TempDir::new().unwrap();
        let f = dir.path().join("note.md");
        fs::write(&f, "").unwrap();
        let req = parse_argv(&argv(&["--future-flag", f.to_str().unwrap()]), dir.path());
        assert_eq!(req.files.len(), 1);
    }

    #[test]
    fn windows_style_path_with_colon_not_treated_as_line() {
        // `C:\foo\bar.md` — the colon after C is a drive letter, not a line spec.
        // We keep `line=None` because no trailing :NUM exists.
        let cwd = std::env::current_dir().unwrap();
        let req = parse_argv(&argv(&["-g", "C:\\foo\\bar.md"]), &cwd);
        assert_eq!(req.files.len(), 1);
        assert_eq!(req.files[0].line, None);
    }

    #[test]
    fn help_text_mentions_program_and_flags() {
        let h = help_text("novelist", "0.2.3");
        assert!(h.contains("novelist"));
        assert!(h.contains("0.2.3"));
        assert!(h.contains("--new-window"));
        assert!(h.contains("--goto"));
    }
}

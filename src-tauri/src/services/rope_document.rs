//! Rope-backed document service for large file viewport editing.
//!
//! The Rope (via `ropey`) is the source of truth for large files.
//! The frontend CM6 editor only holds a window of ~2000 lines.
//! Edits are applied both locally in CM6 and dispatched here.
//!
//! Key operations:
//! - open: Load file into Rope, return metadata
//! - get_lines: Return text for a line range (viewport loading)
//! - apply_edit: Apply a text change at byte offset
//! - save: Write Rope to disk atomically
//! - close: Release memory

use crate::error::AppError;
use ropey::Rope;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct RopeDocument {
    pub rope: Rope,
    pub file_path: PathBuf,
    pub dirty: bool,
}

/// Shared state holding all open Rope documents.
pub struct RopeDocumentState {
    pub docs: Mutex<HashMap<String, RopeDocument>>,
}

impl RopeDocumentState {
    pub fn new() -> Self {
        Self {
            docs: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(serde::Serialize, specta::Type)]
pub struct RopeDocumentMeta {
    pub file_id: String,
    pub total_lines: usize,
    pub total_bytes: usize,
}

#[derive(serde::Serialize, specta::Type)]
pub struct ViewportContent {
    /// The text content for the requested line range
    pub text: String,
    /// Actual start line (0-indexed)
    pub start_line: usize,
    /// Actual end line (exclusive, 0-indexed)
    pub end_line: usize,
    /// Total lines in document (can change after edits)
    pub total_lines: usize,
}

/// Open a large file into a Rope. Returns metadata.
#[tauri::command]
#[specta::specta]
pub fn rope_open(
    path: String,
    state: tauri::State<'_, RopeDocumentState>,
) -> Result<RopeDocumentMeta, AppError> {
    let rope = Rope::from_reader(
        std::io::BufReader::new(std::fs::File::open(&path)?)
    ).map_err(|e| AppError::Custom(format!("Failed to load rope: {}", e)))?;

    let total_lines = rope.len_lines();
    let total_bytes = rope.len_bytes();
    let file_id = path.clone();

    let doc = RopeDocument {
        rope,
        file_path: PathBuf::from(&path),
        dirty: false,
    };

    state.docs.lock().unwrap().insert(file_id.clone(), doc);

    Ok(RopeDocumentMeta {
        file_id,
        total_lines,
        total_bytes,
    })
}

/// Get text for a range of lines. Lines are 0-indexed.
/// Returns the text and actual line range (clamped to document bounds).
#[tauri::command]
#[specta::specta]
pub fn rope_get_lines(
    file_id: String,
    start_line: usize,
    end_line: usize,
    state: tauri::State<'_, RopeDocumentState>,
) -> Result<ViewportContent, AppError> {
    let docs = state.docs.lock().unwrap();
    let doc = docs
        .get(&file_id)
        .ok_or_else(|| AppError::Custom(format!("Rope document not found: {}", file_id)))?;

    let total_lines = doc.rope.len_lines();
    let start = start_line.min(total_lines.saturating_sub(1));
    let end = end_line.min(total_lines);

    if start >= end {
        return Ok(ViewportContent {
            text: String::new(),
            start_line: start,
            end_line: start,
            total_lines,
        });
    }

    let start_char = doc.rope.line_to_char(start);
    let end_char = if end >= total_lines {
        doc.rope.len_chars()
    } else {
        doc.rope.line_to_char(end)
    };

    let text = doc.rope.slice(start_char..end_char).to_string();

    Ok(ViewportContent {
        text,
        start_line: start,
        end_line: end,
        total_lines,
    })
}

/// Apply an edit to the Rope at character offsets.
/// `from` and `to` are character (not byte) offsets relative to the full document.
/// `insert` is the replacement text (empty string = deletion).
#[tauri::command]
#[specta::specta]
pub fn rope_apply_edit(
    file_id: String,
    from_char: usize,
    to_char: usize,
    insert: String,
    state: tauri::State<'_, RopeDocumentState>,
) -> Result<usize, AppError> {
    let mut docs = state.docs.lock().unwrap();
    let doc = docs
        .get_mut(&file_id)
        .ok_or_else(|| AppError::Custom(format!("Rope document not found: {}", file_id)))?;

    // Clamp to valid range
    let len = doc.rope.len_chars();
    let from = from_char.min(len);
    let to = to_char.min(len);

    // Remove old text
    if to > from {
        doc.rope.remove(from..to);
    }

    // Insert new text
    if !insert.is_empty() {
        doc.rope.insert(from, &insert);
    }

    doc.dirty = true;
    Ok(doc.rope.len_lines())
}

/// Save the Rope to disk atomically.
#[tauri::command]
#[specta::specta]
pub async fn rope_save(
    file_id: String,
    state: tauri::State<'_, RopeDocumentState>,
) -> Result<(), AppError> {
    let (text, file_path, line_count) = {
        let docs = state.docs.lock().unwrap();
        let doc = docs
            .get(&file_id)
            .ok_or_else(|| AppError::Custom(format!("Rope document not found: {}", file_id)))?;
        let lines = doc.rope.len_lines();
        let text = doc.rope.to_string();
        tracing::info!(
            "[rope_save] file={}, rope_lines={}, text_bytes={}, text_lines={}",
            doc.file_path.display(), lines, text.len(), text.lines().count()
        );
        (text, doc.file_path.clone(), lines)
    };

    let temp_path = format!("{}.novelist-tmp", file_path.display());
    tokio::fs::write(&temp_path, &text).await?;
    tokio::fs::rename(&temp_path, &file_path).await?;

    tracing::info!("[rope_save] DONE: wrote {} bytes, {} lines to {}", text.len(), line_count, file_path.display());

    // Mark clean
    let mut docs = state.docs.lock().unwrap();
    if let Some(doc) = docs.get_mut(&file_id) {
        doc.dirty = false;
    }

    Ok(())
}

/// Close a Rope document and release memory.
#[tauri::command]
#[specta::specta]
pub fn rope_close(
    file_id: String,
    state: tauri::State<'_, RopeDocumentState>,
) -> Result<(), AppError> {
    state.docs.lock().unwrap().remove(&file_id);
    Ok(())
}

/// Get the character offset for a given line number (0-indexed).
#[tauri::command]
#[specta::specta]
pub fn rope_line_to_char(
    file_id: String,
    line: usize,
    state: tauri::State<'_, RopeDocumentState>,
) -> Result<usize, AppError> {
    let docs = state.docs.lock().unwrap();
    let doc = docs
        .get(&file_id)
        .ok_or_else(|| AppError::Custom(format!("Rope document not found: {}", file_id)))?;
    let clamped = line.min(doc.rope.len_lines().saturating_sub(1));
    Ok(doc.rope.line_to_char(clamped))
}

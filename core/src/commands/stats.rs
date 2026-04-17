use crate::error::AppError;
use crate::services::writing_stats::{self, WritingStatsOverview};
use serde::Deserialize;
use specta::Type;

#[derive(Deserialize, Type)]
pub struct ChapterStatsInput {
    pub file_name: String,
    pub file_path: String,
    pub word_count: usize,
}

#[tauri::command]
#[specta::specta]
pub async fn record_writing_stats(
    project_dir: String,
    word_delta: i64,
    minutes: u64,
) -> Result<(), AppError> {
    writing_stats::record_words(&project_dir, word_delta, minutes).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_writing_stats(
    project_dir: String,
    chapters: Vec<ChapterStatsInput>,
) -> Result<WritingStatsOverview, AppError> {
    let chapter_files: Vec<(String, String, usize)> = chapters
        .into_iter()
        .map(|c| (c.file_name, c.file_path, c.word_count))
        .collect();
    writing_stats::get_stats_overview(&project_dir, chapter_files).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_stats_input_deserialize() {
        let json = r#"{"file_name":"ch1.md","file_path":"/p/ch1.md","word_count":500}"#;
        let input: ChapterStatsInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.file_name, "ch1.md");
        assert_eq!(input.file_path, "/p/ch1.md");
        assert_eq!(input.word_count, 500);
    }

    #[test]
    fn test_chapter_stats_input_conversion() {
        let chapters = vec![
            ChapterStatsInput {
                file_name: "ch1.md".to_string(),
                file_path: "/p/ch1.md".to_string(),
                word_count: 1000,
            },
            ChapterStatsInput {
                file_name: "ch2.md".to_string(),
                file_path: "/p/ch2.md".to_string(),
                word_count: 500,
            },
        ];
        let converted: Vec<(String, String, usize)> = chapters
            .into_iter()
            .map(|c| (c.file_name, c.file_path, c.word_count))
            .collect();
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].0, "ch1.md");
        assert_eq!(converted[0].2, 1000);
        assert_eq!(converted[1].2, 500);
    }
}

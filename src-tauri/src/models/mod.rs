use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: u32,
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
    pub word_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseProgress {
    pub percent: u8,
    pub current_chapter: String,
    pub total_chapters: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenrePreset {
    pub id: String,
    pub label: String,
    pub prompt_keywords: String,
    pub chapter_style_id: String,
    pub preview_image_path: String,
    pub cover_image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverTemplate {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverPosition {
    pub x: f32,
    pub y: f32,
}

// 位置和字体使用相对比例 (0-1)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverPositionRatio {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizeCoverRequest {
    pub title: String,
    pub author: String,
    pub template_path: String,
    pub title_position: CoverPosition,
    pub author_position: CoverPosition,
    pub title_font_size: f32,
    pub author_font_size: f32,
    pub font_family: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportChapterRequest {
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCoverRequest {
    pub mode: Option<String>,
    pub template_path: Option<String>,
    // 绝对像素值（兼容旧版）
    pub title_position: Option<CoverPosition>,
    pub author_position: Option<CoverPosition>,
    pub title_font_size: Option<f32>,
    pub author_font_size: Option<f32>,
    // 相对比例（0-1）+ 基准尺寸
    pub title_position_ratio: Option<CoverPositionRatio>,
    pub author_position_ratio: Option<CoverPositionRatio>,
    pub title_font_ratio: Option<f32>,
    pub author_font_ratio: Option<f32>,
    pub preview_width: Option<u32>,
    pub preview_height: Option<u32>,
    pub font_family: String,
    pub title_color: Option<String>,
    pub author_color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBookRequest {
    pub title: String,
    pub author: String,
    pub chapter_style_id: String,
    pub cover: ExportCoverRequest,
    pub chapters: Vec<ExportChapterRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub phase: String,
    pub percent: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyBooksSession {
    pub host: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyBooksBookSummary {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub formats: Vec<String>,
}

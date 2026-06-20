use serde::Deserialize;
use tauri::AppHandle;

use crate::epub::builder::build_preview_epub;
use crate::parser::encoding::decode_to_utf8;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterRef {
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// 截取章节对应的原文行范围，生成单章节临时 EPUB 供前端 epub.js 预览。
#[tauri::command]
pub fn generate_preview_epub(
    app: AppHandle,
    source_path: String,
    chapter: ChapterRef,
    book_title: String,
    chapter_style_id: String,
    chapter_index: usize,
) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(&source_path).map_err(|e| format!("读取文件失败: {e}"))?;
    let content = decode_to_utf8(&bytes);
    let lines: Vec<&str> = content.lines().collect();

    if chapter.start_line == 0
        || chapter.end_line > lines.len()
        || chapter.start_line > chapter.end_line
    {
        return Err("章节行号超出文件范围".to_string());
    }
    // start_line 指向章节标题行，标题已单独渲染为标题元素，正文需跳过该行
    let body = lines[chapter.start_line..chapter.end_line].join("\n");

    build_preview_epub(
        &app,
        &book_title,
        &chapter.title,
        &body,
        &chapter_style_id,
        chapter_index,
    )
}

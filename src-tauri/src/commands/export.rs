use std::fs;
use std::path::Path;

use tauri::{AppHandle, Emitter, Manager};
use tokio::task::spawn_blocking;

use super::cover::{rasterize_cover, resolve_font_path};
use crate::epub::builder::{build_full_epub, ExportChapterInput};
use crate::models::{ExportBookRequest, ExportProgress};
use crate::parser::encoding::decode_to_utf8;

fn emit_progress(app: &AppHandle, phase: &str, percent: u8, message: &str) {
    let _ = app.emit(
        "export-progress",
        ExportProgress {
            phase: phase.to_string(),
            percent,
            message: message.to_string(),
        },
    );
}

/// 按章节行号范围从原文切出正文，叠字生成封面并组装完整 EPUB，返回完整字节内容。
fn build_epub_bytes(
    app: &AppHandle,
    source_path: &str,
    book: &ExportBookRequest,
) -> Result<Vec<u8>, String> {
    let raw_bytes = fs::read(source_path).map_err(|e| format!("读取文件失败: {e}"))?;
    let content = decode_to_utf8(&raw_bytes);
    let lines: Vec<&str> = content.lines().collect();

    let chapters: Vec<ExportChapterInput> = book
        .chapters
        .iter()
        .map(|chapter| {
            if chapter.start_line == 0
                || chapter.end_line > lines.len()
                || chapter.start_line > chapter.end_line
            {
                return Err(format!("章节《{}》行号超出文件范围", chapter.title));
            }
            Ok(ExportChapterInput {
                title: chapter.title.clone(),
                // start_line 指向章节标题行，标题已单独渲染为标题元素，正文需跳过该行
                body: lines[chapter.start_line..chapter.end_line].join("\n"),
            })
        })
        .collect::<Result<_, String>>()?;

    let cover = match &book.cover.template_path {
        Some(template_path) => {
            emit_progress(app, "cover", 0, "正在生成封面...");
            let font_path = resolve_font_path(app, &book.cover.font_family)?;

            // 预览基准尺寸（与前端 CoverPreview 的 CANVAS_WIDTH/CANVAS_HEIGHT 一致）
            const PREVIEW_WIDTH: u32 = 600;
            const PREVIEW_HEIGHT: u32 = 900;

            let (bytes, width, height) = rasterize_cover(
                Path::new(template_path),
                &font_path,
                &book.title,
                &book.author,
                // 使用相对比例
                book.cover.title_position_ratio.as_ref().map(|p| (p.x, p.y)),
                book.cover.author_position_ratio.as_ref().map(|p| (p.x, p.y)),
                book.cover.title_font_ratio,
                book.cover.author_font_ratio,
                PREVIEW_WIDTH,
                PREVIEW_HEIGHT,
                // 兼容绝对像素值
                book.cover.title_position.as_ref().map(|p| (p.x as u32, p.y as u32)),
                book.cover.author_position.as_ref().map(|p| (p.x as u32, p.y as u32)),
                book.cover.title_font_size,
                book.cover.author_font_size,
                book.cover.title_color.as_deref(),
                book.cover.author_color.as_deref(),
                book.cover.mode.as_deref() != Some("no_text"),
            )?;
            Some((bytes, width, height))
        }
        None => None,
    };
    let cover_ref = cover.as_ref().map(|(bytes, w, h)| (bytes.as_slice(), *w, *h));

    let app_for_progress = app.clone();
    build_full_epub(
        app,
        &book.title,
        &book.author,
        &book.chapter_style_id,
        cover_ref,
        &chapters,
        |phase, percent, message| emit_progress(&app_for_progress, phase, percent, message),
    )
}

/// 按章节行号范围从原文切出正文，组装完整 EPUB 并写入用户选择的路径。
#[tauri::command]
pub async fn generate_epub(
    app: AppHandle,
    source_path: String,
    book: ExportBookRequest,
    save_path: String,
) -> Result<(), String> {
    let app_for_progress = app.clone();
    let epub_bytes = spawn_blocking(move || {
        build_epub_bytes(&app_for_progress, &source_path, &book)
    })
    .await
    .map_err(|e| format!("异步任务失败: {e}"))??;

    let app_for_write = app.clone();
    spawn_blocking(move || {
        fs::write(&save_path, epub_bytes).map_err(|e| format!("保存 EPUB 失败: {e}"))?;
        emit_progress(&app_for_write, "packaging", 100, "EPUB 已保存");
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("异步任务失败: {e}"))?
}

/// 组装完整 EPUB 并写入应用缓存目录，供 MyBooks 回写流程使用，返回生成的临时文件路径。
#[tauri::command]
pub async fn generate_epub_to_cache(
    app: AppHandle,
    source_path: String,
    book: ExportBookRequest,
) -> Result<String, String> {
    let app_for_progress = app.clone();
    let epub_bytes = spawn_blocking(move || {
        build_epub_bytes(&app_for_progress, &source_path, &book)
    })
    .await
    .map_err(|e| format!("异步任务失败: {e}"))??;

    let app_for_write = app.clone();
    let dest_path = spawn_blocking(move || {
        let dest_dir = app_for_write
            .path()
            .app_cache_dir()
            .map_err(|e| format!("无法获取应用缓存目录: {e}"))?
            .join("epub-exports");
        fs::create_dir_all(&dest_dir).map_err(|e| format!("创建目录失败: {e}"))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis();
        let dest_path = dest_dir.join(format!("export-{timestamp}.epub"));
        fs::write(&dest_path, epub_bytes).map_err(|e| format!("保存 EPUB 失败: {e}"))?;

        Ok::<String, String>(dest_path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| format!("异步任务失败: {e}"))??;

    Ok(dest_path)
}

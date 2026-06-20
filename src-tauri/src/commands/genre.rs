use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::models::GenrePreset;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenrePresetRaw {
    id: String,
    prompt_keywords: String,
    chapter_style_id: String,
}

/// 每个章节样式文件夹的中文名称，定义在 chapter-styles/style-names.json 中。
fn load_style_names(app: &AppHandle) -> Result<HashMap<String, String>, String> {
    let names_path = resource_path(
        app,
        "resources/epub-assets/chapter-styles/style-names.json",
    )?;
    let raw_json = fs::read_to_string(&names_path)
        .map_err(|e| format!("读取 style-names.json 失败: {e}"))?;
    serde_json::from_str(&raw_json).map_err(|e| format!("解析 style-names.json 失败: {e}"))
}

pub(crate) fn resource_path(app: &AppHandle, relative: &str) -> Result<std::path::PathBuf, String> {
    app.path()
        .resolve(relative, tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("无法解析资源路径 {relative}: {e}"))
}

const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// 扫描章节样式目录，按文件名排序返回全部 `header-<序号>` 头图（排除装饰线 header-line）。
pub(crate) fn list_header_images(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut header_files: Vec<_> = read_dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            let filename = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            // 排除 header-line.png，只保留 header-<序号>.png
            let is_header = filename.starts_with("header-") && !filename.starts_with("header-line");
            let is_image = path
                .extension()
                .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str()))
                .unwrap_or(false);
            is_header && is_image
        })
        .collect();
    header_files.sort();
    header_files
}

/// 文件名排序后第一张 `header-*` 图片即为预览缩略图。
fn find_preview_image(dir: &Path) -> String {
    list_header_images(dir)
        .first()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// 查找样式文件夹中的 cover.png 文件
fn find_cover_image(dir: &Path) -> Option<String> {
    let cover_path = dir.join("cover.png");
    if cover_path.exists() {
        Some(cover_path.to_string_lossy().to_string())
    } else {
        None
    }
}

/// 校验用户选择的自定义样式目录：必须存在 cover.png 封面图，以及至少一张 header-*（如 header-01.png）章节头图。
/// 校验通过时返回 cover.png 的绝对路径，供前端自动导入为封面底图。
#[tauri::command]
pub fn validate_custom_style_dir(dir: String) -> Result<String, String> {
    let path = Path::new(&dir);
    if !path.is_dir() {
        return Err("所选目录不存在".to_string());
    }
    let cover_path = find_cover_image(path)
        .ok_or_else(|| "目录下缺少封面图片 cover.png".to_string())?;
    if list_header_images(path).is_empty() {
        return Err("目录下缺少章节头图（如 header-01.png）".to_string());
    }
    Ok(cover_path)
}

#[tauri::command]
pub fn get_genre_presets(app: AppHandle) -> Result<Vec<GenrePreset>, String> {
    let presets_path = resource_path(&app, "resources/genre-presets.json")?;
    let raw_json = fs::read_to_string(&presets_path)
        .map_err(|e| format!("读取 genre-presets.json 失败: {e}"))?;
    let raw_presets: Vec<GenrePresetRaw> =
        serde_json::from_str(&raw_json).map_err(|e| format!("解析 genre-presets.json 失败: {e}"))?;
    let style_names = load_style_names(&app)?;

    raw_presets
        .into_iter()
        .map(|preset| {
            let style_dir = resource_path(
                &app,
                &format!(
                    "resources/epub-assets/chapter-styles/{}",
                    preset.chapter_style_id
                ),
            )?;
            let label = style_names.get(&preset.chapter_style_id).cloned().ok_or_else(|| {
                format!(
                    "style-names.json 缺少 {} 的中文名称",
                    preset.chapter_style_id
                )
            })?;
            Ok(GenrePreset {
                preview_image_path: find_preview_image(&style_dir),
                cover_image_path: find_cover_image(&style_dir),
                id: preset.id,
                label,
                prompt_keywords: preset.prompt_keywords,
                chapter_style_id: preset.chapter_style_id,
            })
        })
        .collect()
}

use std::fs;

use tauri::AppHandle;

use crate::models::Chapter;
use crate::parser::{chapter::parse_chapters, encoding::decode_to_utf8};

#[tauri::command]
pub fn parse_txt_file(app: AppHandle, path: String, regex: String) -> Result<Vec<Chapter>, String> {
    let bytes = fs::read(&path).map_err(|e| format!("读取文件失败: {e}"))?;
    let content = decode_to_utf8(&bytes);
    parse_chapters(&content, &regex, &app)
}

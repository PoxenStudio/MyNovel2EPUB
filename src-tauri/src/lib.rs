mod commands;
mod epub;
mod models;
mod mybooks;
mod parser;

use commands::cover::{get_cover_templates, get_font_resource_path, import_cover_image, synthesize_cover};
use commands::export::{generate_epub, generate_epub_to_cache};
use commands::genre::{get_genre_presets, validate_custom_style_dir};
use commands::mybooks::{
    mybooks_fetch_txt, mybooks_login, mybooks_logout, mybooks_publish_epub,
    mybooks_restore_session, mybooks_search,
};
use commands::parse::parse_txt_file;
use commands::preview::generate_preview_epub;
use tauri::{Manager, RunEvent};

/// 应用退出时清理缓存目录：
/// - mybooks-downloads: MyBooks下载的临时TXT文件
/// - imported-covers: 用户导入的封面图片
/// - epub-exports: 导出的EPUB文件
/// - cover-previews: 封面预览生成的PNG文件
fn cleanup_cache_dirs(app: &tauri::AppHandle) {
    if let Ok(cache_dir) = app.path().app_cache_dir() {
        let dirs_to_clean = [
            "mybooks-downloads",
            "imported-covers",
            "epub-exports",
            "cover-previews",
        ];
        for dir_name in dirs_to_clean {
            let dir = cache_dir.join(dir_name);
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            parse_txt_file,
            get_genre_presets,
            validate_custom_style_dir,
            get_cover_templates,
            get_font_resource_path,
            import_cover_image,
            synthesize_cover,
            generate_preview_epub,
            generate_epub,
            generate_epub_to_cache,
            mybooks_login,
            mybooks_restore_session,
            mybooks_logout,
            mybooks_search,
            mybooks_fetch_txt,
            mybooks_publish_epub,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Exit = event {
                cleanup_cache_dirs(app_handle);
            }
        });
}

use tauri::{AppHandle, Emitter};

use crate::models::{ExportProgress, MyBooksBookSummary, MyBooksSession};
use crate::mybooks::{client, session};

#[tauri::command]
pub async fn mybooks_login(
    app: AppHandle,
    host: String,
    username: String,
    password: String,
) -> Result<MyBooksSession, String> {
    client::sign_in(&app, &host, &username, &password).await?;
    Ok(MyBooksSession { host, username })
}

#[tauri::command]
pub async fn mybooks_restore_session(app: AppHandle) -> Result<Option<MyBooksSession>, String> {
    let restored = client::restore_session(&app).await?;
    Ok(restored.map(|(host, username)| MyBooksSession { host, username }))
}

#[tauri::command]
pub fn mybooks_logout(app: AppHandle) -> Result<(), String> {
    session::clear_session(&app)
}

#[tauri::command]
pub async fn mybooks_search(
    app: AppHandle,
    query: String,
    page: Option<u32>,
) -> Result<Vec<MyBooksBookSummary>, String> {
    client::search(&app, &query, page.unwrap_or(1)).await
}

#[tauri::command]
pub async fn mybooks_fetch_txt(app: AppHandle, book_id: i64) -> Result<String, String> {
    client::fetch_txt(&app, book_id).await
}

#[tauri::command]
pub async fn mybooks_publish_epub(
    app: AppHandle,
    book_id: i64,
    epub_path: String,
    had_existing_epub: bool,
) -> Result<(), String> {
    let app_for_progress = app.clone();
    client::publish_epub(&app, book_id, &epub_path, had_existing_epub, move |phase, percent, message| {
        let _ = app_for_progress.emit(
            "export-progress",
            ExportProgress {
                phase: phase.to_string(),
                percent,
                message: message.to_string(),
            },
        );
    })
    .await
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// 落盘的会话信息：host + 用户名 + cookie 字符串。绝不包含密码。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub host: String,
    pub username: String,
    pub cookie: String,
}

fn session_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;
    Ok(dir.join("mybooks_session.json"))
}

pub fn save_session(app: &AppHandle, session: &PersistedSession) -> Result<(), String> {
    let path = session_path(app)?;
    let json = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| format!("保存会话失败: {e}"))
}

pub fn load_session(app: &AppHandle) -> Option<PersistedSession> {
    let path = session_path(app).ok()?;
    let json = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

pub fn clear_session(app: &AppHandle) -> Result<(), String> {
    let path = session_path(app)?;
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("清除会话失败: {e}"))?;
    }
    Ok(())
}

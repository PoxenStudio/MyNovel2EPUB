use std::sync::Arc;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Response, Url};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use super::session::{clear_session, load_session, save_session, PersistedSession};
use crate::models::MyBooksBookSummary;

fn base_url(host: &str) -> Result<Url, String> {
    Url::parse(host).map_err(|e| format!("host 地址无效: {e}"))
}

/// 解析 JSON 响应；失败时附带 HTTP 状态码与响应内容片段，便于定位「服务器返回的并不是预期
/// 接口」这类问题（如 host 配置错误、命中了反代/默认页而非实际 API）。
async fn parse_json<T: DeserializeOwned>(response: Response, context: &str) -> Result<T, String> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("{context}: 读取响应内容失败: {e}"))?;
    serde_json::from_str(&text).map_err(|e| {
        let snippet: String = text.chars().take(300).collect();
        format!("{context}: 解析失败（HTTP {status}）: {e}；响应内容: {snippet}")
    })
}

/// host 为字面 IP 地址时（如 127.0.0.1、192.168.x.x）通常指向用户自托管的本地/局域网
/// 部署，应跳过系统级 HTTP_PROXY/HTTPS_PROXY 环境变量（代理工具常自动写入终端配置文件，
/// 导致对本机/局域网地址的请求也被错误地经过代理转发）；域名 host 则保持默认代理行为。
fn build_client(jar: Arc<Jar>, url: &Url) -> Result<Client, String> {
    let is_ip_host = url
        .host_str()
        .is_some_and(|host| host.parse::<std::net::IpAddr>().is_ok());
    let mut builder = Client::builder().cookie_provider(jar);
    if is_ip_host {
        builder = builder.no_proxy();
    }
    builder
        .build()
        .map_err(|e| format!("创建网络客户端失败: {e}"))
}

fn jar_with_cookie(url: &Url, cookie: Option<&str>) -> Arc<Jar> {
    let jar = Jar::default();
    if let Some(cookie) = cookie {
        jar.add_cookie_str(cookie, url);
    }
    Arc::new(jar)
}

fn extract_cookie_header(jar: &Jar, url: &Url) -> Option<String> {
    jar.cookies(url)
        .and_then(|value| value.to_str().ok().map(|s| s.to_string()))
}

#[derive(Deserialize)]
struct SignInResponse {
    err: String,
    #[serde(default)]
    msg: String,
}

pub async fn sign_in(
    app: &AppHandle,
    host: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let url = base_url(host)?;
    let jar = jar_with_cookie(&url, None);
    let client = build_client(jar.clone(), &url)?;

    let endpoint = url.join("/api/user/sign_in").map_err(|e| e.to_string())?;
    let response = client
        .post(endpoint)
        .form(&[("username", username), ("password", password)])
        .send()
        .await
        .map_err(|e| format!("登录请求失败: {e}"))?;
    let parsed: SignInResponse = parse_json(response, "登录响应").await?;

    match parsed.err.as_str() {
        "ok" => {}
        "params.invalid" => return Err("用户名或密码错误".to_string()),
        "no_user" => return Err("用户不存在".to_string()),
        "permission.inactive" => return Err("账号未激活，请先完成邮箱激活".to_string()),
        _ => {
            return Err(if parsed.msg.is_empty() {
                format!("登录失败: {}", parsed.err)
            } else {
                format!("登录失败: {}", parsed.msg)
            })
        }
    }

    let cookie = extract_cookie_header(&jar, &url).ok_or("登录未返回有效会话")?;
    save_session(
        app,
        &PersistedSession {
            host: host.to_string(),
            username: username.to_string(),
            cookie,
        },
    )?;
    Ok(())
}

#[derive(Deserialize)]
struct UserInfoResponse {
    user: UserInfoUser,
}

#[derive(Deserialize)]
struct UserInfoUser {
    is_login: bool,
}

/// 用落盘的 cookie 重新校验登录状态；会话过期时自动清除本地落盘数据。
pub async fn restore_session(app: &AppHandle) -> Result<Option<(String, String)>, String> {
    let Some(session) = load_session(app) else {
        return Ok(None);
    };
    let url = base_url(&session.host)?;
    let jar = jar_with_cookie(&url, Some(&session.cookie));
    let client = build_client(jar, &url)?;

    let endpoint = url.join("/api/user/info").map_err(|e| e.to_string())?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|e| format!("会话校验请求失败: {e}"))?;
    let parsed: UserInfoResponse = parse_json(response, "用户信息响应").await?;

    if parsed.user.is_login {
        Ok(Some((session.host, session.username)))
    } else {
        let _ = clear_session(app);
        Ok(None)
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    books: Vec<RawBook>,
}

#[derive(Deserialize)]
struct RawBook {
    id: i64,
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    files: Vec<RawFile>,
}

#[derive(Deserialize)]
struct RawFile {
    format: String,
}

pub async fn search(
    app: &AppHandle,
    query: &str,
    page: u32,
) -> Result<Vec<MyBooksBookSummary>, String> {
    let session = load_session(app).ok_or("尚未登录 MyBooks")?;
    let url = base_url(&session.host)?;
    let jar = jar_with_cookie(&url, Some(&session.cookie));
    let client = build_client(jar, &url)?;

    let endpoint = url.join("/api/search").map_err(|e| e.to_string())?;
    let response = client
        .get(endpoint)
        .query(&[
            ("name", format!("title:{query}"))
        ])
        .send()
        .await
        .map_err(|e| format!("搜索请求失败: {e}"))?;
    let parsed: SearchResponse = parse_json(response, "搜索响应").await?;

    Ok(parsed
        .books
        .into_iter()
        .map(|book| MyBooksBookSummary {
            id: book.id,
            title: book.title,
            author: book.author,
            formats: book.files.into_iter().map(|file| file.format).collect(),
        })
        .collect())
}

/// 下载 TXT 到应用缓存目录，复用与本地拖拽文件相同的解析流程。
pub async fn fetch_txt(app: &AppHandle, book_id: i64) -> Result<String, String> {
    let session = load_session(app).ok_or("尚未登录 MyBooks")?;
    let url = base_url(&session.host)?;
    let jar = jar_with_cookie(&url, Some(&session.cookie));
    let client = build_client(jar, &url)?;

    let endpoint = url
        .join(&format!("/api/book/{book_id}.txt"))
        .map_err(|e| e.to_string())?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|e| format!("下载 TXT 失败: {e}"))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取 TXT 内容失败: {e}"))?;

    let dest_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("无法获取应用缓存目录: {e}"))?
        .join("mybooks-downloads");
    std::fs::create_dir_all(&dest_dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let dest_path = dest_dir.join(format!("book-{book_id}-{timestamp}.txt"));
    std::fs::write(&dest_path, &bytes).map_err(|e| format!("保存 TXT 失败: {e}"))?;

    Ok(dest_path.to_string_lossy().to_string())
}

/// MyBooks 搜索接口期望的 query string 形如 `title:<关键字>`（无 `query=` 这层 key），
/// 因此手动百分号编码关键字，而非用 `RequestBuilder::query` 生成 `key=value` 形式。
fn percent_encode_query_value(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn response_is_ok(value: &serde_json::Value) -> bool {
    value.get("err").and_then(|v| v.as_str()) == Some("ok")
}

/// 按需删除旧 epub 格式后上传新文件（携带 `bid` 作为已有书籍的新格式写入）。
pub async fn publish_epub(
    app: &AppHandle,
    book_id: i64,
    epub_path: &str,
    had_existing_epub: bool,
    mut on_progress: impl FnMut(&str, u8, &str),
) -> Result<(), String> {
    let session = load_session(app).ok_or("尚未登录 MyBooks")?;
    let url = base_url(&session.host)?;
    let jar = jar_with_cookie(&url, Some(&session.cookie));
    let client = build_client(jar, &url)?;

    on_progress("checking_format", 0, "正在检查书库中的现有格式...");

    if had_existing_epub {
        on_progress("deleting_old_format", 10, "正在删除旧的 epub 格式...");
        let endpoint = url
            .join(&format!("/api/book/{book_id}/delete_format"))
            .map_err(|e| e.to_string())?;
        let response = client
            .post(endpoint)
            .json(&serde_json::json!({ "format": "epub" }))
            .send()
            .await
            .map_err(|e| format!("删除旧格式失败: {e}"))?;
        let parsed: serde_json::Value = parse_json(response, "删除格式响应").await?;
        if !response_is_ok(&parsed) {
            return Err("删除旧 epub 格式失败".to_string());
        }
    }

    on_progress("uploading", 40, "正在上传新的 epub 文件...");
    let bytes = std::fs::read(epub_path).map_err(|e| format!("读取 epub 文件失败: {e}"))?;
    let filename = std::path::Path::new(epub_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "book.epub".to_string());

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str("application/epub+zip")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new()
        .text("bid", book_id.to_string())
        .part("ebook", part);

    let endpoint = url.join(&format!("/api/book/upload?bid={book_id}&update_metadata=0")).map_err(|e| e.to_string())?;
    let response = client
        .post(endpoint)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("上传 epub 失败: {e}"))?;
    let parsed: serde_json::Value = parse_json(response, "上传响应").await?;
    if !response_is_ok(&parsed) {
        let err_msg = parsed.get("msg").and_then(|v| v.as_str()).unwrap_or("未知错误");
        return Err(format!("上传 epub 失败: {}", err_msg));
    }

    let book_id = parsed.get("book_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let msg = parsed.get("msg").and_then(|v| v.as_str()).unwrap_or("上传成功");
    on_progress("uploading", 100, &format!("上传完成: book_id={}, msg={}", book_id, msg));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_round_trips_through_jar() {
        let url = base_url("https://mybooks.example.com").unwrap();
        let jar = jar_with_cookie(&url, None);
        jar.add_cookie_str("user_id=abc123; Path=/", &url);

        let extracted = extract_cookie_header(&jar, &url).unwrap();
        assert!(extracted.contains("user_id=abc123"));

        let restored_jar = jar_with_cookie(&url, Some(&extracted));
        let restored = extract_cookie_header(&restored_jar, &url).unwrap();
        assert!(restored.contains("user_id=abc123"));
    }

    #[test]
    fn rejects_invalid_host() {
        assert!(base_url("not a url").is_err());
    }

    #[test]
    fn response_is_ok_checks_err_field() {
        assert!(response_is_ok(&serde_json::json!({ "err": "ok" })));
        assert!(!response_is_ok(&serde_json::json!({ "err": "params.invalid" })));
        assert!(!response_is_ok(&serde_json::json!({})));
    }
}

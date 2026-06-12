use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::Serialize;

use tauri::State;
use tokio::sync::Mutex as AsyncMutex;
use evocode_config::load_config;
use evocode_proto::{ServerConfig};

pub struct BridgeState {
    handle: Arc<AsyncMutex<Option<tokio::task::JoinHandle<()>>>>,
    port: u16,
    logs: Arc<Mutex<Vec<String>>>,
}

impl BridgeState {
    fn new(port: u16) -> Self {
        Self {
            handle: Arc::new(AsyncMutex::new(None)),
            port,
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    let primary = home.join(".evocode/config.toml");
    if primary.exists() {
        return Some(primary);
    }
    let typo_compat = home.join(".evocde/config.toml");
    if typo_compat.exists() {
        return Some(typo_compat);
    }
    Some(primary)
}

fn setup_logging(logs: Arc<Mutex<Vec<String>>>) {
    // Custom timer that prints timestamps in the host's local timezone,
    // so log lines are no longer fixed to UTC.
    struct LocalTimer;
    impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
        fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
            write!(w, "{}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z"))
        }
    }
    let _ = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_timer(LocalTimer)
        .with_writer(move || {
            struct W(Arc<Mutex<Vec<String>>>);
            impl std::io::Write for W {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    if let Ok(s) = std::str::from_utf8(buf) {
                        let s = strip_ansi(s);
                        if !s.is_empty() {
                            let mut g = self.0.lock().unwrap();
                            if g.len() >= 1000 { g.remove(0); }
                            g.push(s.trim_end().to_string());
                        }
                    }
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
            }
            W(logs.clone())
        })
        .try_init();
}

fn strip_ansi(s: &str) -> String {
    let mut r = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next() == Some('[') {
                while let Some(&ch) = chars.peek() {
                    match ch {
                        '0'..='9' | ';' => { chars.next(); }
                        'm' => { chars.next(); break; }
                        _ => break,
                    }
                }
            }
        } else if c == '[' {
            let mut buf = String::from("[");
            while let Some(&ch) = chars.peek() {
                match ch {
                    '0'..='9' | ';' => { buf.push(ch); chars.next(); }
                    'm' => { buf.push('m'); chars.next(); break; }
                    _ => { buf.clear(); break; }
                }
            }
            if !buf.is_empty() && buf != "[" {
                continue;
            }
            r.push('[');
        } else {
            r.push(c);
        }
    }
    r
}

#[tauri::command]
async fn start_bridge(state: State<'_, BridgeState>) -> Result<String, String> {
    let mut handle_guard = state.handle.lock().await;
    if handle_guard.is_some() {
        return Err("Bridge is already running".into());
    }

    state.logs.lock().unwrap().clear();
    let config = load_config().unwrap_or_default();
    let codex_home = evocode_config::default_codex_home()
        .map_err(|e| e.to_string())?;

    let mut cfg = ServerConfig::default();
    cfg.codex_home = Some(codex_home);
    cfg.codex_config_overrides = config.codex_config_overrides();
    cfg.codex_env = config.codex_env();
    cfg.providers = config.provider_routes();
    cfg.upstream_url = config.base_url().unwrap_or("http://127.0.0.1:17761").to_string();
    cfg.api_key = config.api_key().unwrap_or("").to_string();
    cfg.api_key_header = config.api_key_header().to_string();
    cfg.protocol = config.protocol();
    cfg.provider = config.provider.clone().unwrap_or_default();

    setup_logging(state.logs.clone());

    let handle = tokio::spawn(async move {
        if let Err(e) = evocode_proto::serve(cfg).await {
            let msg = format!("[ERROR] {}", e);
            eprintln!("{}", msg);
            tracing::error!("bridge error: {}", e);
        }
    });

    *handle_guard = Some(handle);
    Ok(format!("Bridge starting on port {}...", state.port))
}

#[tauri::command]
async fn stop_bridge(state: State<'_, BridgeState>) -> Result<String, String> {
    let mut handle_guard = state.handle.lock().await;
    if let Some(handle) = handle_guard.take() {
        handle.abort();
        return Ok("Bridge stopped".into());
    }
    Ok("Bridge was not running".into())
}

#[tauri::command]
async fn bridge_status(state: State<'_, BridgeState>) -> Result<String, String> {
    let handle_guard = state.handle.lock().await;
    Ok(if handle_guard.is_some() { "running" } else { "stopped" }.into())
}

#[tauri::command]
async fn get_bridge_url(state: State<'_, BridgeState>) -> Result<String, String> {
    Ok(format!("http://127.0.0.1:{}", state.port))
}

#[tauri::command]
async fn get_bridge_logs(state: State<'_, BridgeState>) -> Result<Vec<String>, String> {
    let logs = state.logs.lock().unwrap();
    Ok(logs.clone())
}

#[tauri::command]
async fn read_config() -> Result<String, String> {
    let config_path = get_config_path()
        .ok_or_else(|| "Cannot find config directory".to_string())?;

    if !config_path.exists() {
        return Ok(String::new());
    }

    tokio::fs::read_to_string(&config_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_config(content: String) -> Result<(), String> {
    let config_path = get_config_path()
        .ok_or_else(|| "Cannot find config directory".to_string())?;

    tokio::fs::create_dir_all(config_path.parent().unwrap())
        .await.map_err(|e| e.to_string())?;
    tokio::fs::write(&config_path, content).await.map_err(|e| e.to_string())
}



#[tauri::command]
async fn sync_to_codex() -> Result<(), String> {
    // Reads ~/.evocode/config.toml and syncs the active provider into
    // ~/.codex/config.toml via sync_active_provider_to_codex. The provider
    // block only writes name, requires_openai_auth, and the local bridge
    // URL (127.0.0.1:17761). Other Codex config keys are left untouched.
    let config = evocode_config::load_config().map_err(|e| e.to_string())?;
    let codex_home = evocode_config::default_codex_home()
        .map_err(|e| e.to_string())?;
    config
        .sync_active_provider_to_codex(&codex_home)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").into())
}


#[tauri::command]
async fn check_update() -> Result<String, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let html = client
        .get("https://github.com/evolutions-code/evocode-tauri/releases")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|_| "Network error: cannot reach GitHub".to_string())?
        .text()
        .await
        .map_err(|_| "Failed to read response".to_string())?;

    // Find all "evocode-vX.Y.Z" patterns in the HTML
    let mut latest = String::new();
    let mut pos = 0;
    let bytes = html.as_bytes();
    let prefix = b"evocode-v";

    while pos + prefix.len() < bytes.len() {
        if bytes[pos..].starts_with(prefix) {
            let start = pos + prefix.len();
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'<' && bytes[end] != b'"' && bytes[end] != b' ' && bytes[end] != b'>' {
                end += 1;
            }
            let ver_str = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
            let parts: Vec<u64> = ver_str.split('.').filter_map(|s| s.parse().ok()).collect();
            if parts.len() == 3 {
                let latest_parts: Vec<u64> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
                if parts > latest_parts {
                    latest = ver_str.to_string();
                }
            }
            pos = end;
        } else {
            pos += 1;
        }
    }

    if latest.is_empty() {
        return Err("No releases found".to_string());
    }

    let semver_latest: Vec<u64> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    let semver_current: Vec<u64> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    let release_url = format!("https://github.com/evolutions-code/evocode-tauri/releases/tag/v{}", latest);

    if semver_latest > semver_current {
        Ok(format!("update_available__{}__{}__{}", latest, current, release_url))
    } else {
        Ok(format!("no_update__{}", current))
    }
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    pub total: u32,
    pub used: u32,
    pub rollout_path: String,
}

#[derive(Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionInfo>,
    pub total: u32,
}

#[derive(Serialize)]
pub struct SessionMessage {
    pub timestamp: String,
    pub text: String,
    pub raw: String,
}

#[tauri::command]
async fn get_sessions(offset: u32, limit: u32) -> Result<SessionsResponse, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let codex_home = home.join(".codex");

    // Find the latest state_*.sqlite file
    let db_path = std::fs::read_dir(&codex_home)
        .ok()
        .and_then(|entries| {
            let mut best: Option<std::path::PathBuf> = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("state_") && name.ends_with(".sqlite") {
                        match best {
                            None => best = Some(path),
                            Some(ref current) => {
                                let n_cur = current.file_stem().and_then(|s| s.to_str()).unwrap_or("").trim_start_matches("state_").trim_end_matches(".sqlite").parse::<u32>().unwrap_or(0);
                                let n_new = name.trim_start_matches("state_").trim_end_matches(".sqlite").parse::<u32>().unwrap_or(0);
                                if n_new > n_cur {
                                    best = Some(path);
                                }
                            }
                        }
                    }
                }
            }
            best
        })
        .ok_or_else(|| "Cannot find state database".to_string())?;

    if !db_path.exists() { return Ok(SessionsResponse { sessions: Vec::new(), total: 0 }); }

    // Read default context_window from config
    let mut default_cw: u32 = 256_000;
    if let Ok(content) = std::fs::read_to_string(codex_home.join("config.toml")) {
        for line in content.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("model_context_window = ") {
                if let Ok(n) = val.trim().trim_matches('"').parse::<u32>() { default_cw = n; break; }
            }
        }
    }

    fn model_cw(model: &str, default: u32) -> u32 {
        // Try to read the real context_window from models_catalog.json
        let codex_home = dirs::home_dir().map(|h| h.join(".codex"));
        if let Some(home) = codex_home {
            let catalog_path = home.join("models_catalog.json");
            if let Ok(json_str) = std::fs::read_to_string(&catalog_path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(models) = parsed.get("models").and_then(|m| m.as_array()) {
                        for m in models {
                            if let Some(slug) = m.get("slug").and_then(|s| s.as_str()) {
                                if slug == model {
                                    if let Some(cw) = m.get("context_window").and_then(|c| c.as_u64()) {
                                        return cw as u32;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Fallback to default from config.toml
        default
    }

    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Row;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=ro", db_path.display()))
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    // Get total count
    let total_row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM threads WHERE archived = 0 AND title != '' AND tokens_used > 0"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Count error: {}", e))?;
    let total: u32 = total_row.get("cnt");

    let rows = sqlx::query(
        "SELECT id, title, model, tokens_used, rollout_path
         FROM threads
         WHERE archived = 0 AND title != '' AND tokens_used > 0
         ORDER BY updated_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    pool.close().await;

    let sessions: Vec<SessionInfo> = rows.iter().map(|row| {
        let id: String = row.get("id");
        let name: String = row.get("title");
        let model: String = row.get("model");
        let tokens: u32 = row.get::<u32, _>("tokens_used");
        let rollout_path: String = row.get("rollout_path");
        let cw = model_cw(&model, default_cw);
        SessionInfo {
            id,
            name,
            model,
            total: (cw + 9999) / 10000,
            used: std::cmp::min((tokens + 9999) / 10000, (cw + 9999) / 10000),
            rollout_path,
        }
    }).collect();

    Ok(SessionsResponse { sessions, total })
}

#[tauri::command]
async fn get_session_content(id: String) -> Result<Vec<SessionMessage>, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let codex_home = home.join(".codex");

    // Find the latest state_*.sqlite file
    let db_path = std::fs::read_dir(&codex_home)
        .ok()
        .and_then(|entries| {
            let mut best: Option<std::path::PathBuf> = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("state_") && name.ends_with(".sqlite") {
                        match best {
                            None => best = Some(path),
                            Some(ref current) => {
                                let n_cur = current.file_stem().and_then(|s| s.to_str()).unwrap_or("").trim_start_matches("state_").trim_end_matches(".sqlite").parse::<u32>().unwrap_or(0);
                                let n_new = name.trim_start_matches("state_").trim_end_matches(".sqlite").parse::<u32>().unwrap_or(0);
                                if n_new > n_cur {
                                    best = Some(path);
                                }
                            }
                        }
                    }
                }
            }
            best
        })
        .ok_or_else(|| "Cannot find state database".to_string())?;

    if !db_path.exists() { return Ok(Vec::new()); }

    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Row;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=ro", db_path.display()))
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let row = sqlx::query(
        "SELECT rollout_path FROM threads WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    pool.close().await;

    let rollout_path: String = match row {
        Some(r) => r.get("rollout_path"),
        None => return Ok(Vec::new()),
    };

    let path = std::path::Path::new(&rollout_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Read error: {}", e))?;

    let mut messages = Vec::new();
    for line in content.lines() {
        let timestamp = serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|v| v.get("timestamp").and_then(|t| t.as_str()).map(String::from))
            .unwrap_or_default();

        // Truncate raw line to reasonable length
        let raw = if line.len() > 2000 {
            format!("{}... [truncated {} bytes]", &line[..line.floor_char_boundary(2000)], line.len() - 2000)
        } else {
            line.to_string()
        };

        messages.push(SessionMessage { timestamp, text: String::new(), raw });
    }

    // Take last 50 messages to avoid huge responses
    if messages.len() > 50 {
        messages = messages.split_off(messages.len() - 50);
    }

    Ok(messages)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(BridgeState::new(17761))
        .invoke_handler(tauri::generate_handler![
            start_bridge,
            stop_bridge,
            bridge_status,
            get_bridge_url,
            read_config,
            write_config,
            sync_to_codex,
            get_bridge_logs,
            get_app_version,
            check_update,
            get_sessions,
            get_session_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


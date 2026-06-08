use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    let _ = tracing_subscriber::fmt()
        .with_ansi(false)
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
    let config = load_config().map_err(|e| e.to_string())?;

    let mut cfg = ServerConfig::default();
    cfg.providers = config.provider_routes();
    cfg.codex_config_overrides = config.codex_config_overrides();
    cfg.codex_env = config.codex_env();
    cfg.provider = Some(config.selected_provider());
    cfg.upstream_url = config.base_url().expect("base_url is empty ").to_string();
    cfg.api_key = config.api_key().unwrap_or("").to_string();
    cfg.protocol = config.protocol();

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

    let url = "https://api.github.com/repos/evolutions-code/evocode-tauri/releases/latest";
    let resp = client
        .get(url)
        .header("User-Agent", "evocode-tauri")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| format!("{}__{}", "network_error", current))?;

    if !resp.status().is_success() {
        return Ok(format!("{}__{}", "no_update", current));
    }

    #[derive(serde::Deserialize)]
    struct Release {
        tag_name: String,
        html_url: String,
        #[allow(dead_code)]
        body: Option<String>,
    }

    let release: Release = resp
        .json()
        .await
        .map_err(|_| format!("{}__{}", "parse_error", current))?;

    let latest = release.tag_name.trim_start_matches('v').to_string();
    let semver_latest: Vec<u64> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    let semver_current: Vec<u64> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    if semver_latest > semver_current {
        Ok(format!("update_available__{}__{}__{}", latest, current, release.html_url))
    } else {
        Ok(format!("no_update__{}", current))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

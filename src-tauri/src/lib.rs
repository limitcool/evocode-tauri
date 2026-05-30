use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use serde::Deserialize;
use tauri::State;
use tokio::sync::Mutex as AsyncMutex;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use evocode_proto::{ProviderRoute, ServerConfig, WireProtocol};

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

fn load_config() -> anyhow::Result<ServerConfig> {
    let path = get_config_path().context("failed to locate home directory")?;
    if !path.exists() {
        return Ok(default_server_config());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let config: EvocodeConfig = toml::from_str(&content)?;
    Ok(config.into())
}

fn default_server_config() -> ServerConfig {
    let listen: SocketAddr = "127.0.0.1:17761".parse().unwrap();
    ServerConfig {
        listen,
        codex_bin: String::new(),
        codex_home: None,
        codex_config_overrides: Vec::new(),
        codex_env: HashMap::new(),
        cwd: None,
        model: Some("codex".to_string()),
        upstream_url: String::new(),
        api_key: String::new(),
        protocol: WireProtocol::Openai,
        providers: Vec::new(),
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EvocodeConfig {
    provider: Option<String>,
    model: Option<String>,
    providers: Option<HashMap<String, ProviderConfig>>,
    api_key: Option<String>,
    api_key_env: Option<String>,
    base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProviderConfig {
    base_url: Option<String>,
    wire_api: Option<String>,
    model: Option<String>,
    models: Option<Vec<String>>,
    api_key: Option<String>,
    api_key_env: Option<String>,
    api_key_header: Option<String>,
}

impl From<EvocodeConfig> for ServerConfig {
    fn from(config: EvocodeConfig) -> Self {
        let listen: SocketAddr = "127.0.0.1:17761".parse().unwrap();

        let providers = config.providers.map(|ps| {
            ps.into_iter().map(|(id, p)| {
                let models = p.models.unwrap_or_else(|| {
                    p.model.clone().map(|m| vec![m]).unwrap_or_else(|| vec!["*".to_string()])
                });
                ProviderRoute {
                    id,
                    base_url: p.base_url.unwrap_or_default(),
                    api_key: p.api_key.or_else(|| p.api_key_env.as_ref()
                        .and_then(|e| std::env::var(e).ok())).unwrap_or_default(),
                    api_key_header: p.api_key_header.unwrap_or_else(|| "X-Api-Key".to_string()),
                    protocol: p.wire_api.as_ref().and_then(|v| wire_api_to_protocol(v))
                        .unwrap_or(WireProtocol::Openai),
                    models,
                }
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        ServerConfig {
            listen,
            codex_bin: String::new(),
            codex_home: None,
            codex_config_overrides: Vec::new(),
            codex_env: HashMap::new(),
            cwd: None,
            model: config.model.or(Some("codex".to_string())),
            upstream_url: config.base_url.unwrap_or_default(),
            api_key: config.api_key.unwrap_or_default(),
            protocol: WireProtocol::Openai,
            providers,
        }
    }
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

struct MutexCell<T>(Arc<Mutex<T>>);
struct LogWriter(MutexCell<Vec<String>>);
impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            let s = strip_ansi(s);
            if !s.is_empty() {
                let mut g = self.0 .0.lock().unwrap();
                if g.len() >= 1000 { g.remove(0); }
                // each write call is one line - trim and push
                g.push(s.trim_end().to_string());
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn strip_ansi(s: &str) -> String {
    let mut r = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // skip ANSI escape sequence
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
            // skip soft ANSI like [2m, [0m, [32m
            let mut buf = String::from("[");
            while let Some(&ch) = chars.peek() {
                match ch {
                    '0'..='9' | ';' => { buf.push(ch); chars.next(); }
                    'm' => { buf.push('m'); chars.next(); break; }
                    _ => { buf.clear(); break; }
                }
            }
            if !buf.is_empty() && buf != "[" {
                continue; // skip this soft-ANSI sequence
            }
            r.push('[');
        } else {
            r.push(c);
        }
    }
    r
}

fn wire_api_to_protocol(v: &str) -> Option<WireProtocol> {
    match v {
        "anthropic" | "messages" => Some(WireProtocol::Anthropic),
        "chat" | "chat_completions" | "chat-completions" => Some(WireProtocol::ChatCompletions),
        "openai" | "responses" => Some(WireProtocol::Openai),
        _ => None,
    }
}

#[tauri::command]
async fn start_bridge(state: State<'_, BridgeState>) -> Result<String, String> {
    let mut handle_guard = state.handle.lock().await;
    if handle_guard.is_some() {
        return Err("Bridge is already running".into());
    }

    state.logs.lock().unwrap().clear();
    let config = load_config().map_err(|e| e.to_string())?;

    setup_logging(state.logs.clone());

    let handle = tokio::spawn(async move {
        if let Err(e) = evocode_proto::serve(config).await {
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

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;
    use tauri::Manager;

    let app_handle = app.handle().clone();

    let window = app.get_webview_window("main");

    let show_item = MenuItemBuilder::new("Show Window").id("show").build(app)?;
    let quit_item = MenuItemBuilder::new("Quit").id("quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .tooltip("evocode")
        .menu(&menu)
        .on_menu_event({
            let h = app_handle.clone();
            move |_tray, event| {
                match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = h.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        h.exit(0);
                    }
                    _ => {}
                }
            }
        })
        .build(app)?;

    // hide window on close button instead of quitting
    if let Some(win) = window {
        let handle = app_handle.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(w) = handle.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
        });
    }

    Ok(())
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(tauri::VERSION.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .manage(BridgeState::new(17761))
        .invoke_handler(tauri::generate_handler![
            start_bridge,
            stop_bridge,
            bridge_status,
            get_bridge_url,
            read_config,
            write_config,
            get_bridge_logs,
            get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

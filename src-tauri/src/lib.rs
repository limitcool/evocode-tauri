use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use evocode_config::load_config;
use evocode_proto::ServerConfig;
use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex as AsyncMutex;

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
        fn format_time(
            &self,
            w: &mut tracing_subscriber::fmt::format::Writer<'_>,
        ) -> std::fmt::Result {
            write!(
                w,
                "{}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z")
            )
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
                            if g.len() >= 1000 {
                                g.remove(0);
                            }
                            g.push(s.trim_end().to_string());
                        }
                    }
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
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
                        '0'..='9' | ';' => {
                            chars.next();
                        }
                        'm' => {
                            chars.next();
                            break;
                        }
                        _ => break,
                    }
                }
            }
        } else if c == '[' {
            let mut buf = String::from("[");
            while let Some(&ch) = chars.peek() {
                match ch {
                    '0'..='9' | ';' => {
                        buf.push(ch);
                        chars.next();
                    }
                    'm' => {
                        buf.push('m');
                        chars.next();
                        break;
                    }
                    _ => {
                        buf.clear();
                        break;
                    }
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
    let codex_home = evocode_config::default_codex_home().map_err(|e| e.to_string())?;

    let cfg = ServerConfig {
        codex_home: Some(codex_home),
        codex_config_overrides: config.codex_config_overrides(),
        codex_env: config.codex_env(),
        providers: config.provider_routes(),
        upstream_url: config
            .base_url()
            .unwrap_or("http://127.0.0.1:17761")
            .to_string(),
        api_key: config.api_key().unwrap_or("").to_string(),
        api_key_header: config.api_key_header().to_string(),
        protocol: config.protocol(),
        provider: config.provider.clone().unwrap_or_default(),
        ..Default::default()
    };

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
async fn bridge_is_running(state: State<'_, BridgeState>) -> Result<bool, String> {
    Ok(state.handle.lock().await.is_some())
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
    Ok(if handle_guard.is_some() {
        "running"
    } else {
        "stopped"
    }
    .into())
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
    let config_path =
        get_config_path().ok_or_else(|| "Cannot find config directory".to_string())?;

    if !config_path.exists() {
        return Ok(String::new());
    }

    tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_config(content: String) -> Result<(), String> {
    let config_path =
        get_config_path().ok_or_else(|| "Cannot find config directory".to_string())?;

    tokio::fs::create_dir_all(config_path.parent().unwrap())
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::write(&config_path, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_to_codex() -> Result<(), String> {
    // Reads ~/.evocode/config.toml and syncs the active provider into
    // ~/.codex/config.toml via sync_active_provider_to_codex. The provider
    // block only writes name, requires_openai_auth, and the local bridge
    // URL (127.0.0.1:17761). Other Codex config keys are left untouched.
    let config = evocode_config::load_config().map_err(|e| e.to_string())?;
    let codex_home = evocode_config::default_codex_home().map_err(|e| e.to_string())?;
    config
        .sync_active_provider_to_codex(&codex_home)
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct ConnectivityResult {
    pub ok: bool,
    pub status: u16,
    pub latency_ms: u128,
    pub message: String,
}

#[tauri::command]
async fn test_provider_connectivity(
    base_url: String,
    api_key: String,
    wire_api: String,
    api_key_header: Option<String>,
) -> Result<ConnectivityResult, String> {
    // Trim trailing slash and pick the right probe endpoint per protocol.
    let base = base_url.trim_end_matches('/').to_string();
    if base.is_empty() {
        return Err("Base URL is empty".into());
    }

    // Pick a probe endpoint that is cheap and indicates the upstream is alive.
    // Anthropic: GET /v1/messages (returns 405 for GET, but proves reachability + auth wiring).
    // OpenAI-compatible (chat_completions / openai Responses): GET /models (returns 200 or 401).
    let probe_path = match wire_api.as_str() {
        "anthropic" => "/v1/messages",
        _ => "/models",
    };
    let url = format!("{}{}", base, probe_path);

    let header_name = api_key_header
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(if wire_api == "anthropic" {
            "X-Api-Key"
        } else {
            "Authorization"
        });

    let header_value = if wire_api == "anthropic" {
        api_key.clone()
    } else if header_name.eq_ignore_ascii_case("Authorization") {
        format!("Bearer {}", api_key)
    } else {
        api_key.clone()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let started = std::time::Instant::now();
    let resp = client
        .get(&url)
        .header(header_name, header_value)
        .header("Accept", "application/json")
        .send()
        .await;
    let latency_ms = started.elapsed().as_millis();

    match resp {
        Ok(r) => {
            let status = r.status().as_u16();
            // Treat anything < 500 (incl. 401/403/404/405) as reachable.
            // 5xx and connection-level failures are considered not ok.
            let ok = status < 500;
            let message = if ok {
                if (200..300).contains(&status) {
                    format!("Reachable (HTTP {})", status)
                } else if status == 401 || status == 403 {
                    format!("Reachable, auth rejected (HTTP {})", status)
                } else if status == 404 || status == 405 {
                    format!("Reachable, endpoint not found (HTTP {})", status)
                } else {
                    format!("Reachable (HTTP {})", status)
                }
            } else {
                format!("Server error (HTTP {})", status)
            };
            Ok(ConnectivityResult {
                ok,
                status,
                latency_ms,
                message,
            })
        }
        Err(e) => {
            let kind = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "connect failed"
            } else {
                "request failed"
            };
            Ok(ConnectivityResult {
                ok: false,
                status: 0,
                latency_ms,
                message: format!("{}: {}", kind, e),
            })
        }
    }
}

#[tauri::command]
async fn open_config_dir(app: AppHandle) -> Result<String, String> {
    // Ensure ~/.evocode exists, then open it with the system file manager.
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "Cannot find home directory".to_string())?;
    let dir = home.join(".evocode");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;

    let path_str = dir.to_string_lossy().to_string();
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path_str.clone(), None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(path_str)
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
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
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
            while end < bytes.len()
                && bytes[end] != b'<'
                && bytes[end] != b'"'
                && bytes[end] != b' '
                && bytes[end] != b'>'
            {
                end += 1;
            }
            let ver_str = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
            let parts: Vec<u64> = ver_str.split('.').filter_map(|s| s.parse().ok()).collect();
            if parts.len() == 3 {
                let latest_parts: Vec<u64> =
                    latest.split('.').filter_map(|s| s.parse().ok()).collect();
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

    let release_url = format!(
        "https://github.com/evolutions-code/evocode-tauri/releases/tag/v{}",
        latest
    );

    if semver_latest > semver_current {
        Ok(format!(
            "update_available__{}__{}__{}",
            latest, current, release_url
        ))
    } else {
        Ok(format!("no_update__{}", current))
    }
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    /// Context window size in 10K-token cells (model_context_window / 10000). The card denominator.
    pub total: u32,
    /// Current uncompressed window in 10K-token cells (last_token_usage / 10000, capped at context window). The card numerator.
    pub used: u32,
    /// Exact context window in tokens (model_context_window). Used for the
    /// precise token-count text on the card; the `total` cell field is
    /// rounded up to 10K-cell granularity and can over-report by up to
    /// 10K tokens, so the UI must NOT recompute it as `total * 10000`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    /// Exact current window in tokens (last_token_usage.total_tokens). Same
    /// reason as `total_tokens`: `used * 10000` would round the displayed
    /// number up to the next 10K boundary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_tokens: Option<u64>,
    /// Whole-session accumulated tokens (survives multiple compact events).
    /// u64 because long-lived sessions can easily exceed u32::MAX.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accumulated: Option<u64>,
    pub rollout_path: String,
}

/// Reads the most recent `token_count` event from a rollout `.jsonl` file.
/// Returns (recent_window_tokens, accumulated_tokens, model_context_window).
///
/// The rollout file records `token_count` events emitted by Codex, each containing
/// `total_token_usage` (cumulative across the whole session, keeps growing
/// across compacts) and `last_token_usage` (the current uncompressed window
/// AFTER the latest compact). The card grid MUST show
/// `last_token_usage / model_context_window`, never the cumulative, otherwise
/// long-lived sessions that have been compacted many times report percentages
/// far above 100% even though the live context is mostly empty.
fn read_last_token_info(path: &std::path::Path) -> Option<(u64, u64, u32)> {
    use std::io::{Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    if len == 0 {
        return None;
    }
    // Scan the file from the end in reverse-line chunks until we locate the
    // most recent `token_count` event. We must NOT just take the first
    // successfully-parsed line in the tail: if a more recent line happens to
    // fail to parse (truncated, non-UTF-8 byte, schema change) we would
    // silently fall back to a stale value. Files can be tens of MiB for
    // long-running sessions, so we walk back in bounded chunks instead of
    // slurping the whole file.
    const CHUNK: u64 = 1024 * 1024;
    let mut pos: u64 = len;
    let mut tail: Vec<u8> = Vec::new();
    loop {
        let take = pos.min(CHUNK);
        if take == 0 {
            break;
        }
        pos -= take;
        // Seek the original file each iteration. We use `seek` +
        // `read_to_end` rather than `Read::take`, because `take` consumes
        // the file handle and we need it on the next loop iteration.
        if f.seek(SeekFrom::Start(pos)).is_err() {
            break;
        }
        let mut buf = Vec::with_capacity(take as usize);
        use std::io::Read as _;
        if f.by_ref().take(take).read_to_end(&mut buf).is_err() {
            break;
        }
        // Concatenate this chunk in front of what we already buffered so the
        // resulting buffer stays in original file order.
        let mut combined = Vec::with_capacity(buf.len() + tail.len());
        combined.extend_from_slice(&buf);
        combined.extend_from_slice(&tail);
        tail = combined;
        // Skip a partial first line of this chunk: it continues into the
        // previous chunk, so we can't safely parse it on its own.
        let scan_start = if pos > 0 {
            match tail.iter().position(|b| *b == b'\n') {
                Some(idx) => idx + 1,
                None => continue,
            }
        } else {
            0
        };
        // Compute the slice of `tail` we will actually scan, dealing with
        // possibly non-UTF-8 bytes from a truncated/killed write.
        let scan_end: usize = match std::str::from_utf8(&tail[scan_start..]) {
            Ok(_) => tail.len(),
            Err(e) => scan_start + e.valid_up_to(),
        };
        if scan_end <= scan_start {
            // No usable UTF-8 in this tail at all; drop it and rewind.
            tail.truncate(scan_start);
            if pos == 0 {
                break;
            }
            continue;
        }
        // We may have just trimmed garbage off the end. Reflect that in
        // `tail` so the next iteration starts from clean bytes.
        tail.truncate(scan_end);
        let text: &str = match std::str::from_utf8(&tail[scan_start..]) {
            Ok(s) => s,
            Err(_) => {
                // Should be impossible after the trim above, but stay safe.
                if pos == 0 {
                    break;
                }
                continue;
            }
        };
        if let Some(info) = last_token_count_info_in(text) {
            return Some(info);
        }
        if pos == 0 {
            break;
        }
    }
    None
}

/// Scan a UTF-8 slice (in original file order) for the LAST `token_count`
/// event_msg and return `(last_token_usage.total_tokens,
/// total_token_usage.total_tokens, model_context_window)`.
fn last_token_count_info_in(text: &str) -> Option<(u64, u64, u32)> {
    // Collect (start, end_exclusive) for every complete line (everything
    // up to and including the trailing '\n'), then walk in reverse so the
    // first match is the most recent event in this slice. We must carry
    // the end offset, not just the start, because slicing from `start` to
    // the end of `text` would otherwise drag in every later line and
    // produce invalid JSON.
    let mut line_ranges: Vec<(usize, usize)> = Vec::with_capacity(256);
    let mut line_start = 0usize;
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            line_ranges.push((line_start, i + 1));
            line_start = i + 1;
        }
    }
    // Trailing line without a final newline: include it as a half-open
    // range up to the end of the slice.
    if line_start < text.len() {
        line_ranges.push((line_start, text.len()));
    }
    for (start, end) in line_ranges.iter().rev() {
        let line = &text[*start..*end];
        if line.is_empty() {
            continue;
        }
        if !line.contains("\"token_count\"") || !line.contains("\"event_msg\"") {
            continue;
        }
        if let Some(info) = parse_token_count_payload(line) {
            return Some(info);
        }
    }
    None
}

fn parse_token_count_payload(line: &str) -> Option<(u64, u64, u32)> {
    let v = serde_json::from_str::<serde_json::Value>(line).ok()?;
    let info = v.get("payload").and_then(|p| p.get("info"))?;
    let recent = info
        .get("last_token_usage")
        .and_then(|u| u.get("total_tokens"))
        .and_then(|t| t.as_u64())?;
    let accumulated = info
        .get("total_token_usage")
        .and_then(|u| u.get("total_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(recent);
    let cw = info
        .get("model_context_window")
        .and_then(|c| c.as_u64())? as u32;
    Some((recent, accumulated, cw))
}

#[derive(Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionInfo>,
    pub total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_token_count_event() {
        // Verbatim shape of the `token_count` event written by Codex 0.133+
        let line = r#"{"timestamp":"2026-05-24T06:10:43.419Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":241142674,"cached_input_tokens":235003776,"output_tokens":517683,"reasoning_output_tokens":85073,"total_tokens":241660357},"last_token_usage":{"input_tokens":35351,"cached_input_tokens":34688,"output_tokens":347,"reasoning_output_tokens":0,"total_tokens":35698},"model_context_window":258400},"rate_limits":{}}}"#;
        let dir = std::env::temp_dir();
        let path = dir.join("evocode-tauri-token-count-test.jsonl");
        std::fs::write(&path, format!("{}\n", line)).unwrap();
        let (recent, accumulated, cw) = read_last_token_info(&path).expect("should parse");
        assert_eq!(recent, 35_698, "last_token_usage.total_tokens");
        assert_eq!(accumulated, 241_660_357, "total_token_usage.total_tokens");
        assert_eq!(cw, 258_400, "model_context_window");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn returns_none_when_no_token_count() {
        let dir = std::env::temp_dir();
        let path = dir.join("evocode-tauri-token-count-empty.jsonl");
        std::fs::write(&path, b"{\"type\":\"session_meta\"}\n").unwrap();
        assert!(read_last_token_info(&path).is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn picks_latest_of_multiple_token_count_events() {
        let dir = std::env::temp_dir();
        let path = dir.join("evocode-tauri-token-count-multi.jsonl");
        let older = r#"{"timestamp":"2026-01-01T00:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":1000},"last_token_usage":{"total_tokens":500},"model_context_window":10000}}}"#;
        let newer = r#"{"timestamp":"2026-01-02T00:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":9000},"last_token_usage":{"total_tokens":2000},"model_context_window":12000}}}"#;
        // Pad with junk so the newer line is near the end of a >1 MiB read.
        let mut content = String::new();
        for _ in 0..30000 {
            content.push_str("{\"type\":\"response_item\",\"payload\":{\"x\":\"filler-padding-line\"}}\n");
        }
        content.push_str(older);
        content.push('\n');
        content.push_str(&"{\"type\":\"response_item\",\"payload\":{\"x\":\"more-padding\"}}\n".repeat(5000));
        content.push_str(newer);
        content.push('\n');
        std::fs::write(&path, content.as_bytes()).unwrap();
        let (recent, accumulated, cw) = read_last_token_info(&path).expect("should parse");
        assert_eq!(recent, 2_000);
        assert_eq!(accumulated, 9_000);
        assert_eq!(cw, 12_000);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn finds_latest_event_when_buried_past_first_chunk() {
        // Regression: the old implementation only read the last 1 MiB, so
        // any `token_count` event more than 1 MiB before EOF was invisible.
        // For long-running sessions the latest event can sit well past that
        // boundary (we measured 17+ MiB rollouts in the wild). Build a file
        // whose only `token_count` is several MiB before the end and confirm
        // we still locate it.
        let dir = std::env::temp_dir();
        let path = dir.join("evocode-tauri-token-count-buried.jsonl");
        let event = r#"{"timestamp":"2026-05-20T14:14:16Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":241660357},"last_token_usage":{"total_tokens":35698},"model_context_window":258400}}}"#;
        // ~2 MiB of padding after the event so the only `token_count` is
        // well outside the first 1 MiB tail window.
        let mut content = String::new();
        content.push_str(event);
        content.push('\n');
        let padding_line = "{\"type\":\"response_item\",\"payload\":{\"x\":\"filler-padding-line-that-is-quite-long-to-take-up-bytes\"}}\n";
        for _ in 0..20_000 {
            content.push_str(padding_line);
        }
        std::fs::write(&path, content.as_bytes()).unwrap();
        let (recent, accumulated, cw) = read_last_token_info(&path)
            .expect("should locate the buried event by walking back through the file");
        assert_eq!(recent, 35_698);
        assert_eq!(accumulated, 241_660_357);
        assert_eq!(cw, 258_400);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn picks_latest_when_more_recent_line_fails_to_parse() {
        // Regression: the old implementation returned the first line in the
        // tail it could parse, so a malformed (truncated/garbled) line
        // newer than a valid one would silently cause the function to
        // return the *older* event. The new implementation walks back
        // through the file when the tail slice has no valid event, so we
        // must surface the most recent valid one even if a more recent
        // line is unparseable.
        let dir = std::env::temp_dir();
        let path = dir.join("evocode-tauri-token-count-bad-tail.jsonl");
        let valid = r#"{"timestamp":"2026-01-02T00:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":9000},"last_token_usage":{"total_tokens":2000},"model_context_window":12000}}}"#;
        let mut content = String::new();
        content.push_str(valid);
        content.push('\n');
        // Append a deliberately truncated line that mentions token_count
        // but won't parse as JSON. The function should not get stuck on it.
        content.push_str("{\"timestamp\":\"2026-06-01T00:00:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"tok");
        std::fs::write(&path, content.as_bytes()).unwrap();
        let (recent, accumulated, cw) = read_last_token_info(&path)
            .expect("should find the valid event despite the truncated tail");
        assert_eq!(recent, 2_000);
        assert_eq!(accumulated, 9_000);
        assert_eq!(cw, 12_000);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rollout_parser_emits_structured_entries() {
        // Mini-rollout exercising every entry kind the UI cares about.
        let content = r#"
{"timestamp":"2026-06-12T17:16:57.682Z","type":"session_meta","payload":{"id":"x"}}
{"timestamp":"2026-06-12T17:16:57.683Z","type":"turn_context","payload":{"model":"MiniMax-M3"}}
{"timestamp":"2026-06-12T17:16:57.685Z","type":"event_msg","payload":{"type":"user_message","message":"hi"}}
{"timestamp":"2026-06-12T17:16:57.686Z","type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"system prompt"}]}}
{"timestamp":"2026-06-12T17:17:03.946Z","type":"event_msg","payload":{"type":"agent_reasoning","text":"thinking…"}}
{"timestamp":"2026-06-12T17:17:03.947Z","type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"dup"}]}}
{"timestamp":"2026-06-12T17:17:04.125Z","type":"response_item","payload":{"type":"function_call","name":"exec_command","arguments":"{\"cmd\":\"ls\"}","call_id":"c1"}}
{"timestamp":"2026-06-12T17:17:04.186Z","type":"response_item","payload":{"type":"function_call_output","call_id":"c1","output":"file.rs"}}
{"timestamp":"2026-06-12T17:28:09.033Z","type":"response_item","payload":{"type":"custom_tool_call","name":"apply_patch","input":"@@ -1 +1 @@\n-a\n+b","call_id":"c2"}}
{"timestamp":"2026-06-12T17:28:09.036Z","type":"response_item","payload":{"type":"custom_tool_call_output","call_id":"c2","output":"Success"}}
{"timestamp":"2026-06-12T17:28:09.200Z","type":"event_msg","payload":{"type":"patch_apply_end","call_id":"c2","success":true,"stdout":"Success","stderr":"","changes":{"src/foo.rs":{"type":"update","unified_diff":"@@ -1 +1 @@\n-a\n+b","move_path":null}}}}
{"timestamp":"2026-06-12T17:41:32.046Z","type":"event_msg","payload":{"type":"task_complete","last_agent_message":"bye","duration_ms":1474491}}
{"timestamp":"2026-06-12T17:48:49.770Z","type":"event_msg","payload":{"type":"thread_rolled_back","num_turns":1}}
{"timestamp":"2026-06-12T17:17:55.323Z","type":"event_msg","payload":{"type":"agent_message","message":"hello back"}}
"#;
        let entries = parse_rollout_entries(content);
        // Expected kinds in order: User, Reasoning, ToolCall (function),
        // ToolOutput, ToolCall (custom), ToolOutput, PatchEnd, TurnBoundary,
        // Assistant. session_meta, turn_context, the duplicate reasoning,
        // and thread_rolled_back must be dropped.
        let kinds: Vec<String> = entries
            .iter()
            .map(|e| match e {
                SessionEntry::User { .. } => "user".to_string(),
                SessionEntry::Assistant { .. } => "assistant".to_string(),
                SessionEntry::Reasoning { .. } => "reasoning".to_string(),
                SessionEntry::ToolCall { tool_kind, .. } => tool_kind.clone(),
                SessionEntry::ToolOutput { .. } => "tool_output".to_string(),
                SessionEntry::PatchEnd { .. } => "patch_end".to_string(),
                SessionEntry::TurnBoundary { .. } => "turn_boundary".to_string(),
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                "user".to_string(),
                "reasoning".to_string(),
                "function".to_string(),
                "tool_output".to_string(),
                "custom".to_string(),
                "tool_output".to_string(),
                "patch_end".to_string(),
                "turn_boundary".to_string(),
                "assistant".to_string(),
            ]
        );
        // Reasoning text comes from `event_msg:agent_reasoning` (the
        // `response_item:reasoning` line must be de-duplicated away).
        if let SessionEntry::Reasoning { text, .. } = &entries[1] {
            assert_eq!(text, "thinking…");
        } else {
            panic!("expected reasoning entry at index 1");
        }
        // PatchEnd carries a diff snippet.
        if let SessionEntry::PatchEnd { diffs, success, .. } = &entries[6] {
            assert!(*success);
            assert_eq!(diffs.len(), 1);
            assert_eq!(diffs[0].path, "src/foo.rs");
            assert!(diffs[0].diff.as_deref().unwrap_or("").contains("+b"));
        } else {
            panic!("expected patch_end entry at index 6");
        }
    }

    #[test]
    fn rollout_parser_truncates_huge_tool_outputs() {
        let huge = "x".repeat(TOOL_OUTPUT_MAX_CHARS + 100);
        let content = format!(
            r#"{{"timestamp":"t","type":"response_item","payload":{{"type":"function_call_output","call_id":"c","output":"{}"}}}}"#,
            huge
        );
        let entries = parse_rollout_entries(&content);
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            SessionEntry::ToolOutput { output, truncated, .. } => {
                assert!(*truncated, "truncated flag should be set for oversized output");
                assert!(output.chars().count() <= TOOL_OUTPUT_MAX_CHARS + 4);
            }
            _ => panic!("expected tool_output entry"),
        }
    }
}

/// Structured, frontend-friendly view of a single line from a rollout
/// `.jsonl` file. We parse the raw Codex event stream and emit one of
/// these variants per interesting entry. The frontend discriminates on
/// `kind` to render the appropriate chat / thinking / tool-call view.
///
/// We deliberately keep the noise out (`session_meta`, `turn_context`,
/// `task_started`, `token_count`, `thread_rolled_back`) and deduplicate
/// the doubled reasoning stream (Codex emits both `event_msg:agent_reasoning`
/// AND a `response_item:reasoning` for every thinking step; we keep the
/// event_msg form, which is the human-facing summary).
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionEntry {
    /// User prompt text (from `event_msg:user_message`).
    User {
        timestamp: String,
        text: String,
    },
    /// Assistant text reply (from `event_msg:agent_message`).
    Assistant {
        timestamp: String,
        text: String,
    },
    /// Model thinking / chain-of-thought summary.
    Reasoning {
        timestamp: String,
        text: String,
    },
    /// Tool invocation. `tool_kind` is `"function"` for the standard
    /// `function_call` / `function_call_output` pair and `"custom"` for
    /// `custom_tool_call` / `custom_tool_call_output` (e.g. `apply_patch`).
    ToolCall {
        timestamp: String,
        tool_kind: String,
        name: String,
        call_id: String,
        /// Raw arguments as a JSON-encoded string, kept verbatim so the
        /// frontend can pretty-print or syntax-highlight it.
        arguments: String,
    },
    /// Result of a tool call. The frontend pairs this with the matching
    /// `ToolCall` by `call_id`.
    ToolOutput {
        timestamp: String,
        call_id: String,
        /// Tool stdout / output. May be truncated server-side for huge
        /// blobs (the `truncated` flag tells the UI to show a "show more"
        /// affordance).
        output: String,
        truncated: bool,
    },
    /// Result of `apply_patch`. Carries the stdout/stderr lines and a
    /// short unified-diff snippet per changed file for a quick visual.
    PatchEnd {
        timestamp: String,
        call_id: Option<String>,
        success: bool,
        stdout: String,
        stderr: String,
        /// One short diff per touched file, in `path\n<diff>` form.
        diffs: Vec<PatchFileDiff>,
    },
    /// Boundary marker emitted at the end of every Codex turn.
    TurnBoundary {
        timestamp: String,
        last_message: String,
        duration_ms: u64,
    },
}

#[derive(Serialize)]
pub struct PatchFileDiff {
    pub path: String,
    /// Trimmed unified diff. `None` if the patch is a brand-new file or
    /// if the diff is missing.
    pub diff: Option<String>,
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
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with("state_")
                    && name.ends_with(".sqlite")
                {
                    match best {
                        None => best = Some(path),
                        Some(ref current) => {
                            let n_cur = current
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .trim_start_matches("state_")
                                .trim_end_matches(".sqlite")
                                .parse::<u32>()
                                .unwrap_or(0);
                            let n_new = name
                                .trim_start_matches("state_")
                                .trim_end_matches(".sqlite")
                                .parse::<u32>()
                                .unwrap_or(0);
                            if n_new > n_cur {
                                best = Some(path);
                            }
                        }
                    }
                }
            }
            best
        })
        .ok_or_else(|| "Cannot find state database".to_string())?;

    if !db_path.exists() {
        return Ok(SessionsResponse {
            sessions: Vec::new(),
            total: 0,
        });
    }

    // Read default context_window from config
    let mut default_cw: u32 = 256_000;
    if let Ok(content) = std::fs::read_to_string(codex_home.join("config.toml")) {
        for line in content.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("model_context_window = ")
                && let Ok(n) = val.trim().trim_matches('"').parse::<u32>()
            {
                default_cw = n;
                break;
            }
        }
    }

    fn model_cw(model: &str, default: u32) -> u32 {
        // Try to read the real context_window from models_catalog.json
        let codex_home = dirs::home_dir().map(|h| h.join(".codex"));
        if let Some(home) = codex_home {
            let catalog_path = home.join("models_catalog.json");
            if let Ok(json_str) = std::fs::read_to_string(&catalog_path)
                && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str)
                && let Some(models) = parsed.get("models").and_then(|m| m.as_array())
            {
                for m in models {
                    if let Some(slug) = m.get("slug").and_then(|s| s.as_str())
                        && slug == model
                        && let Some(cw) = m.get("context_window").and_then(|c| c.as_u64())
                    {
                        return cw as u32;
                    }
                }
            }
        }
        // Fallback to default from config.toml
        default
    }

    use sqlx::Row;
    use sqlx::sqlite::SqlitePoolOptions;

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
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    pool.close().await;

    let sessions: Vec<SessionInfo> = rows
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let name: String = row.get("title");
            let model: String = row.get("model");
            let tokens: u32 = row.get::<u32, _>("tokens_used");
            let rollout_path: String = row.get("rollout_path");
            // Prefer the most recent `token_count` event from the rollout
            // file. It reports the actual context window in use for this
            // thread (which can differ from the global config if the user
            // switched models mid-session) and the `last_token_usage` for the
            // current uncompressed window. When the rollout has no
            // `token_count` event yet (very fresh session, or pre-0.133
            // Codex) we still use the catalog/config context window, but we
            // report `recent = 0` instead of substituting the legacy
            // `tokens_used` (session-wide cumulative) counter for it. Using
            // the cumulative as the current window would saturate the card
            // at 100% on every multi-compact session.
            let (recent, accumulated, cw) = match read_last_token_info(
                std::path::Path::new(&rollout_path),
            ) {
                Some((recent, accumulated, cw)) => (recent, accumulated, cw),
                None => {
                    let cw = model_cw(&model, default_cw);
                    (0u64, tokens as u64, cw)
                }
            };
            let total_cells = cw.div_ceil(10000).max(1);
            let used_cells = recent
                .div_ceil(10000)
                .min(total_cells as u64) as u32;
            SessionInfo {
                id,
                name,
                model,
                total: total_cells,
                used: used_cells,
                // The cell fields are quantized to 10K tokens for the grid
                // visualization; carry the exact token totals alongside so
                // the card footer can show the real number (e.g. 106,234
                // rather than 110,000).
                total_tokens: Some(cw as u64),
                used_tokens: Some(recent),
                accumulated: Some(accumulated),
                rollout_path,
            }
        })
        .collect();

    Ok(SessionsResponse { sessions, total })
}

#[tauri::command]
async fn get_session_content(id: String) -> Result<Vec<SessionEntry>, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let codex_home = home.join(".codex");

    // Find the latest state_*.sqlite file
    let db_path = std::fs::read_dir(&codex_home)
        .ok()
        .and_then(|entries| {
            let mut best: Option<std::path::PathBuf> = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with("state_")
                    && name.ends_with(".sqlite")
                {
                    match best {
                        None => best = Some(path),
                        Some(ref current) => {
                            let n_cur = current
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .trim_start_matches("state_")
                                .trim_end_matches(".sqlite")
                                .parse::<u32>()
                                .unwrap_or(0);
                            let n_new = name
                                .trim_start_matches("state_")
                                .trim_end_matches(".sqlite")
                                .parse::<u32>()
                                .unwrap_or(0);
                            if n_new > n_cur {
                                best = Some(path);
                            }
                        }
                    }
                }
            }
            best
        })
        .ok_or_else(|| "Cannot find state database".to_string())?;

    if !db_path.exists() {
        return Ok(Vec::new());
    }

    use sqlx::Row;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=ro", db_path.display()))
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let row = sqlx::query("SELECT rollout_path FROM threads WHERE id = ?")
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

    let content = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;

    let mut entries = parse_rollout_entries(&content);

    // If the rollout is huge, keep only the tail so the IPC payload stays
    // bounded. We pick a generous cap because the frontend renders each
    // entry as a compact card; the cap is mainly to protect against the
    // multi-MiB sessions observed in the wild.
    const MAX_ENTRIES: usize = 500;
    if entries.len() > MAX_ENTRIES {
        let drop = entries.len() - MAX_ENTRIES;
        entries.drain(..drop);
    }
    Ok(entries)
}

/// Maximum length (in characters) of a single tool output / patch stdout
/// blob we keep verbatim. Anything longer is truncated with a `truncated`
/// flag so the frontend can offer a "show more" affordance.
const TOOL_OUTPUT_MAX_CHARS: usize = 4_000;

/// Truncate a string at a char boundary, appending a small marker.
fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    if s.chars().count() <= max {
        return (s.to_string(), false);
    }
    let mut out = String::with_capacity(max + 32);
    for (i, c) in s.char_indices() {
        if i >= max {
            break;
        }
        out.push(c);
    }
    out.push_str("…");
    (out, true)
}

/// Walk the rollout JSONL and convert it into a flat list of
/// `SessionEntry` items. Uninteresting lines (session_meta, turn_context,
/// token_count, task_started, thread_rolled_back, non-user
/// `response_item:message`) are dropped. Reasoning is emitted exactly
/// once even though Codex records it in two parallel forms.
fn parse_rollout_entries(content: &str) -> Vec<SessionEntry> {
    let mut out = Vec::new();
    for line in content.lines() {
        let v = match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let timestamp = v
            .get("timestamp")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match event_type {
            "event_msg" => {
                let payload = v.get("payload");
                let sub = payload
                    .and_then(|p| p.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                match sub {
                    "user_message" => {
                        let text = payload
                            .and_then(|p| p.get("message"))
                            .and_then(|m| m.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !text.is_empty() {
                            out.push(SessionEntry::User { timestamp, text });
                        }
                    }
                    "agent_message" => {
                        let text = payload
                            .and_then(|p| p.get("message"))
                            .and_then(|m| m.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !text.is_empty() {
                            out.push(SessionEntry::Assistant { timestamp, text });
                        }
                    }
                    "agent_reasoning" => {
                        // Preferred form: human-facing reasoning summary.
                        // Codex also writes a `response_item:reasoning`
                        // with the same content; we skip that one below.
                        let text = payload
                            .and_then(|p| p.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !text.is_empty() {
                            out.push(SessionEntry::Reasoning { timestamp, text });
                        }
                    }
                    "patch_apply_end" => {
                        let call_id = payload
                            .and_then(|p| p.get("call_id"))
                            .and_then(|t| t.as_str())
                            .map(String::from);
                        let success = payload
                            .and_then(|p| p.get("success"))
                            .and_then(|t| t.as_bool())
                            .unwrap_or(true);
                        let stdout = payload
                            .and_then(|p| p.get("stdout"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let stderr = payload
                            .and_then(|p| p.get("stderr"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let diffs = collect_patch_diffs(payload.and_then(|p| p.get("changes")));
                        out.push(SessionEntry::PatchEnd {
                            timestamp,
                            call_id,
                            success,
                            stdout,
                            stderr,
                            diffs,
                        });
                    }
                    "task_complete" => {
                        let last_message = payload
                            .and_then(|p| p.get("last_agent_message"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let duration_ms = payload
                            .and_then(|p| p.get("duration_ms"))
                            .and_then(|t| t.as_u64())
                            .unwrap_or(0);
                        out.push(SessionEntry::TurnBoundary {
                            timestamp,
                            last_message,
                            duration_ms,
                        });
                    }
                    // The rest (`task_started`, `token_count`,
                    // `thread_rolled_back`) is intentionally dropped.
                    _ => {}
                }
            }
            "response_item" => {
                let payload = v.get("payload");
                let sub = payload
                    .and_then(|p| p.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                match sub {
                    "function_call" => {
                        let name = payload
                            .and_then(|p| p.get("name"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let arguments = payload
                            .and_then(|p| p.get("arguments"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let call_id = payload
                            .and_then(|p| p.get("call_id"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        out.push(SessionEntry::ToolCall {
                            timestamp,
                            tool_kind: "function".into(),
                            name,
                            call_id,
                            arguments,
                        });
                    }
                    "custom_tool_call" => {
                        let name = payload
                            .and_then(|p| p.get("name"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let call_id = payload
                            .and_then(|p| p.get("call_id"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        // `input` can be a string or a structured object.
                        let arguments = match payload.and_then(|p| p.get("input")) {
                            Some(s) if s.is_string() => s
                                .as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_default(),
                            Some(v) => v.to_string(),
                            None => String::new(),
                        };
                        out.push(SessionEntry::ToolCall {
                            timestamp,
                            tool_kind: "custom".into(),
                            name,
                            call_id,
                            arguments,
                        });
                    }
                    "function_call_output" => {
                        let call_id = payload
                            .and_then(|p| p.get("call_id"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let raw = payload
                            .and_then(|p| p.get("output"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let (output, truncated) = truncate_chars(&raw, TOOL_OUTPUT_MAX_CHARS);
                        out.push(SessionEntry::ToolOutput {
                            timestamp,
                            call_id,
                            output,
                            truncated,
                        });
                    }
                    "custom_tool_call_output" => {
                        let call_id = payload
                            .and_then(|p| p.get("call_id"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let raw = payload
                            .and_then(|p| p.get("output"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let (output, truncated) = truncate_chars(&raw, TOOL_OUTPUT_MAX_CHARS);
                        out.push(SessionEntry::ToolOutput {
                            timestamp,
                            call_id,
                            output,
                            truncated,
                        });
                    }
                    // `response_item:reasoning` is intentionally skipped:
                    // the human-facing `event_msg:agent_reasoning` is
                    // already emitted above and the two carry the same
                    // text. `response_item:message` is also skipped
                    // (the developer / system messages are not part of
                    // the visible transcript).
                    _ => {}
                }
            }
            // `session_meta`, `turn_context`, anything else: skip.
            _ => {}
        }
    }
    out
}

/// Extract a short unified-diff snippet per touched file from
/// `event_msg:patch_apply_end.changes`. We cap the per-file diff size so
/// a 5 MB patch doesn't blow up the IPC payload.
fn collect_patch_diffs(changes: Option<&serde_json::Value>) -> Vec<PatchFileDiff> {
    const DIFF_MAX_CHARS: usize = 2_000;
    let Some(obj) = changes.and_then(|c| c.as_object()) else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(obj.len());
    for (path, change) in obj {
        let diff = change
            .get("unified_diff")
            .and_then(|d| d.as_str())
            .map(|d| {
                let (trimmed, _) = truncate_chars(d, DIFF_MAX_CHARS);
                if trimmed.is_empty() { None } else { Some(trimmed) }
            })
            .unwrap_or(None);
        out.push(PatchFileDiff {
            path: path.clone(),
            diff,
        });
    }
    out
}

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "evocode-tray";
const MENU_SHOW: &str = "tray_show";
const MENU_START: &str = "tray_start";
const MENU_STOP: &str = "tray_stop";
const MENU_QUIT: &str = "tray_quit";

fn load_tray_icon() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("valid tray icon")
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Window", true, None::<&str>)?;
    let start = MenuItem::with_id(app, MENU_START, "Start Bridge", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, MENU_STOP, "Stop Bridge", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    Menu::with_items(app, &[&show, &sep, &start, &stop, &sep, &quit])
}

fn handle_tray_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        MENU_SHOW => show_main_window(app),
        MENU_QUIT => {
            app.exit(0);
        }
        MENU_START | MENU_STOP => {
            show_main_window(app);
            let action = if event.id().as_ref() == MENU_START {
                "start"
            } else {
                "stop"
            };
            let _ = app.emit("tray-bridge-action", action);
        }
        _ => {}
    }
}

fn handle_tray_icon_event(app: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_main_window(app);
    }
}

fn handle_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event
        && window.label() == MAIN_WINDOW_LABEL
    {
        api.prevent_close();
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // A second instance was launched: focus the existing main window.
            show_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .manage(BridgeState::new(17761))
        .invoke_handler(tauri::generate_handler![
            start_bridge,
            stop_bridge,
            bridge_is_running,
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
            test_provider_connectivity,
            open_config_dir,
        ])
        .on_window_event(handle_window_event)
        .setup(|app| {
            let handle = app.handle().clone();
            let icon = load_tray_icon();
            let menu = build_tray_menu(&handle)?;
            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .tooltip("evocode")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(handle_tray_menu_event)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    handle_tray_icon_event(app, event);
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

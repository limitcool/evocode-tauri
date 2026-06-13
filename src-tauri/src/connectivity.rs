use serde::Serialize;

#[derive(Serialize)]
pub struct ConnectivityResult {
    pub ok: bool,
    pub status: u16,
    pub latency_ms: u128,
    pub message: String,
}

pub async fn test_provider(
    base_url: &str,
    api_key: &str,
    wire_api: &str,
    api_key_header: Option<&str>,
    model: Option<&str>,
) -> ConnectivityResult {
    let model_val = model.unwrap_or_default();

    let header_name = api_key_header
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(if wire_api == "anthropic" {
            "X-Api-Key"
        } else {
            "Authorization"
        });

    let header_value = if header_name.eq_ignore_ascii_case("Authorization") {
        format!("Bearer {}", api_key)
    } else {
        api_key.to_string()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let base = base_url.trim_end_matches('/');

    let (url, body) = match wire_api {
        "anthropic" => (
            format!("{}/v1/messages", base),
            serde_json::json!({
                "model": model_val,
                "max_tokens": 100,
                "thinking": { "type": "disabled" },
                "messages": [{"role": "user", "content": [{"type": "text", "text": "Reply with 'ok' immediately."}]}]
            }),
        ),
        "chat_completions" => (
            format!("{}/chat/completions", base),
            serde_json::json!({
                "model": model_val,
                "messages": [{"role": "user", "content": "Reply with 'ok' immediately."}],
                "max_tokens": 10
            }),
        ),
        _ => (
            format!("{}/responses", base),
            serde_json::json!({
                "model": model_val,
                "input": "Reply with 'ok' immediately.",
                "max_output_tokens": 1
            }),
        ),
    };

    let started = std::time::Instant::now();
    let resp = client
        .post(&url)
        .header(header_name, &header_value)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;
    let latency_ms = started.elapsed().as_millis();

    match resp {
        Ok(r) => {
            let status = r.status().as_u16();
            if (200..300).contains(&status) {
                let text = r.text().await.unwrap_or_default();

                fn trunc(s: &str, max: usize) -> String {
                    s.chars().take(max).collect()
                }

                let reply = match wire_api {
                    "anthropic" => serde_json::from_str::<serde_json::Value>(&text)
                        .ok()
                        .and_then(|v| {
                            v["content"]
                                .as_array()?
                                .first()?
                                .get("text")?
                                .as_str()
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_else(|| trunc(&text, 200)),
                    "chat_completions" => serde_json::from_str::<serde_json::Value>(&text)
                        .ok()
                        .and_then(|v| {
                            v["choices"]
                                .as_array()?
                                .first()?["message"]["content"]
                                .as_str()
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_else(|| trunc(&text, 200)),
                    _ => serde_json::from_str::<serde_json::Value>(&text)
                        .ok()
                        .and_then(|v| {
                            v["output"]
                                .as_array()?
                                .first()?["content"]
                                .as_array()?
                                .first()?
                                .get("text")?
                                .as_str()
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_else(|| trunc(&text, 200)),
                };

                ConnectivityResult {
                    ok: true,
                    status,
                    latency_ms,
                    message: format!("OK: {}", reply),
                }
            } else {
                let err_body = r.text().await.unwrap_or_default();
                ConnectivityResult {
                    ok: false,
                    status,
                    latency_ms,
                    message: format!("Error (HTTP {}): {}", status, err_body),
                }
            }
        }
        Err(e) => ConnectivityResult {
            ok: false,
            status: 0,
            latency_ms,
            message: format!(
                "{}: {}",
                if e.is_timeout() {
                    "timeout"
                } else if e.is_connect() {
                    "connection refused"
                } else {
                    "request failed"
                },
                e
            ),
        },
    }
}

use crate::model::{DiscoveryResult, ProviderConfig, ProviderKind};
use crate::{ProofError, Result};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub fn resolve_api_key(cfg: &ProviderConfig) -> Option<String> {
    let trimmed = cfg.api_key.trim();
    if !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }
    let env_name = cfg.kind.env_var_name();
    if env_name.is_empty() {
        return None;
    }
    std::env::var(env_name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn key_status_badge(cfg: &ProviderConfig) -> &'static str {
    if matches!(cfg.kind, ProviderKind::Ollama | ProviderKind::LmStudio) {
        return "local";
    }
    if resolve_api_key(cfg).is_some() {
        "SET"
    } else {
        "unset"
    }
}

pub fn is_embed_only_model(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("embed")
        || lower.contains("bge-")
        || lower.contains("nomic-embed")
        || lower.contains("text-embedding")
        || lower.contains("rerank")
}

pub fn discover_models(cfg: &ProviderConfig) -> DiscoveryResult {
    let start = Instant::now();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(3))
        .timeout_read(Duration::from_secs(8))
        .build();

    let base = cfg.base_url.trim_end_matches('/');
    let res: std::result::Result<Vec<String>, String> = match cfg.kind {
        ProviderKind::Ollama => {
            let url = format!("{base}/api/tags");
            match agent.get(&url).call() {
                Ok(resp) => match resp.into_json::<Value>() {
                    Ok(v) => {
                        let models = v
                            .get("models")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|m| m.get("name").and_then(Value::as_str))
                                    .filter(|n| !is_embed_only_model(n))
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        Ok(models)
                    }
                    Err(e) => Err(format!("invalid Ollama JSON: {e}")),
                },
                Err(e) => Err(format!("Ollama unreachable at {url}: {e}")),
            }
        }
        ProviderKind::LmStudio | ProviderKind::OpenAiCompat => {
            let url = format!("{base}/models");
            let mut req = agent.get(&url);
            if let Some(key) = resolve_api_key(cfg) {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }
            match req.call() {
                Ok(resp) => match resp.into_json::<Value>() {
                    Ok(v) => {
                        let models = v
                            .get("data")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|m| m.get("id").and_then(Value::as_str))
                                    .filter(|n| !is_embed_only_model(n))
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        Ok(models)
                    }
                    Err(e) => Err(format!("invalid /models JSON: {e}")),
                },
                Err(e) => Err(format!("endpoint unreachable at {url}: {e}")),
            }
        }
        ProviderKind::Gemini => {
            let Some(key) = resolve_api_key(cfg) else {
                return DiscoveryResult {
                    ok: false,
                    latency_ms: start.elapsed().as_millis() as u64,
                    models: Vec::new(),
                    message: "GEMINI_API_KEY not set".to_string(),
                };
            };
            let url = format!("{base}/models?key={key}");
            match agent.get(&url).call() {
                Ok(resp) => match resp.into_json::<Value>() {
                    Ok(v) => {
                        let models = v
                            .get("models")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|m| m.get("name").and_then(Value::as_str))
                                    .map(|s| s.trim_start_matches("models/").to_string())
                                    .filter(|n| !is_embed_only_model(n))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        Ok(models)
                    }
                    Err(e) => Err(format!("invalid Gemini JSON: {e}")),
                },
                Err(e) => Err(format!("Gemini API error: {e}")),
            }
        }
        ProviderKind::Anthropic => {
            let Some(key) = resolve_api_key(cfg) else {
                return DiscoveryResult {
                    ok: false,
                    latency_ms: start.elapsed().as_millis() as u64,
                    models: Vec::new(),
                    message: "ANTHROPIC_API_KEY not set".to_string(),
                };
            };
            let url = format!("{base}/models");
            match agent
                .get(&url)
                .set("x-api-key", &key)
                .set("anthropic-version", "2023-06-01")
                .call()
            {
                Ok(resp) => match resp.into_json::<Value>() {
                    Ok(v) => {
                        let models = v
                            .get("data")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|m| m.get("id").and_then(Value::as_str))
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        Ok(models)
                    }
                    Err(e) => Err(format!("invalid Anthropic JSON: {e}")),
                },
                Err(e) => Err(format!("Anthropic API error: {e}")),
            }
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;
    match res {
        Ok(models) => {
            let count = models.len();
            DiscoveryResult {
                ok: true,
                latency_ms,
                models,
                message: format!("Connected ({latency_ms} ms, {count} models discovered)"),
            }
        }
        Err(msg) => DiscoveryResult {
            ok: false,
            latency_ms,
            models: Vec::new(),
            message: msg,
        },
    }
}

pub fn complete(
    cfg: &ProviderConfig,
    model_override: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let model = if !model_override.trim().is_empty() {
        model_override.trim()
    } else if !cfg.default_model.trim().is_empty() {
        cfg.default_model.trim()
    } else {
        return Err(ProofError::Provider(
            "no model selected; run live discovery in Settings first".to_string(),
        ));
    };

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(120))
        .build();
    let base = cfg.base_url.trim_end_matches('/');

    match cfg.kind {
        ProviderKind::Ollama => {
            let url = format!("{base}/api/chat");
            let body = json!({
                "model": model,
                "stream": false,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": user_prompt }
                ]
            });
            let resp = agent
                .post(&url)
                .send_json(body)
                .map_err(|e| ProofError::Provider(format!("Ollama request failed: {e}")))?;
            let v: Value = resp
                .into_json()
                .map_err(|e| ProofError::Provider(format!("Ollama JSON parse failed: {e}")))?;
            v.get("message")
                .and_then(|m| m.get("content"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .ok_or_else(|| {
                    ProofError::Provider("missing message.content in Ollama response".to_string())
                })
        }
        ProviderKind::LmStudio | ProviderKind::OpenAiCompat => {
            let url = format!("{base}/chat/completions");
            let mut req = agent.post(&url);
            if let Some(key) = resolve_api_key(cfg) {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }
            let body = json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": user_prompt }
                ]
            });
            let resp = req
                .send_json(body)
                .map_err(|e| ProofError::Provider(format!("OpenAI-compat request failed: {e}")))?;
            let v: Value = resp.into_json().map_err(|e| {
                ProofError::Provider(format!("OpenAI-compat JSON parse failed: {e}"))
            })?;
            v.get("choices")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .ok_or_else(|| {
                    ProofError::Provider("missing choices[0].message.content".to_string())
                })
        }
        ProviderKind::Gemini => {
            let key = resolve_api_key(cfg)
                .ok_or_else(|| ProofError::Provider("GEMINI_API_KEY not configured".to_string()))?;
            let url = format!("{base}/models/{model}:generateContent?key={key}");
            let body = json!({
                "systemInstruction": { "parts": [{ "text": system_prompt }] },
                "contents": [{ "parts": [{ "text": user_prompt }] }]
            });
            let resp = agent
                .post(&url)
                .send_json(body)
                .map_err(|e| ProofError::Provider(format!("Gemini request failed: {e}")))?;
            let v: Value = resp
                .into_json()
                .map_err(|e| ProofError::Provider(format!("Gemini JSON parse failed: {e}")))?;
            v.get("candidates")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(Value::as_array)
                .and_then(|p| p.first())
                .and_then(|p| p.get("text"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .ok_or_else(|| {
                    ProofError::Provider("missing candidate text in Gemini response".to_string())
                })
        }
        ProviderKind::Anthropic => {
            let key = resolve_api_key(cfg).ok_or_else(|| {
                ProofError::Provider("ANTHROPIC_API_KEY not configured".to_string())
            })?;
            let url = format!("{base}/messages");
            let body = json!({
                "model": model,
                "max_tokens": 4096,
                "system": system_prompt,
                "messages": [{ "role": "user", "content": user_prompt }]
            });
            let resp = agent
                .post(&url)
                .set("x-api-key", &key)
                .set("anthropic-version", "2023-06-01")
                .send_json(body)
                .map_err(|e| ProofError::Provider(format!("Anthropic request failed: {e}")))?;
            let v: Value = resp
                .into_json()
                .map_err(|e| ProofError::Provider(format!("Anthropic JSON parse failed: {e}")))?;
            v.get("content")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|c| c.get("text"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .ok_or_else(|| {
                    ProofError::Provider(
                        "missing content[0].text in Anthropic response".to_string(),
                    )
                })
        }
    }
}

use serde_json::{json, Value};
use std::io::{BufRead, Write};

pub const PROTOCOL_VERSION: &str = "2026-07-28";
pub const MAX_LINE_BYTES: usize = 4 * 1024 * 1024;

pub trait ToolHost {
    fn server_name(&self) -> &str;
    fn server_version(&self) -> &str;
    fn tools(&self) -> Vec<Value>;
    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String>;
    fn resources(&self) -> Vec<Value> {
        Vec::new()
    }
    fn read_resource(&self, uri: &str) -> Result<Value, String> {
        Err(format!("unknown resource {uri}"))
    }
}

#[must_use]
pub fn normalize_tool_name(name: &str) -> String {
    name.replace('.', "_")
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn ok_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

pub fn handle_line<H: ToolHost>(host: &mut H, line: &str) -> Option<Value> {
    let trimmed = line.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() > MAX_LINE_BYTES {
        return Some(error_response(Value::Null, -32600, "request too large"));
    }
    let req: Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return Some(error_response(Value::Null, -32700, "parse error")),
    };
    if !req.is_object() {
        return Some(error_response(
            Value::Null,
            -32600,
            "request must be a JSON object",
        ));
    }
    let id = match req.get("id") {
        Some(v) if !v.is_null() => v.clone(),
        _ => return None,
    };
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    Some(match method {
        "initialize" => ok_response(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "listChanged": false }
                },
                "serverInfo": {
                    "name": host.server_name(),
                    "version": host.server_version()
                }
            }),
        ),
        "ping" => ok_response(id, json!({})),
        "tools/list" => ok_response(id, json!({ "tools": host.tools() })),
        "tools/call" => {
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return Some(error_response(id, -32602, "missing tool name"));
            };
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let norm = normalize_tool_name(name);
            match host.call(&norm, &args) {
                Ok(val) => ok_response(
                    id,
                    json!({
                        "content": [{ "type": "text", "text": val.to_string() }],
                        "isError": false
                    }),
                ),
                Err(msg) => ok_response(
                    id,
                    json!({
                        "content": [{ "type": "text", "text": msg }],
                        "isError": true
                    }),
                ),
            }
        }
        "resources/list" => ok_response(id, json!({ "resources": host.resources() })),
        "resources/read" => {
            let Some(uri) = params.get("uri").and_then(Value::as_str) else {
                return Some(error_response(id, -32602, "missing resource uri"));
            };
            match host.read_resource(uri) {
                Ok(content) => ok_response(id, json!({ "contents": [content] })),
                Err(msg) => error_response(id, -32602, &msg),
            }
        }
        other => error_response(id, -32601, &format!("method not found: {other}")),
    })
}

pub fn run_mcp_stdio<H: ToolHost>(host: &mut H) -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        if let Some(resp) = handle_line(host, &line) {
            let encoded = serde_json::to_string(&resp).map_err(std::io::Error::other)?;
            writeln!(out, "{encoded}")?;
            out.flush()?;
        }
    }
    Ok(())
}

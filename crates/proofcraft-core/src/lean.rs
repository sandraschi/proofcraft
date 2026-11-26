use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanCheckResult {
    pub available: bool,
    pub passed: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn check_lean_snippet(code: &str) -> LeanCheckResult {
    let Ok(mut child) = Command::new("lean")
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    else {
        return LeanCheckResult {
            available: false,
            passed: false,
            stdout: String::new(),
            stderr: "lean executable not found on PATH (optional formalizer not installed)"
                .to_string(),
        };
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(code.as_bytes());
    }

    match child.wait_with_output() {
        Ok(out) => LeanCheckResult {
            available: true,
            passed: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => LeanCheckResult {
            available: true,
            passed: false,
            stdout: String::new(),
            stderr: format!("lean wait failed: {e}"),
        },
    }
}

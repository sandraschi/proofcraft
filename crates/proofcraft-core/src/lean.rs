use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanStatus {
    pub available: bool,
    pub version: String,
    pub executable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanCheckResult {
    pub available: bool,
    pub passed: bool,
    pub sorry_detected: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn resolve_lean_exe() -> Option<PathBuf> {
    if let Ok(out) = Command::new("lean").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("lean"));
        }
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let candidate = PathBuf::from(profile)
            .join(".elan")
            .join("bin")
            .join("lean.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let candidate = PathBuf::from(home).join(".elan").join("bin").join("lean");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn resolve_lake_exe() -> Option<PathBuf> {
    if let Ok(out) = Command::new("lake").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("lake"));
        }
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let candidate = PathBuf::from(profile)
            .join(".elan")
            .join("bin")
            .join("lake.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let candidate = PathBuf::from(home).join(".elan").join("bin").join("lake");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn lean_status() -> LeanStatus {
    let Some(exe) = resolve_lean_exe() else {
        return LeanStatus {
            available: false,
            version: "not installed".to_string(),
            executable: String::new(),
        };
    };
    let version = Command::new(&exe)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    LeanStatus {
        available: !version.is_empty() && version != "unknown",
        version,
        executable: exe.display().to_string(),
    }
}

pub fn contains_sorry_or_admit(code: &str) -> bool {
    for line in code.lines() {
        let trimmed = line.split("--").next().unwrap_or("");
        for tok in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if tok == "sorry" || tok == "admit" {
                return true;
            }
        }
    }
    false
}

pub fn extract_lean_block(raw: &str) -> String {
    if let Some(start_idx) = raw.find("```lean") {
        let after = &raw[start_idx + 7..];
        let after_nl = after.strip_prefix('\r').unwrap_or(after);
        let after_nl = after_nl.strip_prefix('\n').unwrap_or(after_nl);
        if let Some(end_idx) = after_nl.find("```") {
            return after_nl[..end_idx].trim().to_string();
        }
    }
    if let Some(start_idx) = raw.find("```") {
        let after = &raw[start_idx + 3..];
        let after_nl = after.strip_prefix('\r').unwrap_or(after);
        let after_nl = after_nl.strip_prefix('\n').unwrap_or(after_nl);
        if let Some(end_idx) = after_nl.find("```") {
            return after_nl[..end_idx].trim().to_string();
        }
    }
    raw.trim().to_string()
}

pub fn check_lean_snippet(code: &str) -> LeanCheckResult {
    check_lean_in_project(code, None)
}

pub fn check_lean_in_project(code: &str, project_dir: Option<&Path>) -> LeanCheckResult {
    let sorry_detected = contains_sorry_or_admit(code);

    let use_lake = project_dir
        .filter(|p| p.join("lakefile.lean").is_file() || p.join("lakefile.toml").is_file())
        .and_then(|dir| resolve_lake_exe().map(|lake| (lake, dir)));

    let mut child = if let Some((lake_exe, dir)) = use_lake {
        match Command::new(&lake_exe)
            .args(["env", "lean", "--stdin"])
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                return LeanCheckResult {
                    available: false,
                    passed: false,
                    sorry_detected,
                    stdout: String::new(),
                    stderr: "failed to spawn lake env lean --stdin".to_string(),
                };
            }
        }
    } else {
        let Some(exe) = resolve_lean_exe() else {
            return LeanCheckResult {
                available: false,
                passed: false,
                sorry_detected,
                stdout: String::new(),
                stderr: "lean executable not found on PATH or ~/.elan/bin".to_string(),
            };
        };
        match Command::new(&exe)
            .arg("--stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                return LeanCheckResult {
                    available: false,
                    passed: false,
                    sorry_detected,
                    stdout: String::new(),
                    stderr: "failed to spawn lean --stdin".to_string(),
                };
            }
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(code.as_bytes());
    }

    match child.wait_with_output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let mut stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let clean_exit = out.status.success();
            let passed = clean_exit && !sorry_detected;
            if sorry_detected && clean_exit {
                if !stderr.is_empty() {
                    stderr.push('\n');
                }
                stderr.push_str("rejected: proof contains `sorry` or `admit`");
            }
            LeanCheckResult {
                available: true,
                passed,
                sorry_detected,
                stdout,
                stderr,
            }
        }
        Err(e) => LeanCheckResult {
            available: true,
            passed: false,
            sorry_detected,
            stdout: String::new(),
            stderr: format!("lean wait failed: {e}"),
        },
    }
}

mod mcp;

use clap::{Parser, Subcommand};
use mcp::{run_mcp_stdio, ToolHost};
use proofcraft_core::help::BUNDLED_SKILL_MD;
use proofcraft_core::model::{BudgetConfig, ProviderConfig, RoleRouting};
use proofcraft_core::store::ProofStore;
use proofcraft_core::{dispatch, Command};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "proofcraft-mcp",
    version,
    about = "proofcraft CLI and MCP stdio server"
)]
struct Cli {
    #[arg(long, global = true, help = "Path to SQLite database")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    cmd: SubCmd,
}

#[derive(Subcommand)]
enum SubCmd {
    /// Start pure-Rust MCP stdio server
    Mcp,
    /// Show server and problem summary
    Status,
    /// List all proof problems
    ListProblems,
    /// Get full problem detail including attempts, ledger, and advisor notes
    GetProblem {
        #[arg(long)]
        problem_id: String,
    },
    /// Create a new proof problem
    CreateProblem {
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "Theoretical Computer Science")]
        domain: String,
        #[arg(long)]
        statement: String,
        #[arg(long, default_value_t = 200)]
        max_calls: u32,
    },
    /// Add a proof direction to a problem
    AddDirection {
        #[arg(long)]
        problem_id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        claim: String,
    },
    /// List Verified Ledger entries and Dead Ends for a problem
    Ledger {
        #[arg(long)]
        problem_id: String,
    },
    /// Export consolidated manuscript (md or tex)
    Export {
        #[arg(long)]
        problem_id: String,
        #[arg(long, default_value = "md")]
        format: String,
    },
    /// Execute one bounded Cogentic prove-verify-ledger round
    RunRound {
        #[arg(long)]
        problem_id: String,
        #[arg(long, default_value_t = 2)]
        provers: u32,
    },
    /// Clear all demo/stored data
    ClearDemo,
    /// Reload embedded demo seed problems
    LoadDemo,
}

struct ProofHost {
    store: ProofStore,
    provider: ProviderConfig,
    routing: RoleRouting,
    budget: BudgetConfig,
}

impl ToolHost for ProofHost {
    fn server_name(&self) -> &str {
        "proofcraft-mcp"
    }

    fn server_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn tools(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "proof_status",
                "description": "Return proofcraft status, problem count, total calls used, and budget cap.",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            json!({
                "name": "proof_list_problems",
                "description": "List all proof discovery problems in the workspace.",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            json!({
                "name": "proof_get_problem",
                "description": "Get full detail for a problem: directions, round attempts, verifier objections, Verified Ledger, and Process Advisor notes.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "problem_id": { "type": "string" } },
                    "required": ["problem_id"]
                }
            }),
            json!({
                "name": "proof_create_problem",
                "description": "Create a new open research problem for Cogentic-style proof discovery.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "domain": { "type": "string" },
                        "statement": { "type": "string" },
                        "max_calls": { "type": "integer" }
                    },
                    "required": ["title", "statement"]
                }
            }),
            json!({
                "name": "proof_ledger",
                "description": "List Verified Lemmas and Excluded Dead Ends for a problem.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "problem_id": { "type": "string" } },
                    "required": ["problem_id"]
                }
            }),
            json!({
                "name": "proof_export_manuscript",
                "description": "Export consolidated proof manuscript as Markdown (md) or LaTeX (tex).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "problem_id": { "type": "string" },
                        "format": { "type": "string", "enum": ["md", "tex"] }
                    },
                    "required": ["problem_id"]
                }
            }),
            json!({
                "name": "proof_run_round",
                "description": "Execute one bounded prove-verify-ledger round using the configured LLM provider.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "problem_id": { "type": "string" },
                        "provers": { "type": "integer" }
                    },
                    "required": ["problem_id"]
                }
            }),
        ]
    }

    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        let cmd = match name {
            "proof_status" => Command::Status,
            "proof_list_problems" => Command::ListProblems,
            "proof_get_problem" => Command::GetProblem {
                problem_id: args
                    .get("problem_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing problem_id".to_string())?
                    .to_string(),
            },
            "proof_create_problem" => Command::CreateProblem {
                title: args
                    .get("title")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing title".to_string())?
                    .to_string(),
                domain: args
                    .get("domain")
                    .and_then(Value::as_str)
                    .unwrap_or("Mathematics")
                    .to_string(),
                statement: args
                    .get("statement")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing statement".to_string())?
                    .to_string(),
                max_calls: args.get("max_calls").and_then(Value::as_u64).unwrap_or(200) as u32,
            },
            "proof_ledger" => Command::ListLedger {
                problem_id: args
                    .get("problem_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing problem_id".to_string())?
                    .to_string(),
            },
            "proof_export_manuscript" => Command::ExportManuscript {
                problem_id: args
                    .get("problem_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing problem_id".to_string())?
                    .to_string(),
                format: args
                    .get("format")
                    .and_then(Value::as_str)
                    .unwrap_or("md")
                    .to_string(),
            },
            "proof_run_round" => Command::RunRound {
                problem_id: args
                    .get("problem_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing problem_id".to_string())?
                    .to_string(),
                provers: args.get("provers").and_then(Value::as_u64).unwrap_or(2) as u32,
            },
            other => return Err(format!("unknown tool: {other}")),
        };
        dispatch(
            &self.store,
            &self.provider,
            &self.routing,
            &self.budget,
            cmd,
        )
        .map_err(|e| e.to_string())
    }

    fn resources(&self) -> Vec<Value> {
        vec![json!({
            "uri": "skill://proofcraft/SKILL.md",
            "name": "proofcraft skill",
            "mimeType": "text/markdown"
        })]
    }

    fn read_resource(&self, uri: &str) -> Result<Value, String> {
        if uri == "skill://proofcraft/SKILL.md" {
            Ok(json!({
                "uri": uri,
                "mimeType": "text/markdown",
                "text": BUNDLED_SKILL_MD
            }))
        } else {
            Err(format!("unknown resource: {uri}"))
        }
    }
}

fn default_db_path() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local)
            .join("proofcraft")
            .join("proofcraft.sqlite")
    } else {
        PathBuf::from("proofcraft.sqlite")
    }
}

fn main() {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let store = match ProofStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("proofcraft-mcp error opening db: {e}");
            std::process::exit(1);
        }
    };
    let mut host = ProofHost {
        store,
        provider: ProviderConfig::default(),
        routing: RoleRouting::default(),
        budget: BudgetConfig::default(),
    };

    if matches!(cli.cmd, SubCmd::Mcp) {
        if let Err(e) = run_mcp_stdio(&mut host) {
            eprintln!("mcp stdio error: {e}");
            std::process::exit(1);
        }
        return;
    }

    let cmd = match cli.cmd {
        SubCmd::Mcp => unreachable_cmd(),
        SubCmd::Status => Command::Status,
        SubCmd::ListProblems => Command::ListProblems,
        SubCmd::GetProblem { problem_id } => Command::GetProblem { problem_id },
        SubCmd::CreateProblem {
            title,
            domain,
            statement,
            max_calls,
        } => Command::CreateProblem {
            title,
            domain,
            statement,
            max_calls,
        },
        SubCmd::AddDirection {
            problem_id,
            name,
            claim,
        } => Command::AddDirection {
            problem_id,
            name,
            claim,
        },
        SubCmd::Ledger { problem_id } => Command::ListLedger { problem_id },
        SubCmd::Export { problem_id, format } => Command::ExportManuscript { problem_id, format },
        SubCmd::RunRound {
            problem_id,
            provers,
        } => Command::RunRound {
            problem_id,
            provers,
        },
        SubCmd::ClearDemo => Command::ClearDemoData,
        SubCmd::LoadDemo => Command::LoadDemoData,
    };

    match dispatch(
        &host.store,
        &host.provider,
        &host.routing,
        &host.budget,
        cmd,
    ) {
        Ok(val) => {
            if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                println!("{pretty}");
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn unreachable_cmd() -> Command {
    Command::Status
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use mcp::handle_line;

    fn test_host() -> ProofHost {
        let store = ProofStore::open_in_memory().expect("db");
        store.load_embedded_seed().expect("seed");
        ProofHost {
            store,
            provider: ProviderConfig::default(),
            routing: RoleRouting::default(),
            budget: BudgetConfig::default(),
        }
    }

    #[test]
    fn mcp_initialize_and_tools_list() {
        let mut h = test_host();
        let init = handle_line(
            &mut h,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        )
        .expect("resp");
        assert_eq!(init["result"]["protocolVersion"], mcp::PROTOCOL_VERSION);

        let list = handle_line(
            &mut h,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        )
        .expect("list");
        let tools = list["result"]["tools"].as_array().expect("array");
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn mcp_tools_call_status_and_ledger() {
        let mut h = test_host();
        let res = handle_line(
            &mut h,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"proof.status","arguments":{}}}"#,
        )
        .expect("call");
        assert_eq!(res["result"]["isError"], false);

        let led = handle_line(
            &mut h,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"proof_ledger","arguments":{"problem_id":"prob-anytime-experts"}}}"#,
        )
        .expect("ledger");
        assert_eq!(led["result"]["isError"], false);
    }

    #[test]
    fn mcp_resources_list_and_read() {
        let mut h = test_host();
        let r = handle_line(
            &mut h,
            r#"{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"skill://proofcraft/SKILL.md"}}"#,
        )
        .expect("read");
        assert!(r["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("proofcraft"));
    }
}

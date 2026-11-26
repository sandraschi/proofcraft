pub mod harness;
pub mod help;
pub mod lean;
pub mod literature;
pub mod llm;
pub mod model;
pub mod store;

#[cfg(test)]
mod tests;

use model::{
    AdvisorNote, Attempt, BudgetConfig, Direction, LedgerEntry, LedgerKind, Problem,
    ProviderConfig, RoleRouting,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use store::ProofStore;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProofError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("budget cap reached: {0}")]
    BudgetExceeded(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, ProofError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
pub enum Command {
    Status,
    ListProblems,
    GetProblem {
        problem_id: String,
    },
    CreateProblem {
        title: String,
        domain: String,
        statement: String,
        max_calls: u32,
    },
    AddDirection {
        problem_id: String,
        name: String,
        claim: String,
    },
    ListLedger {
        problem_id: String,
    },
    ExportManuscript {
        problem_id: String,
        format: String,
    },
    RunRound {
        problem_id: String,
        provers: u32,
    },
    AddLiterature {
        problem_id: String,
        arxiv_id: String,
    },
    CheckLean {
        code: String,
    },
    LeanStatus,
    FormalizeLemma {
        problem_id: String,
        ledger_id: String,
    },
    ClearDemoData,
    LoadDemoData,
}

pub fn dispatch(
    store: &ProofStore,
    provider: &ProviderConfig,
    routing: &RoleRouting,
    budget: &BudgetConfig,
    cmd: Command,
) -> Result<Value> {
    match cmd {
        Command::Status => {
            let problems = store.list_problems()?;
            let total_calls: u32 = problems.iter().map(|p| p.calls_used).sum();
            Ok(json!({
                "app": "proofcraft",
                "version": env!("CARGO_PKG_VERSION"),
                "problem_count": problems.len(),
                "total_calls_used": total_calls,
                "provider_kind": provider.kind.as_str(),
                "budget_max_calls": budget.max_calls_per_run,
            }))
        }
        Command::ListProblems => {
            let problems = store.list_problems()?;
            Ok(serde_json::to_value(problems)?)
        }
        Command::GetProblem { problem_id } => {
            let detail = store.get_problem_detail(&problem_id)?;
            Ok(serde_json::to_value(detail)?)
        }
        Command::CreateProblem {
            title,
            domain,
            statement,
            max_calls,
        } => {
            if title.trim().is_empty() || statement.trim().is_empty() {
                return Err(ProofError::InvalidInput(
                    "title and statement must not be empty".to_string(),
                ));
            }
            let p = store.create_problem(&title, &domain, &statement, max_calls)?;
            Ok(serde_json::to_value(p)?)
        }
        Command::AddDirection {
            problem_id,
            name,
            claim,
        } => {
            let d = store.add_direction(&problem_id, &name, &claim)?;
            Ok(serde_json::to_value(d)?)
        }
        Command::ListLedger { problem_id } => {
            let entries = store.list_ledger(&problem_id)?;
            Ok(serde_json::to_value(entries)?)
        }
        Command::ExportManuscript { problem_id, format } => {
            let text = store.export_manuscript(&problem_id, &format)?;
            Ok(json!({
                "problem_id": problem_id,
                "format": format,
                "manuscript": text
            }))
        }
        Command::RunRound {
            problem_id,
            provers,
        } => {
            let report = harness::execute_round(
                store,
                provider,
                routing,
                budget,
                &problem_id,
                provers.max(1),
            )?;
            Ok(serde_json::to_value(report)?)
        }
        Command::AddLiterature {
            problem_id,
            arxiv_id,
        } => {
            let lit = literature::fetch_arxiv_metadata(&arxiv_id)?;
            store.add_literature(&problem_id, &lit)?;
            Ok(serde_json::to_value(lit)?)
        }
        Command::CheckLean { code } => {
            let res = lean::check_lean_snippet(&code);
            Ok(serde_json::to_value(res)?)
        }
        Command::LeanStatus => {
            let st = lean::lean_status();
            Ok(serde_json::to_value(st)?)
        }
        Command::FormalizeLemma {
            problem_id,
            ledger_id,
        } => {
            let entries = store.list_ledger(&problem_id)?;
            let entry = entries
                .into_iter()
                .find(|e| e.id == ledger_id)
                .ok_or_else(|| ProofError::NotFound(format!("ledger entry {ledger_id}")))?;
            let sys = "You are a Lean 4 formalizer. Output ONLY a self-contained Lean 4 code block (no Mathlib imports, pure Lean 4 core) proving the lemma or an equivalent finite/concrete instance. Never use `sorry` or `admit`.";
            let prompt = format!(
                "Title: {}\nStatement: {}\nProof Sketch: {}",
                entry.title, entry.statement, entry.proof_sketch
            );
            let raw = llm::complete(provider, &routing.prover_model, sys, &prompt)?;
            let lean_code = lean::extract_lean_block(&raw);
            let check = lean::check_lean_snippet(&lean_code);
            store.update_ledger_lean(&problem_id, &ledger_id, &lean_code, check.passed)?;
            Ok(json!({
                "ledger_id": ledger_id,
                "lean_code": lean_code,
                "check": check
            }))
        }
        Command::ClearDemoData => {
            store.clear_all()?;
            Ok(json!({ "cleared": true }))
        }
        Command::LoadDemoData => {
            let loaded = store.load_embedded_seed()?;
            Ok(json!({ "loaded_problems": loaded }))
        }
    }
}

pub fn problem_summary_line(p: &Problem) -> String {
    format!(
        "[{}] {} ({} calls / {} max, {} rounds)",
        p.status.as_str(),
        p.title,
        p.calls_used,
        p.max_calls,
        p.rounds_completed
    )
}

pub fn ledger_badge(entry: &LedgerEntry) -> &'static str {
    match entry.kind {
        LedgerKind::VerifiedLemma => "LEMMA",
        LedgerKind::DeadEnd => "DEAD END",
    }
}

pub fn format_advisor_line(note: &AdvisorNote) -> String {
    format!(
        "R{}: {} -> {}",
        note.round, note.observation, note.instruction_delta
    )
}

pub fn attempt_passed_both(attempt: &Attempt) -> bool {
    attempt.single_verdict == model::Verdict::Pass && attempt.comp_verdict == model::Verdict::Pass
}

pub fn direction_label(d: &Direction) -> String {
    format!("{} [{}]", d.name, d.status.as_str())
}

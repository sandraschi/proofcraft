use crate::literature::LiteratureRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProblemStatus {
    Queued,
    InProgress,
    Verified,
    BudgetExhausted,
}

impl ProblemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::InProgress => "InProgress",
            Self::Verified => "Verified",
            Self::BudgetExhausted => "BudgetExhausted",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "InProgress" => Self::InProgress,
            "Verified" => Self::Verified,
            "BudgetExhausted" => Self::BudgetExhausted,
            _ => Self::Queued,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectionStatus {
    Active,
    Proved,
    Excluded,
}

impl DirectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Proved => "Proved",
            Self::Excluded => "Excluded",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "Proved" => Self::Proved,
            "Excluded" => Self::Excluded,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Pending,
    Pass,
    Fail,
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Pass => "Pass",
            Self::Fail => "Fail",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "pass" | "passed" | "accept" | "accepted" => Self::Pass,
            "fail" | "failed" | "reject" | "rejected" => Self::Fail,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerKind {
    VerifiedLemma,
    DeadEnd,
}

impl LedgerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VerifiedLemma => "VerifiedLemma",
            Self::DeadEnd => "DeadEnd",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "DeadEnd" => Self::DeadEnd,
            _ => Self::VerifiedLemma,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub title: String,
    pub domain: String,
    pub statement: String,
    pub status: ProblemStatus,
    pub max_calls: u32,
    pub calls_used: u32,
    pub rounds_completed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Direction {
    pub id: String,
    pub name: String,
    pub claim: String,
    pub status: DirectionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub round: u32,
    pub prover: String,
    pub direction_id: String,
    pub method: String,
    pub summary: String,
    pub draft_markdown: String,
    pub single_verdict: Verdict,
    pub single_objection: String,
    pub comp_verdict: Verdict,
    pub comp_objection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub kind: LedgerKind,
    pub title: String,
    pub statement: String,
    pub proof_sketch: String,
    pub isolated_verified: bool,
    pub source_attempt: String,
    #[serde(default)]
    pub lean_code: String,
    #[serde(default)]
    pub lean_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorNote {
    pub round: u32,
    pub observation: String,
    pub instruction_delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetail {
    pub problem: Problem,
    pub directions: Vec<Direction>,
    pub attempts: Vec<Attempt>,
    pub ledger: Vec<LedgerEntry>,
    pub advisor_notes: Vec<AdvisorNote>,
    #[serde(default)]
    pub literature: Vec<LiteratureRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedProblem {
    pub id: String,
    pub title: String,
    pub domain: String,
    pub statement: String,
    pub status: ProblemStatus,
    pub max_calls: u32,
    pub calls_used: u32,
    pub rounds_completed: u32,
    pub directions: Vec<Direction>,
    pub attempts: Vec<Attempt>,
    pub ledger: Vec<LedgerEntry>,
    pub advisor_notes: Vec<AdvisorNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedBundle {
    pub problems: Vec<SeedProblem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Ollama,
    LmStudio,
    OpenAiCompat,
    Gemini,
    Anthropic,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
            Self::OpenAiCompat => "OpenAI-Compat",
            Self::Gemini => "Gemini",
            Self::Anthropic => "Anthropic",
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Ollama => "http://127.0.0.1:11434",
            Self::LmStudio => "http://127.0.0.1:1234/v1",
            Self::OpenAiCompat => "https://api.openai.com/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::Anthropic => "https://api.anthropic.com/v1",
        }
    }

    pub fn env_var_name(&self) -> &'static str {
        match self {
            Self::Ollama | Self::LmStudio => "",
            Self::OpenAiCompat => "OPENAI_API_KEY",
            Self::Gemini => "GEMINI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    pub base_url: String,
    pub api_key: String,
    pub default_model: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: ProviderKind::Ollama,
            base_url: ProviderKind::Ollama.default_base_url().to_string(),
            api_key: String::new(),
            default_model: String::new(), // No blind model defaults; populated via live discovery
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleRouting {
    pub prover_model: String,
    pub verifier_model: String,
    pub summarizer_model: String,
    pub advisor_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub max_calls_per_run: u32,
    pub max_rounds_per_run: u32,
    pub max_usd_per_run: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_calls_per_run: 200,
            max_rounds_per_run: 12,
            max_usd_per_run: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub models: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundReport {
    pub problem_id: String,
    pub round: u32,
    pub calls_spent: u32,
    pub attempts_added: usize,
    pub lemmas_promoted: usize,
    pub verified_solution_found: bool,
    pub advisor_delta: String,
}

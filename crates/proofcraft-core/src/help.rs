#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    GetStarted,
    Functions,
    Usage,
    Tips,
    Pitfalls,
    Planned,
    OtherCrafts,
}

impl HelpTab {
    pub const ALL: [Self; 7] = [
        Self::GetStarted,
        Self::Functions,
        Self::Usage,
        Self::Tips,
        Self::Pitfalls,
        Self::Planned,
        Self::OtherCrafts,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::GetStarted => "Get Started",
            Self::Functions => "Functions",
            Self::Usage => "Usage (GUI / CLI / MCP)",
            Self::Tips => "Tips & Tricks",
            Self::Pitfalls => "Pitfalls & Safety",
            Self::Planned => "Planned Extensions",
            Self::OtherCrafts => "Other Crafts",
        }
    }

    pub fn body(&self) -> &'static str {
        match self {
            Self::GetStarted => concat!(
                "Welcome to proofcraft -- a pure-Rust multi-agent automated proof discovery studio ",
                "modeled on Cogentic (arXiv:2609.40324).\n\n",
                "1. On first launch, proofcraft loads two realistic demo problems into SQLite ",
                "(Anytime Regret with n Experts, and Online Inverse Linear Optimization).\n",
                "2. Open Settings (Ctrl+,) -> LLM & Providers to run Live Model Discovery against ",
                "local Ollama / LM Studio on your RTX 4090 or configure Gemini / Anthropic / OpenAI-Compat keys.\n",
                "3. Assign fast local models to Summarizer & Process Advisor, and your strongest ",
                "reasoning model to Prover & Adversarial Verifiers.\n",
                "4. Click 'Run Proof Round' (or use proofcraft-cli / proofcraft-mcp) to execute a ",
                "bounded prove-verify-ledger round."
            ),
            Self::Functions => concat!(
                "Core Cogentic Pipeline:\n",
                "- Orchestrator: allocates provers across competing directions without mathematical bias.\n",
                "- Independent Summarizers: build separate briefings per prover from Record + Ledger.\n",
                "- Parallel Provers: draft candidate natural-language proofs.\n",
                "- Individual Adversarial Verifier: assumes every step and citation is false until proven.\n",
                "- Comparative Verifier: reads all drafts in a round side-by-side to spot shared blind spots.\n",
                "- Ledger Auditor: extracts partial lemmas from failed drafts and re-verifies them in isolation.\n",
                "- Process Advisor: updates standing prover/verifier rigor rules across rounds.\n",
                "- Manuscript Exporter: compiles accepted proofs + verified lemmas to Markdown or LaTeX."
            ),
            Self::Usage => concat!(
                "GUI:\n",
                "- Switch views via the topbar tabs (toggle with Ctrl+B) or the Window menu.\n",
                "- Zoom any window with Ctrl + Mouse Wheel.\n",
                "- Switch skins (Dark, Light, Ivory Proof) in View -> Interface Theme or Settings (Ctrl+,).\n\n",
                "CLI (proofcraft-mcp):\n",
                "- proofcraft-mcp status --json\n",
                "- proofcraft-mcp list-problems --json\n",
                "- proofcraft-mcp run-round --problem-id prob-anytime-experts --provers 2\n",
                "- proofcraft-mcp export --problem-id prob-anytime-experts --format tex\n\n",
                "MCP Stdio Server:\n",
                "- Run `proofcraft-mcp mcp` (pure JSON-RPC on stdout, diagnostics on stderr)."
            ),
            Self::Tips => concat!(
                "- Hybrid Role Routing: Run Qwen/DeepSeek on your local RTX 4090 (Ollama/LM Studio) ",
                "for Summarizers and Process Advisor, saving ~40-60% of API calls.\n",
                "- Dead Ends matter: Record failed counterexamples in the Verified Ledger so future ",
                "rounds never waste budget revisiting refuted bounds.\n",
                "- Start with 2 provers and a 30-call budget cap on new problems to calibrate difficulty."
            ),
            Self::Pitfalls => concat!(
                "- Budget Explosions: proofcraft enforces a hard call cap and round cap before every ",
                "round starts. Never disable caps on untrusted loops.\n",
                "- Sycophantic Verification: Both verifiers use adversarial system prompts, but weak ",
                "verifier models can still miss subtle gaps. Always inspect the Manuscript Export yourself.\n",
                "- API Keys: Keys are read from environment variables (GEMINI_API_KEY, ANTHROPIC_API_KEY, ",
                "OPENAI_API_KEY) or Settings; never commit .env files."
            ),
            Self::Planned => concat!(
                "- Optional Lean 4 (`lake env lean`) subprocess verification for formalizable sub-lemmas.\n",
                "- Live `arxiv-mcp` Literature Reviewer hook for mid-run citation & theorem retrieval.\n",
                "- Multi-threaded concurrent API dispatch for large prover pools."
            ),
            Self::OtherCrafts => concat!(
                "- rsicraft: Evolutionary code/program search & local LoRA self-improvement (Frontis-MA1).\n",
                "- logiccraft: Formal logic & truth-table/SAT tooling.\n",
                "- writercraft: Long-form technical & academic authoring.\n",
                "- devcraft: Fleet orchestration, claims, and build telemetry."
            ),
        }
    }
}

pub const BUNDLED_SKILL_MD: &str = r#"---
name: proofcraft
description: Orchestrate Cogentic-style multi-agent mathematical proof discovery via proofcraft-mcp.
---
# proofcraft Skill

Use `proofcraft-mcp` tools to manage problems, inspect the Verified Ledger, run bounded prove-verify rounds, and export LaTeX/Markdown manuscripts.
"#;

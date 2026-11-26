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
            Self::GetStarted => "Get Started & Paper",
            Self::Functions => "Functions & Lean 4",
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
                "THE ORIGINAL PAPER: COGENTIC (arXiv:2609.40324)\n",
                "Title: Cogentic: Multi-Agent Orchestration for Automated Proof Discovery\n",
                "Authors: Yang Cai, Vineet Gupta, Yanchen Jiang, Christopher Liaw, Aranyak Mehta, Grigoris Velegkas, Di Wang (Google Research / Google DeepMind / Yale)\n",
                "URL: https://arxiv.org/abs/2609.40324  |  Results: https://sites.google.com/view/cogentic\n\n",
                "Why this paper matters:\n",
                "Frontier models can often spot a clever mathematical idea in one shot, but open research problems require exploring competing conjectures, surviving subtle technical obstructions, and remembering intermediate progress over long horizons. Cogentic solves this with a lean, disk-backed prove-verify-ledger loop that used only O(100) model calls for most problems and O(1000) calls for the hardest -- small enough to orchestrate on a single lab PC.\n\n",
                "Results proved autonomously by Cogentic (verified by domain experts):\n",
                "1. Online Inverse Linear Optimization: First efficient and proper O(d) regret bound uniform in T, at O(d^2) per round (replacing log det with a self-normalized tr(H_t^-1/2) potential).\n",
                "2. Two-Sided Bulow-Klemperer Competition Complexity: Tight +2 sellers on the smaller side suffice (down from >= 20,000 on both sides).\n",
                "3. Anytime Regret with n Experts: Matches the fixed-horizon leading constant sqrt(t ln n / 2) simultaneously for all t >= 1.\n",
                "4. Simple vs. Optimal Revenue (Single Additive Buyer): Improved approximation factor from 5.2 down to 3.52.\n",
                "5. Price of Anarchy for Autobidding Auctions: Tight 1.5 PoA for 2 bidders (pFPA_1) and 2 - 1/(4n+1) for n bidders (pFPA_2n).\n\n",
                "QUICK START IN PROOFCRAFT\n",
                "1. On first launch, proofcraft seeds two demo problems into SQLite (Anytime Regret with n Experts, and Online Inverse Linear Optimization).\n",
                "2. Open Settings (Ctrl+,) -> LLM & Providers, pick Ollama/LM Studio (local RTX 4090) or Gemini/Anthropic/OpenAI-Compat, and click 'Test Connection & Discover Models'.\n",
                "3. Verify Lean 4 is detected in the bottom status bar (e.g. Lean 4.36.0 via ~/.elan/bin).\n",
                "4. Click 'Run Proof Round' in the left sidebar or inspect the Verified Ledger."
            ),
            Self::Functions => concat!(
                "COGENTIC MULTI-AGENT PIPELINE\n",
                "- Orchestrator: Tracks global state and allocates prover slots across competing directions. Barred from holding mathematical opinions.\n",
                "- Literature Reviewers: Fetch real arXiv titles and abstracts into SQLite and inject them into prover briefings so citations stay grounded.\n",
                "- Independent Summarizers: Write separate, context-bounded briefings per prover from the Record and Verified Ledger so parallel provers do not suffer groupthink.\n",
                "- Parallel Provers: Draft candidate proofs concurrently (via std::thread::scope for APIs, queued for single-GPU local models).\n",
                "- Two-Tier Adversarial Verifiers: Every draft must pass BOTH an Individual Verifier (assuming every step and citation is false until justified) AND a Cross-Draft Comparative Verifier (reading all drafts side-by-side to catch shared blind spots).\n",
                "- Verified Ledger & Isolated Auditor: Extracts partial lemmas from failed drafts, re-verifies each in isolation, and logs refuted bounds as Excluded Dead Ends.\n",
                "- Process Advisor: Reads verification logs across rounds to tighten prover/verifier rigor instructions without steering the math.\n",
                "- Consolidation Stage: Polishes accepted proofs and exports Markdown (.md) or LaTeX (.tex).\n\n",
                "LEAN 4 FORMAL VERIFICATION (CORE ENGINE)\n",
                "Natural-language proofs can still hide subtle gaps. proofcraft integrates Lean 4 directly as a mechanical truth anchor:\n",
                "- Toolchain Resolution: Automatically detects `lean` and `lake` on PATH or in `%USERPROFILE%\\.elan\\bin`.\n",
                "- Anti-Cheating `sorry` / `admit` Guard: Lean 4 exits 0 with a warning when a proof contains `sorry` or `admit`. proofcraft scans non-comment tokens and rejects any proof containing `sorry` or `admit`.\n",
                "- Lake / Mathlib Workspaces: Set a `Lake Project Dir` (containing `lakefile.lean` or `lakefile.toml`) in Settings to run checks via `lake env lean --stdin` with full Mathlib imports.\n",
                "- One-Click Lemma Formalization: On any Verified Lemma card, click 'Formalize & Verify in Lean 4' to translate the lemma into Lean 4 and persist the verified proof.\n\n",
                "EXAMPLE LEAN 4 CODE (RUNNABLE IN THE LEDGER SCRATCHPAD)\n\n",
                "-- Example 1: Arithmetic identity (kernel reduction)\n",
                "theorem sum_two_squares : (3 : Nat)^2 + 4^2 = 5^2 := rfl\n\n",
                "-- Example 2: Universal quantifier + omega/simp in pure Lean 4\n",
                "theorem add_comm_twice (a b : Nat) : a + b + a = 2 * a + b := by\n",
                "  omega\n\n",
                "-- Example 3: Inductive proof (sum of first n naturals)\n",
                "def sumTo : Nat -> Nat\n",
                "  | 0 => 0\n",
                "  | n + 1 => sumTo n + (n + 1)\n\n",
                "theorem sumTo_formula (n : Nat) : 2 * sumTo n = n * (n + 1) := by\n",
                "  induction n with\n",
                "  | zero => rfl\n",
                "  | succ k ih =>\n",
                "    simp [sumTo, Nat.mul_add, Nat.add_mul, ih]\n",
                "    omega"
            ),
            Self::Usage => concat!(
                "GUI WORKFLOW\n",
                "- Tabs: Workspace, Rounds & Verifiers, Verified Ledger, Process Advisor, Manuscript, Help Center, Settings.\n",
                "- Shortcuts: Ctrl+Wheel (zoom), Ctrl+B (show/hide tab bar), Ctrl+K (command palette), Ctrl+, (settings).\n",
                "- Skins: Obsidian Dark, Crisp Light, and Ivory Proof (warm paper theme for long reading).\n",
                "- Copilot Dock: Click 'Copilot' in the top-right bar for interactive math Q&A.\n\n",
                "CLI (proofcraft-mcp)\n",
                "- proofcraft-mcp status\n",
                "- proofcraft-mcp list-problems\n",
                "- proofcraft-mcp add-literature --problem-id <id> --arxiv-id 2609.40324\n",
                "- proofcraft-mcp run-round --problem-id <id> --provers 2\n",
                "- proofcraft-mcp lean-status\n",
                "- proofcraft-mcp check-lean --code \"theorem t : 1 + 1 = 2 := rfl\"\n",
                "- proofcraft-mcp formalize-lemma --problem-id <id> --ledger-id <lem>\n",
                "- proofcraft-mcp consolidate --problem-id <id> --format tex\n\n",
                "MCP STDIO SERVER (2026-07-28, 12 TOOLS)\n",
                "- Launch via `proofcraft-mcp mcp` or `start.bat -Mcp`.\n",
                "- Tools: proof_status, proof_list_problems, proof_get_problem, proof_create_problem, proof_ledger, proof_export_manuscript, proof_run_round, proof_add_literature, proof_check_lean, proof_lean_status, proof_formalize_lemma, proof_consolidate."
            ),
            Self::Tips => concat!(
                "- Hybrid 4090 + API Routing: Run a local 30B model in Ollama/LM Studio for Summarizers and Process Advisor, and reserve frontier API calls for Provers and Adversarial Verifiers.\n",
                "- Always Attach arXiv References: Use 'Fetch & Attach' in Workspace so provers see real abstracts instead of guessing theorem statements.\n",
                "- Enable Auto-Formalize in Settings: Turn on 'Auto-formalize promoted lemmas in Lean 4' when working on discrete math, combinatorics, or elementary inequalities.\n",
                "- Record Dead Ends: Refuted bounds in the Verified Ledger save huge call budgets in later rounds."
            ),
            Self::Pitfalls => concat!(
                "- Hard Budget Caps (What is a 'call'?): One 'call' = one single LLM inference request (one prompt sent to Ollama/LM Studio/API and one completion returned). Each round with N provers spends 3*N + 2 calls (1 Summarizer + 1 Prover + 1 Individual Verifier per prover, plus 1 Comparative Verifier + 1 Process Advisor, plus 1 per salvaged lemma audit and +1 if Lean 4 auto-formalize is enabled). Max Calls stops new rounds before crossing the cap.\n",
                "- Lean 4 `sorry` Trap: Lean exits 0 with a warning if `sorry` is present. proofcraft blocks `sorry` and `admit` automatically, so a `LEAN 4 VERIFIED` badge means zero holes.\n",
                "- Mathlib Imports: Standalone `lean --stdin` only includes Lean 4 core. To use `import Mathlib`, point `Lake Project Dir` in Settings at a built Lake workspace.\n",
                "- API Keys: Stored via environment variables (`GEMINI_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) or masked in Settings; never commit `.env`."
            ),
            Self::Planned => concat!(
                "- `arxiv-mcp` / Semantic Scholar full-text PDF ingestion for mid-run Literature Reviewer dispatches.\n",
                "- Multi-file Lake project generation (exporting an entire verified manuscript as a compilable `.lean` project).\n",
                "- Round-by-round token & USD cost telemetry charts in the Workspace view."
            ),
            Self::OtherCrafts => concat!(
                "- rsicraft: Evolutionary code/program search & local LoRA self-improvement (Frontis-MA1) -- uses numeric execution fitness, whereas proofcraft uses adversarial proof verification + Lean 4.\n",
                "- logiccraft: Formal logic & truth-table/SAT tooling.\n",
                "- writercraft: Long-form technical & academic authoring.\n",
                "- devcraft: Fleet orchestration, claims, and build telemetry."
            ),
        }
    }
}

pub const LEAN_EXAMPLE_INDUCTION: &str = concat!(
    "-- 1. Telescoping Potential Descent (Cogentic Theorem 1 trace-potential backbone)\n",
    "def sumStep (f : Nat -> Int) : Nat -> Int\n",
    "  | 0 => 0\n",
    "  | n + 1 => sumStep f n + (f n - f (n + 1))\n\n",
    "theorem telescoping_potential (f : Nat -> Int) (T : Nat) :\n",
    "    sumStep f T = f 0 - f T := by\n",
    "  induction T with\n",
    "  | zero => simp [sumStep]\n",
    "  | succ k ih =>\n",
    "    simp [sumStep, ih]\n",
    "    omega\n\n",
    "theorem potential_nonneg_step_bound (f : Nat -> Int) (T : Nat)\n",
    "    (h_nonneg : f T >= 0) :\n",
    "    sumStep f T <= f 0 := by\n",
    "  rw [telescoping_potential]\n",
    "  omega\n\n",
    "-- 2. Square Non-negativity & AM-GM Inequality in Pure Lean 4 Core\n",
    "theorem int_sq_nonneg (x : Int) : 0 <= x * x := by\n",
    "  rcases Int.le_total 0 x with h | h\n",
    "  · exact Int.mul_nonneg h h\n",
    "  · have hn : 0 <= -x := by omega\n",
    "    have eq : x * x = (-x) * (-x) := by rw [Int.neg_mul_neg]\n",
    "    rw [eq]\n",
    "    exact Int.mul_nonneg hn hn\n\n",
    "theorem am_gm_int (a b : Int) : 2 * (a * b) <= a * a + b * b := by\n",
    "  have hsq : 0 <= (a - b) * (a - b) := int_sq_nonneg (a - b)\n",
    "  have hexp : (a - b) * (a - b) = a * a - a * b - b * a + b * b := by\n",
    "    simp only [Int.sub_mul, Int.mul_sub]\n",
    "    omega\n",
    "  have hcomm : b * a = a * b := Int.mul_comm b a\n",
    "  omega"
);

pub const BUNDLED_SKILL_MD: &str = r#"---
name: proofcraft
description: Orchestrate Cogentic-style multi-agent mathematical proof discovery and Lean 4 formalization via proofcraft-mcp.
---
# proofcraft Skill

Use `proofcraft-mcp` tools to manage problems, attach arXiv literature, inspect the Verified Ledger, run bounded prove-verify rounds, verify lemmas in Lean 4, and export LaTeX/Markdown manuscripts.
"#;

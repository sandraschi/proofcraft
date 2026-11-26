#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    GetStarted,
    CogenticAndCalls,
    Lean4Core,
    LakeAndMathlib,
    Usage,
    TipsAndPitfalls,
    OtherCrafts,
}

impl HelpTab {
    pub const ALL: [Self; 7] = [
        Self::GetStarted,
        Self::CogenticAndCalls,
        Self::Lean4Core,
        Self::LakeAndMathlib,
        Self::Usage,
        Self::TipsAndPitfalls,
        Self::OtherCrafts,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::GetStarted => "1. Get Started & Math",
            Self::CogenticAndCalls => "2. Cogentic & Calls",
            Self::Lean4Core => "3. Lean 4 Core",
            Self::LakeAndMathlib => "4. Lake & Mathlib",
            Self::Usage => "5. GUI / CLI / MCP",
            Self::TipsAndPitfalls => "6. Tips & Pitfalls",
            Self::OtherCrafts => "7. Other Crafts",
        }
    }

    pub fn body(&self) -> &'static str {
        match self {
            Self::GetStarted => concat!(
                "WHAT MATH LEVEL DO YOU NEED?\n",
                "- Playing around & understanding the workflow (High school / 1st-year undergrad): Basic algebra, inequalities (2ab <= a^2 + b^2), induction, and lemma vs. counterexample is plenty. Lean 4 gives a clean binary PASS/FAIL (0 errors, 0 sorrys).\n",
                "- Running your own problems (Undergrad discrete math / CS): Combinatorics, graph properties, algorithm invariants, and elementary number theory work great.\n",
                "- Auditing the two PAPER DEMO problems (Grad-level TCS): Those two examples come from STOC/FOCS online learning theory. For unformalized prose (without LEAN 4 VERIFIED), only trust proofs in domains where you can check the steps yourself.\n\n",
                "QUICK START\n",
                "1. Open Kanban to browse Queued / In Progress / Verified / Budget Cap cards, load a curated Pack (+ Lean Warmup, + Erdos Pack), or import from an arXiv/erdosproblems.com URL.\n",
                "2. Open Settings (Ctrl+,) -> LLM & Providers to run Live Model Discovery against local Ollama/LM Studio or Gemini/Anthropic/OpenAI-Compat.\n",
                "3. Click 'Run Proof Round' in the sidebar and watch the 8-stage animation in Rounds & Verifiers.\n\n",
                "Full guide: docs/MATH_AND_COGENTIC.md"
            ),
            Self::CogenticAndCalls => concat!(
                "THE ORIGINAL PAPER: COGENTIC (arXiv:2609.40324)\n",
                "Authors: Yang Cai, Vineet Gupta, Yanchen Jiang, Christopher Liaw, Aranyak Mehta, Grigoris Velegkas, Di Wang (Google Research / DeepMind / Yale)\n",
                "URL: https://arxiv.org/abs/2609.40324  |  Results: https://sites.google.com/view/cogentic\n\n",
                "8-STAGE PROVE-VERIFY-LEDGER LOOP:\n",
                "1. Orchestrator: allocates provers across directions (no math opinion).\n",
                "2. Independent Summarizers: build separate per-prover briefings from Record + Ledger.\n",
                "3. Parallel Provers: draft candidate proofs.\n",
                "4. Individual Verifier: assumes every step & citation is false until justified.\n",
                "5. Comparative Verifier: reads all drafts side-by-side for shared blind spots.\n",
                "6. Ledger Auditor: re-verifies salvaged lemmas in isolation; logs Dead Ends.\n",
                "7. Lean 4 Gate: mechanically checks formalized lemmas (zero sorrys).\n",
                "8. Process Advisor: tightens prover/verifier rigor instructions across rounds.\n\n",
                "WHAT IS A 'CALL'? (BUDGET MATH)\n",
                "One 'call' = one single LLM inference request (one prompt -> one completion).\n",
                "Each round with N provers spends 3*N + 2 calls (+1 per salvaged lemma audit, +1 if Lean 4 auto-formalize is enabled). Max Calls stops new rounds before crossing the cap."
            ),
            Self::Lean4Core => concat!(
                "LEAN 4 FORMAL VERIFICATION (CORE ENGINE)\n",
                "- Automatic Resolution: Finds `lean` on PATH or in `%USERPROFILE%\\.elan\\bin\\lean.exe`.\n",
                "- Anti-Cheating `sorry` / `admit` Guard: Rejects any proof containing `sorry` or `admit` even if `lean --stdin` exits 0.\n",
                "- One-Click & Auto-Formalization: Click 'Formalize & Verify in Lean 4' on any Verified Lemma card, or enable auto-formalization in Settings.\n\n",
                "RUNNABLE PURE LEAN 4 EXAMPLES (NO MATHLIB REQUIRED):\n\n",
                "-- 1. Telescoping Potential Descent (Cogentic Theorem 1 backbone)\n",
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
                "-- 2. Square Non-negativity & AM-GM (click 'Load Induction Example' in Ledger)\n",
                "Full guide: docs/LEAN4_GUIDE.md"
            ),
            Self::LakeAndMathlib => concat!(
                "USING LAKE & MATHLIB (`import Mathlib`)\n",
                "Standalone `lean --stdin` loads Lean 4 core only. To use Mathlib (`ring`, `nlinarith`, `linarith`, `Real`, `Finset`), point proofcraft at a local Lake workspace.\n\n",
                "1. ONE-TIME MATHLIB SETUP (PowerShell):\n",
                "   cd D:\\Dev\\repos\n",
                "   lake +leanprover/lean4:v4.36.0 new mathlib-lab math\n",
                "   cd mathlib-lab\n",
                "   lake exe cache get\n",
                "   lake build\n\n",
                "2. WIRE INTO PROOFCRAFT:\n",
                "   Settings (Ctrl+,) -> 2. Harness & Budget Caps -> set `Lake Project Dir` to `D:\\Dev\\repos\\mathlib-lab`.\n",
                "   proofcraft will automatically run `lake env lean --stdin` inside that folder.\n\n",
                "3. TEST MATHLIB IN LEDGER SCRATCHPAD:\n",
                "   import Mathlib\n",
                "   theorem cs2 (a1 a2 b1 b2 : Real) : (a1*b1 + a2*b2)^2 <= (a1^2 + a2^2)*(b1^2 + b2^2) := by\n",
                "     nlinarith [sq_nonneg (a1*b2 - a2*b1)]\n\n",
                "4. EXPORT LAKE PROJECT:\n",
                "   Manuscript -> 'Export Lake Project' writes lakefile.toml, lean-toolchain, Main.lean, and VerifiedLemmas.lean.\n",
                "Full guide: docs/LAKE_AND_MATHLIB.md"
            ),
            Self::Usage => concat!(
                "GUI SHORTCUTS & LAUNCHERS\n",
                "- Double-click `start.bat` (or `mcp-central-docs/starts/proofcraft-start.bat`).\n",
                "- Ctrl+Wheel: Zoom | Ctrl+B: Show/Hide Tab Bar | Ctrl+K: Command Palette | Ctrl+,: Settings.\n",
                "- Skins: Obsidian Dark, Crisp Light, Ivory Proof (paper theme).\n\n",
                "CLI (`proofcraft-mcp`)\n",
                "- proofcraft-mcp status | list-problems | get-problem --problem-id <id>\n",
                "- proofcraft-mcp import-url --url <url> | load-pack --pack lean_warmup\n",
                "- proofcraft-mcp run-round --problem-id <id> --provers 2\n",
                "- proofcraft-mcp lean-status | check-lean --code \"...\" | formalize-lemma ...\n",
                "- proofcraft-mcp consolidate --problem-id <id> --format tex | export-lake ...\n\n",
                "MCP STDIO SERVER (2026-07-28, 15 TOOLS): `proofcraft-mcp mcp`\n",
                "Full reference: docs/CLI_AND_MCP.md"
            ),
            Self::TipsAndPitfalls => concat!(
                "TIPS\n",
                "- Hybrid 4090 + API Routing: Run a local 30B model in Ollama/LM Studio for Summarizers & Advisor, and reserve API calls for Provers & Verifiers.\n",
                "- Always ground with arXiv: Use 'Fetch & Attach' in Workspace so provers cite real abstracts.\n\n",
                "PITFALLS & SAFETY\n",
                "- `PAPER DEMO` cards are static examples from Cogentic, not live background runs.\n",
                "- Never commit `.env` or API keys. Keep Max Calls/Run bounded.\n",
                "- Standalone `lean --stdin` has no Mathlib; configure `Lake Project Dir` for `import Mathlib`."
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
description: Orchestrate Cogentic-style multi-agent mathematical proof discovery and Lean 4/Lake formalization via proofcraft-mcp.
---
# proofcraft Skill

Use `proofcraft-mcp` tools to manage problems, import online conjectures, inspect the Verified Ledger, run bounded prove-verify rounds, verify lemmas in Lean 4 / Lake, and export LaTeX/Markdown manuscripts or Lake projects.
"#;

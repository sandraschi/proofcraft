# proofcraft Help & User Guide

## 1. Get Started
1. Launch `proofcraft-app`. On first run, two realistic demo problems are loaded into `%LOCALAPPDATA%\proofcraft\proofcraft.sqlite` from `seed/demo_data.json`.
2. Open **Settings** (`Ctrl+,`) -> **LLM & Providers** to run **Test Connection & Discover Models** against local Ollama (`http://127.0.0.1:11434`), LM Studio (`http://127.0.0.1:1234/v1`), or frontier APIs (Gemini, Anthropic, OpenAI-Compat).
3. Configure **Hybrid Role Routing**: assign a fast local 4090 model to Summarizer & Process Advisor, and your strongest reasoning model to Prover & Verifier.
4. Click **Run Proof Round** in the left sidebar.

## 2. Functions
- **Orchestrator & Summarizers:** Allocate provers across directions and generate independent briefings from the Record and Verified Ledger.
- **Adversarial Two-Tier Verification:** Individual Verifier checks local step rigor; Comparative Verifier reads all drafts in the round side-by-side to catch shared blind spots.
- **Verified Ledger & Dead Ends:** Confirmed intermediate lemmas are re-verified in isolation before promotion; refuted approaches are logged as Dead Ends.
- **Process Advisor:** Synthesizes cross-round verification logs into standing instructions without holding a mathematical opinion.

## 3. Usage (GUI / CLI / MCP)
- **GUI Shortcuts:** `Ctrl+Wheel` zoom, `Ctrl+B` toggle tab bar, `Ctrl+K` command palette, `Ctrl+,` settings.
- **CLI:** `proofcraft-mcp status`, `proofcraft-mcp list-problems`, `proofcraft-mcp run-round --problem-id <id> --provers 2`, `proofcraft-mcp export --problem-id <id> --format tex`.
- **MCP:** `proofcraft-mcp mcp` (JSON-RPC 2.0 on `stdout`, diagnostics on `stderr`).

## 4. Tips & Tricks
- Use **Ivory Proof (Paper)** skin for long reading sessions on exported proofs and ledger lemmas.
- Set a conservative `Max Calls` cap (e.g., 40-100) when testing a new conjecture.

## 5. Pitfalls & Safety
- Never commit `.env` or API keys. Keys are read from environment variables (`GEMINI_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) or entered masked in Settings.
- Adversarial LLM verifiers reduce but do not eliminate subtle proof errors; always audit exported manuscripts.

## 6. Planned Extensions
- Optional Lean 4 (`lake env lean`) formal sub-lemma verification.
- Mid-run `arxiv-mcp` literature reviewer dispatch.

## 7. Other Crafts
- `rsicraft`: evolutionary program search & local LoRA self-improvement.
- `logiccraft`: formal logic tooling.
- `writercraft`: long-form authoring.

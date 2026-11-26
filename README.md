# proofcraft

**Cogentic-style multi-agent automated proof discovery studio -- pure Rust (L1 core, L2 CLI + MCP, L3 egui).**

Inspired by [Cogentic: Multi-Agent Orchestration for Automated Proof Discovery](https://arxiv.org/abs/2609.40324) (arXiv:2609.40324v2), `proofcraft` brings the iterative prove-verify-ledger architecture to a local-first workstation (RTX 4090 / Ollama / LM Studio + optional frontier APIs).

## Architecture

- **`crates/proofcraft-core` (L1):** SQLite-backed Record & Verified Ledger, independent Summarizer briefings, Prover + Two-Tier Adversarial Verifier loop, Isolated Lemma Auditor, non-mathematical Process Advisor, live LLM provider discovery, and hard call/round/USD budget caps.
- **`crates/proofcraft-mcp` (L2):** `clap` CLI and pure-Rust MCP `2026-07-28` stdio server (`proofcraft-mcp mcp`) with `skill://proofcraft/SKILL.md` resource.
- **`crates/proofcraft-app` (L3):** Native `eframe`/`egui 0.37` desktop studio with three high-contrast skins (Obsidian Dark, Crisp Light, Ivory Proof), `Ctrl+Wheel` zoom, universal menu bar, hideable tab bar (`Ctrl+B`), 7-tab Help Center, and full-window Settings (`Ctrl+,`).

## Feature Matrix

| Feature | Status | Notes |
|---|---|---|
| SQLite Record, Directions, & Verified Ledger | `Implemented` | Auto-loads `seed/demo_data.json` on empty DB |
| Two-Tier Adversarial Verification (Single + Comparative) | `Implemented` | Both verdicts required to mark a problem `Verified` |
| Isolated Lemma Auditor & Dead-End Exclusion | `Implemented` | Re-verifies salvaged fragments in isolation |
| Process Advisor (non-mathematical meta-instructions) | `Implemented` | Updates standing instructions per round |
| Hard Budget Cap (calls / rounds / USD failsafe) | `Implemented` | Enforced before each round starts |
| Live LLM Discovery (Ollama, LM Studio, OpenAI-Compat, Gemini, Anthropic) | `Implemented` | Excludes embed-only models; masked key status |
| Hybrid Role Routing (local 4090 + API overrides) | `Implemented` | Separate model overrides for Prover, Verifier, Summarizer, Advisor |
| Pure-Rust CLI & MCP Stdio (`2026-07-28`) | `Implemented` | `proofcraft-mcp mcp`, 7 tools + `skill://` resource |
| Native `egui` Studio (3 skins, Help Center, Settings, Export) | `Implemented` | `--smoke-test` headless gate included |
| Markdown & LaTeX Manuscript Export | `Implemented` | Saves `.md` or `.tex` with verified lemmas |
| Optional Lean 4 (`lake env lean`) sub-lemma check | `Planned` | Subprocess formalizer hook |
| Live `arxiv-mcp` Literature Reviewer dispatch | `Planned` | Mid-run citation & theorem retrieval |

## Quick Start

```powershell
# Build and run all checks
just ci

# Launch desktop studio
cargo run -p proofcraft-app

# Run headless smoke test
cargo run -p proofcraft-app -- --smoke-test

# Run CLI or MCP stdio server
cargo run -p proofcraft-mcp -- status
cargo run -p proofcraft-mcp -- mcp
```

## License

Dual-licensed under `MIT OR Apache-2.0`.

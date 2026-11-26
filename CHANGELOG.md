# Changelog

## 0.2.0 - 2026-11-26
- Added 4-column Kanban pipeline board with `PAPER DEMO` vs `LOCAL` honesty badges.
- Added animated 8-stage Cogentic + Lean 4 loop visualizer and round telemetry chart.
- Added online problem importer (`arXiv`, `erdosproblems.com`, `.lean`), curated problem packs, and standalone Lake project export (`lakefile.toml` + `VerifiedLemmas.lean`).
- Added modular `docs/` stack, 4-panel usage cartoon, and 7-tab Help Center (57 passing tests).

## 0.1.1 - 2026-11-26
- Added Grounded Literature Reviewer (`literature.rs`, arXiv Atom fetch + SQLite persistence + prover briefing injection).
- Added optional Lean 4 subprocess sub-lemma check (`lean.rs`, `proof_check_lean`, Ledger panel).
- Added `std::thread::scope` parallel prover dispatch for API providers.
- Added collapsible Proof Copilot right dock in `proofcraft-app` and topped up workspace tests above Tier 1 floor (>= 20 tests).

## 0.1.0 - 2026-11-26
- Initial greenfield Pure-Rust ArtCraft scaffold (`proofcraft-core`, `proofcraft-mcp`, `proofcraft-app`) authorized by Sandra.
- Implemented Cogentic-inspired prove-verify-ledger engine, SQLite persistence, embedded `seed/demo_data.json`, live LLM discovery/routing, hard budget caps, MCP `2026-07-28` stdio server, and 3-skin `egui` studio with 7-tab Help Center and Settings.

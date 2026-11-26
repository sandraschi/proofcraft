## proofcraft v0.2.0 (2026-11-26)

- **Kanban Pipeline Board & Demo Honesty:** 4-column Kanban (`Queued`, `In Progress`, `Verified`, `Budget Cap`) with explicit `PAPER DEMO (arXiv:2609.40324)` vs `LOCAL` badges.
- **Animated 8-Stage Cogentic + Lean 4 Loop & Telemetry Chart:** Live pulse animation during rounds, Play/Pause replay, and per-round attempt/lemma bar chart.
- **Online Problem Importer & Curated Packs:** Import from `arxiv.org`, `erdosproblems.com`, `openproblemgarden.org`, or raw `.lean`/`.md` URLs, plus one-click *Lean 4 Warmup*, *Erdos Combinatorics*, and *Cogentic Paper* packs.
- **Lean 4.36.0 & Lake / Mathlib Core Engine:** Automatic `elan` resolution, anti-cheating `sorry`/`admit` guard, `lake env lean --stdin` workspace support, one-click & auto-formalization, and standalone `Export Lake Project` (`lakefile.toml` + `Main.lean` + `VerifiedLemmas.lean`).
- **Modular Docs Stack & 7-Tab Help Center:** Separate guides in `docs/` (`MATH_AND_COGENTIC.md`, `LEAN4_GUIDE.md`, `LAKE_AND_MATHLIB.md`, `CLI_AND_MCP.md`), 4-panel usage cartoon, and 57 passing workspace tests.

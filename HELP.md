# proofcraft Help Hub

> Modular guides live in `docs/`:
> - [Math, Cogentic & Calls](docs/MATH_AND_COGENTIC.md)
> - [Lean 4 Core Guide](docs/LEAN4_GUIDE.md)
> - [Lake & Mathlib Guide](docs/LAKE_AND_MATHLIB.md)
> - [CLI, MCP & Launchers](docs/CLI_AND_MCP.md)

## Quick Overview
1. **Fastest Install:** just tell Claude / agy / Cursor: *"install proofcraft from gh sandraschi"*.
2. **Math Level:** High-school / 1st-year undergrad math is enough to use the Kanban, Verifiers, Ledger, and Lean 4 `PASS`/`FAIL` checks. The two `PAPER DEMO` problems are graduate TCS examples from [arXiv:2609.40324](https://arxiv.org/abs/2609.40324).
3. **What is a Call?** `1` call = `1` LLM inference request. One round with $N$ provers spends `3*N + 2` calls (+1 per isolated lemma audit, +1 if Lean 4 auto-formalize is on).
4. **Lean 4 & Mathlib:** Standalone `lean --stdin` works out of the box (with `sorry`/`admit` blocked). To use `import Mathlib`, set **Lake Project Dir** in Settings (`Ctrl+,`) to a `mathlib-lab` folder built with `lake exe cache get && lake build`.
5. **Shortcuts:** `Ctrl+Wheel` zoom, `Ctrl+B` toggle tab bar, `Ctrl+K` command palette, `Ctrl+,` settings.

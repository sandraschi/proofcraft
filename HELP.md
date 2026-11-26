# proofcraft Help & User Guide

## 1. The Original Paper: Cogentic (arXiv:2609.40324)

- **Paper:** [Cogentic: Multi-Agent Orchestration for Automated Proof Discovery](https://arxiv.org/abs/2609.40324) (arXiv:2609.40324v2)
- **Authors:** Yang Cai, Vineet Gupta, Yanchen Jiang, Christopher Liaw, Aranyak Mehta, Grigoris Velegkas, Di Wang (Google Research / Google DeepMind / Yale)
- **Results Index:** <https://sites.google.com/view/cogentic>

### Why Cogentic Works
Single-shot LLM generation is rarely enough for open research problems that require exploring competing conjectures, surviving subtle technical obstructions, and retaining verified progress over a long horizon. Cogentic solves this with a lean, disk-backed **prove-verify-ledger loop** that achieved five STOC/FOCS-level results using **$O(100)$ model calls** for most problems and **$O(1000)$ calls** for the hardest:

1. **Online Inverse Linear Optimization:** First efficient and proper $O(d)$ regret bound uniform in $T$ at $O(d^2)$ per round.
2. **Two-Sided Bulow-Klemperer Competition Complexity:** Tight $+2$ sellers on the smaller side suffice (down from $\ge 20{,}000$ on both sides).
3. **Anytime Regret with $n$ Experts:** Matches the fixed-horizon leading constant $\sqrt{t \ln n / 2}$ simultaneously for all $t \ge 1$.
4. **Simple vs. Optimal Revenue (Single Additive Buyer):** Improved approximation ratio from $5.2$ to $3.52$.
5. **Price of Anarchy for Autobidding Auctions:** Tight $1.5$ PoA for 2 bidders ($\mathsf{pFPA}_1$) and $2 - \frac{1}{4n+1}$ for $n$ bidders ($\mathsf{pFPA}_{2n}$).

---

## 2. Lean 4 Formal Verification (Core Capability)

Natural-language proofs can still conceal subtle gaps. `proofcraft` integrates **Lean 4** (`elan` / `lean` / `lake`) directly into the Verified Ledger and round pipeline:

- **Automatic Toolchain Detection:** Resolves `lean` and `lake` from `PATH` or `%USERPROFILE%\.elan\bin`.
- **Anti-Cheating `sorry` / `admit` Guard:** Lean 4 exits `0` with a warning when a proof contains `sorry` or `admit`. `proofcraft` scans non-comment tokens and rejects any proof containing `sorry` or `admit`.
- **Lake / Mathlib Workspaces:** Configure `Lake Project Dir` in **Settings -> Harness & Budget Caps** (pointing to a folder with `lakefile.lean` or `lakefile.toml`) to run `lake env lean --stdin` with full Mathlib imports.
- **One-Click & Auto-Formalization:** Click **Formalize & Verify in Lean 4** on any Verified Ledger card, or enable **Auto-formalize promoted lemmas in Lean 4** in Settings.

### Example Lean 4 Code (Runnable in the Ledger Scratchpad)

```lean
-- Example 1: Kernel reduction
theorem sum_two_squares : (3 : Nat)^2 + 4^2 = 5^2 := rfl

-- Example 2: Inductive proof (sum of first n naturals)
def sumTo : Nat -> Nat
  | 0 => 0
  | n + 1 => sumTo n + (n + 1)

theorem sumTo_formula (n : Nat) : 2 * sumTo n = n * (n + 1) := by
  induction n with
  | zero => rfl
  | succ k ih =>
    simp [sumTo, Nat.mul_add, Nat.add_mul, ih]
    omega
```

---

## 3. Usage (GUI / CLI / MCP)

- **GUI Shortcuts:** `Ctrl+Wheel` zoom, `Ctrl+B` toggle tab bar, `Ctrl+K` command palette, `Ctrl+,` settings.
- **Launchers:** Double-click `start.bat` (or `mcp-central-docs/starts/proofcraft-start.bat`), or run `.\start.ps1 -Smoke` / `.\start.ps1 -Mcp`.
- **CLI:** `proofcraft-mcp status`, `proofcraft-mcp list-problems`, `proofcraft-mcp add-literature --problem-id <id> --arxiv-id 2609.40324`, `proofcraft-mcp run-round --problem-id <id> --provers 2`, `proofcraft-mcp lean-status`, `proofcraft-mcp check-lean --code "..."`, `proofcraft-mcp formalize-lemma --problem-id <id> --ledger-id <lem>`, `proofcraft-mcp consolidate --problem-id <id> --format tex`.
- **MCP (`2026-07-28`):** `proofcraft-mcp mcp` exposes 12 tools plus `skill://proofcraft/SKILL.md`.

## 4. Pitfalls & Safety
- Never commit `.env` or API keys.
- Keep `Max Calls/Run` bounded when testing new conjectures.
- Standalone `lean --stdin` includes Lean 4 core only; set `Lake Project Dir` for `import Mathlib`.

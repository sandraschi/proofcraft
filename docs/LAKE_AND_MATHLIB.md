# Lake Workspaces & Using Mathlib

Standalone `lean --stdin` loads Lean 4 core only (`Nat`, `Int`, `omega`, `simp`, `induction`). To use **`import Mathlib`** and tactics like `ring`, `nlinarith`, `linarith`, `positivity`, `Real`, or `Finset`, connect `proofcraft` to a local **Lake** workspace.

---

## 1. Do You Have `lake` Installed?
If you installed Lean 4 via `elan`, `lake` is already installed right alongside `lean` (on Windows: `%USERPROFILE%\.elan\bin\lake.exe`). `proofcraft` resolves `lake` automatically.

---

## 2. One-Time Mathlib Workspace Setup

Run these commands once in PowerShell (`lake exe cache get` downloads prebuilt `.olean` binaries in ~1-2 minutes so you do not compile Mathlib from source):

```powershell
cd D:\Dev\repos
lake +leanprover/lean4:v4.36.0 new mathlib-lab math
cd mathlib-lab
lake exe cache get
lake build
```

*(Note: If `mathlib4` pins a matching Lean version in `lean-toolchain`, `elan` automatically downloads that toolchain for `mathlib-lab`.)*

---

## 3. Wire `mathlib-lab` into `proofcraft`

1. Open `proofcraft-app` -> **Settings** (`Ctrl+,`) -> **2. Harness & Budget Caps**.
2. Set **Lake Project Dir** to `D:\Dev\repos\mathlib-lab`.

Whenever **Lake Project Dir** points to a folder containing `lakefile.toml` or `lakefile.lean`, `proofcraft` automatically runs checks via `lake env lean --stdin` inside that directory (while still enforcing the `sorry` / `admit` guard).

---

## 4. Test Mathlib in the Ledger Scratchpad

```lean
import Mathlib

theorem cauchy_schwarz_2d (a1 a2 b1 b2 : ℝ) :
    (a1 * b1 + a2 * b2)^2 ≤ (a1^2 + a2^2) * (b1^2 + b2^2) := by
  nlinarith [sq_nonneg (a1 * b2 - a2 * b1)]
```

---

## 5. Exporting a Standalone Lake Project

In **Manuscript -> Export Lake Project** (or via CLI `export-lake` / MCP `proof_export_lake`), `proofcraft` writes:
- `lakefile.toml`
- `lean-toolchain`
- `VerifiedLemmas.lean` (every `LEAN 4 VERIFIED` lemma from the Ledger)
- `Main.lean`

You can then run `lake build` directly inside the exported folder.

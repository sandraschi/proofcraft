---
name: lean4-formalizer
description: Verify and formalize mathematical lemmas using local Lean 4 and Lake workspaces via proofcraft-mcp.
---
# Lean 4 Formalizer Skill

1. Call `proof_lean_status` to confirm Lean 4.36+ is available.
2. Never emit `sorry` or `admit` -- `proof_check_lean` rejects them even if `lean --stdin` exits 0.
3. Use `proof_formalize_lemma` to translate Verified Ledger entries into self-contained Lean 4 and persist verified code.

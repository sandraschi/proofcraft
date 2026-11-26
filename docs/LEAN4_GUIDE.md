# Lean 4 Core Formal Verification Guide

`proofcraft` integrates **Lean 4** directly into the Verified Ledger and round pipeline as a mechanical truth anchor.

## 1. Automatic Detection & Anti-Cheating Guard

- **Toolchain resolution:** Automatically finds `lean` on `PATH` or in `%USERPROFILE%\.elan\bin\lean.exe` (shown in the bottom status bar, e.g. `Lean (version 4.36.0...)`).
- **`sorry` / `admit` guard:** Lean 4 exits `0` with only a warning when a proof contains `sorry` or `admit`. `proofcraft` scans non-comment tokens and rejects any proof containing `sorry` or `admit`, ensuring a `LEAN 4 VERIFIED` badge means zero holes.
- **One-click & Auto-formalization:** Click **Formalize & Verify in Lean 4** on any Verified Ledger lemma, or enable **Auto-formalize promoted lemmas in Lean 4** in Settings (`Ctrl+,`).

---

## 2. Runnable Pure Lean 4 Examples (No Mathlib Required)

All of these run directly in **Verified Ledger -> Lean 4 Formal Verification**:

### Example 1: Telescoping Potential Descent (Cogentic Theorem 1 Backbone)
```lean
def sumStep (f : Nat -> Int) : Nat -> Int
  | 0 => 0
  | n + 1 => sumStep f n + (f n - f (n + 1))

theorem telescoping_potential (f : Nat -> Int) (T : Nat) :
    sumStep f T = f 0 - f T := by
  induction T with
  | zero => simp [sumStep]
  | succ k ih =>
    simp [sumStep, ih]
    omega

theorem potential_nonneg_step_bound (f : Nat -> Int) (T : Nat)
    (h_nonneg : f T >= 0) :
    sumStep f T <= f 0 := by
  rw [telescoping_potential]
  omega
```

### Example 2: Square Non-negativity & AM-GM Inequality
```lean
theorem int_sq_nonneg (x : Int) : 0 <= x * x := by
  rcases Int.le_total 0 x with h | h
  · exact Int.mul_nonneg h h
  · have hn : 0 <= -x := by omega
    have eq : x * x = (-x) * (-x) := by rw [Int.neg_mul_neg]
    rw [eq]
    exact Int.mul_nonneg hn hn

theorem am_gm_int (a b : Int) : 2 * (a * b) <= a * a + b * b := by
  have hsq : 0 <= (a - b) * (a - b) := int_sq_nonneg (a - b)
  have hexp : (a - b) * (a - b) = a * a - a * b - b * a + b * b := by
    simp only [Int.sub_mul, Int.mul_sub]
    omega
  have hcomm : b * a = a * b := Int.mul_comm b a
  omega
```

### Example 3: Sum of First n Naturals (Induction)
```lean
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

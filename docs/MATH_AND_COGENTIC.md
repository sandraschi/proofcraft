# General Math, Cogentic Paper & Call Budgets

## 1. What Math Level Do You Need?

- **Playing around & understanding the workflow (High school / 1st-year undergrad):** If you know basic algebra, inequalities ($2ab \le a^2 + b^2$), proof by induction, and what a **lemma** vs. a **counterexample** is, the Kanban board, adversarial verifiers, Verified Ledger, and Lean 4 `PASS`/`FAIL` checks all make immediate sense.
- **Running your own problems (Undergrad discrete math / CS):** Combinatorics, graph properties, algorithm invariants, recurrence bounds, and elementary number theory are the sweet spot. With **Lean 4** enabled, `lean --stdin` + the `sorry`/`admit` guard mechanically checks every step.
- **Auditing the two pre-loaded `PAPER DEMO` problems (Grad-level TCS / optimization):** Those two examples come from STOC/FOCS online learning theory. You do not need that background to run `proofcraft` on your own problems, but for *unformalized* natural-language prose (without `LEAN 4 VERIFIED`), only trust proofs in domains where you can check the steps yourself.

---

## 2. The Original Paper: Cogentic (`arXiv:2609.40324`)

- **Paper:** [Cogentic: Multi-Agent Orchestration for Automated Proof Discovery](https://arxiv.org/abs/2609.40324) (Yang Cai, Vineet Gupta, Yanchen Jiang, Christopher Liaw, Aranyak Mehta, Grigoris Velegkas, Di Wang -- Google Research / Google DeepMind / Yale)
- **Results Index:** <https://sites.google.com/view/cogentic>

Cogentic solves open research problems using an iterative **prove-verify-ledger loop** in **$O(100)$ to $O(1000)$ model calls**:
1. **Online Inverse Linear Optimization:** First efficient & proper $O(d)$ regret bound uniform in $T$ at $O(d^2)$ per round.
2. **Two-Sided Bulow-Klemperer Competition Complexity:** Tight $+2$ sellers on the smaller side suffice (down from $\ge 20{,}000$).
3. **Anytime Regret with $n$ Experts:** Matches the fixed-horizon leading constant $\sqrt{t \ln n / 2}$ simultaneously for all $t \ge 1$.
4. **Simple vs. Optimal Revenue (Single Additive Buyer):** Improved factor from $5.2$ to $3.52$.
5. **Price of Anarchy for Autobidding Auctions:** Tight $1.5$ PoA for 2 bidders ($\mathsf{pFPA}_1$) and $2 - \frac{1}{4n+1}$ for $n$ bidders ($\mathsf{pFPA}_{2n}$).

---

## 3. What is a "Call"? (Budget & Round Math)

A **call** is **one single LLM inference request** (one prompt sent to Ollama, LM Studio, Gemini, Anthropic, or OpenAI-Compat, and one completion returned).

Each Cogentic round with $N$ provers spends **`3*N + 2` calls**:
- **3 calls per prover:** `1` Summarizer briefing + `1` Prover draft + `1` Individual Adversarial Verifier.
- **+ 2 calls per round:** `1` Cross-Draft Comparative Verifier + `1` Process Advisor.
- **+ 1 call per salvaged lemma** (Isolated Lemma Auditor re-verification, plus `+1` if **Auto-formalize in Lean 4** is enabled).

A **`Max Calls: 200`** cap stops new rounds before crossing 200 total LLM requests on that problem, preventing runaway API spend or infinite overnight GPU loops.

---

## 4. Where to Find Problems

- **Built-in Packs (Kanban -> Packs):** *Lean 4 Warmup Pack* (3 undergrad problems), *Erdos Combinatorics Pack* (2 problems), *Cogentic Paper Pack* (2 demos).
- **Online Importer (Kanban -> Import Online):** Paste an `arxiv.org` ID/URL, `erdosproblems.com/<id>`, `openproblemgarden.org`, or raw `.lean`/`.md` URL.

use crate::llm;
use crate::model::{
    AdvisorNote, Attempt, BudgetConfig, LedgerEntry, LedgerKind, ProblemDetail, ProblemStatus,
    ProviderConfig, RoleRouting, RoundReport, Verdict,
};
use crate::store::ProofStore;
use crate::{ProofError, Result};
use uuid::Uuid;

pub fn check_budget_remaining(
    detail: &ProblemDetail,
    budget: &BudgetConfig,
    needed_calls: u32,
) -> Result<()> {
    let cap = detail.problem.max_calls.min(budget.max_calls_per_run);
    if detail.problem.calls_used + needed_calls > cap {
        return Err(ProofError::BudgetExceeded(format!(
            "problem {} used {}/{} calls; round needs {} calls",
            detail.problem.id, detail.problem.calls_used, cap, needed_calls
        )));
    }
    if detail.problem.rounds_completed >= budget.max_rounds_per_run {
        return Err(ProofError::BudgetExceeded(format!(
            "problem {} reached max rounds ({})",
            detail.problem.id, budget.max_rounds_per_run
        )));
    }
    Ok(())
}

pub fn build_prover_briefing(detail: &ProblemDetail, prover_idx: u32) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Problem: {}\nStatement: {}\n\n",
        detail.problem.title, detail.problem.statement
    ));
    out.push_str("Verified Ledger (reusable without re-proving):\n");
    for l in detail
        .ledger
        .iter()
        .filter(|e| e.kind == LedgerKind::VerifiedLemma)
    {
        out.push_str(&format!("- {}: {}\n", l.title, l.statement));
    }
    out.push_str("\nExcluded Dead Ends (do NOT revisit):\n");
    for l in detail
        .ledger
        .iter()
        .filter(|e| e.kind == LedgerKind::DeadEnd)
    {
        out.push_str(&format!("- {}: {}\n", l.title, l.statement));
    }
    if !detail.advisor_notes.is_empty() {
        out.push_str("\nProcess Advisor Standing Instructions:\n");
        for n in &detail.advisor_notes {
            out.push_str(&format!("- [R{}] {}\n", n.round, n.instruction_delta));
        }
    }
    out.push_str(&format!(
        "\nYou are Prover-{prover_idx}. Produce a rigorous natural-language proof."
    ));
    out
}

pub fn parse_verdict_response(raw: &str) -> (Verdict, String) {
    let mut lines = raw.lines();
    let first = lines.next().unwrap_or("").trim();
    let verdict = if first.to_ascii_uppercase().contains("PASS") {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    let rest: String = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    let objection = if rest.is_empty() {
        raw.trim().to_string()
    } else {
        rest
    };
    (verdict, objection)
}

pub fn execute_round(
    store: &ProofStore,
    provider: &ProviderConfig,
    routing: &RoleRouting,
    budget: &BudgetConfig,
    problem_id: &str,
    num_provers: u32,
) -> Result<RoundReport> {
    let detail = store.get_problem_detail(problem_id)?;
    // Each prover takes 1 summarizer + 1 prover + 1 single verifier, plus 1 comparative verifier + 1 advisor
    let estimated_calls = num_provers * 3 + 2;
    check_budget_remaining(&detail, budget, estimated_calls)?;

    let next_round = detail.problem.rounds_completed + 1;
    let direction_id = detail
        .directions
        .first()
        .map(|d| d.id.clone())
        .unwrap_or_else(|| "dir-default".to_string());

    let mut calls_spent = 0u32;
    let mut round_attempts = Vec::new();

    for idx in 1..=num_provers {
        let base_brief = build_prover_briefing(&detail, idx);
        let briefing = llm::complete(
            provider,
            &routing.summarizer_model,
            "You are a Cogentic Summarizer. Condense the prior record and ledger into a focused briefing for one independent prover.",
            &base_brief,
        )?;
        calls_spent += 1;

        let draft = llm::complete(
            provider,
            &routing.prover_model,
            "You are a Cogentic Prover. Write a complete, rigorous mathematical proof. State every assumption explicitly.",
            &briefing,
        )?;
        calls_spent += 1;

        let single_raw = llm::complete(
            provider,
            &routing.verifier_model,
            "You are an Adversarial Verifier. Assume EVERY step is wrong and EVERY citation is false until justified. First line MUST be PASS or FAIL, followed by your exact objection.",
            &draft,
        )?;
        calls_spent += 1;
        let (single_verdict, single_objection) = parse_verdict_response(&single_raw);

        let attempt = Attempt {
            id: format!(
                "att-r{}-p{}-{}",
                next_round,
                idx,
                &Uuid::new_v4().to_string()[..6]
            ),
            round: next_round,
            prover: format!("Prover-{idx}"),
            direction_id: direction_id.clone(),
            method: format!("Round {next_round} autonomous direction"),
            summary: draft.lines().take(2).collect::<Vec<_>>().join(" "),
            draft_markdown: draft,
            single_verdict,
            single_objection,
            comp_verdict: Verdict::Pending,
            comp_objection: String::new(),
        };
        round_attempts.push(attempt);
    }

    // Comparative verifier across all drafts in this round
    let combined_drafts = round_attempts
        .iter()
        .map(|a| format!("=== {} ===\n{}", a.prover, a.draft_markdown))
        .collect::<Vec<_>>()
        .join("\n\n");
    let comp_raw = llm::complete(
        provider,
        &routing.verifier_model,
        "You are the Comparative Adversarial Verifier. Compare all drafts side-by-side for shared blind spots. First line MUST be PASS or FAIL.",
        &combined_drafts,
    )?;
    calls_spent += 1;
    let (comp_verdict, comp_objection) = parse_verdict_response(&comp_raw);

    let mut verified_solution_found = false;
    let mut lemmas_promoted = 0usize;

    for mut att in round_attempts.drain(..) {
        att.comp_verdict = comp_verdict;
        att.comp_objection = comp_objection.clone();
        if att.single_verdict == Verdict::Pass && att.comp_verdict == Verdict::Pass {
            verified_solution_found = true;
        } else if att.single_verdict == Verdict::Pass {
            // Salvage confirmed fragment into Verified Ledger after isolated check
            let iso_raw = llm::complete(
                provider,
                &routing.verifier_model,
                "Isolated Lemma Auditor: verify whether the core lemma of this draft stands on its own without external assumptions. First line PASS or FAIL.",
                &att.draft_markdown,
            )?;
            calls_spent += 1;
            let (iso_v, iso_obj) = parse_verdict_response(&iso_raw);
            if iso_v == Verdict::Pass {
                let entry = LedgerEntry {
                    id: format!("lem-r{}-{}", next_round, &Uuid::new_v4().to_string()[..6]),
                    kind: LedgerKind::VerifiedLemma,
                    title: format!("R{} Lemma ({})", next_round, att.prover),
                    statement: att.summary.clone(),
                    proof_sketch: iso_obj,
                    isolated_verified: true,
                    source_attempt: att.id.clone(),
                };
                store.add_ledger_entry(problem_id, &entry)?;
                lemmas_promoted += 1;
            }
        }
        store.record_attempt(problem_id, &att)?;
    }

    // Process Advisor (no mathematical opinion)
    let advisor_obs = llm::complete(
        provider,
        &routing.advisor_model,
        "You are the Process Advisor. You are FORBIDDEN from having any mathematical opinion or suggesting proof techniques. Output one concise guardrail instruction for future provers/verifiers based on failure modes.",
        &format!("Single & comparative verdict notes:\n{comp_objection}"),
    )?;
    calls_spent += 1;

    let note = AdvisorNote {
        round: next_round,
        observation: format!("Round {next_round} audit completed ({calls_spent} calls)."),
        instruction_delta: advisor_obs.trim().to_string(),
    };
    store.add_advisor_note(problem_id, &note)?;

    let next_status = if verified_solution_found {
        ProblemStatus::Verified
    } else {
        ProblemStatus::InProgress
    };
    store.update_problem_progress(problem_id, next_status, calls_spent, next_round)?;

    Ok(RoundReport {
        problem_id: problem_id.to_string(),
        round: next_round,
        calls_spent,
        attempts_added: num_provers as usize,
        lemmas_promoted,
        verified_solution_found,
        advisor_delta: note.instruction_delta,
    })
}

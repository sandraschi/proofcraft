#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::harness::{build_prover_briefing, check_budget_remaining, parse_verdict_response};
use crate::help::HelpTab;
use crate::llm::{is_embed_only_model, key_status_badge};
use crate::model::{
    AdvisorNote, Attempt, BudgetConfig, LedgerEntry, LedgerKind, ProblemStatus, ProviderConfig,
    ProviderKind, RoleRouting, Verdict,
};
use crate::store::ProofStore;
use crate::{attempt_passed_both, dispatch, Command, ProofError};

#[test]
fn embedded_seed_loads_both_demo_problems() {
    let store = ProofStore::open_in_memory().expect("in-memory db");
    let loaded = store.load_embedded_seed().expect("seed load");
    assert_eq!(loaded, 2);
    let problems = store.list_problems().expect("list");
    assert_eq!(problems.len(), 2);
    assert_eq!(problems[0].id, "prob-anytime-experts");
    assert_eq!(problems[0].status, ProblemStatus::Verified);
}

#[test]
fn create_problem_and_direction_round_trip() {
    let store = ProofStore::open_in_memory().expect("db");
    let p = store
        .create_problem("Test Conjecture", "Combinatorics", "Show f(n) <= 2n.", 50)
        .expect("create");
    let d = store
        .add_direction(&p.id, "Induction", "Prove by induction on n.")
        .expect("dir");
    let detail = store.get_problem_detail(&p.id).expect("detail");
    assert_eq!(detail.directions.len(), 1);
    assert_eq!(detail.directions[0].id, d.id);
}

#[test]
fn clear_demo_data_and_reload_works() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    store.clear_all().expect("clear");
    assert!(store.list_problems().expect("list").is_empty());
    store.load_embedded_seed().expect("reload");
    assert_eq!(store.list_problems().expect("list").len(), 2);
}

#[test]
fn budget_cap_blocks_excess_calls() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let detail = store
        .get_problem_detail("prob-anytime-experts")
        .expect("detail");
    let tight_budget = BudgetConfig {
        max_calls_per_run: 90,
        max_rounds_per_run: 10,
        max_usd_per_run: 1.0,
    };
    let err = check_budget_remaining(&detail, &tight_budget, 10).unwrap_err();
    assert!(matches!(err, ProofError::BudgetExceeded(_)));
}

#[test]
fn budget_cap_blocks_max_rounds() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let detail = store
        .get_problem_detail("prob-anytime-experts")
        .expect("detail");
    let tight_budget = BudgetConfig {
        max_calls_per_run: 500,
        max_rounds_per_run: 3,
        max_usd_per_run: 5.0,
    };
    let err = check_budget_remaining(&detail, &tight_budget, 1).unwrap_err();
    assert!(matches!(err, ProofError::BudgetExceeded(_)));
}

#[test]
fn parse_verdict_handles_pass_and_fail() {
    let (v1, o1) = parse_verdict_response("PASS\nAll steps verified.");
    assert_eq!(v1, Verdict::Pass);
    assert_eq!(o1, "All steps verified.");

    let (v2, o2) = parse_verdict_response("FAIL\nMissing base case.");
    assert_eq!(v2, Verdict::Fail);
    assert_eq!(o2, "Missing base case.");
}

#[test]
fn briefing_includes_ledger_and_excludes_dead_ends() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let detail = store
        .get_problem_detail("prob-anytime-experts")
        .expect("detail");
    let brief = build_prover_briefing(&detail, 1);
    assert!(brief.contains("L1: Bounded Active Instance Pool"));
    assert!(brief.contains("D1: Non-restarted single MW"));
    assert!(brief.contains("Prover-1"));
}

#[test]
fn export_manuscript_markdown_and_tex() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let md = store
        .export_manuscript("prob-anytime-experts", "md")
        .expect("md");
    assert!(md.contains("# Anytime Regret"));
    assert!(md.contains("## Verified Ledger"));

    let tex = store
        .export_manuscript("prob-anytime-experts", "tex")
        .expect("tex");
    assert!(tex.contains("\\documentclass{article}"));
    assert!(tex.contains("\\begin{proof}"));
}

#[test]
fn dispatch_status_and_list_problems() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let provider = ProviderConfig::default();
    let routing = RoleRouting::default();
    let budget = BudgetConfig::default();

    let status = dispatch(&store, &provider, &routing, &budget, Command::Status).expect("status");
    assert_eq!(status["app"], "proofcraft");
    assert_eq!(status["problem_count"], 2);

    let ledger = dispatch(
        &store,
        &provider,
        &routing,
        &budget,
        Command::ListLedger {
            problem_id: "prob-anytime-experts".to_string(),
        },
    )
    .expect("ledger");
    assert_eq!(ledger.as_array().map(|a| a.len()), Some(3));
}

#[test]
fn dispatch_rejects_empty_problem_title() {
    let store = ProofStore::open_in_memory().expect("db");
    let err = dispatch(
        &store,
        &ProviderConfig::default(),
        &RoleRouting::default(),
        &BudgetConfig::default(),
        Command::CreateProblem {
            title: "  ".to_string(),
            domain: "Math".to_string(),
            statement: "Prove X".to_string(),
            max_calls: 50,
        },
    )
    .unwrap_err();
    assert!(matches!(err, ProofError::InvalidInput(_)));
}

#[test]
fn embed_only_model_filter_works() {
    assert!(is_embed_only_model("nomic-embed-text:latest"));
    assert!(is_embed_only_model("BAAI/bge-large-en"));
    assert!(!is_embed_only_model("qwen3:32b"));
    assert!(!is_embed_only_model("gemini-3.1-pro"));
}

#[test]
fn key_status_badge_distinguishes_local_and_api() {
    let local = ProviderConfig::default();
    assert_eq!(key_status_badge(&local), "local");

    let api_with_key = ProviderConfig {
        kind: ProviderKind::Gemini,
        base_url: ProviderKind::Gemini.default_base_url().to_string(),
        api_key: "secret-test-key".to_string(),
        default_model: String::new(),
    };
    assert_eq!(key_status_badge(&api_with_key), "SET");
}

#[test]
fn attempt_passed_both_requires_two_passes() {
    let a = Attempt {
        id: "a1".to_string(),
        round: 1,
        prover: "P1".to_string(),
        direction_id: "d1".to_string(),
        method: "m".to_string(),
        summary: "s".to_string(),
        draft_markdown: "d".to_string(),
        single_verdict: Verdict::Pass,
        single_objection: String::new(),
        comp_verdict: Verdict::Fail,
        comp_objection: "gap".to_string(),
    };
    assert!(!attempt_passed_both(&a));
}

#[test]
fn all_seven_help_tabs_have_content() {
    assert_eq!(HelpTab::ALL.len(), 7);
    for tab in HelpTab::ALL {
        assert!(!tab.title().is_empty());
        assert!(tab.body().len() > 40);
    }
}

#[test]
fn advisor_note_and_ledger_entry_persist() {
    let store = ProofStore::open_in_memory().expect("db");
    let p = store.create_problem("P", "D", "S", 100).expect("problem");
    store
        .add_ledger_entry(
            &p.id,
            &LedgerEntry {
                id: "l1".to_string(),
                kind: LedgerKind::VerifiedLemma,
                title: "L1".to_string(),
                statement: "x >= 0".to_string(),
                proof_sketch: "square".to_string(),
                isolated_verified: true,
                source_attempt: "a1".to_string(),
                lean_code: String::new(),
                lean_verified: false,
            },
        )
        .expect("ledger");
    store
        .add_advisor_note(
            &p.id,
            &AdvisorNote {
                round: 1,
                observation: "obs".to_string(),
                instruction_delta: "check bounds".to_string(),
            },
        )
        .expect("advisor");
    let detail = store.get_problem_detail(&p.id).expect("detail");
    assert_eq!(detail.ledger.len(), 1);
    assert_eq!(detail.advisor_notes.len(), 1);
}

#[test]
fn arxiv_id_normalization_and_atom_parse() {
    use crate::literature::{normalize_arxiv_id, parse_arxiv_atom};

    assert_eq!(
        normalize_arxiv_id("https://arxiv.org/abs/2609.40324v2"),
        "2609.40324v2"
    );
    assert_eq!(normalize_arxiv_id("arxiv:1706.03762"), "1706.03762");

    let sample_xml = r#"<feed><entry>
        <title>Cogentic: Multi-Agent   Proof Discovery</title>
        <summary>Iterative prove-verify loop.</summary>
    </entry></feed>"#;
    let parsed = parse_arxiv_atom(sample_xml, "2609.40324").expect("parse");
    assert_eq!(parsed.title, "Cogentic: Multi-Agent Proof Discovery");
    assert_eq!(parsed.summary, "Iterative prove-verify loop.");
    assert_eq!(parsed.url, "https://arxiv.org/abs/2609.40324");
}

#[test]
fn literature_persists_on_problem_and_briefing() {
    use crate::literature::LiteratureRef;

    let store = ProofStore::open_in_memory().expect("db");
    let p = store.create_problem("P", "D", "S", 100).expect("p");
    store
        .add_literature(
            &p.id,
            &LiteratureRef {
                id: "lit-2609".to_string(),
                arxiv_id: "2609.40324".to_string(),
                title: "Cogentic".to_string(),
                summary: "Prove-verify loop".to_string(),
                url: "https://arxiv.org/abs/2609.40324".to_string(),
            },
        )
        .expect("add lit");
    let detail = store.get_problem_detail(&p.id).expect("detail");
    assert_eq!(detail.literature.len(), 1);
    let brief = build_prover_briefing(&detail, 1);
    assert!(brief.contains("Grounded Literature References"));
    assert!(brief.contains("2609.40324"));
}

#[test]
fn lean_check_returns_structured_result() {
    use crate::lean::{check_lean_snippet, extract_lean_block, lean_status};

    let st = lean_status();
    assert!(
        st.available,
        "Lean 4 should be available via PATH or ~/.elan/bin"
    );

    let ok = check_lean_snippet("theorem t_ok : 1 + 1 = 2 := rfl");
    assert!(ok.available);
    assert!(
        ok.passed,
        "expected valid Lean 4 theorem to pass: {}",
        ok.stderr
    );
    assert!(!ok.sorry_detected);

    let bad = check_lean_snippet("theorem t_bad : 1 + 1 = 3 := rfl");
    assert!(bad.available);
    assert!(!bad.passed);

    let cheater = check_lean_snippet("theorem t_sorry : 1 + 1 = 3 := by sorry");
    assert!(cheater.available);
    assert!(cheater.sorry_detected);
    assert!(
        !cheater.passed,
        "sorry must be rejected even if lean exits 0"
    );

    let extracted =
        extract_lean_block("Here is the code:\n```lean\ntheorem x : True := trivial\n```");
    assert_eq!(extracted, "theorem x : True := trivial");
}

#[test]
fn dispatch_add_direction_and_export_manuscript() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let dir_val = dispatch(
        &store,
        &ProviderConfig::default(),
        &RoleRouting::default(),
        &BudgetConfig::default(),
        Command::AddDirection {
            problem_id: "prob-anytime-experts".to_string(),
            name: "Potential Function".to_string(),
            claim: "Use hyperbolic cosine potential.".to_string(),
        },
    )
    .expect("add dir");
    assert_eq!(dir_val["name"], "Potential Function");

    let exp = dispatch(
        &store,
        &ProviderConfig::default(),
        &RoleRouting::default(),
        &BudgetConfig::default(),
        Command::ExportManuscript {
            problem_id: "prob-anytime-experts".to_string(),
            format: "tex".to_string(),
        },
    )
    .expect("export");
    assert_eq!(exp["format"], "tex");
    assert!(exp["manuscript"]
        .as_str()
        .unwrap()
        .contains("\\documentclass{article}"));
}

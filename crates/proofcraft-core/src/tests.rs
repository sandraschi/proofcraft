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
        ..BudgetConfig::default()
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
        ..BudgetConfig::default()
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
fn lean_sorry_detector_ignores_comments_and_catches_code() {
    use crate::lean::contains_sorry_or_admit;

    assert!(!contains_sorry_or_admit(
        "-- sorry this is just a comment\ntheorem a : True := trivial"
    ));
    assert!(contains_sorry_or_admit("theorem a : False := by sorry"));
    assert!(contains_sorry_or_admit("theorem b : False := by admit"));
    assert!(!contains_sorry_or_admit(
        "theorem sorry_helper : True := trivial"
    ));
}

#[test]
fn lean_extract_block_handles_generic_fence_and_plain() {
    use crate::lean::extract_lean_block;

    assert_eq!(
        extract_lean_block("```\ntheorem a : True := trivial\n```"),
        "theorem a : True := trivial"
    );
    assert_eq!(
        extract_lean_block("  theorem b : True := trivial  "),
        "theorem b : True := trivial"
    );
}

#[test]
fn lean_check_in_project_falls_back_when_no_lakefile() {
    use crate::lean::check_lean_in_project;

    let res = check_lean_in_project(
        "theorem t : 2 * 3 = 6 := rfl",
        Some(std::path::Path::new(".")),
    );
    assert!(res.available);
    assert!(res.passed);
}

#[test]
fn update_ledger_lean_persists_and_rejects_missing_id() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    store
        .update_ledger_lean(
            "prob-anytime-experts",
            "lem-active-pool",
            "theorem pool : 1 = 1 := rfl",
            true,
        )
        .expect("update");
    let ledger = store.list_ledger("prob-anytime-experts").expect("list");
    let item = ledger
        .iter()
        .find(|l| l.id == "lem-active-pool")
        .expect("item");
    assert!(item.lean_verified);
    assert_eq!(item.lean_code, "theorem pool : 1 = 1 := rfl");

    let err = store
        .update_ledger_lean("prob-anytime-experts", "does-not-exist", "", false)
        .unwrap_err();
    assert!(matches!(err, ProofError::NotFound(_)));
}

#[test]
fn file_backed_sqlite_persists_across_reopen() {
    let tmp = std::env::temp_dir().join(format!("proofcraft_test_{}.sqlite", uuid::Uuid::new_v4()));
    {
        let store = ProofStore::open(&tmp).expect("open");
        let p = store
            .create_problem("Disk Persistence", "Algebra", "Prove x = x.", 42)
            .expect("create");
        assert!(!p.id.is_empty());
    }
    {
        let store2 = ProofStore::open(&tmp).expect("reopen");
        let list = store2.list_problems().expect("list");
        assert!(list.iter().any(|p| p.title == "Disk Persistence"));
    }
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn provider_kind_defaults_and_env_names() {
    assert_eq!(
        ProviderKind::Ollama.default_base_url(),
        "http://127.0.0.1:11434"
    );
    assert_eq!(
        ProviderKind::LmStudio.default_base_url(),
        "http://127.0.0.1:1234/v1"
    );
    assert_eq!(ProviderKind::Gemini.env_var_name(), "GEMINI_API_KEY");
    assert_eq!(ProviderKind::Anthropic.env_var_name(), "ANTHROPIC_API_KEY");
    assert_eq!(ProviderKind::OpenAiCompat.env_var_name(), "OPENAI_API_KEY");
}

#[test]
fn complete_without_model_returns_provider_error() {
    let err = crate::llm::complete(&ProviderConfig::default(), "", "sys", "usr").unwrap_err();
    assert!(matches!(err, ProofError::Provider(_)));
}

#[test]
fn arxiv_parser_rejects_missing_entry_and_normalizes_pdf_url() {
    use crate::literature::{normalize_arxiv_id, parse_arxiv_atom};

    assert_eq!(
        normalize_arxiv_id("https://arxiv.org/pdf/2609.40324.pdf"),
        "2609.40324"
    );
    let err = parse_arxiv_atom("<feed></feed>", "2609.40324").unwrap_err();
    assert!(matches!(err, ProofError::NotFound(_)));
}

#[test]
fn problem_status_and_verdict_lossy_parsers() {
    use crate::model::{DirectionStatus, ProblemStatus};

    assert_eq!(
        ProblemStatus::from_str_lossy("Verified"),
        ProblemStatus::Verified
    );
    assert_eq!(
        ProblemStatus::from_str_lossy("BudgetExhausted"),
        ProblemStatus::BudgetExhausted
    );
    assert_eq!(
        ProblemStatus::from_str_lossy("unknown"),
        ProblemStatus::Queued
    );

    assert_eq!(
        DirectionStatus::from_str_lossy("Proved"),
        DirectionStatus::Proved
    );
    assert_eq!(
        DirectionStatus::from_str_lossy("Excluded"),
        DirectionStatus::Excluded
    );
    assert_eq!(
        DirectionStatus::from_str_lossy("other"),
        DirectionStatus::Active
    );

    assert_eq!(Verdict::from_str_lossy("accepted"), Verdict::Pass);
    assert_eq!(Verdict::from_str_lossy("REJECT"), Verdict::Fail);
    assert_eq!(Verdict::from_str_lossy("maybe"), Verdict::Pending);
}

#[test]
fn consolidate_manuscript_falls_back_deterministically_without_llm() {
    let store = ProofStore::open_in_memory().expect("db");
    store.load_embedded_seed().expect("seed");
    let res = dispatch(
        &store,
        &ProviderConfig::default(),
        &RoleRouting::default(),
        &BudgetConfig::default(),
        Command::ConsolidateManuscript {
            problem_id: "prob-anytime-experts".to_string(),
            format: "md".to_string(),
        },
    )
    .expect("consolidate");
    assert_eq!(res["mode"], "deterministic_export");
    assert!(res["manuscript"]
        .as_str()
        .unwrap()
        .contains("Anytime Regret"));
}

#[test]
fn get_missing_problem_returns_not_found() {
    let store = ProofStore::open_in_memory().expect("db");
    let err = store.get_problem_detail("nonexistent-id").unwrap_err();
    assert!(matches!(err, ProofError::NotFound(_)));
}

#[test]
fn helper_formatters_produce_expected_output() {
    use crate::model::{Direction, DirectionStatus, Problem};
    use crate::{direction_label, format_advisor_line, ledger_badge, problem_summary_line};

    let p = Problem {
        id: "p1".to_string(),
        title: "Test".to_string(),
        domain: "Math".to_string(),
        statement: "S".to_string(),
        status: ProblemStatus::InProgress,
        max_calls: 100,
        calls_used: 12,
        rounds_completed: 2,
    };
    assert!(problem_summary_line(&p).contains("12 calls / 100 max"));

    let d = Direction {
        id: "d1".to_string(),
        name: "Dir".to_string(),
        claim: "C".to_string(),
        status: DirectionStatus::Proved,
    };
    assert_eq!(direction_label(&d), "Dir [Proved]");

    let n = AdvisorNote {
        round: 2,
        observation: "obs".to_string(),
        instruction_delta: "delta".to_string(),
    };
    assert_eq!(format_advisor_line(&n), "R2: obs -> delta");

    let l = LedgerEntry {
        id: "l1".to_string(),
        kind: LedgerKind::DeadEnd,
        title: "D".to_string(),
        statement: "S".to_string(),
        proof_sketch: "P".to_string(),
        isolated_verified: false,
        source_attempt: "a".to_string(),
        lean_code: String::new(),
        lean_verified: false,
    };
    assert_eq!(ledger_badge(&l), "DEAD END");
}

#[test]
fn dispatch_clear_and_load_demo_commands() {
    let store = ProofStore::open_in_memory().expect("db");
    let p = &ProviderConfig::default();
    let r = &RoleRouting::default();
    let b = &BudgetConfig::default();
    let loaded = dispatch(&store, p, r, b, Command::LoadDemoData).expect("load");
    assert_eq!(loaded["loaded_problems"], 2);
    let cleared = dispatch(&store, p, r, b, Command::ClearDemoData).expect("clear");
    assert_eq!(cleared["cleared"], true);
}

#[test]
fn dispatch_lean_status_command_succeeds() {
    let store = ProofStore::open_in_memory().expect("db");
    let res = dispatch(
        &store,
        &ProviderConfig::default(),
        &RoleRouting::default(),
        &BudgetConfig::default(),
        Command::LeanStatus,
    )
    .expect("lean status");
    assert_eq!(res["available"], true);
}

#[test]
fn budget_config_and_role_routing_defaults() {
    let b = BudgetConfig::default();
    assert_eq!(b.max_calls_per_run, 200);
    assert_eq!(b.max_rounds_per_run, 12);
    assert!(!b.auto_formalize_lemmas);
    assert!(b.lake_project_dir.is_empty());

    let r = RoleRouting::default();
    assert!(r.prover_model.is_empty());
    assert!(r.verifier_model.is_empty());
}

#[test]
fn ledger_kind_round_trip_and_lossy() {
    assert_eq!(LedgerKind::from_str_lossy("DeadEnd"), LedgerKind::DeadEnd);
    assert_eq!(
        LedgerKind::from_str_lossy("VerifiedLemma"),
        LedgerKind::VerifiedLemma
    );
    assert_eq!(
        LedgerKind::from_str_lossy("other"),
        LedgerKind::VerifiedLemma
    );
}

#[test]
fn help_tabs_have_distinct_titles() {
    let mut titles: Vec<&str> = HelpTab::ALL.iter().map(HelpTab::title).collect();
    titles.sort_unstable();
    titles.dedup();
    assert_eq!(titles.len(), 7);
}

#[test]
fn provider_default_has_empty_model_for_live_discovery() {
    let p = ProviderConfig::default();
    assert!(p.default_model.is_empty());
    assert_eq!(p.kind, ProviderKind::Ollama);
}

#[test]
fn bundled_skill_md_contains_frontmatter() {
    assert!(crate::help::BUNDLED_SKILL_MD.starts_with("---"));
    assert!(crate::help::BUNDLED_SKILL_MD.contains("name: proofcraft"));
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

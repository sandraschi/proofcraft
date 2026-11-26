use crate::literature::{extract_xml_tag, fetch_arxiv_metadata, normalize_arxiv_id};
use crate::model::{Problem, SeedBundle};
use crate::store::ProofStore;
use crate::{ProofError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProblemPack {
    LeanWarmup,
    ErdosCombinatorics,
    CogenticPaper,
}

impl ProblemPack {
    pub const ALL: [Self; 3] = [
        Self::LeanWarmup,
        Self::ErdosCombinatorics,
        Self::CogenticPaper,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::LeanWarmup => "lean_warmup",
            Self::ErdosCombinatorics => "erdos_combinatorics",
            Self::CogenticPaper => "cogentic_paper",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::LeanWarmup => "Undergrad Lean 4 Warmup Pack (3 problems)",
            Self::ErdosCombinatorics => "Erdos Combinatorics Open Pack (2 problems)",
            Self::CogenticPaper => "Cogentic Paper Demo Pack (2 problems)",
        }
    }

    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "lean_warmup" | "warmup" => Some(Self::LeanWarmup),
            "erdos_combinatorics" | "erdos" => Some(Self::ErdosCombinatorics),
            "cogentic_paper" | "cogentic" | "demo" => Some(Self::CogenticPaper),
            _ => None,
        }
    }
}

pub fn load_pack(store: &ProofStore, pack: ProblemPack) -> Result<usize> {
    match pack {
        ProblemPack::CogenticPaper => store.load_embedded_seed(),
        ProblemPack::LeanWarmup => {
            let mut count = 0usize;
            let items = [
                (
                    "Warmup: Sum of First n Odds is n^2",
                    "Number Theory / Induction",
                    "Prove by induction on n : Nat that sum_{i=0}^{n-1} (2*i + 1) = n^2, and formalize it in Lean 4 without sorry.",
                    60u32,
                    "Induction",
                    "Base case n=0 is 0=0; step adds 2n+1 to n^2 giving (n+1)^2.",
                ),
                (
                    "Warmup: AM-GM for Two Non-Negative Reals / Ints",
                    "Inequalities",
                    "Show that 2*a*b <= a^2 + b^2 for all integers a, b, and extend to (a+b)^2 >= 4*a*b.",
                    60u32,
                    "Square Non-Negativity",
                    "Expand (a - b)^2 >= 0.",
                ),
                (
                    "Warmup: Pigeonhole Principle on Finite Maps",
                    "Combinatorics",
                    "Prove that any function f : Fin (n + 1) -> Fin n has two distinct inputs with equal outputs.",
                    80u32,
                    "Counting / Induction",
                    "Induct on n; split on whether n is in the image.",
                ),
            ];
            for (title, domain, stmt, cap, dir_name, dir_claim) in items {
                let p = store.create_problem(title, domain, stmt, cap)?;
                let _ = store.add_direction(&p.id, dir_name, dir_claim);
                count += 1;
            }
            Ok(count)
        }
        ProblemPack::ErdosCombinatorics => {
            let mut count = 0usize;
            let items = [
                (
                    "Erdos-Style: Sidon Set Density Upper Bound",
                    "Additive Combinatorics",
                    "Bound the maximum size of a Sidon set A subset {1..N} (all pairwise sums a+b distinct) as |A| <= sqrt(N) + O(N^{1/4}).",
                    150u32,
                    "Pairwise Difference Counting",
                    "Count distinct nonzero differences a - b in {1..N-1}.",
                ),
                (
                    "Erdos-Style: Unit Distances in Convex Position",
                    "Discrete Geometry",
                    "Show that n points in strictly convex position in R^2 determine O(n log n) unit-distance pairs (Furedi 1990 bound).",
                    180u32,
                    "Forbidden Davenport-Schinzel Submatrix",
                    "Order vertices cyclically and forbid alternating 2x2 unit-distance patterns.",
                ),
            ];
            for (title, domain, stmt, cap, dir_name, dir_claim) in items {
                let p = store.create_problem(title, domain, stmt, cap)?;
                let _ = store.add_direction(&p.id, dir_name, dir_claim);
                count += 1;
            }
            Ok(count)
        }
    }
}

pub fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len().min(4096));
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn import_problem_from_url(store: &ProofStore, input: &str) -> Result<Problem> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ProofError::InvalidInput(
            "URL or ID must not be empty".to_string(),
        ));
    }

    // 1. arXiv URL or ID
    if trimmed.contains("arxiv.org") || trimmed.starts_with("arxiv:") || is_bare_arxiv_id(trimmed) {
        let lit = fetch_arxiv_metadata(trimmed)?;
        let p = store.create_problem(
            &lit.title,
            "arXiv Research",
            &format!("Source: {}\n\n{}", lit.url, lit.summary),
            200,
        )?;
        let _ = store.add_literature(&p.id, &lit);
        let _ = store.add_direction(
            &p.id,
            "Primary Conjecture",
            "Formalize main theorem statement from attached arXiv abstract.",
        );
        return Ok(p);
    }

    // 2. General HTTP(S) URL (erdosproblems.com, openproblemgarden.org, raw .lean / .md)
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(ProofError::InvalidInput(
            "expected https:// URL or arXiv ID (e.g. 2609.40324)".to_string(),
        ));
    }

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(15))
        .build();
    let resp = agent
        .get(trimmed)
        .set("User-Agent", "proofcraft/0.1.0 (problem-importer)")
        .call()
        .map_err(|e| ProofError::Provider(format!("fetch failed for {trimmed}: {e}")))?;
    let body = resp
        .into_string()
        .map_err(|e| ProofError::Provider(format!("body read failed: {e}")))?;

    let (title, domain, statement) = parse_web_problem_page(trimmed, &body);
    let p = store.create_problem(&title, &domain, &statement, 180)?;
    let _ = store.add_direction(&p.id, "Initial Exploration", "Extract key lemma targets.");
    Ok(p)
}

pub fn parse_web_problem_page(url: &str, body: &str) -> (String, String, String) {
    let domain = if url.contains("erdosproblems.com") {
        "Erdos Problems"
    } else if url.contains("openproblemgarden.org") {
        "Open Problem Garden"
    } else if url.ends_with(".lean") {
        "Lean 4 Benchmark"
    } else {
        "Web Import"
    }
    .to_string();

    if url.ends_with(".lean") || url.ends_with(".md") || !body.contains('<') {
        let first_line = body
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("Imported Problem")
            .trim_start_matches(['#', '-', ' '])
            .chars()
            .take(90)
            .collect::<String>();
        let snippet: String = body.chars().take(1800).collect();
        return (first_line, domain, format!("Source: {url}\n\n{snippet}"));
    }

    let title = extract_xml_tag(body, "title")
        .unwrap_or_else(|| format!("Imported from {domain}"))
        .chars()
        .take(100)
        .collect::<String>();
    let text = strip_html_tags(body);
    let excerpt: String = text.chars().take(1500).collect();
    (title, domain, format!("Source: {url}\n\n{excerpt}"))
}

fn is_bare_arxiv_id(s: &str) -> bool {
    let norm = normalize_arxiv_id(s);
    let mut parts = norm.split('.');
    let Some(a) = parts.next() else { return false };
    let Some(b) = parts.next() else { return false };
    a.len() == 4
        && a.chars().all(|c| c.is_ascii_digit())
        && b.len() >= 4
        && b.chars().take(4).all(|c| c.is_ascii_digit())
}

pub fn _ensure_seed_bundle_used(_: SeedBundle) {}

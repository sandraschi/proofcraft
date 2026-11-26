use crate::literature::LiteratureRef;
use crate::model::{
    AdvisorNote, Attempt, Direction, DirectionStatus, LedgerEntry, LedgerKind, Problem,
    ProblemDetail, ProblemStatus, SeedBundle, Verdict,
};
use crate::{ProofError, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

const EMBEDDED_SEED_JSON: &str = include_str!("../../../seed/demo_data.json");

pub struct ProofStore {
    conn: Mutex<Connection>,
}

impl ProofStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;",
        )?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_schema()?;
        if store.list_problems()?.is_empty() {
            let _ = store.load_embedded_seed()?;
        }
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_schema()?;
        Ok(store)
    }

    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| ProofError::InvalidInput("database mutex poisoned".to_string()))
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS problems (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                domain TEXT NOT NULL,
                statement TEXT NOT NULL,
                status TEXT NOT NULL,
                max_calls INTEGER NOT NULL,
                calls_used INTEGER NOT NULL,
                rounds_completed INTEGER NOT NULL,
                is_demo INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS directions (
                id TEXT PRIMARY KEY,
                problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                claim TEXT NOT NULL,
                status TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS attempts (
                id TEXT PRIMARY KEY,
                problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
                round INTEGER NOT NULL,
                prover TEXT NOT NULL,
                direction_id TEXT NOT NULL,
                method TEXT NOT NULL,
                summary TEXT NOT NULL,
                draft_markdown TEXT NOT NULL,
                single_verdict TEXT NOT NULL,
                single_objection TEXT NOT NULL,
                comp_verdict TEXT NOT NULL,
                comp_objection TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS ledger (
                id TEXT PRIMARY KEY,
                problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                statement TEXT NOT NULL,
                proof_sketch TEXT NOT NULL,
                isolated_verified INTEGER NOT NULL,
                source_attempt TEXT NOT NULL,
                lean_code TEXT NOT NULL DEFAULT '',
                lean_verified INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS advisor_notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
                round INTEGER NOT NULL,
                observation TEXT NOT NULL,
                instruction_delta TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS literature (
                id TEXT PRIMARY KEY,
                problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
                arxiv_id TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                url TEXT NOT NULL
            );",
        )?;
        // Safe migration for pre-existing databases lacking Lean columns
        let _ = conn.execute(
            "ALTER TABLE ledger ADD COLUMN lean_code TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE ledger ADD COLUMN lean_verified INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE problems ADD COLUMN is_demo INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "UPDATE problems SET is_demo = 1 WHERE id IN ('prob-anytime-experts', 'prob-inverse-linear-opt')",
            [],
        );
        Ok(())
    }

    pub fn load_embedded_seed(&self) -> Result<usize> {
        let bundle: SeedBundle = serde_json::from_str(EMBEDDED_SEED_JSON)?;
        let conn = self.lock_conn()?;
        let mut count = 0usize;
        for sp in bundle.problems {
            conn.execute(
                "INSERT OR IGNORE INTO problems
                 (id, title, domain, statement, status, max_calls, calls_used, rounds_completed, is_demo)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    sp.id,
                    sp.title,
                    sp.domain,
                    sp.statement,
                    sp.status.as_str(),
                    sp.max_calls,
                    sp.calls_used,
                    sp.rounds_completed,
                    if sp.is_demo { 1 } else { 0 }
                ],
            )?;
            for d in sp.directions {
                conn.execute(
                    "INSERT OR IGNORE INTO directions (id, problem_id, name, claim, status)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![d.id, sp.id, d.name, d.claim, d.status.as_str()],
                )?;
            }
            for a in sp.attempts {
                conn.execute(
                    "INSERT OR IGNORE INTO attempts
                     (id, problem_id, round, prover, direction_id, method, summary, draft_markdown,
                      single_verdict, single_objection, comp_verdict, comp_objection)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        a.id,
                        sp.id,
                        a.round,
                        a.prover,
                        a.direction_id,
                        a.method,
                        a.summary,
                        a.draft_markdown,
                        a.single_verdict.as_str(),
                        a.single_objection,
                        a.comp_verdict.as_str(),
                        a.comp_objection
                    ],
                )?;
            }
            for l in sp.ledger {
                conn.execute(
                    "INSERT OR IGNORE INTO ledger
                     (id, problem_id, kind, title, statement, proof_sketch, isolated_verified, source_attempt, lean_code, lean_verified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        l.id,
                        sp.id,
                        l.kind.as_str(),
                        l.title,
                        l.statement,
                        l.proof_sketch,
                        if l.isolated_verified { 1 } else { 0 },
                        l.source_attempt,
                        l.lean_code,
                        if l.lean_verified { 1 } else { 0 }
                    ],
                )?;
                if !l.lean_code.is_empty() {
                    let _ = conn.execute(
                        "UPDATE ledger SET lean_code = ?2, lean_verified = ?3
                         WHERE id = ?1 AND lean_code = ''",
                        params![l.id, l.lean_code, if l.lean_verified { 1 } else { 0 }],
                    );
                }
            }
            for n in sp.advisor_notes {
                conn.execute(
                    "INSERT INTO advisor_notes (problem_id, round, observation, instruction_delta)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![sp.id, n.round, n.observation, n.instruction_delta],
                )?;
            }
            count += 1;
        }
        Ok(count)
    }

    pub fn clear_all(&self) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "DELETE FROM literature;
             DELETE FROM advisor_notes;
             DELETE FROM ledger;
             DELETE FROM attempts;
             DELETE FROM directions;
             DELETE FROM problems;",
        )?;
        Ok(())
    }

    pub fn list_problems(&self) -> Result<Vec<Problem>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, domain, statement, status, max_calls, calls_used, rounds_completed, is_demo
             FROM problems ORDER BY rowid ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(4)?;
            let demo_int: i64 = row.get(8)?;
            Ok(Problem {
                id: row.get(0)?,
                title: row.get(1)?,
                domain: row.get(2)?,
                statement: row.get(3)?,
                status: ProblemStatus::from_str_lossy(&status_str),
                max_calls: row.get(5)?,
                calls_used: row.get(6)?,
                rounds_completed: row.get(7)?,
                is_demo: demo_int != 0,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn create_problem(
        &self,
        title: &str,
        domain: &str,
        statement: &str,
        max_calls: u32,
    ) -> Result<Problem> {
        let id = format!("prob-{}", &Uuid::new_v4().to_string()[..8]);
        let problem = Problem {
            id: id.clone(),
            title: title.trim().to_string(),
            domain: if domain.trim().is_empty() {
                "General Math".to_string()
            } else {
                domain.trim().to_string()
            },
            statement: statement.trim().to_string(),
            status: ProblemStatus::Queued,
            max_calls: max_calls.max(10),
            calls_used: 0,
            rounds_completed: 0,
            is_demo: false,
        };
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO problems (id, title, domain, statement, status, max_calls, calls_used, rounds_completed, is_demo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
            params![
                problem.id,
                problem.title,
                problem.domain,
                problem.statement,
                problem.status.as_str(),
                problem.max_calls,
                problem.calls_used,
                problem.rounds_completed
            ],
        )?;
        Ok(problem)
    }

    pub fn add_direction(&self, problem_id: &str, name: &str, claim: &str) -> Result<Direction> {
        let id = format!("dir-{}", &Uuid::new_v4().to_string()[..8]);
        let dir = Direction {
            id: id.clone(),
            name: name.trim().to_string(),
            claim: claim.trim().to_string(),
            status: DirectionStatus::Active,
        };
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "INSERT INTO directions (id, problem_id, name, claim, status)
             SELECT ?1, id, ?3, ?4, ?5 FROM problems WHERE id = ?2",
            params![dir.id, problem_id, dir.name, dir.claim, dir.status.as_str()],
        )?;
        if affected == 0 {
            return Err(ProofError::NotFound(format!("problem {problem_id}")));
        }
        Ok(dir)
    }

    pub fn record_attempt(&self, problem_id: &str, attempt: &Attempt) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO attempts
             (id, problem_id, round, prover, direction_id, method, summary, draft_markdown,
              single_verdict, single_objection, comp_verdict, comp_objection)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                attempt.id,
                problem_id,
                attempt.round,
                attempt.prover,
                attempt.direction_id,
                attempt.method,
                attempt.summary,
                attempt.draft_markdown,
                attempt.single_verdict.as_str(),
                attempt.single_objection,
                attempt.comp_verdict.as_str(),
                attempt.comp_objection
            ],
        )?;
        Ok(())
    }

    pub fn add_ledger_entry(&self, problem_id: &str, entry: &LedgerEntry) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO ledger
             (id, problem_id, kind, title, statement, proof_sketch, isolated_verified, source_attempt, lean_code, lean_verified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.id,
                problem_id,
                entry.kind.as_str(),
                entry.title,
                entry.statement,
                entry.proof_sketch,
                if entry.isolated_verified { 1 } else { 0 },
                entry.source_attempt,
                entry.lean_code,
                if entry.lean_verified { 1 } else { 0 }
            ],
        )?;
        Ok(())
    }

    pub fn update_ledger_lean(
        &self,
        problem_id: &str,
        ledger_id: &str,
        lean_code: &str,
        lean_verified: bool,
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "UPDATE ledger SET lean_code = ?3, lean_verified = ?4
             WHERE problem_id = ?1 AND id = ?2",
            params![
                problem_id,
                ledger_id,
                lean_code,
                if lean_verified { 1 } else { 0 }
            ],
        )?;
        if affected == 0 {
            return Err(ProofError::NotFound(format!("ledger entry {ledger_id}")));
        }
        Ok(())
    }

    pub fn add_advisor_note(&self, problem_id: &str, note: &AdvisorNote) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO advisor_notes (problem_id, round, observation, instruction_delta)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                problem_id,
                note.round,
                note.observation,
                note.instruction_delta
            ],
        )?;
        Ok(())
    }

    pub fn add_literature(&self, problem_id: &str, lit: &LiteratureRef) -> Result<()> {
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "INSERT OR REPLACE INTO literature (id, problem_id, arxiv_id, title, summary, url)
             SELECT ?1, id, ?3, ?4, ?5, ?6 FROM problems WHERE id = ?2",
            params![
                lit.id,
                problem_id,
                lit.arxiv_id,
                lit.title,
                lit.summary,
                lit.url
            ],
        )?;
        if affected == 0 {
            return Err(ProofError::NotFound(format!("problem {problem_id}")));
        }
        Ok(())
    }

    pub fn update_problem_progress(
        &self,
        problem_id: &str,
        status: ProblemStatus,
        calls_delta: u32,
        rounds_completed: u32,
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE problems
             SET status = ?2, calls_used = calls_used + ?3, rounds_completed = ?4
             WHERE id = ?1",
            params![problem_id, status.as_str(), calls_delta, rounds_completed],
        )?;
        Ok(())
    }

    pub fn list_ledger(&self, problem_id: &str) -> Result<Vec<LedgerEntry>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, kind, title, statement, proof_sketch, isolated_verified, source_attempt, lean_code, lean_verified
             FROM ledger WHERE problem_id = ?1 ORDER BY rowid ASC",
        )?;
        let rows = stmt.query_map(params![problem_id], |row| {
            let kind_str: String = row.get(1)?;
            let iso: i64 = row.get(5)?;
            let lv: i64 = row.get(8)?;
            Ok(LedgerEntry {
                id: row.get(0)?,
                kind: LedgerKind::from_str_lossy(&kind_str),
                title: row.get(2)?,
                statement: row.get(3)?,
                proof_sketch: row.get(4)?,
                isolated_verified: iso != 0,
                source_attempt: row.get(6)?,
                lean_code: row.get(7)?,
                lean_verified: lv != 0,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_problem_detail(&self, problem_id: &str) -> Result<ProblemDetail> {
        let conn = self.lock_conn()?;
        let problem = conn
            .query_row(
                "SELECT id, title, domain, statement, status, max_calls, calls_used, rounds_completed, is_demo
                 FROM problems WHERE id = ?1",
                params![problem_id],
                |row| {
                    let status_str: String = row.get(4)?;
                    let demo_int: i64 = row.get(8)?;
                    Ok(Problem {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        domain: row.get(2)?,
                        statement: row.get(3)?,
                        status: ProblemStatus::from_str_lossy(&status_str),
                        max_calls: row.get(5)?,
                        calls_used: row.get(6)?,
                        rounds_completed: row.get(7)?,
                        is_demo: demo_int != 0,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ProofError::NotFound(format!("problem {problem_id}"))
                }
                other => ProofError::Db(other),
            })?;

        let mut dir_stmt = conn.prepare(
            "SELECT id, name, claim, status FROM directions WHERE problem_id = ?1 ORDER BY rowid ASC",
        )?;
        let directions = dir_stmt
            .query_map(params![problem_id], |row| {
                let s: String = row.get(3)?;
                Ok(Direction {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    claim: row.get(2)?,
                    status: DirectionStatus::from_str_lossy(&s),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut att_stmt = conn.prepare(
            "SELECT id, round, prover, direction_id, method, summary, draft_markdown,
                    single_verdict, single_objection, comp_verdict, comp_objection
             FROM attempts WHERE problem_id = ?1 ORDER BY round ASC, rowid ASC",
        )?;
        let attempts = att_stmt
            .query_map(params![problem_id], |row| {
                let sv: String = row.get(7)?;
                let cv: String = row.get(9)?;
                Ok(Attempt {
                    id: row.get(0)?,
                    round: row.get(1)?,
                    prover: row.get(2)?,
                    direction_id: row.get(3)?,
                    method: row.get(4)?,
                    summary: row.get(5)?,
                    draft_markdown: row.get(6)?,
                    single_verdict: Verdict::from_str_lossy(&sv),
                    single_objection: row.get(8)?,
                    comp_verdict: Verdict::from_str_lossy(&cv),
                    comp_objection: row.get(10)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        drop(dir_stmt);
        drop(att_stmt);
        drop(conn);

        let ledger = self.list_ledger(problem_id)?;
        let conn2 = self.lock_conn()?;
        let mut adv_stmt = conn2.prepare(
            "SELECT round, observation, instruction_delta
             FROM advisor_notes WHERE problem_id = ?1 ORDER BY round ASC, id ASC",
        )?;
        let advisor_notes = adv_stmt
            .query_map(params![problem_id], |row| {
                Ok(AdvisorNote {
                    round: row.get(0)?,
                    observation: row.get(1)?,
                    instruction_delta: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut lit_stmt = conn2.prepare(
            "SELECT id, arxiv_id, title, summary, url
             FROM literature WHERE problem_id = ?1 ORDER BY rowid ASC",
        )?;
        let literature = lit_stmt
            .query_map(params![problem_id], |row| {
                Ok(LiteratureRef {
                    id: row.get(0)?,
                    arxiv_id: row.get(1)?,
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    url: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(ProblemDetail {
            problem,
            directions,
            attempts,
            ledger,
            advisor_notes,
            literature,
        })
    }

    pub fn export_manuscript(&self, problem_id: &str, format: &str) -> Result<String> {
        let detail = self.get_problem_detail(problem_id)?;
        let is_tex = format.eq_ignore_ascii_case("tex") || format.eq_ignore_ascii_case("latex");
        let verified_attempts: Vec<&Attempt> = detail
            .attempts
            .iter()
            .filter(|a| a.single_verdict == Verdict::Pass && a.comp_verdict == Verdict::Pass)
            .collect();

        let mut out = String::new();
        if is_tex {
            out.push_str("\\documentclass{article}\n\\usepackage{amsmath,amssymb,amsthm}\n\\begin{document}\n");
            out.push_str(&format!(
                "\\title{{{}}}\n\\maketitle\n",
                detail.problem.title
            ));
            out.push_str(&format!(
                "\\section{{Problem Statement}}\n{}\n\n",
                detail.problem.statement
            ));
            out.push_str("\\section{Verified Intermediate Lemmas}\n");
            for l in detail
                .ledger
                .iter()
                .filter(|e| e.kind == LedgerKind::VerifiedLemma)
            {
                out.push_str(&format!(
                    "\\subsection*{{{}}}\n{}\n\\begin{{proof}}\n{}\n\\end{{proof}}\n\n",
                    l.title, l.statement, l.proof_sketch
                ));
            }
            out.push_str("\\section{Accepted Proof}\n");
            if let Some(best) = verified_attempts.last() {
                out.push_str(&best.draft_markdown);
                out.push('\n');
            } else {
                out.push_str("% No candidate has cleared both adversarial verifiers yet.\n");
            }
            out.push_str("\\end{document}\n");
        } else {
            out.push_str(&format!("# {}\n\n", detail.problem.title));
            out.push_str(&format!(
                "**Domain:** {} | **Status:** {} | **Calls Used:** {}/{}\n\n",
                detail.problem.domain,
                detail.problem.status.as_str(),
                detail.problem.calls_used,
                detail.problem.max_calls
            ));
            out.push_str("## Problem Statement\n\n");
            out.push_str(&detail.problem.statement);
            out.push_str("\n\n## Verified Ledger\n\n");
            for l in &detail.ledger {
                let tag = if l.kind == LedgerKind::VerifiedLemma {
                    "VERIFIED LEMMA"
                } else {
                    "EXCLUDED DEAD END"
                };
                out.push_str(&format!(
                    "### [{}] {}\n{}\n\n*Proof/Justification:* {}\n\n",
                    tag, l.title, l.statement, l.proof_sketch
                ));
            }
            out.push_str("## Accepted Proof\n\n");
            if let Some(best) = verified_attempts.last() {
                out.push_str(&best.draft_markdown);
                out.push('\n');
            } else {
                out.push_str("*No candidate has cleared both adversarial verifiers yet.*\n");
            }
        }
        Ok(out)
    }

    pub fn export_lake_project(&self, problem_id: &str, out_dir: &Path) -> Result<usize> {
        let detail = self.get_problem_detail(problem_id)?;
        std::fs::create_dir_all(out_dir)?;

        let pkg_name: String = detail
            .problem
            .id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();

        let lakefile = format!(
            "name = \"{pkg_name}\"\nversion = \"0.1.0\"\ndefaultTargets = [\"{pkg_name}\"]\n\n[[lean_lib]]\nname = \"VerifiedLemmas\"\n\n[[lean_exe]]\nname = \"{pkg_name}\"\nroot = \"Main\"\n"
        );
        std::fs::write(out_dir.join("lakefile.toml"), lakefile)?;
        std::fs::write(out_dir.join("lean-toolchain"), "leanprover/lean4:v4.36.0\n")?;

        let mut lemmas_src = format!(
            "-- Generated by proofcraft for: {}\n\n",
            detail.problem.title
        );
        let mut count = 0usize;
        for (idx, l) in detail.ledger.iter().enumerate() {
            if l.lean_verified && !l.lean_code.trim().is_empty() {
                lemmas_src.push_str(&format!(
                    "namespace Lemma_{idx}\n-- {}\n{}\nend Lemma_{idx}\n\n",
                    l.title,
                    l.lean_code.trim()
                ));
                count += 1;
            }
        }
        if count == 0 {
            lemmas_src.push_str("theorem placeholder_verified : 1 + 1 = 2 := rfl\n");
        }
        std::fs::write(out_dir.join("VerifiedLemmas.lean"), lemmas_src)?;

        let main_src = format!(
            "import VerifiedLemmas\n\ndef main : IO Unit :=\n  IO.println \"proofcraft Lake export: {} ({count} Lean-verified lemmas)\"\n",
            detail.problem.id
        );
        std::fs::write(out_dir.join("Main.lean"), main_src)?;

        Ok(count)
    }
}

use crate::theme::{Palette, Skin, UiPrefs};
use eframe::egui::{self, Color32, RichText, Stroke};
use proofcraft_core::help::HelpTab;
use proofcraft_core::llm::{discover_models, key_status_badge};
use proofcraft_core::model::{
    BudgetConfig, DiscoveryResult, LedgerKind, ProblemDetail, ProviderConfig, ProviderKind,
    RoleRouting, Verdict,
};
use proofcraft_core::store::ProofStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    Workspace,
    Rounds,
    Ledger,
    Advisor,
    Manuscript,
    Help,
    Settings,
}

impl MainTab {
    pub const ALL: [Self; 7] = [
        Self::Workspace,
        Self::Rounds,
        Self::Ledger,
        Self::Advisor,
        Self::Manuscript,
        Self::Help,
        Self::Settings,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Workspace => "Workspace",
            Self::Rounds => "Rounds & Verifiers",
            Self::Ledger => "Verified Ledger",
            Self::Advisor => "Process Advisor",
            Self::Manuscript => "Manuscript",
            Self::Help => "Help Center",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Budget,
    LlmProviders,
    CoopAndData,
}

pub fn badge(ui: &mut egui::Ui, label: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.linear_multiply(0.18))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(label).small().strong().color(color));
        });
}

pub fn render_workspace(
    ui: &mut egui::Ui,
    pal: &Palette,
    detail: &ProblemDetail,
    new_dir_name: &mut String,
    new_dir_claim: &mut String,
    store: &ProofStore,
    status_msg: &mut String,
) {
    ui.heading(RichText::new(&detail.problem.title).color(pal.text));
    ui.horizontal(|ui| {
        badge(ui, &detail.problem.domain, pal.accent);
        badge(ui, detail.problem.status.as_str(), pal.pass);
        ui.label(
            RichText::new(format!(
                "Rounds: {}  |  Calls: {} / {}",
                detail.problem.rounds_completed,
                detail.problem.calls_used,
                detail.problem.max_calls
            ))
            .color(pal.muted),
        );
    });
    ui.add_space(6.0);

    let frac = (detail.problem.calls_used as f32 / detail.problem.max_calls.max(1) as f32).min(1.0);
    ui.add(egui::ProgressBar::new(frac).text(format!(
        "Budget: {} / {} calls",
        detail.problem.calls_used, detail.problem.max_calls
    )));
    ui.add_space(10.0);

    egui::Frame::new()
        .fill(pal.card)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(6.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new("Problem Statement")
                    .strong()
                    .color(pal.accent),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(&detail.problem.statement).color(pal.text));
        });

    ui.add_space(12.0);
    ui.heading(RichText::new("Active Proof Directions").color(pal.text));
    for d in &detail.directions {
        egui::Frame::new()
            .fill(pal.card)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&d.name).strong().color(pal.text));
                    let col = match d.status {
                        proofcraft_core::model::DirectionStatus::Proved => pal.pass,
                        proofcraft_core::model::DirectionStatus::Excluded => pal.fail,
                        proofcraft_core::model::DirectionStatus::Active => pal.accent,
                    };
                    badge(ui, d.status.as_str(), col);
                });
                ui.label(RichText::new(&d.claim).color(pal.muted));
            });
        ui.add_space(4.0);
    }

    ui.add_space(8.0);
    ui.collapsing("Add New Direction", |ui| {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(new_dir_name);
        });
        ui.horizontal(|ui| {
            ui.label("Claim:");
            ui.text_edit_singleline(new_dir_claim);
        });
        if ui.button("Add Direction").clicked() {
            match store.add_direction(&detail.problem.id, new_dir_name, new_dir_claim) {
                Ok(_) => {
                    *status_msg = "Direction added.".to_string();
                    new_dir_name.clear();
                    new_dir_claim.clear();
                }
                Err(e) => *status_msg = format!("Error: {e}"),
            }
        }
    });
}

pub fn render_rounds(ui: &mut egui::Ui, pal: &Palette, detail: &ProblemDetail) {
    ui.heading(RichText::new("Rounds & Two-Tier Adversarial Verification").color(pal.text));
    ui.label(
        RichText::new("Every candidate draft must pass BOTH the Individual Verifier and the Cross-Draft Comparative Verifier.")
            .color(pal.muted),
    );
    ui.add_space(8.0);

    if detail.attempts.is_empty() {
        ui.label(RichText::new("No proof attempts recorded yet.").color(pal.muted));
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for att in &detail.attempts {
            egui::Frame::new()
                .fill(pal.card)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        badge(ui, &format!("Round {}", att.round), pal.accent);
                        ui.label(RichText::new(&att.prover).strong().color(pal.text));
                        let sv_col = verdict_color(att.single_verdict, pal);
                        badge(
                            ui,
                            &format!("Single: {}", att.single_verdict.as_str()),
                            sv_col,
                        );
                        let cv_col = verdict_color(att.comp_verdict, pal);
                        badge(
                            ui,
                            &format!("Comparative: {}", att.comp_verdict.as_str()),
                            cv_col,
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("Method: {}", att.method)).color(pal.muted));
                    ui.label(RichText::new(&att.summary).color(pal.text));
                    ui.add_space(4.0);
                    ui.collapsing(format!("Verdicts & Draft ({})", att.id), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "Individual Objection: {}",
                                att.single_objection
                            ))
                            .color(pal.muted),
                        );
                        ui.label(
                            RichText::new(format!("Comparative Objection: {}", att.comp_objection))
                                .color(pal.muted),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(&att.draft_markdown)
                                .monospace()
                                .color(pal.text),
                        );
                    });
                });
            ui.add_space(6.0);
        }
    });
}

fn verdict_color(v: Verdict, pal: &Palette) -> Color32 {
    match v {
        Verdict::Pass => pal.pass,
        Verdict::Fail => pal.fail,
        Verdict::Pending => pal.warn,
    }
}

pub fn render_ledger(ui: &mut egui::Ui, pal: &Palette, detail: &ProblemDetail) {
    ui.heading(RichText::new("Verified Ledger & Excluded Dead Ends").color(pal.text));
    ui.label(
        RichText::new(
            "Confirmed intermediate lemmas are re-verified in isolation before promotion.",
        )
        .color(pal.muted),
    );
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in &detail.ledger {
            let (tag, col) = match entry.kind {
                LedgerKind::VerifiedLemma => ("VERIFIED LEMMA", pal.pass),
                LedgerKind::DeadEnd => ("DEAD END", pal.fail),
            };
            egui::Frame::new()
                .fill(pal.card)
                .stroke(Stroke::new(1.0, col.linear_multiply(0.6)))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        badge(ui, tag, col);
                        if entry.isolated_verified {
                            badge(ui, "ISOLATED CHECKED", pal.accent);
                        }
                        ui.label(RichText::new(&entry.title).strong().color(pal.text));
                    });
                    ui.add_space(4.0);
                    ui.label(RichText::new(&entry.statement).color(pal.text));
                    ui.label(
                        RichText::new(format!("Proof/Basis: {}", entry.proof_sketch))
                            .color(pal.muted),
                    );
                });
            ui.add_space(6.0);
        }
    });
}

pub fn render_advisor(ui: &mut egui::Ui, pal: &Palette, detail: &ProblemDetail) {
    ui.heading(RichText::new("Process Advisor Log").color(pal.text));
    ui.label(
        RichText::new("Non-mathematical meta-agent adjusting rigor & prover/verifier instructions across rounds.")
            .color(pal.muted),
    );
    ui.add_space(8.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        for note in &detail.advisor_notes {
            egui::Frame::new()
                .fill(pal.card)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    badge(ui, &format!("Round {}", note.round), pal.warn);
                    ui.label(RichText::new(&note.observation).color(pal.muted));
                    ui.label(
                        RichText::new(format!("Instruction Delta: {}", note.instruction_delta))
                            .strong()
                            .color(pal.text),
                    );
                });
            ui.add_space(6.0);
        }
    });
}

pub fn render_manuscript(
    ui: &mut egui::Ui,
    pal: &Palette,
    store: &ProofStore,
    problem_id: &str,
    export_tex: &mut bool,
    status_msg: &mut String,
) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Manuscript Export").color(pal.text));
        ui.selectable_value(export_tex, false, "Markdown (.md)");
        ui.selectable_value(export_tex, true, "LaTeX (.tex)");
        if ui.button("Save to Workspace File").clicked() {
            let fmt = if *export_tex { "tex" } else { "md" };
            if let Ok(text) = store.export_manuscript(problem_id, fmt) {
                let fname = format!("{problem_id}.{fmt}");
                match std::fs::write(&fname, text) {
                    Ok(_) => *status_msg = format!("Saved {fname}"),
                    Err(e) => *status_msg = format!("Save error: {e}"),
                }
            }
        }
    });
    ui.add_space(6.0);
    let fmt = if *export_tex { "tex" } else { "md" };
    match store.export_manuscript(problem_id, fmt) {
        Ok(mut text) => {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
            });
        }
        Err(e) => {
            ui.label(RichText::new(format!("Export error: {e}")).color(pal.fail));
        }
    }
}

pub fn render_help(ui: &mut egui::Ui, pal: &Palette, active_tab: &mut HelpTab) {
    ui.heading(RichText::new("proofcraft Help Center").color(pal.text));
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for tab in HelpTab::ALL {
            ui.selectable_value(active_tab, tab, tab.title());
        }
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::new()
            .fill(pal.card)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(6.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(active_tab.title())
                        .heading()
                        .color(pal.accent),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(active_tab.body()).color(pal.text));
            });
    });
}

#[allow(clippy::too_many_arguments)]
pub fn render_settings(
    ui: &mut egui::Ui,
    pal: &Palette,
    section: &mut SettingsSection,
    prefs: &mut UiPrefs,
    provider: &mut ProviderConfig,
    routing: &mut RoleRouting,
    budget: &mut BudgetConfig,
    discovery: &mut Option<DiscoveryResult>,
    store: &ProofStore,
    status_msg: &mut String,
) {
    ui.heading(RichText::new("Settings").color(pal.text));
    ui.separator();
    ui.columns(2, |cols| {
        cols[0].set_max_width(210.0);
        cols[0].vertical(|ui| {
            ui.selectable_value(
                section,
                SettingsSection::Appearance,
                "1. Appearance & Skins",
            );
            ui.selectable_value(section, SettingsSection::Budget, "2. Harness & Budget Caps");
            ui.selectable_value(section, SettingsSection::LlmProviders, "3. LLM & Providers");
            ui.selectable_value(section, SettingsSection::CoopAndData, "4. MCP Coop & Data");
        });

        cols[1].vertical(|ui| match section {
            SettingsSection::Appearance => {
                ui.label(RichText::new("Interface Skin").strong().color(pal.text));
                for s in Skin::ALL {
                    ui.radio_value(&mut prefs.skin, s, s.label());
                }
                ui.add_space(8.0);
                ui.add(
                    egui::Slider::new(&mut prefs.zoom_factor, 0.75..=2.0).text("Zoom (Ctrl+Wheel)"),
                );
                ui.checkbox(&mut prefs.show_tab_bar, "Show Top Tab Bar (Ctrl+B)");
            }
            SettingsSection::Budget => {
                ui.label(
                    RichText::new("Hard Budget Cap Failsafes")
                        .strong()
                        .color(pal.text),
                );
                ui.add(
                    egui::DragValue::new(&mut budget.max_calls_per_run)
                        .range(10..=5000)
                        .prefix("Max Calls/Run: "),
                );
                ui.add(
                    egui::DragValue::new(&mut budget.max_rounds_per_run)
                        .range(1..=100)
                        .prefix("Max Rounds/Run: "),
                );
                ui.add(
                    egui::DragValue::new(&mut budget.max_usd_per_run)
                        .range(0.1..=500.0)
                        .speed(0.5)
                        .prefix("Max USD/Run: $"),
                );
            }
            SettingsSection::LlmProviders => {
                ui.label(
                    RichText::new("Provider & Live Model Discovery")
                        .strong()
                        .color(pal.text),
                );
                ui.horizontal(|ui| {
                    for kind in [
                        ProviderKind::Ollama,
                        ProviderKind::LmStudio,
                        ProviderKind::OpenAiCompat,
                        ProviderKind::Gemini,
                        ProviderKind::Anthropic,
                    ] {
                        if ui
                            .selectable_label(provider.kind == kind, kind.as_str())
                            .clicked()
                        {
                            provider.kind = kind;
                            provider.base_url = kind.default_base_url().to_string();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Base URL:");
                    ui.text_edit_singleline(&mut provider.base_url);
                });
                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.add(egui::TextEdit::singleline(&mut provider.api_key).password(true));
                    let b = key_status_badge(provider);
                    let col = if b == "unset" { pal.warn } else { pal.pass };
                    badge(ui, b, col);
                });
                if ui.button("Test Connection & Discover Models").clicked() {
                    let res = discover_models(provider);
                    *status_msg = res.message.clone();
                    *discovery = Some(res);
                }
                if let Some(d) = discovery {
                    let col = if d.ok { pal.pass } else { pal.fail };
                    ui.label(RichText::new(&d.message).color(col));
                    if !d.models.is_empty() {
                        egui::ComboBox::from_label("Default Model")
                            .selected_text(&provider.default_model)
                            .show_ui(ui, |ui| {
                                for m in &d.models {
                                    ui.selectable_value(&mut provider.default_model, m.clone(), m);
                                }
                            });
                    }
                }
                ui.separator();
                ui.label(RichText::new("Hybrid Role Overrides (blank = Default Model)").strong());
                ui.horizontal(|ui| {
                    ui.label("Prover:");
                    ui.text_edit_singleline(&mut routing.prover_model);
                });
                ui.horizontal(|ui| {
                    ui.label("Verifier:");
                    ui.text_edit_singleline(&mut routing.verifier_model);
                });
                ui.horizontal(|ui| {
                    ui.label("Summarizer (local 4090):");
                    ui.text_edit_singleline(&mut routing.summarizer_model);
                });
                ui.horizontal(|ui| {
                    ui.label("Advisor (local 4090):");
                    ui.text_edit_singleline(&mut routing.advisor_model);
                });
            }
            SettingsSection::CoopAndData => {
                ui.label(
                    RichText::new("Agentic DE Coop (MCP Stdio)")
                        .strong()
                        .color(pal.text),
                );
                ui.label(
                    RichText::new("Command: proofcraft-mcp mcp")
                        .monospace()
                        .color(pal.accent),
                );
                ui.add_space(10.0);
                ui.label(RichText::new("Demo & Database").strong().color(pal.text));
                ui.horizontal(|ui| {
                    if ui.button("Clear Demo Data / Start Fresh").clicked() {
                        match store.clear_all() {
                            Ok(_) => *status_msg = "All problems cleared.".to_string(),
                            Err(e) => *status_msg = format!("Error: {e}"),
                        }
                    }
                    if ui.button("Reload Demo Seed").clicked() {
                        match store.load_embedded_seed() {
                            Ok(n) => *status_msg = format!("Loaded {n} demo problems."),
                            Err(e) => *status_msg = format!("Error: {e}"),
                        }
                    }
                });
            }
        });
    });
}

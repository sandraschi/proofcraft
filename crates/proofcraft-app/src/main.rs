mod theme;
mod views;

use eframe::egui::{self, Key, RichText};
use proofcraft_core::help::HelpTab;
use proofcraft_core::model::{
    BudgetConfig, DiscoveryResult, ProblemDetail, ProviderConfig, RoleRouting, RoundReport,
};
use proofcraft_core::store::ProofStore;
use proofcraft_core::{dispatch, Command};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use theme::{apply_skin, load_prefs, palette, save_prefs, Skin, UiPrefs};
use views::{
    render_advisor, render_help, render_ledger, render_manuscript, render_rounds, render_settings,
    render_workspace, MainTab, SettingsSection,
};

fn default_db_path() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local)
            .join("proofcraft")
            .join("proofcraft.sqlite")
    } else {
        PathBuf::from("proofcraft.sqlite")
    }
}

fn run_smoke_test() -> i32 {
    let Ok(store) = ProofStore::open_in_memory() else {
        eprintln!("smoke-test failed: could not open in-memory store");
        return 1;
    };
    let Ok(loaded) = store.load_embedded_seed() else {
        eprintln!("smoke-test failed: could not load embedded seed");
        return 1;
    };
    if loaded < 2 {
        eprintln!("smoke-test failed: expected >= 2 demo problems, got {loaded}");
        return 1;
    }
    let Ok(problems) = store.list_problems() else {
        eprintln!("smoke-test failed: list_problems failed");
        return 1;
    };
    let Ok(detail) = store.get_problem_detail(&problems[0].id) else {
        eprintln!("smoke-test failed: get_problem_detail failed");
        return 1;
    };
    if detail.ledger.is_empty() || detail.attempts.is_empty() {
        eprintln!("smoke-test failed: seed detail missing ledger or attempts");
        return 1;
    }
    println!(
        "proofcraft-app smoke-test OK ({} problems, {} ledger entries in first problem)",
        problems.len(),
        detail.ledger.len()
    );
    0
}

fn main() {
    if std::env::args().any(|a| a == "--smoke-test") {
        std::process::exit(run_smoke_test());
    }

    let db_path = default_db_path();
    let store = match ProofStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open proofcraft database: {e}");
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([920.0, 600.0])
            .with_title("proofcraft - Cogentic Automated Proof Discovery Studio"),
        ..Default::default()
    };

    let res = eframe::run_native(
        "proofcraft",
        options,
        Box::new(|cc| {
            let prefs = load_prefs();
            apply_skin(&cc.egui_ctx, &prefs);
            Ok(Box::new(ProofCraftApp::new(store, prefs)))
        }),
    );

    if let Err(e) = res {
        eprintln!("eframe error: {e}");
        std::process::exit(1);
    }
}

struct ProofCraftApp {
    store: ProofStore,
    prefs: UiPrefs,
    active_tab: MainTab,
    help_tab: HelpTab,
    settings_section: SettingsSection,
    selected_problem_id: String,
    provider: ProviderConfig,
    routing: RoleRouting,
    budget: BudgetConfig,
    discovery: Option<DiscoveryResult>,
    status_msg: String,
    export_tex: bool,
    new_prob_title: String,
    new_prob_domain: String,
    new_prob_stmt: String,
    new_prob_budget: u32,
    new_dir_name: String,
    new_dir_claim: String,
    provers_per_round: u32,
    show_palette: bool,
    palette_query: String,
    show_shortcuts: bool,
    show_about: bool,
    round_rx: Option<Receiver<std::result::Result<RoundReport, String>>>,
    round_running: bool,
}

impl ProofCraftApp {
    fn new(store: ProofStore, prefs: UiPrefs) -> Self {
        let selected_problem_id = store
            .list_problems()
            .ok()
            .and_then(|v| v.first().map(|p| p.id.clone()))
            .unwrap_or_default();

        Self {
            store,
            prefs,
            active_tab: MainTab::Workspace,
            help_tab: HelpTab::GetStarted,
            settings_section: SettingsSection::Appearance,
            selected_problem_id,
            provider: ProviderConfig::default(),
            routing: RoleRouting::default(),
            budget: BudgetConfig::default(),
            discovery: None,
            status_msg: "Ready. Select a problem or configure LLM discovery in Settings (Ctrl+,)."
                .to_string(),
            export_tex: false,
            new_prob_title: String::new(),
            new_prob_domain: "Theoretical Computer Science".to_string(),
            new_prob_stmt: String::new(),
            new_prob_budget: 200,
            new_dir_name: String::new(),
            new_dir_claim: String::new(),
            provers_per_round: 2,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_about: false,
            round_rx: None,
            round_running: false,
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let mut prefs_changed = false;
        ctx.input(|i| {
            if i.modifiers.ctrl && i.smooth_scroll_delta.y.abs() > 0.1 {
                let delta = if i.smooth_scroll_delta.y > 0.0 {
                    0.05
                } else {
                    -0.05
                };
                self.prefs.zoom_factor = (self.prefs.zoom_factor + delta).clamp(0.75, 2.0);
                prefs_changed = true;
            }
            if i.modifiers.ctrl && i.key_pressed(Key::B) {
                self.prefs.show_tab_bar = !self.prefs.show_tab_bar;
                prefs_changed = true;
            }
            if i.modifiers.ctrl && i.key_pressed(Key::K) {
                self.show_palette = !self.show_palette;
            }
            if i.modifiers.ctrl && i.key_pressed(Key::Comma) {
                self.active_tab = MainTab::Settings;
            }
        });
        if prefs_changed {
            apply_skin(ctx, &self.prefs);
            save_prefs(&self.prefs);
        }
    }

    fn trigger_round(&mut self) {
        if self.round_running || self.selected_problem_id.is_empty() {
            return;
        }
        let pid = self.selected_problem_id.clone();
        let provers = self.provers_per_round;
        let provider = self.provider.clone();
        let routing = self.routing.clone();
        let budget = self.budget.clone();
        let db_path = default_db_path();
        let (tx, rx) = mpsc::channel();
        self.round_rx = Some(rx);
        self.round_running = true;
        self.status_msg = format!("Running Cogentic round ({provers} provers) on {pid}...");

        std::thread::spawn(move || {
            let res = match ProofStore::open(&db_path) {
                Ok(s) => dispatch(
                    &s,
                    &provider,
                    &routing,
                    &budget,
                    Command::RunRound {
                        problem_id: pid,
                        provers,
                    },
                )
                .and_then(|v| serde_json::from_value::<RoundReport>(v).map_err(Into::into))
                .map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(res);
        });
    }

    fn poll_background(&mut self) {
        if let Some(rx) = &self.round_rx {
            if let Ok(res) = rx.try_recv() {
                self.round_running = false;
                self.round_rx = None;
                match res {
                    Ok(report) => {
                        self.status_msg = format!(
                            "Round {} complete: {} calls, {} lemmas promoted, verified={}",
                            report.round,
                            report.calls_spent,
                            report.lemmas_promoted,
                            report.verified_solution_found
                        );
                    }
                    Err(err) => {
                        self.status_msg = format!("Round failed: {err}");
                    }
                }
            }
        }
    }
}

impl eframe::App for ProofCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.poll_background();
        let pal = palette(self.prefs.skin);

        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Export Markdown...").clicked() {
                        self.export_tex = false;
                        self.active_tab = MainTab::Manuscript;
                        ui.close();
                    }
                    if ui.button("Export LaTeX...").clicked() {
                        self.export_tex = true;
                        self.active_tab = MainTab::Manuscript;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Preferences... (Ctrl+,)").clicked() {
                        self.active_tab = MainTab::Settings;
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.prefs.show_tab_bar, "Show Tab Bar (Ctrl+B)")
                        .changed()
                    {
                        save_prefs(&self.prefs);
                    }
                    ui.menu_button("Interface Theme", |ui| {
                        for s in Skin::ALL {
                            if ui.radio_value(&mut self.prefs.skin, s, s.label()).clicked() {
                                apply_skin(&ctx, &self.prefs);
                                save_prefs(&self.prefs);
                                ui.close();
                            }
                        }
                    });
                    if ui.button("Reset Zoom (100%)").clicked() {
                        self.prefs.zoom_factor = 1.0;
                        apply_skin(&ctx, &self.prefs);
                        save_prefs(&self.prefs);
                        ui.close();
                    }
                });
                ui.menu_button("Run", |ui| {
                    if ui.button("Run Proof Round").clicked() {
                        self.trigger_round();
                        ui.close();
                    }
                    if ui.button("View Rounds & Verifiers").clicked() {
                        self.active_tab = MainTab::Rounds;
                        ui.close();
                    }
                });
                ui.menu_button("Ledger", |ui| {
                    if ui.button("Verified Lemmas & Dead Ends").clicked() {
                        self.active_tab = MainTab::Ledger;
                        ui.close();
                    }
                    if ui.button("Process Advisor Log").clicked() {
                        self.active_tab = MainTab::Advisor;
                        ui.close();
                    }
                });
                ui.menu_button("Window", |ui| {
                    for tab in MainTab::ALL {
                        if ui.button(tab.label()).clicked() {
                            self.active_tab = tab;
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Command Palette... (Ctrl+K)").clicked() {
                        self.show_palette = true;
                        ui.close();
                    }
                    if ui.button("Keyboard Shortcuts...").clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                    ui.separator();
                    for h in HelpTab::ALL {
                        if ui.button(h.title()).clicked() {
                            self.help_tab = h;
                            self.active_tab = MainTab::Help;
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("About proofcraft").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(self.prefs.skin.label()).clicked() {
                        self.prefs.skin = match self.prefs.skin {
                            Skin::Dark => Skin::Light,
                            Skin::Light => Skin::IvoryProof,
                            Skin::IvoryProof => Skin::Dark,
                        };
                        apply_skin(&ctx, &self.prefs);
                        save_prefs(&self.prefs);
                    }
                });
            });

            if self.prefs.show_tab_bar {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    for tab in MainTab::ALL {
                        ui.selectable_value(&mut self.active_tab, tab, tab.label());
                    }
                });
            }
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status_msg).color(pal.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Provider: {} | Zoom: {:.0}%",
                            self.provider.kind.as_str(),
                            self.prefs.zoom_factor * 100.0
                        ))
                        .color(pal.muted),
                    );
                });
            });
        });

        egui::Panel::left("problems_sidebar")
            .default_size(270.0)
            .show(ui, |ui| {
                ui.heading(RichText::new("Research Problems").color(pal.text));
                ui.add_space(4.0);
                let problems = self.store.list_problems().unwrap_or_default();
                for p in &problems {
                    let selected = p.id == self.selected_problem_id;
                    if ui
                        .selectable_label(selected, format!("[{}] {}", p.status.as_str(), p.title))
                        .clicked()
                    {
                        self.selected_problem_id = p.id.clone();
                    }
                }
                if self.selected_problem_id.is_empty() {
                    if let Some(first) = problems.first() {
                        self.selected_problem_id = first.id.clone();
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Provers:");
                    ui.add(egui::DragValue::new(&mut self.provers_per_round).range(1..=8));
                });
                let btn_label = if self.round_running {
                    "Running Round..."
                } else {
                    "Run Proof Round"
                };
                if ui
                    .add_enabled(
                        !self.round_running && !self.selected_problem_id.is_empty(),
                        egui::Button::new(btn_label),
                    )
                    .clicked()
                {
                    self.trigger_round();
                }

                ui.separator();
                ui.collapsing("New Problem", |ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_prob_title);
                    ui.label("Domain:");
                    ui.text_edit_singleline(&mut self.new_prob_domain);
                    ui.label("Statement:");
                    ui.text_edit_multiline(&mut self.new_prob_stmt);
                    ui.add(
                        egui::DragValue::new(&mut self.new_prob_budget)
                            .range(10..=2000)
                            .prefix("Max Calls: "),
                    );
                    if ui.button("Create Problem").clicked() {
                        match self.store.create_problem(
                            &self.new_prob_title,
                            &self.new_prob_domain,
                            &self.new_prob_stmt,
                            self.new_prob_budget,
                        ) {
                            Ok(p) => {
                                self.selected_problem_id = p.id;
                                self.new_prob_title.clear();
                                self.new_prob_stmt.clear();
                                self.status_msg = "Created new problem.".to_string();
                            }
                            Err(e) => self.status_msg = format!("Error: {e}"),
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let detail: Option<ProblemDetail> = if self.selected_problem_id.is_empty() {
                None
            } else {
                self.store.get_problem_detail(&self.selected_problem_id).ok()
            };

            match self.active_tab {
                MainTab::Help => render_help(ui, &pal, &mut self.help_tab),
                MainTab::Settings => render_settings(
                    ui,
                    &pal,
                    &mut self.settings_section,
                    &mut self.prefs,
                    &mut self.provider,
                    &mut self.routing,
                    &mut self.budget,
                    &mut self.discovery,
                    &self.store,
                    &mut self.status_msg,
                ),
                _ => {
                    let Some(d) = detail else {
                        ui.label("No problem selected. Create one in the sidebar or reload Demo Data in Settings.");
                        return;
                    };
                    match self.active_tab {
                        MainTab::Workspace => render_workspace(
                            ui,
                            &pal,
                            &d,
                            &mut self.new_dir_name,
                            &mut self.new_dir_claim,
                            &self.store,
                            &mut self.status_msg,
                        ),
                        MainTab::Rounds => render_rounds(ui, &pal, &d),
                        MainTab::Ledger => render_ledger(ui, &pal, &d),
                        MainTab::Advisor => render_advisor(ui, &pal, &d),
                        MainTab::Manuscript => render_manuscript(
                            ui,
                            &pal,
                            &self.store,
                            &d.problem.id,
                            &mut self.export_tex,
                            &mut self.status_msg,
                        ),
                        MainTab::Help | MainTab::Settings => {}
                    }
                }
            }
        });

        if self.show_palette {
            egui::Window::new("Command Palette (Ctrl+K)")
                .collapsible(false)
                .open(&mut self.show_palette)
                .show(&ctx, |ui| {
                    ui.text_edit_singleline(&mut self.palette_query);
                    let q = self.palette_query.to_ascii_lowercase();
                    for tab in MainTab::ALL {
                        if (q.is_empty() || tab.label().to_ascii_lowercase().contains(&q))
                            && ui.button(format!("Go to: {}", tab.label())).clicked()
                        {
                            self.active_tab = tab;
                        }
                    }
                });
        }

        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .collapsible(false)
                .open(&mut self.show_shortcuts)
                .show(&ctx, |ui| {
                    ui.label("Ctrl + Mouse Wheel : Zoom UI");
                    ui.label("Ctrl + B : Show/Hide Tab Bar");
                    ui.label("Ctrl + K : Command Palette");
                    ui.label("Ctrl + , : Settings");
                });
        }

        if self.show_about {
            egui::Window::new("About proofcraft")
                .collapsible(false)
                .open(&mut self.show_about)
                .show(&ctx, |ui| {
                    ui.heading("proofcraft v0.1.0");
                    ui.label("Cogentic-inspired multi-agent automated proof discovery studio.");
                    ui.label("Pure Rust (L1 core + L2 CLI/MCP + L3 egui).");
                    ui.label("License: MIT OR Apache-2.0");
                });
        }
    }
}

use eframe::egui::{self, Color32, Stroke, Visuals};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Skin {
    Dark,
    Light,
    IvoryProof,
}

impl Skin {
    pub const ALL: [Self; 3] = [Self::Dark, Self::Light, Self::IvoryProof];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Dark => "Obsidian Dark",
            Self::Light => "Crisp Light",
            Self::IvoryProof => "Ivory Proof (Paper)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrefs {
    pub skin: Skin,
    pub zoom_factor: f32,
    pub show_tab_bar: bool,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            skin: Skin::Dark,
            zoom_factor: 1.0,
            show_tab_bar: true,
        }
    }
}

pub fn prefs_path() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local).join("proofcraft").join("ui.json")
    } else {
        PathBuf::from("proofcraft_ui.json")
    }
}

pub fn load_prefs() -> UiPrefs {
    let path = prefs_path();
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(p) = serde_json::from_str::<UiPrefs>(&raw) {
            return p;
        }
    }
    UiPrefs::default()
}

pub fn save_prefs(prefs: &UiPrefs) {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(prefs) {
        let _ = std::fs::write(path, json);
    }
}

pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub pass: Color32,
    pub fail: Color32,
    pub warn: Color32,
}

pub fn palette(skin: Skin) -> Palette {
    match skin {
        Skin::Dark => Palette {
            bg: Color32::from_rgb(15, 19, 28),
            panel: Color32::from_rgb(22, 27, 39),
            card: Color32::from_rgb(28, 35, 50),
            border: Color32::from_rgb(56, 68, 92),
            text: Color32::from_rgb(235, 241, 249),
            muted: Color32::from_rgb(165, 178, 200),
            accent: Color32::from_rgb(86, 146, 255),
            pass: Color32::from_rgb(34, 197, 94),
            fail: Color32::from_rgb(239, 68, 68),
            warn: Color32::from_rgb(245, 158, 11),
        },
        Skin::Light => Palette {
            bg: Color32::from_rgb(244, 246, 250),
            panel: Color32::from_rgb(255, 255, 255),
            card: Color32::from_rgb(249, 250, 252),
            border: Color32::from_rgb(190, 200, 215),
            text: Color32::from_rgb(18, 24, 38),
            muted: Color32::from_rgb(68, 78, 96),
            accent: Color32::from_rgb(29, 78, 216),
            pass: Color32::from_rgb(21, 128, 61),
            fail: Color32::from_rgb(185, 28, 28),
            warn: Color32::from_rgb(180, 83, 9),
        },
        Skin::IvoryProof => Palette {
            bg: Color32::from_rgb(248, 244, 234),
            panel: Color32::from_rgb(253, 250, 243),
            card: Color32::from_rgb(244, 238, 224),
            border: Color32::from_rgb(188, 176, 152),
            text: Color32::from_rgb(26, 22, 18),
            muted: Color32::from_rgb(82, 72, 58),
            accent: Color32::from_rgb(124, 45, 18),
            pass: Color32::from_rgb(21, 128, 61),
            fail: Color32::from_rgb(185, 28, 28),
            warn: Color32::from_rgb(161, 98, 7),
        },
    }
}

pub fn apply_skin(ctx: &egui::Context, prefs: &UiPrefs) {
    let pal = palette(prefs.skin);
    let mut visuals = match prefs.skin {
        Skin::Dark => Visuals::dark(),
        Skin::Light | Skin::IvoryProof => Visuals::light(),
    };
    visuals.panel_fill = pal.panel;
    visuals.window_fill = pal.bg;
    visuals.extreme_bg_color = pal.card;
    visuals.override_text_color = Some(pal.text);
    visuals.widgets.noninteractive.bg_fill = pal.card;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, pal.border);
    visuals.widgets.inactive.bg_fill = pal.card;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, pal.border);
    visuals.selection.bg_fill = pal.accent.linear_multiply(0.35);
    ctx.set_visuals(visuals);
    ctx.set_pixels_per_point(prefs.zoom_factor.clamp(0.75, 2.0));
}

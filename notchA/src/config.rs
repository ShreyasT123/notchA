use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const SIZES: [f64; 3] = [0.8, 1.0, 1.25];

pub fn snap_scale(scale: f64) -> f64 {
    if scale < 0.9 {
        SIZES[0]
    } else if scale < 1.125 || !scale.is_finite() {
        SIZES[1]
    } else {
        SIZES[2]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraySlot {
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeProfileConfig {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "yes")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub claude_profiles: Vec<ClaudeProfileConfig>,
    #[serde(default = "default_taskbar_mode")]
    pub taskbar_mode: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub bar_x: Option<i32>,
    #[serde(default)]
    pub bar_y: Option<i32>,
    #[serde(default)]
    pub bar_w: Option<u32>,
    #[serde(default)]
    pub drag_enabled: bool,
    #[serde(default)]
    pub notch_along: BTreeMap<String, f64>,
    #[serde(default = "default_notch_edge")]
    pub notch_edge: String,
    #[serde(default)]
    pub notch_monitor: Option<String>,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default = "default_weekly_ring")]
    pub weekly_ring: String,
    #[serde(default)]
    pub weekly_ring_dashed: bool,
    #[serde(default = "default_color_transition")]
    pub color_transition: String,
    #[serde(default = "default_watch_limit")]
    pub watch_limit: f64,
    #[serde(default = "default_critical_limit")]
    pub critical_limit: f64,
    #[serde(default = "default_theme", deserialize_with = "deserialize_theme_or_system")]
    pub theme: String,
    #[serde(default)]
    pub notch_slots: Vec<TraySlot>,
    #[serde(default = "default_antigravity_limit")]
    pub antigravity_limit: String,
    #[serde(default = "default_antigravity_model")]
    pub antigravity_model: String,
    #[serde(default = "yes")]
    pub notch_visible: bool,
    #[serde(default = "yes")]
    pub notch_on_hover: bool,
    #[serde(default = "yes")]
    pub tray_visible: bool,
    #[serde(default = "yes")]
    pub reset_notifications: bool,
    #[serde(default = "yes")]
    pub reset_notification_sound: bool,
    #[serde(default)]
    pub adaptive_pill: bool,
    #[serde(default = "yes")]
    pub fold_for_full_screen: bool,
    #[serde(default)]
    pub notch_shows_remaining: bool,
    #[serde(default)]
    pub show_usage_pace: bool,
    #[serde(default = "yes")]
    pub require_event_token: bool,
}

fn default_taskbar_mode() -> String { "off".into() }
fn default_notch_edge() -> String { "right".into() }
pub const EDGES: [&str; 4] = ["left", "right", "top", "bottom"];

pub fn edge_or_right(value: &str) -> String {
    if EDGES.contains(&value) { value.to_string() } else { "right".into() }
}

pub fn edge_is_vertical(edge: &str) -> bool {
    matches!(edge, "left" | "right")
}

impl Config {
    pub fn along(&self, edge: &str) -> f64 {
        self.notch_along.get(edge).copied().unwrap_or(0.5).clamp(0.0, 1.0)
    }
    pub fn set_along(&mut self, edge: &str, along: f64) {
        self.notch_along.insert(edge.to_string(), along.clamp(0.0, 1.0));
    }
}

fn default_scale() -> f64 { 1.0 }
fn default_weekly_ring() -> String { "off".into() }
fn default_color_transition() -> String { "hard_step".into() }
pub fn default_watch_limit() -> f64 { 0.5 }
pub fn default_critical_limit() -> f64 { 0.7 }

pub fn clamp_watch_limit(watch: f64, critical: f64) -> f64 {
    watch.clamp(0.01, (critical - 0.01).max(0.01))
}

pub fn clamp_critical_limit(critical: f64, watch: f64) -> f64 {
    critical.clamp((watch + 0.01).min(1.0), 1.0)
}

fn default_theme() -> String { "system".into() }

pub fn theme_or_system(value: &str) -> String {
    match value {
        "light" | "dark" => value.to_string(),
        _ => default_theme(),
    }
}

fn deserialize_theme_or_system<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<serde_json::Value>::deserialize(deserializer)
        .ok()
        .flatten()
        .and_then(|value| value.as_str().map(theme_or_system))
        .unwrap_or_else(default_theme))
}

pub fn weekly_ring_or_off(value: &str) -> String {
    match value {
        "inside" | "outside" => value.to_string(),
        _ => default_weekly_ring(),
    }
}

pub fn color_transition_or_step(value: &str) -> String {
    match value {
        "ramp" => value.to_string(),
        _ => default_color_transition(),
    }
}

fn yes() -> bool { true }
fn default_antigravity_limit() -> String { "automatic".into() }
fn default_antigravity_model() -> String { "gemini".into() }
fn default_port() -> u16 { 48666 }
fn default_lang() -> String { "auto".into() }

impl Default for Config {
    fn default() -> Self {
        Self {
            claude_profiles: Vec::new(),
            taskbar_mode: default_taskbar_mode(),
            port: default_port(),
            lang: default_lang(),
            bar_x: None,
            bar_y: None,
            bar_w: None,
            drag_enabled: false,
            notch_along: BTreeMap::new(),
            notch_edge: default_notch_edge(),
            notch_monitor: None,
            scale: default_scale(),
            weekly_ring: default_weekly_ring(),
            weekly_ring_dashed: false,
            color_transition: default_color_transition(),
            watch_limit: default_watch_limit(),
            critical_limit: default_critical_limit(),
            theme: default_theme(),
            notch_slots: Vec::new(),
            antigravity_limit: default_antigravity_limit(),
            antigravity_model: default_antigravity_model(),
            notch_visible: true,
            notch_on_hover: true,
            tray_visible: true,
            reset_notifications: true,
            reset_notification_sound: true,
            adaptive_pill: false,
            fold_for_full_screen: true,
            notch_shows_remaining: false,
            show_usage_pace: false,
            require_event_token: true,
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("notchA")
        .join("config.json")
}

fn without_bom(text: String) -> String {
    text.strip_prefix('\u{feff}').map(str::to_string).unwrap_or(text)
}

fn set_aside(path: &std::path::Path, err: &serde_json::Error) {
    let kept = path.with_file_name("config.unreadable.json");
    let note = match std::fs::copy(path, &kept) {
        Ok(_) => "the file is kept as config.unreadable.json".to_string(),
        Err(e) => format!("could not keep a copy ({e})"),
    };
    crate::applog(&format!("config: config.json could not be read ({err}), running on defaults; {note}"));
}

pub fn load() -> Config {
    let path = config_path();
    let raw = std::fs::read_to_string(&path).ok().map(without_bom);
    let mut cfg: Config = match raw.as_deref().map(serde_json::from_str::<Config>) {
        Some(Ok(cfg)) => cfg,
        Some(Err(e)) => {
            set_aside(&path, &e);
            Config::default()
        }
        None => Config::default(),
    };

    if !cfg.notch_visible && !cfg.tray_visible {
        cfg.tray_visible = true;
    }
    cfg.scale = snap_scale(cfg.scale);
    cfg.weekly_ring = weekly_ring_or_off(&cfg.weekly_ring);
    cfg.color_transition = color_transition_or_step(&cfg.color_transition);
    cfg.theme = theme_or_system(&cfg.theme);
    cfg.critical_limit = cfg.critical_limit.clamp(0.02, 1.0);
    cfg.watch_limit = clamp_watch_limit(cfg.watch_limit, cfg.critical_limit);
    cfg
}

pub fn save(cfg: &Config) {
    let _ = save_checked(cfg);
}

pub fn save_checked(cfg: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|_| "Unable to create the settings folder".to_string())?;
    }
    let txt = serde_json::to_string_pretty(cfg).map_err(|_| "Unable to serialize settings".to_string())?;
    std::fs::write(path, txt).map_err(|_| "Unable to save settings".to_string())
}
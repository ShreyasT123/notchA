use crate::{activity, config, glyphs, reset_alert, state, usage};
use serde::Serialize;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;

#[allow(dead_code)]
#[allow(dead_code)]
pub const BUILD: &str = "r33-iced";
pub const NOTCH_W: f64 = 360.0;
pub const NOTCH_LONG: f64 = 650.0;

pub const TRAY_PROVIDER_IDS: [&str; 5] = ["claude", "codex", "copilot", "gemini", "opencode"];

pub static DRAGGING: AtomicBool = AtomicBool::new(false);
pub static EXPANDED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[allow(dead_code)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicalPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

pub enum UiEvent {
    BackendEvent {
        target: Option<String>,
        name: String,
        payload: serde_json::Value,
    },
    MainThread(Box<dyn FnOnce() + Send>),
    PlaceNotch,
    GlideNotch,
    ApplyVisibility,
    ApplyTheme,
    OpenSettings,
    ShowTaskbarHover {
        id: String,
        x: f64,
        width: f64,
    },
    HideTaskbarHover,
    OpenTaskbarDetails,
    ShowResetAlert,
    HideResetAlert,
    MenuAction(String),
    Quit,
}

impl std::fmt::Debug for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MainThread(_) => write!(f, "MainThread(..)"),
            Self::BackendEvent { target, name, .. } => {
                write!(f, "BackendEvent({target:?}, {name})")
            }
            Self::PlaceNotch => write!(f, "PlaceNotch"),
            Self::GlideNotch => write!(f, "GlideNotch"),
            Self::ApplyVisibility => write!(f, "ApplyVisibility"),
            Self::ApplyTheme => write!(f, "ApplyTheme"),
            Self::OpenSettings => write!(f, "OpenSettings"),
            Self::ShowTaskbarHover { id, .. } => write!(f, "ShowTaskbarHover({id})"),
            Self::HideTaskbarHover => write!(f, "HideTaskbarHover"),
            Self::OpenTaskbarDetails => write!(f, "OpenTaskbarDetails"),
            Self::ShowResetAlert => write!(f, "ShowResetAlert"),
            Self::HideResetAlert => write!(f, "HideResetAlert"),
            Self::MenuAction(id) => write!(f, "MenuAction({id})"),
            Self::Quit => write!(f, "Quit"),
        }
    }
}

pub struct AppState {
    pub store: Mutex<state::Store>,
    pub cfg: Mutex<config::Config>,
    pub usage: Mutex<usage::UsageSnapshot>,
    pub codex: Mutex<usage::UsageSnapshot>,
    pub copilot: Mutex<usage::UsageSnapshot>,
    pub antigravity: Mutex<usage::UsageSnapshot>,
    pub opencode: Mutex<usage::UsageSnapshot>,
    pub reset_alerts: reset_alert::AlertQueue,
    pub glyphs: Mutex<HashMap<String, glyphs::Glyph>>,
    pub activity: Mutex<Vec<activity::Activity>>,
}

#[derive(Default)]
pub struct WindowRegistry {
    pub hwnds: Mutex<HashMap<String, isize>>,
}

pub struct AppCore {
    pub app_state: AppState,
    pub windows: WindowRegistry,
    pub tx: Mutex<Option<mpsc::UnboundedSender<UiEvent>>>,
}

pub type AppHandle = Arc<AppCore>;

static GLOBAL_CORE: OnceLock<AppHandle> = OnceLock::new();

pub fn init_core(cfg: config::Config) -> (AppHandle, mpsc::UnboundedReceiver<UiEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let core = Arc::new(AppCore {
        app_state: AppState {
            store: Mutex::new(Default::default()),
            cfg: Mutex::new(cfg),
            usage: Mutex::new(usage::load_persisted()),
            codex: Mutex::new(crate::codex::load_persisted()),
            copilot: Mutex::new(crate::copilot::load_persisted()),
            antigravity: Mutex::new(crate::antigravity::load_persisted()),
            opencode: Mutex::new(crate::opencode::load_persisted()),
            reset_alerts: reset_alert::AlertQueue::default(),
            glyphs: Mutex::new(Default::default()),
            activity: Mutex::new(Vec::new()),
        },
        windows: WindowRegistry::default(),
        tx: Mutex::new(Some(tx)),
    });
    let _ = GLOBAL_CORE.set(core.clone());
    (core, rx)
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn global_core() -> Option<AppHandle> {
    GLOBAL_CORE.get().cloned()
}

pub trait Manager {
    fn state<T: 'static>(&self) -> &T;
}

impl Manager for AppHandle {
    fn state<T: 'static>(&self) -> &T {
        (&self.app_state as &dyn Any)
            .downcast_ref::<T>()
            .expect("AppHandle::state only stores AppState")
    }
}

pub trait Emitter {
    fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String>;
    fn emit_to<S: Serialize>(&self, target: &str, event: &str, payload: S) -> Result<(), String>;
}

impl Emitter for AppHandle {
    fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String> {
        let value = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
        self.send_ui(UiEvent::BackendEvent {
            target: None,
            name: event.to_string(),
            payload: value,
        });
        Ok(())
    }

    fn emit_to<S: Serialize>(&self, target: &str, event: &str, payload: S) -> Result<(), String> {
        let value = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
        self.send_ui(UiEvent::BackendEvent {
            target: Some(target.to_string()),
            name: event.to_string(),
            payload: value,
        });
        Ok(())
    }
}

impl AppCore {
    pub fn send_ui(&self, ev: UiEvent) {
        if let Some(tx) = self.tx.lock().unwrap().as_ref() {
            let _ = tx.send(ev);
        }
    }

    pub fn register_hwnd(&self, label: &str, hwnd: isize) {
        self.windows.hwnds.lock().unwrap().insert(label.to_string(), hwnd);
    }

    pub fn unregister_hwnd(&self, label: &str) {
        self.windows.hwnds.lock().unwrap().remove(label);
    }

    pub fn get_webview_window(&self, label: &str) -> Option<WindowHandle> {
        let hwnd = *self.windows.hwnds.lock().unwrap().get(label)?;
        if hwnd == 0 {
            return None;
        }
        Some(WindowHandle { hwnd })
    }

    pub fn run_on_main_thread<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        let tx = self.tx.lock().unwrap();
        let Some(tx) = tx.as_ref() else {
            return Err("event loop closed".into());
        };
        tx.send(UiEvent::MainThread(Box::new(f)))
            .map_err(|_| "failed to send to UI loop".to_string())
    }

    pub fn exit(&self, _code: i32) {
        self.send_ui(UiEvent::Quit);
    }

    #[cfg(windows)]
    pub fn cursor_position(&self) -> Result<PhysicalPosition, String> {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        let mut pt = POINT::default();
        unsafe { GetCursorPos(&mut pt) }.map_err(|e| e.to_string())?;
        Ok(PhysicalPosition {
            x: pt.x as f64,
            y: pt.y as f64,
        })
    }

    #[cfg(not(windows))]
    pub fn cursor_position(&self) -> Result<PhysicalPosition, String> {
        Err("unsupported".into())
    }
}

/// Lightweight HWND wrapper so `backdrop.rs`, `fullscreen.rs`, and `topmost.rs`
/// work without changing their Win32 logic.
#[derive(Clone, Copy)]
pub struct WindowHandle {
    pub hwnd: isize,
}

#[derive(Clone, Copy)]
pub struct RawHwnd(pub isize);

impl WindowHandle {
    pub fn hwnd(&self) -> Result<RawHwnd, String> {
        Ok(RawHwnd(self.hwnd))
    }

    #[cfg(windows)]
    pub fn is_visible(&self) -> Result<bool, String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{IsWindow, IsWindowVisible};
        let h = HWND(self.hwnd as _);
        unsafe { Ok(IsWindow(h).as_bool() && IsWindowVisible(h).as_bool()) }
    }

    #[cfg(windows)]
    pub fn outer_position(&self) -> Result<PhysicalPosition, String> {
        use windows::Win32::Foundation::{HWND, RECT};
        use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
        let mut r = RECT::default();
        unsafe { GetWindowRect(HWND(self.hwnd as _), &mut r) }.map_err(|e| e.to_string())?;
        Ok(PhysicalPosition {
            x: r.left as f64,
            y: r.top as f64,
        })
    }

    #[cfg(windows)]
    pub fn outer_size(&self) -> Result<PhysicalSize, String> {
        use windows::Win32::Foundation::{HWND, RECT};
        use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
        let mut r = RECT::default();
        unsafe { GetWindowRect(HWND(self.hwnd as _), &mut r) }.map_err(|e| e.to_string())?;
        Ok(PhysicalSize {
            width: (r.right - r.left).max(0) as u32,
            height: (r.bottom - r.top).max(0) as u32,
        })
    }

    #[cfg(not(windows))]
    pub fn is_visible(&self) -> Result<bool, String> { Ok(false) }
    #[cfg(not(windows))]
    pub fn outer_position(&self) -> Result<PhysicalPosition, String> { Err("unsupported".into()) }
    #[cfg(not(windows))]
    pub fn outer_size(&self) -> Result<PhysicalSize, String> { Err("unsupported".into()) }
}

// ---------------- Logging & Shared Helpers ----------------

const RUN_LOG_MAX_BYTES: u64 = 1024 * 1024;

pub fn applog(line: &str) {
    use std::io::Write;
    let log = config::config_path().with_file_name("run.log");
    let full = std::fs::metadata(&log)
        .map(|m| m.len() > RUN_LOG_MAX_BYTES)
        .unwrap_or(false);
    let opened = if full {
        std::fs::File::create(&log)
    } else {
        std::fs::OpenOptions::new().create(true).append(true).open(&log)
    };
    if let Ok(mut f) = opened {
        let _ = writeln!(f, "{line}");
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn ui_scale(app: &AppHandle) -> f64 {
    let c = app.state::<AppState>().cfg.lock().unwrap();
    config::snap_scale(c.scale)
}

pub fn broadcast(app: &AppHandle) {
    let st = app.state::<AppState>();
    let snap = {
        let store = st.store.lock().unwrap();
        let _cfg = st.cfg.lock().unwrap();
        store.snapshot("en", "en", crate::i18n::clock_24h(), false)
    };
    let _ = app.emit("state", &snap);
}

pub fn provider_label(id: &str) -> &'static str {
    match id {
        "codex" => "Codex",
        "copilot" => "GitHub Copilot",
        "gemini" => "Antigravity",
        "opencode" => "OpenCode",
        _ => "Claude",
    }
}

pub fn provider_page(provider: &str) -> Option<(&'static str, &'static str)> {
    Some(match provider {
        "claude" => ("https://claude.ai/settings/usage", "claude.ai"),
        "codex" => ("https://chatgpt.com/#settings/Account", "chatgpt.com"),
        "copilot" => ("https://github.com/settings/copilot", "github.com"),
        "gemini" => ("https://antigravity.google", "antigravity.google"),
        "opencode" => ("https://opencode.ai", "opencode.ai"),
        _ => return None,
    })
}

pub fn open_provider_page(provider: &str) {
    let Some((url, _)) = provider_page(provider) else { return };
    let mut cmd = std::process::Command::new("cmd");
    cmd.args(["/C", "start", "", url]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd.spawn();
}

pub fn open_data_dir() {
    let dir = config::config_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let _ = std::fs::create_dir_all(glyphs::user_dir());
    let mut cmd = std::process::Command::new("explorer");
    cmd.arg(dir.as_os_str());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd.spawn();
}

pub fn snapshot_of(app: &AppHandle, id: &str) -> usage::UsageSnapshot {
    let st = app.state::<AppState>();
    match id {
        "codex" => st.codex.lock().unwrap().clone(),
        "copilot" => st.copilot.lock().unwrap().clone(),
        "gemini" => st.antigravity.lock().unwrap().clone(),
        "opencode" => st.opencode.lock().unwrap().clone(),
        _ => st.usage.lock().unwrap().clone(),
    }
}

pub fn refresh_provider(provider: &str) -> bool {
    match provider {
        "claude" => usage::request_refresh(),
        "codex" => crate::codex::request_refresh(),
        "copilot" => crate::copilot::request_refresh(),
        "gemini" => crate::antigravity::request_refresh(),
        "opencode" => crate::opencode::request_refresh(),
        _ => return false,
    }
    true
}

pub fn refresh_all(app: &AppHandle) {
    for provider in TRAY_PROVIDER_IDS {
        refresh_provider(provider);
    }
    let a = app.clone();
    std::thread::spawn(move || reload_glyphs(&a));
}

pub fn reload_glyphs(app: &AppHandle) {
    let m = glyphs::collect();
    *app.state::<AppState>().glyphs.lock().unwrap() = m.clone();
    let _ = app.emit("glyphs", &m);
}

pub fn tightest<'a>(
    windows: impl Iterator<Item = &'a usage::LimitWindow>,
) -> Option<&'a usage::LimitWindow> {
    windows
        .filter(|w| w.metered())
        .max_by(|a, b| a.used.total_cmp(&b.used).then_with(|| b.id.cmp(&a.id)))
}

pub fn lane_family(w: &usage::LimitWindow) -> &'static str {
    let id = w.id.to_lowercase();
    if id.starts_with("gemini") {
        "gemini"
    } else if id.starts_with("3p") || id.starts_with("claude") {
        "3p"
    } else {
        ""
    }
}

pub fn lane_is(w: &usage::LimitWindow, limit: &str) -> bool {
    let text = format!("{} {}", w.id, w.label).to_lowercase();
    match limit {
        "weekly" => text.contains("weekly"),
        _ => ["5h", "5-hour", "five hour", "five-hour", "hourly", "session"]
            .iter()
            .any(|k| text.contains(k)),
    }
}

pub fn ring_window<'a>(
    provider: &str,
    windows: &'a [usage::LimitWindow],
    antigravity_limit: &str,
    antigravity_model: &str,
) -> Option<&'a usage::LimitWindow> {
    let by_id = |id: &str| windows.iter().find(|w| w.id == id);
    match provider {
        "claude" => by_id("session").or_else(|| by_id("spend")),
        "codex" => by_id("primary"),
        "copilot" => by_id("premium_interactions").or_else(|| windows.first()),
        "opencode" => by_id("rolling"),
        _ => antigravity_lane(windows, antigravity_limit, antigravity_model),
    }
}

pub fn antigravity_lane<'a>(
    windows: &'a [usage::LimitWindow],
    limit: &str,
    model: &str,
) -> Option<&'a usage::LimitWindow> {
    let family: Vec<_> = windows.iter().filter(|w| lane_family(w) == model).collect();
    let lanes = if family.is_empty() {
        windows.iter().collect()
    } else {
        family
    };
    if limit != "automatic" {
        if let Some(w) = tightest(lanes.iter().copied().filter(|w| lane_is(w, limit))) {
            return Some(w);
        }
    }
    tightest(lanes.iter().copied().filter(|w| w.used < 1.0))
        .or_else(|| tightest(lanes.iter().copied()))
        .or_else(|| lanes.first().copied())
}

pub fn ring_fraction(app: &AppHandle, provider: &str) -> Option<f64> {
    let snap = snapshot_of(app, provider);
    if snap.status == "absent" {
        return None;
    }
    let (limit, model) = {
        let c = app.state::<AppState>().cfg.lock().unwrap();
        (c.antigravity_limit.clone(), c.antigravity_model.clone())
    };
    ring_window(provider, &snap.windows, &limit, &model)
        .filter(|w| w.metered())
        .map(|w| w.used.clamp(0.0, 1.0))
}

pub fn resolved_theme(app: &AppHandle) -> &'static str {
    let theme = app.state::<AppState>().cfg.lock().unwrap().theme.clone();
    match theme.as_str() {
        "light" => "light",
        "dark" => "dark",
        _ => {
            if windows_apps_use_light_theme() {
                "light"
            } else {
                "dark"
            }
        }
    }
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn theme_choice(app: &AppHandle) -> Option<Theme> {
    match app.state::<AppState>().cfg.lock().unwrap().theme.as_str() {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        _ => None,
    }
}

#[cfg(windows)]
pub fn windows_apps_use_light_theme() -> bool {
    use windows::core::HSTRING;
    use windows::Win32::System::Registry::{
        RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
    };
    let key = HSTRING::from(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let val = HSTRING::from("AppsUseLightTheme");
    let mut data: u32 = 0;
    let mut size: u32 = 4;
    let ok = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &val,
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    }
    .is_ok();
    ok && data != 0
}

#[cfg(not(windows))]
pub fn windows_apps_use_light_theme() -> bool {
    false
}

pub fn keeps_open(app: &AppHandle) -> bool {
    !app.state::<AppState>().cfg.lock().unwrap().notch_on_hover
}

pub fn toggle_keep_open(app: &AppHandle) {
    {
        let mut c = app.state::<AppState>().cfg.lock().unwrap();
        c.notch_on_hover = !c.notch_on_hover;
        config::save(&c);
    }
    apply_visibility(app);
}

pub fn apply_visibility(app: &AppHandle) {
    app.send_ui(UiEvent::ApplyVisibility);
}

pub fn place_notch(app: &AppHandle) {
    app.send_ui(UiEvent::PlaceNotch);
}

pub fn glide_notch(app: &AppHandle) {
    app.send_ui(UiEvent::GlideNotch);
}

pub fn reset_bar(app: &AppHandle) {
    let stranded = {
        let want = app.state::<AppState>().cfg.lock().unwrap().notch_monitor.clone();
        want.is_some_and(|name| !screens().iter().any(|s| s.name.as_deref() == Some(name.as_str())))
    };
    {
        let mut c = app.state::<AppState>().cfg.lock().unwrap();
        if stranded {
            c.notch_edge = "right".into();
            c.notch_monitor = None;
        }
        let edge = config::edge_or_right(&c.notch_edge);
        c.set_along(&edge, 0.5);
        config::save(&c);
    }
    place_notch(app);
}

// ---------------- Monitors & Placement Math ----------------

#[derive(Clone, Debug)]
pub struct Screen {
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f64,
    pub work: (i32, i32, i32, i32),
}

impl Screen {
    pub fn area(&self) -> (i32, i32, i32, i32) {
        let (x, y, w, h) = self.work;
        if w > 0 && h > 0 {
            (x, y, w, h)
        } else {
            (self.x, self.y, self.w, self.h)
        }
    }
}

#[cfg(windows)]
pub fn screens() -> Vec<Screen> {
    use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
    };
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;

    struct Collected {
        primary: bool,
        screen: Screen,
    }

    unsafe extern "system" fn cb(
        hmon: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let list = &mut *(data.0 as *mut Vec<Collected>);
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmon, &mut info.monitorInfo as *mut _).as_bool() {
            let mut dpi_x = 96u32;
            let mut dpi_y = 96u32;
            let _ = GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            let scale = (dpi_x.max(96) as f64) / 96.0;
            let r = info.monitorInfo.rcMonitor;
            let w = info.monitorInfo.rcWork;
            let raw_name = &info.szDevice;
            let end = raw_name.iter().position(|&c| c == 0).unwrap_or(raw_name.len());
            let dev = String::from_utf16_lossy(&raw_name[..end]);
            let primary = (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
            list.push(Collected {
                primary,
                screen: Screen {
                    name: (!dev.is_empty()).then_some(dev),
                    x: r.left,
                    y: r.top,
                    w: r.right - r.left,
                    h: r.bottom - r.top,
                    scale,
                    work: (w.left, w.top, w.right - w.left, w.bottom - w.top),
                },
            });
        }
        BOOL(1)
    }

    let mut list: Vec<Collected> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(cb), LPARAM(&mut list as *mut _ as isize));
    }
    list.sort_by_key(|c| !c.primary);
    list.into_iter().map(|c| c.screen).collect()
}

#[cfg(not(windows))]
pub fn screens() -> Vec<Screen> {
    Vec::new()
}

pub fn attached_screen(app: &AppHandle) -> Option<Screen> {
    let want = app.state::<AppState>().cfg.lock().unwrap().notch_monitor.clone();
    let list = screens();
    if let Some(name) = want {
        if let Some(s) = list.iter().find(|s| s.name.as_deref() == Some(name.as_str())) {
            return Some(s.clone());
        }
    }
    list.into_iter().next()
}

pub fn target_screen(app: &AppHandle) -> Option<Screen> {
    attached_screen(app).map(crate::fullscreen::apply)
}

pub fn notch_window_size(edge: &str) -> (f64, f64) {
    if config::edge_is_vertical(edge) {
        (NOTCH_W, NOTCH_LONG)
    } else {
        (NOTCH_LONG, NOTCH_LONG)
    }
}

pub fn edge_origin(s: &Screen, edge: &str, ww: i32, wh: i32, ratio: f64) -> (i32, i32) {
    let (ax, ay, aw, ah) = s.area();
    let along = |span: i32, len: i32| -> i32 {
        let v = (span as f64 * ratio - len as f64 / 2.0).round() as i32;
        v.clamp(0, (span - len).max(0))
    };
    match edge {
        "left" => (ax, ay + along(ah, wh)),
        "top" => (ax + along(aw, ww), ay),
        "bottom" => (ax + along(aw, ww), ay + ah - wh),
        _ => (ax + aw - ww, ay + along(ah, wh)),
    }
}

pub fn along_at(pos: i32, len: i32, start: i32, span: i32) -> f64 {
    (((pos - start) as f64 + len as f64 / 2.0) / span.max(1) as f64).clamp(0.0, 1.0)
}

pub fn work_insets(s: &Screen, x: i32, y: i32, ww: i32, wh: i32) -> [i32; 4] {
    let (wx, wy, waw, wah) = s.work;
    [
        (wy - y).clamp(0, wh),
        ((x + ww) - (wx + waw)).clamp(0, ww),
        ((y + wh) - (wy + wah)).clamp(0, wh),
        (wx - x).clamp(0, ww),
    ]
}

pub fn screen_at(list: &[Screen], x: f64, y: f64) -> Option<&Screen> {
    list.iter()
        .find(|s| x >= s.x as f64 && x < (s.x + s.w) as f64 && y >= s.y as f64 && y < (s.y + s.h) as f64)
}

pub fn same_screen(a: &Screen, b: &Screen) -> bool {
    (a.x, a.y, a.w, a.h) == (b.x, b.y, b.w, b.h)
}

#[cfg(windows)]
pub fn left_button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

#[cfg(not(windows))]
pub fn left_button_down() -> bool {
    false
}
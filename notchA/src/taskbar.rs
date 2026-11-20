use crate::core::{self, AppHandle, AppState, Manager, UiEvent};
use crate::usage::UsageSnapshot;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

#[allow(dead_code)]
#[allow(dead_code)]
const DETAILS: &str = "taskbar-details";
#[allow(dead_code)]
#[allow(dead_code)]
const HOVER: &str = "taskbar-hover";
static READY: AtomicBool = AtomicBool::new(false);
static STATUS: Mutex<String> = Mutex::new(String::new());
#[cfg(windows)]
static CHILD: Mutex<ChildState> = Mutex::new(ChildState {
    hwnd: 0,
    parent: 0,
    generation: 0,
    creating: false,
    error: String::new(),
});
#[cfg(windows)]
static APPLY_REVISION: AtomicU64 = AtomicU64::new(0);
#[cfg(windows)]
struct ChildState {
    hwnd: isize,
    parent: isize,
    generation: u64,
    creating: bool,
    error: String,
}

static HOVER_STATE: Mutex<HoverState> = Mutex::new(HoverState {
    item_id: None,
    bounds: None,
    anchor: None,
    epoch: 0,
    show_epoch: 0,
    over_strip: false,
    held: false,
});

struct HoverState {
    item_id: Option<String>,
    bounds: Option<Rect>,
    anchor: Option<HoverAnchor>,
    epoch: u64,
    show_epoch: u64,
    over_strip: bool,
    held: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy)]
struct HoverAnchor {
    strip: Rect,
    screen: Rect,
    x: f64,
    width: f64,
    scale: f64,
}

impl HoverAnchor {
    fn position(self) -> Option<Rect> {
        hover_position(self.strip, self.screen, self.x, self.width, self.scale, 360.0 * self.scale)
    }
}

pub struct StripGeometry {
    pub bounds: Rect,
    pub screen: Rect,
    pub scale: f64,
}

#[derive(Clone, Serialize)]
pub struct Options {
    pub mode: String,
    pub supported: bool,
    pub status: String,
}

#[derive(Serialize)]
pub struct Item {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) provider: String,
    pub(crate) headline_id: Option<String>,
    pub(crate) work_state: String,
    pub(crate) snapshot: UsageSnapshot,
}

const NOTCH_ORDER: [&str; 4] = ["codex", "opencode", "copilot", "gemini"];

pub fn strip_geometry(_app: &AppHandle) -> Option<StripGeometry> {
    #[cfg(windows)]
    {
        use windows::Win32::{
            Foundation::{HWND, RECT},
            Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY},
            UI::{
                HiDpi::GetDpiForWindow,
                WindowsAndMessaging::{GetWindowRect, IsWindow, IsWindowVisible},
            },
        };
        let raw = CHILD.lock().unwrap().hwnd;
        let hwnd = HWND(raw as _);
        unsafe {
            if raw == 0 || !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() {
                return None;
            }
            let mut bounds = RECT::default();
            GetWindowRect(hwnd, &mut bounds).ok()?;
            let mut monitor = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if !GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY), &mut monitor).as_bool() {
                return None;
            }
            let screen = monitor.rcMonitor;
            Some(StripGeometry {
                bounds: Rect { left: bounds.left, top: bounds.top, right: bounds.right, bottom: bounds.bottom },
                screen: Rect { left: screen.left, top: screen.top, right: screen.right, bottom: screen.bottom },
                scale: GetDpiForWindow(hwnd).max(96) as f64 / 96.0,
            })
        }
    }
    #[cfg(not(windows))]
    { None }
}

#[cfg(windows)]
fn native_frame(app: &AppHandle) -> crate::taskbar_model::Frame {
    let items = get_taskbar_items(app.clone());
    let cfg = app.state::<AppState>().cfg.lock().unwrap().clone();
    crate::taskbar_model::from_items(&items, core::resolved_theme(app) != "light", &cfg)
}

#[cfg(windows)]
pub(crate) fn native_destroyed(app: &AppHandle, hwnd: isize) {
    let generation = {
        let mut child = CHILD.lock().unwrap();
        if child.hwnd != hwnd {
            None
        } else {
            child.hwnd = 0;
            child.parent = 0;
            child.creating = false;
            Some(child.generation)
        }
    };
    let Some(generation) = generation else { return };
    READY.store(false, Ordering::Release);
    APPLY_REVISION.fetch_add(1, Ordering::AcqRel);
    core::applog("taskbar child: native window destroyed; waiting for Explorer");
    let app_c = app.clone();
    let _ = app.run_on_main_thread(move || { if CHILD.lock().unwrap().generation == generation { hide_hover_now(&app_c); } });
}

pub fn get_taskbar_options(app: AppHandle) -> Options {
    let cfg = app.state::<AppState>().cfg.lock().unwrap();
    Options {
        mode: cfg.taskbar_mode.clone(),
        supported: cfg!(windows),
        status: STATUS.lock().unwrap().clone(),
    }
}

pub fn set_taskbar_options(app: AppHandle, mode: String) -> Result<Options, String> {
    if !["off", "left", "right"].contains(&mode.as_str()) {
        return Err("Invalid taskbar position".into());
    }
    if mode != "off" && !cfg!(windows) {
        return Err("Taskbar display requires Windows".into());
    }
    {
        let st = app.state::<AppState>();
        let mut cfg = st.cfg.lock().unwrap();
        let mut next = cfg.clone();
        if mode != "off" {
            next.notch_visible = false;
            next.tray_visible = true;
        } else if next.taskbar_mode != "off" {
            next.notch_visible = true;
        }
        next.taskbar_mode = mode;
        crate::config::save_checked(&next)?;
        *cfg = next;
    }
    hide_hover_now(&app);
    core::apply_visibility(&app);
    Ok(get_taskbar_options(app))
}

pub fn get_taskbar_items(app: AppHandle) -> Vec<Item> {
    let st = app.state::<AppState>();
    let cfg = st.cfg.lock().unwrap().clone();
    let usage = st.usage.lock().unwrap().clone();
    let mut items = Vec::new();
    let profiles = crate::claude_accounts::profiles(&cfg);
    for p in profiles
        .into_iter()
        .filter(|p| p.enabled && (cfg.notch_slots.is_empty() || cfg.notch_slots.iter().any(|s| s.provider == "claude")))
    {
        let snapshot = if let Some(a) = usage.accounts.iter().find(|a| a.id == p.id) {
            UsageSnapshot {
                status: a.status.clone(),
                windows: a.windows.clone(),
                fetched_at: a.fetched_at,
                note: a.note.clone(),
                backoff_until: a.backoff_until,
                ..Default::default()
            }
        } else if p.id == "default" && usage.accounts.is_empty() {
            let mut snap = usage.clone();
            snap.windows.retain(|w| !w.id.contains('@'));
            snap
        } else {
            UsageSnapshot { status: "needsAuth".into(), ..Default::default() }
        };
        let headline_id = core::ring_window("claude", &snapshot.windows, "automatic", "gemini").map(|w| w.id.clone());
        let work_state = crate::taskbar_activity::work_state(&app, "claude", Some(&p.id));
        items.push(Item {
            id: format!("claude@{}", p.id),
            headline_id,
            work_state,
            name: p.name,
            provider: "claude".into(),
            snapshot,
        });
    }
    for id in NOTCH_ORDER {
        if !cfg.notch_slots.is_empty() && !cfg.notch_slots.iter().any(|s| s.provider == id) {
            continue;
        }
        let snapshot = core::snapshot_of(&app, id);
        if snapshot.status == "absent" {
            continue;
        }
        let headline_id = core::ring_window(id, &snapshot.windows, &cfg.antigravity_limit, &cfg.antigravity_model)
            .map(|w| w.id.clone());
        let work_state = crate::taskbar_activity::work_state(&app, id, None);
        items.push(Item {
            id: id.into(),
            headline_id,
            work_state,
            name: core::provider_label(id).into(),
            provider: id.into(),
            snapshot,
        });
    }
    items
}

pub fn current_hover_info() -> Option<(String, Rect, bool, f64)> {
    let state = HOVER_STATE.lock().unwrap();
    let id = state.item_id.clone()?;
    let bounds = state.bounds?;
    let anchor = state.anchor?;
    let below = bounds.top >= anchor.strip.bottom;
    let ring_x = anchor.strip.left as f64
        + (anchor.x + anchor.width / 2.0).clamp(0.0, (anchor.strip.right - anchor.strip.left) as f64);
    Some((id, bounds, below, ring_x - bounds.left as f64))
}

pub fn show_taskbar_hover(app: AppHandle, id: String, anchor_x: f64, anchor_width: f64) -> Result<(), String> {
    if !anchor_x.is_finite() || !anchor_width.is_finite() || anchor_width <= 0.0 {
        return Err("Invalid taskbar hover anchor".into());
    }
    if !get_taskbar_items(app.clone()).iter().any(|item| item.id == id) {
        return Err("Unknown taskbar account".into());
    }
    let Some(geom) = strip_geometry(&app) else {
        return Ok(());
    };
    let anchor = HoverAnchor {
        strip: geom.bounds,
        screen: geom.screen,
        x: anchor_x,
        width: anchor_width,
        scale: geom.scale,
    };
    let Some(bounds) = anchor.position() else {
        return Ok(());
    };
    {
        let mut state = HOVER_STATE.lock().unwrap();
        state.epoch = state.epoch.wrapping_add(1);
        state.show_epoch = state.epoch;
        state.item_id = Some(id.clone());
        state.anchor = Some(anchor);
        state.bounds = Some(bounds);
        state.over_strip = true;
        state.held = false;
    }
    app.send_ui(UiEvent::ShowTaskbarHover { id, x: anchor_x, width: anchor_width });
    Ok(())
}

pub fn hide_taskbar_hover(app: AppHandle) {
    let epoch = {
        let mut state = HOVER_STATE.lock().unwrap();
        state.over_strip = false;
        state.epoch = state.epoch.wrapping_add(1);
        state.epoch
    };
    schedule_hover_hide(app, epoch);
}

pub(crate) fn hover_open() -> bool {
    HOVER_STATE.lock().unwrap().item_id.is_some()
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn hold_taskbar_hover(app: AppHandle, held: bool) {
    let epoch = {
        let mut state = HOVER_STATE.lock().unwrap();
        if state.item_id.is_none() {
            return;
        }
        state.held = held;
        state.epoch = state.epoch.wrapping_add(1);
        state.epoch
    };
    if !held {
        schedule_hover_hide(app, epoch);
    }
}

fn schedule_hover_hide(app: AppHandle, epoch: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(180));
        let hide = {
            let mut state = HOVER_STATE.lock().unwrap();
            if state.epoch != epoch || state.over_strip || state.held {
                false
            } else {
                state.item_id = None;
                state.bounds = None;
                state.anchor = None;
                true
            }
        };
        if hide {
            app.send_ui(UiEvent::HideTaskbarHover);
        }
    });
}

fn hide_hover_now(app: &AppHandle) {
    {
        let mut state = HOVER_STATE.lock().unwrap();
        state.epoch = state.epoch.wrapping_add(1);
        state.item_id = None;
        state.show_epoch = state.epoch;
        state.bounds = None;
        state.anchor = None;
        state.over_strip = false;
        state.held = false;
    }
    app.send_ui(UiEvent::HideTaskbarHover);
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn open_taskbar_details(app: AppHandle) {
    hide_hover_now(&app);
    app.send_ui(UiEvent::OpenTaskbarDetails);
}

pub fn start(app: &AppHandle) {
    #[cfg(windows)]
    {
        READY.store(false, Ordering::Release);
        let handle = app.clone();
        std::thread::spawn(move || native::watch(handle));
    }
    #[cfg(not(windows))]
    let _ = app;
}

const CARD_WIDTH: f64 = 246.0;
const TAIL_GAP: f64 = 10.5;

fn hover_position(strip: Rect, screen: Rect, anchor_x: f64, anchor_width: f64, scale: f64, content_height: f64) -> Option<Rect> {
    if !anchor_x.is_finite() || !anchor_width.is_finite() || anchor_width <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let gap = (8.0 * scale).round().max(1.0) as i32;
    let tail_gap = (TAIL_GAP * scale).round().max(1.0) as i32;
    let available_width = screen.right - screen.left - 2 * gap;
    let available_height = screen.bottom - screen.top - 2 * gap;
    if available_width <= 0 || available_height <= 0 || strip.right <= strip.left || strip.bottom <= strip.top {
        return None;
    }
    let width = ((CARD_WIDTH * scale).round() as i32).min(available_width);
    let height = (content_height.round() as i32).max(1).min(available_height);
    let center = strip.left as f64 + (anchor_x + anchor_width / 2.0).clamp(0.0, (strip.right - strip.left) as f64);
    let x = (center.round() as i32 - width / 2).clamp(screen.left + gap, screen.right - width - gap);
    let above = strip.top - height - tail_gap;
    let y = if above >= screen.top + gap { above } else { strip.bottom + tail_gap };
    let y = y.clamp(screen.top + gap, screen.bottom - height - gap);
    Some(Rect { left: x, top: y, right: x + width, bottom: y + height })
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ChildDisposition { Disabled, Waiting, Create, Recreate, Keep }

fn child_disposition(enabled: bool, parent: Option<isize>, child_parent: Option<isize>) -> ChildDisposition {
    if !enabled { return ChildDisposition::Disabled; }
    let Some(parent) = parent else { return ChildDisposition::Waiting; };
    match child_parent {
        None => ChildDisposition::Create,
        Some(current) if current != parent => ChildDisposition::Recreate,
        Some(_) => ChildDisposition::Keep,
    }
}

fn parent_client_rect(screen: Rect, origin_x: i32, origin_y: i32) -> Rect {
    Rect { left: screen.left - origin_x, top: screen.top - origin_y, right: screen.right - origin_x, bottom: screen.bottom - origin_y }
}

fn needs_child_placement(desired: Rect, actual: Option<Rect>, visible: bool) -> bool {
    !visible || actual != Some(desired)
}

fn client_covers_monitor(client: Rect, monitor: Rect) -> bool {
    client.left <= monitor.left + 1 && client.top <= monitor.top + 1
        && client.right >= monitor.right - 1 && client.bottom >= monitor.bottom - 1
}

fn shell_surface(class: &str, process: &str) -> bool {
    ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "NotifyIconOverflowWindow"].contains(&class)
        || ["startmenuexperiencehost.exe", "searchhost.exe", "searchapp.exe", "shellexperiencehost.exe"].contains(&process)
        || (process == "explorer.exe" && ["XamlExplorerHostIslandWindow", "MultitaskingViewFrame"].contains(&class))
}

fn position_still_free(bar: Rect, occupied: &[Rect], position: Rect, gap: i32) -> bool {
    position.left >= bar.left + gap && position.right <= bar.right - gap
        && position.top == bar.top + 3 && position.bottom == bar.bottom - 3
        && !occupied.iter().any(|r| r.bottom > bar.top && r.top < bar.bottom
            && r.right + gap > position.left && r.left - gap < position.right)
}

fn refresh_position(bar: Rect, occupied: Option<&[Rect]>, previous: Option<Rect>, right: bool, wanted: i32, gap: i32, minimum: i32) -> Option<Rect> {
    let Some(occupied) = occupied else { return previous; };
    let candidate = free_position(bar, occupied, right, wanted, gap, minimum);
    if let (Some(old), Some(next)) = (previous, candidate) {
        if old.right - old.left == next.right - next.left && position_still_free(bar, occupied, old, gap) {
            return Some(old);
        }
    }
    candidate
}

fn free_position(bar: Rect, occupied: &[Rect], right: bool, wanted: i32, gap: i32, minimum: i32) -> Option<Rect> {
    if bar.right - bar.left < minimum || bar.bottom <= bar.top || bar.bottom - bar.top > bar.right - bar.left {
        return None;
    }
    let mut blocks: Vec<(i32, i32)> = occupied
        .iter()
        .filter(|r| r.bottom > bar.top && r.top < bar.bottom && r.right > bar.left && r.left < bar.right)
        .map(|r| ((r.left - gap).max(bar.left), (r.right + gap).min(bar.right)))
        .collect();
    blocks.sort();
    let mut gaps = Vec::new();
    let mut edge = bar.left + gap;
    for (left, stop) in blocks {
        if left - edge >= minimum {
            gaps.push((edge, left));
        }
        edge = edge.max(stop);
    }
    if bar.right - gap - edge >= minimum {
        gaps.push((edge, bar.right - gap));
    }
    let middle = bar.left + (bar.right - bar.left) / 2;
    gaps = gaps
        .into_iter()
        .map(|(left, stop)| if right { (left.max(middle), stop) } else { (left, stop.min(middle)) })
        .filter(|(left, stop)| stop - left >= minimum)
        .collect();
    if right {
        gaps.reverse();
    }
    let (left, stop) = gaps.into_iter().next()?;
    let width = wanted.min(stop - left);
    let x = if right { stop - width } else { left };
    Some(Rect { left: x, right: x + width, top: bar.top + 3, bottom: bar.bottom - 3 })
}

#[cfg(windows)]
mod native {
    use super::*;
    use windows::{
        core::{Interface, PWSTR},
        Win32::{
            Foundation::{CloseHandle, HWND, POINT, RECT},
            Graphics::Gdi::{ClientToScreen, GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY},
            System::{
                Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
                Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION},
            },
            UI::{Accessibility::*, WindowsAndMessaging::*},
        },
    };

    static LAST_NATIVE_ERROR: Mutex<String> = Mutex::new(String::new());
    fn log_native_error(operation: &str, error: &windows::core::Error) {
        let line = format!("taskbar child: {operation} failed, code=0x{:08x}", error.code().0 as u32);
        let mut last = LAST_NATIVE_ERROR.lock().unwrap();
        if *last != line {
            core::applog(&line);
            *last = line;
        }
    }
    fn child_error(message: &str) {
        READY.store(false, Ordering::Release);
        let mut child = CHILD.lock().unwrap();
        child.error = message.into();
        child.creating = false;
    }
    fn ensure_child(app: &AppHandle, parent: isize) {
        {
            let mut child = CHILD.lock().unwrap();
            if child.creating { return; }
            child.creating = true;
        }
        let handle = app.clone();
        if app.run_on_main_thread(move || {
            let bar = HWND(parent as _);
            if handle.state::<AppState>().cfg.lock().unwrap().taskbar_mode == "off" || !unsafe { IsWindow(bar).as_bool() } {
                CHILD.lock().unwrap().creating = false;
                return;
            }
            if let Some(hwnd) = crate::taskbar_native::hwnd() {
                if unsafe { IsWindow(hwnd).as_bool() && GetParent(hwnd).is_ok_and(|p| p == bar) } {
                    let mut child = CHILD.lock().unwrap();
                    child.hwnd = hwnd.0 as isize;
                    child.parent = parent;
                    child.creating = false;
                    READY.store(true, Ordering::Release);
                    return;
                }
                crate::taskbar_native::destroy();
            }
            READY.store(false, Ordering::Release);
            {
                let mut child = CHILD.lock().unwrap();
                child.generation = child.generation.wrapping_add(1);
                child.parent = parent;
                child.hwnd = 0;
                child.creating = true;
            }
            let scale = unsafe { windows::Win32::UI::HiDpi::GetDpiForWindow(bar) }.max(96) as f32 / 96.0;
            let frame = native_frame(&handle);
            let width = (280.0 * scale).round() as u32;
            let height = (42.0 * scale).round() as u32;
            match crate::taskbar_native::create(handle.clone(), bar, width, height, scale, frame) {
                Ok(hwnd) => {
                    let mut child = CHILD.lock().unwrap();
                    child.hwnd = hwnd;
                    child.parent = parent;
                    child.error.clear();
                    child.creating = false;
                    READY.store(true, Ordering::Release);
                }
                Err(reason) => {
                    core::applog(&format!("taskbar child: native creation failed: {reason}"));
                    child_error("Unable to create taskbar child window.");
                }
            }
        }).is_err() {
            child_error("Unable to schedule taskbar child creation.");
        }
    }
    fn rect(hwnd: HWND) -> Option<Rect> {
        let mut r = RECT::default();
        unsafe { GetWindowRect(hwnd, &mut r).ok()?; }
        Some(Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom })
    }
    fn class(hwnd: HWND) -> String {
        let mut text = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, &mut text) };
        String::from_utf16_lossy(&text[..len.max(0) as usize])
    }
    fn client_rect(hwnd: HWND) -> Option<Rect> {
        let mut client = RECT::default();
        let mut origin = POINT::default();
        unsafe {
            GetClientRect(hwnd, &mut client).ok()?;
            if !ClientToScreen(hwnd, &mut origin).as_bool() { return None; }
        }
        Some(Rect {
            left: origin.x,
            top: origin.y,
            right: origin.x + client.right - client.left,
            bottom: origin.y + client.bottom - client.top,
        })
    }
    fn foreground_process(hwnd: HWND, cache: &mut Option<(isize, u32, String)>) -> &str {
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
        if cache.as_ref().is_none_or(|(window, process, _)| *window != hwnd.0 as isize || *process != pid) {
            let mut path = [0u16; 1024];
            let mut size = path.len() as u32;
            let mut name = String::new();
            unsafe {
                if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                    if QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), PWSTR(path.as_mut_ptr()), &mut size).is_ok() {
                        let full = String::from_utf16_lossy(&path[..size as usize]);
                        name = full.rsplit('\\').next().unwrap_or("").to_ascii_lowercase();
                    }
                    let _ = CloseHandle(process);
                }
            }
            *cache = Some((hwnd.0 as isize, pid, name));
        }
        &cache.as_ref().unwrap().2
    }
    fn desktop_visible(bar: HWND, bounds: Rect, foreground: HWND, process_cache: &mut Option<(isize, u32, String)>) -> Result<(), &'static str> {
        unsafe {
            if !IsWindowVisible(bar).as_bool() { return Err("hidden_taskbar"); }
            let mut monitor = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
            if !GetMonitorInfoW(MonitorFromWindow(bar, MONITOR_DEFAULTTOPRIMARY), &mut monitor).as_bool() { return Err("monitor_unavailable"); }
            let m = monitor.rcMonitor;
            if bounds.top < m.top || bounds.bottom > m.bottom || bounds.bottom - bounds.top < 20 { return Err("autohidden_taskbar"); }
            let kind = class(foreground);
            if shell_surface(&kind, "") { return Ok(()); }
            let screen = Rect { left: m.left, top: m.top, right: m.right, bottom: m.bottom };
            let client = client_rect(foreground).or_else(|| {
                let style = GetWindowLongPtrW(foreground, GWL_STYLE) as u32;
                if style & (WS_CAPTION.0 | WS_THICKFRAME.0) == 0 { rect(foreground) } else { None }
            });
            if !client.is_some_and(|r| client_covers_monitor(r, screen)) { return Ok(()); }
            if shell_surface(&kind, foreground_process(foreground, process_cache)) { Ok(()) } else { Err("fullscreen_application") }
        }
    }
    fn occupied(uia: &IUIAutomation, bar: HWND, bounds: Rect, strip: HWND) -> Option<Vec<Rect>> {
        unsafe {
            let root = uia.ElementFromHandle(bar).ok()?;
            let walker = uia.RawViewWalker().ok()?;
            let own_root = if strip.0.is_null() { None } else { uia.ElementFromHandle(strip).ok() };
            let mut pending = vec![walker.GetFirstChildElement(&root).ok()?];
            let mut visited = 0;
            let mut occupied = Vec::new();
            while let Some(e) = pending.pop() {
                visited += 1;
                if visited > 1600 { return None; }
                if let Ok(sibling) = walker.GetNextSiblingElement(&e) { pending.push(sibling); }
                let native = e.CurrentNativeWindowHandle().unwrap_or_default();
                let ours = (!strip.0.is_null() && (native == strip || (!native.0.is_null() && IsChild(strip, native).as_bool())))
                    || e.CurrentProcessId().unwrap_or(0) == std::process::id() as i32
                    || own_root.as_ref().is_some_and(|own| uia.CompareElements(&e, own).unwrap_or_default().as_bool());
                if ours || e.CurrentIsOffscreen().unwrap_or_default().as_bool() { continue; }
                if let Ok(child) = walker.GetFirstChildElement(&e) { pending.push(child); }
                let kind = e.CurrentControlType().ok()?;
                if ![UIA_ButtonControlTypeId, UIA_ListItemControlTypeId, UIA_MenuItemControlTypeId,
                    UIA_TextControlTypeId, UIA_ImageControlTypeId, UIA_HyperlinkControlTypeId].contains(&kind) { continue; }
                let r = e.CurrentBoundingRectangle().ok()?;
                if r.right > r.left && r.bottom > r.top && r.right - r.left < (bounds.right - bounds.left) / 2 {
                    occupied.push(Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom });
                }
            }
            if let Ok(tray) = FindWindowExW(bar, None, windows::core::w!("TrayNotifyWnd"), None) {
                if let Some(mut r) = rect(tray) { r.right = bounds.right; occupied.push(r); }
            }
            if occupied.is_empty() { return None; }
            Some(occupied)
        }
    }
    pub fn watch(app: AppHandle) {
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() };
        if !initialized { *STATUS.lock().unwrap() = "Windows taskbar accessibility is unavailable".into(); return; }
        let uia: windows::core::Result<IUIAutomation> = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) };
        let Ok(uia) = uia else { unsafe { CoUninitialize(); } return; };
        if let Ok(uia2) = uia.cast::<IUIAutomation2>() { unsafe { let _ = uia2.SetConnectionTimeout(400); let _ = uia2.SetTransactionTimeout(400); } }
        use std::time::{Duration, Instant};
        let mut previous = (0, None);
        let mut layout_key = None;
        let mut cached_position = None;
        let mut layout_status = String::new();
        let mut pending_change = None::<Instant>;
        let mut last_verified = None::<Instant>;
        let mut process_cache = None;
        let mut last_foreground = 0;
        let mut next_layout_read = Instant::now();
        let mut next_create = Instant::now();
        let mut next_z_check = Instant::now();
        let mut next_frame = Instant::now();
        let mut last_frame = None;
        let mut wanted_width = 0;
        let mut last_wanted = 0;
        loop {
            let now = Instant::now();
            let foreground = unsafe { GetForegroundWindow() };
            let foreground_changed = last_foreground != foreground.0 as isize;
            last_foreground = foreground.0 as isize;
            let reassert = foreground_changed || now >= next_z_check;
            if reassert { next_z_check = now + Duration::from_secs(1); }
            let cfg = app.state::<AppState>().cfg.lock().unwrap().clone();
            let enabled = cfg.taskbar_mode != "off";
            if !enabled {
                last_frame = None;
                next_frame = now;
            }
            let bar = if enabled { unsafe { FindWindowW(windows::core::w!("Shell_TrayWnd"), None) }.ok() } else { None };
            let (child_hwnd, generation, creation_error) = {
                let child = CHILD.lock().unwrap();
                (child.hwnd, child.generation, child.error.clone())
            };
            let child_parent = if child_hwnd != 0 && unsafe { IsWindow(HWND(child_hwnd as _)).as_bool() } {
                unsafe { GetParent(HWND(child_hwnd as _)) }.ok().map(|p| p.0 as isize)
            } else { None };
            let disposition = child_disposition(enabled, bar.map(|w| w.0 as isize), child_parent);
            if matches!(disposition, ChildDisposition::Create | ChildDisposition::Recreate) && now >= next_create {
                if let Some(bar) = bar { ensure_child(&app, bar.0 as isize); }
                next_create = now + Duration::from_secs(if creation_error.is_empty() { 2 } else { 5 });
            }
            let layout = if disposition == ChildDisposition::Keep && READY.load(Ordering::Acquire) {
                bar.and_then(|bar| {
                    let bounds = rect(bar)?;
                    desktop_visible(bar, bounds, foreground, &mut process_cache).ok().map(|_| (bar, bounds))
                })
            } else { None };
            let position = if let Some((bar, bounds)) = layout {
                let scale = unsafe { windows::Win32::UI::HiDpi::GetDpiForWindow(bar) }.max(96) as f64 / 96.0;
                let wanted = if wanted_width > 0 { wanted_width } else { (100.0 * scale) as i32 };
                let key = (bar.0 as isize, bounds, cfg.taskbar_mode == "right", scale.to_bits(), generation);
                if last_wanted != wanted { last_wanted = wanted; next_layout_read = now; }
                if layout_key != Some(key) {
                    layout_key = Some(key);
                    cached_position = None;
                    layout_status.clear();
                    pending_change = None;
                    last_verified = None;
                    next_layout_read = now;
                }
                if now >= next_layout_read {
                    if let Some(blocks) = occupied(&uia, bar, bounds, HWND(child_hwnd as _)) {
                        let candidate = refresh_position(bounds, Some(&blocks), cached_position, cfg.taskbar_mode == "right",
                            wanted, (8.0 * scale) as i32, wanted.min((100.0 * scale) as i32));
                        last_verified = Some(now);
                        let settling = cached_position.is_some() && candidate != cached_position
                            && pending_change.is_none_or(|first| now.duration_since(first) < Duration::from_millis(300));
                        if settling {
                            if pending_change.is_none() { pending_change = Some(now); }
                            next_layout_read = now + Duration::from_millis(300);
                        } else {
                            cached_position = candidate;
                            pending_change = None;
                            next_layout_read = now + Duration::from_secs(if candidate.is_some() { 2 } else { 1 });
                            layout_status = if candidate.is_none() { "No free space on the taskbar.".into() } else { String::new() };
                        }
                    } else {
                        if last_verified.is_none_or(|last| now.duration_since(last) >= Duration::from_secs(6)) {
                            cached_position = None;
                            pending_change = None;
                            layout_status = "Taskbar layout unavailable.".into();
                        }
                        next_layout_read = now + Duration::from_millis(500);
                    }
                }
                cached_position
            } else {
                layout_key = None;
                cached_position = None;
                layout_status.clear();
                pending_change = None;
                last_verified = None;
                next_layout_read = now;
                None
            };
            *STATUS.lock().unwrap() = layout_status.clone();
            if enabled && child_parent.is_some() && READY.load(Ordering::Acquire) && now >= next_frame {
                next_frame = now + Duration::from_millis(500);
                if let Some(bar) = bar {
                    let scale = unsafe { windows::Win32::UI::HiDpi::GetDpiForWindow(bar) }.max(96) as f32 / 96.0;
                    let frame = native_frame(&app);
                    let height = rect(bar).map(|r| r.bottom - r.top - 6).unwrap_or(0) as f32;
                    wanted_width = crate::taskbar_model::content_width(&frame, height, scale) as i32;
                    let key = (generation, scale.to_bits(), frame.clone());
                    if last_frame.as_ref() != Some(&key) {
                        last_frame = Some(key);
                        let handle = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            if CHILD.lock().unwrap().generation != generation { return; }
                            if handle.state::<AppState>().cfg.lock().unwrap().taskbar_mode != "off" {
                                crate::taskbar_native::update(frame, scale);
                            }
                        });
                    }
                }
            }
            let moved = position != previous.1 || generation != previous.0;
            previous = (generation, position);
            if moved || (position.is_some() && reassert) {
                let revision = APPLY_REVISION.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
                let parent = bar.map(|w| w.0 as isize);
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || {
                    if APPLY_REVISION.load(Ordering::Acquire) != revision || CHILD.lock().unwrap().generation != generation { return; }
                    let Some(hwnd) = crate::taskbar_native::hwnd() else { return; };
                    if let Some(r) = position {
                        if handle.state::<AppState>().cfg.lock().unwrap().taskbar_mode == "off" { return; }
                        let Some(parent) = parent else { return; };
                        let bar = HWND(parent as _);
                        if !unsafe { IsWindow(bar).as_bool() && GetParent(hwnd).is_ok_and(|p| p == bar) } { return; }
                        let mut origin = POINT::default();
                        if !unsafe { ClientToScreen(bar, &mut origin).as_bool() } {
                            log_native_error("ClientToScreen", &windows::core::Error::from_win32());
                            return;
                        }
                        let local = parent_client_rect(r, origin.x, origin.y);
                        let was_visible = unsafe { IsWindowVisible(hwnd).as_bool() };
                        let actual = rect(hwnd);
                        if needs_child_placement(r, actual, was_visible) {
                            if actual != Some(r) { hide_hover_now(&handle); }
                            unsafe {
                                let _ = SetWindowPos(hwnd, HWND_TOP, local.left, local.top, local.right - local.left, local.bottom - local.top, SWP_NOACTIVATE);
                                let _ = SetWindowPos(hwnd, HWND_TOP, local.left, local.top, local.right - local.left, local.bottom - local.top, SWP_NOACTIVATE | SWP_SHOWWINDOW);
                            }
                        } else {
                            unsafe { let _ = SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE); }
                        }
                    } else {
                        hide_hover_now(&handle);
                        unsafe { let _ = ShowWindow(hwnd, SW_HIDE); }
                    }
                });
            }
            std::thread::sleep(Duration::from_millis(150));
        }
    }
}
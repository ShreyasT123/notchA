use crate::core::{self, AppHandle, AppState, Manager, UiEvent};
use crate::reset_watch::ResetEvent;
use crate::{config, glyphs, traymenu};
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::System::Diagnostics::Debug::MessageBeep;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::MB_ICONASTERISK;

pub const CARD_W: f64 = 280.0;
pub const CARD_H: f64 = 150.0;
const SHOW_FOR: Duration = Duration::from_secs(6);
pub const PILL_DEPTH: f64 = 70.0;

#[derive(Default)]
struct QueueState {
    pending: VecDeque<ResetEvent>,
    running: bool,
    dismissed: bool,
    active_token: u64,
    active_card: Option<CardPayload>,
}

#[derive(Default)]
pub struct AlertQueue {
    state: Mutex<QueueState>,
    changed: Condvar,
}

#[derive(Clone, Debug)]
pub struct CardPayload {
    pub edge: String,
    pub attached: bool,
    pub glyph: glyphs::Glyph,
    pub token: u64,
    pub title: String,
    pub subtitle: String,
    pub status: String,
    pub next: String,
    pub pos: (i32, i32),
    pub size: (u32, u32),
}

pub fn active_card(app: &AppHandle) -> Option<CardPayload> {
    app.state::<AppState>()
        .reset_alerts
        .state
        .lock()
        .unwrap()
        .active_card
        .clone()
}

pub fn enqueue(app: &AppHandle, event: ResetEvent) {
    let enabled = app.state::<AppState>().cfg.lock().unwrap().reset_notifications;
    if enabled {
        enqueue_unchecked(app, event);
    }
}

fn enqueue_unchecked(app: &AppHandle, event: ResetEvent) {
    let queue = &app.state::<AppState>().reset_alerts;
    let mut state = queue.state.lock().unwrap();
    state.pending.push_back(event);
    if state.running {
        return;
    }
    state.running = true;
    let app = app.clone();
    std::thread::spawn(move || run(app));
}

fn run(app: AppHandle) {
    loop {
        let (event, token) = {
            let queue = &app.state::<AppState>().reset_alerts;
            let mut state = queue.state.lock().unwrap();
            let Some(event) = state.pending.pop_front() else {
                state.running = false;
                state.active_token = 0;
                state.active_card = None;
                app.send_ui(UiEvent::HideResetAlert);
                return;
            };
            state.active_token = state.active_token.wrapping_add(1).max(1);
            state.dismissed = false;
            (event, state.active_token)
        };

        if let Some(payload) = build_payload(&app, event, token) {
            {
                let queue = &app.state::<AppState>().reset_alerts;
                queue.state.lock().unwrap().active_card = Some(payload);
            }
            app.send_ui(UiEvent::ShowResetAlert);
            if app.state::<AppState>().cfg.lock().unwrap().reset_notification_sound {
                play_notification_sound();
            }
            let queue = &app.state::<AppState>().reset_alerts;
            let state = queue.state.lock().unwrap();
            let _ = queue.changed.wait_timeout_while(state, SHOW_FOR, |s| !s.dismissed);
        }
        {
            let queue = &app.state::<AppState>().reset_alerts;
            queue.state.lock().unwrap().active_card = None;
        }
        app.send_ui(UiEvent::HideResetAlert);
    }
}

fn build_payload(app: &AppHandle, event: ResetEvent, token: u64) -> Option<CardPayload> {
    let screen = core::target_screen(app)?;
    let (edge, cfg_scale, visible) = {
        let cfg = app.state::<AppState>().cfg.lock().unwrap();
        (
            config::edge_or_right(&cfg.notch_edge),
            config::snap_scale(cfg.scale),
            cfg.notch_visible,
        )
    };
    let density = screen.scale * cfg_scale;
    let (ax, ay, aw, ah) = screen.area();
    let width = ((CARD_W * density).round() as u32).min(aw.max(1) as u32);
    let height = ((CARD_H * density).round() as u32).min(ah.max(1) as u32);

    let notch_geom = app.get_webview_window("notch").and_then(|w| {
        let p = w.outer_position().ok()?;
        let s = w.outer_size().ok()?;
        Some((p.x as i32, p.y as i32, s.width as i32, s.height as i32))
    });

    let (x, y) = match notch_geom {
        Some((nx, ny, nw, nh)) => {
            let pill = (PILL_DEPTH * density).round() as i32;
            match edge.as_str() {
                "left" => (nx + pill, ny + nh / 2 - height as i32 / 2),
                "top" => (nx + nw / 2 - width as i32 / 2, ny + pill),
                "bottom" => (nx + nw / 2 - width as i32 / 2, ny + nh - pill - height as i32),
                _ => (nx + nw - pill - width as i32, ny + nh / 2 - height as i32 / 2),
            }
        }
        None => {
            let inset = ((if visible { 100.0 } else { 8.0 }) * density).round() as i32;
            match edge.as_str() {
                "left" => (ax + inset, ay + (ah - height as i32) / 2),
                "top" => (ax + (aw - width as i32) / 2, ay + inset),
                "bottom" => (ax + (aw - width as i32) / 2, ay + ah - height as i32 - inset),
                _ => (ax + aw - width as i32 - inset, ay + (ah - height as i32) / 2),
            }
        }
    };
    let x = x.clamp(ax, ax + aw - width as i32);
    let y = y.clamp(ay, ay + ah - height as i32);

    let glyph = app
        .state::<AppState>()
        .glyphs
        .lock()
        .unwrap()
        .get(&event.provider)
        .cloned()
        .unwrap_or_default();
    let now = core::now_ms();
    let provider_name = core::provider_label(&event.provider);
    let window_label = traymenu::label(&event.window_label, "en");

    Some(CardPayload {
        edge,
        attached: visible,
        glyph,
        token,
        title: format!("{provider_name} renewed"),
        subtitle: format!("{window_label} renewed"),
        status: format!("Quota available · {}%", traymenu::pct(event.used_fraction)),
        next: event
            .next_reset_at
            .map(|at| traymenu::reset_text(at, now, "en"))
            .unwrap_or_default(),
        pos: (x, y),
        size: (width, height),
    })
}

#[cfg(windows)]
fn play_notification_sound() {
    let _ = unsafe { MessageBeep(MB_ICONASTERISK) };
}

#[cfg(not(windows))]
fn play_notification_sound() {}

pub fn disable(app: &AppHandle) {
    let queue = &app.state::<AppState>().reset_alerts;
    let mut state = queue.state.lock().unwrap();
    state.pending.clear();
    state.dismissed = true;
    state.active_card = None;
    queue.changed.notify_all();
}

pub fn dismiss_reset_alert(app: &AppHandle, token: u64) {
    let queue = &app.state::<AppState>().reset_alerts;
    let mut state = queue.state.lock().unwrap();
    if state.active_token == token {
        state.dismissed = true;
        queue.changed.notify_all();
    }
}

pub fn preview_reset_alert(app: &AppHandle) -> Result<(), String> {
    if core::target_screen(app).is_none() {
        return Err("No display available for the reset card".into());
    }
    let now = core::now_ms();
    enqueue_unchecked(
        app,
        ResetEvent {
            provider: "codex".into(),
            window_label: "5h limit".into(),
            used_fraction: 0.0,
            next_reset_at: Some(now + 300 * 60_000),
        },
    );
    Ok(())
}
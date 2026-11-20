//! Native Win32 window management, hit-testing, frame pacing, and input handling
//! for the Codenotch desktop overlay.
//! Replaces all webview/iced windowing overhead with pure OS primitives.
#![cfg(windows)]

use crate::{
    config,
    core::{self, AppHandle, AppState, Manager},
    motion::{self, Anim, Curve, Tween, FOLD, CARD, EASE_IN_OUT, EASE_OUT},
    notch_render::{CellRenderData, D2dPalette, NotchFrameData, NotchRenderer},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{
            Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F},
            Dwm::DwmFlush,
            Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, ScreenToClient},
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{TrackMouseEvent, TRACKMOUSEEVENT, TME_LEAVE},
            WindowsAndMessaging::{
                self as wm, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
                GetCursorPos, GetMessageW, GetWindowRect, LoadCursorW, PeekMessageW,
                PostQuitMessage, RegisterClassExW, SetWindowPos, ShowWindow,
                TranslateMessage, CS_HREDRAW, CS_VREDRAW, HMENU, HTCLIENT, HTTRANSPARENT,
                IDC_ARROW, IDC_HAND, PM_REMOVE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
                SWP_NOZORDER, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WM_DPICHANGED,
                WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
                WM_NCHITTEST, WM_PAINT, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW, WS_EX_NOACTIVATE,
                WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
            },
        },
    },
};

const WM_MOUSELEAVE: u32 = 0x02A3;

static WINDOW_CLASS: OnceLock<Result<usize, String>> = OnceLock::new();
static RUNNING: AtomicBool = AtomicBool::new(false);

thread_local! {
    static NOTCH_STATE: RefCell<Option<Rc<NotchState>>> = const { RefCell::new(None) };
}

pub struct NotchState {
    pub app: AppHandle,
    pub hwnd: Cell<HWND>,
    pub renderer: RefCell<Option<NotchRenderer>>,
    pub anim: RefCell<Anim>,
    pub width: Cell<u32>,
    pub height: Cell<u32>,
    pub scale: Cell<f32>,
    pub edge: RefCell<String>,
    pub dark: Cell<bool>,
    pub folded: Cell<bool>,
    pub hover_cell: Cell<Option<usize>>,
    pub orb_hover: Cell<bool>,
    pub card_open: Cell<bool>,
    pub card_cell: Cell<Option<usize>>,
    pub card_glide: RefCell<Tween>,
    pub card_pos: Cell<D2D_POINT_2F>,
    pub pressing: Cell<Option<usize>>,
    pub refresh_starts: RefCell<HashMap<String, Instant>>,
    pub started: Instant,
    pub last_frame: Cell<Instant>,
    pub animating: Cell<bool>,
}

fn register_class() -> Result<HINSTANCE, String> {
    WINDOW_CLASS
        .get_or_init(|| unsafe {
            let instance = HINSTANCE(
                GetModuleHandleW(None)
                    .map_err(|e| format!("Failed to get module handle: 0x{:08X}", e.code().0 as u32))?
                    .0,
            );
            let class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: w!("CodenotchNativeNotch"),
                ..Default::default()
            };
            if RegisterClassExW(&class) == 0 {
                return Err("Failed to register CodenotchNativeNotch window class".into());
            }
            Ok(instance.0 as usize)
        })
        .clone()
        .map(|inst| HINSTANCE(inst as _))
}

pub fn create_and_run(app: AppHandle) -> Result<(), String> {
    let instance = register_class()?;
    let cfg = {
        let st = app.state::<AppState>();
        st.cfg.lock().unwrap().clone()
    };
    let edge = config::edge_or_right(&cfg.notch_edge);
    let scale = config::snap_scale(cfg.scale) as f32;
    let dark = core::resolved_theme(&app) != "light";
    let (lw, lh) = core::notch_window_size(&edge);

    let (px, py, pw, ph) = if let Some(mon) = core::target_screen(&app) {
        let ww = (lw * mon.scale * scale as f64).round() as i32;
        let wh = (lh * mon.scale * scale as f64).round() as i32;
        let (x, y) = core::edge_origin(&mon, &edge, ww, wh, cfg.along(&edge));
        (x, y, ww as u32, wh as u32)
    } else {
        (100, 100, (lw * scale as f64) as u32, (lh * scale as f64) as u32)
    };

    let mut anim = Anim::default();
    if cfg.notch_on_hover {
        anim.fold.jump(1.0);
        anim.fold_t = 1.0;
    }

    let state = Rc::new(NotchState {
        app: app.clone(),
        hwnd: Cell::new(HWND::default()),
        renderer: RefCell::new(None),
        anim: RefCell::new(anim),
        width: Cell::new(pw),
        height: Cell::new(ph),
        scale: Cell::new(scale),
        edge: RefCell::new(edge),
        dark: Cell::new(dark),
        folded: Cell::new(cfg.notch_on_hover),
        hover_cell: Cell::new(None),
        orb_hover: Cell::new(false),
        card_open: Cell::new(false),
        card_cell: Cell::new(None),
        card_glide: RefCell::new(Tween::new(0.0, 300, CARD)),
        card_pos: Cell::new(D2D_POINT_2F { x: 0.0, y: 0.0 }),
        pressing: Cell::new(None),
        refresh_starts: RefCell::new(HashMap::new()),
        started: Instant::now(),
        last_frame: Cell::new(Instant::now()),
        animating: Cell::new(true),
    });

    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
            w!("CodenotchNativeNotch"),
            w!("Codenotch"),
            WS_POPUP,
            px,
            py,
            pw as i32,
            ph as i32,
            None,
            None,
            instance,
            Some(Rc::as_ptr(&state) as *const _),
        )
        .map_err(|e| format!("CreateWindowExW failed: 0x{:08X}", e.code().0 as u32))?;

        state.hwnd.set(hwnd);
        let renderer = NotchRenderer::new(hwnd, pw, ph)
            .map_err(|e| format!("NotchRenderer::new failed: 0x{:08X}", e.code().0 as u32))?;
        *state.renderer.borrow_mut() = Some(renderer);
        NOTCH_STATE.with(|slot| *slot.borrow_mut() = Some(state.clone()));

        app.register_hwnd("notch", hwnd.0 as isize);
        let _ = ShowWindow(hwnd, SW_SHOW);
    }

    // Run animation and frame loop
    run_frame_loop(state);
    Ok(())
}

fn run_frame_loop(state: Rc<NotchState>) {
    RUNNING.store(true, Ordering::SeqCst);
    let hwnd = state.hwnd.get();

    while RUNNING.load(Ordering::SeqCst) {
        unsafe {
            // Process any incoming Win32 messages without blocking
            let mut msg = wm::MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == wm::WM_QUIT {
                    RUNNING.store(false, Ordering::SeqCst);
                    return;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Step animations & sync to DWM display refresh
            let now = Instant::now();
            let dt = (now.saturating_duration_since(state.last_frame.get())).as_secs_f32();
            state.last_frame.set(now);

            let is_animating = update_animations(&state, now, dt);
            state.animating.set(is_animating);

            // Redraw frame
            render_frame(&state);

            // Synchronize with DWM display refresh (blocks until v-blank, 0% CPU spin)
            let _ = DwmFlush();

            if !is_animating {
                // When completely at rest, sleep for 16ms to avoid unnecessary rendering
                std::thread::sleep(Duration::from_millis(16));
            }
        }
    }
}

fn update_animations(state: &NotchState, now: Instant, dt: f32) -> bool {
    let mut anim = state.anim.borrow_mut();
    anim.fold_t = anim.fold.value(now);
    anim.card_t = anim.card.value(now);
    anim.orb_t = anim.orb.value(now);
    anim.release_t = anim.release.value(now);
    anim.grip_out = anim.grip.step(dt);
    anim.press_t = anim.press.step(dt);

    let active = !anim.fold.done(now)
        || !anim.card.done(now)
        || !anim.orb.done(now)
        || !anim.release.done(now)
        || !anim.grip.settled()
        || !anim.press.settled();

    // Check if any activity spin / pulse is running in background
    let acts_busy = {
        let st = state.app.state::<AppState>();
        let acts = st.activity.lock().unwrap();
        acts.iter().any(|a| a.state == "busy" || a.state == "waiting")
    };

    active || acts_busy
}

fn render_frame(state: &NotchState) {
    let mut renderer_guard = state.renderer.borrow_mut();
    let Some(renderer) = renderer_guard.as_mut() else { return };

    let st = state.app.state::<AppState>();
    let cfg = st.cfg.lock().unwrap().clone();
    let usage = st.usage.lock().unwrap().clone();
    let codex = st.codex.lock().unwrap().clone();
    let copilot = st.copilot.lock().unwrap().clone();
    let ag = st.antigravity.lock().unwrap().clone();
    let opencode = st.opencode.lock().unwrap().clone();
    let acts = st.activity.lock().unwrap().clone();
    let now_sec = state.started.elapsed().as_secs_f32();

    let edge = state.edge.borrow().clone();
    let scale = state.scale.get();
    let anim = state.anim.borrow().clone();

    // Build cells data
    let mut cells: Vec<CellRenderData> = Vec::new();
    let providers = [
        ("claude", &usage),
        ("codex", &codex),
        ("copilot", &copilot),
        ("gemini", &ag),
        ("opencode", &opencode),
    ];

    for (prov_id, snap) in providers {
        if snap.status == "absent" { continue; }
        let head = snap.windows.first();
        let used = head.map(|w| w.used as f32);
        let week = snap.windows.get(1).map(|w| w.used as f32);
        let work = if acts.iter().any(|a| a.provider == prov_id && a.state == "busy") {
            "busy".to_string()
        } else if acts.iter().any(|a| a.provider == prov_id && a.state == "waiting") {
            "waiting".to_string()
        } else {
            "idle".to_string()
        };

        let pct_str = head.map(|w| {
            let frac = if cfg.notch_shows_remaining { 1.0 - w.used } else { w.used };
            format!("{:.0}%", (frac * 100.0).clamp(0.0, 100.0))
        }).unwrap_or_default();

        let pressing = state.pressing.get().map(|idx| idx == cells.len()).unwrap_or(false);

        cells.push(CellRenderData {
            id: prov_id.to_string(),
            provider: prov_id.to_string(),
            headline: used,
            weekly: week,
            weekly_dashed: cfg.weekly_ring_dashed,
            weekly_inside: cfg.weekly_ring == "inside",
            text: pct_str,
            stale: false,
            spent: used.map(|u| u >= 1.0).unwrap_or(false),
            work_state: work,
            press: if pressing { 1.0 } else { 0.0 },
            turn_t: anim.turn_t,
        });
    }

    let cell_count = cells.len().max(1);
    let (lw, lh) = core::notch_window_size(&edge);
    let is_vert = matches!(edge.as_str(), "left" | "right");

    // Layout math
    let pill_w = 70.0f32;
    let pill_len = (cell_count as f32 * 68.0 + 36.0).max(120.0);
    let (pill_rect, rest_rect, orb_center, grip_center) = if is_vert {
        let py = (lh as f32 - pill_len) * 0.5;
        let px = if edge == "left" { 0.0 } else { lw as f32 - pill_w };
        let pr = D2D_RECT_F { left: px, top: py, right: px + pill_w, bottom: py + pill_len };
        let rr = D2D_RECT_F {
            left: if edge == "left" { 0.0 } else { lw as f32 - 10.0 },
            top: (lh as f32 - 79.0) * 0.5,
            right: if edge == "left" { 10.0 } else { lw as f32 },
            bottom: (lh as f32 + 79.0) * 0.5,
        };
        let oc = D2D_POINT_2F {
            x: if edge == "left" { 38.7 } else { lw as f32 - 38.7 },
            y: py - 18.0,
        };
        let gc = D2D_POINT_2F { x: oc.x, y: oc.y - 18.0 };
        (pr, rr, oc, gc)
    } else {
        let px = (lw as f32 - pill_len) * 0.5;
        let py = if edge == "top" { 0.0 } else { lh as f32 - pill_w };
        let pr = D2D_RECT_F { left: px, top: py, right: px + pill_len, bottom: py + pill_w };
        let rr = D2D_RECT_F {
            left: (lw as f32 - 79.0) * 0.5,
            top: if edge == "top" { 0.0 } else { lh as f32 - 10.0 },
            right: (lw as f32 + 79.0) * 0.5,
            bottom: if edge == "top" { 10.0 } else { lh as f32 },
        };
        let oc = D2D_POINT_2F {
            x: px - 18.0,
            y: if edge == "top" { 38.7 } else { lh as f32 - 38.7 },
        };
        let gc = D2D_POINT_2F { x: oc.x - 18.0, y: oc.y };
        (pr, rr, oc, gc)
    };

    let mut ring_centers = Vec::new();
    for i in 0..cell_count {
        let center = if is_vert {
            D2D_POINT_2F {
                x: pill_rect.left + pill_w * 0.5,
                y: pill_rect.top + 34.0 + i as f32 * 68.0,
            }
        } else {
            D2D_POINT_2F {
                x: pill_rect.left + 34.0 + i as f32 * 68.0,
                y: pill_rect.top + pill_w * 0.5,
            }
        };
        ring_centers.push(center);
    }

    let frame_data = NotchFrameData {
        edge,
        scale,
        fold_t: anim.fold_t,
        pill_rect,
        rest_rect,
        cells,
        ring_centers,
        orb_center,
        orb_hover: anim.orb_t,
        grip_out: anim.grip_out,
        grip_center,
        card_open: state.card_open.get(),
        card_t: anim.card_t,
        card_rect: None,
        tail_rect: None,
        card_headline: String::new(),
        card_subtitle: String::new(),
        card_body: Vec::new(),
        card_used: None,
        seconds: now_sec,
        watch_limit: cfg.watch_limit,
        critical_limit: cfg.critical_limit,
        shows_remaining: cfg.notch_shows_remaining,
    };

    let _ = renderer.draw(&frame_data);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            // Precise hit testing: only clicks inside the active pill/card return HTCLIENT!
            // Clicks in transparent margins return HTTRANSPARENT (fall straight through to desktop).
            let x = (lparam.0 & 0xffff) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;
            let mut pt = POINT { x, y };
            let _ = ScreenToClient(hwnd, &mut pt);

            let hit = NOTCH_STATE.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    let w = state.width.get() as i32;
                    let h = state.height.get() as i32;
                    let edge = state.edge.borrow();
                    let is_vert = matches!(edge.as_str(), "left" | "right");
                    if state.folded.get() {
                        // Folded rest bar
                        if is_vert {
                            pt.x >= (w - 20) && pt.y >= (h / 2 - 50) && pt.y <= (h / 2 + 50)
                        } else {
                            pt.y >= (h - 20) && pt.x >= (w / 2 - 50) && pt.x <= (w / 2 + 50)
                        }
                    } else {
                        // Expanded pill
                        if is_vert {
                            if *edge == "left" { pt.x <= 80 } else { pt.x >= (w - 80) }
                        } else {
                            if *edge == "top" { pt.y <= 80 } else { pt.y >= (h - 80) }
                        }
                    }
                } else {
                    true
                }
            });

            if hit {
                LRESULT(HTCLIENT as isize)
            } else {
                LRESULT(HTTRANSPARENT as isize)
            }
        }
        WM_MOUSEMOVE => {
            NOTCH_STATE.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    if state.folded.get() {
                        state.folded.set(false);
                        state.anim.borrow_mut().set_folded(false, Instant::now(), true);
                    }
                    state.orb_hover.set(true);
                    state.anim.borrow_mut().set_orb(true, Instant::now(), true);
                }
            });

            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tme);
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            NOTCH_STATE.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    let cfg = {
                        let st = state.app.state::<AppState>();
                        st.cfg.lock().unwrap().clone()
                    };
                    if cfg.notch_on_hover {
                        state.folded.set(true);
                        state.anim.borrow_mut().set_folded(true, Instant::now(), true);
                    }
                    state.orb_hover.set(false);
                    state.anim.borrow_mut().set_orb(false, Instant::now(), true);
                }
            });
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            NOTCH_STATE.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    state.pressing.set(Some(0));
                    state.anim.borrow_mut().press.retarget(1.0);
                }
            });
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            NOTCH_STATE.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    state.pressing.set(None);
                    state.anim.borrow_mut().press.retarget(0.0);
                    core::refresh_all(&state.app);
                }
            });
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            NOTCH_STATE.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    crate::notchmenu::show_notch_menu(&state.app, hwnd.0 as isize, None);
                }
            });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_DESTROY => {
            RUNNING.store(false, Ordering::SeqCst);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

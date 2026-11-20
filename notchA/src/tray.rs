use crate::core::{self, AppHandle, AppState, Manager, UiEvent};
use crate::i18n::tr;
use crate::traymenu;
use std::cell::RefCell;
use std::sync::Mutex;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

thread_local! {
    static TRAY: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
}
static SHOWN: Mutex<Option<Vec<(String, String, bool)>>> = Mutex::new(None);

pub fn setup(app: &AppHandle) -> Result<(), String> {
    let ev_app = app.clone();
    MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
        ev_app.send_ui(UiEvent::MenuAction(ev.id().0.clone()));
    }));

    let menu = build_menu(app)?;
    let mut builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip(app))
        .with_menu_on_left_click(true);

    if let Some(icon) = crate::trayicon::app_mark() {
        builder = builder.with_icon(icon);
    }

    let tray = builder.build().map_err(|e| e.to_string())?;
    let visible = app.state::<AppState>().cfg.lock().unwrap().tray_visible;
    let _ = tray.set_visible(visible);
    TRAY.with(|slot| *slot.borrow_mut() = Some(tray));
    Ok(())
}

pub fn set_visible(visible: bool) {
    TRAY.with(|slot| {
        if let Some(tray) = slot.borrow().as_ref() {
            let _ = tray.set_visible(visible);
        }
    });
}

fn menu_lines(app: &AppHandle) -> Vec<(String, String, bool)> {
    let now = core::now_ms();
    let mut lines = Vec::new();
    for id in core::TRAY_PROVIDER_IDS {
        let snap = core::snapshot_of(app, id);
        if snap.status == "absent" {
            continue;
        }
        let head = traymenu::header(
            core::provider_label(id),
            core::ring_fraction(app, id),
            traymenu::stale_since(&snap, now),
            now,
            "en",
        );
        lines.push((format!("refresh:{id}"), head, true));
        for (n, line) in traymenu::provider_lines(&snap, now, "en").iter().enumerate() {
            lines.push((format!("line:{id}:{n}"), format!("    {line}"), false));
        }
    }
    lines
}

pub fn build_menu(app: &AppHandle) -> Result<Menu, String> {
    build_menu_from(&menu_lines(app))
}

fn build_menu_from(lines: &[(String, String, bool)]) -> Result<Menu, String> {
    let menu = Menu::new();
    if lines.is_empty() {
        let waiting = MenuItem::with_id("waiting", tr("en", "waiting"), false, None);
        let _ = menu.append(&waiting);
    } else {
        for (id, text, enabled) in lines {
            let item = MenuItem::with_id(id, text, *enabled, None);
            let _ = menu.append(&item);
        }
    }
    let sep1 = PredefinedMenuItem::separator();
    let refresh = MenuItem::with_id("refresh", tr("en", "refresh_all"), true, None);
    let settings = MenuItem::with_id("settings", tr("en", "settings"), true, None);
    let sep2 = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id("quit", tr("en", "quit_app"), true, None);
    let _ = menu.append_items(&[&sep1, &refresh, &settings, &sep2, &quit]);
    Ok(menu)
}

fn tooltip(app: &AppHandle) -> String {
    let mut parts = Vec::new();
    for id in core::TRAY_PROVIDER_IDS {
        if core::snapshot_of(app, id).status == "absent" {
            continue;
        }
        let value = core::ring_fraction(app, id)
            .map(|f| format!("{}%", traymenu::pct(f)))
            .unwrap_or_else(|| "—".into());
        parts.push(format!("{} {value}", core::provider_label(id)));
    }
    if parts.is_empty() {
        concat!("notchA v", env!("CARGO_PKG_VERSION")).to_string()
    } else {
        format!("notchA — {}", parts.join(" · "))
    }
}

pub fn refresh_menu(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let lines = menu_lines(&handle);
        let tip = tooltip(&handle);
        TRAY.with(|slot| {
            let guard = slot.borrow();
            let Some(tray) = guard.as_ref() else { return };
            if SHOWN.lock().unwrap().as_ref() == Some(&lines) {
                let _ = tray.set_tooltip(Some(&tip));
                return;
            }
            if let Ok(menu) = build_menu_from(&lines) {
                tray.set_menu(Some(Box::new(menu)));
                let _ = tray.set_tooltip(Some(&tip));
                *SHOWN.lock().unwrap() = Some(lines);
            }
        });
    });
}

pub fn handle_menu_id(app: &AppHandle, id: &str) {
    if crate::notchmenu::handle(app, id) {
        return;
    }
    if let Some(provider) = id.strip_prefix("refresh:") {
        core::refresh_provider(provider);
        return;
    }
    match id {
        "refresh" => core::refresh_all(app),
        "settings" => app.send_ui(UiEvent::OpenSettings),
        "quit" => app.exit(0),
        _ => {}
    }
}
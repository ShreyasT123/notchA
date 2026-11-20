use crate::core::{self, AppHandle};
use crate::i18n::tr;
use tray_icon::menu::{CheckMenuItem, ContextMenu, Menu, MenuItem, PredefinedMenuItem};

const PREFIX: &str = "notch:";

pub fn show_notch_menu(app: &AppHandle, hwnd: isize, provider: Option<&str>) {
    let menu = Menu::new();
    let refresh = MenuItem::with_id(format!("{PREFIX}refresh"), tr("en", "refresh_now"), true, None);
    let _ = menu.append(&refresh);

    if let Some(p) = provider {
        if let Some((_, host)) = core::provider_page(p) {
            let open = MenuItem::with_id(
                format!("{PREFIX}open:{p}"),
                tr("en", "open_host").replace("%@", host),
            true,
            None,
            );
            let _ = menu.append(&open);
        }
    }

    let sep1 = PredefinedMenuItem::separator();
    let keep_open = CheckMenuItem::with_id(
        format!("{PREFIX}keep_open"),
        tr("en", "keep_open"),
        true,
        core::keeps_open(app),
        None,
    );
    let sep2 = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id(format!("{PREFIX}quit"), tr("en", "quit_app"), true, None);
    let _ = menu.append_items(&[&sep1, &keep_open, &sep2, &quit]);

    #[cfg(windows)]
    {
        let before = foreground();
        unsafe {
            menu.show_context_menu_for_hwnd(hwnd, None);
        }
        give_back(before, hwnd);
    }
    #[cfg(not(windows))]
    let _ = hwnd;
}

#[cfg(windows)]
fn foreground() -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow().0 as isize }
}

#[cfg(windows)]
fn give_back(before: isize, notch: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
    if before == 0 || before == notch || foreground() != notch {
        return;
    }
    unsafe {
        let _ = SetForegroundWindow(HWND(before as _));
    }
}

pub fn handle(app: &AppHandle, id: &str) -> bool {
    let Some(item) = id.strip_prefix(PREFIX) else {
        return false;
    };
    if let Some(provider) = item.strip_prefix("open:") {
        core::open_provider_page(provider);
        return true;
    }
    match item {
        "refresh" => core::refresh_all(app),
        "keep_open" => core::toggle_keep_open(app),
        "quit" => app.exit(0),
        _ => {}
    }
    true
}
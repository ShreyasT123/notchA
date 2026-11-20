#[allow(dead_code)]
#[allow(dead_code)]
pub fn resolve_auto() -> &'static str {
    "en"
}

pub fn tr(_lang: &str, key: &str) -> &'static str {
    match key {
        "waiting" => "Waiting for first reading…",
        "refresh_all" => "Refresh all",
        "refresh_now" => "Refresh now",
        "open_host" => "Open %@",
        "keep_open" => "Keep open",
        "settings" => "Settings…",
        "quit_app" => "Quit notchA",
        "stale" => "stale",
        "needs_auth" => "not signed in",
        "backoff" => "rate limited",
        "error" => "unavailable",
        "resets_in_min" => "Resets in %@ min",
        "resets_at" => "Resets at %@",
        "resets_on" => "Resets %@",
        "resetting" => "Resetting…",
        "updated_min" => "Updated %@m ago",
        "updated_hr" => "Updated %@h ago",
        _ => "",
    }
}

#[cfg(windows)]
pub fn clock_24h() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_SSHORTTIME};
    let mut buf = [0u16; 32];
    let len = unsafe { GetLocaleInfoEx(PCWSTR::null(), LOCALE_SSHORTTIME, Some(&mut buf)) };
    if len <= 0 {
        return false;
    }
    let fmt = String::from_utf16_lossy(&buf[..(len as usize).saturating_sub(1)]);
    fmt.contains('H')
}

#[cfg(not(windows))]
pub fn clock_24h() -> bool {
    false
}
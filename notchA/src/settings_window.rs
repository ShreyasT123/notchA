use crate::core::{self, AppHandle, Theme, UiEvent};
const FIRST_MICA_BUILD: u32 = 22000;

pub fn open(app: &AppHandle) {
    app.send_ui(UiEvent::OpenSettings);
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn quit_app(app: &AppHandle) {
    app.exit(0);
}

pub fn open_author_page() {
    let mut cmd = std::process::Command::new("cmd");
    cmd.args(["/C", "start", "", "https://x.com/hivinz_"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd.spawn();
}

#[derive(Clone, Debug)]
pub struct SystemLook {
    pub mica: bool,
    pub accent: Vec<iced::Color>,
}

pub fn get_system_look() -> SystemLook {
    let accent = reg_binary(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent",
        "AccentPalette",
    )
    .map(|bytes| palette_colors(&bytes))
    .unwrap_or_default();

    SystemLook {
        mica: has_mica(),
        accent,
    }
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn follow_theme(app: &AppHandle, theme: Option<Theme>) {
    let Some(w) = app.get_webview_window("settings") else {
        return;
    };
    let dark = match theme {
        Some(Theme::Light) => false,
        Some(Theme::Dark) => true,
        None => !core::windows_apps_use_light_theme(),
    };
    apply_mica(w.hwnd, dark);
}

#[cfg(windows)]
pub fn apply_mica(hwnd_raw: isize, dark: bool) {
    use windows::Win32::Foundation::{BOOL, HWND};
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_IMMERSIVE_DARK_MODE,
    };

    if hwnd_raw == 0 {
        return;
    }
    let hwnd = HWND(hwnd_raw as _);
    let dark_val = BOOL::from(dark);
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_val as *const _ as *const _,
            std::mem::size_of::<BOOL>() as u32,
        );
        if has_mica() {
            let backdrop = DWMSBT_MAINWINDOW.0;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &backdrop as *const _ as *const _,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }
}

#[cfg(not(windows))]
pub fn apply_mica(_hwnd_raw: isize, _dark: bool) {}

pub fn has_mica() -> bool {
    reg_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "CurrentBuildNumber")
        .and_then(|build| build.trim().parse::<u32>().ok())
        .is_some_and(|build| build >= FIRST_MICA_BUILD)
}

fn palette_colors(bytes: &[u8]) -> Vec<iced::Color> {
    bytes
        .chunks_exact(4)
        .take(7)
        .map(|c| iced::Color::from_rgb8(c[0], c[1], c[2]))
        .collect()
}

#[cfg(windows)]
fn reg_binary(key: &str, value: &str) -> Option<Vec<u8>> {
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_BINARY};
    let mut data = vec![0u8; 64];
    let mut size = data.len() as u32;
    reg_get(HKEY_CURRENT_USER, key, value, RRF_RT_REG_BINARY, data.as_mut_ptr().cast(), &mut size)
        .then(|| {
            data.truncate(size as usize);
            data
        })
}

#[cfg(windows)]
fn reg_string(key: &str, value: &str) -> Option<String> {
    use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let mut data = vec![0u16; 64];
    let mut size = (data.len() * 2) as u32;
    reg_get(HKEY_LOCAL_MACHINE, key, value, RRF_RT_REG_SZ, data.as_mut_ptr().cast(), &mut size)
        .then(|| {
            let chars = (size as usize / 2).min(data.len());
            String::from_utf16_lossy(&data[..chars])
                .trim_end_matches('\0')
                .to_string()
        })
}

#[cfg(windows)]
fn reg_get(
    root: windows::Win32::System::Registry::HKEY,
    key: &str,
    value: &str,
    kind: windows::Win32::System::Registry::REG_ROUTINE_FLAGS,
    data: *mut std::ffi::c_void,
    size: &mut u32,
) -> bool {
    use windows::core::HSTRING;
    use windows::Win32::System::Registry::RegGetValueW;
    let (key, value) = (HSTRING::from(key), HSTRING::from(value));
    unsafe { RegGetValueW(root, &key, &value, kind, None, Some(data), Some(size)) }.is_ok()
}

#[cfg(not(windows))]
fn reg_binary(_key: &str, _value: &str) -> Option<Vec<u8>> { None }
#[cfg(not(windows))]
fn reg_string(_key: &str, _value: &str) -> Option<String> { None }
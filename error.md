    Checking notchA v0.1.0 (C:\Users\sstha\Desktop\ai_\projects\natchos\notchA\notchA)
error[E0433]: cannot find module or crate `ring` in this scope
  --> notchA\src\server.rs:24:9
   |
24 |     use ring::rand::SecureRandom;
   |         ^^^^ use of unresolved module or unlinked crate `ring`
   |
   = help: if you wanted to use a crate named `ring`, use `cargo add ring` to add it to your `Cargo.toml`

error[E0432]: unresolved import `windows::Win32::Globalization::LOCALE_NAME_USER_DEFAULT`
  --> notchA\src\i18n.rs:30:58
   |
30 |     use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_NAME_USER_DEFAULT, LOCALE_SSHORTTIME};
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^ no `LOCALE_NAME_USER_DEFAULT` in `Win32::Globalization`
   |
help: a similar name exists in the module
   |
30 -     use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_NAME_USER_DEFAULT, LOCALE_SSHORTTIME};
30 +     use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_NAME_SYSTEM_DEFAULT, LOCALE_SSHORTTIME};
   |

error[E0433]: cannot find `cursor` in the crate root
   --> notchA\src\activity.rs:131:41
    |
131 |             cursor: DbCache::new(crate::cursor::store_url().unwrap_or_default()),
    |                                         ^^^^^^ unresolved import
    |
help: a similar path exists
    |
131 |             cursor: DbCache::new(crate::iced::widget::text_input::cursor::store_url().unwrap_or_default()),
    |                                         ++++++++++++++++++++++++++

error[E0433]: cannot find `cursor` in the crate root
   --> notchA\src\activity.rs:600:24
    |
600 |         cursor: crate::cursor::present(),
    |                        ^^^^^^ unresolved import
    |
help: a similar path exists
    |
600 |         cursor: crate::iced::widget::text_input::cursor::present(),
    |                        ++++++++++++++++++++++++++

error[E0433]: cannot find module or crate `ring` in this scope
  --> notchA\src\server.rs:25:15
   |
25 |     let rng = ring::rand::SystemRandom::new();
   |               ^^^^ use of unresolved module or unlinked crate `ring`
   |
   = help: if you wanted to use a crate named `ring`, use `cargo add ring` to add it to your `Cargo.toml`

error[E0423]: cannot find function `canvas` in this scope
   --> notchA\src\notch_view.rs:879:21
    |
879 |     let bg_canvas = canvas(NotchCanvasProgram {
    |                     ^^^^^^
    |
    = note: a module named `canvas` exists in another namespace
help: you might have meant to use `:` for type annotation
    |
879 -     let bg_canvas = canvas(NotchCanvasProgram {
879 +     let bg_canvas: canvas(NotchCanvasProgram {
    |
help: consider importing this function instead
    |
  1 + use iced::widget::canvas;
    |

error[E0423]: cannot find function `canvas` in this scope
   --> notchA\src\reset_view.rs:134:14
    |
134 |     let bg = canvas(TailCanvas {
    |              ^^^^^^
    |
    = note: a module named `canvas` exists in another namespace
help: you might have meant to use `:` for type annotation
    |
134 -     let bg = canvas(TailCanvas {
134 +     let bg: canvas(TailCanvas {
    |
help: consider importing this function instead
    |
  1 + use iced::widget::canvas;
    |

error[E0433]: cannot find `ffi` in `core`
   --> notchA\src\settings_window.rs:137:22
    |
137 |     data: *mut core::ffi::c_void,
    |                      ^^^ could not find `ffi` in `core`
    |
help: consider importing one of these modules
    |
  1 + use std::ffi;
    |
  1 + use ::core::ffi;
    |
help: if you import `ffi`, refer to it directly
    |
137 -     data: *mut core::ffi::c_void,
137 +     data: *mut ffi::c_void,
    |

warning: unused import: `Manager`
  --> notchA\src\topmost.rs:12:30
   |
12 | use crate::core::{AppHandle, Manager, WindowHandle as WebviewWindow};
   |                              ^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0119]: conflicting implementations of trait `std::fmt::Debug` for type `UiEvent`
  --> notchA\src\core.rs:36:10
   |
36 | #[derive(Debug)]
   |          ^^^^^ conflicting implementation for `UiEvent`
...
62 | impl std::fmt::Debug for UiEvent {
   | -------------------------------- first implementation here

error[E0277]: `Rc<RefCell<tray_icon::platform_impl::platform::TrayIcon>>` cannot be sent between threads safely
   --> notchA\src\tray.rs:8:14
    |
  8 | static TRAY: OnceLock<Mutex<Option<TrayIcon>>> = OnceLock::new();
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `Rc<RefCell<tray_icon::platform_impl::platform::TrayIcon>>` cannot be sent between threads safely
    |
    = help: within `std::option::Option<TrayIcon>`, the trait `std::marker::Send` is not implemented for `Rc<RefCell<tray_icon::platform_impl::platform::TrayIcon>>`
note: required because it appears within the type `TrayIcon`
   --> C:\Users\sstha\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tray-icon-0.26.1\src\lib.rs:469:12
    |
469 | pub struct TrayIcon {
    |            ^^^^^^^^
note: required because it appears within the type `std::option::Option<TrayIcon>`
   --> C:\Users\sstha\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library/core\src\option.rs:598:10
    |
598 | pub enum Option<T> {
    |          ^^^^^^
    = note: required for `std::sync::Mutex<std::option::Option<TrayIcon>>` to implement `Sync`
    = note: 1 redundant requirement hidden
    = note: required for `OnceLock<std::sync::Mutex<std::option::Option<TrayIcon>>>` to implement `Sync`
    = note: shared static variables must have a type that implements `Sync`

error[E0277]: cannot add `i32` to `f64`
   --> notchA\src\backdrop.rs:199:30
    |
199 |     luminance(&capture(pos.x + x, pos.y + y, width, height)?)
    |                              ^ no implementation for `f64 + i32`
    |
    = help: the trait `std::ops::Add<i32>` is not implemented for `f64`
    = help: the following other types implement trait `std::ops::Add<Rhs>`:
              `&f64` implements `std::ops::Add<f64>`
              `&f64` implements `std::ops::Add`
              `f64` implements `std::ops::Add<&f64>`
              `f64` implements `std::ops::Add<glam::f64::dvec2::DVec2>`
              `f64` implements `std::ops::Add<glam::f64::dvec3::DVec3>`
              `f64` implements `std::ops::Add<glam::f64::dvec4::DVec4>`
              `f64` implements `std::ops::Add<zerocopy::byteorder::F64<O>>`
              `f64` implements `std::ops::Add`

error[E0277]: cannot add `i32` to `f64`
   --> notchA\src\backdrop.rs:199:41
    |
199 |     luminance(&capture(pos.x + x, pos.y + y, width, height)?)
    |                                         ^ no implementation for `f64 + i32`
    |
    = help: the trait `std::ops::Add<i32>` is not implemented for `f64`
    = help: the following other types implement trait `std::ops::Add<Rhs>`:
              `&f64` implements `std::ops::Add<f64>`
              `&f64` implements `std::ops::Add`
              `f64` implements `std::ops::Add<&f64>`
              `f64` implements `std::ops::Add<glam::f64::dvec2::DVec2>`
              `f64` implements `std::ops::Add<glam::f64::dvec3::DVec3>`
              `f64` implements `std::ops::Add<glam::f64::dvec4::DVec4>`
              `f64` implements `std::ops::Add<zerocopy::byteorder::F64<O>>`
              `f64` implements `std::ops::Add`

error[E0277]: `&Box<(dyn FnOnce() + std::marker::Send + 'static)>` doesn't implement `std::fmt::Debug`
  --> notchA\src\core.rs:43:16
   |
36 | #[derive(Debug)]
   |          ----- in this derive macro expansion
...
43 |     MainThread(Box<dyn FnOnce() + Send>),
   |                ^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
   = help: within `&Box<(dyn FnOnce() + std::marker::Send + 'static)>`, the trait `std::fmt::Debug` is not implemented for `&Box<(dyn FnOnce() + std::marker::Send + 'static)>`

error[E0616]: field `enabled` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:425:38
    |
425 |         let is_on = claude_on && acc.enabled;
    |                                      ^^^^^^^ private field

error[E0616]: field `id` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:427:26
    |
427 |         let acc_id = acc.id.clone();
    |                          ^^ private field

error[E0616]: field `id` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:428:27
    |
428 |         let acc_id2 = acc.id.clone();
    |                           ^^ private field

error[E0616]: field `connected` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:430:27
    |
430 |         if is_on && (!acc.connected || acc.status == "needsAuth") && !auth.busy {
    |                           ^^^^^^^^^ private field

error[E0616]: field `status` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:430:44
    |
430 |         if is_on && (!acc.connected || acc.status == "needsAuth") && !auth.busy {
    |                                            ^^^^^^ private field

error[E0616]: field `name` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:436:34
    |
436 |             row![glyph, text(acc.name.clone()).size(13.5).color(p.text), space::horizontal(), right_ctrl]
    |                                  ^^^^ private field

error[E0616]: field `address` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:443:17
    |
443 |             acc.address.clone().unwrap_or_default()
    |                 ^^^^^^^ private field

error[E0616]: field `id` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:444:71
    |
444 |         } else if auth.busy && auth.profile_id.as_deref() == Some(acc.id.as_str()) {
    |                                                                       ^^ private field

error[E0616]: field `connected` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:446:24
    |
446 |         } else if !acc.connected || acc.status == "needsAuth" {
    |                        ^^^^^^^^^ private field

error[E0616]: field `status` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:446:41
    |
446 |         } else if !acc.connected || acc.status == "needsAuth" {
    |                                         ^^^^^^ private field

error[E0616]: field `status` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:449:32
    |
449 |             let st = match acc.status.as_str() {
    |                                ^^^^^^ private field

error[E0616]: field `address` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:456:18
    |
456 |             [acc.address.as_deref().unwrap_or(""), st]
    |                  ^^^^^^^ private field

error[E0616]: field `id` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:465:16
    |
465 |         if acc.id == "default" {
    |                ^^ private field

error[E0616]: field `id` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:901:55
    |
901 |             if let Some(acc) = list.iter().find(|a| a.id == id) {
    |                                                       ^^ private field

error[E0616]: field `enabled` of struct `AccountView` is private
   --> notchA\src\settings_view.rs:902:99
    |
902 |                 if let Err(e) = claude_accounts::set_claude_account_enabled(app.clone(), id, !acc.enabled) {
    |                                                                                                   ^^^^^^^ private field

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> notchA\src\taskbar_activity.rs:123:8
    |
123 |     Ok(crate::refresh_provider(&app, &id))
    |        ^^^^^^^^^^^^^^^^^^^^^^^ ---- unexpected argument #1 of type `&std::sync::Arc<AppCore>`
    |
note: function defined here
   --> notchA\src\core.rs:392:8
    |
392 | pub fn refresh_provider(provider: &str) -> bool {
    |        ^^^^^^^^^^^^^^^^
help: remove the extra argument
    |
123 -     Ok(crate::refresh_provider(&app, &id))
123 +     Ok(crate::refresh_provider(&id))
    |

error[E0282]: type annotations needed for `iced::Task<_>`
   --> notchA\src\main.rs:391:25
    |
391 |                     let _ = window::close(id);
    |                         ^
    |
help: consider giving this pattern a type, where the type for type parameter `T` is specified
    |
391 |                     let _: iced::Task<T> = window::close(id);
    |                          +++++++++++++++

warning: unused import: `HasWindowHandle`
   --> notchA\src\main.rs:773:33
    |
773 |         use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    |                                 ^^^^^^^^^^^^^^^

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
   --> notchA\src\core.rs:638:25
    |
638 |         let list = &mut *(data.0 as *mut Vec<Collected>);
    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer
    |
    = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\core.rs:632:5
    |
632 | /     unsafe extern "system" fn cb(
633 | |         hmon: HMONITOR,
634 | |         _hdc: HDC,
635 | |         _rect: *mut RECT,
636 | |         data: LPARAM,
637 | |     ) -> BOOL {
    | |_____________^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>
    = note: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::GetMonitorInfoW` is unsafe and requires unsafe block
   --> notchA\src\core.rs:641:12
    |
641 |         if GetMonitorInfoW(hmon, &mut info.monitorInfo as *mut _).as_bool() {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::HiDpi::GetDpiForMonitor` is unsafe and requires unsafe block
   --> notchA\src\core.rs:644:21
    |
644 |             let _ = GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
  --> notchA\src\focus.rs:62:22
   |
62 |         let v = &mut *(l.0 as *mut Vec<(isize, u32)>);
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer
   |
   = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
  --> notchA\src\focus.rs:61:5
   |
61 |     unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::IsWindowVisible` is unsafe and requires unsafe block
  --> notchA\src\focus.rs:63:12
   |
63 |         if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
   |            ^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW` is unsafe and requires unsafe block
  --> notchA\src\focus.rs:63:47
   |
63 |         if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
   |                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId` is unsafe and requires unsafe block
  --> notchA\src\focus.rs:65:13
   |
65 |             GetWindowThreadProcessId(hwnd, Some(&mut pid));
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
   --> notchA\src\focus.rs:218:22
    |
218 |         let v = &mut *(l.0 as *mut Vec<(isize, u32)>);
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer
    |
    = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\focus.rs:217:5
    |
217 |     unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::IsWindowVisible` is unsafe and requires unsafe block
   --> notchA\src\focus.rs:219:12
    |
219 |         if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
    |            ^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW` is unsafe and requires unsafe block
   --> notchA\src\focus.rs:219:47
    |
219 |         if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
    |                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId` is unsafe and requires unsafe block
   --> notchA\src\focus.rs:221:13
    |
221 |             GetWindowThreadProcessId(hwnd, Some(&mut pid));
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

error[E0505]: cannot move out of `app` because it is borrowed
   --> notchA\src\taskbar.rs:163:36
    |
162 |     let app = app.clone();
    |         --- binding `app` declared here
163 |     let _ = app.run_on_main_thread(move || {
    |             --- ------------------ ^^^^^^^ move out of `app` occurs here
    |             |   |
    |             |   borrow later used by call
    |             borrow of `app` occurs here
...
167 |         hide_hover_now(&app);
    |                         --- move occurs due to use in closure

warning[E0133]: call to unsafe function `windows::Win32::UI::HiDpi::GetWindowDpiAwarenessContext` is unsafe and requires unsafe block
  --> notchA\src\taskbar_native.rs:93:30
   |
93 |         let parent_context = GetWindowDpiAwarenessContext(parent);
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
  --> notchA\src\taskbar_native.rs:92:5
   |
92 |     unsafe fn parent(parent: HWND) -> Result<Self, String> {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::HiDpi::GetThreadDpiAwarenessContext` is unsafe and requires unsafe block
  --> notchA\src\taskbar_native.rs:94:30
   |
94 |         let thread_context = GetThreadDpiAwarenessContext();
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::HiDpi::AreDpiAwarenessContextsEqual` is unsafe and requires unsafe block
  --> notchA\src\taskbar_native.rs:98:12
   |
98 |         if AreDpiAwarenessContextsEqual(parent_context, thread_context).as_bool() { return Ok(Self(None)); }
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::HiDpi::SetThreadDpiAwarenessContext` is unsafe and requires unsafe block
  --> notchA\src\taskbar_native.rs:99:24
   |
99 |         let previous = SetThreadDpiAwarenessContext(parent_context);
   |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:355:23
    |
355 |         let create = &*(lparam.0 as *const CREATESTRUCTW);
    |                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer
    |
    = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_native.rs:353:1
    |
353 | unsafe extern "system" fn window_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `std::rc::Rc::<T>::increment_strong_count` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:360:9
    |
360 |         Rc::increment_strong_count(raw);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:361:9
    |
361 |         (*raw).hwnd.set(window);
    |         ^^^^^^ dereference of raw pointer
    |
    = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:362:9
    |
362 |         SetWindowLongPtrW(window, GWLP_USERDATA, raw as isize);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:365:15
    |
365 |     let raw = GetWindowLongPtrW(window, GWLP_USERDATA) as *const NativeState;
    |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::DefWindowProcW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:366:31
    |
366 |     if raw.is_null() { return DefWindowProcW(window, message, wparam, lparam); }
    |                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `std::rc::Rc::<T>::increment_strong_count` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:367:5
    |
367 |     Rc::increment_strong_count(raw);
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `std::rc::Rc::<T>::from_raw` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:368:19
    |
368 |     let current = Rc::from_raw(raw);
    |                   ^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::BeginPaint` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:377:21
    |
377 |             let _ = BeginPaint(window, &mut paint);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::EndPaint` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:380:21
    |
380 |             let _ = EndPaint(window, &paint);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::Input::KeyboardAndMouse::TrackMouseEvent` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:397:37
    |
397 |                 if let Err(error) = TrackMouseEvent(&mut tracking) { critical_close(&current, "track mouse leave", &error); }
    |                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::InvalidateRect` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:405:25
    |
405 |                 let _ = InvalidateRect(window, None, false);
    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::InvalidateRect` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:412:21
    |
412 |             let _ = InvalidateRect(window, None, false);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::InvalidateRect` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:432:21
    |
432 |             let _ = InvalidateRect(window, None, false);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::InvalidateRect` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:442:21
    |
442 |             let _ = InvalidateRect(window, None, false);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::InvalidateRect` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:457:21
    |
457 |             let _ = InvalidateRect(window, None, false);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::DefWindowProcW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:460:26
    |
460 |             let result = DefWindowProcW(window, message, wparam, lparam);
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::IsWindowVisible` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:462:17
    |
462 |             if !IsWindowVisible(window).as_bool() {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::IsWindowVisible` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:470:16
    |
470 |             if IsWindowVisible(window).as_bool() { let _ = InvalidateRect(window, None, false); }
    |                ^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Gdi::InvalidateRect` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:470:60
    |
470 |             if IsWindowVisible(window).as_bool() { let _ = InvalidateRect(window, None, false); }
    |                                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::IsWindowVisible` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:474:16
    |
474 |             if IsWindowVisible(window).as_bool() {
    |                ^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::DestroyWindow` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:482:33
    |
482 |             if let Err(error) = DestroyWindow(window) { log_error("close child", &error); }
    |                                 ^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::KillTimer` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:487:21
    |
487 |             let _ = KillTimer(window, ANIMATION_TIMER);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::KillTimer` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:488:21
    |
488 |             let _ = KillTimer(window, RECOVERY_TIMER);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:493:13
    |
493 |             SetWindowLongPtrW(window, GWLP_USERDATA, 0);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::DefWindowProcW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:504:26
    |
504 |             let result = DefWindowProcW(window, message, wparam, lparam);
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `std::rc::Rc::<T>::from_raw` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:505:18
    |
505 |             drop(Rc::from_raw(raw)); // Release the HWND's reference, not another thread's state.
    |                  ^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::DefWindowProcW` is unsafe and requires unsafe block
   --> notchA\src\taskbar_native.rs:510:5
    |
510 |     DefWindowProcW(window, message, wparam, lparam)
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::PushAxisAlignedClip` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:228:9
    |
228 | /         self.context.PushAxisAlignedClip(&D2D_RECT_F { left:slot.x, top:0.0,
229 | |             right:slot.x + slot.width, bottom:self.height as f32 }, D2D1_ANTIALIAS_MODE_ALIASED);
    | |________________________________________________________________________________________________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_render.rs:226:5
    |
226 | /     unsafe fn draw_cell(&self, item: &Indicator, slot: &Slot, frame: &Frame, colours: &Palette, pressed: bool,
227 | |         seconds: f32, origin: (f32, f32), pct: &IDWriteTextFormat, letter_format: &IDWriteTextFormat) -> Result<()> {
    | |___________________________________________________________________________________________________________________^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::SetTransform` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:233:9
    |
233 |         self.context.SetTransform(&matrix(s, origin.0 + cx * (1.0 - s), origin.1 + cy * (1.0 - s)));
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::faded` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:238:9
    |
238 | /         self.faded(item.stale, || {
239 | |             self.brush.SetColor(&colours.hole);
240 | |             self.brush.SetOpacity(1.0);
241 | |             self.context.FillEllipse(&circle(center, 22.0 * u), &self.brush);
...   |
257 | |             Ok(())
258 | |         })?;
    | |__________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:239:13
    |
239 |             self.brush.SetColor(&colours.hole);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:240:13
    |
240 |             self.brush.SetOpacity(1.0);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::FillEllipse` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:241:13
    |
241 |             self.context.FillEllipse(&circle(center, 22.0 * u), &self.brush);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:242:13
    |
242 |             self.brush.SetColor(&colours.track);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawEllipse` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:243:13
    |
243 |             self.context.DrawEllipse(&circle(center, 25.0 * u), &self.brush, 5.0 * u, &self.round);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:248:17
    |
248 |                 self.brush.SetColor(&colours.track);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:249:17
    |
249 |                 self.brush.SetOpacity(0.7);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::dashed` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:250:29
    |
250 |                 if dashed { self.dashed(center, r, 1.0, 2.4 * u, u)?; }
    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawEllipse` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:251:24
    |
251 |                 else { self.context.DrawEllipse(&circle(center, r), &self.brush, 2.4 * u, &self.flat); }
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:252:17
    |
252 |                 self.brush.SetColor(&tone(week, frame));
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:253:17
    |
253 |                 self.brush.SetOpacity(0.85);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::dashed` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:254:29
    |
254 |                 if dashed { self.dashed(center, r, fraction, 2.4 * u, u)?; }
    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::arc` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:255:24
    |
255 |                 else { self.arc(center, r, 0.0, fraction, 2.4 * u, &self.round)?; }
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::faded` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:261:13
    |
261 | / ...   self.faded(item.stale, || {
262 | | ...       self.brush.SetColor(&tone(used, frame));
263 | | ...       self.brush.SetOpacity(1.0);
264 | | ...       self.arc(center, 25.0 * u, 0.0, taskbar_model::meter_fraction(used, remaining).clamp(0.0, 1.0), 5.0 * u, &self.round)
265 | | ...   })?;
    | |________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:262:17
    |
262 |                 self.brush.SetColor(&tone(used, frame));
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:263:17
    |
263 |                 self.brush.SetOpacity(1.0);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::arc` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:264:17
    |
264 |                 self.arc(center, 25.0 * u, 0.0, taskbar_model::meter_fraction(used, remaining).clamp(0.0, 1.0), 5.0 * u, &self.round)
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:272:17
    |
272 |                 self.brush.SetColor(&colours.ink);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:273:17
    |
273 |                 self.brush.SetOpacity(1.0);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::arc` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:274:17
    |
274 |                 self.arc(center, 19.0 * u, step, 0.28, 2.5 * u, &self.round)?;
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:277:17
    |
277 |                 self.brush.SetColor(&colours.watch);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:278:17
    |
278 |                 self.brush.SetOpacity(pulse(seconds));
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawEllipse` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:279:17
    |
279 |                 self.context.DrawEllipse(&circle(center, 19.0 * u), &self.brush, 2.5 * u, &self.flat);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::faded` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:284:9
    |
284 | /         self.faded(item.spent || item.stale, || {
285 | |             if let Some(documents) = self.logos.get(item.provider.as_str()) {
286 | |                 let size = 26.0 * u;
287 | |                 self.context.SetTransform(&matrix(s * size / 24.0, origin.0 + cx - s * size / 2.0, origin.1 + cy - s * size / 2.0));
...   |
298 | |             Ok(())
299 | |         })?;
    | |__________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::SetTransform` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:287:17
    |
287 |                 self.context.SetTransform(&matrix(s * size / 24.0, origin.0 + cx - s * size / 2.0, origin.1 + cy - s * size / 2.0));
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1DeviceContext5::DrawSvgDocument` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:288:17
    |
288 |                 self.svg_context.DrawSvgDocument(&documents[usize::from(!frame.dark)]);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::SetTransform` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:289:17
    |
289 |                 self.context.SetTransform(&matrix(s, origin.0 + cx * (1.0 - s), origin.1 + cy * (1.0 - s)));
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:291:17
    |
291 |                 self.brush.SetColor(&colours.glyph);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:292:17
    |
292 |                 self.brush.SetOpacity(1.0);
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawText` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:295:17
    |
295 | / ...   self.context.DrawText(&text, letter_format, &D2D_RECT_F { left:cx - half, top:cy - half, right:cx + half, bottom:cy + half },
296 | | ...       &self.brush, D2D1_DRAW_TEXT_OPTIONS_CLIP, DWRITE_MEASURING_MODE_NATURAL);
    | |__________________________________________________________________________________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::SetTransform` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:300:9
    |
300 |         self.context.SetTransform(&matrix(1.0, origin.0, origin.1));
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::DirectWrite::IDWriteFactory::CreateTextLayout` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:305:26
    |
305 |             let layout = self.write_factory.CreateTextLayout(&text, pct, (right - slot.text_x).max(1.0), self.height as f32)?;
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::DirectWrite::IDWriteTextLayout::SetTypography` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:306:13
    |
306 |             layout.SetTypography(&self.tabular, DWRITE_TEXT_RANGE { startPosition:0, length:text.len() as u32 })?;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush::SetColor` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:307:13
    |
307 |             self.brush.SetColor(&colours.ink);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Brush::SetOpacity` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:308:13
    |
308 |             self.brush.SetOpacity(1.0);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawTextLayout` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:309:13
    |
309 |             self.context.DrawTextLayout(D2D_POINT_2F { x:slot.text_x, y:0.0 }, &layout, &self.brush, D2D1_DRAW_TEXT_OPTIONS_CLIP);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::PopAxisAlignedClip` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:311:9
    |
311 |         self.context.PopAxisAlignedClip();
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1DeviceContext::PushLayer` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:321:9
    |
321 |         self.context.PushLayer(&parameters, None::<&ID2D1Layer>);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_render.rs:316:5
    |
316 |     unsafe fn faded(&self, faded: bool, paint: impl FnOnce() -> Result<()>) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::PopLayer` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:323:9
    |
323 |         self.context.PopLayer();
    |         ^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawEllipse` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:331:30
    |
331 |         if fraction >= 1.0 { self.context.DrawEllipse(&circle(center, radius), &self.brush, width, style); return Ok(()); }
    |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_render.rs:329:5
    |
329 |     unsafe fn arc(&self, center: D2D_POINT_2F, radius: f32, start: f32, fraction: f32, width: f32, style: &ID2D1StrokeStyle) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1Factory::CreatePathGeometry` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:336:20
    |
336 |         let path = self.factory.CreatePathGeometry()?;
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1PathGeometry::Open` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:337:20
    |
337 |         let sink = path.Open()?;
    |                    ^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::Common::ID2D1SimplifiedGeometrySink::BeginFigure` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:338:9
    |
338 |         sink.BeginFigure(at(start), D2D1_FIGURE_BEGIN_HOLLOW);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1GeometrySink::AddArc` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:341:13
    |
341 | /             sink.AddArc(&D2D1_ARC_SEGMENT { point:at(end), size:D2D_SIZE_F { width:radius, height:radius },
342 | |                 rotationAngle:0.0, sweepDirection:D2D1_SWEEP_DIRECTION_CLOCKWISE, arcSize:D2D1_ARC_SIZE_SMALL });
    | |________________________________________________________________________________________________________________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::Common::ID2D1SimplifiedGeometrySink::EndFigure` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:344:9
    |
344 |         sink.EndFigure(D2D1_FIGURE_END_OPEN);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::Common::ID2D1SimplifiedGeometrySink::Close` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:345:9
    |
345 |         sink.Close()?;
    |         ^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1RenderTarget::DrawGeometry` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:346:9
    |
346 |         self.context.DrawGeometry(&path, &self.brush, width, style);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `taskbar_render::Renderer::arc` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:353:13
    |
353 |             self.arc(center, radius, start / TAU, sweep, width, &self.flat)?;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_render.rs:351:5
    |
351 |     unsafe fn dashed(&self, center: D2D_POINT_2F, radius: f32, fraction: f32, width: f32, unit: f32) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::DirectWrite::IDWriteFactory::CreateTextFormat` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:383:18
    |
383 |       let format = factory.CreateTextFormat(w!("Segoe UI"), None::<&IDWriteFontCollection>,
    |  __________________^
384 | |         weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, size, w!("en-US"))?;
    | |________________________________________________________________________________________^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_render.rs:382:1
    |
382 | unsafe fn text_format(factory: &IDWriteFactory, weight: DWRITE_FONT_WEIGHT, size: f32) -> Result<IDWriteTextFormat> {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::DirectWrite::IDWriteTextFormat::SetTextAlignment` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:386:5
    |
386 |     format.SetTextAlignment(if weight == DWRITE_FONT_WEIGHT_BOLD { DWRITE_TEXT_ALIGNMENT_CENTER } else { DWRITE_TEXT_ALIGNMENT_LEADING })?;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::DirectWrite::IDWriteTextFormat::SetParagraphAlignment` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:387:5
    |
387 |     format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::DirectWrite::IDWriteTextFormat::SetWordWrapping` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:388:5
    |
388 |     format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::Shell::SHCreateMemStream` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:397:26
    |
397 |             let stream = SHCreateMemStream(Some(xml.as_bytes()))
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
   --> notchA\src\taskbar_render.rs:392:1
    |
392 | unsafe fn create_logos(context: &ID2D1DeviceContext5) -> Result<HashMap<&'static str, [ID2D1SvgDocument; 2]>> {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::Graphics::Direct2D::ID2D1DeviceContext5::CreateSvgDocument` is unsafe and requires unsafe block
   --> notchA\src\taskbar_render.rs:399:13
    |
399 |             context.CreateSvgDocument(&stream, D2D_SIZE_F { width:24.0, height:24.0 })
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
    |
    = note: consult the function's documentation for information on how to avoid undefined behavior
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
  --> notchA\src\topmost.rs:58:26
   |
58 |         let state = &mut *(l.0 as *mut State);
   |                          ^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer
   |
   = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
  --> notchA\src\topmost.rs:57:5
   |
57 |     unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::IsWindowVisible` is unsafe and requires unsafe block
  --> notchA\src\topmost.rs:59:13
   |
59 |         if !IsWindowVisible(hwnd).as_bool() {
   |             ^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning[E0133]: call to unsafe function `windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW` is unsafe and requires unsafe block
  --> notchA\src\topmost.rs:66:18
   |
66 |         let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

warning: variable does not need to be mutable
   --> notchA\src\tray.rs:114:13
    |
114 |         let mut guard = tray_slot().lock().unwrap();
    |             ----^^^^^
    |             |
    |             help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

error[E0515]: cannot return value referencing local variable `payload`
   --> notchA\src\main.rs:666:24
    |
666 |                 return reset_view::view(&payload, self.notch_ui.dark).map(Msg::Reset);
    |                        ^^^^^^^^^^^^^^^^^--------^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |                        |                |
    |                        |                `payload` is borrowed here
    |                        returns a value referencing data owned by the current function

warning: unused variable: `rx`
   --> notchA\src\main.rs:726:21
    |
726 |         if let Some(rx) = self.rx.lock().unwrap().take() {
    |                     ^^ help: if this is intentional, prefix it with an underscore: `_rx`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

error[E0433]: cannot find module or crate `tauri` in this scope
   --> notchA\src\reset_watch.rs:141:19
    |
141 | pub fn start(app: tauri::AppHandle) {
    |                   ^^^^^ use of unresolved module or unlinked crate `tauri`
    |
    = help: if you wanted to use a crate named `tauri`, use `cargo add tauri` to add it to your `Cargo.toml`

Some errors have detailed explanations: E0061, E0119, E0133, E0277, E0282, E0423, E0432, E0433, E0505...
For more information about an error, try `rustc --explain E0061`.
warning: `notchA` (bin "notchA") generated 107 warnings
error: could not compile `notchA` (bin "notchA") due to 33 previous errors; 107 warnings emitted
warning: unused dependency `iced_anim`
  --> notchA\Cargo.toml:10:1
   |
10 | iced_anim = "0.3"
   | ^^^^^^^^^
   |
   = note: `cargo::unused_dependencies` is set to `warn` by default
help: consider removing the dependency on `iced_anim`
warning: `notchA` (manifest) generated 1 warning

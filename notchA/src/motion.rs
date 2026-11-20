//! Motion primitives shared by the notch, carry overlay and taskbar renderer.
//!
//! Ported from the reference `notch.html` / `carry.html` CSS curves and JS springs.
//! Everything here is pure math and is unit-testable without a window.

use std::time::{Duration, Instant};

pub const FOLD_MS: u32 = 360;
pub const CARD_MS: u32 = 300;
pub const CARD_FALLBACK_MS: u32 = 400;
pub const PRESS_IN_MS: u32 = 120;
pub const PRESS_OUT_MS: u32 = 430;
pub const TURN_MS: u32 = 950;
pub const PALETTE_MS: u32 = 200;
pub const GLIDE_MS: u32 = 360;
pub const RELEASE_MS: u32 = 260;

/// A named CSS cubic-bezier, evaluated with a bisection solve on x.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Curve {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Curve {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// CSS `cubic-bezier(x1,y1,x2,y2)` evaluated at progress `t`.
    pub fn at(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let sample = |a: f32, b: f32, s: f32| {
            3.0 * a * s * (1.0 - s) * (1.0 - s) + 3.0 * b * s * s * (1.0 - s) + s * s * s
        };
        if self.x1 == self.y1 && self.x2 == self.y2 {
            return sample(self.y1, self.y2, t);
        }
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            let mid = (lo + hi) * 0.5;
            if sample(self.x1, self.x2, mid) < t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        sample(self.y1, self.y2, (lo + hi) * 0.5)
    }
}

pub const FOLD: Curve = Curve::new(0.32, 0.72, 0.24, 1.0);
pub const CARD: Curve = Curve::new(0.33, 1.0, 0.68, 1.0);
pub const ARC_DIVIDE: Curve = Curve::new(0.32, 0.0, 0.14, 1.0);
pub const EASE_IN_OUT: Curve = Curve::new(0.42, 0.0, 0.58, 1.0);
pub const EASE_OUT: Curve = Curve::new(0.0, 0.0, 0.58, 1.0);
pub const RELEASE: Curve = Curve::new(0.0, 0.0, 0.2, 1.0);
pub const FOLLOW: Curve = Curve::new(0.2, 0.84, 0.2, 1.0);
pub const SETTLE: Curve = Curve::new(0.3, 0.74, 0.3, 1.0);

/// Interpolates `from` to `to` with a named curve. Retargeting keeps the current
/// value as the new start, so changing cells mid-glide never snaps.
#[derive(Debug, Clone, Copy)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub start: Instant,
    pub duration: Duration,
    pub curve: Curve,
}

impl Default for Tween {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            start: Instant::now(),
            duration: Duration::ZERO,
            curve: EASE_IN_OUT,
        }
    }
}

impl Tween {
    pub fn new(to: f32, duration: u32, curve: Curve) -> Self {
        Self {
            from: to,
            to,
            start: Instant::now(),
            duration: Duration::from_millis(duration as u64),
            curve,
        }
    }

    pub fn value(&self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return self.to;
        }
        let t = now.saturating_duration_since(self.start).as_secs_f32()
            / self.duration.as_secs_f32();
        if t >= 1.0 {
            return self.to;
        }
        self.from + (self.to - self.from) * self.curve.at(t)
    }

    pub fn done(&self, now: Instant) -> bool {
        self.duration.is_zero() || now.saturating_duration_since(self.start) >= self.duration
    }

    /// Retarget from the value currently on screen.
    pub fn retarget(&mut self, to: f32, duration: u32, curve: Curve, now: Instant) {
        if (self.to - to).abs() < 0.0005 {
            return;
        }
        self.from = self.value(now);
        self.to = to;
        self.start = now;
        self.duration = Duration::from_millis(duration as u64);
        self.curve = curve;
    }

    pub fn jump(&mut self, to: f32) {
        self.from = to;
        self.to = to;
        self.start = Instant::now();
        self.duration = Duration::ZERO;
    }
}

/// Semi-implicit Euler spring, matching the reference `springStep`.
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    pub response: f32,
    pub damping: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self { value: 0.0, velocity: 0.0, target: 0.0, response: 22.0, damping: 0.8 }
    }
}

impl Spring {
    pub fn new(response: f32, damping: f32) -> Self {
        Self { response, damping, ..Self::default() }
    }

    pub fn step(&mut self, dt: f32) -> f32 {
        let dt = dt.clamp(0.0, 0.05);
        let accel = (self.target - self.value) * self.response * self.response
            - 2.0 * self.damping * self.response * self.velocity;
        self.velocity += accel * dt;
        self.value += self.velocity * dt;
        self.value
    }

    pub fn settled(&self) -> bool {
        (self.target - self.value).abs() < 0.002 && self.velocity.abs() < 0.01
    }

    pub fn retarget(&mut self, target: f32) {
        self.target = target;
    }

    pub fn jump(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }
}

/// All animated scalars for the notch. Kept in `NotchUiState` so the canvas can
/// read the interpolated values without owning the clocks.
#[derive(Debug, Clone)]
pub struct Anim {
    pub fold: Tween,
    pub card: Tween,
    pub orb: Tween,
    pub palette: Tween,
    pub turn: Tween,
    pub release: Tween,
    pub grip: Spring,
    pub press: Spring,
    pub card_pos: Spring,
    pub tail_pos: Spring,
    pub card_h: Spring,
    pub fold_t: f32,
    pub card_t: f32,
    pub orb_t: f32,
    pub palette_t: f32,
    pub turn_t: f32,
    pub release_t: f32,
    pub grip_out: f32,
    pub press_t: f32,
    pub card_pos_t: f32,
    pub tail_pos_t: f32,
    pub card_h_t: f32,
}

impl Default for Anim {
    fn default() -> Self {
        Self {
            fold: Tween::new(0.0, 0, EASE_IN_OUT),
            card: Tween::new(0.0, 0, CARD),
            orb: Tween::new(0.0, 0, EASE_OUT),
            palette: Tween::new(0.0, 0, EASE_IN_OUT),
            turn: Tween::new(0.0, 0, ARC_DIVIDE),
            release: Tween::new(0.0, 0, RELEASE),
            grip: Spring::new(24.0, 0.76),
            press: Spring::new(34.0, 0.9),
            card_pos: Spring::new(20.0, 0.85),
            tail_pos: Spring::new(28.0, 0.78),
            card_h: Spring::new(24.0, 0.85),
            fold_t: 0.0,
            card_t: 0.0,
            orb_t: 0.0,
            palette_t: 0.0,
            turn_t: 0.0,
            release_t: 0.0,
            grip_out: 0.0,
            press_t: 0.0,
            card_pos_t: 0.0,
            tail_pos_t: 0.0,
            card_h_t: 160.0,
        }
    }
}

impl Anim {
    pub fn set_folded(&mut self, folded: bool, now: Instant, enabled: bool) {
        self.fold.retarget(folded as u8 as f32, FOLD_MS, FOLD, now);
        if !enabled {
            self.fold.jump(folded as u8 as f32);
        }
    }

    pub fn set_card(&mut self, open: bool, now: Instant, enabled: bool) {
        self.card.retarget(open as u8 as f32, CARD_MS, CARD, now);
        if !enabled {
            self.card.jump(open as u8 as f32);
        }
    }

    pub fn retarget_card_pos(&mut self, pos: f32, height: f32, instant: bool) {
        if instant || self.card_t < 0.01 {
            self.card_pos.jump(pos);
            self.tail_pos.jump(pos);
            self.card_h.jump(height);
            self.card_pos_t = pos;
            self.tail_pos_t = pos;
            self.card_h_t = height;
        } else {
            self.card_pos.retarget(pos);
            self.tail_pos.retarget(pos);
            self.card_h.retarget(height);
        }
    }

    pub fn set_orb(&mut self, hovered: bool, now: Instant, enabled: bool) {
        self.orb.retarget(hovered as u8 as f32, 180, EASE_OUT, now);
        if !enabled {
            self.orb.jump(hovered as u8 as f32);
        }
    }

    pub fn set_palette(&mut self, light: bool, now: Instant, enabled: bool) {
        self.palette.retarget(light as u8 as f32, PALETTE_MS, EASE_IN_OUT, now);
        if !enabled {
            self.palette.jump(light as u8 as f32);
        }
    }

    pub fn set_press(&mut self, pressed: bool) {
        self.press.retarget(pressed as u8 as f32);
    }

    pub fn set_grip(&mut self, out: bool) {
        self.grip.retarget(out as u8 as f32);
    }

    pub fn turn(&mut self, seconds: f32, now: Instant, enabled: bool) {
        self.turn.retarget(seconds, TURN_MS, ARC_DIVIDE, now);
        if !enabled {
            self.turn.jump(seconds);
        }
    }

    pub fn release(&mut self, now: Instant, enabled: bool) {
        self.release.retarget(1.0, RELEASE_MS, RELEASE, now);
        if !enabled {
            self.release.jump(1.0);
        }
    }

    /// Advances every animation. `dt` is in seconds.
    pub fn tick(&mut self, now: Instant, dt: f32) {
        let unit = |v: f32| v.clamp(0.0, 1.0);
        self.fold_t = unit(self.fold.value(now));
        self.card_t = unit(self.card.value(now));
        self.orb_t = unit(self.orb.value(now));
        self.palette_t = unit(self.palette.value(now));
        self.turn_t = self.turn.value(now);
        self.release_t = unit(self.release.value(now));
        self.grip_out = unit(self.grip.step(dt));
        self.press_t = unit(self.press.step(dt));
        self.card_pos_t = self.card_pos.step(dt);
        self.tail_pos_t = self.tail_pos.step(dt);
        self.card_h_t = self.card_h.step(dt);
        // The release arc is a one-shot pulse; clear it once it has played.
        if self.release.to > 0.0 && self.release.done(now) {
            self.release.jump(0.0);
            self.release_t = 0.0;
        }
    }

    pub fn busy(&self, now: Instant) -> bool {
        !self.fold.done(now)
            || !self.card.done(now)
            || !self.orb.done(now)
            || !self.palette.done(now)
            || !self.turn.done(now)
            || !self.release.done(now)
            || !self.grip.settled()
            || !self.press.settled()
            || !self.card_pos.settled()
            || !self.tail_pos.settled()
            || !self.card_h.settled()
    }
}

/// Per-cell refresh press, matching the reference's 120 ms in / 430 ms out.
pub fn cell_press(started: Instant, now: Instant, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    let ms = now.saturating_duration_since(started).as_millis() as u32;
    if ms < PRESS_IN_MS {
        EASE_OUT.at(ms as f32 / PRESS_IN_MS as f32)
    } else {
        1.0 - RELEASE.at((ms - PRESS_IN_MS) as f32 / PRESS_OUT_MS as f32)
    }
}

/// Native window glide curve (the old Tauri `glide_curve`, now shared).
pub fn glide(t: f32) -> f32 {
    FOLD.at(t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(windows)]
pub fn motion_enabled() -> bool {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SYSTEM_PARAMETERS_INFO_ACTION,
    };
    let mut enabled = BOOL(1);
    // SPI_GETCLIENTAREAANIMATION
    unsafe {
        let _ = SystemParametersInfoW(
            SYSTEM_PARAMETERS_INFO_ACTION(0x1042),
            0,
            Some(&mut enabled as *mut _ as *mut std::ffi::c_void),
            Default::default(),
        );
    }
    enabled.as_bool()
}

#[cfg(not(windows))]
pub fn motion_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_are_monotonic_and_bounded() {
        for curve in [FOLD, CARD, ARC_DIVIDE, EASE_IN_OUT, EASE_OUT, RELEASE, FOLLOW, SETTLE] {
            let mut prev = -1.0;
            for i in 0..=20 {
                let v = curve.at(i as f32 / 20.0);
                assert!((0.0..=1.0).contains(&v), "{curve:?} out of range: {v}");
                assert!(v + 0.001 >= prev, "{curve:?} not monotonic at {i}");
                prev = v;
            }
        }
    }

    #[test]
    fn tween_retarget_starts_from_current_value() {
        let now = Instant::now();
        let mut t = Tween::new(0.0, 0, EASE_IN_OUT);
        t.retarget(1.0, 100, EASE_IN_OUT, now);
        assert!(t.value(now) < 0.01);
        assert!(t.value(now + Duration::from_millis(100)) > 0.99);
    }

    #[test]
    fn spring_settles() {
        let mut s = Spring::new(22.0, 0.8);
        s.retarget(1.0);
        for _ in 0..240 {
            s.step(1.0 / 60.0);
        }
        assert!(s.settled(), "value={} vel={}", s.value, s.velocity);
    }
}

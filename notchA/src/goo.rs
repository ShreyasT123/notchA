//! Shared metaball/goo geometry for the notch orb, grip and carried handle.
//!
//! The reference UI uses an SVG blur+threshold filter. A real gaussian
//! threshold pass would need a custom wgpu pipeline, so this module builds
//! equivalent *analytic* silhouettes: overlapping discs joined by tapered
//! quadratic bridges. Both the notch and the carry overlay call the same
//! helpers, which keeps the handover frame continuous.

use iced::widget::canvas::{path, Path};
use iced::{Point, Radians, Rectangle};
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::TAU;

pub const MAX_STRETCH: f32 = 0.14;
pub const STRETCH_SPEED: f32 = 2600.0;

pub fn disc(center: Point, radius: f32) -> Path {
    Path::circle(center, radius.max(0.1))
}

/// Quarter arc tucked into a fillet, used by the resting orb.
pub fn quarter(center: Point, radius: f32, start_turns: f32, span_turns: f32) -> Path {
    let mut b = path::Builder::new();
    let a0 = (start_turns - 0.25) * TAU;
    b.arc(path::Arc {
        center,
        radius,
        start_angle: Radians(a0),
        end_angle: Radians(a0 + span_turns * TAU),
    });
    b.build()
}

/// Tapered bridge between two discs. `pull` controls how far the neck pinches
/// inward (1.0 = straight capsule, 0.0 = fully separated).
pub fn strand(a: Point, b: Point, ra: f32, rb: f32, pull: f32) -> Path {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let nx = -dy / len;
    let ny = dx / len;
    let a1 = Point::new(a.x + nx * ra, a.y + ny * ra);
    let a2 = Point::new(a.x - nx * ra, a.y - ny * ra);
    let b1 = Point::new(b.x + nx * rb, b.y + ny * rb);
    let b2 = Point::new(b.x - nx * rb, b.y - ny * rb);

    let waist = ((ra + rb) * 0.5 * pull).max(0.4);
    let mid = Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let m1 = Point::new(mid.x + nx * waist, mid.y + ny * waist);
    let m2 = Point::new(mid.x - nx * waist, mid.y - ny * waist);

    let mut builder = path::Builder::new();
    builder.move_to(a1);
    builder.quadratic_curve_to(m1, b1);
    builder.line_to(b2);
    builder.quadratic_curve_to(m2, a2);
    builder.close();
    builder.build()
}

/// Approximate the SVG filter's `discMerge`: a big disc plus necks to the
/// individual grip dots, so the six dots read as one liquid grip.
pub fn disc_merge(
    center: Point,
    dots: &[Point],
    radius: f32,
    dot_radius: f32,
    wetness: f32,
) -> Vec<Path> {
    let mut paths = Vec::with_capacity(dots.len() * 2 + 1);
    paths.push(disc(center, radius));
    for dot in dots {
        let dist = ((dot.x - center.x).powi(2) + (dot.y - center.y).powi(2)).sqrt();
        let neck = ((1.0 - (dist - radius - dot_radius) / 18.0).clamp(0.0, 1.0) * wetness).max(0.0);
        if neck > 0.02 {
            paths.push(strand(center, *dot, radius * 0.82, dot_radius * 1.1, neck));
        }
        paths.push(disc(*dot, dot_radius));
    }
    paths
}

/// Six-dot grip layout, identical to the HTML reference.
pub fn grip_dots(center: Point, reach: f32, vertical: bool) -> [Point; 6] {
    let mut out = [center; 6];
    let mut i = 0;
    for line in [-0.5f32, 0.5] {
        for row in [-1.0f32, 0.0, 1.0] {
            let (ox, oy) = if vertical {
                (row * reach, line * reach)
            } else {
                (line * reach, row * reach)
            };
            out[i] = Point::new(center.x + ox, center.y + oy);
            i += 1;
        }
    }
    out
}

/// Velocity-driven stretch of the carried silhouette, clamped so a fast
/// flick cannot tear the shape apart.
pub fn stretch_rect(rect: Rectangle, edge: &str, vx: f32, vy: f32) -> Rectangle {
    let speed = (vx * vx + vy * vy).sqrt();
    let k = (speed / STRETCH_SPEED).clamp(0.0, 1.0) * MAX_STRETCH;
    let along_x = !matches!(edge, "left" | "right");
    let (dx, dy) = if along_x { (vx, 0.0) } else { (0.0, vy) };
    let grow = k * if along_x { rect.width } else { rect.height };
    let dir = if dx + dy >= 0.0 { 1.0 } else { -1.0 };
    if along_x {
        let shift = if dir > 0.0 { 0.0 } else { grow };
        Rectangle::new(
            Point::new(rect.x - shift, rect.y),
            iced::Size::new(rect.width + grow, rect.height),
        )
    } else {
        let shift = if dir > 0.0 { 0.0 } else { grow };
        Rectangle::new(
            Point::new(rect.x, rect.y - shift),
            iced::Size::new(rect.width, rect.height + grow),
        )
    }
}

/// Corner morph used while the carried notch approaches a new screen edge.
pub fn corner_radius(base: f32, morph: f32) -> f32 {
    base * (1.0 - 0.45 * morph.clamp(0.0, 1.0))
}

/// Arc that grows out of the orb on release (disc-to-arc divide).
pub fn release_arc(center: Point, radius: f32, phase: f32) -> Path {
    let span = (0.08 + 0.92 * phase.clamp(0.0, 1.0)) * TAU;
    let mut b = path::Builder::new();
    b.arc(path::Arc {
        center,
        radius,
        start_angle: Radians(-FRAC_PI_2),
        end_angle: Radians(-FRAC_PI_2 + span),
    });
    b.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grip_layout_has_six_dots() {
        let dots = grip_dots(Point::new(10.0, 10.0), 9.9, false);
        assert_eq!(dots.len(), 6);
        assert!((dots[0].x - 5.05).abs() < 0.001);
    }

    #[test]
    fn stretch_is_clamped() {
        let r = Rectangle::new(Point::ORIGIN, iced::Size::new(100.0, 50.0));
        let s = stretch_rect(r, "top", 1_000_000.0, 0.0);
        assert!(s.width <= r.width * (1.0 + MAX_STRETCH) + 0.001);
    }
}

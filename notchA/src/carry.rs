use crate::core::{self, AppHandle, AppState, Manager, Screen};
use std::sync::atomic::Ordering::SeqCst;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

impl Edge {
    const ALL: [Edge; 4] = [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];
    pub fn parse(s: &str) -> Edge {
        match s {
            "top" => Edge::Top,
            "bottom" => Edge::Bottom,
            "left" => Edge::Left,
            _ => Edge::Right,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Edge::Top => "top",
            Edge::Right => "right",
            Edge::Bottom => "bottom",
            Edge::Left => "left",
        }
    }
    pub fn is_vertical(self) -> bool {
        matches!(self, Edge::Left | Edge::Right)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopRight,
    BottomRight,
    BottomLeft,
    TopLeft,
}

impl Corner {
    const ALL: [Corner; 4] = [Corner::TopRight, Corner::BottomRight, Corner::BottomLeft, Corner::TopLeft];
    pub fn edges(self) -> (Edge, Edge) {
        match self {
            Corner::TopRight => (Edge::Top, Edge::Right),
            Corner::BottomRight => (Edge::Right, Edge::Bottom),
            Corner::BottomLeft => (Edge::Bottom, Edge::Left),
            Corner::TopLeft => (Edge::Left, Edge::Top),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderTrack {
    pub width: f64,
    pub height: f64,
}

impl BorderTrack {
    pub fn perimeter(&self) -> f64 { 2.0 * (self.width + self.height) }
    pub fn corner_position(&self, corner: Corner) -> f64 {
        match corner {
            Corner::TopRight => self.width,
            Corner::BottomRight => self.width + self.height,
            Corner::BottomLeft => 2.0 * self.width + self.height,
            Corner::TopLeft => 0.0,
        }
    }
    pub fn position_on(&self, edge: Edge, x: f64, y: f64) -> f64 {
        let (w, h) = (self.width, self.height);
        match edge {
            Edge::Top => x.clamp(0.0, w),
            Edge::Right => w + y.clamp(0.0, h),
            Edge::Bottom => 2.0 * w + h - x.clamp(0.0, w),
            Edge::Left => self.wrapped(2.0 * w + 2.0 * h - y.clamp(0.0, h)),
        }
    }
    pub fn place_at(&self, position: f64) -> (Edge, f64) {
        let (w, h) = (self.width, self.height);
        let s = self.wrapped(position);
        if s < w { (Edge::Top, s) }
        else if s < w + h { (Edge::Right, s - w) }
        else if s < 2.0 * w + h { (Edge::Bottom, 2.0 * w + h - s) }
        else { (Edge::Left, 2.0 * w + 2.0 * h - s) }
    }
    pub fn wrapped(&self, position: f64) -> f64 { position.rem_euclid(self.perimeter()) }
    pub fn signed(&self, d: f64) -> f64 {
        let p = self.perimeter();
        let d = d.rem_euclid(p);
        if d > p / 2.0 { d - p } else { d }
    }
}

const EDGE_SWITCH_MARGIN: f64 = 40.0;
const CORNER_REACH: f64 = 160.0;
const CROSS: f64 = 150.0;

fn distance_from(edge: Edge, x: f64, y: f64, track: &BorderTrack) -> f64 {
    match edge {
        Edge::Top => y,
        Edge::Bottom => track.height - y,
        Edge::Left => x,
        Edge::Right => track.width - x,
    }
}

pub fn sticky_edge(current: Edge, x: f64, y: f64, track: &BorderTrack) -> Edge {
    let nearest = Edge::ALL
        .into_iter()
        .min_by(|a, b| distance_from(*a, x, y, track).total_cmp(&distance_from(*b, x, y, track)))
        .unwrap_or(current);
    if distance_from(nearest, x, y, track) + EDGE_SWITCH_MARGIN < distance_from(current, x, y, track) {
        nearest
    } else {
        current
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Edge(Edge),
    Corner(Corner),
}

pub fn reading(x: f64, y: f64, track: &BorderTrack, nearest: Edge) -> Reading {
    Corner::ALL
        .into_iter()
        .find(|c| {
            let (before, after) = c.edges();
            distance_from(before, x, y, track).max(0.0) < CORNER_REACH
                && distance_from(after, x, y, track).max(0.0) < CORNER_REACH
        })
        .map_or(Reading::Edge(nearest), Reading::Corner)
}

pub fn place(x: f64, y: f64, track: &BorderTrack, reading: Reading) -> f64 {
    match reading {
        Reading::Edge(edge) => track.position_on(edge, x, y),
        Reading::Corner(corner) => {
            let (before, after) = corner.edges();
            track.wrapped(
                track.corner_position(corner) + distance_from(before, x, y, track).max(0.0)
                    - distance_from(after, x, y, track).max(0.0),
            )
        }
    }
}

fn track_of(s: &Screen) -> BorderTrack {
    let (_, _, aw, ah) = s.area();
    BorderTrack { width: aw as f64 / s.scale, height: ah as f64 / s.scale }
}

fn local(s: &Screen, x: f64, y: f64) -> (f64, f64) {
    let (ax, ay, _, _) = s.area();
    ((x - ax as f64) / s.scale, (y - ay as f64) / s.scale)
}

fn beyond(s: &Screen, x: f64, y: f64) -> f64 {
    let (left, top, right, bottom) = (s.x as f64, s.y as f64, (s.x + s.w) as f64, (s.y + s.h) as f64);
    let dx = (left - x).max(x - right).max(0.0);
    let dy = (top - y).max(y - bottom).max(0.0);
    dx.max(dy)
}

pub fn begin_carry(app: AppHandle) {
    if core::DRAGGING.swap(true, SeqCst) {
        core::carry_visual(&app, true);
        return;
    }
    std::thread::spawn(move || {
        run(&app);
        core::carry_visual(&app, false);
        core::DRAGGING.store(false, SeqCst);
        core::place_notch(&app);
    });
}

fn run(app: &AppHandle) {
    let (Some(start), Some(notch)) = (core::target_screen(app), app.get_webview_window("notch")) else {
        return;
    };
    let (Ok(pos), Ok(win), Ok(cur)) = (notch.outer_position(), notch.outer_size(), app.cursor_position()) else {
        return;
    };
    let from = Edge::parse(&crate::config::edge_or_right(
        &app.state::<AppState>().cfg.lock().unwrap().notch_edge,
    ));
    let mut mon = start.clone();
    let mut track = track_of(&mon);
    let (mx, my) = local(&mon, pos.x + win.width as f64 / 2.0, pos.y + win.height as f64 / 2.0);
    let middle = track.position_on(from, mx, my);
    let (px, py) = local(&mon, cur.x, cur.y);
    let mut pointer_edge = from;
    let mut read = reading(px, py, &track, from);
    let mut grip = track.signed(middle - place(px, py, &track, read));
    let all = core::screens();
    let mut last_target = middle;

    while core::left_button_down() {
        if let Ok(cur) = app.cursor_position() {
            if let Some(s) = core::screen_at(&all, cur.x, cur.y)
                .filter(|s| !core::same_screen(s, &mon) && beyond(&mon, cur.x, cur.y) / s.scale >= CROSS)
            {
                mon = s.clone();
                track = track_of(&mon);
                let (px, py) = local(&mon, cur.x, cur.y);
                pointer_edge = sticky_edge(pointer_edge, px, py, &track);
                read = reading(px, py, &track, pointer_edge);
                grip = 0.0;
            }
            let (px, py) = local(&mon, cur.x, cur.y);
            let edge = sticky_edge(pointer_edge, px, py, &track);
            let now = reading(px, py, &track, edge);
            if now != read {
                grip += track.signed(place(px, py, &track, read) - place(px, py, &track, now));
                read = now;
            }
            pointer_edge = edge;
            let target = track.wrapped(place(px, py, &track, read) + grip);
            if (target - last_target).abs() > 1.0 {
                last_target = target;
                let (new_edge, along) = track.place_at(target);
                let (ax, ay, aw, ah) = mon.area();
                let from_start = (along * mon.scale).round() as i32;
                let ratio = if new_edge.is_vertical() {
                    core::along_at(ay + from_start, 0, ay, ah)
                } else {
                    core::along_at(ax + from_start, 0, ax, aw)
                };
                {
                    let mut c = app.state::<AppState>().cfg.lock().unwrap();
                    c.notch_edge = new_edge.as_str().into();
                    c.set_along(new_edge.as_str(), ratio);
                    if !core::same_screen(&mon, &start) && mon.name.is_some() {
                        c.notch_monitor = mon.name.clone();
                    }
                }
                core::place_notch(app);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(12));
    }

    let c = app.state::<AppState>().cfg.lock().unwrap();
    crate::config::save(&c);
}
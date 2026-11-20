use crate::core::{self, AppHandle, AppState, Manager, NOTCH_LONG, NOTCH_W};
use crate::{activity, claude_auth, config, glyphs, goo, motion, state, theme, traymenu, usage};
use iced::alignment::Vertical;
use iced::mouse;
use iced::widget::canvas::{self, path, Frame, Geometry, LineCap, LineJoin, Path, Stroke};
use iced::widget::{button, column, container, image, mouse_area, row, space, stack, svg, text, Column};
use iced::{Border, Color, Element, Font, Length, Padding, Point, Radians, Rectangle, Renderer, Size, Theme};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum NotchMsg {
    PointerMoved(Point),
    PointerEntered,
    PointerExited,
    LeftPress,
    LeftRelease,
    RightPress,
    ClaudeSignIn,
}

#[derive(Debug, Clone)]
pub struct NotchCell {
    pub id: String,
    pub base: String,
    pub name: String,
    pub glyph_fallback: String,
    pub account_line: String,
    pub snap: usage::UsageSnapshot,
}

#[derive(Debug, Clone, Default)]
pub struct NotchUiState {
    pub edge: String,
    pub scale: f32,
    pub dark: bool,
    pub folded: bool,
    pub full_screen: bool,
    pub pointer_in: bool,
    pub menu_open: bool,
    pub behind: Option<String>,
    pub hover_id: Option<String>,
    pub card_open: bool,
    pub orb_hovered: bool,
    pub grip_hovered: bool,
    pub grip_out: f32,
    pub pressing: Option<(Point, String)>,
    pub refreshing_ids: std::collections::HashMap<String, std::time::Instant>,
    pub insets: [f32; 4],
    pub anim_secs: f32,
    pub anim: motion::Anim,
}

impl NotchUiState {
    pub fn is_vertical(&self) -> bool {
        config::edge_is_vertical(&self.edge)
    }

    pub fn window_logical_size(&self) -> (f32, f32) {
        let (w, h) = core::notch_window_size(&self.edge);
        (w as f32 * self.scale, h as f32 * self.scale)
    }
}

pub fn build_cells(app: &AppHandle) -> Vec<NotchCell> {
    let st = app.state::<AppState>();
    let cfg = st.cfg.lock().unwrap().clone();
    let usage = st.usage.lock().unwrap().clone();
    let codex = st.codex.lock().unwrap().clone();
    let copilot = st.copilot.lock().unwrap().clone();
    let ag = st.antigravity.lock().unwrap().clone();
    let opencode = st.opencode.lock().unwrap().clone();

    let mut list: Vec<NotchCell> = Vec::new();

    if !usage.accounts.is_empty() {
        let several = usage.accounts.len() > 1;
        for a in &usage.accounts {
            let plan = a.plan.as_deref().filter(|s| !s.is_empty()).map(|s| {
                let mut c = s.chars();
                match c.next() {
                    Some(first) => first.to_uppercase().chain(c).collect::<String>(),
                    None => String::new(),
                }
            });
            let account_line = if several {
                [a.address.as_deref().unwrap_or(""), plan.as_deref().unwrap_or("")]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" · ")
            } else {
                String::new()
            };
            list.push(NotchCell {
                id: if a.id == "default" { "claude".into() } else { format!("claude@{}", a.id) },
                base: "claude".into(),
                name: if a.name == "default" { "Claude".into() } else { a.name.clone() },
                glyph_fallback: "C".into(),
                account_line,
                snap: usage::UsageSnapshot {
                    status: a.status.clone(),
                    windows: a.windows.clone(),
                    fetched_at: a.fetched_at,
                    note: a.note.clone(),
                    backoff_until: a.backoff_until,
                    ..Default::default()
                },
            });
        }
    } else if usage.status != "absent" {
        list.push(NotchCell {
            id: "claude".into(),
            base: "claude".into(),
            name: "Claude".into(),
            glyph_fallback: "C".into(),
            account_line: String::new(),
            snap: usage,
        });
    }

    let others = [
        ("codex", "Codex", "Cx", codex),
        ("opencode", "OpenCode", "OC", opencode),
        ("copilot", "GitHub Copilot", "Cp", copilot),
        ("gemini", "Antigravity", "Ag", ag),
    ];

    for (id, name, glyph_fallback, snap) in others {
        if snap.status != "absent" {
            list.push(NotchCell {
                id: id.into(),
                base: id.into(),
                name: name.into(),
                glyph_fallback: glyph_fallback.into(),
                account_line: String::new(),
                snap,
            });
        }
    }

    if !cfg.notch_slots.is_empty() {
        let picked: Vec<NotchCell> = list
            .iter()
            .filter(|c| cfg.notch_slots.iter().any(|s| s.provider == c.base))
            .cloned()
            .collect();
        if !picked.is_empty() {
            return picked;
        }
    }
    list
}

pub fn work_state_of(base: &str, store_snap: &state::Snapshot, acts: &[activity::Activity]) -> &'static str {
    if base == "claude" {
        if store_snap.agg == state::ST_RUNNING { return "running"; }
        if store_snap.agg == state::ST_ATTENTION { return "attention"; }
        if acts.iter().any(|a| a.provider == "claude" && a.state == "busy") { return "running"; }
        return "idle";
    }
    let matching: Vec<_> = acts.iter().filter(|a| a.provider == base).collect();
    if matching.iter().any(|a| a.state == "waiting") {
        "attention"
    } else if matching.iter().any(|a| a.state == "busy") {
        "running"
    } else {
        "idle"
    }
}

pub fn weekly_window<'a>(
    provider: &str,
    windows: &'a [usage::LimitWindow],
    ag_model: &str,
) -> Option<&'a usage::LimitWindow> {
    let metered: Vec<_> = windows.iter().filter(|w| w.metered()).collect();
    let by_id = |id: &str| metered.iter().copied().find(|w| w.id == id);
    match provider {
        "claude" => by_id("seven_day").or_else(|| by_id("weekly_all")).or_else(|| by_id("weekly")),
        "codex" => by_id("secondary"),
        "opencode" => by_id("weekly"),
        "copilot" => None,
        _ => {
            let fam: Vec<_> = metered
                .iter()
                .copied()
                .filter(|w| core::lane_family(w) == ag_model)
                .collect();
            let lanes = if fam.is_empty() { metered } else { fam };
            core::tightest(lanes.into_iter().filter(|w| core::lane_is(w, "weekly")))
        }
    }
}

pub fn stale_of(snap: &usage::UsageSnapshot, now: u64) -> bool {
    snap.status == "stale"
        || (snap.fetched_at > 0 && now.saturating_sub(snap.fetched_at) > 15 * 60 * 1000)
}

pub fn claude_quota_current(snap: &usage::UsageSnapshot, w: &usage::LimitWindow, now: u64) -> bool {
    snap.status == "ok"
        && snap.fetched_at > 0
        && !stale_of(snap, now)
        && w.resets_at.is_none_or(|r| r > now)
}

pub fn pct_figure(snap: &usage::UsageSnapshot, head: Option<&usage::LimitWindow>, remaining: bool) -> String {
    if snap.status == "needsAuth" || snap.status == "none" {
        return "—".into();
    }
    match head {
        Some(w) if w.count.is_some() => format!("~{}", w.count.unwrap()),
        Some(w) if w.amount.is_some() => w.amount.clone().unwrap_or_default(),
        Some(w) if w.used.is_finite() => {
            let prefix = if w.derived { "~" } else { "" };
            let frac = if remaining {
                (1.0f64 - w.used).clamp(0.0, 1.0)
            } else {
                w.used.clamp(0.0, 1.0)
            };
            format!("{prefix}{}%", traymenu::pct(frac))
        }
        _ if snap.windows.is_empty() => "…".into(),
        _ => "—".into(),
    }
}

#[derive(Clone, Debug)]
pub struct NotchLayout {
    pub pill_rect: Rectangle,
    pub rest_rect: Rectangle,
    pub wake_rect: Rectangle,
    pub probe_rect: Rectangle,
    pub orb_center: Point,
    pub grip_center: Point,
    pub cell_centers: Vec<Point>,
    pub ring_centers: Vec<Point>,
    pub card_rect: Option<Rectangle>,
    pub tail_rect: Option<Rectangle>,
    pub card_edge_pt: Option<Point>,
    pub pill_edge_pt: Option<Point>,
}

pub fn cell_card_height(cell: &NotchCell) -> f32 {
    let mut h: f32 = 32.0; // padding top (16) + bottom (16)
    h += 24.0; // title row
    h += 6.0; // spacing

    if !cell.account_line.is_empty() {
        h += 17.0;
    }

    let snap = &cell.snap;
    if snap.status == "needsAuth" {
        h += 62.0;
    } else if snap.windows.is_empty() {
        h += 24.0;
    } else {
        for w in &snap.windows {
            if w.count.is_some() || w.amount.is_some() {
                h += 34.0;
            } else {
                h += 46.0;
            }
        }
        if !snap.note.is_empty() {
            h += 18.0;
        }
    }
    h.clamp(110.0, 340.0)
}

pub fn estimated_card_height(
    cell: &NotchCell,
    store_snap: &state::Snapshot,
    acts: &[activity::Activity],
) -> f32 {
    let mut h = cell_card_height(cell);
    if cell.base == "claude" {
        let active_count = store_snap.sessions.iter().filter(|s| s.state != state::ST_IDLE).count().min(5);
        if active_count > 0 {
            h += 6.0 + active_count as f32 * 17.0;
        }
    } else {
        let matching_count = acts.iter().filter(|a| a.provider == cell.base).count().min(5);
        if matching_count > 0 {
            h += 6.0 + matching_count as f32 * 17.0;
        }
    }
    h.clamp(110.0, 340.0)
}

pub fn compute_layout(ui: &NotchUiState, cells: &[NotchCell]) -> NotchLayout {
    let vert = ui.is_vertical();
    let win_w = if vert { NOTCH_W as f32 } else { NOTCH_LONG as f32 };
    let win_h = NOTCH_LONG as f32;
    let n = cells.len().max(1);

    let cell_w = 44.0f32;
    let cell_h = 66.0f32; // 44 ring + 6 gap + 16 pct
    let gap = 14.0f32;
    let pad_along = 18.0f32;
    let pad_cross = 12.0f32;
    let fillet = 38.7f32;

    let pill_rect = match ui.edge.as_str() {
        "left" => {
            let h = pad_along * 2.0 + n as f32 * cell_h + (n - 1) as f32 * gap;
            Rectangle::new(Point::new(0.0, (win_h - h) / 2.0), Size::new(70.0, h))
        }
        "top" => {
            let w = pad_along * 2.0 + n as f32 * cell_w + (n - 1) as f32 * gap;
            let h = pad_cross * 2.0 + cell_h;
            Rectangle::new(Point::new((win_w - w) / 2.0, 0.0), Size::new(w, h))
        }
        "bottom" => {
            let w = pad_along * 2.0 + n as f32 * cell_w + (n - 1) as f32 * gap;
            let h = pad_cross * 2.0 + cell_h;
            Rectangle::new(Point::new((win_w - w) / 2.0, win_h - h), Size::new(w, h))
        }
        _ => {
            let h = pad_along * 2.0 + n as f32 * cell_h + (n - 1) as f32 * gap;
            Rectangle::new(Point::new(win_w - 70.0, (win_h - h) / 2.0), Size::new(70.0, h))
        }
    };

    let (rest_rect, wake_rect, probe_rect) = match ui.edge.as_str() {
        "left" => {
            let r = Rectangle::new(Point::new(0.0, (win_h - 79.0) / 2.0), Size::new(10.0, 79.0));
            let w = Rectangle::new(Point::new(0.0, r.y), Size::new(10.0 + 34.0, 79.0));
            let p = Rectangle::new(Point::new(r.x + r.width + 4.0, r.y), Size::new(12.0, 79.0));
            (r, w, p)
        }
        "top" => {
            let r = Rectangle::new(Point::new((win_w - 79.0) / 2.0, 0.0), Size::new(79.0, 10.0));
            let w = Rectangle::new(Point::new(r.x, 0.0), Size::new(79.0, 10.0 + 34.0));
            let p = Rectangle::new(Point::new(r.x, r.y + r.height + 4.0), Size::new(79.0, 12.0));
            (r, w, p)
        }
        "bottom" => {
            let r = Rectangle::new(Point::new((win_w - 79.0) / 2.0, win_h - 10.0), Size::new(79.0, 10.0));
            let w = Rectangle::new(Point::new(r.x, r.y - 34.0), Size::new(79.0, 10.0 + 34.0));
            let p = Rectangle::new(Point::new(r.x, r.y - 16.0), Size::new(79.0, 12.0));
            (r, w, p)
        }
        _ => {
            let r = Rectangle::new(Point::new(win_w - 10.0, (win_h - 79.0) / 2.0), Size::new(10.0, 79.0));
            let w = Rectangle::new(Point::new(r.x - 34.0, r.y), Size::new(10.0 + 34.0, 79.0));
            let p = Rectangle::new(Point::new(r.x - 16.0, r.y), Size::new(12.0, 79.0));
            (r, w, p)
        }
    };

    let mut cell_centers = Vec::with_capacity(cells.len());
    let mut ring_centers = Vec::with_capacity(cells.len());
    for i in 0..cells.len() {
        if vert {
            let cx = pill_rect.x + pill_rect.width / 2.0;
            let cy = pill_rect.y + pad_along + 22.0 + i as f32 * (cell_h + gap);
            ring_centers.push(Point::new(cx, cy));
            cell_centers.push(Point::new(cx, cy + 11.0));
        } else {
            let cx = pill_rect.x + pad_along + 22.0 + i as f32 * (cell_w + gap);
            let cy = pill_rect.y + pad_cross + 22.0;
            ring_centers.push(Point::new(cx, cy));
            cell_centers.push(Point::new(cx, cy + 11.0));
        }
    }

    let orb_center = match ui.edge.as_str() {
        "left" => Point::new(pill_rect.x + fillet, pill_rect.y + pill_rect.height + fillet),
        "top" => Point::new(pill_rect.x + pill_rect.width + fillet, pill_rect.y + fillet),
        "bottom" => Point::new(pill_rect.x + pill_rect.width + fillet, pill_rect.y + pill_rect.height - fillet),
        _ => Point::new(pill_rect.x + pill_rect.width - fillet, pill_rect.y + pill_rect.height + fillet),
    };
    const GRIP_REACH: f32 = 23.3 + 11.9 + (9.9 + 6.1) / 2.0;
    let grip_center = if vert {
        Point::new(orb_center.x, orb_center.y + GRIP_REACH)
    } else {
        Point::new(orb_center.x + GRIP_REACH, orb_center.y)
    };

    let mut card_rect = None;
    let mut tail_rect = None;
    let mut card_edge_pt = None;
    let mut pill_edge_pt = None;

    if ui.anim.card_t > 0.001 && !ui.folded && !cells.is_empty() {
        let idx = ui
            .hover_id
            .as_deref()
            .and_then(|id| cells.iter().position(|c| c.id == id))
            .unwrap_or(0);
        let rc = ring_centers[idx];
        let cw = 246.0f32;
        let ch = if ui.anim.card_h_t > 50.0 { ui.anim.card_h_t } else { cell_card_height(&cells[idx]) };
        let tail_len = 22.0f32;

        if vert {
            let rc_y = if ui.anim.card_pos_t > 0.1 { ui.anim.card_pos_t } else { rc.y };
            let tail_y = if ui.anim.tail_pos_t > 0.1 { ui.anim.tail_pos_t } else { rc.y };

            let top = (rc_y - ch / 2.0).clamp(8.0 + ui.insets[0], win_h - ui.insets[2] - ch - 8.0);
            let (x, card_pt_x, pill_pt_x) = if ui.edge == "left" {
                let x = pill_rect.x + pill_rect.width + tail_len;
                (x, x, pill_rect.x + pill_rect.width)
            } else {
                let x = pill_rect.x - tail_len - cw;
                (x, x + cw, pill_rect.x)
            };

            let ty = rc_y.clamp(top + 22.0, (top + ch - 22.0).max(top + 22.0));
            card_edge_pt = Some(Point::new(card_pt_x, ty));
            pill_edge_pt = Some(Point::new(pill_pt_x, tail_y));

            card_rect = Some(Rectangle::new(Point::new(x, top), Size::new(cw, ch)));
            let (tx, tw) = if ui.edge == "left" {
                (pill_rect.x + pill_rect.width, tail_len)
            } else {
                (x + cw, tail_len)
            };
            let min_y = ty.min(tail_y) - 24.0;
            let max_y = ty.max(tail_y) + 24.0;
            tail_rect = Some(Rectangle::new(Point::new(tx, min_y), Size::new(tw, max_y - min_y)));
        } else {
            let rc_x = if ui.anim.card_pos_t > 0.1 { ui.anim.card_pos_t } else { rc.x };
            let tail_x = if ui.anim.tail_pos_t > 0.1 { ui.anim.tail_pos_t } else { rc.x };

            let left = (rc_x - cw / 2.0).clamp(8.0 + ui.insets[3], win_w - ui.insets[1] - cw - 8.0);
            let (y, card_pt_y, pill_pt_y) = if ui.edge == "top" {
                let y = pill_rect.y + pill_rect.height + tail_len;
                (y, y, pill_rect.y + pill_rect.height)
            } else {
                let y = pill_rect.y - tail_len - ch;
                (y, y + ch, pill_rect.y)
            };

            let tx = rc_x.clamp(left + 22.0, (left + cw - 22.0).max(left + 22.0));
            card_edge_pt = Some(Point::new(tx, card_pt_y));
            pill_edge_pt = Some(Point::new(tail_x, pill_pt_y));

            card_rect = Some(Rectangle::new(Point::new(left, y), Size::new(cw, ch)));
            let (ty, th) = if ui.edge == "top" {
                (pill_rect.y + pill_rect.height, tail_len)
            } else {
                (y + ch, tail_len)
            };
            let min_x = tx.min(tail_x) - 24.0;
            let max_x = tx.max(tail_x) + 24.0;
            tail_rect = Some(Rectangle::new(Point::new(min_x, ty), Size::new(max_x - min_x, th)));
        }
    }

    if let (Some(cr), Some(tr)) = (card_rect, tail_rect) {
        let t = ui.anim.card_t.clamp(0.0, 1.0);
        if t < 0.999 {
            let (dx, dy) = match ui.edge.as_str() {
                "left" => (-26.0, 0.0), "right" => (26.0, 0.0),
                "top" => (0.0, -26.0), _ => (0.0, 26.0),
            };
            let from_card = Rectangle::new(Point::new(cr.x + dx, cr.y + dy), Size::new(cr.width * 0.94, cr.height * 0.94));
            let from_tail = Rectangle::new(tr.position(), Size::new(tr.width * 0.6, tr.height * 0.6));
            card_rect = Some(lerp_rect(from_card, cr, t));
            tail_rect = Some(lerp_rect(from_tail, tr, t));
        }
    }
    NotchLayout {
        pill_rect,
        rest_rect,
        wake_rect,
        probe_rect,
        orb_center,
        grip_center,
        cell_centers,
        ring_centers,
        card_rect,
        tail_rect,
        card_edge_pt,
        pill_edge_pt,
    }
}

fn lerp_rect(a: Rectangle, b: Rectangle, t: f32) -> Rectangle {
    Rectangle::new(
        Point::new(motion::lerp(a.x, b.x, t), motion::lerp(a.y, b.y, t)),
        Size::new(motion::lerp(a.width, b.width, t), motion::lerp(a.height, b.height, t)),
    )
}

pub fn hot_rects_physical(ui: &NotchUiState, cells: &[NotchCell], monitor_scale: f64) -> Vec<[f64; 4]> {
    let layout = compute_layout(ui, cells);
    let k = monitor_scale * ui.scale as f64;
    let to_phys = |r: Rectangle| [r.x as f64 * k, r.y as f64 * k, r.width as f64 * k, r.height as f64 * k];
    if ui.folded {
        return vec![to_phys(layout.wake_rect)];
    }
    let mut rects = vec![to_phys(layout.pill_rect)];
    let orb = Rectangle::new(
        Point::new(layout.orb_center.x - 28.5, layout.orb_center.y - 28.5),
        Size::new(57.0, 57.0),
    );
    rects.push(to_phys(orb));
    if ui.grip_out > 0.05 {
        let grip = Rectangle::new(
            Point::new(layout.grip_center.x - 26.3, layout.grip_center.y - 26.3),
            Size::new(52.6, 52.6),
        );
        rects.push(to_phys(grip));
    }
    if let Some(c) = layout.card_rect {
        rects.push(to_phys(c));
    }
    if let Some(t) = layout.tail_rect {
        rects.push(to_phys(t));
    }
    rects
}

pub fn probe_rect_physical(ui: &NotchUiState, cells: &[NotchCell], monitor_scale: f64) -> [f64; 4] {
    let layout = compute_layout(ui, cells);
    let k = monitor_scale * ui.scale as f64;
    let r = layout.probe_rect;
    [r.x as f64 * k, r.y as f64 * k, r.width as f64 * k, r.height as f64 * k]
}

// ---------------- Canvas Program ----------------

struct NotchCanvasProgram {
    ui: NotchUiState,
    cells: Vec<NotchCell>,
    layout: NotchLayout,
    cfg: config::Config,
    store_snap: state::Snapshot,
    acts: Vec<activity::Activity>,
    now: u64,
}

impl canvas::Program<NotchMsg> for NotchCanvasProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let s = self.ui.scale;
        frame.scale(s);

        let pal = *theme::palette(true);

        let fold = self.ui.anim.fold_t.clamp(0.0, 1.0);

        if fold >= 0.999 {
            let r = self.layout.rest_rect;
            let rad = match self.ui.edge.as_str() {
                "left" => iced::border::Radius { top_left: 0.0, top_right: 6.0, bottom_right: 6.0, bottom_left: 0.0 },
                "top" => iced::border::Radius { top_left: 0.0, top_right: 0.0, bottom_right: 6.0, bottom_left: 6.0 },
                "bottom" => iced::border::Radius { top_left: 6.0, top_right: 6.0, bottom_right: 0.0, bottom_left: 0.0 },
                _ => iced::border::Radius { top_left: 6.0, top_right: 0.0, bottom_right: 0.0, bottom_left: 6.0 },
            };
            let mut b = path::Builder::new();
            b.rounded_rectangle(r.position(), r.size(), rad);
            let p = b.build();
            frame.fill(&p, pal.pill);
            frame.stroke(&p, Stroke::default().with_color(pal.rest_edge).with_width(1.0));
            return vec![frame.into_geometry()];
        }

        // Clean seamless silhouette with pure cubic bezier contours (no diagonal partition)
        let morph_rect = lerp_rect(self.layout.pill_rect, self.layout.rest_rect, fold);
        let (fill_path, stroke_path) = build_pill_contours(
            &self.ui.edge,
            morph_rect,
            motion::lerp(20.0, 6.0, fold),
            motion::lerp(38.7, 6.0, fold),
        );
        frame.fill(&fill_path, pal.pill);
        frame.stroke(&stroke_path, Stroke::default().with_color(pal.edge).with_width(1.0));

        let reveal = 1.0 - fold;
        {

            let k = 44.0 / 56.0;
            let ramp = self.cfg.color_transition == "ramp";
            let rest_center = Point::new(
                self.layout.rest_rect.x + self.layout.rest_rect.width * 0.5,
                self.layout.rest_rect.y + self.layout.rest_rect.height * 0.5,
            );
            for (i, cell) in self.cells.iter().enumerate() {
                let stagger = (fold * 1.6 - i as f32 * 0.12).clamp(0.0, 1.0);
                let cell_reveal = (reveal - stagger * reveal).max(0.0);
                if cell_reveal <= 0.001 { continue; }
                let target = self.layout.ring_centers[i];
                let rc = Point::new(
                    motion::lerp(target.x, rest_center.x, fold),
                    motion::lerp(target.y, rest_center.y, fold),
                );
                let press = self.ui.refreshing_ids.get(&cell.id)
                    .map(|started| motion::cell_press(*started, Instant::now(), true))
                    .unwrap_or(0.0)
                    .max(self.ui.anim.press_t * 0.35);
                let ring_scale = (1.0 - 0.07 * press) * k;
                let stale = stale_of(&cell.snap, self.now);
                let fade = (if stale { 0.55 } else { 1.0 }) * cell_reveal;

                let head = core::ring_window(
                    &cell.base,
                    &cell.snap.windows,
                    &self.cfg.antigravity_limit,
                    &self.cfg.antigravity_model,
                )
                .filter(|w| cell.base != "claude" || claude_quota_current(&cell.snap, w, self.now));

                let ws = work_state_of(&cell.base, &self.store_snap, &self.acts);

                let hole_col = Color { a: pal.hole.a * fade, ..pal.hole };
                let track_col = Color { a: pal.track.a * fade, ..pal.track };
                frame.fill(&Path::circle(rc, 22.0 * ring_scale), hole_col);
                frame.stroke(
                    &Path::circle(rc, 25.0 * ring_scale),
                    Stroke::default().with_color(track_col).with_width(5.0 * ring_scale),
                );

                if self.cfg.weekly_ring != "off" {
                    let taken = self.cfg.weekly_ring == "inside" && ws != "idle";
                    let wk = weekly_window(&cell.base, &cell.snap.windows, &self.cfg.antigravity_model)
                        .filter(|w| cell.base != "claude" || claude_quota_current(&cell.snap, w, self.now));
                    if let Some(wk) = wk {
                        if head.is_none_or(|h| h.id != wk.id) && !taken && wk.metered() {
                            let r = (if self.cfg.weekly_ring == "inside" { 16.0 } else { 31.0 }) * ring_scale;
                            let frac = if self.cfg.notch_shows_remaining {
                                (1.0f64 - wk.used).clamp(0.0, 1.0)
                            } else { wk.used.clamp(0.0, 1.0) } as f32;
                            let wk_col = theme::tone(wk.used, self.cfg.watch_limit, self.cfg.critical_limit, ramp, &pal);
                            let wk_col = Color { a: 0.85 * fade, ..wk_col };
                            let tr_col = Color { a: pal.track.a * 0.7 * fade, ..pal.track };
                            if self.cfg.weekly_ring_dashed {
                                draw_dashed_arc(&mut frame, rc, r, 1.0, 2.4 * ring_scale, ring_scale, tr_col);
                                draw_dashed_arc(&mut frame, rc, r, frac, 2.4 * ring_scale, ring_scale, wk_col);
                            } else {
                                frame.stroke(&Path::circle(rc, r), Stroke::default().with_color(tr_col).with_width(2.4 * ring_scale));
                                draw_arc(&mut frame, rc, r, 0.0, frac, 2.4 * ring_scale, wk_col, LineCap::Round);
                            }
                        }
                    }
                }

                if let Some(h) = head {
                    if h.metered() && h.used.is_finite() {
                        let frac = if self.cfg.notch_shows_remaining {
                            (1.0f64 - h.used).clamp(0.0, 1.0)
                        } else { h.used.clamp(0.0, 1.0) } as f32;
                        let col = theme::tone(h.used, self.cfg.watch_limit, self.cfg.critical_limit, ramp, &pal);
                        let col = Color { a: fade, ..col };
                        draw_arc(
                            &mut frame,
                            rc,
                            25.0 * ring_scale,
                            self.ui.anim.turn_t,
                            frac,
                            5.0 * ring_scale,
                            col,
                            LineCap::Round,
                        );
                    }
                }

                match ws {
                    "running" => {
                        let step = ((self.ui.anim_secs / 1.2).rem_euclid(1.0) * 12.0).floor() / 12.0;
                        draw_arc(&mut frame, rc, 19.0 * ring_scale, step, 0.28, 2.5 * ring_scale,
                            Color { a: fade, ..pal.ink }, LineCap::Round);
                    }
                    "attention" => {
                        let phase = (self.ui.anim_secs / 1.1).rem_euclid(1.0);
                        let half = (phase * 2.0).fract();
                        let st = (half * 6.0).floor() / 6.0;
                        let alpha = if phase < 0.5 { 1.0 - 0.75 * st } else { 0.25 + 0.75 * st };
                        let col = Color { a: alpha * fade, ..pal.watch };
                        frame.stroke(&Path::circle(rc, 19.0 * ring_scale),
                            Stroke::default().with_color(col).with_width(2.5 * ring_scale));
                    }
                    _ => {}
                }

                let pct_str = pct_figure(&cell.snap, head, self.cfg.notch_shows_remaining);
                frame.fill_text(canvas::Text {
                    content: pct_str,
                    position: Point::new(rc.x, rc.y + 34.0),
                    color: Color { a: cell_reveal, ..pal.ink },
                    size: iced::Pixels(13.5),
                    font: Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT },
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center.into(),
                    ..canvas::Text::default()
                });
            }

            draw_orb_and_grip(&mut frame, &self.ui, &self.layout, &pal, reveal);

            if let (Some(cpt), Some(ppt)) = (self.layout.card_edge_pt, self.layout.pill_edge_pt) {
                let tail_path = build_gooey_tail(&self.ui.edge, cpt, ppt, 42.0, 16.0, 7.0);
                frame.fill(&tail_path, Color { a: self.ui.anim.card_t, ..pal.card });
            }
        }
        vec![frame.into_geometry()]
    }
}

const KAPPA: f32 = 0.55228475;

fn build_pill_contours(edge: &str, r: Rectangle, rad: f32, f: f32) -> (Path, Path) {
    let mut fill_b = path::Builder::new();
    let mut stroke_b = path::Builder::new();

    match edge {
        "left" => {
            let (x0, x1, y0, y1) = (r.x, r.x + r.width, r.y, r.y + r.height);
            let start = Point::new(x0, y0 - f);
            fill_b.move_to(start);
            stroke_b.move_to(start);

            // Top fillet: concave curve into the pill
            let cp1 = Point::new(x0, y0 - f + f * KAPPA);
            let cp2 = Point::new(x0 + f - f * KAPPA, y0);
            let p1 = Point::new(x0 + f, y0);
            fill_b.bezier_curve_to(cp1, cp2, p1);
            stroke_b.bezier_curve_to(cp1, cp2, p1);

            // Top edge
            let p2 = Point::new(x1 - rad, y0);
            fill_b.line_to(p2);
            stroke_b.line_to(p2);

            // Top-right corner
            let cp3 = Point::new(x1 - rad + rad * KAPPA, y0);
            let cp4 = Point::new(x1, y0 + rad - rad * KAPPA);
            let p3 = Point::new(x1, y0 + rad);
            fill_b.bezier_curve_to(cp3, cp4, p3);
            stroke_b.bezier_curve_to(cp3, cp4, p3);

            // Right vertical outer edge
            let p4 = Point::new(x1, y1 - rad);
            fill_b.line_to(p4);
            stroke_b.line_to(p4);

            // Bottom-right corner
            let cp5 = Point::new(x1, y1 - rad + rad * KAPPA);
            let cp6 = Point::new(x1 - rad + rad * KAPPA, y1);
            let p5 = Point::new(x1 - rad, y1);
            fill_b.bezier_curve_to(cp5, cp6, p5);
            stroke_b.bezier_curve_to(cp5, cp6, p5);

            // Bottom edge
            let p6 = Point::new(x0 + f, y1);
            fill_b.line_to(p6);
            stroke_b.line_to(p6);

            // Bottom fillet: concave curve to the screen edge
            let cp7 = Point::new(x0 + f - f * KAPPA, y1);
            let cp8 = Point::new(x0, y1 + f - f * KAPPA);
            let end_stroke = Point::new(x0, y1 + f);
            fill_b.bezier_curve_to(cp7, cp8, end_stroke);
            stroke_b.bezier_curve_to(cp7, cp8, end_stroke);

            fill_b.line_to(start);
            fill_b.close();
        }
        "top" => {
            let (x0, x1, y0, y1) = (r.x, r.x + r.width, r.y, r.y + r.height);
            let start = Point::new(x0 - f, y0);
            fill_b.move_to(start);
            stroke_b.move_to(start);

            // Left fillet: concave curve into the pill
            let cp1 = Point::new(x0 - f + f * KAPPA, y0);
            let cp2 = Point::new(x0, y0 + f - f * KAPPA);
            let p1 = Point::new(x0, y0 + f);
            fill_b.bezier_curve_to(cp1, cp2, p1);
            stroke_b.bezier_curve_to(cp1, cp2, p1);

            // Left edge
            let p2 = Point::new(x0, y1 - rad);
            fill_b.line_to(p2);
            stroke_b.line_to(p2);

            // Bottom-left corner
            let cp3 = Point::new(x0, y1 - rad + rad * KAPPA);
            let cp4 = Point::new(x0 + rad - rad * KAPPA, y1);
            let p3 = Point::new(x0 + rad, y1);
            fill_b.bezier_curve_to(cp3, cp4, p3);
            stroke_b.bezier_curve_to(cp3, cp4, p3);

            // Bottom outer horizontal edge
            let p4 = Point::new(x1 - rad, y1);
            fill_b.line_to(p4);
            stroke_b.line_to(p4);

            // Bottom-right corner
            let cp5 = Point::new(x1 - rad + rad * KAPPA, y1);
            let cp6 = Point::new(x1, y1 - rad + rad * KAPPA);
            let p5 = Point::new(x1, y1 - rad);
            fill_b.bezier_curve_to(cp5, cp6, p5);
            stroke_b.bezier_curve_to(cp5, cp6, p5);

            // Right edge
            let p6 = Point::new(x1, y0 + f);
            fill_b.line_to(p6);
            stroke_b.line_to(p6);

            // Right fillet: concave curve to the screen edge
            let cp7 = Point::new(x1, y0 + f - f * KAPPA);
            let cp8 = Point::new(x1 + f - f * KAPPA, y0);
            let end_stroke = Point::new(x1 + f, y0);
            fill_b.bezier_curve_to(cp7, cp8, end_stroke);
            stroke_b.bezier_curve_to(cp7, cp8, end_stroke);

            fill_b.line_to(start);
            fill_b.close();
        }
        "bottom" => {
            let (x0, x1, y0, y1) = (r.x, r.x + r.width, r.y, r.y + r.height);
            let start = Point::new(x0 - f, y1);
            fill_b.move_to(start);
            stroke_b.move_to(start);

            // Left fillet: concave curve into the pill
            let cp1 = Point::new(x0 - f + f * KAPPA, y1);
            let cp2 = Point::new(x0, y1 - f + f * KAPPA);
            let p1 = Point::new(x0, y1 - f);
            fill_b.bezier_curve_to(cp1, cp2, p1);
            stroke_b.bezier_curve_to(cp1, cp2, p1);

            // Left edge
            let p2 = Point::new(x0, y0 + rad);
            fill_b.line_to(p2);
            stroke_b.line_to(p2);

            // Top-left corner
            let cp3 = Point::new(x0, y0 + rad - rad * KAPPA);
            let cp4 = Point::new(x0 + rad - rad * KAPPA, y0);
            let p3 = Point::new(x0 + rad, y0);
            fill_b.bezier_curve_to(cp3, cp4, p3);
            stroke_b.bezier_curve_to(cp3, cp4, p3);

            // Top outer horizontal edge
            let p4 = Point::new(x1 - rad, y0);
            fill_b.line_to(p4);
            stroke_b.line_to(p4);

            // Top-right corner
            let cp5 = Point::new(x1 - rad + rad * KAPPA, y0);
            let cp6 = Point::new(x1, y0 + rad - rad * KAPPA);
            let p5 = Point::new(x1, y0 + rad);
            fill_b.bezier_curve_to(cp5, cp6, p5);
            stroke_b.bezier_curve_to(cp5, cp6, p5);

            // Right edge
            let p6 = Point::new(x1, y1 - f);
            fill_b.line_to(p6);
            stroke_b.line_to(p6);

            // Right fillet: concave curve to the screen edge
            let cp7 = Point::new(x1, y1 - f + f * KAPPA);
            let cp8 = Point::new(x1 + f - f * KAPPA, y1);
            let end_stroke = Point::new(x1 + f, y1);
            fill_b.bezier_curve_to(cp7, cp8, end_stroke);
            stroke_b.bezier_curve_to(cp7, cp8, end_stroke);

            fill_b.line_to(start);
            fill_b.close();
        }
        _ => {
            // "right"
            let (x0, x1, y0, y1) = (r.x, r.x + r.width, r.y, r.y + r.height);
            let start = Point::new(x1, y0 - f);
            fill_b.move_to(start);
            stroke_b.move_to(start);

            // Top fillet: concave curve into the pill
            let cp1 = Point::new(x1, y0 - f + f * KAPPA);
            let cp2 = Point::new(x1 - f + f * KAPPA, y0);
            let p1 = Point::new(x1 - f, y0);
            fill_b.bezier_curve_to(cp1, cp2, p1);
            stroke_b.bezier_curve_to(cp1, cp2, p1);

            // Top edge
            let p2 = Point::new(x0 + rad, y0);
            fill_b.line_to(p2);
            stroke_b.line_to(p2);

            // Top-left corner
            let cp3 = Point::new(x0 + rad - rad * KAPPA, y0);
            let cp4 = Point::new(x0, y0 + rad - rad * KAPPA);
            let p3 = Point::new(x0, y0 + rad);
            fill_b.bezier_curve_to(cp3, cp4, p3);
            stroke_b.bezier_curve_to(cp3, cp4, p3);

            // Left vertical outer edge
            let p4 = Point::new(x0, y1 - rad);
            fill_b.line_to(p4);
            stroke_b.line_to(p4);

            // Bottom-left corner
            let cp5 = Point::new(x0, y1 - rad + rad * KAPPA);
            let cp6 = Point::new(x0 + rad - rad * KAPPA, y1);
            let p5 = Point::new(x0 + rad, y1);
            fill_b.bezier_curve_to(cp5, cp6, p5);
            stroke_b.bezier_curve_to(cp5, cp6, p5);

            // Bottom edge
            let p6 = Point::new(x1 - f, y1);
            fill_b.line_to(p6);
            stroke_b.line_to(p6);

            // Bottom fillet: concave curve to the screen edge
            let cp7 = Point::new(x1 - f + f * KAPPA, y1);
            let cp8 = Point::new(x1, y1 + f - f * KAPPA);
            let end_stroke = Point::new(x1, y1 + f);
            fill_b.bezier_curve_to(cp7, cp8, end_stroke);
            stroke_b.bezier_curve_to(cp7, cp8, end_stroke);

            // Fill closes along the screen edge; stroke ends open without drawing on the bezel
            fill_b.line_to(start);
            fill_b.close();
        }
    }

    (fill_b.build(), stroke_b.build())
}

/// Builds the curved "trumpet" tail wedge that bridges a detached card to the
/// notch pill. Mirrors the native Direct2D `build_tail_path` so the Iced card
/// keeps the same silhouette on every edge.
pub fn build_tail_path(edge: &str, r: Rectangle) -> Path {
    let mut b = path::Builder::new();
    let (x0, y0, w, h) = (r.x, r.y, r.width, r.height);

    match edge {
        "left" => {
            // Pill on the left, card on the right: flat base on the right, tip pointing left.
            b.move_to(Point::new(x0 + w, y0));
            b.bezier_curve_to(
                Point::new(x0 + w, y0 + h * 0.25),
                Point::new(x0 + w * 0.42, y0 + h * 0.38),
                Point::new(x0, y0 + h * 0.5),
            );
            b.bezier_curve_to(
                Point::new(x0 + w * 0.42, y0 + h * 0.62),
                Point::new(x0 + w, y0 + h * 0.75),
                Point::new(x0 + w, y0 + h),
            );
        }
        "top" => {
            // Pill at the top, card below: flat base at the bottom, tip pointing up.
            b.move_to(Point::new(x0, y0 + h));
            b.bezier_curve_to(
                Point::new(x0 + w * 0.25, y0 + h),
                Point::new(x0 + w * 0.38, y0 + h * 0.42),
                Point::new(x0 + w * 0.5, y0),
            );
            b.bezier_curve_to(
                Point::new(x0 + w * 0.62, y0 + h * 0.42),
                Point::new(x0 + w * 0.75, y0 + h),
                Point::new(x0 + w, y0 + h),
            );
        }
        "bottom" => {
            // Pill at the bottom, card above: flat base at the top, tip pointing down.
            b.move_to(Point::new(x0, y0));
            b.bezier_curve_to(
                Point::new(x0 + w * 0.25, y0),
                Point::new(x0 + w * 0.38, y0 + h * 0.58),
                Point::new(x0 + w * 0.5, y0 + h),
            );
            b.bezier_curve_to(
                Point::new(x0 + w * 0.62, y0 + h * 0.58),
                Point::new(x0 + w * 0.75, y0),
                Point::new(x0 + w, y0),
            );
        }
        _ => {
            // "right": card on the left, pill on the right: flat base on the left, tip pointing right.
            b.move_to(Point::new(x0, y0));
            b.bezier_curve_to(
                Point::new(x0, y0 + h * 0.25),
                Point::new(x0 + w * 0.58, y0 + h * 0.38),
                Point::new(x0 + w, y0 + h * 0.5),
            );
            b.bezier_curve_to(
                Point::new(x0 + w * 0.58, y0 + h * 0.62),
                Point::new(x0, y0 + h * 0.75),
                Point::new(x0, y0 + h),
            );
        }
    }
    b.close();
    b.build()
}

pub fn build_gooey_tail(
    edge: &str,
    card_pt: Point,
    pill_pt: Point,
    base_h: f32,
    waist_h: f32,
    tip_r: f32,
) -> Path {
    let mut b = path::Builder::new();
    match edge {
        "left" => {
            // Pill on left, card on right
            let dx = pill_pt.x - card_pt.x; // negative
            let dy = pill_pt.y - card_pt.y;

            let p_top_base = Point::new(card_pt.x, card_pt.y - base_h * 0.5);
            let p_bot_base = Point::new(card_pt.x, card_pt.y + base_h * 0.5);

            let mx = card_pt.x + dx * 0.44;
            let my = card_pt.y + dy * 0.65;
            let m_top = Point::new(mx, my - waist_h * 0.5);
            let m_bot = Point::new(mx, my + waist_h * 0.5);

            let tip_top = Point::new(pill_pt.x + tip_r * 0.85, pill_pt.y - tip_r * 0.9);
            let tip_bot = Point::new(pill_pt.x + tip_r * 0.85, pill_pt.y + tip_r * 0.9);

            b.move_to(p_top_base);
            b.bezier_curve_to(
                Point::new(card_pt.x + dx * 0.18, card_pt.y - base_h * 0.45),
                Point::new(mx - dx * 0.16, my - waist_h * 0.52),
                m_top,
            );
            b.bezier_curve_to(
                Point::new(mx + dx * 0.22, my - waist_h * 0.45),
                Point::new(pill_pt.x + tip_r * 1.2, pill_pt.y - tip_r * 0.95),
                tip_top,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x - tip_r * 0.35, pill_pt.y - tip_r * 0.45),
                Point::new(pill_pt.x - tip_r * 0.35, pill_pt.y + tip_r * 0.45),
                tip_bot,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x + tip_r * 1.2, pill_pt.y + tip_r * 0.95),
                Point::new(mx + dx * 0.22, my + waist_h * 0.45),
                m_bot,
            );
            b.bezier_curve_to(
                Point::new(mx - dx * 0.16, my + waist_h * 0.52),
                Point::new(card_pt.x + dx * 0.18, card_pt.y + base_h * 0.45),
                p_bot_base,
            );
            b.line_to(p_top_base);
        }
        "top" => {
            // Pill at top, card below
            let dx = pill_pt.x - card_pt.x;
            let dy = pill_pt.y - card_pt.y; // negative

            let p_left_base = Point::new(card_pt.x - base_h * 0.5, card_pt.y);
            let p_right_base = Point::new(card_pt.x + base_h * 0.5, card_pt.y);

            let mx = card_pt.x + dx * 0.65;
            let my = card_pt.y + dy * 0.44;
            let m_left = Point::new(mx - waist_h * 0.5, my);
            let m_right = Point::new(mx + waist_h * 0.5, my);

            let tip_left = Point::new(pill_pt.x - tip_r * 0.9, pill_pt.y + tip_r * 0.85);
            let tip_right = Point::new(pill_pt.x + tip_r * 0.9, pill_pt.y + tip_r * 0.85);

            b.move_to(p_left_base);
            b.bezier_curve_to(
                Point::new(card_pt.x - base_h * 0.45, card_pt.y + dy * 0.18),
                Point::new(mx - waist_h * 0.52, my - dy * 0.16),
                m_left,
            );
            b.bezier_curve_to(
                Point::new(mx - waist_h * 0.45, my + dy * 0.22),
                Point::new(pill_pt.x - tip_r * 0.95, pill_pt.y + tip_r * 1.2),
                tip_left,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x - tip_r * 0.45, pill_pt.y - tip_r * 0.35),
                Point::new(pill_pt.x + tip_r * 0.45, pill_pt.y - tip_r * 0.35),
                tip_right,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x + tip_r * 0.95, pill_pt.y + tip_r * 1.2),
                Point::new(mx + waist_h * 0.45, my + dy * 0.22),
                m_right,
            );
            b.bezier_curve_to(
                Point::new(mx + waist_h * 0.52, my - dy * 0.16),
                Point::new(card_pt.x + base_h * 0.45, card_pt.y + dy * 0.18),
                p_right_base,
            );
            b.line_to(p_left_base);
        }
        "bottom" => {
            // Pill at bottom, card above
            let dx = pill_pt.x - card_pt.x;
            let dy = pill_pt.y - card_pt.y; // positive

            let p_left_base = Point::new(card_pt.x - base_h * 0.5, card_pt.y);
            let p_right_base = Point::new(card_pt.x + base_h * 0.5, card_pt.y);

            let mx = card_pt.x + dx * 0.65;
            let my = card_pt.y + dy * 0.44;
            let m_left = Point::new(mx - waist_h * 0.5, my);
            let m_right = Point::new(mx + waist_h * 0.5, my);

            let tip_left = Point::new(pill_pt.x - tip_r * 0.9, pill_pt.y - tip_r * 0.85);
            let tip_right = Point::new(pill_pt.x + tip_r * 0.9, pill_pt.y - tip_r * 0.85);

            b.move_to(p_left_base);
            b.bezier_curve_to(
                Point::new(card_pt.x - base_h * 0.45, card_pt.y + dy * 0.18),
                Point::new(mx - waist_h * 0.52, my - dy * 0.16),
                m_left,
            );
            b.bezier_curve_to(
                Point::new(mx - waist_h * 0.45, my + dy * 0.22),
                Point::new(pill_pt.x - tip_r * 0.95, pill_pt.y - tip_r * 1.2),
                tip_left,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x - tip_r * 0.45, pill_pt.y + tip_r * 0.35),
                Point::new(pill_pt.x + tip_r * 0.45, pill_pt.y + tip_r * 0.35),
                tip_right,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x + tip_r * 0.95, pill_pt.y - tip_r * 1.2),
                Point::new(mx + waist_h * 0.45, my + dy * 0.22),
                m_right,
            );
            b.bezier_curve_to(
                Point::new(mx + waist_h * 0.52, my - dy * 0.16),
                Point::new(card_pt.x + base_h * 0.45, card_pt.y + dy * 0.18),
                p_right_base,
            );
            b.line_to(p_left_base);
        }
        _ => {
            // Pill on right, card on left
            let dx = pill_pt.x - card_pt.x; // positive
            let dy = pill_pt.y - card_pt.y;

            let p_top_base = Point::new(card_pt.x, card_pt.y - base_h * 0.5);
            let p_bot_base = Point::new(card_pt.x, card_pt.y + base_h * 0.5);

            let mx = card_pt.x + dx * 0.44;
            let my = card_pt.y + dy * 0.65;
            let m_top = Point::new(mx, my - waist_h * 0.5);
            let m_bot = Point::new(mx, my + waist_h * 0.5);

            let tip_top = Point::new(pill_pt.x - tip_r * 0.85, pill_pt.y - tip_r * 0.9);
            let tip_bot = Point::new(pill_pt.x - tip_r * 0.85, pill_pt.y + tip_r * 0.9);

            b.move_to(p_top_base);
            b.bezier_curve_to(
                Point::new(card_pt.x + dx * 0.18, card_pt.y - base_h * 0.45),
                Point::new(mx - dx * 0.16, my - waist_h * 0.52),
                m_top,
            );
            b.bezier_curve_to(
                Point::new(mx + dx * 0.22, my - waist_h * 0.45),
                Point::new(pill_pt.x - tip_r * 1.2, pill_pt.y - tip_r * 0.95),
                tip_top,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x + tip_r * 0.35, pill_pt.y - tip_r * 0.45),
                Point::new(pill_pt.x + tip_r * 0.35, pill_pt.y + tip_r * 0.45),
                tip_bot,
            );
            b.bezier_curve_to(
                Point::new(pill_pt.x - tip_r * 1.2, pill_pt.y + tip_r * 0.95),
                Point::new(mx + dx * 0.22, my + waist_h * 0.45),
                m_bot,
            );
            b.bezier_curve_to(
                Point::new(mx - dx * 0.16, my + waist_h * 0.52),
                Point::new(card_pt.x + dx * 0.18, card_pt.y + base_h * 0.45),
                p_bot_base,
            );
            b.line_to(p_top_base);
        }
    }
    b.close();
    b.build()
}

fn draw_arc(
    frame: &mut Frame,
    center: Point,
    radius: f32,
    start_turns: f32,
    fraction: f32,
    width: f32,
    color: Color,
    cap: LineCap,
) {
    if !fraction.is_finite() || fraction <= 0.0 || radius <= 0.0 { return; }
    let stroke = Stroke {
        style: canvas::Style::Solid(color),
        width,
        line_cap: cap,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    if fraction >= 1.0 {
        frame.stroke(&Path::circle(center, radius), stroke);
        return;
    }
    let a0 = (start_turns - 0.25) * TAU;
    let a1 = a0 + fraction * TAU;
    let mut b = path::Builder::new();
    b.arc(path::Arc { center, radius, start_angle: Radians(a0), end_angle: Radians(a1) });
    frame.stroke(&b.build(), stroke);
}

fn draw_dashed_arc(
    frame: &mut Frame,
    center: Point,
    radius: f32,
    fraction: f32,
    width: f32,
    unit: f32,
    color: Color,
) {
    let circ = TAU * radius;
    let total_len = circ * fraction.clamp(0.0, 1.0);
    let mut pos = 0.0f32;
    while pos < total_len {
        let dash = (4.0 * unit).min(total_len - pos);
        draw_arc(frame, center, radius, pos / circ, dash / circ, width, color, LineCap::Butt);
        pos += 6.0 * unit;
    }
}

fn draw_orb_and_grip(
    frame: &mut Frame,
    ui: &NotchUiState,
    layout: &NotchLayout,
    pal: &theme::Palette,
    reveal: f32,
) {
    let c = layout.orb_center;
    let hover = ui.anim.orb_t.clamp(0.0, 1.0);
    let a = reveal;

    // By default, the arc rests pointing top-right (no continuous spin!).
    let start_turns = match ui.edge.as_str() {
        "left" => 0.75,
        "top" => 0.5,
        "bottom" => 0.25,
        _ => 0.0,
    };

    // The arc does NOT spin!
    // At rest (hover = 0.0), it spans 0.28 turns (quarter arc in top-right quadrant).
    // As hover increases, it morphs into the full closed circle (span -> 1.0)!
    let arc_radius = 21.0;
    let arc_span = motion::lerp(0.28, 1.0, hover);
    let arc_path = goo::quarter(c, arc_radius, start_turns, arc_span);

    // Dark settings circle disc inside the morphing arc (fades in as arc morphs into circle)
    if hover > 0.01 {
        let disc = Path::circle(c, arc_radius);
        frame.fill(&disc, Color { a: hover * 0.95 * a, ..pal.pill });
    }

    // Arc stroke that expands to complete the circle outline
    let arc_color = if hover > 0.05 {
        Color { a: motion::lerp(0.65, 0.95, hover) * a, ..pal.ink }
    } else {
        Color { a: 0.65 * a, ..pal.edge }
    };
    let arc_width = if hover > 0.05 { 2.4 } else { 2.0 };
    frame.stroke(
        &arc_path,
        Stroke::default()
            .with_color(arc_color)
            .with_width(arc_width)
            .with_line_cap(LineCap::Round),
    );

    // Drag-drop handle: JUST 6 DOTS (NO PILL CAPSULE)
    // Spawns/emerges directly from the settings circle!
    let out = ui.anim.grip_out.clamp(0.0, 1.0);
    if out > 0.01 {
        let vert = ui.is_vertical();
        let reach = 9.9 * out;
        let spawn_center = if vert {
            Point::new(c.x, motion::lerp(c.y + arc_radius * 0.5, layout.grip_center.y, out))
        } else {
            Point::new(motion::lerp(c.x + arc_radius * 0.5, layout.grip_center.x, out), c.y)
        };
        let dots = goo::grip_dots(spawn_center, reach, vert);
        let dot_r = 2.4;

        // Render purely the 6 dots - no pill capsule background or border
        for pt in &dots {
            frame.fill(&Path::circle(*pt, dot_r), Color { a: out * 0.85 * a, ..pal.ink_4 });
        }
    }
}

// ---------------- View Composition ----------------

pub fn view<'a>(
    app: &'a AppHandle,
    ui: &'a NotchUiState,
    cells: &'a [NotchCell],
) -> Element<'a, NotchMsg> {
    let layout = compute_layout(ui, cells);
    let st = app.state::<AppState>();
    let cfg = st.cfg.lock().unwrap().clone();
    let glyphs_map = st.glyphs.lock().unwrap().clone();
    let store_snap = st.store.lock().unwrap().snapshot("en", "en", crate::i18n::clock_24h(), false);
    let acts = st.activity.lock().unwrap().clone();
    let now = core::now_ms();
    let pal = theme::palette(ui.dark);
    let s = ui.scale;
    let (win_w, win_h) = ui.window_logical_size();

    let bg_canvas = iced::widget::canvas(NotchCanvasProgram {
        ui: ui.clone(),
        cells: cells.to_vec(),
        layout: layout.clone(),
        cfg: cfg.clone(),
        store_snap: store_snap.clone(),
        acts: acts.clone(),
        now,
    })
    .width(Length::Fixed(win_w))
    .height(Length::Fixed(win_h));

    let mut layers: Vec<Element<'a, NotchMsg>> = vec![bg_canvas.into()];

    if ui.anim.fold_t < 0.98 {
        // Center SVG icons inside the 44px ring centers
        for (i, cell) in cells.iter().enumerate() {
            let target = layout.ring_centers[i];
            let rest_center = Point::new(
                layout.rest_rect.x + layout.rest_rect.width * 0.5,
                layout.rest_rect.y + layout.rest_rect.height * 0.5,
            );
            let fold = ui.anim.fold_t.clamp(0.0, 1.0);
            let rc = Point::new(
                motion::lerp(target.x, rest_center.x, fold),
                motion::lerp(target.y, rest_center.y, fold),
            );
            let cell_reveal = ((1.0 - fold) * (1.0 - (fold * 1.6 - i as f32 * 0.12).clamp(0.0, 1.0))).max(0.0);
            if cell_reveal < 0.02 { continue; }
            let stale = stale_of(&cell.snap, now);
            let head = core::ring_window(
                &cell.base,
                &cell.snap.windows,
                &cfg.antigravity_limit,
                &cfg.antigravity_model,
            )
            .filter(|w| cell.base != "claude" || claude_quota_current(&cell.snap, w, now));
            let spent = head.is_some_and(|w| w.used >= 1.0);
            let dim = spent || stale;

            let icon_size = 20.4 * s;
            let glyph_el = render_glyph_widget(
                glyphs_map.get(&cell.base),
                &cell.glyph_fallback,
                icon_size,
                Color { a: cell_reveal, ..pal.ink_2 },
                dim,
            );

            // Container takes full stage with exact padding to pin the glyph
            layers.push(
                container(glyph_el)
                    .width(Length::Fixed(win_w))
                    .height(Length::Fixed(win_h))
                    .padding(Padding {
                        top: (rc.y - 10.2) * s,
                        left: (rc.x - 10.2) * s,
                        ..Padding::ZERO
                    })
                    .into(),
            );
        }

        // Gear icon when hovered (options) - fades into the circle
        if ui.anim.orb_t > 0.05 {
            let gear_svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M10.03 4.66L10.14 1.97 A10.20 10.20 0 0 1 13.86 1.97 L13.97 4.66A7.60 7.60 0 0 1 15.80 5.42L17.78 3.59 A10.20 10.20 0 0 1 20.41 6.22 L18.58 8.20A7.60 7.60 0 0 1 19.34 10.03L22.03 10.14 A10.20 10.20 0 0 1 22.03 13.86 L19.34 13.97A7.60 7.60 0 0 1 18.58 15.80L20.41 17.78 A10.20 10.20 0 0 1 17.78 20.41 L15.80 18.58A7.60 7.60 0 0 1 13.97 19.34L13.86 22.03 A10.20 10.20 0 0 1 10.14 22.03 L10.03 19.34A7.60 7.60 0 0 1 8.20 18.58L6.22 20.41 A10.20 10.20 0 0 1 3.59 17.78 L5.42 15.80A7.60 7.60 0 0 1 4.66 13.97L1.97 13.86 A10.20 10.20 0 0 1 1.97 10.14 L4.66 10.03A7.60 7.60 0 0 1 5.42 8.20L3.59 6.22 A10.20 10.20 0 0 1 6.22 3.59 L8.20 5.42A7.60 7.60 0 0 1 10.03 4.66 ZM7.90 12a4.10 4.10 0 1 0 8.20 0a4.10 4.10 0 1 0 -8.20 0Z"/></svg>"#;
            let gear_alpha = ((ui.anim.orb_t - 0.05) / 0.95).clamp(0.0, 1.0);
            let gear_size = 17.0 * s;
            let gear = svg(svg::Handle::from_memory(gear_svg.as_slice()))
                .width(Length::Fixed(gear_size))
                .height(Length::Fixed(gear_size))
                .style(move |_, _| svg::Style { color: Some(Color { a: gear_alpha, ..pal.ink_2 }) });
            layers.push(
                container(gear)
                    .width(Length::Fixed(win_w))
                    .height(Length::Fixed(win_h))
                    .padding(Padding {
                        top: (layout.orb_center.y - 8.5) * s,
                        left: (layout.orb_center.x - 8.5) * s,
                        ..Padding::ZERO
                    })
                    .into(),
            );
        }

        // Hover Card
        if let Some(cr) = layout.card_rect {
            let hovered_cell = ui
                .hover_id
                .as_deref()
                .and_then(|id| cells.iter().find(|c| c.id == id))
                .or_else(|| cells.first());
            if let Some(cell) = hovered_cell {
                let card = render_card(
                    cell,
                    glyphs_map.get(&cell.base),
                    &cfg,
                    &store_snap,
                    &acts,
                    pal,
                    s,
                    now,
                );
                layers.push(
                    container(card)
                        .width(Length::Fixed(win_w))
                        .height(Length::Fixed(win_h))
                        .padding(Padding {
                            top: cr.y * s,
                            left: cr.x * s,
                            ..Padding::ZERO
                        })
                        .into(),
                );
            }
        }
    }

    let content = stack(layers)
        .width(Length::Fixed(win_w))
        .height(Length::Fixed(win_h));

    mouse_area(content)
        .on_enter(NotchMsg::PointerEntered)
        .on_exit(NotchMsg::PointerExited)
        .on_move(NotchMsg::PointerMoved)
        .on_press(NotchMsg::LeftPress)
        .on_release(NotchMsg::LeftRelease)
        .on_right_press(NotchMsg::RightPress)
        .into()
}

pub fn render_glyph_widget<'a, M: 'a>(
    g: Option<&glyphs::Glyph>,
    fallback: &str,
    size_px: f32,
    ink: Color,
    dim: bool,
) -> Element<'a, M> {
    let opacity = if dim { 0.55 } else { 1.0 };
    if let Some(g) = g {
        if g.kind == "svg" && !g.svg.is_empty() {
            return svg(svg::Handle::from_memory(g.svg.as_bytes().to_vec()))
                .width(Length::Fixed(size_px))
                .height(Length::Fixed(size_px))
                .opacity(opacity)
                .style(move |_, _| svg::Style { color: Some(ink) })
                .into();
        }
        if let Some(bytes) = &g.png_bytes {
            return image(image::Handle::from_bytes(bytes.clone()))
                .width(Length::Fixed(size_px))
                .height(Length::Fixed(size_px))
                .opacity(opacity)
                .into();
        }
    }
    let font_size = if fallback.chars().count() > 1 { size_px * 0.65 } else { size_px * 0.85 };
    container(
        text(fallback.to_string())
            .size(font_size)
            .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT })
            .color(Color { a: opacity, ..ink }),
    )
    .width(Length::Fixed(size_px))
    .height(Length::Fixed(size_px))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}

#[allow(clippy::too_many_arguments)]
pub fn render_card<'a>(
    cell: &NotchCell,
    glyph: Option<&glyphs::Glyph>,
    cfg: &config::Config,
    store_snap: &state::Snapshot,
    acts: &[activity::Activity],
    pal: &'static theme::Palette,
    s: f32,
    now: u64,
) -> Element<'a, NotchMsg> {
    let snap = &cell.snap;
    let ramp = cfg.color_transition == "ramp";
    let mut col = Column::new().spacing(6.0 * s);

    let title_name = if cell.account_line.is_empty() { &cell.name } else { "Claude" };
    let head = row![
        render_glyph_widget(glyph, &cell.glyph_fallback, 16.0 * s, pal.ink_2, false),
        text(format!("{title_name} Usage"))
            .size(14.0 * s)
            .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT })
            .color(pal.ink),
    ]
    .spacing(8.0 * s)
    .align_y(Vertical::Center);
    col = col.push(head);

    if !cell.account_line.is_empty() {
        col = col.push(text(cell.account_line.clone()).size(11.0 * s).color(pal.ink_dim));
    }
    if stale_of(snap, now) && snap.fetched_at > 0 {
        col = col.push(
            text(format!("Updated {}", traymenu::ago(snap.fetched_at, now, "en")))
                .size(11.0 * s)
                .color(pal.ink_dim),
        );
    }

    if snap.status == "needsAuth" {
        let who = match cell.base.as_str() {
            "claude" => "Sign in to Claude Code to see usage.",
            "codex" => "Sign in to Codex to see usage.",
            "copilot" => "Run gh auth login to see usage.",
            "opencode" => "Run opencode auth login to see usage.",
            "gemini" => "Sign in to Antigravity to see usage.",
            _ => "",
        };
        let msg = if snap.note.is_empty() { who.to_string() } else { format!("{who}\n{}", snap.note) };
        col = col.push(text(msg).size(12.0 * s).color(pal.ink_3));
    } else if snap.windows.is_empty() {
        let note = if snap.note.is_empty() { "Waiting for first reading…".to_string() } else { snap.note.clone() };
        col = col.push(text(note).size(12.0 * s).color(pal.ink_3));
    } else {
        if cell.base == "claude" && snap.windows.iter().any(|w| !claude_quota_current(snap, w, now)) {
            col = col.push(text("Last known usage — not current.").size(11.0 * s).color(pal.ink_3));
        }

        for w in &snap.windows {
            if let Some(count) = w.count {
                let req_str = if count > 0 {
                    format!("~{count} {} today", if count == 1 { "request" } else { "requests" })
                } else {
                    "no requests today".into()
                };
                col = col.push(
                    column![
                        text(w.label.clone()).size(12.0 * s).color(pal.ink_2),
                        text(req_str).size(11.0 * s).color(pal.ink_dim),
                    ]
                    .spacing(3.0 * s),
                );
                continue;
            }
            if let Some(amount) = &w.amount {
                col = col.push(
                    column![
                        text(w.label.clone()).size(12.0 * s).color(pal.ink_2),
                        text(amount.clone()).size(11.0 * s).color(pal.ink_dim),
                    ]
                    .spacing(3.0 * s),
                );
                continue;
            }

            let reset_str = if cell.base == "claude" && !claude_quota_current(snap, w, now) {
                String::new()
            } else {
                w.resets_at.map(|ms| traymenu::reset_text(ms, now, "en")).unwrap_or_default()
            };

            let fill_frac = if cfg.notch_shows_remaining {
                (1.0f64 - w.used).clamp(0.0, 1.0)
            } else {
                w.used.clamp(0.0, 1.0)
            } as f32;
            let fill_col = theme::tone(w.used, cfg.watch_limit, cfg.critical_limit, ramp, pal);
            let bar_w = (246.0 - 32.0) * s;
            let fill_w = (bar_w * fill_frac).max(if fill_frac > 0.0 { 2.0 * s } else { 0.0 });

            let bar = container(
                container(space::horizontal().width(Length::Fixed(fill_w)).height(Length::Fixed(4.0 * s)))
                    .style(move |_| container::Style {
                        background: Some(fill_col.into()),
                        border: Border { radius: (2.0 * s).into(), ..Border::default() },
                        ..container::Style::default()
                    }),
            )
            .width(Length::Fixed(bar_w))
            .height(Length::Fixed(4.0 * s))
            .style(move |_| container::Style {
                background: Some(pal.bar.into()),
                border: Border { radius: (2.0 * s).into(), ..Border::default() },
                ..container::Style::default()
            });

            let mut used_copy = traymenu::used_left(w, "en");
            if cfg.show_usage_pace {
                if let Some(def) = traymenu::deficit_text(w, now, "en") {
                    used_copy.push_str(" · ");
                    used_copy.push_str(&def);
                }
            }

            col = col.push(
                column![
                    row![
                        text(w.label.clone())
                            .size(12.0 * s)
                            .font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT })
                            .color(pal.ink_2),
                        space::horizontal(),
                        text(reset_str).size(11.0 * s).color(pal.ink_dim),
                    ]
                    .align_y(Vertical::Center),
                    bar,
                    text(used_copy).size(11.0 * s).color(pal.ink_dim),
                ]
                .spacing(4.0 * s),
            );
        }

        if !snap.note.is_empty() {
            col = col.push(text(snap.note.clone()).size(11.5 * s).color(pal.ink_3));
        }
    }

    if cell.id == "claude" && snap.status == "needsAuth" {
        let auth = claude_auth::state();
        let label = if auth.busy { "Signing in..." } else { "Sign in" };
        let btn = button(text(label).size(11.0 * s).color(pal.ink))
            .padding([5.0 * s, 9.0 * s])
            .style(move |_, status| {
                let bg = if matches!(status, button::Status::Hovered) { pal.btn_hover } else { pal.btn };
                button::Style {
                    background: Some(bg.into()),
                    border: Border { color: pal.btn_line, width: 1.0, radius: (7.0 * s).into() },
                    text_color: pal.ink,
                    ..button::Style::default()
                }
            });
        let btn = if auth.busy { btn } else { btn.on_press(NotchMsg::ClaudeSignIn) };
        col = col.push(btn);
        if !auth.message.is_empty() {
            col = col.push(text(auth.message).size(10.0 * s).color(pal.ink_3));
        }
    }

    if cell.base == "claude" {
        let active: Vec<_> = store_snap.sessions.iter().filter(|s| s.state != state::ST_IDLE).collect();
        if !active.is_empty() {
            let mut s_col = Column::new().spacing(3.0 * s);
            for sess in active.iter().take(5) {
                let dot_col = match sess.state.as_str() {
                    state::ST_RUNNING => pal.ink,
                    state::ST_ATTENTION => pal.watch,
                    state::ST_DONE => pal.ample,
                    _ => pal.track,
                };
                let dot = container(space::horizontal().width(6.0 * s).height(6.0 * s)).style(move |_| container::Style {
                    background: Some(dot_col.into()),
                    border: Border { radius: (3.0 * s).into(), ..Border::default() },
                    ..container::Style::default()
                });
                s_col = s_col.push(
                    row![dot, text(sess.title.clone()).size(11.0 * s).color(pal.ink_4)]
                        .spacing(6.0 * s)
                        .align_y(Vertical::Center),
                );
            }
            if active.len() > 5 {
                s_col = s_col.push(text(format!("and {} more", active.len() - 5)).size(11.0 * s).color(pal.ink_dim));
            }
            col = col.push(s_col);
        }
    } else {
        let matching: Vec<_> = acts.iter().filter(|a| a.provider == cell.base).collect();
        if !matching.is_empty() {
            let mut a_col = Column::new().spacing(3.0 * s);
            for a in matching.iter().take(5) {
                let dot_col = if a.state == "waiting" { pal.watch } else { pal.ink };
                let dot = container(space::horizontal().width(6.0 * s).height(6.0 * s)).style(move |_| container::Style {
                    background: Some(dot_col.into()),
                    border: Border { radius: (3.0 * s).into(), ..Border::default() },
                    ..container::Style::default()
                });
                a_col = a_col.push(
                    row![
                        dot,
                        text(a.name.clone()).size(11.0 * s).color(pal.ink_4),
                        space::horizontal(),
                        text(a.detail.clone()).size(10.5 * s).color(pal.ink_dim),
                    ]
                    .spacing(6.0 * s)
                    .align_y(Vertical::Center),
                );
            }
            col = col.push(a_col);
        }
    }

    container(col)
        .width(Length::Fixed(246.0 * s))
        .padding(16.0 * s)
        .style(move |_| container::Style {
            background: Some(pal.card.into()),
            border: Border { color: pal.card_line, width: 1.0, radius: (16.0 * s).into() },
            ..container::Style::default()
        })
        .into()
}

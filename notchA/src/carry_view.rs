use crate::{goo, motion, theme};
use iced::widget::canvas::{self, path, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct CarryUiState {
    pub active: bool,
    pub edge: String,
    pub dark: bool,
    pub scale: f32,
    pub started: Option<Instant>,
    pub follow: f32,
    pub velocity: (f32, f32),
    pub corner_morph: f32,
    pub release: f32,
}

impl CarryUiState {
    pub fn begin(&mut self, edge: &str, dark: bool, scale: f32, now: Instant) {
        self.active = true;
        self.edge = edge.to_string();
        self.dark = dark;
        self.scale = scale;
        self.started = Some(now);
        self.follow = 0.0;
        self.velocity = (0.0, 0.0);
        self.corner_morph = 0.0;
        self.release = 0.0;
    }

    pub fn end(&mut self) { self.active = false; }

    pub fn tick(&mut self, now: Instant, cursor: Option<Point>, last: Point, dt: f32) {
        if !self.active { return; }
        if let Some(start) = self.started {
            let ms = now.saturating_duration_since(start).as_millis() as f32;
            self.follow = motion::FOLLOW.at(ms / 180.0);
            self.corner_morph = motion::SETTLE.at((ms / 260.0).min(1.0)) * 0.6;
        }
        if let Some(c) = cursor {
            self.velocity = ((c.x - last.x) / dt.max(0.001), (c.y - last.y) / dt.max(0.001));
        }
    }

    pub fn busy(&self) -> bool { self.active && self.follow < 0.999 }
}

struct CarryCanvas { ui: CarryUiState }

impl canvas::Program<()> for CarryCanvas {
    type State = ();

    fn draw(&self, _state: &(), renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: iced::mouse::Cursor) -> Vec<Geometry> {
        if !self.ui.active { return vec![]; }
        let mut frame = Frame::new(renderer, bounds.size());
        let pal = theme::palette(self.ui.dark);
        let grow = 1.0 + 0.06 * self.ui.follow;
        let base = Rectangle::new(
            Point::new(18.0, 14.0),
            Size::new((bounds.width - 36.0) * grow, (bounds.height - 28.0) * grow),
        );
        let r = goo::stretch_rect(base, &self.ui.edge, self.ui.velocity.0, self.ui.velocity.1);
        let rad = goo::corner_radius(20.0, self.ui.corner_morph);
        let mut b = path::Builder::new();
        b.rounded_rectangle(r.position(), r.size(), iced::border::Radius {
            top_left: rad, top_right: rad, bottom_right: rad, bottom_left: rad,
        });
        let silhouette = b.build();
        frame.fill(&silhouette, pal.pill);
        frame.stroke(&silhouette, Stroke::default().with_color(pal.edge).with_width(1.0));

        let speed = (self.ui.velocity.0 * self.ui.velocity.0 + self.ui.velocity.1 * self.ui.velocity.1).sqrt();
        let flare = (speed / goo::STRETCH_SPEED).clamp(0.0, 1.0);
        if flare > 0.02 {
            let along_x = !matches!(self.ui.edge.as_str(), "left" | "right");
            for sign in [-1.0f32, 1.0] {
                let c = if along_x {
                    Point::new(if sign > 0.0 { r.x + r.width } else { r.x }, r.y + r.height * 0.5)
                } else {
                    Point::new(r.x + r.width * 0.5, if sign > 0.0 { r.y + r.height } else { r.y })
                };
                let arc = goo::quarter(c, 8.0 + 10.0 * flare, if sign > 0.0 { 0.25 } else { 0.75 }, 0.5);
                frame.stroke(&arc, Stroke::default().with_color(pal.edge).with_width(2.0));
            }
        }

        let grip = Point::new(r.x + r.width - 34.0, r.y + r.height + 12.0);
        let dots = goo::grip_dots(grip, 9.9 * self.ui.follow, matches!(self.ui.edge.as_str(), "left" | "right"));
        for p in goo::disc_merge(grip, &dots, 17.0, 2.8, 0.9 * self.ui.follow) {
            frame.fill(&p, pal.pill);
            frame.stroke(&p, Stroke::default().with_color(pal.edge).with_width(0.8));
        }
        if self.ui.release > 0.01 {
            let arc = goo::release_arc(grip, 30.0, self.ui.release);
            frame.stroke(&arc, Stroke::default().with_color(pal.edge).with_width(2.0));
        }
        for i in 0..3 {
            let mark = Path::circle(Point::new(r.x + 24.0 + i as f32 * 14.0, r.y + r.height - 12.0), 2.0);
            frame.fill(&mark, Color { a: 0.5, ..pal.ink_3 });
        }
        vec![frame.into_geometry()]
    }
}

pub fn view(ui: &CarryUiState) -> Element<'_, ()> {
    iced::widget::canvas(CarryCanvas { ui: ui.clone() })
        .width(Length::Fill).height(Length::Fill).into()
}

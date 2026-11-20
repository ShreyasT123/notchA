use crate::notch_view::{build_tail_path, render_glyph_widget};
use crate::reset_alert::CardPayload;
use crate::theme;
use iced::alignment::Vertical;
use iced::widget::canvas::{self, Frame, Geometry};
use iced::widget::{button, column, container, row, space, stack, text};
use iced::{Border, Element, Font, Length, Padding, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug, Clone)]
pub enum ResetMsg {
    Dismiss(u64),
}

struct TailCanvas {
    edge: String,
    attached: bool,
    dark: bool,
}

impl canvas::Program<ResetMsg> for TailCanvas {
    type State = ();
    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        if !self.attached {
            return vec![];
        }
        let mut frame = Frame::new(renderer, bounds.size());
        let pal = theme::palette(self.dark);
        let tail_rect = match self.edge.as_str() {
            "left" => Rectangle::new(Point::new(1.0, 59.0), Size::new(34.0, 32.0)),
            "top" => Rectangle::new(Point::new(123.0, 0.0), Size::new(34.0, 23.0)),
            "bottom" => Rectangle::new(Point::new(123.0, 127.0), Size::new(34.0, 23.0)),
            _ => Rectangle::new(Point::new(245.0, 59.0), Size::new(34.0, 32.0)),
        };
        let path = build_tail_path(&self.edge, tail_rect);
        frame.fill(&path, pal.card);
        vec![frame.into_geometry()]
    }
}

pub fn view(payload: &CardPayload, dark: bool) -> Element<'_, ResetMsg> {
    let pal = theme::palette(dark);
    let (card_x, card_y) = if !payload.attached {
        (17.0, 11.0)
    } else {
        match payload.edge.as_str() {
            "left" => (34.0, 11.0),
            "top" => (17.0, 22.0),
            "bottom" => (17.0, 0.0),
            _ => (0.0, 11.0),
        }
    };

    let glyph = render_glyph_widget(
        Some(&payload.glyph),
        payload.title.chars().next().map(|c| c.to_string()).as_deref().unwrap_or("?"),
        21.0,
        pal.ink,
        false,
    );

    let dismiss_btn = button(text("×").size(16).color(pal.ink_3))
        .padding([2, 7])
        .on_press(ResetMsg::Dismiss(payload.token))
        .style(move |_, status| button::Style {
            background: Some(
                if matches!(status, button::Status::Hovered) {
                    pal.btn_hover
                } else {
                    iced::Color::TRANSPARENT
                }
                .into(),
            ),
            border: Border { radius: 7.0.into(), ..Border::default() },
            text_color: pal.ink_3,
            ..button::Style::default()
        });

    let dot = container(space::horizontal().width(8).height(8)).style(move |_| container::Style {
        background: Some(pal.ample.into()),
        border: Border { radius: 4.0.into(), ..Border::default() },
        ..container::Style::default()
    });

    let mut body = column![
        row![
            glyph,
            column![
                text(&payload.title)
                    .size(14)
                    .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT })
                    .color(pal.ink),
                text(&payload.subtitle).size(11).color(pal.ink_3),
            ]
            .spacing(1),
            space::horizontal(),
            dismiss_btn,
        ]
        .spacing(9)
        .align_y(Vertical::Center),
        row![dot, text(&payload.status).size(12).color(pal.ample)]
            .spacing(8)
            .align_y(Vertical::Center),
    ]
    .spacing(10);

    if !payload.next.is_empty() {
        body = body.push(
            container(text(&payload.next).size(11).color(pal.ink_3))
                .padding(Padding { left: 16.0, ..Padding::ZERO }),
        );
    }

    let card = container(body)
        .width(Length::Fixed(246.0))
        .height(Length::Fixed(128.0))
        .padding([13, 15])
        .style(move |_| container::Style {
            background: Some(pal.card.into()),
            border: Border {
                color: pal.card_line,
                width: 1.0,
                radius: 16.0.into(),
            },
            ..container::Style::default()
        });

    let bg = iced::widget::canvas(TailCanvas {
        edge: payload.edge.clone(),
        attached: payload.attached,
        dark,
    })
    .width(Length::Fixed(280.0))
    .height(Length::Fixed(150.0));

    stack![
        bg,
        container(card).padding(Padding {
            top: card_y,
            left: card_x,
            ..Padding::ZERO
        })
    ]
    .width(Length::Fixed(280.0))
    .height(Length::Fixed(150.0))
    .into()
}
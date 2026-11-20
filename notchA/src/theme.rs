use iced::Color;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Palette {
    pub pill: Color,
    pub edge: Color,
    pub rest_edge: Color,
    pub card: Color,
    pub card_line: Color,
    pub card_rule: Color,
    pub bar: Color,
    pub track: Color,
    pub hole: Color,
    pub ample: Color,
    pub watch: Color,
    pub crit: Color,
    pub ink: Color,
    pub ink_2: Color,
    pub ink_3: Color,
    pub ink_4: Color,
    pub ink_dim: Color,
    pub btn: Color,
    pub btn_line: Color,
    pub btn_hover: Color,
    pub btn_line_hover: Color,
}

const fn hex(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xFF) as f32 / 255.0,
        ((rgb >> 8) & 0xFF) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
    )
}

pub const DARK: Palette = Palette {
    pill: hex(0x050505),
    edge: Color::from_rgba(1.0, 1.0, 1.0, 0.16),
    rest_edge: hex(0x5c5c5c),
    card: hex(0x121214),
    card_line: hex(0x28282c),
    card_rule: hex(0x222226),
    bar: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
    track: Color::from_rgba(1.0, 1.0, 1.0, 0.16),
    hole: hex(0x1a1a1c),
    ample: hex(0x00FF88),
    watch: hex(0xF2FF00),
    crit: hex(0xFF3F00),
    ink: hex(0xffffff),
    ink_2: hex(0xe8e8ea),
    ink_3: hex(0xc8c8c8),
    ink_4: hex(0xb0b0b3),
    ink_dim: hex(0x8e8e93),
    btn: hex(0x252528),
    btn_line: hex(0x444448),
    btn_hover: hex(0x38383c),
    btn_line_hover: hex(0xaaaaaa),
};

pub const LIGHT: Palette = Palette {
    pill: hex(0xf5f5f7),
    edge: hex(0xd2d2d7),
    rest_edge: hex(0x86868b),
    card: hex(0xffffff),
    card_line: hex(0xe4e4e8),
    card_rule: hex(0xe8e8ec),
    bar: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
    track: Color::from_rgba(0.0, 0.0, 0.0, 0.16),
    hole: hex(0xffffff),
    ample: hex(0x00A356),
    watch: hex(0xB08800),
    crit: hex(0xFF3F00),
    ink: hex(0x1d1d1f),
    ink_2: hex(0x1d1d1f),
    ink_3: hex(0x3c3c43),
    ink_4: hex(0x4a4a4f),
    ink_dim: hex(0x6b6b6b),
    btn: hex(0xffffff),
    btn_line: hex(0xc6c6cc),
    btn_hover: hex(0xf0f0f2),
    btn_line_hover: hex(0x8e8e93),
};

pub fn palette(dark: bool) -> &'static Palette {
    if dark { &DARK } else { &LIGHT }
}

pub fn tone(
    used: f64,
    watch_limit: f64,
    critical_limit: f64,
    ramp: bool,
    pal: &Palette,
) -> Color {
    let f = used.clamp(0.0, 1.0) as f32;
    let watch = watch_limit as f32;
    let crit = critical_limit as f32;

    if !ramp {
        return if f >= crit {
            pal.crit
        } else if f >= watch {
            pal.watch
        } else {
            pal.ample
        };
    }

    if f < watch {
        lerp_color(pal.ample, pal.watch, (f / watch.max(0.001)).clamp(0.0, 1.0))
    } else {
        lerp_color(
            pal.watch,
            pal.crit,
            ((f - watch) / (1.0 - watch).max(0.001)).clamp(0.0, 1.0),
        )
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
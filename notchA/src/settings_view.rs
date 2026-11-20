use crate::core::{self, AppHandle, AppState, Manager};
use crate::notch_view::render_glyph_widget;
use crate::{claude_accounts, claude_auth, config, hooks_install, reset_alert, settings_window, taskbar};
use iced::alignment::Vertical;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, slider, space, text, toggler, Column,
};
use iced::{Border, Color, Element, Font, Length, Padding};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Accounts,
    Appearance,
    General,
}

#[derive(Debug, Clone)]
pub enum SettingsMsg {
    Tab(SettingsTab),
    Close,
    Quit,
    ToggleProvider(String),
    ToggleClaudeAccount(String),
    ClaudeSignIn(String),
    AddClaudeAccount,
    ToggleHooks(bool),
    SetAgLimit(String),
    SetAgModel(String),
    SetTaskbarMode(String),
    SetShowMode(String),
    ToggleFoldFullscreen(bool),
    ToggleAdaptivePill(bool),
    SetScale(f64),
    SetTheme(String),
    SetWeeklyRing(String),
    ToggleWeeklyDashed(bool),
    ToggleRemaining(bool),
    TogglePace(bool),
    SetEdge(String),
    SetMonitor(String),
    Recentre,
    SetColorTransition(String),
    SetWatchLimit(u8),
    SetCriticalLimit(u8),
    ResetLimits,
    SetTrayVisible(bool),
    ToggleResetAlert(bool),
    ToggleResetSound(bool),
    PreviewResetAlert,
    OpenDataDir,
    OpenAuthor,
}

#[derive(Clone)]
struct LookPalette {
    accent: Color,
    on_accent: Color,
    text: Color,
    text2: Color,
    pane: Color,
    side: Color,
    group: Color,
    line: Color,
    sep: Color,
    seg_bg: Color,
    btn: Color,
    btn_hover: Color,
    hover: Color,
    sel_soft: Color,
    danger_bg: Color,
    danger: Color,
}

fn look_palette(dark: bool, look: &settings_window::SystemLook) -> LookPalette {
    let accent = if dark {
        look.accent.get(1).copied().unwrap_or(Color::from_rgb8(0x4c, 0xa3, 0xff))
    } else {
        look.accent.get(4).copied().unwrap_or(Color::from_rgb8(0x0a, 0x7a, 0xff))
    };
    let on_accent = if dark { Color::BLACK } else { Color::WHITE };

    if dark {
        LookPalette {
            accent,
            on_accent,
            text: Color::from_rgb8(0xf2, 0xf2, 0xf4),
            text2: Color::from_rgba8(235, 235, 245, 0.65),
            pane: if look.mica {
                Color::from_rgba(0.12, 0.12, 0.14, 0.72)
            } else {
                Color::from_rgb8(0x2b, 0x2b, 0x2e)
            },
            side: if look.mica {
                Color::from_rgba(0.0, 0.0, 0.0, 0.35)
            } else {
                Color::from_rgb8(0x1f, 0x20, 0x22)
            },
            group: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            line: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            sep: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            seg_bg: Color::from_rgba(1.0, 1.0, 1.0, 0.09),
            btn: Color::from_rgba(1.0, 1.0, 1.0, 0.10),
            btn_hover: Color::from_rgba(1.0, 1.0, 1.0, 0.16),
            hover: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            sel_soft: Color::from_rgba(1.0, 1.0, 1.0, 0.09),
            danger_bg: Color::from_rgba8(255, 99, 99, 0.14),
            danger: Color::from_rgb8(0xff, 0x8a, 0x8a),
        }
    } else {
        LookPalette {
            accent,
            on_accent,
            text: Color::from_rgb8(0x1d, 0x1d, 0x1f),
            text2: Color::from_rgba8(60, 60, 67, 0.75),
            pane: if look.mica {
                Color::from_rgba(0.96, 0.96, 0.98, 0.78)
            } else {
                Color::from_rgb8(0xf0, 0xf0, 0xf2)
            },
            side: if look.mica {
                Color::from_rgba(0.0, 0.0, 0.0, 0.06)
            } else {
                Color::from_rgb8(0xe2, 0xe2, 0xe6)
            },
            group: Color::from_rgba(1.0, 1.0, 1.0, 0.75),
            line: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            sep: Color::from_rgba(0.0, 0.0, 0.0, 0.07),
            seg_bg: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
            btn: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
            btn_hover: Color::from_rgba(0.0, 0.0, 0.0, 0.10),
            hover: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
            sel_soft: Color::from_rgba(0.0, 0.0, 0.0, 0.07),
            danger_bg: Color::from_rgba8(196, 43, 28, 0.10),
            danger: Color::from_rgb8(0xc4, 0x2b, 0x1c),
        }
    }
}

pub fn view<'a>(
    app: &'a AppHandle,
    tab: SettingsTab,
    dark: bool,
    status_err: Option<&'a str>,
    toast: Option<&'a str>,
    look: &settings_window::SystemLook,
) -> Element<'a, SettingsMsg> {
    let p = look_palette(dark, look);
    let st = app.state::<AppState>();
    let cfg = st.cfg.lock().unwrap().clone();
    let glyphs_map = st.glyphs.lock().unwrap().clone();

    let side_btn = |label: &'static str, t: SettingsTab| {
        let sel = tab == t;
        let bg = if sel { p.sel_soft } else { Color::TRANSPARENT };
        let p_c = p.clone();
        button(
            row![
                container(space::horizontal().width(3).height(14)).style(move |_| container::Style {
                    background: Some(if sel { p_c.accent } else { Color::TRANSPARENT }.into()),
                    border: Border { radius: 2.0.into(), ..Border::default() },
                    ..container::Style::default()
                }),
                text(label).size(13.5).color(p.text),
            ]
            .spacing(8)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .padding([7, 8])
        .on_press(SettingsMsg::Tab(t))
        .style(move |_, status| button::Style {
            background: Some(
                if matches!(status, button::Status::Hovered) && !sel {
                    p_c.hover
                } else {
                    bg
                }
                .into(),
            ),
            border: Border { radius: 8.0.into(), ..Border::default() },
            text_color: p_c.text,
            ..button::Style::default()
        })
    };

    let p_quit = p.clone();
    let quit_btn = button(text("Quit notchA").size(13).color(p.text))
        .width(Length::Fill)
        .padding([7, 10])
        .on_press(SettingsMsg::Quit)
        .style(move |_, status| button::Style {
            background: Some(
                if matches!(status, button::Status::Hovered) {
                    p_quit.danger_bg
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            border: Border { radius: 8.0.into(), ..Border::default() },
            text_color: p_quit.text,
            ..button::Style::default()
        });

    let p_side = p.clone();
    let sidebar = container(
        column![
            space::vertical().height(18),
            side_btn("Accounts", SettingsTab::Accounts),
            side_btn("Appearance", SettingsTab::Appearance),
            side_btn("General", SettingsTab::General),
            space::vertical(),
            quit_btn,
        ]
        .spacing(4),
    )
    .width(Length::Fixed(188.0))
    .height(Length::Fill)
    .padding(10)
    .style(move |_| container::Style {
        background: Some(p_side.side.into()),
        border: Border {
            color: p_side.line,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..container::Style::default()
    });

    let title_str = match tab {
        SettingsTab::Accounts => "Accounts",
        SettingsTab::Appearance => "Appearance",
        SettingsTab::General => "General",
    };

    let p_close = p.clone();
    let close_btn = button(text("×").size(17).color(p.text2))
        .padding([4, 12])
        .on_press(SettingsMsg::Close)
        .style(move |_, status| button::Style {
            background: Some(
                if matches!(status, button::Status::Hovered) {
                    Color::from_rgb8(0xc4, 0x2b, 0x1c)
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            text_color: if matches!(status, button::Status::Hovered) {
                Color::WHITE
            } else {
                p_close.text2
            },
            ..button::Style::default()
        });

    let mut header_col = column![row![
        text(title_str)
            .size(22)
            .font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT })
            .color(p.text),
        space::horizontal(),
        close_btn,
    ]
    .align_y(Vertical::Center)]
    .spacing(8);

    if let Some(err) = status_err {
        let p_err = p.clone();
        header_col = header_col.push(
            container(text(err).size(12).color(p.danger))
                .width(Length::Fill)
                .padding([6, 10])
                .style(move |_| container::Style {
                    background: Some(p_err.danger_bg.into()),
                    border: Border { radius: 8.0.into(), ..Border::default() },
                    ..container::Style::default()
                }),
        );
    }

    let body: Element<'a, SettingsMsg> = match tab {
        SettingsTab::Accounts => view_accounts(app, &cfg, &glyphs_map, &p),
        SettingsTab::Appearance => view_appearance(&cfg, &p),
        SettingsTab::General => view_general(&cfg, &p),
    };

    let mut right_pane = column![header_col, scrollable(body).height(Length::Fill)]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding { top: 10.0, right: 16.0, bottom: 14.0, left: 16.0 });

    if let Some(msg) = toast {
        right_pane = right_pane.push(
            row![
                space::horizontal(),
                text(msg).size(12).color(p.text2),
            ],
        );
    }

    let p_root = p.clone();
    container(row![sidebar, right_pane].spacing(6).padding(6))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(p_root.pane.into()),
            ..container::Style::default()
        })
        .into()
}

fn group_box<'a>(items: Vec<Element<'a, SettingsMsg>>, p: &LookPalette) -> Element<'a, SettingsMsg> {
    let mut col = Column::new();
    let len = items.len();
    for (i, item) in items.into_iter().enumerate() {
        col = col.push(container(item).padding([9, 12]).width(Length::Fill));
        if i + 1 < len {
            let sep = p.sep;
            col = col.push(
                container(space::horizontal().width(Length::Fill).height(1))
                    .style(move |_| container::Style {
                        background: Some(sep.into()),
                        ..container::Style::default()
                    }),
            );
        }
    }
    let p_g = p.clone();
    container(col)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(p_g.group.into()),
            border: Border {
                color: p_g.line,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn seg_control<'a>(
    options: &[(&'static str, &'static str)],
    current: &str,
    p: &LookPalette,
    make_msg: impl Fn(&'static str) -> SettingsMsg + 'a,
) -> Element<'a, SettingsMsg> {
    let mut r = row![].spacing(2);
    for &(val, label) in options {
        let on = val == current;
        let p_c = p.clone();
        r = r.push(
            button(text(label).size(12.5).color(if on { p.on_accent } else { p.text }))
                .padding([4, 11])
                .on_press(make_msg(val))
                .style(move |_, status| button::Style {
                    background: Some(
                        if on {
                            p_c.accent
                        } else if matches!(status, button::Status::Hovered) {
                            p_c.hover
                        } else {
                            Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    border: Border { radius: 5.0.into(), ..Border::default() },
                    text_color: if on { p_c.on_accent } else { p_c.text },
                    ..button::Style::default()
                }),
        );
    }
    let bg = p.seg_bg;
    container(r)
        .padding(2)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            border: Border { radius: 7.0.into(), ..Border::default() },
            ..container::Style::default()
        })
        .into()
}

fn action_btn<'a>(label: &'static str, msg: SettingsMsg, p: &LookPalette) -> Element<'a, SettingsMsg> {
    let p_c = p.clone();
    button(text(label).size(12.5).color(p.text))
        .padding([5, 12])
        .on_press(msg)
        .style(move |_, status| button::Style {
            background: Some(
                if matches!(status, button::Status::Hovered) {
                    p_c.btn_hover
                } else {
                    p_c.btn
                }
                .into(),
            ),
            border: Border { radius: 6.0.into(), ..Border::default() },
            text_color: p_c.text,
            ..button::Style::default()
        })
        .into()
}

fn view_accounts<'a>(
    app: &'a AppHandle,
    cfg: &config::Config,
    glyphs_map: &std::collections::HashMap<String, crate::glyphs::Glyph>,
    p: &LookPalette,
) -> Element<'a, SettingsMsg> {
    let mut rows: Vec<Element<'a, SettingsMsg>> = Vec::new();
    let all_on = cfg.notch_slots.is_empty();
    let provider_on = |id: &str| all_on || cfg.notch_slots.iter().any(|s| s.provider == id);

    // Claude accounts
    let claude_on = provider_on("claude");
    let claude_list = claude_accounts::get_claude_accounts(app.clone());
    let auth = claude_auth::state();
    for acc in &claude_list {
        let is_on = claude_on && acc.enabled;
        let glyph = render_glyph_widget(glyphs_map.get("claude"), "C", 16.0, p.text, !is_on);
        let acc_id = acc.id.clone();
        let acc_id2 = acc.id.clone();
        let mut right_ctrl = row![].spacing(8).align_y(Vertical::Center);
        if is_on && (!acc.connected || acc.status == "needsAuth") && !auth.busy {
            right_ctrl = right_ctrl.push(action_btn("Sign in", SettingsMsg::ClaudeSignIn(acc_id2), p));
        }
        right_ctrl = right_ctrl.push(toggler(is_on).on_toggle(move |_| SettingsMsg::ToggleClaudeAccount(acc_id.clone())));

        let mut item_col = column![
            row![glyph, text(acc.name.clone()).size(13.5).color(p.text), space::horizontal(), right_ctrl]
                .spacing(10)
                .align_y(Vertical::Center)
        ]
        .spacing(4);

        let detail = if !is_on {
            acc.address.clone().unwrap_or_default()
        } else if auth.busy && auth.profile_id.as_deref() == Some(acc.id.as_str()) {
            "Finish signing in in the browser or terminal window.".into()
        } else if !acc.connected || acc.status == "needsAuth" {
            "not signed in".into()
        } else {
            let st = match acc.status.as_str() {
                "ok" => "",
                "stale" => "last reading is old",
                "backoff" => "rate limited, trying again soon",
                "error" => "not reachable",
                other => other,
            };
            [acc.address.as_deref().unwrap_or(""), st]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" · ")
        };
        if !detail.is_empty() {
            item_col = item_col.push(text(detail).size(12).color(p.text2));
        }
        if acc.id == "default" {
            let hooks_on = hooks_install::is_installed();
            item_col = item_col.push(
                row![
                    text("Let Claude Code notify notchA").size(12.5).color(p.text),
                    space::horizontal(),
                    toggler(hooks_on).on_toggle(SettingsMsg::ToggleHooks),
                ]
                .align_y(Vertical::Center),
            );
        }
        rows.push(item_col.into());
    }

    rows.push(
        row![
            text("Add a Claude account").size(13).color(p.text),
            space::horizontal(),
            action_btn("Add…", SettingsMsg::AddClaudeAccount, p),
        ]
        .align_y(Vertical::Center)
        .into(),
    );

    // Other 4 providers
    for id in ["codex", "opencode", "copilot", "gemini"] {
        let on = provider_on(id);
        let snap = core::snapshot_of(app, id);
        let label = core::provider_label(id);
        let fb = match id {
            "codex" => "Cx",
            "opencode" => "OC",
            "copilot" => "Cp",
            _ => "Ag",
        };
        let glyph = render_glyph_widget(glyphs_map.get(id), fb, 16.0, p.text, !on);
        let id_owned = id.to_string();
        let mut item_col = column![
            row![
                glyph,
                text(label).size(13.5).color(p.text),
                space::horizontal(),
                toggler(on).on_toggle(move |_| SettingsMsg::ToggleProvider(id_owned.clone())),
            ]
            .spacing(10)
            .align_y(Vertical::Center)
        ]
        .spacing(4);

        let st_txt: String = match snap.status.as_str() {
            "ok" => String::new(),
            "stale" => "last reading is old".into(),
            "needsAuth" => "not signed in".into(),
            "absent" => "not installed".into(),
            "backoff" => "rate limited, trying again soon".into(),
            "error" => "not reachable".into(),
            other => other.to_string(),
        };
        if !st_txt.is_empty() {
            item_col = item_col.push(text(st_txt).size(12).color(p.text2));
        }
        if id == "gemini" && on {
            let lims = vec!["automatic".to_string(), "5h".to_string(), "weekly".to_string()];
            let mods = vec!["gemini".to_string(), "3p".to_string()];
            item_col = item_col
                .push(
                    row![
                        text("Notch reads").size(12.5).color(p.text2),
                        space::horizontal(),
                        pick_list(lims, Some(cfg.antigravity_limit.clone()), SettingsMsg::SetAgLimit),
                    ]
                    .align_y(Vertical::Center),
                )
                .push(
                    row![
                        text("Model data").size(12.5).color(p.text2),
                        space::horizontal(),
                        pick_list(mods, Some(cfg.antigravity_model.clone()), SettingsMsg::SetAgModel),
                    ]
                    .align_y(Vertical::Center),
                );
        }
        rows.push(item_col.into());
    }

    column![
        text("Providers").size(13.5).font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT }).color(p.text),
        group_box(rows, p),
    ]
    .spacing(8)
    .into()
}

fn view_appearance<'a>(cfg: &config::Config, p: &LookPalette) -> Element<'a, SettingsMsg> {
    let tb_modes = vec!["off".to_string(), "left".to_string(), "right".to_string()];
    let tb_group = group_box(
        vec![
            row![
                text("Show rings in").size(13.5).color(p.text),
                space::horizontal(),
                pick_list(tb_modes, Some(cfg.taskbar_mode.clone()), SettingsMsg::SetTaskbarMode),
            ]
            .align_y(Vertical::Center)
            .into(),
            text("Use taskbar free space (left/right) or the floating Notch (off).").size(12).color(p.text2).into(),
        ],
        p,
    );

    let show_mode = if !cfg.notch_visible {
        "hide"
    } else if cfg.notch_on_hover {
        "hover"
    } else {
        "show"
    };
    let size_str = if cfg.scale < 0.9 {
        "0.8"
    } else if cfg.scale < 1.125 {
        "1"
    } else {
        "1.25"
    };

    let monitors: Vec<String> = core::screens()
        .into_iter()
        .filter_map(|s| s.name)
        .collect();
    let cur_mon = cfg
        .notch_monitor
        .clone()
        .or_else(|| monitors.first().cloned())
        .unwrap_or_default();

    let mut notch_items: Vec<Element<'a, SettingsMsg>> = vec![
        row![
            text("Show").size(13.5).color(p.text),
            space::horizontal(),
            seg_control(&[("show", "Always"), ("hover", "On hover"), ("hide", "Hide")], show_mode, p, |v| {
                SettingsMsg::SetShowMode(v.into())
            }),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Fold for full-screen apps").size(13.5).color(p.text),
            space::horizontal(),
            toggler(cfg.fold_for_full_screen).on_toggle(SettingsMsg::ToggleFoldFullscreen),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Adaptive pill").size(13.5).color(p.text),
            space::horizontal(),
            toggler(cfg.adaptive_pill).on_toggle(SettingsMsg::ToggleAdaptivePill),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Size").size(13.5).color(p.text),
            space::horizontal(),
            seg_control(&[("0.8", "Small"), ("1", "Medium"), ("1.25", "Large")], size_str, p, |v| {
                SettingsMsg::SetScale(v.parse().unwrap_or(1.0))
            }),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Theme").size(13.5).color(p.text),
            space::horizontal(),
            seg_control(&[("system", "System"), ("light", "Light"), ("dark", "Dark")], &cfg.theme, p, |v| {
                SettingsMsg::SetTheme(v.into())
            }),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Weekly ring").size(13.5).color(p.text),
            space::horizontal(),
            seg_control(&[("off", "Off"), ("inside", "Inside"), ("outside", "Outside")], &cfg.weekly_ring, p, |v| {
                SettingsMsg::SetWeeklyRing(v.into())
            }),
        ]
        .align_y(Vertical::Center)
        .into(),
    ];

    if cfg.weekly_ring != "off" {
        notch_items.push(
            row![
                text("Dashed weekly ring").size(13.5).color(p.text),
                space::horizontal(),
                toggler(cfg.weekly_ring_dashed).on_toggle(SettingsMsg::ToggleWeeklyDashed),
            ]
            .align_y(Vertical::Center)
            .into(),
        );
    }

    notch_items.extend([
        row![
            text("Show remaining instead of used").size(13.5).color(p.text),
            space::horizontal(),
            toggler(cfg.notch_shows_remaining).on_toggle(SettingsMsg::ToggleRemaining),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Show usage pace").size(13.5).color(p.text),
            space::horizontal(),
            toggler(cfg.show_usage_pace).on_toggle(SettingsMsg::TogglePace),
        ]
        .align_y(Vertical::Center)
        .into(),
        row![
            text("Edge").size(13.5).color(p.text),
            space::horizontal(),
            seg_control(&[("left", "Left"), ("right", "Right"), ("top", "Top"), ("bottom", "Bottom")], &cfg.notch_edge, p, |v| {
                SettingsMsg::SetEdge(v.into())
            }),
        ]
        .align_y(Vertical::Center)
        .into(),
    ]);

    if monitors.len() > 1 {
        notch_items.push(
            row![
                text("Screen").size(13.5).color(p.text),
                space::horizontal(),
                pick_list(monitors, Some(cur_mon), SettingsMsg::SetMonitor),
            ]
            .align_y(Vertical::Center)
            .into(),
        );
    }

    notch_items.push(
        row![
            text("Hold the 6 dots or Alt+drag to move around edges.").size(12).color(p.text2),
            space::horizontal(),
            action_btn("Recentre", SettingsMsg::Recentre, p),
        ]
        .spacing(12)
        .align_y(Vertical::Center)
        .into(),
    );

    let watch_pct = (cfg.watch_limit * 100.0).round().clamp(1.0, 99.0) as u8;
    let crit_pct = (cfg.critical_limit * 100.0).round().clamp(1.0, 100.0) as u8;
    let limits_group = group_box(
        vec![
            row![
                text("Colour transition").size(13.5).color(p.text),
                space::horizontal(),
                seg_control(&[("hard_step", "Hard step"), ("ramp", "Colour ramp")], &cfg.color_transition, p, |v| {
                    SettingsMsg::SetColorTransition(v.into())
                }),
            ]
            .align_y(Vertical::Center)
            .into(),
            row![
                text("Watch limit").size(13.5).color(p.text),
                space::horizontal(),
                text(format!("{watch_pct}%")).size(13).color(p.text2),
                slider(1..=99, watch_pct, SettingsMsg::SetWatchLimit).width(Length::Fixed(160.0)),
            ]
            .spacing(10)
            .align_y(Vertical::Center)
            .into(),
            row![
                text("Critical limit").size(13.5).color(p.text),
                space::horizontal(),
                text(format!("{crit_pct}%")).size(13).color(p.text2),
                slider(1..=100, crit_pct, SettingsMsg::SetCriticalLimit).width(Length::Fixed(160.0)),
            ]
            .spacing(10)
            .align_y(Vertical::Center)
            .into(),
            row![
                space::horizontal(),
                action_btn("Reset to defaults", SettingsMsg::ResetLimits, p),
            ]
            .into(),
        ],
        p,
    );

    let app_group = group_box(
        vec![
            row![
                text("Show tray icon").size(13.5).color(p.text),
                space::horizontal(),
                toggler(cfg.tray_visible).on_toggle(SettingsMsg::SetTrayVisible),
            ]
            .align_y(Vertical::Center)
            .into(),
        ],
        p,
    );

    column![
        text("Taskbar").size(13.5).font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT }).color(p.text),
        tb_group,
        text("Notch").size(13.5).font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT }).color(p.text),
        group_box(notch_items, p),
        text("Usage Limits").size(13.5).font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT }).color(p.text),
        limits_group,
        text("App").size(13.5).font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT }).color(p.text),
        app_group,
    ]
    .spacing(10)
    .into()
}

fn view_general<'a>(cfg: &config::Config, p: &LookPalette) -> Element<'a, SettingsMsg> {
let g1 = group_box(
        vec![
            row![
                text("Version").size(13.5).color(p.text),
                space::horizontal(),
                text(env!("CARGO_PKG_VERSION")).size(13).color(p.text2),
            ]
            .align_y(Vertical::Center)
            .into(),
        ],
        p,
    );

    let g2 = group_box(
        vec![
            row![
                text("Reset notifications").size(13.5).color(p.text),
                space::horizontal(),
                toggler(cfg.reset_notifications).on_toggle(SettingsMsg::ToggleResetAlert),
            ]
            .align_y(Vertical::Center)
            .into(),
            row![
                text("Notification sound").size(13.5).color(p.text),
                space::horizontal(),
                toggler(cfg.reset_notification_sound).on_toggle(SettingsMsg::ToggleResetSound),
            ]
            .align_y(Vertical::Center)
            .into(),
            row![
                text("Preview the reset card").size(13.5).color(p.text),
                space::horizontal(),
                action_btn("Preview card", SettingsMsg::PreviewResetAlert, p),
            ]
            .align_y(Vertical::Center)
            .into(),
        ],
        p,
    );

    let g3 = group_box(
        vec![
            row![
                text("Data folder").size(13.5).color(p.text),
                space::horizontal(),
                action_btn("Open folder", SettingsMsg::OpenDataDir, p),
            ]
            .align_y(Vertical::Center)
            .into(),
            row![
                text("Based on Codenotch by").size(12).color(p.text2),
                button(text("@hivinz_").size(12).color(p.accent))
                    .padding(0)
                    .on_press(SettingsMsg::OpenAuthor)
                    .style(|_, _| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
            ]
            .spacing(4)
            .align_y(Vertical::Center)
            .into(),
        ],
        p,
    );

    column![
        g1,
        text("Notifications").size(13.5).font(Font { weight: iced::font::Weight::Semibold, ..Font::DEFAULT }).color(p.text),
        g2,
        g3,
    ]
    .spacing(10)
    .into()
}

pub fn handle_msg(app: &AppHandle, msg: SettingsMsg) -> (Option<SettingsTab>, Option<String>, Option<String>) {
    let mut tab_out = None;
    let mut err_out = None;
    let mut toast_out = None;

    match msg {
        SettingsMsg::Tab(t) => tab_out = Some(t),
        SettingsMsg::Close | SettingsMsg::Quit => {}
        SettingsMsg::ToggleProvider(id) => {
            let ids = ["claude", "codex", "opencode", "copilot", "gemini"];
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            let mut on: Vec<String> = if c.notch_slots.is_empty() {
                ids.iter().map(|s| s.to_string()).collect()
            } else {
                c.notch_slots.iter().map(|s| s.provider.clone()).collect()
            };
            if let Some(pos) = on.iter().position(|p| p == &id) {
                if on.len() <= 1 {
                    toast_out = Some("At least one provider stays on the notch".into());
                } else {
                    on.remove(pos);
                }
            } else {
                on.push(id);
            }
            on.sort_by_key(|p| ids.iter().position(|&x| x == p).unwrap_or(99));
            c.notch_slots = if on.len() == ids.len() {
                Vec::new()
            } else {
                on.into_iter().map(|provider| config::TraySlot { provider }).collect()
            };
            config::save(&c);
            toast_out = toast_out.or(Some("Saved".into()));
        }
        SettingsMsg::ToggleClaudeAccount(id) => {
            let list = claude_accounts::get_claude_accounts(app.clone());
            if let Some(acc) = list.iter().find(|a| a.id == id) {
                if let Err(e) = claude_accounts::set_claude_account_enabled(app.clone(), id, !acc.enabled) {
                    err_out = Some(e);
                } else {
                    toast_out = Some("Saved".into());
                }
            }
        }
        SettingsMsg::ClaudeSignIn(id) => {
            if let Err(e) = claude_accounts::connect_claude_account(app.clone(), id) {
                err_out = Some(e);
            }
        }
        SettingsMsg::AddClaudeAccount => {
            if let Err(e) = claude_accounts::add_claude_account(app.clone()) {
                err_out = Some(e);
            }
        }
        SettingsMsg::ToggleHooks(on) => {
            let r = if on { hooks_install::install() } else { hooks_install::uninstall() };
            match r {
                Ok(_) => toast_out = Some("Saved".into()),
                Err(e) => err_out = Some(e),
            }
        }
        SettingsMsg::SetAgLimit(limit) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.antigravity_limit = limit;
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetAgModel(model) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.antigravity_model = model;
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetTaskbarMode(mode) => {
            if let Err(e) = taskbar::set_taskbar_options(app.clone(), mode) {
                err_out = Some(e);
            } else {
                toast_out = Some("Saved".into());
            }
        }
        SettingsMsg::SetShowMode(mode) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.notch_visible = mode != "hide";
                if c.notch_visible {
                    c.notch_on_hover = mode == "hover";
                }
                if !c.notch_visible {
                    c.tray_visible = true;
                }
                config::save(&c);
            }
            core::apply_visibility(app);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::ToggleFoldFullscreen(on) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.fold_for_full_screen = on;
                config::save(&c);
            }
            crate::fullscreen::tell(app);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::ToggleAdaptivePill(on) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.adaptive_pill = on;
                config::save(&c);
            }
            crate::backdrop::set_enabled(app, on);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetScale(scale) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.scale = config::snap_scale(scale);
                config::save(&c);
            }
            core::place_notch(app);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetTheme(theme) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.theme = config::theme_or_system(&theme);
                config::save(&c);
            }
            app.send_ui(core::UiEvent::ApplyTheme);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetWeeklyRing(placement) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.weekly_ring = config::weekly_ring_or_off(&placement);
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::ToggleWeeklyDashed(on) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.weekly_ring_dashed = on;
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::ToggleRemaining(on) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.notch_shows_remaining = on;
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::TogglePace(on) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.show_usage_pace = on;
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetEdge(edge) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.notch_edge = config::edge_or_right(&edge);
                config::save(&c);
            }
            core::place_notch(app);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetMonitor(id) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.notch_monitor = if id.is_empty() { None } else { Some(id) };
                config::save(&c);
            }
            core::place_notch(app);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::Recentre => {
            core::reset_bar(app);
            toast_out = Some("Notch centred".into());
        }
        SettingsMsg::SetColorTransition(style) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.color_transition = config::color_transition_or_step(&style);
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::SetWatchLimit(v) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.watch_limit = config::clamp_watch_limit(v as f64 / 100.0, c.critical_limit);
            config::save(&c);
        }
        SettingsMsg::SetCriticalLimit(v) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.critical_limit = config::clamp_critical_limit(v as f64 / 100.0, c.watch_limit);
            config::save(&c);
        }
        SettingsMsg::ResetLimits => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.critical_limit = config::default_critical_limit();
            c.watch_limit = config::default_watch_limit();
            config::save(&c);
            toast_out = Some("Reset".into());
        }
        SettingsMsg::SetTrayVisible(on) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.tray_visible = if c.notch_visible { on } else { true };
                config::save(&c);
            }
            core::apply_visibility(app);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::ToggleResetAlert(on) => {
            {
                let mut c = app.state::<AppState>().cfg.lock().unwrap();
                c.reset_notifications = on;
                config::save(&c);
            }
            if !on {
                reset_alert::disable(app);
            }
            toast_out = Some("Saved".into());
        }
        SettingsMsg::ToggleResetSound(on) => {
            let mut c = app.state::<AppState>().cfg.lock().unwrap();
            c.reset_notification_sound = on;
            config::save(&c);
            toast_out = Some("Saved".into());
        }
        SettingsMsg::PreviewResetAlert => {
            if let Err(e) = reset_alert::preview_reset_alert(app) {
                err_out = Some(e);
            }
        }
        SettingsMsg::OpenDataDir => core::open_data_dir(),
        SettingsMsg::OpenAuthor => settings_window::open_author_page(),
    }

    (tab_out, err_out, toast_out)
}

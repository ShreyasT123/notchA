//! Native Direct2D + DirectComposition renderer for the Codenotch desktop overlay.
//! Provides hardware-accelerated rendering, 100% fluid frame pacing via DWM,
//! and organic liquid gooey geometry for the orb and moving dots without overlapping seams.
//! Enforces SLEEK DARK MODE ONLY with seamless concave fillets (no diagonal partitions).
#![cfg(windows)]

use std::{collections::HashMap, f32::consts::PI, marker::PhantomData, rc::Rc};
use windows::{
    core::{w, Error, Interface, Result},
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::{E_INVALIDARG, E_OUTOFMEMORY, E_UNEXPECTED, HMODULE, HWND, POINT},
        Graphics::{
            Direct2D::{
                Common::{
                    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BEZIER_SEGMENT, D2D1_COLOR_F,
                    D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED,
                    D2D1_FIGURE_END_OPEN, D2D_POINT_2F, D2D_RECT_F, D2D_SIZE_F,
                },
                D2D1CreateDevice, ID2D1Device, ID2D1DeviceContext, ID2D1DeviceContext5, ID2D1Factory,
                ID2D1Image, ID2D1Layer, ID2D1PathGeometry, ID2D1SolidColorBrush, ID2D1StrokeStyle,
                ID2D1SvgDocument, D2D1_ANTIALIAS_MODE_ALIASED, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                D2D1_ARC_SEGMENT, D2D1_ARC_SIZE_SMALL, D2D1_CAP_STYLE_FLAT,
                D2D1_CAP_STYLE_ROUND, D2D1_DASH_STYLE_SOLID, D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
                D2D1_DRAW_TEXT_OPTIONS_CLIP, D2D1_ELLIPSE, D2D1_LAYER_PARAMETERS1,
                D2D1_LINE_JOIN_ROUND, D2D1_QUADRATIC_BEZIER_SEGMENT, D2D1_ROUNDED_RECT,
                D2D1_STROKE_STYLE_PROPERTIES, D2D1_SWEEP_DIRECTION_CLOCKWISE,
                D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE, D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
            },
            Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP},
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
            },
            DirectComposition::{
                DCompositionCreateDevice, IDCompositionDevice, IDCompositionSurface,
                IDCompositionTarget, IDCompositionVisual,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteTypography,
                DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FEATURE,
                DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES, DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_BOLD,
                DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_WEIGHT_SEMI_BOLD,
                DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_RANGE,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM},
                IDXGIAdapter, IDXGIDevice, IDXGISurface,
            },
        },
        UI::Shell::SHCreateMemStream,
    },
};

const TAU: f32 = std::f32::consts::TAU;
const FADED: f32 = 0.55;
const PRESSED: f32 = 0.93;
/// Circle approximation constant kappa = 4/3 * (sqrt(2) - 1)
const KAPPA: f32 = 0.55228475;

#[derive(Clone, Copy, Debug)]
pub struct D2dPalette {
    pub pill: D2D1_COLOR_F,
    pub edge: D2D1_COLOR_F,
    pub rest_edge: D2D1_COLOR_F,
    pub card: D2D1_COLOR_F,
    pub card_line: D2D1_COLOR_F,
    pub card_rule: D2D1_COLOR_F,
    pub hole: D2D1_COLOR_F,
    pub track: D2D1_COLOR_F,
    pub ample: D2D1_COLOR_F,
    pub watch: D2D1_COLOR_F,
    pub crit: D2D1_COLOR_F,
    pub ink: D2D1_COLOR_F,
    pub ink_2: D2D1_COLOR_F,
    pub ink_dim: D2D1_COLOR_F,
}

/// Enforce Sleek Dark Mode Only forever.
pub fn get_palette(_dark: bool) -> D2dPalette {
    D2dPalette {
        pill: rgb(0x000000),
        edge: rgb(0x2e2e2e),
        rest_edge: rgb(0x5c5c5c),
        card: rgb(0x0a0a0a),
        card_line: rgb(0x242424),
        card_rule: rgb(0x1e1e1e),
        hole: rgba(1.0, 1.0, 1.0, 42.0 / 255.0),
        track: rgba(1.0, 1.0, 1.0, 48.0 / 255.0),
        ample: rgb(0x00ff88),
        watch: rgb(0xf2ff00),
        crit: rgb(0xff3f00),
        ink: rgb(0xffffff),
        ink_2: rgb(0xe8e8ea),
        ink_dim: rgb(0x808080),
    }
}

pub struct CellRenderData {
    pub id: String,
    pub provider: String,
    pub headline: Option<f32>,
    pub weekly: Option<f32>,
    pub weekly_dashed: bool,
    pub weekly_inside: bool,
    pub text: String,
    pub stale: bool,
    pub spent: bool,
    pub work_state: String,
    pub press: f32,
    pub turn_t: f32,
}

pub struct NotchFrameData {
    pub edge: String,
    pub scale: f32,
    pub fold_t: f32,
    pub pill_rect: D2D_RECT_F,
    pub rest_rect: D2D_RECT_F,
    pub cells: Vec<CellRenderData>,
    pub ring_centers: Vec<D2D_POINT_2F>,
    pub orb_center: D2D_POINT_2F,
    pub orb_hover: f32,
    pub grip_out: f32,
    pub grip_center: D2D_POINT_2F,
    pub card_open: bool,
    pub card_t: f32,
    pub card_rect: Option<D2D_RECT_F>,
    pub tail_rect: Option<D2D_RECT_F>,
    pub card_headline: String,
    pub card_subtitle: String,
    pub card_body: Vec<String>,
    pub card_used: Option<f32>,
    pub seconds: f32,
    pub watch_limit: f64,
    pub critical_limit: f64,
    pub shows_remaining: bool,
}

pub struct NotchRenderer {
    context: ID2D1DeviceContext,
    svg_context: ID2D1DeviceContext5,
    factory: ID2D1Factory,
    logos: HashMap<&'static str, [ID2D1SvgDocument; 2]>,
    brush: ID2D1SolidColorBrush,
    round: ID2D1StrokeStyle,
    flat: ID2D1StrokeStyle,
    write_factory: IDWriteFactory,
    tabular: IDWriteTypography,
    formats: HashMap<(i32, u32), IDWriteTextFormat>,
    surface: IDCompositionSurface,
    target: IDCompositionTarget,
    visual: IDCompositionVisual,
    composition: IDCompositionDevice,
    _d2d_device: ID2D1Device,
    _dxgi_device: IDXGIDevice,
    _d3d_device: ID3D11Device,
    width: u32,
    height: u32,
    driver: &'static str,
    _thread_bound: PhantomData<Rc<()>>,
}

impl NotchRenderer {
    pub fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        validate_size(width, height)?;
        Self::with_driver(hwnd, width, height, D3D_DRIVER_TYPE_HARDWARE, "hardware").or_else(
            |hardware_error| {
                Self::with_driver(hwnd, width, height, D3D_DRIVER_TYPE_WARP, "warp").map_err(
                    |warp_error| {
                        Error::new(
                            warp_error.code(),
                            format!(
                                "Notch D2D failed: hw=0x{:08x}, warp=0x{:08x}",
                                hardware_error.code().0 as u32,
                                warp_error.code().0 as u32
                            ),
                        )
                    },
                )
            },
        )
    }

    fn with_driver(
        hwnd: HWND,
        width: u32,
        height: u32,
        kind: D3D_DRIVER_TYPE,
        driver: &'static str,
    ) -> Result<Self> {
        unsafe {
            let mut d3d_device = None;
            D3D11CreateDevice(
                None::<&IDXGIAdapter>,
                kind,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )?;
            let d3d_device =
                d3d_device.ok_or_else(|| Error::new(E_UNEXPECTED, "D3D11 returned no device"))?;
            let dxgi_device: IDXGIDevice = d3d_device.cast()?;
            let d2d_device = D2D1CreateDevice(&dxgi_device, None)?;
            let context = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            context.SetDpi(96.0, 96.0);
            context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            let svg_context: ID2D1DeviceContext5 = context.cast()?;
            let factory = context.GetFactory()?;
            let logos = create_logos(&svg_context)?;
            let brush = context.CreateSolidColorBrush(&rgb(0xffffff), None)?;
            let stroke = |cap| {
                factory.CreateStrokeStyle(
                    &D2D1_STROKE_STYLE_PROPERTIES {
                        startCap: cap,
                        endCap: cap,
                        dashCap: cap,
                        lineJoin: D2D1_LINE_JOIN_ROUND,
                        miterLimit: 10.0,
                        dashStyle: D2D1_DASH_STYLE_SOLID,
                        dashOffset: 0.0,
                    },
                    None,
                )
            };
            let (round, flat) = (stroke(D2D1_CAP_STYLE_ROUND)?, stroke(D2D1_CAP_STYLE_FLAT)?);
            let write_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let tabular = write_factory.CreateTypography()?;
            tabular.AddFontFeature(DWRITE_FONT_FEATURE {
                nameTag: DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES,
                parameter: 1,
            })?;
            let composition: IDCompositionDevice = DCompositionCreateDevice(&dxgi_device)?;
            let target = composition.CreateTargetForHwnd(hwnd, true)?;
            let visual = composition.CreateVisual()?;
            let surface = composition.CreateSurface(
                width,
                height,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                DXGI_ALPHA_MODE_PREMULTIPLIED,
            )?;
            visual.SetContent(&surface)?;
            target.SetRoot(&visual)?;

            Ok(Self {
                context,
                svg_context,
                factory,
                logos,
                brush,
                round,
                flat,
                write_factory,
                tabular,
                formats: HashMap::new(),
                surface,
                target,
                visual,
                composition,
                _d2d_device: d2d_device,
                _dxgi_device: dxgi_device,
                _d3d_device: d3d_device,
                width,
                height,
                driver,
                _thread_bound: PhantomData,
            })
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        validate_size(width, height)?;
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        unsafe {
            let surface = self.composition.CreateSurface(
                width,
                height,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                DXGI_ALPHA_MODE_PREMULTIPLIED,
            )?;
            self.visual.SetContent(&surface)?;
            self.surface = surface;
            self.width = width;
            self.height = height;
        }
        Ok(())
    }

    pub fn draw(&mut self, data: &NotchFrameData) -> Result<()> {
        let pal = get_palette(true); // Always Dark Mode
        let fold = data.fold_t.clamp(0.0, 1.0);
        let reveal = 1.0 - fold;

        unsafe {
            let mut offset = POINT::default();
            let dxgi_surface: IDXGISurface = self.surface.BeginDraw(None, &mut offset)?;
            let draw_result = (|| -> Result<()> {
                let target_bitmap = self.context.CreateBitmapFromDxgiSurface(&dxgi_surface, None)?;
                self.context.SetTarget(&target_bitmap);
                self.context.BeginDraw();
                self.context.Clear(Some(&rgba(0.0, 0.0, 0.0, 0.0)));

                let s = data.scale;
                self.context.SetTransform(&Matrix3x2 {
                    M11: s,
                    M12: 0.0,
                    M21: 0.0,
                    M22: s,
                    M31: offset.x as f32,
                    M32: offset.y as f32,
                });

                if fold >= 0.999 {
                    self.draw_folded_rest(data.rest_rect, &data.edge, &pal)?;
                } else {
                    let morph_rect = lerp_rect(data.pill_rect, data.rest_rect, fold);
                    let (fill_path, stroke_path) = self.build_pill_paths(&data.edge, morph_rect, fold)?;

                    // Fill pill with pitch black
                    self.brush.SetColor(&pal.pill);
                    self.brush.SetOpacity(1.0);
                    self.context.FillGeometry(&fill_path, &self.brush, None);

                    // Stroke outer boundary (omitting screen edge to prevent bezel seam)
                    self.brush.SetColor(&pal.edge);
                    self.context.DrawGeometry(&stroke_path, &self.brush, 1.0, &self.flat);

                    // Draw rings / cells
                    if reveal > 0.001 {
                        self.draw_cells(data, &pal, reveal)?;
                    }

                    // Settings Orb & Fluid MoveGrip
                    self.draw_orb_and_grip(data, &pal, reveal)?;

                    // Gooey Toast Card & Tail Glue
                    if data.card_t > 0.001 {
                        self.draw_card_and_tail(data, &pal)?;
                    }
                }

                self.context.EndDraw(None, None)
            })();

            self.context.SetTarget(None::<&ID2D1Image>);
            let surface_result = self.surface.EndDraw();
            draw_result.and(surface_result)?;
            self.composition.Commit()
        }
    }

    unsafe fn draw_folded_rest(&self, r: D2D_RECT_F, edge: &str, pal: &D2dPalette) -> Result<()> {
        let radius: (f32, f32, f32, f32) = match edge {
            "left" => (0.0, 6.0, 6.0, 0.0),
            "top" => (0.0, 0.0, 6.0, 6.0),
            "bottom" => (6.0, 6.0, 0.0, 0.0),
            _ => (6.0, 0.0, 0.0, 6.0),
        };
        let rounded = D2D1_ROUNDED_RECT {
            rect: r,
            radiusX: radius.0.max(radius.1),
            radiusY: radius.2.max(radius.3),
        };
        self.brush.SetColor(&pal.pill);
        self.brush.SetOpacity(1.0);
        self.context.FillRoundedRectangle(&rounded, &self.brush);
        self.brush.SetColor(&pal.rest_edge);
        self.context.DrawRoundedRectangle(&rounded, &self.brush, 1.0, &self.flat);
        Ok(())
    }

    /// Builds mathematically exact cubic Bézier concave fillets without diagonal artifacts.
    /// Returns (fill_path, open_stroke_path).
    unsafe fn build_pill_paths(
        &self,
        edge: &str,
        r: D2D_RECT_F,
        fold: f32,
    ) -> Result<(ID2D1PathGeometry, ID2D1PathGeometry)> {
        let f = lerp(38.7, 6.0, fold);
        let rad = lerp(20.0, 6.0, fold);

        let fill_path = self.factory.CreatePathGeometry()?;
        let stroke_path = self.factory.CreatePathGeometry()?;

        let fill_sink = fill_path.Open()?;
        let stroke_sink = stroke_path.Open()?;

        let (x0, x1, y0, y1) = (r.left, r.right, r.top, r.bottom);

        match edge {
            "left" => {
                let start_pt = D2D_POINT_2F { x: x0, y: y0 - f };
                fill_sink.BeginFigure(start_pt, D2D1_FIGURE_BEGIN_FILLED);
                stroke_sink.BeginFigure(start_pt, D2D1_FIGURE_BEGIN_HOLLOW);

                // Top fillet concave
                let seg1 = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0, y: y0 - f + f * KAPPA },
                    point2: D2D_POINT_2F { x: x0 + f - f * KAPPA, y: y0 },
                    point3: D2D_POINT_2F { x: x0 + f, y: y0 },
                };
                fill_sink.AddBezier(&seg1);
                stroke_sink.AddBezier(&seg1);

                // Top edge & right corners
                let line_top = D2D_POINT_2F { x: x1 - rad, y: y0 };
                fill_sink.AddLine(line_top);
                stroke_sink.AddLine(line_top);

                let cr1 = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x1 - rad + rad * KAPPA, y: y0 },
                    point2: D2D_POINT_2F { x: x1, y: y0 + rad - rad * KAPPA },
                    point3: D2D_POINT_2F { x: x1, y: y0 + rad },
                };
                fill_sink.AddBezier(&cr1);
                stroke_sink.AddBezier(&cr1);

                let line_right = D2D_POINT_2F { x: x1, y: y1 - rad };
                fill_sink.AddLine(line_right);
                stroke_sink.AddLine(line_right);

                let cr2 = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x1, y: y1 - rad + rad * KAPPA },
                    point2: D2D_POINT_2F { x: x1 - rad + rad * KAPPA, y: y1 },
                    point3: D2D_POINT_2F { x: x1 - rad, y: y1 },
                };
                fill_sink.AddBezier(&cr2);
                stroke_sink.AddBezier(&cr2);

                let line_bottom = D2D_POINT_2F { x: x0 + f, y: y1 };
                fill_sink.AddLine(line_bottom);
                stroke_sink.AddLine(line_bottom);

                // Bottom fillet concave
                let seg2 = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0 + f - f * KAPPA, y: y1 },
                    point2: D2D_POINT_2F { x: x0, y: y1 + f - f * KAPPA },
                    point3: D2D_POINT_2F { x: x0, y: y1 + f },
                };
                fill_sink.AddBezier(&seg2);
                stroke_sink.AddBezier(&seg2);

                stroke_sink.EndFigure(D2D1_FIGURE_END_OPEN);

                fill_sink.AddLine(start_pt);
                fill_sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            }
            _ => {
                // "right"
                let start_pt = D2D_POINT_2F { x: x1, y: y0 - f };
                fill_sink.BeginFigure(start_pt, D2D1_FIGURE_BEGIN_FILLED);
                stroke_sink.BeginFigure(start_pt, D2D1_FIGURE_BEGIN_HOLLOW);

                // Top fillet: smooth concave curve into the pill
                let top_fillet = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x1, y: y0 - f + f * KAPPA },
                    point2: D2D_POINT_2F { x: x1 - f + f * KAPPA, y: y0 },
                    point3: D2D_POINT_2F { x: x1 - f, y: y0 },
                };
                fill_sink.AddBezier(&top_fillet);
                stroke_sink.AddBezier(&top_fillet);

                // Top pill edge
                let line_top = D2D_POINT_2F { x: x0 + rad, y: y0 };
                fill_sink.AddLine(line_top);
                stroke_sink.AddLine(line_top);

                // Top-left corner
                let tl_corner = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0 + rad - rad * KAPPA, y: y0 },
                    point2: D2D_POINT_2F { x: x0, y: y0 + rad - rad * KAPPA },
                    point3: D2D_POINT_2F { x: x0, y: y0 + rad },
                };
                fill_sink.AddBezier(&tl_corner);
                stroke_sink.AddBezier(&tl_corner);

                // Left outer pill vertical edge
                let line_left = D2D_POINT_2F { x: x0, y: y1 - rad };
                fill_sink.AddLine(line_left);
                stroke_sink.AddLine(line_left);

                // Bottom-left corner
                let bl_corner = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0, y: y1 - rad + rad * KAPPA },
                    point2: D2D_POINT_2F { x: x0 + rad - rad * KAPPA, y: y1 },
                    point3: D2D_POINT_2F { x: x0 + rad, y: y1 },
                };
                fill_sink.AddBezier(&bl_corner);
                stroke_sink.AddBezier(&bl_corner);

                // Bottom pill edge
                let line_bottom = D2D_POINT_2F { x: x1 - f, y: y1 };
                fill_sink.AddLine(line_bottom);
                stroke_sink.AddLine(line_bottom);

                // Bottom fillet: smooth concave curve to the screen edge
                let bottom_fillet = D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x1 - f + f * KAPPA, y: y1 },
                    point2: D2D_POINT_2F { x: x1, y: y1 + f - f * KAPPA },
                    point3: D2D_POINT_2F { x: x1, y: y1 + f },
                };
                fill_sink.AddBezier(&bottom_fillet);
                stroke_sink.AddBezier(&bottom_fillet);

                // Stroke ends without drawing on the screen edge bezel
                stroke_sink.EndFigure(D2D1_FIGURE_END_OPEN);

                // Fill closes along the screen edge
                fill_sink.AddLine(start_pt);
                fill_sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            }
        }

        fill_sink.Close()?;
        stroke_sink.Close()?;

        Ok((fill_path, stroke_path))
    }

    unsafe fn draw_cells(
        &mut self,
        data: &NotchFrameData,
        pal: &D2dPalette,
        reveal: f32,
    ) -> Result<()> {
        let k = 44.0 / 56.0;
        let pct_format = self.format(DWRITE_FONT_WEIGHT_SEMI_BOLD, 13.5)?;
        let letter_format = self.format(DWRITE_FONT_WEIGHT_BOLD, 18.0)?;

        for (i, cell) in data.cells.iter().enumerate() {
            if i >= data.ring_centers.len() {
                break;
            }
            let stagger = (data.fold_t * 1.6 - i as f32 * 0.12).clamp(0.0, 1.0);
            let cell_reveal = (reveal - stagger * reveal).max(0.0);
            if cell_reveal <= 0.001 {
                continue;
            }

            let rc = data.ring_centers[i];
            let s = (1.0 - (1.0 - PRESSED) * cell.press.clamp(0.0, 1.0)) * k;
            let center = rc;

            let fade = if cell.stale { FADED } else { 1.0 } * cell_reveal;

            // Hole & Track
            self.brush.SetColor(&pal.hole);
            self.brush.SetOpacity(fade);
            self.context.FillEllipse(&circle(center, 22.0 * s), &self.brush);
            self.brush.SetColor(&pal.track);
            self.context.DrawEllipse(&circle(center, 25.0 * s), &self.brush, 5.0 * s, &self.round);

            // Weekly ring
            if let Some(week) = cell.weekly {
                let r = if cell.weekly_inside { 16.0 } else { 31.0 } * s;
                let fraction = if data.shows_remaining { 1.0 - week } else { week }.clamp(0.0, 1.0);
                self.brush.SetColor(&pal.track);
                self.brush.SetOpacity(0.7 * fade);
                self.context.DrawEllipse(&circle(center, r), &self.brush, 2.4 * s, &self.flat);
                let col = tone_color(week, data.watch_limit, data.critical_limit, pal);
                self.brush.SetColor(&col);
                self.brush.SetOpacity(0.85 * fade);
                self.arc(center, r, 0.0, fraction, 2.4 * s, &self.round)?;
            }

            // Headline meter arc
            if let Some(used) = cell.headline {
                let col = tone_color(used, data.watch_limit, data.critical_limit, pal);
                self.brush.SetColor(&col);
                self.brush.SetOpacity(fade);
                let fraction = if data.shows_remaining { 1.0 - used } else { used }.clamp(0.0, 1.0);
                self.arc(center, 25.0 * s, cell.turn_t, fraction, 5.0 * s, &self.round)?;
            }

            // Activity arc
            match cell.work_state.as_str() {
                "busy" => {
                    let step = ((data.seconds / 1.2).rem_euclid(1.0) * 12.0).floor() / 12.0;
                    self.brush.SetColor(&pal.ink);
                    self.brush.SetOpacity(cell_reveal);
                    self.arc(center, 19.0 * s, step, 0.28, 2.5 * s, &self.round)?;
                }
                "waiting" => {
                    self.brush.SetColor(&pal.watch);
                    self.brush.SetOpacity(pulse_step(data.seconds) * cell_reveal);
                    self.context.DrawEllipse(&circle(center, 19.0 * s), &self.brush, 2.5 * s, &self.flat);
                }
                _ => {}
            }

            // Provider glyph
            if let Some(documents) = self.logos.get(cell.provider.as_str()) {
                let size = 26.0 * s;
                let mut prev = Matrix3x2::default();
                self.context.GetTransform(&mut prev);
                let glyph_matrix = Matrix3x2 {
                    M11: s * size / 24.0,
                    M12: 0.0,
                    M21: 0.0,
                    M22: s * size / 24.0,
                    M31: prev.M31 + center.x - s * size / 2.0,
                    M32: prev.M32 + center.y - s * size / 2.0,
                };
                self.context.SetTransform(&glyph_matrix);
                self.svg_context.DrawSvgDocument(&documents[0]); // Dark mode SVG
                self.context.SetTransform(&prev);
            } else {
                self.brush.SetColor(&pal.ink_2);
                self.brush.SetOpacity(fade);
                let text: Vec<u16> = letter(&cell.provider).encode_utf16().collect();
                let half = 22.0 * s;
                let text_rect = D2D_RECT_F {
                    left: center.x - half,
                    top: center.y - half,
                    right: center.x + half,
                    bottom: center.y + half,
                };
                self.context.DrawText(
                    &text,
                    &letter_format,
                    &text_rect,
                    &self.brush,
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }

            // Percentage text under ring
            if !cell.text.is_empty() {
                let wide: Vec<u16> = cell.text.encode_utf16().collect();
                let layout = self.write_factory.CreateTextLayout(
                    &wide,
                    &pct_format,
                    60.0,
                    20.0,
                )?;
                layout.SetTypography(
                    &self.tabular,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: wide.len() as u32,
                    },
                )?;
                self.brush.SetColor(&pal.ink);
                self.brush.SetOpacity(cell_reveal);
                self.context.DrawTextLayout(
                    D2D_POINT_2F {
                        x: center.x - 30.0,
                        y: center.y + 24.0,
                    },
                    &layout,
                    &self.brush,
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                );
            }
        }
        Ok(())
    }

    /// Draws the settings orb and the fluid MoveGrip liquid droplet with fluid pop-up
    unsafe fn draw_orb_and_grip(
        &self,
        data: &NotchFrameData,
        pal: &D2dPalette,
        reveal: f32,
    ) -> Result<()> {
        let c = data.orb_center;
        let hover = data.orb_hover.clamp(0.0, 1.0);
        let out = data.grip_out.clamp(0.0, 1.0);

        // 1. Resting quarter arc when orb is not hovered
        if hover < 0.999 {
            let rest_a = (1.0 - hover) * reveal;
            let scale_rest = 1.0 - 0.14 * hover; // smooth shrink
            let start_turns = match data.edge.as_str() {
                "left" | "top" => 0.5,
                "bottom" => 0.25,
                _ => 0.75,
            };
            self.brush.SetColor(&pal.edge);
            self.brush.SetOpacity(rest_a);
            self.arc(c, 28.5 * scale_rest, start_turns, 0.25, 8.8, &self.round)?;
            self.brush.SetColor(&pal.pill);
            self.arc(c, 28.5 * scale_rest, start_turns, 0.25, 6.8, &self.round)?;
        }

        // 2. Popping orb disc + fluid neck + grip dots
        if hover > 0.001 {
            let bounce_r = 23.3 * hover;
            let disc_geom = self.circle_geometry(c, bounce_r)?;

            // Fill & stroke the popping disc
            self.brush.SetColor(&pal.pill);
            self.brush.SetOpacity(hover * reveal);
            self.context.FillGeometry(&disc_geom, &self.brush, None);
            self.brush.SetColor(&pal.edge);
            self.context.DrawGeometry(&disc_geom, &self.brush, 1.0, &self.flat);

            // Draw spinning gear icon with fluid bounce
            self.draw_gear_icon(c, hover, data.seconds)?;

            if out > 0.01 {
                // FLUID GOO: build fused liquid neck geometry
                let gc = data.grip_center;
                let neck_geom = self.build_neck_geometry(c, gc, bounce_r * 0.9, 8.5 * out, out)?;
                let dots = self.get_grip_dots(gc, 9.9 * out, matches!(data.edge.as_str(), "left" | "right"));
                let dot_r = 3.05 * (1.0 + 0.3 * hover);

                // Fill all liquid components with pill color
                self.brush.SetColor(&pal.pill);
                self.brush.SetOpacity(hover * reveal);
                self.context.FillGeometry(&neck_geom, &self.brush, None);
                for dot in &dots {
                    self.context.FillEllipse(&circle(*dot, dot_r), &self.brush);
                }

                // Stroke unified outer contours
                self.brush.SetColor(&pal.edge);
                self.brush.SetOpacity(hover * reveal);
                self.context.DrawGeometry(&neck_geom, &self.brush, 0.9, &self.flat);
                for dot in &dots {
                    self.context.DrawEllipse(&circle(*dot, dot_r), &self.brush, 0.9, &self.flat);
                }
            }
        }
        Ok(())
    }

    /// Draws the settings gear icon rotating and scaling with fluid bounce
    unsafe fn draw_gear_icon(&self, center: D2D_POINT_2F, hover: f32, seconds: f32) -> Result<()> {
        let mut prev = Matrix3x2::default();
        self.context.GetTransform(&mut prev);
        let scale = 0.5 + 0.5 * hover;
        let angle = (hover * 360.0 - 60.0) * (PI / 180.0);

        let transform = Matrix3x2 {
            M11: scale * angle.cos(),
            M12: scale * angle.sin(),
            M21: -scale * angle.sin(),
            M22: scale * angle.cos(),
            M31: prev.M31 + center.x,
            M32: prev.M32 + center.y,
        };
        self.context.SetTransform(&transform);

        // Gear icon (21px box centered)
        self.brush.SetColor(&rgb(0xe8e8ea));
        self.brush.SetOpacity(hover);

        // Core hub circle
        self.context.DrawEllipse(&circle(D2D_POINT_2F { x: 0.0, y: 0.0 }, 4.1), &self.brush, 1.6, &self.round);

        // 8 cogs
        for i in 0..8 {
            let cog_angle = i as f32 * (TAU / 8.0);
            let p1 = D2D_POINT_2F { x: cog_angle.cos() * 6.5, y: cog_angle.sin() * 6.5 };
            let p2 = D2D_POINT_2F { x: cog_angle.cos() * 9.5, y: cog_angle.sin() * 9.5 };
            self.context.DrawLine(p1, p2, &self.brush, 2.4, &self.round);
        }

        self.context.SetTransform(&prev);
        Ok(())
    }

    /// Constructs the organic waist-pinched neck strand connecting two circular bodies
    unsafe fn build_neck_geometry(
        &self,
        from: D2D_POINT_2F,
        to: D2D_POINT_2F,
        r_from: f32,
        r_to: f32,
        pull: f32,
    ) -> Result<ID2D1PathGeometry> {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let nx = -dy / len;
        let ny = dx / len;

        let a1 = D2D_POINT_2F { x: from.x + nx * r_from, y: from.y + ny * r_from };
        let a2 = D2D_POINT_2F { x: from.x - nx * r_from, y: from.y - ny * r_from };
        let b1 = D2D_POINT_2F { x: to.x + nx * r_to, y: to.y + ny * r_to };
        let b2 = D2D_POINT_2F { x: to.x - nx * r_to, y: to.y - ny * r_to };

        let waist = ((r_from + r_to) * 0.5 * pull).max(0.4);
        let mid = D2D_POINT_2F { x: (from.x + to.x) * 0.5, y: (from.y + to.y) * 0.5 };
        let m1 = D2D_POINT_2F { x: mid.x + nx * waist, y: mid.y + ny * waist };
        let m2 = D2D_POINT_2F { x: mid.x - nx * waist, y: mid.y - ny * waist };

        let path = self.factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(a1, D2D1_FIGURE_BEGIN_FILLED);
        sink.AddQuadraticBezier(&D2D1_QUADRATIC_BEZIER_SEGMENT { point1: m1, point2: b1 });
        sink.AddLine(b2);
        sink.AddQuadraticBezier(&D2D1_QUADRATIC_BEZIER_SEGMENT { point1: m2, point2: a2 });
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        Ok(path)
    }

    fn get_grip_dots(&self, center: D2D_POINT_2F, reach: f32, vertical: bool) -> [D2D_POINT_2F; 6] {
        let mut out = [center; 6];
        let mut i = 0;
        for line in [-0.5f32, 0.5] {
            for row in [-1.0f32, 0.0, 1.0] {
                let (ox, oy) = if vertical {
                    (row * reach, line * reach)
                } else {
                    (line * reach, row * reach)
                };
                out[i] = D2D_POINT_2F { x: center.x + ox, y: center.y + oy };
                i += 1;
            }
        }
        out
    }

    unsafe fn circle_geometry(&self, c: D2D_POINT_2F, r: f32) -> Result<ID2D1PathGeometry> {
        let path = self.factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(D2D_POINT_2F { x: c.x + r, y: c.y }, D2D1_FIGURE_BEGIN_FILLED);
        sink.AddArc(&D2D1_ARC_SEGMENT {
            point: D2D_POINT_2F { x: c.x - r, y: c.y },
            size: D2D_SIZE_F { width: r, height: r },
            rotationAngle: 0.0,
            sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
            arcSize: D2D1_ARC_SIZE_SMALL,
        });
        sink.AddArc(&D2D1_ARC_SEGMENT {
            point: D2D_POINT_2F { x: c.x + r, y: c.y },
            size: D2D_SIZE_F { width: r, height: r },
            rotationAngle: 0.0,
            sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
            arcSize: D2D1_ARC_SIZE_SMALL,
        });
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        Ok(path)
    }

    /// Draws the toast/status card and its trumpet tail "glue" bridge connecting to the notch
    unsafe fn draw_card_and_tail(&mut self, data: &NotchFrameData, pal: &D2dPalette) -> Result<()> {
        let alpha = data.card_t.clamp(0.0, 1.0);

        // Trumpet Tail Wedge (The Gooey Glue Bridge)
        if let Some(tr) = data.tail_rect {
            let tail_path = self.build_tail_path(&data.edge, tr)?;
            self.brush.SetColor(&pal.card);
            self.brush.SetOpacity(alpha);
            self.context.FillGeometry(&tail_path, &self.brush, None);
            self.brush.SetColor(&pal.card_line);
            self.context.DrawGeometry(&tail_path, &self.brush, 1.0, &self.flat);
        }

        // Card body
        if let Some(cr) = data.card_rect {
            let rounded = D2D1_ROUNDED_RECT { rect: cr, radiusX: 16.0, radiusY: 16.0 };
            self.brush.SetColor(&pal.card);
            self.brush.SetOpacity(alpha);
            self.context.FillRoundedRectangle(&rounded, &self.brush);
            self.brush.SetColor(&pal.card_line);
            self.context.DrawRoundedRectangle(&rounded, &self.brush, 1.0, &self.flat);

            // Card Text formatting
            let title_fmt = self.format(DWRITE_FONT_WEIGHT_BOLD, 13.5)?;
            let sub_fmt = self.format(DWRITE_FONT_WEIGHT_REGULAR, 11.0)?;
            let body_fmt = self.format(DWRITE_FONT_WEIGHT_SEMI_BOLD, 11.5)?;

            // Header Title
            let wide_title: Vec<u16> = data.card_headline.encode_utf16().collect();
            let text_rect = D2D_RECT_F {
                left: cr.left + 14.0,
                top: cr.top + 14.0,
                right: cr.right - 14.0,
                bottom: cr.top + 32.0,
            };
            self.brush.SetColor(&pal.ink);
            self.context.DrawText(
                &wide_title,
                &title_fmt,
                &text_rect,
                &self.brush,
                D2D1_DRAW_TEXT_OPTIONS_CLIP,
                DWRITE_MEASURING_MODE_NATURAL,
            );

            // Subtitle
            if !data.card_subtitle.is_empty() {
                let wide_sub: Vec<u16> = data.card_subtitle.encode_utf16().collect();
                let sub_rect = D2D_RECT_F {
                    left: cr.left + 14.0,
                    top: cr.top + 32.0,
                    right: cr.right - 14.0,
                    bottom: cr.top + 48.0,
                };
                self.brush.SetColor(&pal.ink_dim);
                self.context.DrawText(
                    &wide_sub,
                    &sub_fmt,
                    &sub_rect,
                    &self.brush,
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }

            // Quota Progress Bar (Gooey Glue Status)
            if let Some(used) = data.card_used {
                let bar_top = cr.top + 54.0;
                let bar_width = (cr.right - cr.left) - 28.0;
                let track_rect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: cr.left + 14.0,
                        top: bar_top,
                        right: cr.right - 14.0,
                        bottom: bar_top + 6.0,
                    },
                    radiusX: 3.0,
                    radiusY: 3.0,
                };
                self.brush.SetColor(&pal.track);
                self.brush.SetOpacity(1.0);
                self.context.FillRoundedRectangle(&track_rect, &self.brush);

                let fill_frac = if data.shows_remaining { 1.0 - used } else { used }.clamp(0.0, 1.0);
                let fill_rect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: cr.left + 14.0,
                        top: bar_top,
                        right: cr.left + 14.0 + bar_width * fill_frac,
                        bottom: bar_top + 6.0,
                    },
                    radiusX: 3.0,
                    radiusY: 3.0,
                };
                let col = tone_color(used, data.watch_limit, data.critical_limit, pal);
                self.brush.SetColor(&col);
                self.context.FillRoundedRectangle(&fill_rect, &self.brush);
            }

            let mut y = cr.top + 70.0;
            self.brush.SetColor(&pal.ink_dim);
            for line in &data.card_body {
                let wide_line: Vec<u16> = line.encode_utf16().collect();
                let line_rect = D2D_RECT_F {
                    left: cr.left + 14.0,
                    top: y,
                    right: cr.right - 14.0,
                    bottom: y + 18.0,
                };
                self.context.DrawText(
                    &wide_line,
                    &body_fmt,
                    &line_rect,
                    &self.brush,
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
                y += 18.0;
            }
        }
        Ok(())
    }

    /// Constructs the curved trumpet tail wedge matching `clip-path: path('M0 0C0 8.18 16.36 12.43 28.2 16.36C16.36 20.29 0 24.54 0 32.72Z')`
    unsafe fn build_tail_path(&self, edge: &str, r: D2D_RECT_F) -> Result<ID2D1PathGeometry> {
        let (x0, y0, w, h) = (r.left, r.top, r.right - r.left, r.bottom - r.top);
        let path = self.factory.CreatePathGeometry()?;
        let sink = path.Open()?;

        match edge {
            "left" => {
                // Card on the right of the pill, tail points left
                sink.BeginFigure(D2D_POINT_2F { x: x0 + w, y: y0 }, D2D1_FIGURE_BEGIN_FILLED);
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0 + w, y: y0 + h * 0.25 },
                    point2: D2D_POINT_2F { x: x0 + w * 0.42, y: y0 + h * 0.38 },
                    point3: D2D_POINT_2F { x: x0, y: y0 + h * 0.5 },
                });
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0 + w * 0.42, y: y0 + h * 0.62 },
                    point2: D2D_POINT_2F { x: x0 + w, y: y0 + h * 0.75 },
                    point3: D2D_POINT_2F { x: x0 + w, y: y0 + h },
                });
            }
            _ => {
                // "right": Card on the left of the pill, tail points right towards the ring
                sink.BeginFigure(D2D_POINT_2F { x: x0, y: y0 }, D2D1_FIGURE_BEGIN_FILLED);
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0, y: y0 + h * 0.25 },
                    point2: D2D_POINT_2F { x: x0 + w * 0.58, y: y0 + h * 0.38 },
                    point3: D2D_POINT_2F { x: x0 + w, y: y0 + h * 0.5 },
                });
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: D2D_POINT_2F { x: x0 + w * 0.58, y: y0 + h * 0.62 },
                    point2: D2D_POINT_2F { x: x0, y: y0 + h * 0.75 },
                    point3: D2D_POINT_2F { x: x0, y: y0 + h },
                });
            }
        }
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        Ok(path)
    }

    unsafe fn arc(
        &self,
        center: D2D_POINT_2F,
        radius: f32,
        start_turns: f32,
        fraction: f32,
        width: f32,
        style: &ID2D1StrokeStyle,
    ) -> Result<()> {
        if fraction.is_nan() || fraction <= 0.0 || radius.is_nan() || radius <= 0.0 {
            return Ok(());
        }
        if fraction >= 1.0 {
            self.context.DrawEllipse(&circle(center, radius), &self.brush, width, style);
            return Ok(());
        }
        let at = |turns: f32| {
            let angle = (turns - 0.25) * TAU;
            D2D_POINT_2F {
                x: center.x + radius * angle.cos(),
                y: center.y + radius * angle.sin(),
            }
        };
        let path = self.factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(at(start_turns), D2D1_FIGURE_BEGIN_HOLLOW);
        for end in [start_turns + fraction * 0.5, start_turns + fraction] {
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: at(end),
                size: D2D_SIZE_F { width: radius, height: radius },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            });
        }
        sink.EndFigure(D2D1_FIGURE_END_OPEN);
        sink.Close()?;
        self.context.DrawGeometry(&path, &self.brush, width, style);
        Ok(())
    }

    fn format(&mut self, weight: DWRITE_FONT_WEIGHT, size: f32) -> Result<IDWriteTextFormat> {
        let size = if size.is_finite() { size.clamp(1.0, 200.0) } else { 12.0 };
        let key = (weight.0, size.to_bits());
        if let Some(format) = self.formats.get(&key) {
            return Ok(format.clone());
        }
        if self.formats.len() > 16 {
            self.formats.clear();
        }
        let format = unsafe { text_format(&self.write_factory, weight, size)? };
        self.formats.insert(key, format.clone());
        Ok(format)
    }
}

// ---------------- Helper Utilities ----------------

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn lerp_rect(a: D2D_RECT_F, b: D2D_RECT_F, t: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: lerp(a.left, b.left, t),
        top: lerp(a.top, b.top, t),
        right: lerp(a.right, b.right, t),
        bottom: lerp(a.bottom, b.bottom, t),
    }
}

fn circle(c: D2D_POINT_2F, r: f32) -> D2D1_ELLIPSE {
    D2D1_ELLIPSE { point: c, radiusX: r, radiusY: r }
}

fn rgb(hex: u32) -> D2D1_COLOR_F {
    rgba(
        ((hex >> 16) & 0xff) as f32 / 255.0,
        ((hex >> 8) & 0xff) as f32 / 255.0,
        (hex & 0xff) as f32 / 255.0,
        1.0,
    )
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r, g, b, a }
}

fn pulse_step(seconds: f32) -> f32 {
    let phase = (seconds / 1.1).rem_euclid(1.0);
    let half = (phase * 2.0).fract();
    let st = (half * 6.0).floor() / 6.0;
    if phase < 0.5 { 1.0 - 0.75 * st } else { 0.25 + 0.75 * st }
}

fn tone_color(value: f32, watch_limit: f64, critical_limit: f64, pal: &D2dPalette) -> D2D1_COLOR_F {
    let v = value as f64;
    if v >= critical_limit {
        pal.crit
    } else if v >= watch_limit {
        pal.watch
    } else {
        pal.ample
    }
}

fn letter(provider: &str) -> &'static str {
    match provider {
        "claude" => "C",
        "codex" => "Cx",
        "glm" => "Z.ai",
        "deepseek" => "DS",
        "opencode" => "OC",
        "commandcode" => "CC",
        "cursor" => "Cu",
        "grok" => "Gk",
        "copilot" => "Cp",
        "gemini" => "Ag",
        "qoder" => "Q",
        "mistral" => "M",
        _ => "?",
    }
}

fn validate_size(width: u32, height: u32) -> Result<()> {
    if (1..=8192).contains(&width) && (1..=8192).contains(&height) {
        Ok(())
    } else {
        Err(Error::new(E_INVALIDARG, "Invalid width/height"))
    }
}

unsafe fn text_format(
    factory: &IDWriteFactory,
    weight: DWRITE_FONT_WEIGHT,
    size: f32,
) -> Result<IDWriteTextFormat> {
    factory.CreateTextFormat(
        w!("Segoe UI"),
        None,
        weight,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        size,
        w!("en-US"),
    )
}

unsafe fn create_logos(
    context: &ID2D1DeviceContext5,
) -> Result<HashMap<&'static str, [ID2D1SvgDocument; 2]>> {
    let mut map = HashMap::new();
    for (provider, raw) in crate::glyphs::BUILTIN {
        let (dark_doc, light_doc) = (load_svg(context, raw, true)?, load_svg(context, raw, false)?);
        map.insert(provider, [dark_doc, light_doc]);
    }
    Ok(map)
}

unsafe fn load_svg(
    context: &ID2D1DeviceContext5,
    svg_data: &str,
    dark: bool,
) -> Result<ID2D1SvgDocument> {
    let ink = if dark { "#e8e8ea" } else { "#1d1d1f" };
    let recolored = svg_data.replace("currentColor", ink);
    let bytes = recolored.as_bytes();
    let stream = SHCreateMemStream(Some(bytes)).ok_or_else(|| Error::new(E_OUTOFMEMORY, "stream"))?;
    context.CreateSvgDocument(&stream, D2D_SIZE_F { width: 24.0, height: 24.0 })
}

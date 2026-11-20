use std::io::Cursor;

pub const APP_MARK_PNG: &[u8] = include_bytes!("../icons/tray-color.png");

pub fn app_mark() -> Option<tray_icon::Icon> {
    let mut decoder = png::Decoder::new(Cursor::new(APP_MARK_PNG));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let raw = &buf[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => raw.to_vec(),
        png::ColorType::Rgb => raw
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => raw
            .chunks_exact(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
        png::ColorType::Grayscale => raw
            .iter()
            .flat_map(|&g| [g, g, g, 255])
            .collect(),
        _ => return None,
    };

    tray_icon::Icon::from_rgba(rgba, info.width, info.height).ok()
}
use image::{Rgba, RgbaImage};
use rusttype::{point, Font, Scale};

use crate::request::caption::TextAlign;

pub fn parse_hex_color(hex: &str) -> [u8; 4] {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16);
            let g = u8::from_str_radix(&hex[2..4], 16);
            let b = u8::from_str_radix(&hex[4..6], 16);
            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => [r, g, b, 255],
                _ => [255, 255, 255, 255],
            }
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16);
            let g = u8::from_str_radix(&hex[2..4], 16);
            let b = u8::from_str_radix(&hex[4..6], 16);
            let a = u8::from_str_radix(&hex[6..8], 16);
            match (r, g, b, a) {
                (Ok(r), Ok(g), Ok(b), Ok(a)) => [r, g, b, a],
                _ => [255, 255, 255, 255],
            }
        }
        _ => [255, 255, 255, 255],
    }
}

fn measure_line_width(line: &str, font: &Font, scale: Scale) -> f32 {
    let mut width = 0.0f32;
    let mut last_glyph_id = None;
    for c in line.chars() {
        let glyph = font.glyph(c);
        if let Some(last_id) = last_glyph_id {
            width += font.pair_kerning(scale, last_id, glyph.id());
        }
        let scaled = glyph.scaled(scale);
        width += scaled.h_metrics().advance_width;
        last_glyph_id = Some(scaled.id());
    }
    width
}

pub fn rasterize_caption_page(
    lines: &[String],
    font: &Font<'static>,
    size_px: f32,
    line_height_mult: f32,
    align: TextAlign,
    text_color: [u8; 4],
    bg_color: Option<[u8; 4]>,
    padding: f32,
) -> RgbaImage {
    let scale = Scale::uniform(size_px);
    let v_metrics = font.v_metrics(scale);
    let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) * line_height_mult;

    let line_widths: Vec<f32> = lines
        .iter()
        .map(|l| measure_line_width(l, font, scale))
        .collect();

    let max_line_width = line_widths.iter().cloned().fold(0.0f32, f32::max);

    let img_width = (max_line_width + 2.0 * padding).ceil() as u32;
    let img_height = (lines.len() as f32 * line_height + 2.0 * padding).ceil() as u32;

    let img_width = img_width.max(1);
    let img_height = img_height.max(1);

    let mut img = RgbaImage::new(img_width, img_height);

    if let Some(bg) = bg_color {
        for pixel in img.pixels_mut() {
            *pixel = Rgba(bg);
        }
    }

    for (i, line) in lines.iter().enumerate() {
        let lw = line_widths[i];
        let x_offset = match align {
            TextAlign::Left => padding,
            TextAlign::Center => (img_width as f32 - lw) / 2.0,
            TextAlign::Right => img_width as f32 - lw - padding,
        };

        let y_offset = padding + i as f32 * line_height + v_metrics.ascent;

        let mut cursor_x = x_offset;
        let mut last_glyph_id = None;

        for c in line.chars() {
            let glyph = font.glyph(c);
            if let Some(last_id) = last_glyph_id {
                cursor_x += font.pair_kerning(scale, last_id, glyph.id());
            }
            let scaled = glyph.scaled(scale);
            let advance = scaled.h_metrics().advance_width;
            last_glyph_id = Some(scaled.id());

            let positioned = scaled.positioned(point(cursor_x, y_offset));
            if let Some(bb) = positioned.pixel_bounding_box() {
                positioned.draw(|gx, gy, v| {
                    let px = gx as i32 + bb.min.x;
                    let py = gy as i32 + bb.min.y;
                    if px >= 0
                        && py >= 0
                        && (px as u32) < img_width
                        && (py as u32) < img_height
                    {
                        let alpha = (v * text_color[3] as f32) as u8;
                        let pixel = img.get_pixel_mut(px as u32, py as u32);
                        let bg_r = pixel[0] as f32;
                        let bg_g = pixel[1] as f32;
                        let bg_b = pixel[2] as f32;
                        let bg_a = pixel[3] as f32;
                        let src_a = alpha as f32 / 255.0;
                        let inv = 1.0 - src_a;
                        let out_a = src_a + bg_a / 255.0 * inv;
                        if out_a > 0.0 {
                            let out_r =
                                (text_color[0] as f32 * src_a + bg_r * (bg_a / 255.0) * inv) / out_a;
                            let out_g =
                                (text_color[1] as f32 * src_a + bg_g * (bg_a / 255.0) * inv) / out_a;
                            let out_b =
                                (text_color[2] as f32 * src_a + bg_b * (bg_a / 255.0) * inv) / out_a;
                            *pixel = Rgba([
                                out_r as u8,
                                out_g as u8,
                                out_b as u8,
                                (out_a * 255.0) as u8,
                            ]);
                        }
                    }
                });
            }

            cursor_x += advance;
        }
    }

    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_rgb() {
        assert_eq!(parse_hex_color("#FF8800"), [255, 136, 0, 255]);
    }

    #[test]
    fn parse_hex_rgba() {
        assert_eq!(parse_hex_color("#FF880080"), [255, 136, 0, 128]);
    }

    #[test]
    fn parse_hex_invalid() {
        assert_eq!(parse_hex_color("nope"), [255, 255, 255, 255]);
    }

    #[test]
    fn empty_lines_produce_valid_image() {
        let font_data = include_bytes!("../../../../resources/fonts/Roboto-Regular.ttf").to_vec();
        let font = Font::try_from_vec(font_data).unwrap();
        let img = rasterize_caption_page(
            &[],
            &font,
            32.0,
            1.3,
            TextAlign::Center,
            [255, 255, 255, 255],
            None,
            10.0,
        );
        assert!(img.width() >= 1);
        assert!(img.height() >= 1);
    }
}

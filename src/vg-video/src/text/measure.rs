use rusttype::{Font, Scale};
use unicode_segmentation::UnicodeSegmentation;

pub enum WidthMode {
    Pixels { font: Font<'static>, scale: Scale },
    Chars,
}

impl WidthMode {
    pub fn from_font_data(font_data: Vec<u8>, size_px: f32) -> Self {
        let font = Font::try_from_vec(font_data).expect("Failed to load font");
        let scale = Scale::uniform(size_px);
        WidthMode::Pixels { font, scale }
    }

    pub fn chars_mode() -> Self {
        WidthMode::Chars
    }

    pub fn measure(&self, text: &str) -> f32 {
        match self {
            WidthMode::Pixels { font, scale } => {
                let mut width = 0.0f32;
                let mut last_glyph_id = None;
                for c in text.chars() {
                    let glyph = font.glyph(c);
                    if let Some(last_id) = last_glyph_id {
                        width += font.pair_kerning(*scale, last_id, glyph.id());
                    }
                    let scaled = glyph.scaled(*scale);
                    width += scaled.h_metrics().advance_width;
                    last_glyph_id = Some(scaled.id());
                }
                width
            }
            WidthMode::Chars => text.graphemes(true).count() as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chars_mode_ascii() {
        let mode = WidthMode::chars_mode();
        assert_eq!(mode.measure("hello"), 5.0);
    }

    #[test]
    fn chars_mode_empty() {
        let mode = WidthMode::chars_mode();
        assert_eq!(mode.measure(""), 0.0);
    }

    #[test]
    fn chars_mode_grapheme_clusters() {
        let mode = WidthMode::chars_mode();
        assert_eq!(mode.measure("e\u{0301}"), 1.0);
    }

    #[test]
    fn chars_mode_cjk() {
        let mode = WidthMode::chars_mode();
        assert_eq!(mode.measure("你好"), 2.0);
    }
}

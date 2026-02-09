use std::fs;

use image::DynamicImage;
use log::{info, warn};
use rusttype::Font;

use vg_gl::{Quad, Texture, Vertex};

use crate::request::caption::{
    AnchorPosition, CaptionConfig, MaxWidth, TextAlign,
};
use crate::text::measure::WidthMode;
use crate::text::rasterize::{parse_hex_color, rasterize_caption_page};
use crate::text::split::{split_text_into_pages, CaptionPage};

pub struct CaptionOverlayPage {
    pub page: CaptionPage,
    pub texture: Texture,
    pub img_width: f32,
    pub img_height: f32,
}

pub struct CaptionOverlay {
    pub pages: Vec<CaptionOverlayPage>,
    pub anchor: AnchorPosition,
    pub margin_x: f32,
    pub margin_y: f32,
}

impl CaptionOverlay {
    pub fn from_config(
        config: &CaptionConfig,
        clip_length: f32,
        canvas_width: f32,
        _canvas_height: f32,
    ) -> anyhow::Result<Self> {
        let font_data: Vec<u8> = if let Some(ref font_config) = config.font {
            match fs::read(&font_config.src) {
                Ok(data) => {
                    info!("Loaded font from {}", font_config.src);
                    data
                }
                Err(e) => {
                    warn!("Failed to load font from {}: {}, using default", font_config.src, e);
                    include_bytes!("../../../../resources/fonts/Roboto-Regular.ttf").to_vec()
                }
            }
        } else {
            include_bytes!("../../../../resources/fonts/Roboto-Regular.ttf").to_vec()
        };

        let size_px = config
            .font
            .as_ref()
            .map(|f| f.size_px)
            .unwrap_or(36);

        let layout = config.layout.as_ref();

        let (max_width_val, width_mode) = match layout.and_then(|l| l.max_width.as_ref()) {
            Some(MaxWidth::Px { px }) => {
                (*px as f32, WidthMode::from_font_data(font_data.clone(), size_px as f32))
            }
            Some(MaxWidth::Chars { chars }) => {
                (*chars as f32, WidthMode::chars_mode())
            }
            None => {
                (canvas_width * 0.8, WidthMode::from_font_data(font_data.clone(), size_px as f32))
            }
        };

        let max_lines = layout.map(|l| l.max_lines).unwrap_or(2);
        let line_height_mult = layout.map(|l| l.line_height_mult).unwrap_or(1.3);
        let min_page_duration = config
            .timing
            .as_ref()
            .map(|t| t.min_page_duration)
            .unwrap_or(1.0);

        let pages = split_text_into_pages(
            &config.text,
            max_width_val,
            max_lines,
            clip_length,
            min_page_duration,
            &width_mode,
        );

        let style = config.style.as_ref();
        let text_color = parse_hex_color(
            &style
                .map(|s| s.color.clone())
                .unwrap_or_else(|| "#FFFFFF".to_string()),
        );
        let bg_color = style
            .and_then(|s| s.bg_color.as_ref())
            .map(|c| parse_hex_color(c));
        let padding_px = style.map(|s| s.padding_px).unwrap_or(8.0);
        let alignment = layout.map(|l| &l.align);
        let align = match alignment {
            Some(TextAlign::Left) => TextAlign::Left,
            Some(TextAlign::Right) => TextAlign::Right,
            _ => TextAlign::Center,
        };

        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse font data"))?;

        let mut overlay_pages = Vec::with_capacity(pages.len());
        for page in pages {
            let rgba_img = rasterize_caption_page(
                &page.lines,
                &font,
                size_px as f32,
                line_height_mult,
                align,
                text_color,
                bg_color,
                padding_px,
            );

            let img_width = rgba_img.width() as f32;
            let img_height = rgba_img.height() as f32;

            let dynamic_img = DynamicImage::ImageRgba8(rgba_img);
            let texture = Texture::new_without_unit(gl::TEXTURE_2D);
            texture.load_from_image(dynamic_img)?;
            texture.set_filtering(gl::LINEAR);
            texture.set_wrapping(gl::CLAMP_TO_EDGE);

            overlay_pages.push(CaptionOverlayPage {
                page,
                texture,
                img_width,
                img_height,
            });
        }

        let placement = config.placement.as_ref();
        let anchor = match placement {
            Some(p) => match p.anchor {
                AnchorPosition::TopLeft => AnchorPosition::TopLeft,
                AnchorPosition::TopCenter => AnchorPosition::TopCenter,
                AnchorPosition::TopRight => AnchorPosition::TopRight,
                AnchorPosition::CenterLeft => AnchorPosition::CenterLeft,
                AnchorPosition::Center => AnchorPosition::Center,
                AnchorPosition::CenterRight => AnchorPosition::CenterRight,
                AnchorPosition::BottomLeft => AnchorPosition::BottomLeft,
                AnchorPosition::BottomCenter => AnchorPosition::BottomCenter,
                AnchorPosition::BottomRight => AnchorPosition::BottomRight,
            },
            None => AnchorPosition::BottomCenter,
        };
        let margin_x = placement
            .and_then(|p| p.margin_px.as_ref())
            .map(|m| m.x)
            .unwrap_or(40.0);
        let margin_y = placement
            .and_then(|p| p.margin_px.as_ref())
            .map(|m| m.y)
            .unwrap_or(60.0);

        Ok(CaptionOverlay {
            pages: overlay_pages,
            anchor,
            margin_x,
            margin_y,
        })
    }

    pub fn active_page_at(&self, local_time: f32) -> Option<&CaptionOverlayPage> {
        self.pages
            .iter()
            .find(|p| local_time >= p.page.start_time && local_time < p.page.end_time)
    }
}

impl CaptionOverlayPage {
    pub fn quad(
        &self,
        canvas_width: f32,
        canvas_height: f32,
        anchor: &AnchorPosition,
        margin_x: f32,
        margin_y: f32,
    ) -> Quad {
        let (px_x, px_y) = match anchor {
            AnchorPosition::TopLeft => {
                (margin_x, margin_y)
            }
            AnchorPosition::TopCenter => {
                ((canvas_width - self.img_width) / 2.0, margin_y)
            }
            AnchorPosition::TopRight => {
                (canvas_width - self.img_width - margin_x, margin_y)
            }
            AnchorPosition::CenterLeft => {
                (margin_x, (canvas_height - self.img_height) / 2.0)
            }
            AnchorPosition::Center => {
                ((canvas_width - self.img_width) / 2.0, (canvas_height - self.img_height) / 2.0)
            }
            AnchorPosition::CenterRight => {
                (canvas_width - self.img_width - margin_x, (canvas_height - self.img_height) / 2.0)
            }
            AnchorPosition::BottomLeft => {
                (margin_x, canvas_height - self.img_height - margin_y)
            }
            AnchorPosition::BottomCenter => {
                ((canvas_width - self.img_width) / 2.0, canvas_height - self.img_height - margin_y)
            }
            AnchorPosition::BottomRight => {
                (canvas_width - self.img_width - margin_x, canvas_height - self.img_height - margin_y)
            }
        };

        let x0 = px_x / canvas_width * 2.0 - 1.0;
        let y0 = px_y / canvas_height * 2.0 - 1.0;
        let x1 = (px_x + self.img_width) / canvas_width * 2.0 - 1.0;
        let y1 = (px_y + self.img_height) / canvas_height * 2.0 - 1.0;

        Quad([
            Vertex([x0, y0], [0.0, 0.0], 0.0),
            Vertex([x1, y0], [1.0, 0.0], 0.0),
            Vertex([x1, y1], [1.0, 1.0], 0.0),
            Vertex([x0, y1], [0.0, 1.0], 0.0),
        ])
    }
}

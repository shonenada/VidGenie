extern crate image as image_crate;

use serde::Deserialize;

use crate::asset::MediaAsset;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeKind {
    Rectangle,
    Circle,
    Line,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillSpec {
    pub color: String,
    pub opacity: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrokeSpec {
    pub color: String,
    pub width: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RectangleSpec {
    pub width: u32,
    pub height: u32,
    #[serde(rename = "cornerRadius")]
    pub corner_radius: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CircleSpec {
    pub radius: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LineSpec {
    pub length: u32,
    pub thickness: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShapeAssetSpec {
    pub shape: ShapeKind,
    pub fill: Option<FillSpec>,
    pub stroke: Option<StrokeSpec>,
    pub rectangle: Option<RectangleSpec>,
    pub circle: Option<CircleSpec>,
    pub line: Option<LineSpec>,
}

pub struct ShapeAsset {
    pub(crate) spec: ShapeAssetSpec,
    pub(crate) data: image_crate::DynamicImage,
}

impl MediaAsset for ShapeAsset {
    fn load(&mut self) -> anyhow::Result<()> {
        self.data = render_shape_to_image(&self.spec)?;
        Ok(())
    }
}

fn parse_hex_color(hex: &str) -> [u8; 4] {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            [r, g, b, 255]
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            [r, g, b, a]
        }
        _ => [0, 0, 0, 255],
    }
}

pub fn render_shape_to_image(spec: &ShapeAssetSpec) -> anyhow::Result<image_crate::DynamicImage> {
    match spec.shape {
        ShapeKind::Rectangle => {
            let rect = spec
                .rectangle
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("rectangle spec is required for Rectangle shape"))?;
            let w = rect.width;
            let h = rect.height;
            let corner_radius = rect.corner_radius.unwrap_or(0);

            let mut img = image_crate::RgbaImage::new(w, h);

            let fill_rgba = spec
                .fill
                .as_ref()
                .map(|f| {
                    let mut c = parse_hex_color(&f.color);
                    if let Some(opacity) = f.opacity {
                        c[3] = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
                    }
                    c
                });

            let stroke_rgba = spec.stroke.as_ref().map(|s| parse_hex_color(&s.color));
            let stroke_width = spec.stroke.as_ref().map(|s| s.width).unwrap_or(0);

            for y in 0..h {
                for x in 0..w {
                    if corner_radius > 0 {
                        let cr = corner_radius as f64;
                        let corners = [
                            (corner_radius, corner_radius),
                            (w - corner_radius - 1, corner_radius),
                            (corner_radius, h - corner_radius - 1),
                            (w - corner_radius - 1, h - corner_radius - 1),
                        ];
                        let mut in_corner = false;
                        let mut outside_radius = false;
                        for &(cx, cy) in &corners {
                            if (x < corner_radius || x > w - corner_radius - 1)
                                && (y < corner_radius || y > h - corner_radius - 1)
                            {
                                let dx = x as f64 - cx as f64;
                                let dy = y as f64 - cy as f64;
                                let dist = (dx * dx + dy * dy).sqrt();
                                if dist > cr {
                                    outside_radius = true;
                                }
                                in_corner = true;
                            }
                        }
                        if in_corner && outside_radius {
                            img.put_pixel(x, y, image_crate::Rgba([0, 0, 0, 0]));
                            continue;
                        }
                    }

                    let on_stroke = stroke_width > 0
                        && (x < stroke_width
                            || x >= w - stroke_width
                            || y < stroke_width
                            || y >= h - stroke_width);

                    if on_stroke {
                        if let Some(sc) = stroke_rgba {
                            img.put_pixel(x, y, image_crate::Rgba(sc));
                        }
                    } else if let Some(fc) = fill_rgba {
                        img.put_pixel(x, y, image_crate::Rgba(fc));
                    }
                }
            }

            Ok(image_crate::DynamicImage::ImageRgba8(img))
        }
        ShapeKind::Circle => {
            let circle = spec
                .circle
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("circle spec is required for Circle shape"))?;
            let radius = circle.radius;
            let size = radius * 2;
            let mut img = image_crate::RgbaImage::new(size, size);

            let fill_rgba = spec
                .fill
                .as_ref()
                .map(|f| {
                    let mut c = parse_hex_color(&f.color);
                    if let Some(opacity) = f.opacity {
                        c[3] = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
                    }
                    c
                });

            let stroke_rgba = spec.stroke.as_ref().map(|s| parse_hex_color(&s.color));
            let stroke_width = spec.stroke.as_ref().map(|s| s.width).unwrap_or(0);

            let center = radius as f64;
            let r = radius as f64;

            for y in 0..size {
                for x in 0..size {
                    let dx = x as f64 + 0.5 - center;
                    let dy = y as f64 + 0.5 - center;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > r {
                        img.put_pixel(x, y, image_crate::Rgba([0, 0, 0, 0]));
                    } else if stroke_width > 0 && dist > r - stroke_width as f64 {
                        let edge_dist = r - dist;
                        let alpha = edge_dist.clamp(0.0, 1.0);
                        if let Some(sc) = stroke_rgba {
                            let mut c = sc;
                            c[3] = (c[3] as f64 * alpha) as u8;
                            img.put_pixel(x, y, image_crate::Rgba(c));
                        }
                    } else if let Some(fc) = fill_rgba {
                        img.put_pixel(x, y, image_crate::Rgba(fc));
                    }
                }
            }

            Ok(image_crate::DynamicImage::ImageRgba8(img))
        }
        ShapeKind::Line => {
            let line = spec
                .line
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("line spec is required for Line shape"))?;
            let w = line.length;
            let h = line.thickness;
            let mut img = image_crate::RgbaImage::new(w, h);

            let color = if let Some(ref fill) = spec.fill {
                let mut c = parse_hex_color(&fill.color);
                if let Some(opacity) = fill.opacity {
                    c[3] = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
                }
                c
            } else if let Some(ref stroke) = spec.stroke {
                parse_hex_color(&stroke.color)
            } else {
                [0, 0, 0, 255]
            };

            for y in 0..h {
                for x in 0..w {
                    img.put_pixel(x, y, image_crate::Rgba(color));
                }
            }

            Ok(image_crate::DynamicImage::ImageRgba8(img))
        }
    }
}

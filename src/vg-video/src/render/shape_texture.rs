extern crate image as image_crate;

use gl::types::GLenum;
use log::info;

use crate::asset::shape::{render_shape_to_image, ShapeAssetSpec};
use crate::request::transform::{interpolate_keyframes, Keyframe};
use vg_gl::{Indices, Quad, Texture, Transformer, Vertex, VERTEX_PER_QUAD};

pub struct ShapeClipTexture {
    spec: ShapeAssetSpec,
    texture_idx: u32,

    texture: Option<Texture>,
    canvas_width: f32,
    canvas_height: f32,
    shape_width: f32,
    shape_height: f32,
    offset_x: f32,
    offset_y: f32,
    base_scale: f32,
    base_rotate: f32,
    position: String,
    keyframes: Option<Vec<Keyframe>>,
}

impl ShapeClipTexture {
    pub fn new(
        spec: ShapeAssetSpec,
        canvas_width: f32,
        canvas_height: f32,
        idx: u32,
        scale: f32,
        rotate: f32,
        position: &str,
    ) -> Self {
        Self {
            spec,
            texture_idx: idx,
            canvas_width,
            canvas_height,
            base_scale: scale,
            base_rotate: rotate,
            position: position.to_string(),
            keyframes: None,
            texture: None,
            offset_x: 0.0,
            offset_y: 0.0,
            shape_width: 0.0,
            shape_height: 0.0,
        }
    }

    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset_x = x;
        self.offset_y = y;
    }

    pub fn set_keyframes(&mut self, keyframes: Vec<Keyframe>) {
        self.keyframes = Some(keyframes);
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        info!("generating shape texture");
        let data = render_shape_to_image(&self.spec)?;

        self.shape_width = data.width() as f32;
        self.shape_height = data.height() as f32;

        let unit = (gl::TEXTURE0 + self.texture_idx) as GLenum;
        let texture = Texture::new(gl::TEXTURE_2D, unit);
        texture.load_from_image(data)?;
        texture.set_wrapping(gl::REPEAT);
        texture.set_filtering(gl::LINEAR);
        texture.bind_unit();

        self.texture = Some(texture);

        Ok(())
    }

    fn build_transformer(&self, local_time: f32) -> Transformer {
        let mut transformer = Transformer::default();

        if let Some(ref keyframes) = self.keyframes {
            let kf = interpolate_keyframes(keyframes, local_time);
            transformer.set_scale(self.base_scale * kf.scale);
            transformer.set_rotate(self.base_rotate + kf.rotate);
            transformer.set_translation(self.offset_x + kf.x, self.offset_y + kf.y, 0.0);
            transformer.set_skew(kf.skew_x, kf.skew_y);
            transformer.set_flip(kf.flip_x, kf.flip_y);
            transformer.set_opacity(kf.opacity);
        } else {
            transformer.set_scale(self.base_scale);
            transformer.set_rotate(self.base_rotate);
            transformer.set_translation(self.offset_x, self.offset_y, 0.0);
            transformer.set_opacity(1.0);
        }

        transformer
    }

    pub fn quad_at_time(&self, local_time: f32) -> Quad {
        let transformer = self.build_transformer(local_time);
        let idx = self.texture_idx as f32;

        let (pos_offset_x, pos_offset_y) = if self.position == "center" {
            let offset_x = (self.canvas_width - self.shape_width) / 2.0;
            let offset_y = (self.canvas_height - self.shape_height) / 2.0;
            (offset_x, offset_y)
        } else {
            (0.0, 0.0)
        };

        let (x0, y0) = (pos_offset_x, pos_offset_y);
        let (x1, y1) = (pos_offset_x + self.shape_width, pos_offset_y + self.shape_height);

        let mid_x = x0 + (x1 - x0) / 2.0;
        let mid_y = y0 + (y1 - y0) / 2.0;

        let p0 = transformer.apply_similarity(x0, y0, 1.0, mid_x, mid_y, self.canvas_width, self.canvas_height);
        let p1 = transformer.apply_similarity(x1, y0, 1.0, mid_x, mid_y, self.canvas_width, self.canvas_height);
        let p2 = transformer.apply_similarity(x1, y1, 1.0, mid_x, mid_y, self.canvas_width, self.canvas_height);
        let p3 = transformer.apply_similarity(x0, y1, 1.0, mid_x, mid_y, self.canvas_width, self.canvas_height);

        Quad([
            Vertex([p0.0, p0.1], [0.0, 0.0], idx),
            Vertex([p1.0, p1.1], [1.0, 0.0], idx),
            Vertex([p2.0, p2.1], [1.0, 1.0], idx),
            Vertex([p3.0, p3.1], [0.0, 1.0], idx),
        ])
    }

    pub fn opacity_at_time(&self, local_time: f32) -> f32 {
        let transformer = self.build_transformer(local_time);
        transformer.get_opacity()
    }

    pub fn indices(&self) -> Indices {
        let idx = self.texture_idx as usize;
        let offset = (idx * VERTEX_PER_QUAD) as i32;
        Indices([
            offset,
            offset + 1,
            offset + 2,
            offset + 2,
            offset + 3,
            offset,
        ])
    }

    pub fn texture(&self) -> Option<Texture> {
        self.texture.clone()
    }
}

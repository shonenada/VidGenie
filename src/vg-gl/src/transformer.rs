use std::f32::consts::PI;

use glm::Mat4;
use nalgebra::Point3;

#[derive(Clone)]
struct Scale {
    scale_x: f32,
    scale_y: f32,
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

#[derive(Default, Clone)]
struct Translation {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, Clone)]
struct Rotation {
    angle: f32,
}

#[derive(Default, Clone)]
struct Skew {
    x: f32,
    y: f32,
}

#[derive(Default, Clone)]
struct Flip {
    x: bool,
    y: bool,
}

#[derive(Default, Clone)]
pub struct Transformer {
    scale: Scale,
    translation: Translation,
    rotation: Rotation,
    skew: Skew,
    flip: Flip,
    opacity: f32,
}

impl Transformer {
    pub fn set_scale(&mut self, val: f32) {
        self.scale.scale_x = val;
        self.scale.scale_y = val;
    }

    pub fn set_rotate(&mut self, angle: f32) {
        self.rotation.angle = angle
    }

    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation.x = x;
        self.translation.y = y;
        self.translation.z = z;
    }

    pub fn set_skew(&mut self, x: f32, y: f32) {
        self.skew.x = x;
        self.skew.y = y;
    }

    pub fn set_flip(&mut self, flip_x: bool, flip_y: bool) {
        self.flip.x = flip_x;
        self.flip.y = flip_y;
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }

    fn get_skew_mat(&self) -> Mat4 {
        let tan_x = (self.skew.x * (PI / 180.0)).tan();
        let tan_y = (self.skew.y * (PI / 180.0)).tan();
        #[rustfmt::skip]
        let skew = Mat4::new(
            1.0,   tan_y, 0.0, 0.0,
            tan_x, 1.0,   0.0, 0.0,
            0.0,   0.0,   1.0, 0.0,
            0.0,   0.0,   0.0, 1.0,
        );
        skew
    }

    pub fn get_similar_mat(&self, mid_x: f32, mid_y: f32) -> Mat4 {
        let mut trans = Mat4::identity();
        trans = glm::translate(
            &trans,
            &glm::vec3(self.translation.x, self.translation.y, 0.0),
        );

        let flip_x = if self.flip.x { -1.0 } else { 1.0 };
        let flip_y = if self.flip.y { -1.0 } else { 1.0 };
        trans = glm::scale(
            &trans,
            &glm::vec3(
                self.scale.scale_x * flip_x,
                self.scale.scale_y * flip_y,
                1.0,
            ),
        );

        if self.skew.x.abs() > 0.0001 || self.skew.y.abs() > 0.0001 {
            trans = glm::translate(&trans, &glm::vec3(mid_x, mid_y, 0.0));
            trans = trans * self.get_skew_mat();
            trans = glm::translate(&trans, &glm::vec3(-mid_x, -mid_y, 0.0));
        }

        if self.rotation.angle.abs() > 0.0001 {
            let rad = self.rotation.angle * (PI / 180.0);
            trans = glm::translate(&trans, &glm::vec3(mid_x, mid_y, 0.0));
            trans = glm::rotate(&trans, rad, &glm::vec3(0.0, 0.0, 1.0));
            trans = glm::translate(&trans, &glm::vec3(-mid_x, -mid_y, 0.0));
        }

        trans
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_similarity(
        &self,
        x: f32,
        y: f32,
        z: f32,
        mid_x: f32,
        mid_y: f32,
        width: f32,
        height: f32,
    ) -> (f32, f32) {
        let proj = glm::ortho(0.0, width, 0.0, height, -1.0, 1.0);
        let view = Mat4::identity();
        let trans = self.get_similar_mat(mid_x, mid_y);

        let mat = proj * view * trans;
        let p = Point3::new(x, y, z);
        let ret = mat.transform_point(&p);
        (ret.x, ret.y)
    }
}

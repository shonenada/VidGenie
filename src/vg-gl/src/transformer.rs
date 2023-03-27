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
        Self { scale_x: 1.0, scale_y: 1.0 }
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
pub struct Transformer {
    scale: Scale,
    translation: Translation,
    rotation: Rotation,
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

    pub fn get_similar_mat(&self, mid_x: f32, mid_y: f32) -> Mat4 {
        let mut trans = Mat4::identity();
        trans = glm::translate(&trans, &glm::vec3(self.translation.x, self.translation.y, 0.0));

        trans = glm::scale(&trans, &glm::vec3(self.scale.scale_x, self.scale.scale_y, 1.0));

        if self.rotation.angle > 0.0 {
            let rad = self.rotation.angle * (PI / 180.0);
            trans = glm::translate(&trans, &glm::vec3(mid_x, mid_y, 0.0));
            trans = glm::rotate(&trans, rad, &glm::vec3(0.0, 0.0, 1.0));
            trans = glm::translate(&trans, &glm::vec3(-mid_x, -mid_y, 0.0));
        }

        trans
    }

    // TODO: Simplify
    #[allow(clippy::too_many_arguments)]
    pub fn apply_similarity(&self, x: f32, y: f32, z: f32, mid_x: f32, mid_y: f32, width: f32, height: f32) -> (f32, f32) {
        let proj = glm::ortho(0.0, width, 0.0, height, -1.0, 1.0);
        let view = Mat4::identity();
        let trans = self.get_similar_mat(mid_x, mid_y);

        let mat = proj * view * trans;
        let p = Point3::new(x, y, z);
        let ret = mat.transform_point(&p);
        (ret.x, ret.y)
    }
}
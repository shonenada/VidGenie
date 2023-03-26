use std::f32::consts::PI;

use glm::TMat4;
use log::debug;
use nalgebra::{Point3, Similarity3, Vector3};

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

impl Into<Vector3<f32>> for Translation {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.x, self.y, self.z)
    }
}

#[derive(Default, Clone)]
struct Rotation {
    angle: f32,
}

impl Into<Vector3<f32>> for Rotation {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.angle * PI / 180.0, 0.0, 0.0)
    }
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

    pub fn get_similar_mat(&self) -> Similarity3<f32> {
        let translation = self.translation.clone();
        let rotate = self.rotation.clone();
        let scale = self.scale.scale_x;

        Similarity3::new(
            translation.into(),
            rotate.into(),
            scale,
        )
    }

    pub fn apply_similarity(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let p = Point3::new(x, y, z);
        let trans = self.get_similar_mat();
        let ret: Point3<f32> = trans * p;
        (ret.x, ret.y, ret.z)
    }
    /// Keep here for allowing any transform.
    pub fn get_transform_mat(&self) -> glm::Mat4 {
        let mut trans: TMat4<f32> = glm::convert(glm::Mat4::identity());

        let scale_vec = glm::vec3(self.scale.scale_x, self.scale.scale_y, 1.0);
        trans = glm::scale(&trans, &scale_vec);

        let translation_vec = glm::vec3(self.translation.x, self.translation.y, 0.0);
        // trans = trans.append_translation(&translation_vec);
        trans = glm::translate(&trans, &translation_vec);

        let rotation_vec = glm::vec3(0.0, 0.0, 1.0);
        trans = glm::rotate(&trans, self.rotation.angle, &rotation_vec);

        trans
    }

    /// Keep here for allowing scale for x and y separately.
    pub fn apply_transform_with_axis_scale(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let p = Point3::new(x, y, z);
        let trans = self.get_transform_mat();
        let r = trans.transform_point(&p);
        (r.x, r.y, r.z)
    }
}
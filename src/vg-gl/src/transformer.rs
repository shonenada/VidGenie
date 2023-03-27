use std::f32::consts::PI;

use glm::TMat4;
use log::debug;
use nalgebra::{Point2, Point3, Similarity2, Similarity3, Vector2, Vector3};

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

impl Into<Vector2<f32>> for Translation {
    fn into(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
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

    pub fn get_similar_mat(&self) -> Similarity2<f32> {
        let translation = self.translation.clone();
        let scale = self.scale.scale_x;

        Similarity2::new(
            translation.into(),
            0.0,
            scale,
        )
    }

    pub fn apply_similarity(&self, x: f32, y: f32, _z: f32) -> (f32, f32) {
        let p = Point2::new(x, y);
        let trans = self.get_similar_mat();
        let ret: Point2<f32> = trans * p;
        (ret.x, ret.y)
    }
}
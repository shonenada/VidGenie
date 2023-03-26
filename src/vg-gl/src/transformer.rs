use glm::TMat4;
use log::debug;

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

    pub fn get_transform_mat(&self) -> glm::Mat4 {
        let mut trans: TMat4<f32> = glm::convert(glm::Mat4::identity());

        let scale_vec = glm::vec3(self.scale.scale_x, self.scale.scale_y, 1.0);
        trans = glm::scale(&trans, &scale_vec);

        let translation_vec = glm::vec3(self.translation.x, self.translation.y, 0.0);
        trans = glm::translate(&trans, &translation_vec);

        let rotation_vec = glm::vec3(0.0, 0.0, 1.0);
        trans = glm::rotate(&trans, self.rotation.angle, &rotation_vec);

        debug!("Transform: {}", trans);

        trans
    }

    pub fn apply_transform(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let p = glm::vec3(x, y, z);
        let trans = self.get_transform_mat();
        let r = trans.transform_vector(&p);
        (r.x, r.y, r.z)
    }
}
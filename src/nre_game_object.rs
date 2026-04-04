use crate::nre_model::NreModel;
use glam;

pub struct NreGameObject {
    pub model: NreModel,
    pub translation: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
}

impl NreGameObject {
    //
    pub fn new(model: NreModel) -> Self {
        Self {
            model,
            translation: glam::Vec3::ZERO,
            rotation: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn transform(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_euler(
                glam::EulerRot::YXZ,
                self.rotation.y,
                self.rotation.x,
                self.rotation.z,
            ),
            self.translation,
        )
    }
}

use glam;

pub trait Camera {
    fn view_matrix(&self) -> glam::Mat4;
    fn projection_matrix(&self) -> glam::Mat4;
    fn position(&self) -> glam::Vec3;
}

pub struct PerspectiveCamera {
    pub world_position: glam::Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub plane_near: f32,
    pub plane_far: f32,
}

impl Camera for PerspectiveCamera {
    //
    fn position(&self) -> glam::Vec3 {
        self.world_position
    }
    //
    fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.fov_y,
            self.aspect_ratio,
            self.plane_near,
            self.plane_far,
        )
    }
    //
    fn view_matrix(&self) -> glam::Mat4 {
        let forward = glam::Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();
        glam::Mat4::look_at_rh(
            self.world_position,
            self.world_position + forward,
            glam::Vec3::Y,
        )
    }
}

impl PerspectiveCamera {
    pub fn new(aspect_ratio: f32, fov_y: f32) -> Self {
        Self {
            world_position: glam::Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            aspect_ratio,
            fov_y,
            plane_near: 0.1,
            plane_far: 100.0,
        }
    }
}

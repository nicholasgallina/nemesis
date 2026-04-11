// dependencies
use glam;

pub struct Controller {
    pub move_speed: f32,
    pub look_speed: f32,
    pub velocity: glam::Vec3,
    pub damping: f32,
}

impl Controller {
    //
    pub fn new() -> Self {
        Self {
            move_speed: 6.0,
            look_speed: 0.010,
            velocity: glam::Vec3::ZERO,
            damping: 0.85,
        }
    }
    //
    pub fn update(
        &mut self,
        dt: f32,
        keys: &std::collections::HashSet<winit::keyboard::KeyCode>,
        camera: &mut crate::nre_camera::PerspectiveCamera,
    ) {
        use winit::keyboard::KeyCode;

        if keys.contains(&KeyCode::ArrowRight) {
            camera.yaw += self.look_speed;
        }
        if keys.contains(&KeyCode::ArrowLeft) {
            camera.yaw -= self.look_speed;
        }
        if keys.contains(&KeyCode::ArrowUp) {
            camera.pitch -= self.look_speed;
        }
        if keys.contains(&KeyCode::ArrowDown) {
            camera.pitch += self.look_speed;
        }

        let forward = glam::Vec3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos(),
        )
        .normalize();
        let right = forward.cross(glam::Vec3::Y).normalize();

        let mut input = glam::Vec3::ZERO;
        if keys.contains(&KeyCode::KeyW) {
            input += forward;
        }
        if keys.contains(&KeyCode::KeyS) {
            input -= forward;
        }
        if keys.contains(&KeyCode::KeyA) {
            input -= right;
        }
        if keys.contains(&KeyCode::KeyD) {
            input += right;
        }
        if keys.contains(&KeyCode::KeyE) {
            input -= glam::Vec3::Y;
        }
        if keys.contains(&KeyCode::KeyQ) {
            input += glam::Vec3::Y;
        }

        self.velocity = self.velocity * self.damping + input * self.move_speed * dt;
        camera.world_position += self.velocity * dt;
    }
}

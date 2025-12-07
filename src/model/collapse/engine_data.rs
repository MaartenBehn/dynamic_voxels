use octa_force::{camera::Camera, glam::Vec3};

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineData {
    pub cam_position: Vec3,
}

impl EngineData {
    pub fn new(camera: &Camera) -> Self {
        Self { 
            cam_position: camera.get_position_in_meters(), 
        }
    }
}

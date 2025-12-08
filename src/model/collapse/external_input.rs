use octa_force::{camera::Camera, glam::Vec3};

use crate::VOXELS_PER_METER;

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalInput {
    pub cam_position: Vec3,
}

impl ExternalInput {
    pub fn new(camera: &Camera) -> Self {
        
        // TODO External input should be in meters, but this is simpler for voxel output.
        Self { 
            cam_position: camera.get_position_in_meters() * VOXELS_PER_METER as f32, 
        }
    }
}

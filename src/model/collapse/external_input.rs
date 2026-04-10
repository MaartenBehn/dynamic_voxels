use octa_force::{camera::Camera, glam::Vec3};

use crate::{model::collapse::collapser::Collapser, util::shader_constants::VOXELS_PER_METER};

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalInput {
    pub cam_position: Vec3,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalInputMarker {
    pub cam: bool
}

impl ExternalInput {
    pub fn new(camera: &Camera) -> Self {
        
        // TODO External input should be in meters, but this is simpler for voxel output.
        Self { 
            cam_position: camera.get_position_in_meters() * VOXELS_PER_METER as f32, 
        }
    }
}

impl ExternalInputMarker {
    pub fn any(&self) -> bool {
        self.cam
    }
}

impl Collapser {

    pub fn external_input_changed(&mut self) {
        for (template_index, nodes) in self.nodes_per_template_index.iter().enumerate() {
            if !self.template.nodes[template_index].external_input_marker.any() {
                continue;
            }

            for node_key in nodes {
                self.pending.push_collpase(self.template.nodes[template_index].level, *node_key);
            }
        }
    }
}

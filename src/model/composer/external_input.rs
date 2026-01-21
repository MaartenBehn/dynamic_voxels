use itertools::Itertools;
use octa_force::camera::Camera;

use crate::{model::collapse::external_input::ExternalInput};

use super::{nodes::ComposeNodeType, ModelComposer};

impl ModelComposer {
    pub fn update_external_input(&mut self, camera: &Camera) {
        let engine_data = ExternalInput::new(camera);

        let cam_changed = self.external_input.cam_position != engine_data.cam_position;
        self.external_input = engine_data;

        if cam_changed {
            self.graph.flags.set_cam_notes_as_changed(&self.graph.snarl);
        }
    }
}

use octa_force::OctaResult;

use crate::volume::remove_trait::VolumeRemove;

use super::{collapse::{CollapseNodeKey, CollapseOperation, Collapser}, template::TemplateTree, traits::ModelGenerationTypes};

impl<T: ModelGenerationTypes> Collapser<T> {
    pub fn push_restricts_collapse_opperations(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) {
        let node = &self.nodes[node_index];

        for (identifier, index) in node.restricts.iter() {
            self.pending_collapse_opperations.push_back(CollapseOperation::RestrictHook { 
                index: node_index, 
                identifier: node.identifier, 
                restricts_index: *index, 
                restricts_identifier: *identifier, 
            });
        }
    }

    pub fn restrict_number_range<F: Fn(i32) -> bool>(&mut self, index: CollapseNodeKey, filter: F) {
        let values = self.get_number_values_mut(index);
        values.retain(|i| filter(*i));
    }

    pub fn restricts_volume(&mut self, index: CollapseNodeKey, remove: T::Volume) {
        let volume = self.get_volume_mut(index);
        volume.remove_volume(remove);
    }

    pub fn restricts_volume2d(&mut self, index: CollapseNodeKey, remove: T::Volume2D) {
        let volume = self.get_volume2d_mut(index);
        volume.remove_volume(remove);
    }

}

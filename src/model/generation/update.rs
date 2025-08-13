use octa_force::{anyhow::{self, bail, anyhow}, OctaResult};

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{builder::ModelSynthesisBuilder, collapse::Collapser, pos_set::PositionSet, template::{NodeTemplateValue, TemplateIndex, TemplateTree}, traits::ModelGenerationTypes};

impl<T: ModelGenerationTypes> TemplateTree<T> {
    pub fn get_node_position_set(&mut self, identifier: T::Identifier) -> &mut PositionSet<T> {
        let index = self.get_node_index_by_identifier(identifier);
        self.get_node_position_set_by_index(index)
    }

    pub fn get_node_position_set_by_index(&mut self, index: TemplateIndex) -> &mut PositionSet<T> {
        let node = &mut self.nodes[index];

        if !matches!(node.value, NodeTemplateValue::PosSet(..)) {
            panic!("Node Value is not Position Set {node:?}");
        }

        let NodeTemplateValue::PosSet(pos_set) = &mut node.value else { unreachable!() };
        pos_set
    }

    pub fn get_node_index_by_identifier(&self, identifier: T::Identifier) -> TemplateIndex {
        self.nodes.iter()
            .position(|n| n.identifier == identifier)
            .expect("No Node with Identifier")
    }
}

impl<T: ModelGenerationTypes> Collapser<T> { 
    pub fn re_collapse_all_nodes_with_identifier(&mut self, identifier: T::Identifier) {
        for (key, node) in self.nodes.iter()
            .filter(|(_, n)| n.identifier == identifier) {

            self.pending_collapses.push(node.level, key);
        }
    }
}

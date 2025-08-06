use octa_force::{anyhow::{self, bail, anyhow}, OctaResult};

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{builder::ModelSynthesisBuilder, collapse::Collapser, pos_set::PositionSet, template::{NodeTemplateValue, TemplateIndex, TemplateTree}, traits::ModelGenerationTypes};

impl<T: ModelGenerationTypes> TemplateTree<T> {
    pub fn get_node_position_set(&mut self, identifier: T::Identifier) -> OctaResult<&mut PositionSet<T>> {
        let index = self.get_node_index_by_identifier(identifier)
            .ok_or(anyhow!("Identifier not found in template!"))?;
        self.get_node_position_set_by_index(index)
    }

    pub fn get_node_position_set_by_index(&mut self, index: TemplateIndex) -> OctaResult<&mut PositionSet<T>> {
        let node = &mut self.nodes[index];

        if !matches!(node.value, NodeTemplateValue::PosSet(..)) {
            bail!("Node Value is not Position Set {node:?}");
        }

        let NodeTemplateValue::PosSet(pos_set) = &mut node.value else { unreachable!() };
        Ok(pos_set)
    }

    pub fn get_node_index_by_identifier(&self, identifier: T::Identifier) -> Option<TemplateIndex> {
        self.nodes.iter().position(|n| n.identifier == identifier)
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

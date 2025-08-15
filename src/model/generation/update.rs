use octa_force::{anyhow::{self, anyhow, bail, Context}, OctaResult};

use super::{builder::ModelSynthesisBuilder, collapse::Collapser, pos_set::PositionSet, template::{NodeTemplateValue, TemplateIndex, TemplateTree}, traits::ModelGenerationTypes};

impl<T: ModelGenerationTypes> TemplateTree<T> {
    pub fn get_node_position_set(&mut self, identifier: T::Identifier) -> OctaResult<&mut PositionSet<T>> {
        let index = self.get_node_index_by_identifier(identifier)
            .context("Tyring to get template position set node by identifier")?;

        self.get_node_position_set_by_index(index)
            .context("Tyring to get template position set node by identifier")
    }

    pub(super) fn get_node_position_set_by_index(&mut self, index: TemplateIndex) -> OctaResult<&mut PositionSet<T>> {
        let node = &mut self.nodes[index];

        let NodeTemplateValue::PosSet(pos_set) = &mut node.value else { bail!("{:?} is not Pos Set", node.identifier) };
        Ok(pos_set)
    }

    pub(super) fn get_node_index_by_identifier(&self, identifier: T::Identifier) -> OctaResult<TemplateIndex> {
        self.nodes.iter()
            .position(|n| n.identifier == identifier)
            .context(format!("No Node with Identifier {:?}", identifier))
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

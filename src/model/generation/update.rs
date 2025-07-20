use octa_force::{anyhow::{self, bail, anyhow}, OctaResult};

use crate::volume::VolumeQureyPosValid;

use super::{builder::{BU, IT}, collapse::{Collapser, NodeOperationType}, pending_operations::NodeOperation, pos_set::PositionSet, template::{NodeTemplateValue, TemplateIndex, TemplateTree}};

impl<I: IT, V: VolumeQureyPosValid> TemplateTree<I, V> {
    pub fn get_node_position_set(&mut self, identifier: I) -> OctaResult<&mut PositionSet<V>> {
        let index = self.get_node_index_by_identifier(identifier)
            .ok_or(anyhow!("Identifier not found in template!"))?;
        self.get_node_position_set_by_index(index)
    }

    pub fn get_node_position_set_by_index(&mut self, index: TemplateIndex) -> OctaResult<&mut PositionSet<V>> {
        let node = &mut self.nodes[index];

        if !matches!(node.value, NodeTemplateValue::PosSet(..)) {
            bail!("Node Value is not Position Set {node:?}");
        }

        let NodeTemplateValue::PosSet(pos_set) = &mut node.value else { unreachable!() };
        Ok(pos_set)
    }

    pub fn get_node_index_by_identifier(&self, identifier: I) -> Option<TemplateIndex> {
        self.nodes.iter().position(|n| n.identifier == identifier)
    }
}

impl<I: IT, U: BU, V: VolumeQureyPosValid> Collapser<I, U, V> { 
    pub fn re_collapse_all_nodes_with_identifier(&mut self, identifier: I) {
        for (key, node) in self.nodes.iter()
            .filter(|(_, n)| n.identfier == identifier) {

            self.pending_operations.push(node.level, NodeOperation { 
                key, 
                typ: NodeOperationType::CollapseValue,
            });
        }
    }
}

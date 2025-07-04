use octa_force::{anyhow::{self, ensure, anyhow}, log::info, OctaResult};
use slotmap::Key;

use crate::{model::generation::{collapse::{CollapseOperation, NodeOperationType}, pending_operations::NodeOperation}, volume::VolumeQureyPosValid};

use super::{builder::{BU, IT}, collapse::{CollapseNodeKey, Collapser}};



impl<'a, I: IT, U: BU, V: VolumeQureyPosValid> Collapser<'a, I, U, V> {

    pub fn reset_node(&mut self, node_index: CollapseNodeKey) -> OctaResult<()> {
        let node = self.get_node_ref_from_node_index(node_index)?;
        info!("{:?} Reset {:?}", node_index, node.identfier);

        let node_template = Self::get_template_from_node_ref_unpacked(&self.template, node);
        for child in node.children.iter()
            .filter(|(template_index, _)| node_template.defines_ammount.iter()
                .find(|ammount| ammount.index == *template_index)
                .is_none())
            .map(|(_, c)| c)
            .flatten()
            .copied()
            .collect::<Vec<_>>() {

            self.delete_node(child, true)?;
        }

        let node = self.get_node_ref_from_node_index(node_index)?; 
        
        self.pending_operations.push(node_template.level, NodeOperation { 
            index: node_index, 
            typ: NodeOperationType::CollapseValue,
        });

        Ok(())
    }

    pub fn delete_node(&mut self, node_index: CollapseNodeKey, recreate: bool) -> OctaResult<()> {
        let node = self.nodes.remove(node_index);
        if node.is_none() {
            return Ok(());
        }
        let node = node.unwrap();
        ensure!(!node.defined_by.is_null(), "Trying to delete root node!");

        info!("{:?} Delete {:?}", node_index, node.identfier);

        let template_node = self.get_template_from_node_ref(&node);

        self.pending_operations.delete(template_node.level, node_index);

        for (_, depends) in node.depends.iter() {
            let depends_node = self.get_node_mut_from_node_index(*depends);
            if depends_node.is_err() {
                continue;
            }
            let depends_node = depends_node.unwrap();

            let children = depends_node.children.iter_mut()
                .find(|(template_index, _)| *template_index == node.template_index)
                .map(|(_, c)| c)
                .ok_or(anyhow!("When deleting node the template index of the node was not present in the children of a dependency"))?;

            let i = children.iter()
                .position(|t| *t == node_index)
                .ok_or(anyhow!("When deleting node index of the node was not present in the children of a dependency"))?;
            
            children.swap_remove(i);
        }

        self.pending_collapse_opperations.push(CollapseOperation::Undo { 
            identifier: node.identfier, 
            undo_data: node.undo_data,
        });

        for child in node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {

            self.delete_node(*child, recreate)?;
        }

        if recreate {
            let node_template = self.get_template_from_node_ref(&node); 
            if self.has_index(node.defined_by) {
                
                self.pending_operations.push(node_template.level, NodeOperation { 
                    index: node.defined_by, 
                    typ: NodeOperationType::UpdateDefined(node.template_index),
                });
            } 
        }

        return Ok(());
    }

    pub fn set_next_reset(&mut self, index: CollapseNodeKey, set_to: CollapseNodeKey) -> OctaResult<()> {
        let node = self.get_node_mut_from_node_index(index)?;
        node.next_reset = set_to;

        Ok(())
    }

}

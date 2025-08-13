use octa_force::{anyhow::{self, ensure, anyhow}, log::info, OctaResult};
use slotmap::Key;

use crate::{model::generation::collapse::CollapseOperation, volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{collapse::{CollapseNodeKey, Collapser}, template::TemplateTree, traits::ModelGenerationTypes};



impl<T: ModelGenerationTypes> Collapser<T> {

    pub fn reset_node(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) {
        let node = &self.nodes[node_index];
        info!("{:?} Reset {:?}", node_index, node.identifier);

        let node_template = self.get_template_from_node_ref(node, template);
        for child in node.children.iter()
            .filter(|(template_index, _)| node_template.defines_n.iter()
                .map(|a| a.index)
                .chain(node_template.defines_by_value.iter()
                    .map(|a| a.index)
                )
                .find(|index| *index == *template_index)
                .is_none())
            .map(|(_, c)| c)
            .flatten()
            .copied()
            .collect::<Vec<_>>() {

            self.delete_node(child, template);
        }

        let node = &self.nodes[node_index]; 
        
        self.pending_collapses.push(node_template.level, node_index);
    }

    pub fn delete_node(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) {
        let node = self.nodes.remove(node_index);
        if node.is_none() {
            return;
        }
        let node = node.unwrap();
        assert!(!node.defined_by.is_null(), "Trying to delete root node!");

        info!("{:?} Delete {:?}", node_index, node.identifier);

        let template_node = self.get_template_from_node_ref(&node, template);

        self.pending_collapses.delete(template_node.level, node_index);

        for (_, depends) in node.depends.iter() {
            let Some(depends_node) = self.nodes.get_mut(*depends) else { 
                continue;
            };

            let children = depends_node.children.iter_mut()
                .find(|(template_index, _)| *template_index == node.template_index)
                .map(|(_, c)| c)
                .expect("When deleting node the template index of the node was not present in the children of a dependency");

            let i = children.iter()
                .position(|t| *t == node_index)
                .expect("When deleting node index of the node was not present in the children of a dependency");
            
            children.swap_remove(i);
        }

        self.pending_collapse_opperations.push_back(CollapseOperation::Undo { 
            identifier: node.identifier, 
            undo_data: node.undo_data,
        });

        for child in node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {

            self.delete_node(*child, template);
        }
    }

    pub fn set_next_reset(&mut self, index: CollapseNodeKey, set_to: CollapseNodeKey) {
        let node = &mut self.nodes[index];
        node.next_reset = set_to;
    }

}

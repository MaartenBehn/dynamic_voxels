use std::iter;

use itertools::Itertools;
use slotmap::Key;

use crate::{model::composer::{build::BS, dependency_tree::{DependencyPath, DependencyTree}, template::{ComposeTemplate, TemplateIndex, TemplateNode}}, util::{number::Nu, vector::Ve}};

use super::{add_nodes::GetValueData, collapser::{CollapseNode, CollapseNodeKey, Collapser}};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub fn get_depends<'a>(
        &self, 
        parent_index: CollapseNodeKey,
        template: &'a ComposeTemplate<V2, V3, T, B>,
        template_node: &'a TemplateNode<V2, V3, T, B>,
        new_node_template: &'a TemplateNode<V2, V3, T, B>,
    ) -> Vec<(TemplateIndex, Vec<CollapseNodeKey>)> {

        // Contains a list of node indecies matching the template dependency
        let mut depends = iter::repeat_with(|| vec![])
            .take(new_node_template.depends.len())
            .collect::<Vec<_>>();

        if depends.is_empty() {
            return vec![];
        }
        
        let mut pending_paths = vec![(&new_node_template.dependecy_tree.steps[0], parent_index)];
        while let Some((step, index)) = pending_paths.pop() {
            let step_node = &self.nodes[index];

            if let Some(i) = step.leaf {
                depends[i].push(index);
            }

            for child_index in step.children.iter() {
                let child_step = &new_node_template.dependecy_tree.steps[*child_index];

                let edges = if child_step.up { 
                    step_node.depends.iter()
                        .filter(|(template_index, _)| *template_index == child_step.into_index)
                        .map(|(_, c)| c)
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>()
                } else { 
                    step_node.children.iter()
                        .filter(|(template_index, _)| *template_index == child_step.into_index)
                        .map(|(_, c)|c)
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>()
                };

                for edge in edges {
                    pending_paths.push((child_step, edge));
                }
            }
        }

        let depends = new_node_template.depends.iter()
            .zip(depends)
            .map(|(depend_template_node, nodes)| {

                let depend_template_node = &template.nodes[*depend_template_node];
                assert!(nodes.len() > 0, 
                    "No nodes for dependency or knows of node found! \n {:?} tyring to find {:?}", 
                    new_node_template.node_id, depend_template_node.node_id);
                (depend_template_node.index, nodes)
            }).collect::<Vec<_>>();

        depends
    }

    pub fn get_depends_from_loop_path<'a>(
        &self,
        get_value_data: GetValueData,
        path: &'a DependencyPath,
    ) -> Vec<CollapseNodeKey> {
        assert!(get_value_data.index != CollapseNodeKey::null(), 
            "Tying to calculate loop path but node is not created. This was called to evaluate ammount");

        let mut pointers = vec![get_value_data.index];
       
        for step in path.steps.iter() { 
            
            if step.up {
                pointers = pointers.into_iter()
                    .map(|pointer| {
                        let pointer_node = &self.nodes[pointer];

                        let (_, is) = pointer_node.depends.iter()
                            .find(|(t_index, _)| *t_index == step.into_index)
                            .expect("Path used step into depends that was not found");

                        is.iter()
                    })
                    .flatten()
                    .copied()
                    .collect_vec();
            } else {
                pointers = pointers.into_iter()
                    .map(|pointer| {
                        let pointer_node = &self.nodes[pointer];

                        pointer_node.children.iter()
                            .find(|(t_index, _)| *t_index == step.into_index)
                            .map(|(_, is)| is.iter())
                    })
                    .flatten()
                    .flatten()
                    .copied()
                    .collect_vec();
            }

        } 
        pointers
    }
}

use std::iter;

use itertools::Itertools;
use slotmap::Key;

use crate::{model::{collapse::collapser::CollapseChildKey, composer::build::BS, template::{Template, TemplateIndex, dependency_tree::DependencyPath, nodes::TemplateNode}}, util::{number::Nu, vector::Ve}};

use super::{add_nodes::GetValueData, collapser::{CollapseNode, CollapseNodeKey, Collapser}};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub fn get_depends<'a>(
        &self, 
        parent_index: CollapseNodeKey,
        child_key: CollapseChildKey,
        template: &'a Template<V2, V3, T, B>,
        template_node: &'a TemplateNode,
        new_node_template: &'a TemplateNode,
    ) -> Vec<(TemplateIndex, Vec<(CollapseNodeKey, CollapseChildKey)>)> {

        dbg!(&self.nodes);
        dbg!(new_node_template);

        // Contains a list of node indecies matching the template dependency
        let mut depends = iter::repeat_with(|| vec![])
            .take(new_node_template.depends.len())
            .collect::<Vec<_>>();

        if depends.is_empty() {
            return vec![];
        }
      
        // If step node is a child with a child index only go into the edge of other
        // children with the same index.

        let mut pending_paths = vec![(&new_node_template.dependecy_tree.steps[0], parent_index, child_key)];
        while let Some((step, index, child_key)) = pending_paths.pop() {

            let step_node = &self.nodes[index];

            if let Some(i) = step.leaf {
                depends[i].push((index, child_key));
            }

            for child_index in step.children.iter() {
                let child_step = &new_node_template.dependecy_tree.steps[*child_index];

                if child_step.up {
                    for (i, k) in step_node.depends.iter()
                        .find(|(template_index, _)| *template_index == child_step.into_index)
                        .map(|(_, c)| c)
                        .into_iter()
                        .flatten()
                        .copied() { 
                        pending_paths.push((child_step, i, k));
                    }    
                } else { 
                    let mut children_of_step = step_node.children.iter()
                        .find(|(template_index, _)| *template_index == child_step.into_index)
                        .map(|(_, c)|c)
                        .into_iter()
                        .flatten()
                        .copied();

                    if !child_key.is_null() {
                        
                        let child_of_step = children_of_step.find(|(_, k)| *k == child_key);

                        if let Some((i, k)) = child_of_step {
                            pending_paths.push((child_step, i, CollapseChildKey::null()));
                            continue;
                        }
                    }

                    for (i, k) in children_of_step {
                        pending_paths.push((child_step, i, CollapseChildKey::null()));
                    } 
                }
            }
        }

        let depends = new_node_template.depends.iter()
            .zip(depends)
            .map(|(depend_template_node, nodes)| {

                let depend_template_node = &template.nodes[*depend_template_node];
                assert!(nodes.len() > 0, 
                    "No nodes for dependency or knows of node found! \n {:?} tyring to find {:?}", 
                    new_node_template.index, depend_template_node.index);
                (depend_template_node.index, nodes)
            }).collect::<Vec<_>>();

        dbg!(&depends);

        depends
    }

    pub fn get_depends_from_loop_path<'a>(
        &self,
        get_value_data: GetValueData,
        path: &'a DependencyPath,
    ) -> Vec<(CollapseNodeKey, CollapseChildKey)> {
        return vec![];
        /*

        assert!(get_value_data.index != CollapseNodeKey::null(), 
            "Tying to calculate loop path but node is not created. This was called to evaluate ammount");

        let mut pointers = vec![get_value_data.index];
       
        for step in path.steps.iter() { 
            
            if step.up {
                pointers = pointers.into_iter()
                    .map(|pointer| {
                        let pointer_node = &self.nodes[pointer];

                        let (_, is) = pointer_node.depends.iter()
                            .find(|((t_index, _), _)| *t_index == step.into_index)
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
*/
    }
}

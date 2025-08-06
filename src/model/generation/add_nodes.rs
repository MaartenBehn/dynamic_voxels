use std::iter;

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;

use crate::{model::generation::collapse::CollapseChildKey, volume::{VolumeQureyPosValid, VolumeQureyPosValid2D}};

use super::{collapse::{CollapseNode, CollapseNodeKey, Collapser, NodeDataType}, pos_set::PositionSet, relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateAmmountN, TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes};


impl<T: ModelGenerationTypes> Collapser<T> {

    pub fn create_defines_n(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) -> OctaResult<()> {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for ammount in template_node.defines_n.iter() {
            let (depends, knows) = self.get_depends_and_knows_for_template(
                node_index, 
                ammount.index,   
                template,
                template_node,
                &ammount.dependecy_tree)?;

            for _ in 0..ammount.ammount {
                self.add_node(ammount.index, depends.clone(), knows.clone(), node_index, CollapseChildKey::null(), template); 
            }
        }

        Ok(())
    }

    pub fn update_defined_by_number_range(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>, n: usize) -> OctaResult<()> {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for ammount in template_node.defines_by_value.iter() {
            let node = &self.nodes[node_index];
            let present_children = node.children.iter()
                .find(|(template_index, _)| *template_index == ammount.index)
                .map(|(_, children)| children.as_slice())
                .unwrap_or(&[]);

            let present_children_len = present_children.len(); 
            if present_children_len < n {
                let (depends, knows) = self.get_depends_and_knows_for_template(
                    node_index, 
                    ammount.index,   
                    template,
                    template_node,
                    &ammount.dependecy_tree)?;

                for _ in present_children_len..n {
                    self.add_node(ammount.index, depends.clone(), knows.clone(), node_index, CollapseChildKey::null(), template); 
                }
            } else if present_children_len > n {

                for child in present_children.to_owned().into_iter().take(n) {
                    self.delete_node(child, template)?;
                }
            } 
        }

        Ok(())
    }

    pub fn upadte_defined_by_pos_set<'a>(
        &mut self, 
        node_index: CollapseNodeKey, 
        to_create_children: &[CollapseChildKey], 
        template: &'a TemplateTree<T>, 
        template_node: &'a TemplateNode<T>
    ) -> OctaResult<()> {

        for ammount in template_node.defines_by_value.iter() {
            let node = &self.nodes[node_index];
            let NodeDataType::PosSet(pos_set) = &node.data else { unreachable!() }; 

            let to_remove_children = node.children.iter()
                .find(|(template_index, _)| *template_index == ammount.index)
                .map(|(_, children)| children)
                .unwrap_or(&vec![])
                .iter()
                .map(|key| (*key, &self.nodes[*key]) )
                .filter(|(_, child)| !pos_set.is_valid_child(child.child_key))
                .map(|(key, _)| key )
                .collect::<Vec<_>>();


            let (depends, knows) = self.get_depends_and_knows_for_template(
                node_index, 
                ammount.index, 
                template,
                template_node,
                &ammount.dependecy_tree)?;

            for child_index in to_remove_children {
                self.delete_node(child_index, template)?;
            }

            for new_child in to_create_children {
                self.add_node(ammount.index, depends.clone(), knows.clone(), node_index, *new_child, template); 
            }
        }

        Ok(())
    }
  
    pub fn get_depends_and_knows_for_template<'a>(
        &self, 
        node_index: CollapseNodeKey,
        new_template_node_index: usize, 
        template: &'a TemplateTree<T>,
        template_node: &'a TemplateNode<T>,
        tree: &'a RelativePathTree,
    ) -> OctaResult<(Vec<(T::Identifier, CollapseNodeKey)>, Vec<(T::Identifier, CollapseNodeKey)>)> {
        let new_node_template = &template.nodes[new_template_node_index];

        // Contains a list of node indecies matching the template dependency
        let mut depends = iter::repeat_with(|| vec![])
            .take(new_node_template.depends.len())
            .collect::<Vec<_>>();
        let mut knows = iter::repeat_with(|| vec![])
            .take(new_node_template.knows.len())
            .collect::<Vec<_>>();

        let mut pending_paths = tree.starts.iter()
            .map(|start| {
                (&tree.steps[*start], node_index)
            }).collect::<Vec<_>>();

        while let Some((step, index)) = pending_paths.pop() {
            let step_node = &self.nodes[index];
             
            let edges = if step.up { 
                step_node.depends.iter()
                    .map(|(_, i)|*i)
                    .filter(|i| self.nodes[*i].template_index == step.into_index)
                    .collect::<Vec<_>>()
            } else { 
                step_node.children.iter()
                    .filter(|(template_index, _)| *template_index == step.into_index)
                    .map(|(template_index, c)|c)
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>()
            };

            match step.leaf {
                LeafType::None => {},
                LeafType::Depends(i) => {
                    for edge in edges.iter() {
                        depends[i].push(*edge);
                    }
                },
                LeafType::Knows(i) => {
                    for edge in edges.iter() {
                         knows[i].push(*edge);
                    }
                },
            }

            for edge in edges {
                for child_index in step.children.iter() {
                    let child_step = &tree.steps[*child_index];
                    pending_paths.push((child_step, edge))
                }
            }  
        }

        let transform_depends_and_knows = |
            template_list: &[TemplateIndex], 
            found_list: Vec<Vec<CollapseNodeKey>>
        | -> Vec<(T::Identifier, CollapseNodeKey)> {
            template_list.iter()
                .zip(found_list)
                .map(|(depend_template_node, nodes)| {
                    if *depend_template_node == template_node.index {
                        return (template_node.identifier, node_index);
                    }                    

                    let depend_template_node = &template.nodes[*depend_template_node];
                    assert_eq!(nodes.len(), 1, "Invalid number of nodes for dependency or knows of node found");
                    (depend_template_node.identifier, nodes[0])
                }).collect::<Vec<_>>()
        };

        let depends = transform_depends_and_knows(&new_node_template.depends, depends);
        let knows = transform_depends_and_knows(&new_node_template.knows, knows);

        Ok((depends, knows))
    }

    pub fn add_node(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        depends: Vec<(T::Identifier, CollapseNodeKey)>, 
        knows: Vec<(T::Identifier, CollapseNodeKey)>,
        defined_by: CollapseNodeKey,
        child_key: CollapseChildKey,
        template: &TemplateTree<T>,
    ) {
        let new_node_template = &template.nodes[new_node_template_index];

        let data = match &(&template.nodes[new_node_template_index]).value {
            NodeTemplateValue::Groupe => NodeDataType::None,
            NodeTemplateValue::BuildHook => NodeDataType::Build,

            NodeTemplateValue::NumberRangeHook 
            |NodeTemplateValue::PosSetHook => NodeDataType::NotValid, 
            NodeTemplateValue::NumberRange(number_range) => NodeDataType::NumberRange(number_range.to_owned()),
            NodeTemplateValue::PosSet(pos_set) => NodeDataType::PosSet(pos_set.to_owned()),
        };

        self.push_new_node(new_node_template, depends, knows, defined_by, child_key, data)
    }
 
    pub fn push_new_node(&mut self, 
        new_node_template: &TemplateNode<T>, 
        depends: Vec<(T::Identifier, CollapseNodeKey)>, 
        knows: Vec<(T::Identifier, CollapseNodeKey)>, 
        defined_by: CollapseNodeKey, 
        child_key: CollapseChildKey, 
        data: NodeDataType<T>
    ) {
        
        let index = self.nodes.insert(CollapseNode {
            template_index: new_node_template.index,
            identifier: new_node_template.identifier,
            level: new_node_template.level,
            children: vec![],
            depends: depends.clone(),
            knows,
            defined_by,
            data,
            next_reset: CollapseNodeKey::null(),
            undo_data: T::UndoData::default(),
            child_key,
        });
        info!("{:?} Node added {:?}", index, new_node_template.identifier);

        for (_, depend) in depends {
            let children_list = self.nodes[depend].children.iter_mut()
                .find(|(template_index, _)| *template_index == new_node_template.index)
                .map(|(_, c)| c);

            if children_list.is_none() {
                self.nodes[depend].children.push((new_node_template.index, vec![index]));
            } else {
                children_list.unwrap().push(index);
            };
        }
        
        self.pending_collapses.push(new_node_template.level, index);
    }
}

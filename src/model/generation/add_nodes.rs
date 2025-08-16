use std::iter;

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::generation::collapse::CollapseChildKey};

use super::{collapse::{CollapseNode, CollapseNodeKey, Collapser, NodeDataType, CreateDefinesOperation}, pos_set::PositionSet, relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateAmmountN, TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes};


impl<T: ModelGenerationTypes> Collapser<T> {

    pub fn create_defined(&mut self, opperation: CreateDefinesOperation, template: &TemplateTree<T>) {
        match opperation {
            CreateDefinesOperation::CreateN { parent_index, ammount_index } => {
                self.create_defined_n(parent_index, ammount_index, template);
            },
            CreateDefinesOperation::CreateByNumberRange { parent_index, by_value_index, ammount } => {
                self.create_defined_by_number_range(parent_index, by_value_index, ammount, template);
            },
            CreateDefinesOperation::CreateByPosSet { parent_index, by_value_index, to_create_children } => {
                self.create_defined_by_pos_set(parent_index, by_value_index, to_create_children, template);
            },
        }
    }

    pub fn update_defines_n(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, ammount) in template_node.defines_n.iter().enumerate() {
            let new_template_node = &template.nodes[ammount.template_index];
            
            self.pending.push_create_defined(new_template_node.level, CreateDefinesOperation::CreateN { 
                parent_index: node_index,
                ammount_index: i,
            });
        }
    }

    pub fn create_defined_n(&mut self, parent_index: CollapseNodeKey, ammount_index: usize, template: &TemplateTree<T>) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let ammount = &template_node.defines_n[ammount_index];

        let (restricts, depends, knows) = self.get_restricts_depends_and_knows_for_template(
            parent_index, 
            ammount.template_index,   
            template,
            template_node,
            &ammount.dependecy_tree);

        for _ in 0..ammount.ammount {
            self.add_node(
                ammount.template_index, 
                restricts.clone(), 
                depends.clone(), 
                knows.clone(), 
                parent_index, 
                CollapseChildKey::null(), 
                template); 
        }

    }

    pub fn update_defined_by_number_range(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>, n: usize) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, by_value) in template_node.defines_by_value.iter().enumerate() {
            let node = &self.nodes[node_index];
            let present_children = node.children.iter()
                .find(|(template_index, _)| *template_index == by_value.template_index)
                .map(|(_, children)| children.as_slice())
                .unwrap_or(&[]);

            let present_children_len = present_children.len(); 
            if present_children_len < n {
                let new_template_node = &template.nodes[by_value.template_index];

                self.pending.push_create_defined(new_template_node.level, CreateDefinesOperation::CreateByNumberRange { 
                    parent_index: node_index, 
                    by_value_index: i, 
                    ammount: n - present_children_len, 
                }); 

            } else if present_children_len > n {

                for child in present_children.to_owned().into_iter().take(n) {
                    self.delete_node(child, template);
                }
            } 
        }
    }

    pub fn create_defined_by_number_range(&mut self, parent_index: CollapseNodeKey, by_value_index: usize, ammount: usize, template: &TemplateTree<T>) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let by_value = &template_node.defines_by_value[by_value_index];

        let (restricts, depends, knows) = self.get_restricts_depends_and_knows_for_template(
            parent_index, 
            by_value.template_index,   
            template,
            template_node,
            &by_value.dependecy_tree);

        for _ in 0..ammount {
            self.add_node(
                by_value.template_index, 
                restricts.clone(), 
                depends.clone(), 
                knows.clone(), 
                parent_index, 
                CollapseChildKey::null(), 
                template); 
        }

    }

    pub fn update_defined_by_pos_set<'a>(
        &mut self, 
        node_index: CollapseNodeKey, 
        to_create_children: Vec<CollapseChildKey>, 
        template: &'a TemplateTree<T>, 
        template_node: &'a TemplateNode<T>
    ) {

        for (i, by_value) in template_node.defines_by_value.iter().enumerate() {
            let node = &self.nodes[node_index];
            let NodeDataType::PosSet(pos_set) = &node.data else { unreachable!() };

            let to_remove_children = node.children.iter()
                .find(|(template_index, _)| *template_index == by_value.template_index)
                .map(|(_, children)| children)
                .unwrap_or(&vec![])
                .iter()
                .map(|key| (*key, &self.nodes[*key]) )
                .filter(|(_, child)| !pos_set.is_valid_child(child.child_key))
                .map(|(key, _)| key )
                .collect::<Vec<_>>();

            if !to_create_children.is_empty() {
                let new_template_node = &template.nodes[by_value.template_index];

                self.pending.push_create_defined(new_template_node.level, CreateDefinesOperation::CreateByPosSet { 
                    parent_index: node_index, 
                    by_value_index: i, 
                    to_create_children: to_create_children.clone() 
                });
            } 
            
            for child_index in to_remove_children {
                self.delete_node(child_index, template);
            }
        }
    }

    pub fn create_defined_by_pos_set(
        &mut self, 
        parent_index: CollapseNodeKey, 
        by_value_index: usize,         
        to_create_children: Vec<CollapseChildKey>, 
        template: &TemplateTree<T>
    ) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let by_value = &template_node.defines_by_value[by_value_index];

        let (restricts, depends, knows) = self.get_restricts_depends_and_knows_for_template(
            parent_index, 
            by_value.template_index,   
            template,
            template_node,
            &by_value.dependecy_tree);

        for new_child in to_create_children {
            self.add_node(
                by_value.template_index, 
                restricts.clone(), 
                depends.clone(), 
                knows.clone(), 
                parent_index, 
                new_child, 
                template); 
        }
    }
  
    pub fn get_restricts_depends_and_knows_for_template<'a>(
        &self, 
        parent_index: CollapseNodeKey,
        new_template_node_index: usize, 
        template: &'a TemplateTree<T>,
        template_node: &'a TemplateNode<T>,
        tree: &'a RelativePathTree,
    ) -> (Vec<(T::Identifier, Vec<CollapseNodeKey>)>, Vec<(T::Identifier, Vec<CollapseNodeKey>)>, Vec<(T::Identifier, Vec<CollapseNodeKey>)>) {
        let new_node_template = &template.nodes[new_template_node_index];

        // Contains a list of node indecies matching the template dependency
        let mut restricts = iter::repeat_with(|| vec![])
            .take(new_node_template.restricts.len())
            .collect::<Vec<_>>();
        let mut depends = iter::repeat_with(|| vec![])
            .take(new_node_template.depends.len())
            .collect::<Vec<_>>();
        let mut knows = iter::repeat_with(|| vec![])
            .take(new_node_template.knows.len())
            .collect::<Vec<_>>();

        let mut pending_paths = vec![(&tree.steps[0], parent_index)];
        while let Some((step, index)) = pending_paths.pop() {
            let step_node = &self.nodes[index];

            for leaf in step.leafs.iter() {
                match leaf {
                    LeafType::Restricts(i) => {
                        restricts[*i].push(index);
                    },
                    LeafType::Depends(i) => {
                        depends[*i].push(index);
                    },
                    LeafType::Knows(i) => {
                        knows[*i].push(index);
                    },
                }
            }

            for child_index in step.children.iter() {
                let child_step = &tree.steps[*child_index];

                let edges = if child_step.up { 
                    step_node.depends.iter()
                        .map(|(_, is)|is)
                        .filter(|is| self.nodes[is[0]].template_index == child_step.into_index)
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>()
                } else { 
                    step_node.children.iter()
                        .filter(|(template_index, _)| *template_index == child_step.into_index)
                        .map(|(template_index, c)|c)
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>()
                };

                for edge in edges {
                    pending_paths.push((child_step, edge));
                }
            }
        }

        let transform_depends_and_knows = |
            template_list: &[TemplateIndex], 
            found_list: Vec<Vec<CollapseNodeKey>>
        | -> Vec<(T::Identifier, Vec<CollapseNodeKey>)> {
            template_list.iter()
                .zip(found_list)
                .map(|(depend_template_node, nodes)| {

                    let depend_template_node = &template.nodes[*depend_template_node];
                    assert!(nodes.len() > 0, 
                        "No nodes for dependency or knows of node found! \n {:?} tyring to find {:?}", 
                        new_node_template.identifier, depend_template_node.identifier);
                    (depend_template_node.identifier, nodes)
                }).collect::<Vec<_>>()
        };

        let restricts = transform_depends_and_knows(&new_node_template.restricts, restricts);
        let depends = transform_depends_and_knows(&new_node_template.depends, depends);
        let knows = transform_depends_and_knows(&new_node_template.knows, knows);

        (restricts, depends, knows)
    }

    pub fn add_node(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        restricts: Vec<(T::Identifier, Vec<CollapseNodeKey>)>, 
        depends: Vec<(T::Identifier, Vec<CollapseNodeKey>)>, 
        knows: Vec<(T::Identifier, Vec<CollapseNodeKey>)>,
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

        self.push_new_node(new_node_template, restricts, depends, knows, defined_by, child_key, data)
    }
 
    pub fn push_new_node(&mut self, 
        new_node_template: &TemplateNode<T>, 
        restricts: Vec<(T::Identifier, Vec<CollapseNodeKey>)>, 
        depends: Vec<(T::Identifier, Vec<CollapseNodeKey>)>, 
        knows: Vec<(T::Identifier, Vec<CollapseNodeKey>)>,
        defined_by: CollapseNodeKey, 
        child_key: CollapseChildKey, 
        data: NodeDataType<T>
    ) {        
        let index = self.nodes.insert(CollapseNode {
            template_index: new_node_template.index,
            identifier: new_node_template.identifier,
            level: new_node_template.level,
            children: vec![],
            restricts: restricts.clone(),
            depends: depends.clone(),
            knows,
            defined_by,
            data,
            next_reset: CollapseNodeKey::null(),
            undo_data: T::UndoData::default(),
            child_key,
        });
        info!("{:?} Node added {:?}", index, new_node_template.identifier);

        for (_, dependend) in restricts.iter().chain(depends.iter()) {
            for depend in dependend.iter() {
                let children_list = self.nodes[*depend].children.iter_mut()
                    .find(|(template_index, _)| *template_index == new_node_template.index)
                    .map(|(_, c)| c);

                if children_list.is_none() {
                    self.nodes[*depend].children.push((new_node_template.index, vec![index]));
                } else {
                    children_list.unwrap().push(index);
                };
            }
        }
                
        self.pending.push_collpase(new_node_template.level, index);
    }
}

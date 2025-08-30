use std::iter;

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::model::composer::{collapse::collapser::CollapseNode, dependency_tree::DependencyTree, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex, TemplateNode}};

use super::{collapser::{CollapseChildKey, CollapseNodeKey, Collapser, CreateDefinesOperation, NodeDataType}, number_space::NumberSet, position_space::PositionSet};


impl Collapser {

    /*
    pub fn create_defined(&mut self, opperation: CreateDefinesOperation, template: &ComposeTemplate) {
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

    pub fn update_defines_n(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, ammount) in template_node.defines.iter().enumerate() {
            let new_template_node = &template.nodes[ammount.template_index];
            
            self.pending.push_create_defined(new_template_node.level, CreateDefinesOperation::CreateN { 
                parent_index: node_index,
                ammount_index: i,
            });
        }
    }

    pub fn create_defined_n(&mut self, parent_index: CollapseNodeKey, ammount_index: usize, template: &ComposeTemplate) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let ammount = &template_node.defines[ammount_index];

        let depends = self.get_restricts_depends_and_knows_for_template(
            parent_index, 
            ammount.template_index,   
            template,
            template_node,
            &ammount.dependecy_tree);

        for _ in 0..ammount.ammount {
            self.add_node(
                ammount.template_index, 
                depends.clone(), 
                parent_index, 
                CollapseChildKey::null(), 
                template); 
        }

    }

    pub fn update_defined_by_number_range(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate, n: usize) {
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

    pub fn create_defined_by_number_range(&mut self, parent_index: CollapseNodeKey, by_value_index: usize, ammount: usize, template: &ComposeTemplate) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let by_value = &template_node.defines_by_value[by_value_index];

        let depends = self.get_restricts_depends_and_knows_for_template(
            parent_index, 
            by_value.template_index,   
            template,
            template_node,
            &by_value.dependecy_tree);

        for _ in 0..ammount {
            self.add_node(
                by_value.template_index, 
                depends.clone(), 
                parent_index, 
                CollapseChildKey::null(), 
                template); 
        }
    }

    pub fn update_defined_by_pos_set<'a>(
        &mut self, 
        node_index: CollapseNodeKey, 
        to_create_children: Vec<CollapseChildKey>, 
        template: &'a ComposeTemplate, 
        template_node: &'a TemplateNode
    ) {

        for (i, by_value) in template_node.defines_by_value.iter().enumerate() {
            let node = &self.nodes[node_index];
            let NodeDataType::PositionSet(pos_set) = &node.data else { unreachable!() };

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
        template: &ComposeTemplate
    ) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let by_value = &template_node.defines_by_value[by_value_index];

        let depends = self.get_restricts_depends_and_knows_for_template(
            parent_index, 
            by_value.template_index,   
            template,
            template_node,
            &by_value.dependecy_tree);

        for new_child in to_create_children {
            self.add_node(
                by_value.template_index, 
                depends.clone(), 
                parent_index, 
                new_child, 
                template); 
        }
    }
    */
  
    pub fn get_depends_for_template<'a>(
        &self, 
        parent_index: CollapseNodeKey,
        new_template_node_index: usize, 
        template: &'a ComposeTemplate,
        template_node: &'a TemplateNode,
        tree: &'a DependencyTree,
    ) -> Vec<(TemplateIndex, Vec<CollapseNodeKey>)> {
        let new_node_template = &template.nodes[new_template_node_index];

        // Contains a list of node indecies matching the template dependency
        let mut depends = iter::repeat_with(|| vec![])
            .take(new_node_template.depends.len())
            .collect::<Vec<_>>();
        
        let mut pending_paths = vec![(&tree.steps[0], parent_index)];
        while let Some((step, index)) = pending_paths.pop() {
            let step_node = &self.nodes[index];

            if let Some(i) = step.leaf {
                depends[i].push(index);
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

    pub fn add_node(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
        defined_by: CollapseNodeKey,
        child_key: CollapseChildKey,
        template: &ComposeTemplate,
    ) {
        let new_node_template = &template.nodes[new_node_template_index];

        let data = match &(&template.nodes[new_node_template_index]).value {
            ComposeTemplateValue::None => NodeDataType::None,
            ComposeTemplateValue::NumberSpace(space) 
                => NodeDataType::NumberSet(NumberSet::from_space(space, &depends, &self)),

            ComposeTemplateValue::PositionSpace(space) 
                => NodeDataType::PositionSpace(PositionSet::from_space(space, &depends, &self)),

            ComposeTemplateValue::Object() => todo!(),
        };

        self.push_new_node(new_node_template, depends, defined_by, child_key, data)
    }
 
    pub fn push_new_node(&mut self, 
        new_node_template: &TemplateNode, 
        depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
        defined_by: CollapseNodeKey, 
        child_key: CollapseChildKey, 
        data: NodeDataType
    ) {        
        let index = self.nodes.insert(CollapseNode {
            template_index: new_node_template.index,
            level: new_node_template.level,
            children: vec![],
            depends: depends.clone(),
            defined_by,
            data,
            next_reset: CollapseNodeKey::null(),
            child_key,
        });
        info!("{:?} Node added {:?}", index, new_node_template.node_id);

        for (_, dependend) in depends.iter() {
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

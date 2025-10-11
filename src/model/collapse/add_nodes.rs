use std::{hint::unreachable_unchecked, iter};

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::{collapse::collapser::CollapseNode, composer::{build::BS, dependency_tree::DependencyPath, template::{ComposeTemplate, TemplateIndex}}, data_types::ammount::AmmountType}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseChildKey, CollapseNodeKey, Collapser, UpdateDefinesOperation, NodeDataType}, number_space::NumberSpace, position_space::PositionSpace};

#[derive(Debug, Clone, Copy)]
pub struct GetValueData<'a> {
    pub defined_by: CollapseNodeKey,
    pub index: CollapseNodeKey,
    pub child_index: CollapseChildKey,
    pub depends: &'a [(TemplateIndex, Vec<CollapseNodeKey>)],
    pub depends_loop: &'a [(TemplateIndex, DependencyPath)],
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub fn push_defined(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, ammount) in template_node.defines.iter().enumerate() {
            let new_template_node = &template.nodes[ammount.template_index];
           
            let operation = match &ammount.t {
                AmmountType::NPer(n) => UpdateDefinesOperation::N { 
                    parent_index: node_index, 
                    defines_index: i, 
                },
                AmmountType::ByPosSpace => UpdateDefinesOperation::ByNode { 
                    parent_index: node_index, 
                    defines_index: i, 
                },
            };

            self.pending.push_create_defined(new_template_node.level, operation);
        }
    }

    pub async fn upadte_defined(
        &mut self, 
        opperation: UpdateDefinesOperation, 
        template: &ComposeTemplate<V2, V3, T, B>,
        state: &mut B
    ) {
        match opperation {
            UpdateDefinesOperation::N { parent_index, defines_index } => {
                self.update_defined_n(parent_index, defines_index, template, state).await;
            },
            UpdateDefinesOperation::ByNode { parent_index, defines_index } => {
                self.update_defined_by_node(parent_index, defines_index, template, state).await;
            },
        }
    }
 
    pub async fn update_defined_n(
        &mut self, 
        parent_index: CollapseNodeKey, 
        defines_index: usize, 
        template: &ComposeTemplate<V2, V3, T, B>,
        state: &mut B,
    ) {

        let parent = &self.nodes[parent_index];
        let parent_template_node = &template.nodes[parent.template_index];
        let ammount = &parent_template_node.defines[defines_index];

        let depends = self.get_depends_from_tree(
            parent_index, 
            ammount.template_index,   
            template,
            parent_template_node,
            &ammount.dependecy_tree);

        let get_value_data = GetValueData {
            defined_by: parent_index,
            child_index: CollapseChildKey::null(),
            depends: &depends,
            depends_loop: &[],
            index: CollapseNodeKey::null(),
        };

        let n = match &ammount.t {
            AmmountType::NPer(n) => {
                let (n, r) = n.get_value(get_value_data, &self);
                assert!(!r, "Cant use recompute when evaluating ammount");
                
                n.to_usize()
            },
            _ => unreachable!()
        };

        let present_children = parent.children.iter()
            .find(|(template_index, _)| *template_index == ammount.template_index)
            .map(|(_, children)| children.as_slice())
            .unwrap_or(&[]);

        let present_children_len = present_children.len(); 
        if present_children_len < n {

            for _ in present_children_len..n {
                self.add_node(
                    ammount.template_index, 
                    depends.clone(), 
                    parent_index, 
                    CollapseChildKey::null(), 
                    template, 
                    state,
                ).await; 
            }

        } else if present_children_len > n {

            for child in present_children.to_owned().into_iter().take(n - present_children_len) {
                self.delete_node(child, template, state);
            }
        } 
    }

    pub async fn update_defined_by_node(
        &mut self, 
        parent_index: CollapseNodeKey, 
        defines_index: usize, 
        template: &ComposeTemplate<V2, V3, T, B>, 
        state: &mut B,
    ) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let ammount = &template_node.defines[defines_index];

        
        let to_create_children= match &node.data {
            NodeDataType::PositionSpace2D(d) => d.get_new_children(),
            NodeDataType::PositionSpace3D(d) => d.get_new_children(),
            _ => panic!("Template Node {:?} is not of Type Position Space Set", node.template_index)
        };

        let to_remove_children = node.children.iter()
            .find(|(template_index, _)| *template_index == ammount.template_index)
            .map(|(_, children)| children)
            .unwrap_or(&vec![])
            .iter()
            .map(|key| (*key, &self.nodes[*key]) )
            .filter(|(_, child)| {
                match &node.data {
                    NodeDataType::PositionSpace2D(d) => !d.is_child_valid(child.child_key),
                    NodeDataType::PositionSpace3D(d) => !d.is_child_valid(child.child_key),

                    // Safety: Enum gets checked by to_create_children 
                    _ => unsafe { unreachable_unchecked() }
                }
            })
            .map(|(key, _)| key )
            .collect::<Vec<_>>();

        if !to_create_children.is_empty() {
            let new_template_node = &template.nodes[ammount.template_index];
            let depends = self.get_depends_from_tree(
                parent_index, 
                ammount.template_index,   
                template,
                template_node,
                &ammount.dependecy_tree);

            for new_child in to_create_children.to_owned() {
                self.add_node(
                    ammount.template_index, 
                    depends.clone(), 
                    parent_index, 
                    new_child, 
                    template, 
                    state,
                ).await; 
            }
        }

        for child_index in to_remove_children {
            self.delete_node(child_index, template, state);
        }
    }
  
    
    pub async fn add_node(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
        defined_by: CollapseNodeKey,
        child_key: CollapseChildKey,
        template: &ComposeTemplate<V2, V3, T, B>,
        state: &mut B,
    ) {
        let new_node_template = &template.nodes[new_node_template_index];
       
        let index = self.nodes.insert(CollapseNode {
            template_index: new_node_template.index,
            level: new_node_template.level,
            children: vec![],
            depends: depends.clone(),
            defined_by,
            data: NodeDataType::Pending,
            next_reset: CollapseNodeKey::null(),
            child_key,
        });

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

        info!("{:?} Node added {:?}", index, new_node_template.node_id);
                
        self.pending.push_collpase(new_node_template.level, index);
    }
}

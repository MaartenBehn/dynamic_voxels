use std::{hint::unreachable_unchecked, iter};

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::{collapse::collapser::CollapseNode, composer::{build::BS, dependency_tree::DependencyPath, template::{AmmountType, ComposeTemplate, ComposeTemplateValue, TemplateIndex}}}, util::{number::Nu, vector::Ve}};

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

        for (i, creates) in template_node.creates.iter().enumerate() {
            let new_template_node = &template.nodes[creates.to_create];
           
            let operation = match &creates.own_ammount_type {
                AmmountType::One => UpdateDefinesOperation::One { 
                    template_index: creates.to_create,
                    parent_index: node_index, 
                },
                AmmountType::PerPosition(_) => UpdateDefinesOperation::Creates { 
                    template_index: creates.to_create,
                    parent_index: node_index, 
                    creates_index: i, 
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
        dbg!(&opperation);

        match opperation {
            UpdateDefinesOperation::One { template_index, parent_index } => {
                self.update_defined_one(template_index, parent_index, template, state).await;
            },
            UpdateDefinesOperation::Creates { template_index, parent_index, creates_index: defines_index } => {
                self.update_defined_by_creates(template_index, parent_index, defines_index, template, state).await;
            },
        }
    }
 
    pub async fn update_defined_one(
        &mut self, 
        template_index: TemplateIndex,
        parent_index: CollapseNodeKey, 
        template: &ComposeTemplate<V2, V3, T, B>,
        state: &mut B,
    ) {
        let template_node = &template.nodes[template_index];
        let parent = &self.nodes[parent_index];
        let parent_template_node = &template.nodes[parent.template_index];

        if parent.children.iter().find(|(i, _)| *i == template_index).is_some() {
            return;
        }

        let depends = self.get_depends(
            parent_index, 
            template,
            parent_template_node,
            template_node);

        self.add_node(
            template_index, 
            depends, 
            parent_index, 
            CollapseChildKey::null(), 
            template, 
            state,
        ).await;    
    }

    pub async fn update_defined_by_creates(
        &mut self, 
        template_index: TemplateIndex,
        parent_index: CollapseNodeKey, 
        creates_index: usize, 
        template: &ComposeTemplate<V2, V3, T, B>, 
        state: &mut B,
    ) {
        let parent_node = &self.nodes[parent_index];
        let parent_template_node = &template.nodes[parent_node.template_index];
        let creates = &parent_template_node.creates[creates_index];
         
        let to_create_children= match &parent_node.data {
            NodeDataType::PositionSpace2D(d) => d.get_new_children(),
            NodeDataType::PositionSpace3D(d) => d.get_new_children(),
            _ => panic!("Template Node {:?} is not of Type Position Space Set", parent_node.template_index)
        };

        let to_remove_children = parent_node.children.iter()
            .find(|(template_index, _)| *template_index == creates.to_create)
            .map(|(_, children)| children)
            .unwrap_or(&vec![])
            .iter()
            .map(|key| (*key, &self.nodes[*key]) )
            .filter(|(_, child)| {
                match &parent_node.data {
                    NodeDataType::PositionSpace2D(d) => !d.is_child_valid(child.child_key),
                    NodeDataType::PositionSpace3D(d) => !d.is_child_valid(child.child_key),

                    // Safety: Enum gets checked by to_create_children 
                    _ => unsafe { unreachable_unchecked() }
                }
            })
            .map(|(key, _)| key )
            .collect::<Vec<_>>();

        if !to_create_children.is_empty() {
            
            let template_node = &template.nodes[template_index];
            let depends = self.get_depends(
                parent_index,    
                template,
                parent_template_node,
                template_node);

            for new_child in to_create_children.to_owned() {
                self.add_node(
                    creates.to_create, 
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
       
        let data = match new_node_template.value {
            ComposeTemplateValue::None => NodeDataType::None,
            ComposeTemplateValue::NumberSpace(_) => NodeDataType::NumberSet(Default::default()),
            ComposeTemplateValue::PositionSpace2D(_) => NodeDataType::PositionSpace2D(Default::default()),
            ComposeTemplateValue::PositionSpace3D(_) =>  NodeDataType::PositionSpace3D(Default::default()),
            ComposeTemplateValue::Build(_) => NodeDataType::Build(B::CollapseValue::default()),
        };

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

use std::{hint::unreachable_unchecked, iter};

use itertools::Itertools;
use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::{collapse::collapser::CollapseNode, composer::build::BS, template::{dependency_tree::DependencyPath, nodes::{Creates, CreatesType}, value::ComposeTemplateValue, ComposeTemplate, TemplateIndex}}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseChildKey, CollapseNodeKey, Collapser, UpdateDefinesOperation, NodeDataType}, number_space::NumberSpace, position_space::PositionSpace};

#[derive(Debug, Clone, Copy)]
pub struct GetValueData<'a> {
    pub defined_by: CollapseNodeKey,
    pub index: CollapseNodeKey,
    pub child_indexs: &'a [(CollapseNodeKey, CollapseChildKey)],
    pub depends: &'a [(TemplateIndex, Vec<CollapseNodeKey>)],
    pub depends_loop: &'a [(TemplateIndex, DependencyPath)],
}

#[derive(Debug, Clone, Copy)]
pub struct GetNewChildrenData<'a> {
    pub defined_by_template_index: TemplateIndex,
    pub defined_by: CollapseNodeKey,
    pub depends: &'a [(TemplateIndex, Vec<CollapseNodeKey>)],
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub fn push_defined(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, creates) in template_node.creates.iter().enumerate() {
            let new_template_node = &template.nodes[creates.to_create];
           
            let operation = match &creates.t {
                CreatesType::One => UpdateDefinesOperation::One { 
                    template_index: creates.to_create,
                    parent_index: node_index, 
                },
                CreatesType::Children => UpdateDefinesOperation::Creates { 
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
            vec![], 
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
        let creates: &Creates = &parent_template_node.creates[creates_index];
        let template_node = &template.nodes[template_index];

        let depends = self.get_depends(
            parent_index,    
            template,
            parent_template_node,
            template_node);
         
        let mut to_create_children_list = vec![];
        let own_to_create_children= self.get_new_children(parent_index); 
        to_create_children_list.push(own_to_create_children.collect_vec());
        
        for other_template_index in creates.others.iter() {
            let other_to_create_children = self.get_dependend_new_children(
                *other_template_index, &depends);
            to_create_children_list.push(other_to_create_children.collect_vec());
        }

        let to_create_children = to_create_children_list.into_iter().multi_cartesian_product();

        let to_remove_children = parent_node.children.iter()
            .find(|(template_index, _)| *template_index == creates.to_create)
            .map(|(_, children)| children)
            .unwrap_or(&vec![])
            .iter()
            .map(|key| (*key, &self.nodes[*key]) )
            .filter(|(_, child)| {

                child.child_keys.iter()
                    .enumerate()
                    .all(|(i, (index, child_key))| 
                        self.is_child_valid(*index, *child_key))
                })
            .map(|(key, _)| key )
            .collect::<Vec<_>>();

        for new_children in to_create_children.to_owned() {
            self.add_node(
                creates.to_create, 
                depends.clone(), 
                parent_index, 
                new_children, 
                template, 
                state,
            ).await; 
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
        child_keys: Vec<(CollapseNodeKey, CollapseChildKey)>,
        template: &ComposeTemplate<V2, V3, T, B>,
        state: &mut B,
    ) {
        let new_node_template = &template.nodes[new_node_template_index];
      
        let value = &template.values[new_node_template.value_index];
        let data = match value {
            ComposeTemplateValue::None => NodeDataType::None,
            ComposeTemplateValue::Number(number_template) => todo!(),
            ComposeTemplateValue::NumberSpace(_) => NodeDataType::NumberSet(Default::default()),
            ComposeTemplateValue::Position2D(position_template) => todo!(),
            ComposeTemplateValue::Position3D(position_template) => todo!(),
            ComposeTemplateValue::PositionSet2D(position_set_template) => todo!(),
            ComposeTemplateValue::PositionSet3D(position_set_template) => todo!(),
            ComposeTemplateValue::PositionSpace2D(position_space_template) => NodeDataType::PositionSpace2D(Default::default()),
            ComposeTemplateValue::PositionSpace3D(position_space_template) => NodeDataType::PositionSpace3D(Default::default()),
            ComposeTemplateValue::Volume2D(volume_template) => todo!(),
            ComposeTemplateValue::Volume3D(volume_template) => todo!(),
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
            child_keys,
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

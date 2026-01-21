use std::{hint::unreachable_unchecked, iter};

use itertools::Itertools;
use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::{collapse::collapser::CollapseNode, template::{dependency_tree::DependencyPath, nodes::{Creates, CreatesType}, value::TemplateValue, Template, TemplateIndex}}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseChildKey, CollapseNodeKey, Collapser, NodeDataType, UpdateDefinesOperation}, external_input::ExternalInput, number_set::NumberSet};

#[derive(Debug, Clone, Copy)]
pub struct GetValueData<'a> {
    pub defined_by: CollapseNodeKey,
    pub depends: &'a [(TemplateIndex, Vec<(CollapseNodeKey, CollapseChildKey)>)],
    pub depends_loop: &'a [(TemplateIndex, DependencyPath)],

    pub external_input: ExternalInput,
}

#[derive(Debug, Clone, Copy)]
pub struct GetNewChildrenData<'a> {
    pub defined_by_template_index: TemplateIndex,
    pub defined_by: CollapseNodeKey,
    pub depends: &'a [(TemplateIndex, Vec<CollapseNodeKey>)],
}

impl Collapser {
    pub fn push_defined(&mut self, node_index: CollapseNodeKey, template: &Template) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, creates) in template_node.creates.iter().enumerate() {
            let new_template_node = &template.nodes[creates.to_create];
            
            self.pending.push_create_defined(new_template_node.level, UpdateDefinesOperation { 
                template_index: creates.to_create,
                parent_index: node_index, 
                creates_index: i,
            });
        }
    }

    pub async fn update_defined(
        &mut self, 
        opperation: UpdateDefinesOperation, 
        template: &Template,
    ) {
        let parent_node = &self.nodes[opperation.parent_index];
        let parent_template_node = &template.nodes[parent_node.template_index];
        let creates: &Creates = &parent_template_node.creates[opperation.creates_index];

        #[cfg(debug_assertions)]
        info!("{:?} Update {:?}", opperation.parent_index, opperation.template_index);

        match creates.t {
            CreatesType::One => {
                self.update_defined_one(
                    opperation.template_index, 
                    opperation.parent_index, 
                    template, 
                ).await;
            },
            CreatesType::Children => {
                self.update_defined_by_creates(
                    opperation.template_index, 
                    opperation.parent_index, 
                    opperation.creates_index, 
                    template, 
                ).await;
            },
        }
    }
 
    pub async fn update_defined_one(
        &mut self, 
        template_index: TemplateIndex,
        parent_index: CollapseNodeKey, 
        template: &Template,
    ) {
        let template_node = &template.nodes[template_index];
        let parent = &self.nodes[parent_index];
        let parent_template_node = &template.nodes[parent.template_index];

        if parent.children.iter().find(|(i, _)| *i == template_index).is_some() {
            return;
        }

        let depends = self.get_depends(
            parent_index,
            CollapseChildKey::null(),
            template,
            parent_template_node,
            template_node);

        self.add_node(
            template_index, 
            depends, 
            parent_index, 
            CollapseChildKey::null(), 
            template).await;    
    }

    pub async fn update_defined_by_creates(
        &mut self, 
        template_index: TemplateIndex,
        parent_index: CollapseNodeKey, 
        creates_index: usize, 
        template: &Template, 
    ) {
        let parent_node = &self.nodes[parent_index];
        let parent_template_node = &template.nodes[parent_node.template_index];
        let creates: &Creates = &parent_template_node.creates[creates_index];
        let template_node = &template.nodes[template_index];

        let to_remove_children = parent_node.children.iter()
            .find(|(template_index, _)| *template_index == creates.to_create)
            .map(|(_, children)| children)
            .unwrap_or(&vec![])
            .iter()
            .map(|(i, _)| (*i, &self.nodes[*i]) )
            .filter(|(_, child)| 
                !self.is_child_valid(parent_index, child.child_key))  
            .map(|(i, _)| i )
            .collect::<Vec<_>>();
        
        for child_key in self.get_new_children(parent_index).collect_vec() {
            let depends = self.get_depends(
                parent_index,   
                child_key,
                template,
                parent_template_node,
                template_node);


            self.add_node(
                creates.to_create, 
                depends, 
                parent_index, 
                child_key, 
                template, 
            ).await; 
        }

        for child_index in to_remove_children {
            self.delete_node(child_index, template);
        }
    }
  
    
    pub async fn add_node(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        depends: Vec<(TemplateIndex, Vec<(CollapseNodeKey, CollapseChildKey)>)>, 
        defined_by: CollapseNodeKey,
        child_key: CollapseChildKey,
        template: &Template,
    ) {
        let new_node_template = &template.nodes[new_node_template_index];
      
        let value = &template.values[new_node_template.value_index];
        let data = match value {
            TemplateValue::None => NodeDataType::None,
            TemplateValue::NumberSet(_) => NodeDataType::NumberSet(Default::default()),
            TemplateValue::PositionSet2D(position_set_template) => NodeDataType::PositionSet2D(Default::default()),
            TemplateValue::PositionSet3D(position_set_template) => NodeDataType::PositionSet3D(Default::default()),
            TemplateValue::PositionPairSet2D(position_pair_set_template) => NodeDataType::PositionPairSet2D(Default::default()),
            TemplateValue::PositionPairSet3D(position_pair_set_template) => NodeDataType::PositionPairSet3D(Default::default()),
            TemplateValue::Voxels(_) => NodeDataType::Voxels(Default::default()), 
            TemplateValue::Mesh(_) => NodeDataType::Mesh(Default::default()), 
            _ => unreachable!()
        };

        let index = self.nodes.insert(CollapseNode {
            template_index: new_node_template.index,
            children: vec![],
            depends: depends.clone(),
            defined_by,
            data,
            next_reset: CollapseNodeKey::null(),
            child_key,
        });
        self.nodes_per_template_index[new_node_template.index].push(index);

        for (_, dependend) in depends.iter() {
            for (depend, depend_child_key) in dependend.iter() {
                let children_list = self.nodes[*depend].children.iter_mut()
                    .find(|(template_index, _)| *template_index == new_node_template.index)
                    .map(|(_, c)| c);

                if children_list.is_none() {
                    self.nodes[*depend].children.push((new_node_template.index, vec![(index, *depend_child_key)]));
                } else {
                    children_list.unwrap().push((index, *depend_child_key));
                };
            }
        }

        #[cfg(debug_assertions)]
        info!("{:?} Node added {:?}", index, new_node_template.index);
                
        self.pending.push_collpase(new_node_template.level, index);
    }
}

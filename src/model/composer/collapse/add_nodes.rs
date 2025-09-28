use std::{hint::unreachable_unchecked, iter};

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::composer::{ammount::{Ammount, AmmountType}, build::{GetCollapseValueArgs, BS}, collapse::collapser::CollapseNode, dependency_tree::DependencyTree, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex, TemplateNode}}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseChildKey, CollapseNodeKey, Collapser, UpdateDefinesOperation, NodeDataType}, number_space::NumberSpace, position_space::PositionSpace};

#[derive(Debug, Clone, Copy)]
pub struct GetValueData<'a> {
    pub defined_by: CollapseNodeKey,
    pub child_index: CollapseChildKey,
    pub depends: &'a [(TemplateIndex, Vec<CollapseNodeKey>)] ,
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

        let depends = self.get_depends_for_template(
            parent_index, 
            ammount.template_index,   
            template,
            parent_template_node,
            &ammount.dependecy_tree);

        let get_value_data = GetValueData {
            defined_by: parent_index,
            child_index: CollapseChildKey::null(),
            depends: &depends,
        };

        let n = match &ammount.t {
            AmmountType::NPer(n) => n.get_value(get_value_data, &self).to_usize(),
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
            let depends = self.get_depends_for_template(
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
  
    pub fn get_depends_for_template<'a>(
        &self, 
        parent_index: CollapseNodeKey,
        new_template_node_index: usize, 
        template: &'a ComposeTemplate<V2, V3, T, B>,
        template_node: &'a TemplateNode<V2, V3, T, B>,
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
      
        let get_value_data = GetValueData {
            defined_by,
            child_index: child_key,
            depends: &depends,
        };

        let data = match &(&template.nodes[new_node_template_index]).value {
            ComposeTemplateValue::None => NodeDataType::None,
            ComposeTemplateValue::NumberSpace(space) 
            => NodeDataType::NumberSet(NumberSpace::from_template(space, get_value_data, &self)),

            ComposeTemplateValue::PositionSpace2D(space) 
            => NodeDataType::PositionSpace2D(PositionSpace::from_template(space, get_value_data, &self)),

            ComposeTemplateValue::PositionSpace3D(space) 
            => NodeDataType::PositionSpace3D(PositionSpace::from_template(space, get_value_data, &self)),


            ComposeTemplateValue::Build(t) 
            => NodeDataType::Build(B::get_collapse_value(GetCollapseValueArgs { 
                template_value: t,
                get_value_data,
                collapser: &self, 
                template, 
                state 
            }).await),
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

use std::iter;

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, log::{debug, info}, OctaResult};
use slotmap::Key;
use tree64::Node;

use crate::{model::composer::{ammount::{Ammount, AmmountType}, build::BS, collapse::collapser::CollapseNode, dependency_tree::DependencyTree, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex, TemplateNode}}, util::{number::Nu, vector::Ve}};

use super::{collapser::{CollapseChildKey, CollapseNodeKey, Collapser, UpdateDefinesOperation, NodeDataType}, number_space::NumberSpace, position_space::PositionSpace};


impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> {
    pub fn push_defined(&mut self, node_index: CollapseNodeKey, template: &ComposeTemplate<V2, V3, T, B>) {
        let node = &self.nodes[node_index];
        let template_node = &template.nodes[node.template_index];

        for (i, ammount) in template_node.defines.iter().enumerate() {
            let new_template_node = &template.nodes[ammount.template_index];
           
            let operation = match ammount.t {
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

    pub fn upadte_defined(&mut self, opperation: UpdateDefinesOperation, template: &ComposeTemplate<V2, V3, T, B>) {
        match opperation {
            UpdateDefinesOperation::N { parent_index, defines_index } => {
                self.update_defined_n(parent_index, defines_index, template);
            },
            UpdateDefinesOperation::ByNode { parent_index, defines_index } => {
                self.update_defined_by_node(parent_index, defines_index, template);
            },
        }
    }
 
    pub fn update_defined_n(
        &mut self, 
        parent_index: CollapseNodeKey, 
        defines_index: usize, 
        template: &ComposeTemplate<V2, V3, T, B>
    ) {

        let parent = &self.nodes[parent_index];
        let parent_template_node = &template.nodes[parent.template_index];
        let ammount = &parent_template_node.defines[defines_index];

        let n = match ammount.t {
            AmmountType::NPer(n) => n.get_value(&parent.depends, &self).to_usize(),
            _ => unreachable!()
        };

        let present_children = parent.children.iter()
            .find(|(template_index, _)| *template_index == ammount.template_index)
            .map(|(_, children)| children.as_slice())
            .unwrap_or(&[]);

        let present_children_len = present_children.len(); 
        if present_children_len < n {
            let depends = self.get_depends_for_template(
                parent_index, 
                ammount.template_index,   
                template,
                parent_template_node,
                &ammount.dependecy_tree);

            for _ in present_children_len..n {
                self.add_node(
                    ammount.template_index, 
                    depends.clone(), 
                    parent_index, 
                    CollapseChildKey::null(), 
                    template); 
            }

        } else if present_children_len > n {

            for child in present_children.to_owned().into_iter().take(n - present_children_len) {
                self.delete_node(child, template);
            }
        } 
    }

    pub fn update_defined_by_node(
        &mut self, 
        parent_index: CollapseNodeKey, 
        defines_index: usize, 
        template: &ComposeTemplate<V2, V3, T, B>, 
    ) {
        let node = &self.nodes[parent_index];
        let template_node = &template.nodes[node.template_index];
        let ammount = &template_node.defines[defines_index];

        
        let (to_create_children, is_valid) = match &node.data {
            NodeDataType::PositionSpace(d) => {
                (d.get_new_children(), |index| {d.is_child_valid(index)})
            },
            _ => panic!("Template Node {:?} is not of Type Position Space Set", node.template_index)
        };

        let to_remove_children = node.children.iter()
            .find(|(template_index, _)| *template_index == ammount.template_index)
            .map(|(_, children)| children)
            .unwrap_or(&vec![])
            .iter()
            .map(|key| (*key, &self.nodes[*key]) )
            .filter(|(_, child)| !is_valid(child.child_key))
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
                    template); 
            }
        }

        for child_index in to_remove_children {
            self.delete_node(child_index, template);
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

    pub fn add_node(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
        defined_by: CollapseNodeKey,
        child_key: CollapseChildKey,
        template: &ComposeTemplate<V2, V3, T, B>,
    ) {
        let new_node_template = &template.nodes[new_node_template_index];

        let data = match &(&template.nodes[new_node_template_index]).value {
            ComposeTemplateValue::None => NodeDataType::None,
            ComposeTemplateValue::NumberSpace(space) 
            => NodeDataType::NumberSet(NumberSpace::from_template(space, &depends, &self)),

            ComposeTemplateValue::PositionSpace(space) 
            => NodeDataType::PositionSpace(PositionSpace::from_template(space, &depends, &self)),

            ComposeTemplateValue::Build(t) 
            => NodeDataType::Build(B::from_template(t, &depends, &self)),
        };

        self.push_new_node(new_node_template, depends, defined_by, child_key, data)
    }
 
    pub fn push_new_node(&mut self, 
        new_node_template: &TemplateNode<V2, V3, T, B>, 
        depends: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
        defined_by: CollapseNodeKey, 
        child_key: CollapseChildKey, 
        data: NodeDataType<V2, V3, T, B>
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

use std::iter;

use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, OctaResult};
use slotmap::Key;

use super::{builder::{BU, IT}, collapse::{CollapseNode, CollapseNodeKey, Collapser, GridData, NodeDataType, NodeOperation, NodeOperationType, NumberData, PosData, PosSetData}, relative_path::LeafType, template::{NodeTemplateValue, TemplateAmmountType, TemplateIndex, TemplateNode}};


impl<'a, I: IT, U: BU> Collapser<'a, I, U> {

    pub fn update_defined(&mut self, node_index: CollapseNodeKey, to_create_template_index: TemplateIndex) -> OctaResult<()> {
        let node = &self.nodes[node_index];
        let template_node = self.get_template_from_node_ref(node);
        let template_ammount = template_node.defines_ammount.iter()
            .find(|ammount| ammount.index == to_create_template_index)
            .ok_or(anyhow!("Node Template to create has no defines ammout in parent"))?;
 
        match template_ammount.typ {
            TemplateAmmountType::N(n) => {
                self.create_n_defined_nodes(node_index, to_create_template_index, n)?;
            },
            TemplateAmmountType::Value => {
                match &node.data {
                    NodeDataType::Pos(_)
                    | NodeDataType::Build
                    | NodeDataType::None => {
                        panic!("TemplateAmmount Value is not allowed on {:?}", &node.data);
                    },
                    NodeDataType::Number(data) => {
                        self.create_n_defined_nodes(node_index, to_create_template_index, data.value as usize)?;
                    },
                    NodeDataType::PosSet(set) => {
                        
                    },

                }
            },
        };
 
        Ok(())
    }

    fn create_n_defined_nodes (&mut self, node_index: CollapseNodeKey, to_create_template_index: TemplateIndex, n: usize) -> OctaResult<()> {
        let node = &self.nodes[node_index];
        let present_children = node.children.iter()
            .find(|(template_index, _)| *template_index == to_create_template_index)
            .map(|(_, children)| children.as_slice())
            .unwrap_or(&[]);

        let present_children_len = present_children.len(); 
        if present_children_len < n {
            let (depends, knows) = self.get_depends_and_knows_for_template(node_index, to_create_template_index)?;

            for _ in present_children_len..n {
                self.add_node(to_create_template_index, depends.clone(), knows.clone(), node_index); 
            }
        } else if present_children_len > n {

            for child in present_children.to_owned().into_iter().take(n) {
                self.delete_node(child, false)?;
            }
        } 
        
        Ok(())
    }

    fn create_pos_set_nodes (&mut self, node_index: CollapseNodeKey, to_create_template_index: TemplateIndex, mut points: Vec<Vec3>) -> OctaResult<()> { 
        let node = &self.nodes[node_index];
        let mut present_children = node.children.iter()
            .find(|(template_index, _)| *template_index == to_create_template_index)
            .map(|(_, children)| children.to_vec())
            .unwrap_or(vec![]);
       
        for i in (0..present_children.len()).rev() {

            let child = present_children[i];
            let pos = if let NodeDataType::Pos(pos) =  &self.nodes[child].data {
                pos.value
            } else {
                bail!("Node that is defined by Pos set is not a Pos");
            };
            if let Some(j) = points.iter()
                .position(|p| *p == pos) {
                // Remove present_child that we want to keep
                present_children.swap_remove(i);

                // Remove point that we already have
                points.swap_remove(j);
            }
        }
        
        // Remove the rest
        for child in present_children {
            self.delete_node(child, false)?;
        }

        // Add nodes for the rest of the points
        let (depends, knows) = self.get_depends_and_knows_for_template(node_index, to_create_template_index)?;
        for point in points {
            self.add_pos(
                to_create_template_index, 
                depends.clone(), 
                knows.clone(), 
                node_index, 
                point
            ); 
        }
        
        Ok(())
    }

    fn get_depends_and_knows_for_template(&mut self, node_index: CollapseNodeKey, to_create_template_index: TemplateIndex) 
    -> OctaResult<(Vec<(I, CollapseNodeKey)>, Vec<(I, CollapseNodeKey)>)> {
        let template_node = self.get_template_from_node_ref(&self.nodes[node_index]);
        let template_ammount = template_node.defines_ammount.iter()
            .find(|ammount| ammount.index == to_create_template_index)
            .ok_or(anyhow!("Node Template to create has no defines ammout in parent"))?;

        let new_node_template = &self.template.nodes[template_ammount.index];
        let tree = &template_ammount.dependecy_tree;

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
        | -> Vec<(I, CollapseNodeKey)> {
            template_list.iter()
                .zip(found_list)
                .map(|(depend_template_node, nodes)| {
                    if *depend_template_node == template_node.index {
                        return (template_node.identifier, node_index);
                    }

                    let depend_template_node = &self.template.nodes[*depend_template_node];
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
        depends: Vec<(I, CollapseNodeKey)>, 
        knows: Vec<(I, CollapseNodeKey)>,
        defined_by: CollapseNodeKey,
    ) {
        let new_node_template = &self.template.nodes[new_node_template_index];

        let data = match &(&self.template.nodes[new_node_template_index]).value {
            NodeTemplateValue::Groupe { .. } => {
                NodeDataType::None
            },
            NodeTemplateValue::NumberRangeHook 
            | NodeTemplateValue::NumberRange { .. } => {
                NodeDataType::Number(NumberData {
                    value: 0,
                    perm_counter: 0,
                })
            },
            NodeTemplateValue::PosHook
            | NodeTemplateValue::Pos { .. } => {
                NodeDataType::Pos(PosData {
                    value: Vec3::ZERO,
                })
            },
            NodeTemplateValue::BuildHook { .. } => {
                NodeDataType::Build
            },
            NodeTemplateValue::PosSetHook
            | NodeTemplateValue::PosSet { .. } => {
                NodeDataType::PosSet(PosSetData {

                })
            }
        };

        self.push_new_node(new_node_template, depends, knows, defined_by, data)
    }

    pub fn add_pos(
        &mut self, 
        new_node_template_index: TemplateIndex, 
        depends: Vec<(I, CollapseNodeKey)>, 
        knows: Vec<(I, CollapseNodeKey)>,
        defined_by: CollapseNodeKey,
        pos: Vec3,
    ) {
        let new_node_template = &self.template.nodes[new_node_template_index];

        let data = NodeDataType::Pos(PosData { value: pos });
        self.push_new_node(new_node_template, depends, knows, defined_by, data)
    }

    pub fn push_new_node(&mut self, new_node_template: &TemplateNode<I>, depends: Vec<(I, CollapseNodeKey)>, knows: Vec<(I, CollapseNodeKey)>, defined_by: CollapseNodeKey, data: NodeDataType) {
        
        let index = self.nodes.insert(CollapseNode {
            template_index: new_node_template.index,
            identfier: new_node_template.identifier,
            level: new_node_template.level,
            children: vec![],
            depends: depends.clone(),
            knows,
            defined_by,
            data,
            next_reset: CollapseNodeKey::null(),
            undo_data: U::default(),
        });
        //info!("{:?} Node added {:?}", index, new_node_template.identifier);

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

        self.insert_opperation(NodeOperation {
            level: new_node_template.level,
            index,
            typ: NodeOperationType::CollapseValue,
        });
    }

}

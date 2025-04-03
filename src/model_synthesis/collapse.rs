
use std::{collections::HashMap, fmt::{Debug, Octal}, iter, marker::PhantomData, task::ready, usize};

use feistel_permutation_rs::{DefaultBuildHasher, OwnedPermutationIterator, Permutation, PermutationIterator};
use octa_force::{anyhow::{anyhow, bail, ensure}, glam::{vec3, IVec3, Vec3}, log::{debug, error, info}, OctaResult};
use slotmap::{new_key_type, Key, SlotMap};


use crate::{model_synthesis::volume::PossibleVolume, vec_csg_tree::tree::VecCSGTree, volume::Volume};

use super::{builder::{BuilderNode, ModelSynthesisBuilder, NodeTemplateValue, BU, IT}, relative_path::{LeafType, RelativePathTree}, template::{TemplateAmmountType, TemplateIndex, TemplateNode, TemplateTree}};

new_key_type! { pub struct CollapseNodeKey; }

#[derive(Debug, Clone)]
pub struct Collapser<'a, I: IT, U: BU, V: Volume> {
    pub template: &'a TemplateTree<I, V>,
    pub nodes: SlotMap<CollapseNodeKey, Node<I, U>>,
    pub pending_operations: Vec<NodeOperation>,
    pub pending_collapse_opperations: Vec<CollapseOperation<I, U>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub struct NodeOperation {
    pub level: usize,
    pub index: CollapseNodeKey,
    pub typ: NodeOperationType,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum NodeOperationType {
    CollapseValue,
    UpdateDefined(TemplateIndex),
}

#[derive(Debug, Clone)]
pub struct Node<I: IT, U: BU> {
    pub template_index: usize,
    pub identfier: I,
    pub children: Vec<(TemplateIndex, Vec<CollapseNodeKey>)>, 
    pub depends: Vec<(I, CollapseNodeKey)>,
    pub knows: Vec<(I, CollapseNodeKey)>,
    pub defined_by: CollapseNodeKey,
    pub data: NodeDataType,
    pub next_reset: CollapseNodeKey,
    pub undo_data: U,
}

#[derive(Debug, Clone)]
pub enum NodeDataType {
    Number(NumberData),
    Pos(PosData),
    Grid(GridData),
    Build,
    None,
}


#[derive(Debug, Clone)]
pub struct NumberData {
    pub value: i32,
    pub perm_counter: usize,
}

#[derive(Debug, Clone)]
pub struct PosData {
    pub value: Vec3,
}

#[derive(Debug, Clone)]
pub struct GridData {
}

#[derive(Debug, Clone)]
pub enum CollapseOperation<I, U> {
    None,
    CollapsePos {
        index: CollapseNodeKey,
    },
    CollapseBuild {
        index: CollapseNodeKey,
        identifier: I, 
    },
    Undo {
        identifier: I,
        undo_data: U,
    }
}

impl<'a, I: IT, U: BU, V: Volume> Collapser<'a, I, U, V> {
    pub fn next(&mut self) -> OctaResult<Option<(CollapseOperation<I, U>, &mut Collapser<'a, I, U, V>)>> { 
        if let Some(collapse_opperation) = self.pending_collapse_opperations.pop() {
            Ok(Some((collapse_opperation, self)))

        } else if let Some(operation) = self.pending_operations.pop() {
            match operation.typ {
                NodeOperationType::CollapseValue => {
                    let opperation = self.collapse_node(operation.index)?;
                    Ok(Some((opperation, self)))    
                },
                NodeOperationType::UpdateDefined(defined_index) => {
                    self.update_defined(operation.index, defined_index)?;
                    Ok(Some((CollapseOperation::None, self)))
                },
            }
        } else {
            Ok(None)
        }
    }

    fn collapse_node(&mut self, node_index: CollapseNodeKey) -> OctaResult<CollapseOperation<I, U>> {
        let node = &mut self.nodes[node_index];
        //info!("{:?} Collapse: {:?}", node_index, node.identfier);

        if let NodeDataType::Pos(pos_data) = &mut node.data {
            self.push_pending_defineds(node_index);
            return Ok(CollapseOperation::CollapsePos { 
                index: node_index
            });
        }

        if let NodeDataType::Build = node.data {

            return Ok(CollapseOperation::CollapseBuild { 
                index: node_index, 
                identifier: node.identfier,
            });
        }

        let node_template = &self.template.nodes[node.template_index];
        if let NodeTemplateValue::NumberRange { min, max, permutation  } = &node_template.value {

            let node = &mut self.nodes[node_index];
            let number_value = node.data.get_number_mut();

            if number_value.perm_counter >= permutation.max() as usize {
                info!("{:?} Resetting Number faild", node_index);

                number_value.perm_counter = 0;
                
                let next_reset = node.next_reset;
                self.reset_node(next_reset)?;

                self.insert_opperation(NodeOperation {
                    index: node_index,
                    level: node_template.level,
                    typ: NodeOperationType::CollapseValue,
                });

                return Ok(CollapseOperation::None);
            }

            let value = permutation.get(number_value.perm_counter as _) as i32 + min;
            number_value.perm_counter += 1;

            number_value.value = value;
            info!("{:?} {:?}: {}", node_index, node.identfier, number_value.value); 
        }

        self.push_pending_defineds(node_index);
        Ok(CollapseOperation::None)
    }

    fn push_pending_defineds(&mut self, node_index: CollapseNodeKey) {
        let node = &self.nodes[node_index];
        let template_node = &self.template.nodes[node.template_index]; 
        for ammount in template_node.defines_ammount.iter() {
            let new_node_template = &self.template.nodes[ammount.index];

            self.insert_opperation(NodeOperation{
                level: new_node_template.level,
                index: node_index,
                typ: NodeOperationType::UpdateDefined(ammount.index),
            }); 
        }
    }

    fn update_defined(&mut self, node_index: CollapseNodeKey, to_create_template_index: TemplateIndex) -> OctaResult<()> {
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
                if let NodeDataType::Number(data) = &node.data {
                    self.create_n_defined_nodes(node_index, to_create_template_index, data.value as usize)?;
                } else {
                    panic!("TemplateAmmount Value is not allowed on {:?}", &node.data);
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

        let data = match &new_node_template.value {
            NodeTemplateValue::Groupe { .. } => {
                NodeDataType::None
            },
            NodeTemplateValue::NumberRange { .. } => {
                NodeDataType::Number(NumberData {
                    value: 0,
                    perm_counter: 0,
                })
            },
            NodeTemplateValue::Pos { .. } => {
                NodeDataType::Pos(PosData {
                    value: Vec3::ZERO,
                })
            },
            NodeTemplateValue::BuildHook { .. } => {
                NodeDataType::Build
            },
            NodeTemplateValue::Grid { .. } => {
                NodeDataType::Grid(GridData {

                })
            }
        };

        let index = self.nodes.insert(Node {
            template_index: new_node_template_index,
            identfier: new_node_template.identifier,
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
                .find(|(template_index, _)| *template_index == new_node_template_index)
                .map(|(_, c)| c);

            if children_list.is_none() {
                self.nodes[depend].children.push((new_node_template_index, vec![index]));
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

    pub fn pos_collapse_failed(&mut self, index: CollapseNodeKey) {

    }

    pub fn collapse_failed(&mut self, index: CollapseNodeKey) -> OctaResult<()> {        
        let node = self.nodes.get(index).expect("Reset CollapseNodeKey not valid!");
        info!("{:?} Collapse Faild {:?}", index, node.identfier);

        let level = self.template.nodes[node.template_index].level;
        
        Self::insert_opperation_unpacked(&mut self.pending_operations, NodeOperation{
            level,
            index,
            typ: NodeOperationType::CollapseValue,
        });

        let mut last = CollapseNodeKey::null();
        for (_, i) in node.depends.to_owned().into_iter().rev() {
            if last.is_null() {
                self.reset_node(i)?;
            } else {
                self.set_next_reset(last, i)?; 
            }

            last = i;
        }

        Ok(())
    }

    fn reset_node(&mut self, node_index: CollapseNodeKey) -> OctaResult<()> {
        let node = self.get_node_ref_from_node_index(node_index)?;
        info!("{:?} Reset {:?}", node_index, node.identfier);

        let node_template = Self::get_template_from_node_ref_unpacked(&self.template, node);
        for child in node.children.iter()
            .filter(|(template_index, _)| node_template.defines_ammount.iter()
                .find(|ammount| ammount.index == *template_index)
                .is_none())
            .map(|(_, c)| c)
            .flatten()
            .copied()
            .collect::<Vec<_>>() {

            self.delete_node(child, true)?;
        }

        let node = self.get_node_ref_from_node_index(node_index)?; 
        self.insert_opperation(NodeOperation {
            index: node_index,
            level: node_template.level,
            typ: NodeOperationType::CollapseValue,
        });

        Ok(())
    }

    fn delete_node(&mut self, node_index: CollapseNodeKey, recreate: bool) -> OctaResult<()> {
        let node = self.nodes.remove(node_index);
        if node.is_none() {
            return Ok(());
        }
        let node = node.unwrap();
        ensure!(!node.defined_by.is_null(), "Trying to delete root node!");

        info!("{:?} Delete {:?}", node_index, node.identfier);

        self.pending_operations = self.pending_operations.iter()
            .filter(|opperation| opperation.index != node_index)
            .copied()
            .collect();

        for (_, depends) in node.depends.iter() {
            let depends_node = self.get_node_mut_from_node_index(*depends);
            if depends_node.is_err() {
                continue;
            }
            let depends_node = depends_node.unwrap();

            let children = depends_node.children.iter_mut()
                .find(|(template_index, _)| *template_index == node.template_index)
                .map(|(_, c)| c)
                .ok_or(anyhow!("When deleting node the template index of the node was not present in the children of a dependency"))?;

            let i = children.iter()
                .position(|t| *t == node_index)
                .ok_or(anyhow!("When deleting node index of the node was not present in the children of a dependency"))?;
            
            children.swap_remove(i);
        }

        self.pending_collapse_opperations.push(CollapseOperation::Undo { 
            identifier: node.identfier, 
            undo_data: node.undo_data,
        });

        for child in node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {

            self.delete_node(*child, recreate)?;
        }

        if recreate {
            let node_template = self.get_template_from_node_ref(&node); 
            if self.has_index(node.defined_by) {

                self.insert_opperation(NodeOperation {
                    index: node.defined_by,
                    level: node_template.level,
                    typ: NodeOperationType::UpdateDefined(node.template_index)
                });
            } 
        }

        return Ok(());
    }

    fn set_next_reset(&mut self, index: CollapseNodeKey, set_to: CollapseNodeKey) -> OctaResult<()> {
        let node = self.get_node_mut_from_node_index(index)?;
        node.next_reset = set_to;

        Ok(())
    }
 
    pub fn set_undo_data(&mut self, index: CollapseNodeKey, data: U) -> OctaResult<()> {
        let node = self.nodes.get_mut(index)
            .ok_or(anyhow!("Index of build node to set data is not valid!"))?;
        
        node.undo_data = data;
        
        Ok(())
    }

}


impl NodeDataType {
    pub fn get_number_mut(&mut self) -> &mut NumberData {
        match self {
            NodeDataType::Number(d) => d,
            _ => unreachable!()
        }
    } 
}


impl<I: IT, V: Volume> TemplateTree<I, V> {
    pub fn get_collaper<U: BU>(&self) -> Collapser<I, U, V> {
        let mut collapser = Collapser{
            template: self,
            nodes: SlotMap::with_key(),
            pending_operations: vec![],
            pending_collapse_opperations: vec![],
        };

        collapser.add_node(0, vec![], vec![], CollapseNodeKey::null());
        collapser
    }
}

impl Ord for NodeOperation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.level.cmp(&self.level)
            .then(other.typ.cmp(&self.typ))
            .then(other.index.cmp(&self.index))
    }
}

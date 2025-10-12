use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use octa_force::glam::Vec3;
use octa_force::log::{self, debug, trace};
use smallvec::{SmallVec, smallvec};
use crate::model::composer::dependency_tree::{get_dependency_tree_and_loop_paths, DependencyTree};
use crate::model::data_types::data_type::ComposeDataType;
use crate::model::data_types::number::NumberTemplate;
use crate::model::data_types::number_space::NumberSpaceTemplate;
use crate::model::data_types::position_set::PositionSetTemplate;
use crate::model::data_types::position_space::PositionSpaceTemplate;
use crate::util::number::Nu;

use crate::util::vector::Ve;
use super::build::{GetTemplateValueArgs, TemplateValueTrait, BS};
use super::dependency_tree::DependencyPath;
use super::nodes::{ComposeNode, ComposeNodeType};
use super::ModelComposer;

pub type TemplateIndex = usize;
pub type OutputIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct ComposeTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub nodes: Vec<TemplateNode<V2, V3, T, B>>,
    pub max_level: usize,
}

#[derive(Debug, Clone)]
pub enum ComposeTemplateValue<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    None,
    NumberSpace(NumberSpaceTemplate<V2, V3, T>),
    PositionSpace2D(PositionSpaceTemplate<V2, V2, V3, T, 2>),
    PositionSpace3D(PositionSpaceTemplate<V3, V2, V3, T, 3>),
    Build(B::TemplateValue)
}

#[derive(Debug, Clone)]
pub struct TemplateNode<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub node_id: NodeId,
    pub index: TemplateIndex,
    pub value: ComposeTemplateValue<V2, V3, T, B>,

    pub level: usize,
    pub creates: SmallVec<[Creates<V2, V3, T>; 4]>,

    pub depends: SmallVec<[TemplateIndex; 4]>,
    pub dependecy_tree: DependencyTree,
    pub depends_loop: SmallVec<[(TemplateIndex, DependencyPath); 4]>,
    
    pub dependend: SmallVec<[TemplateIndex; 4]>,
}

pub struct MakeTemplateData<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub building_template_index: TemplateIndex,
    pub template: &'a ComposeTemplate<V2, V3, T, B>,
    pub ammounts: SmallVec<[AmmountType<V2, V3, T>; 4]>, 
    pub depends: SmallVec<[TemplateIndex; 4]>,
}

#[derive(Debug, Clone)]
pub struct Creates<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub to_create: TemplateIndex,
    pub own_ammount_type: AmmountType<V2, V3, T>,
    pub other_ammounts: SmallVec<[OtherAmmount<V2, V3, T>; 2]>
}

#[derive(Debug, Clone)]
pub struct OtherAmmount<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub other_parent: TemplateIndex,
    pub t: AmmountType<V2, V3, T>,
}

#[derive(Debug, Clone)]
pub enum AmmountType<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    One,
    PerPosition(PositionSetTemplate<V2, V3, T>),
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> {
    pub fn empty() -> Self {
        Self {
            nodes: vec![TemplateNode {
                node_id: NodeId(usize::MAX),
                index: 0,
                value: ComposeTemplateValue::None,
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                dependecy_tree: Default::default(),
            }],
            max_level: 1,
        }
    }

    pub fn new(composer: &ModelComposer<V2, V3, T, B>) -> Self {
        let mut nodes = vec![
            TemplateNode {
                node_id: NodeId(usize::MAX),
                index: 0,
                value: ComposeTemplateValue::None,
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                dependecy_tree: Default::default(),
            }
        ]; 

        nodes.extend(composer.snarl.nodes()
            .map(|node| {
                match &node.t {
                    ComposeNodeType::TemplateNumberSet 
                    | ComposeNodeType::TemplatePositionSet2D
                    | ComposeNodeType::TemplatePositionSet3D => Some(node.id),
                    ComposeNodeType::Build(t) => if B::is_template_node(t) {
                        Some(node.id) 
                    } else {
                        None
                    }
                    _ => {None}
                }
            })
            .flatten()
            .enumerate()
            .map(|(i, node_id)| {
                TemplateNode {
                    node_id: node_id,
                    index: i + 1,
                    value: ComposeTemplateValue::None,
                    depends_loop: smallvec![],
                    depends: smallvec![],
                    dependend: smallvec![],
                    level: 0,
                    creates: smallvec![],
                    dependecy_tree: Default::default(),
                }
            }));

        let mut template = ComposeTemplate {
            nodes,
            max_level: 1,
        };

        // value, depends, defined
        for i in 1..template.nodes.len() {
            let template_node =  &template.nodes[i]; 
            let composer_node = composer.snarl.get_node(template_node.node_id)
                .expect("Composer Node for Template not found");


            let mut data = MakeTemplateData {
                building_template_index: i,
                template: &template,
                ammounts: SmallVec::new(),
                depends: SmallVec::new(),
            };

            let value = match &composer_node.t { 
                ComposeNodeType::TemplatePositionSet2D => {
                    let space = composer.make_pos_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::PositionSpace2D), &mut data);
                    
                    ComposeTemplateValue::PositionSpace2D(space)
                },
                ComposeNodeType::TemplatePositionSet3D => {
                    let space = composer.make_pos_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::PositionSpace3D), &mut data);
                    
                    ComposeTemplateValue::PositionSpace3D(space)
                },
                ComposeNodeType::TemplateNumberSet => {
                    let space = composer.make_number_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::NumberSpace), &mut data);
                    
                    ComposeTemplateValue::NumberSpace(space)
                },
                ComposeNodeType::Build(t) => {
                    let value = B::get_template_value(GetTemplateValueArgs { 
                        compose_type: t, 
                        composer_node, 
                        composer: &composer, 
                    }, &mut data);

                    ComposeTemplateValue::Build(value)
                },
                _ => unreachable!()
            };

            let mut depends = data.depends;
            let mut ammounts = data.ammounts;

            depends.sort();
            depends.dedup();

            ammounts.sort();
            ammounts.dedup();

            if let Some(ammount_type) = ammounts.pop() {
                let creates_index = match &ammount_type {
                    AmmountType::One => unreachable!(),
                    AmmountType::PerPosition(set) => set.get_ammount_hook(),
                };

                let other_ammounts = ammounts.into_iter()
                    .map(|ammount_type| {
                        let defines = match &ammount_type {
                            AmmountType::One => unreachable!(),
                            AmmountType::PerPosition(set) => set.get_ammount_hook(),
                        };

                        OtherAmmount {
                            other_parent: i,
                            t: ammount_type,
                        }
                    })
                    .collect();

                template.nodes[creates_index].creates.push(Creates {
                    to_create: i,
                    own_ammount_type: ammount_type,
                    other_ammounts,
                });
            } else {
                template.nodes[0].creates.push(Creates {
                    to_create: i,
                    own_ammount_type: AmmountType::One,
                    other_ammounts: smallvec![],
                });
                depends.push(0);
            }
            
            let node =  &mut template.nodes[i]; 
            node.depends = depends;
            node.value = value;
        }

        // Levels, cut loops and dependend
        for i in 0..template.nodes.len() {
            if template.nodes[i].level == 0 {
                template.cut_loops(i, vec![]);
            }
            
            for j in 0..template.nodes[i].depends.len() {
                let depends_index = template.nodes[i].depends[j]; 
                template.nodes[depends_index].dependend.push(i);
            }
        }

        // Dependency Tree and Loop Paths
        for i in 0..template.nodes.len() {
            for j in 0..template.nodes[i].creates.len() {
                let new_index = template.nodes[i].creates[j].to_create; 
                let new_node = &template.nodes[new_index];

                let (tree, loop_paths) = get_dependency_tree_and_loop_paths(
                    &template, 
                    i, 
                    &new_node.depends, 
                    &new_node.dependend, 
                    &new_node.depends_loop,
                );
                
                template.nodes[new_index].dependecy_tree = tree;
                template.nodes[new_index].depends_loop = loop_paths;
            }
        }

        template
    }

    fn cut_loops(&mut self, index: usize, mut index_seen: Vec<usize>) -> usize {
        index_seen.push(index);

        trace!("Set level of node {}, index_seen: {:?}", index, &index_seen);

        let node: &mut TemplateNode<V2, V3, T, B> = &mut self.nodes[index];
        
        let mut max_level = 0;
        for (i, depends_index) in node.depends.to_owned().iter().enumerate().rev() {
            trace!("Node {}, depends on {}", index, *depends_index);

            if let Some(_) = index_seen.iter().find(|p| **p == *depends_index) {
                let node: &mut TemplateNode<V2, V3, T, B> = &mut self.nodes[index];

                trace!("Loop found from {} to {:?}", index, depends_index);

                match &mut node.value {
                    ComposeTemplateValue::NumberSpace(number_space_template) => {
                        number_space_template.cut_loop(*depends_index);
                    },
                    ComposeTemplateValue::PositionSpace2D(position_space_template) => {
                        position_space_template.cut_loop(*depends_index)
                    },
                    ComposeTemplateValue::PositionSpace3D(position_space_template) => {
                        position_space_template.cut_loop(*depends_index);
                    },
                    _ => {} 
                }

                node.depends.swap_remove(i);
                node.depends_loop.push((*depends_index, DependencyPath::default()));
 
                continue;
            }

            let mut level = self.nodes[*depends_index].level; 
            if level == 0 {
                level = self.cut_loops(*depends_index, index_seen.to_owned());
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;
        self.max_level = self.max_level.max(node_level);

        node_level
    }

    pub fn get_index_by_out_pin(&self, pin: OutPinId) -> TemplateIndex {
        self.nodes.iter()
            .position(|n| n.node_id == pin.node)
            .expect("No Template Node for node id found")
    }

    pub fn get_index_by_in_pin(&self, pin: InPinId) -> TemplateIndex {
        self.nodes.iter()
            .position(|n| n.node_id == pin.node)
            .expect("No Template Node for node id found")
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn get_input_pin_index_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> usize {
        node.inputs.iter()
            .position(|i|  i.data_type == t)
            .expect(&format!("No Node {:?} input of type {:?}", node.t, t))
    }
 
    pub fn get_input_remote_pin_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> OutPinId {
        self.get_input_remote_pin_by_index(node, self.get_input_pin_index_by_type(node, t))
    }

    pub fn get_input_remote_pin_by_index(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> OutPinId {
        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: index }).remotes;
        if remotes.is_empty() {
            panic!("No node connected to {:?}", node.t);
        }

        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", node.t);
        }

        remotes[0]
    }

    pub fn get_output_pin_index_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> usize {
        node.outputs.iter()
            .position(|i|  i.data_type == t)
            .expect(&format!("No Node {:?} output of type {:?}", node.t, t))
    }

    pub fn get_output_first_remote_pin_by_type(&self, node: &ComposeNode<B::ComposeType>, t: ComposeDataType) -> InPinId {
        self.get_output_first_remote_pin_by_index(node, self.get_output_pin_index_by_type(node, t))
    }

    pub fn get_output_first_remote_pin_by_index(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> InPinId {
        let remotes = self.snarl.out_pin(OutPinId{ node: node.id, output: index }).remotes;
        if remotes.is_empty() {
            panic!("No output node connected to {:?}", node.t);
        }

        remotes[0]
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> PartialEq for AmmountType<V2, V3, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PerPosition(a), Self::PerPosition(b)) => {
                a.get_ammount_hook() == b.get_ammount_hook()
            },
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> Eq for AmmountType<V2, V3, T> {}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> PartialOrd for AmmountType<V2, V3, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (Self::PerPosition(a), Self::PerPosition(b)) => {
                a.get_ammount_hook().cmp(&b.get_ammount_hook())
            },
            _ => self.enum_index().cmp(&other.enum_index()),
        })
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> Ord for AmmountType<V2, V3, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> AmmountType<V2, V3, T> {
    pub fn enum_index(&self) -> u8 {
        match self {
            AmmountType::One => 0,
            AmmountType::PerPosition(_) => 1,
        }
    }
} 

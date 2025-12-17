use std::usize;

use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::log::trace;
use smallvec::{SmallVec, smallvec};

use crate::{model::{composer::{build::{GetTemplateValueArgs, BS}, graph::{self, ComposerGraph, ComposerNodeFlags}, nodes::{ComposeNode, ComposeNodeType}, ModelComposer}, data_types::{data_type::ComposeDataType, number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, position_pair_set::PositionPairSetTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate}, template::{dependency_tree::DependencyPath, nodes::{Creates, CreatesType}, value::VALUE_INDEX_NODE}}, util::{number::Nu, vector::Ve}};

use super::{dependency_tree::get_dependency_tree_and_loop_paths, nodes::TemplateNode, value::{TemplateValue, ValueIndex}, Template, TemplateIndex, TEMPLATE_INDEX_NONE};

pub struct MakeTemplateData<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub building_template_index: TemplateIndex,
    pub template: &'a mut Template<V2, V3, T, B>,
}

pub struct MakeTemplateNodeData {
    pub building_template_index: TemplateIndex,
    pub created_by_node_id: Option<NodeId>,
}

#[derive(Debug)]
pub enum TemplateNodeUpdate {
    Delete(TemplateIndex),
    New{ new: TemplateIndex, parent: TemplateIndex, creates_index: usize, new_level: usize },
    Unchanged{ old: TemplateIndex, new: TemplateIndex },
    Changed{ old: TemplateIndex, new: TemplateIndex, level: usize },
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Template<V2, V3, T, B> {
    pub fn empty() -> Self {
        Self {
            nodes: vec![TemplateNode {
                index: 0,
                value_index: 0,
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                created_by: (TEMPLATE_INDEX_NONE, 0),
                dependecy_tree: Default::default(),
            }],
            values: vec![TemplateValue::None],
            max_level: 1,
            map_node_id: vec![],
        }
    }

    pub fn update(&mut self, graph: &ComposerGraph<V2, V3, T, B>) -> Vec<TemplateNodeUpdate> {
        
        let mut template = Template {
            nodes: vec![
                TemplateNode {
                    index: 0,
                    value_index: 0,
                    depends_loop: smallvec![],
                    depends: smallvec![],
                    dependend: smallvec![],
                    level: 1,
                    creates: smallvec![],
                    created_by: (TEMPLATE_INDEX_NONE, 0),
                    dependecy_tree: Default::default(),
                }
            ],
            values: vec![TemplateValue::None],
            max_level: 1,
            map_node_id: vec![],
        }; 

        for composer_node in graph.snarl.nodes() {
            let node_id = composer_node.id;
            template.enshure_map_size(node_id);
        }
        
        for composer_node in graph.snarl.nodes() {             
            let node_id = composer_node.id;
            
            match &composer_node.t {
                ComposeNodeType::Build(t) => {
                    if B::is_template_node(t) {
                        let mut data = MakeTemplateData {
                            building_template_index: TEMPLATE_INDEX_NONE,
                            template: &mut template,
                        };

                        let node_data = graph.start_template_node(composer_node, &mut data);

                        let value = TemplateValue::Build(B::get_template_value(GetTemplateValueArgs { 
                            compose_type: t, 
                            composer_node,
                            graph,
                        }, &mut data));

                        let value_index = data.add_value(value);
                        node_data.finish_template_node(value_index, &mut data);
                    }
                }
                _ => {}
            };
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

        let mut needs_update = vec![]; 
        for composer_node in graph.snarl.nodes() {  
            let old_index = self.get_template_index_from_node_id(composer_node.id);
            let new_index = template.get_template_index_from_node_id(composer_node.id);
            
            if new_index.is_none() {
                if old_index.is_some() {
                    needs_update.push(TemplateNodeUpdate::Delete(old_index.unwrap()));
                }
                continue;
            }
            let new = new_index.unwrap();

            if old_index.is_none() {
                let node: &TemplateNode = &template.nodes[new];
                
                // If the parent is also new skip this one.
                // if graph.flags.added_nodes.get(node.created_by.0).as_deref().copied().unwrap() {
                //    continue;
                // }

                needs_update.push(TemplateNodeUpdate::New{
                    new,
                    parent: node.created_by.0,
                    creates_index: node.created_by.1,
                    new_level: node.level,
                });
                continue;
            }
            let old = old_index.unwrap();

            if graph.flags.needs_collapse_nodes.get(composer_node.id.0).as_deref().copied().unwrap_or(false) {
                let node: &TemplateNode = &template.nodes[new];

                needs_update.push(TemplateNodeUpdate::Changed{ new, old, level: node.level });
            } else if new != old {
                needs_update.push(TemplateNodeUpdate::Unchanged{ new, old });
            } else {
                
                match composer_node.t {
                    ComposeNodeType::CamPosition => {},
                    _ => {}
                }

            }
        }
 
        (*self) = template;

        needs_update
    }

    fn cut_loops(&mut self, index: usize, mut index_seen: Vec<usize>) -> usize {
        index_seen.push(index);

        trace!("Set level of node {}, index_seen: {:?}", index, &index_seen);

        let node: &mut TemplateNode = &mut self.nodes[index];
        
        let mut max_level = 0;
        for (i, depends_index) in node.depends.to_owned().iter().enumerate().rev() {
            trace!("Node {}, depends on {}", index, *depends_index);

            if let Some(_) = index_seen.iter().find(|p| **p == *depends_index) {
                trace!("Loop found from {} to {:?}", index, depends_index);
                
                let value_index = self.nodes[index].value_index;
                self.cut_loop_inner(value_index, *depends_index);

                let node = &mut self.nodes[index];
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

    pub fn cut_loop_inner(
        &mut self,
        value_index: ValueIndex,
        to_index: usize, 
    ) {
        let value = &mut self.values[value_index];

        match value {
            TemplateValue::None => {},
            TemplateValue::Number(number_template) => {
                match number_template {
                    NumberTemplate::Const(_) => {},
                    NumberTemplate::Hook(hook) => {
                        hook.loop_cut |= hook.template_index == to_index;
                    },
                    NumberTemplate::SplitPosition2D((p, _)) => {
                        let p = *p;
                        self.cut_loop_inner(p, to_index);
                    },
                    NumberTemplate::SplitPosition3D((p, _)) => {
                        let p = *p;
                        self.cut_loop_inner(p, to_index);
                    },
                    NumberTemplate::Position3DTo2D(p) => {
                        let p = *p;
                        self.cut_loop_inner(p, to_index);
                    },
                }
            },
            TemplateValue::NumberSet(number_space_template) => {
                match number_space_template {
                    NumberSpaceTemplate::NumberRange { min, max, step } => {
                        let min = *min;
                        let max = *max;
                        let step = *step;

                        self.cut_loop_inner(min, to_index);
                        self.cut_loop_inner(max, to_index);
                        self.cut_loop_inner(step, to_index);
                    },
                }
            },
            TemplateValue::PositionSpace2D(position_space_template)
            | TemplateValue::PositionSpace3D(position_space_template)=> {
                match position_space_template {
                    PositionSpaceTemplate::Grid(grid_template) => {
                        let volume = grid_template.volume;
                        let spacing = grid_template.spacing;

                        self.cut_loop_inner(volume, to_index);
                        self.cut_loop_inner(spacing, to_index);
                    },
                    PositionSpaceTemplate::LeafSpread(leaf_spread_template) => {
                        let volume = leaf_spread_template.volume;
                        let samples = leaf_spread_template.samples;

                        self.cut_loop_inner(volume, to_index);
                        self.cut_loop_inner(samples, to_index);
                    },
                    PositionSpaceTemplate::Path(path_template) => {
                        let start = path_template.start;
                        let end = path_template.end;
                        let spacing = path_template.spacing;
                        let side_variance = path_template.side_variance;

                        self.cut_loop_inner(start, to_index);
                        self.cut_loop_inner(end, to_index);
                        self.cut_loop_inner(spacing, to_index);
                        self.cut_loop_inner(side_variance, to_index);
                    },
                }
            },
            TemplateValue::Position2D(position_template) => {
                match position_template {
                    PositionTemplate::Const(_) => {},
                    PositionTemplate::FromNumbers(n) => {
                        let n = *n;
                        self.cut_loop_inner(n[0], to_index);
                        self.cut_loop_inner(n[1], to_index);
                    },
                    PositionTemplate::Add((a, b)) => {
                        let (a, b) = (*a, *b);
                        self.cut_loop_inner(a, to_index);
                        self.cut_loop_inner(b, to_index);
                    },
                    PositionTemplate::Sub((a, b)) => {
                        let (a, b) = (*a, *b);
                        self.cut_loop_inner(a, to_index);
                        self.cut_loop_inner(b, to_index);
                    },
                    PositionTemplate::PerPosition(hook) => {
                        hook.loop_cut |= hook.template_index == to_index;
                    },
                    PositionTemplate::PerPair((hook, _)) => {
                        hook.loop_cut |= hook.template_index == to_index;
                    },
                    PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                    PositionTemplate::Cam => {},
                    PositionTemplate::Position2DTo3D((p, n)) => {
                        let (p, n) = (*p, *n);
                        self.cut_loop_inner(p, to_index);
                        self.cut_loop_inner(n, to_index);
                    },
                    PositionTemplate::Position3DTo2D(p) => {
                        let p = *p;
                        self.cut_loop_inner(p, to_index);
                    },
                }
            },
            TemplateValue::Position3D(position_template) => {
                match position_template {
                    PositionTemplate::Const(_) => {},
                    PositionTemplate::Add((a, b)) => {
                        let (a, b) = (*a, *b);
                        self.cut_loop_inner(a, to_index);
                        self.cut_loop_inner(b, to_index);
                    },
                    PositionTemplate::Sub((a, b)) => {
                        let (a, b) = (*a, *b);
                        self.cut_loop_inner(a, to_index);
                        self.cut_loop_inner(b, to_index);
                    },
                    PositionTemplate::FromNumbers(n) => {
                        let n = *n;
                        self.cut_loop_inner(n[0], to_index);
                        self.cut_loop_inner(n[1], to_index);
                        self.cut_loop_inner(n[2], to_index);
                    },
                    PositionTemplate::PerPosition(hook) => {
                        hook.loop_cut |= hook.template_index == to_index;
                    },
                    PositionTemplate::PerPair((hook, _)) => {
                        hook.loop_cut |= hook.template_index == to_index;
                    },
                    PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                    PositionTemplate::Cam => {},
                    PositionTemplate::Position2DTo3D((p, n)) => {
                        let (p, n) = (*p, *n);
                        self.cut_loop_inner(p, to_index);
                        self.cut_loop_inner(n, to_index);
                    },
                    PositionTemplate::Position3DTo2D(p) => {
                        let p = *p;
                        self.cut_loop_inner(p, to_index);
                    },
                }
            },
            TemplateValue::PositionSet2D(position_set_template)
            | TemplateValue::PositionSet3D(position_set_template)=> {
                match position_set_template {
                    PositionSetTemplate::All(space) => {
                        let space = *space;

                        self.cut_loop_inner(space, to_index);
                    },
                }
            },
            TemplateValue::PositionPairSet2D(position_pair_set_template)
            | TemplateValue::PositionPairSet3D(position_pair_set_template) => {
                match position_pair_set_template {
                    PositionPairSetTemplate::ByDistance((space, distance)) => {
                        let space = *space;
                        let distance = *distance;

                        self.cut_loop_inner(space, to_index);
                        self.cut_loop_inner(distance, to_index);
                    },
                }
            },
            TemplateValue::Volume2D(volume_template)
            | TemplateValue::Volume3D(volume_template)=> {
                match volume_template {
                    VolumeTemplate::Sphere { pos, size } => {
                        let pos = *pos;
                        let size = *size;
                        self.cut_loop_inner(pos, to_index);
                        self.cut_loop_inner(size, to_index);
                    },
                    VolumeTemplate::Disk { pos, size, height } => {
                        let (pos, size, height) = (*pos, *size, *height);
                        self.cut_loop_inner(pos, to_index);
                        self.cut_loop_inner(size, to_index);
                        self.cut_loop_inner(height, to_index);
                    },
                    VolumeTemplate::Box { pos, size } => {
                        let pos = *pos;
                        let size = *size;
                        self.cut_loop_inner(pos, to_index);
                        self.cut_loop_inner(size, to_index);
                    },
                    VolumeTemplate::Union { a, b } => {
                        let a = *a;
                        let b = *b;
                        self.cut_loop_inner(a, to_index);
                        self.cut_loop_inner(b, to_index);
                    },
                    VolumeTemplate::Cut { base, cut } => {
                        let base = *base;
                        let cut = *cut;
                        self.cut_loop_inner(base, to_index);
                        self.cut_loop_inner(cut, to_index);
                    },
                }
            },
            TemplateValue::Build(_) => todo!(),
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposerGraph<V2, V3, T, B> {
    pub fn start_template_node<'a>(
        &self, 
        node: &ComposeNode<B::ComposeType>, 
        data: &mut MakeTemplateData<'a, V2, V3, T, B>
    ) -> MakeTemplateNodeData {
        
        let inactive = MakeTemplateNodeData {
            building_template_index: data.building_template_index,
            created_by_node_id: self.get_creates_input_remote_pin(node),
        };

        let template_index = data.template.nodes.len();
 
        data.template.nodes.push(
            TemplateNode {
                index: template_index,
                value_index: VALUE_INDEX_NODE,
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 0,
                creates: smallvec![],
                created_by: (TEMPLATE_INDEX_NONE, 0),
                dependecy_tree: Default::default(),
            }
        );
        data.template.map_node_id[node.id.0].0 = template_index;

        data.building_template_index = template_index;
        inactive
    }
}

impl MakeTemplateNodeData{
    pub fn finish_template_node<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>>(
        self,
        value_index: ValueIndex,
        data: &mut MakeTemplateData<'a, V2, V3, T, B>
    ) -> TemplateIndex {
        let node: &mut TemplateNode = &mut data.template.nodes[data.building_template_index]; 
        node.value_index = value_index;

        if let Some(create_by_node_id) = self.created_by_node_id {
            let create_by_template_index = data.template.map_node_id[create_by_node_id.0].0;
            
            let others = node.depends.iter()
                .filter(|i| **i != create_by_template_index)
                .copied()
                .collect();
            
            let creates_index = data.template.nodes[create_by_template_index].creates.len();
            data.template.nodes[data.building_template_index].created_by = (create_by_template_index, creates_index);

            data.template.nodes[create_by_template_index].creates.push(Creates {
                to_create: data.building_template_index,
                t: CreatesType::Children,
                others,
            });
        } else {
            node.depends.push(0);

            let creates_index = data.template.nodes[0].creates.len();
            data.template.nodes[data.building_template_index].created_by = (0, creates_index);
            
            data.template.nodes[0].creates.push(Creates {
                to_create: data.building_template_index,
                t: CreatesType::One,
                others: Default::default(),
            }); 
        }

        let template_index = data.building_template_index; 
        data.building_template_index = self.building_template_index;

        if data.building_template_index != TEMPLATE_INDEX_NONE {
            data.template.nodes[data.building_template_index].depends.push(template_index);
        } 

        template_index
    }
}

impl<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> MakeTemplateData<'a, V2, V3, T, B> {
    pub fn get_value_index_from_node_id(&mut self, node_id: NodeId) -> Option<ValueIndex> {
        self.template.get_value_index_from_node_id(node_id)
    }

    pub fn set_value(&mut self, node_id: NodeId, value: TemplateValue<V2, V3, T, B>) -> ValueIndex {
        let value_index = self.template.values.len(); 
        self.template.values.push(value);
        self.template.map_node_id[node_id.0].1 = value_index;
        value_index
    }

    
    pub fn add_value(&mut self, value: TemplateValue<V2, V3, T, B>) -> ValueIndex {
        let value_index = self.template.values.len(); 
        self.template.values.push(value);
        value_index
    }


    pub fn finish_template_node(
        &mut self,
        value_index: ValueIndex,
        inactive: MakeTemplateNodeData
    ) -> TemplateIndex {
        let node: &mut TemplateNode = &mut self.template.nodes[self.building_template_index]; 
        node.value_index = value_index;

        if node.depends.is_empty() {
            node.depends.push(0);

            let creates_index = self.template.nodes[0].creates.len();
            self.template.nodes[self.building_template_index].created_by = (0, creates_index);
            
            self.template.nodes[0].creates.push(Creates {
                to_create: self.building_template_index,
                t: CreatesType::One,
                others: smallvec![],
            });
        } else {
            let picked_depend = node.depends[0];
            
            let creates_index = self.template.nodes[picked_depend].creates.len();
            self.template.nodes[self.building_template_index].created_by = (picked_depend, creates_index);

            self.template.nodes[picked_depend].creates.push(Creates {
                to_create: self.building_template_index,
                t: CreatesType::Children,
                others: smallvec![],
            });
        }

        let template_index = self.building_template_index; 
        self.building_template_index = inactive.building_template_index;

        self.template.nodes[self.building_template_index].depends.push(template_index);

        template_index
    }
}

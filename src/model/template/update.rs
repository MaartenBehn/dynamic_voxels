use std::usize;

use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::log::trace;
use smallvec::{SmallVec, smallvec};

use crate::{model::{composer::{build::{GetTemplateValueArgs, BS}, nodes::{ComposeNode, ComposeNodeType}, ModelComposer}, data_types::{data_type::ComposeDataType, number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate}, template::{dependency_tree::DependencyPath, nodes::{Creates, CreatesType}, value::{ValuePerNodeId, VALUE_INDEX_NODE}}}, util::{number::Nu, vector::Ve}};

use super::{dependency_tree::get_dependency_tree_and_loop_paths, nodes::TemplateNode, value::{ComposeTemplateValue, ValueIndex}, ComposeTemplate, TemplateIndex};


pub struct MakeTemplateData<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub building_template_index: TemplateIndex,
    pub template: &'a mut ComposeTemplate<V2, V3, T, B>,
    pub value_per_node_id: &'a mut ValuePerNodeId,
    pub creates: SmallVec<[TemplateIndex; 2]>, 
    pub depends: SmallVec<[TemplateIndex; 4]>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> {
    pub fn empty() -> Self {
        Self {
            nodes: vec![TemplateNode {
                index: 0,
                value_index: 0,
                node_id: NodeId(usize::MAX),
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                dependecy_tree: Default::default(),
            }],
            values: vec![ComposeTemplateValue::None],
            max_level: 1,
        }
    }

    pub fn new(composer: &ModelComposer<V2, V3, T, B>) -> Self {
        let mut nodes = vec![
            TemplateNode {
                index: 0,
                value_index: 0,
                node_id: NodeId(usize::MAX),
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                dependecy_tree: Default::default(),
            }
        ];

        let mut value_per_node_id = ValuePerNodeId::new();


        for composer_node in composer.snarl.nodes() {
            let node_id = composer_node.id;

            value_per_node_id.enshure_size(node_id);
             
            let is_template_node = match &composer_node.t {
                ComposeNodeType::TemplateNumberSet 
                | ComposeNodeType::TemplatePositionSet2D
                | ComposeNodeType::TemplatePositionSet3D => true,
                ComposeNodeType::Build(t) => B::is_template_node(t),
                _ => false
            };

            if is_template_node {
                let i = nodes.len();
                nodes.push(
                    TemplateNode {
                        index: i,
                        value_index: VALUE_INDEX_NODE,
                        node_id,
                        depends_loop: smallvec![],
                        depends: smallvec![],
                        dependend: smallvec![],
                        level: 0,
                        creates: smallvec![],
                        dependecy_tree: Default::default(),
                    }
                );
            }
        }

        let mut template = ComposeTemplate {
            nodes,
            values: vec![ComposeTemplateValue::None],
            max_level: 1,
        };

        // value, depends, defined
        for i in 1..template.nodes.len() {
            let node_id = template.nodes[i].node_id;
            let composer_node = composer.snarl.get_node(node_id)
                .expect("Composer Node for Template not found");


            let mut data = MakeTemplateData {
                building_template_index: i,
                template: &mut template,
                value_per_node_id: &mut value_per_node_id,
                creates: SmallVec::new(),
                depends: SmallVec::new(),
            };

            let mut only_one = false;
            let value_index = match &composer_node.t { 
                ComposeNodeType::TemplatePositionSet2D
                | ComposeNodeType::TemplatePositionSet3D => {
                    composer.make_pos_space(
                        composer.get_input_remote_pin_by_index(composer_node, 0), &mut data)
                },
                ComposeNodeType::TemplateNumberSet => {
                    composer.make_number_space(
                        composer.get_input_remote_pin_by_index(composer_node, 0), &mut data)
                },
                ComposeNodeType::Build(t) => {
                    let value = B::get_template_value(GetTemplateValueArgs { 
                        compose_type: t, 
                        composer_node, 
                        composer: &composer, 
                    }, &mut data);

                    only_one = true;
                    data.set_value(node_id, ComposeTemplateValue::Build(value))
                },
                _ => unreachable!()
            };

            let mut depends = data.depends;
            let mut creates = data.creates;

            depends.sort();
            depends.dedup();

            creates.sort();
            creates.dedup();

            if let Some(creates_index) = creates.pop() {
                //depends.extend(creates.iter().copied());
                template.nodes[creates_index].creates.push(Creates {
                    to_create: i,
                    t: CreatesType::Children,
                    others: creates,
                });
            } else {
                if depends.is_empty() || only_one {
                    depends.push(0);
                    template.nodes[0].creates.push(Creates {
                        to_create: i,
                        t: CreatesType::One,
                        others: smallvec![],
                    });
                } else {
                    template.nodes[depends[0]].creates.push(Creates {
                        to_create: i,
                        t: CreatesType::One,
                        others: smallvec![],
                    });
                }
            }
            
            let node =  &mut template.nodes[i]; 
            node.depends = depends;
            node.value_index = value_index;
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

        //dbg!(&template);

        template
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
            ComposeTemplateValue::None => {},
            ComposeTemplateValue::Number(number_template) => {
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
                }
            },
            ComposeTemplateValue::NumberSpace(number_space_template) => {
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
            ComposeTemplateValue::PositionSpace2D(position_space_template)
            | ComposeTemplateValue::PositionSpace3D(position_space_template)=> {
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
            ComposeTemplateValue::Position2D(position_template) => {
                match position_template {
                    PositionTemplate::Const(_) => {},
                    PositionTemplate::FromNumbers(n) => {
                        let n = *n;
                        self.cut_loop_inner(n[0], to_index);
                        self.cut_loop_inner(n[1], to_index);
                    },
                    PositionTemplate::PerPosition(n) => {
                        let n = *n;
                        self.cut_loop_inner(n, to_index);
                    },
                    PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                }
            },
            ComposeTemplateValue::Position3D(position_template) => {
                match position_template {
                    PositionTemplate::Const(_) => {},
                    PositionTemplate::FromNumbers(n) => {
                        let n = *n;
                        self.cut_loop_inner(n[0], to_index);
                        self.cut_loop_inner(n[1], to_index);
                        self.cut_loop_inner(n[2], to_index);
                    },
                    PositionTemplate::PerPosition(n) => {
                        let n = *n;
                        self.cut_loop_inner(n, to_index);
                    },
                    PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                }
            },
            ComposeTemplateValue::PositionSet2D(position_set_template)
            | ComposeTemplateValue::PositionSet3D(position_set_template)=> {
                match position_set_template {
                    PositionSetTemplate::Hook(hook) => {
                        hook.loop_cut |= hook.template_index == to_index;
                    },
                    PositionSetTemplate::T2Dto3D(position_set2_dto3_dtemplate) => todo!(),
                }
            },
            ComposeTemplateValue::Volume2D(volume_template)
            | ComposeTemplateValue::Volume3D(volume_template)=> {
                match volume_template {
                    VolumeTemplate::Sphere { pos, size } => {
                        let pos = *pos;
                        let size = *size;
                        self.cut_loop_inner(pos, to_index);
                        self.cut_loop_inner(size, to_index);
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
                    VolumeTemplate::SphereUnion { position_set, size } => {
                        let position_set = *position_set;
                        let size = *size;
                        self.cut_loop_inner(position_set, to_index);
                        self.cut_loop_inner(size, to_index);
                    },
                }
            },
            ComposeTemplateValue::Build(_) => todo!(),
        }
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
 
    pub fn get_output_first_remote_pin_by_index(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> InPinId {
        let remotes = self.snarl.out_pin(OutPinId{ node: node.id, output: index }).remotes;
        if remotes.is_empty() {
            panic!("No output node connected to {:?}", node.t);
        }

        remotes[0]
    }
}

impl<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> MakeTemplateData<'a, V2, V3, T, B> {
    pub fn set_value(&mut self, node_id: NodeId, value: ComposeTemplateValue<V2, V3, T, B>) -> ValueIndex {
        let value_index = self.template.values.len(); 
        self.template.values.push(value);
        self.value_per_node_id.set_value(node_id, value_index);
        value_index
    }

    pub fn add_value(&mut self, value: ComposeTemplateValue<V2, V3, T, B>) -> ValueIndex {
        let value_index = self.template.values.len(); 
        self.template.values.push(value);
        value_index
    }
}

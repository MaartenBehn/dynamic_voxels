use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::log::trace;
use smallvec::{SmallVec, smallvec};

use crate::{model::{composer::{build::{GetTemplateValueArgs, BS}, nodes::{ComposeNode, ComposeNodeType}, ModelComposer}, data_types::data_type::ComposeDataType, template::{dependency_tree::DependencyPath, nodes::{Creates, CreatesType}, value::VALUE_INDEX_NODE}}, util::{number::Nu, vector::Ve}};

use super::{dependency_tree::get_dependency_tree_and_loop_paths, nodes::TemplateNode, value::{ComposeTemplateValue, ValueIndex}, ComposeTemplate, TemplateIndex};


pub struct MakeTemplateData<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub building_template_index: TemplateIndex,
    pub template: &'a ComposeTemplate<V2, V3, T, B>,
    pub creates: SmallVec<[TemplateIndex; 2]>, 
    pub depends: SmallVec<[TemplateIndex; 4]>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> {
    pub fn empty() -> Self {
        Self {
            nodes: vec![TemplateNode {
                index: 0,
                value_index: VALUE_INDEX_NODE,
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
                index: 0,
                value_index: VALUE_INDEX_NODE,
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                dependecy_tree: Default::default(),
            }
        ];

        for composer_node in composer.snarl.nodes() {
            let node_id = composer_node.id;
            let value_index: ValueIndex = node_id.0;
 
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
                        value_index,
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
            values: vec![],
            max_level: 1,
        };

        // value, depends, defined
        for i in 1..template.nodes.len() {
            let value_index = template.nodes[i].value_index; 
            let composer_node = composer.snarl.get_node(NodeId(value_index))
                .expect("Composer Node for Template not found");


            let mut data = MakeTemplateData {
                building_template_index: i,
                template: &mut template,
                creates: SmallVec::new(),
                depends: SmallVec::new(),
            };

            match &composer_node.t { 
                ComposeNodeType::TemplatePositionSet2D
                | ComposeNodeType::TemplatePositionSet3D => {
                    composer.make_pos_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::PositionSpace2D), &mut data);
                },
                ComposeNodeType::TemplateNumberSet => {
                    composer.make_number_space(
                        composer.get_input_remote_pin_by_type(composer_node, ComposeDataType::NumberSpace), &mut data);
                },
                ComposeNodeType::Build(t) => {
                    let value = B::get_template_value(GetTemplateValueArgs { 
                        compose_type: t, 
                        composer_node, 
                        composer: &composer, 
                    }, &mut data);

                    template.set_value(value_index, ComposeTemplateValue::Build(value));
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
                if depends.is_empty() {
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

        dbg!(&template);

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

                match &mut node.value_index {
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

use std::usize;

use egui_snarl::{InPinId, NodeId, OutPinId};
use octa_force::log::{self, trace};
use smallvec::{SmallVec, smallvec};

use crate::{model::{composer::{ModelComposer, graph::{self, ComposerGraph}, nodes::{ComposeNode, ComposeNodeType}}, data_types::{data_type::ComposeDataType, number::NumberValue, number_space::NumberSpaceValue, position::PositionValue, position_pair_set::PositionPairSetValue, position_set::PositionSetValue, position_space::PositionSpaceValue, volume::VolumeValue}, template::{TEMPLATE_INDEX_NONE, Template, TemplateIndex, dependency_tree::{DependencyPath, get_dependency_tree_and_loop_paths}, nodes::{Creates, CreatesType, TemplateNode}, value::{TemplateValue, VALUE_INDEX_NODE, ValueIndex}, value_hook_iterator::ValueHooksIterator}}, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

pub struct MakeTemplateData<'a> {
    pub building_template_index: TemplateIndex,
    pub template: &'a mut Template,
    pub palette: &'a mut SharedPalette,
    pub map_node_id: &'a mut Vec<(TemplateIndex, ValueIndex)>,
}

pub struct MakeTemplateNodeData {
    pub building_template_index: TemplateIndex,
    pub created_by_node_id: Option<NodeId>,
}

#[derive(Debug)]
pub enum TemplateNodeUpdate {
    Delete(TemplateIndex),
    New { new: TemplateIndex, parent: TemplateIndex, creates_index: usize, new_level: usize },
    Changed { old: TemplateIndex, new: TemplateIndex, level: usize },
    UpdateIndex { old: TemplateIndex, new: TemplateIndex },
    None(TemplateIndex),
}

impl ComposerGraph { 
    pub fn make_template(&self, palette: &mut SharedPalette) -> Template {
        
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
        }; 

        let mut map_node_id = vec![];
        for composer_node in self.snarl.nodes() {             
            let node_id = composer_node.id;

            let mut data = MakeTemplateData {
                building_template_index: TEMPLATE_INDEX_NONE,
                template: &mut template,
                palette,
                map_node_id: &mut map_node_id,
            };

            match &composer_node.t {
                ComposeNodeType::Voxels => { 
                    self.make_voxels(composer_node, &mut data);                    
                },
                ComposeNodeType::Mesh => {
                    self.make_mesh(composer_node, &mut data);                    
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

        template
    }
}

impl ComposerGraph {
    pub fn start_template_node<'a>(
        &self, 
        node: &ComposeNode, 
        data: &mut MakeTemplateData<'a>
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

        data.enshure_map_size(node.id);
        data.map_node_id[node.id.0].0 = template_index;

        data.building_template_index = template_index;
        inactive
    }
}

impl MakeTemplateNodeData{
    pub fn finish_template_node<'a>(
        self,
        value_index: ValueIndex,
        data: &mut MakeTemplateData<'a>
    ) -> TemplateIndex {
        let node: &mut TemplateNode = &mut data.template.nodes[data.building_template_index]; 
        node.value_index = value_index;

        if let Some(create_by_node_id) = self.created_by_node_id {
            let create_by_template_index = data.map_node_id[create_by_node_id.0].0;

            if !node.depends.contains(&create_by_template_index) {
                node.depends.push(create_by_template_index);
            }
            
            let creates_index = data.template.nodes[create_by_template_index].creates.len();
            data.template.nodes[data.building_template_index].created_by = (create_by_template_index, creates_index);

            data.template.nodes[create_by_template_index].creates.push(Creates {
                to_create: data.building_template_index,
                t: CreatesType::Children,
            });
        } else {
            node.depends.push(0);

            let creates_index = data.template.nodes[0].creates.len();
            data.template.nodes[data.building_template_index].created_by = (0, creates_index);
            
            data.template.nodes[0].creates.push(Creates {
                to_create: data.building_template_index,
                t: CreatesType::One,
            }); 
        }

        let template_index = data.building_template_index; 
        data.building_template_index = self.building_template_index;

        if data.building_template_index != TEMPLATE_INDEX_NONE {
            let depends = &mut data.template.nodes[data.building_template_index].depends; 

            if !depends.contains(&template_index) {
                depends.push(template_index);
            }
        } 

        template_index
    }
}

impl<'a> MakeTemplateData<'a> {
    pub fn enshure_map_size(&mut self, node_id: NodeId) {
        if node_id.0 >= self.map_node_id.len() {
            self.map_node_id.resize(node_id.0 + 1, (TEMPLATE_INDEX_NONE, VALUE_INDEX_NODE));
        }
    }

    
    pub fn get_template_index_from_node_id(&self, node_id: NodeId) -> Option<TemplateIndex> { 
        if self.map_node_id.len() <= node_id.0 {
            return None;
        }

        if self.map_node_id[node_id.0].0 != TEMPLATE_INDEX_NONE {
            Some(self.map_node_id[node_id.0].0)
        } else {
            None
        }
    }

    pub fn get_value_index_from_node_id(&self, node_id: NodeId) -> Option<ValueIndex> {
        if self.map_node_id.len() <= node_id.0 {
            return None;
        }

        if self.map_node_id[node_id.0].1 != VALUE_INDEX_NODE {
            Some(self.map_node_id[node_id.0].1)
        } else {
            None
        }
    }
   
    pub fn set_value(&mut self, node_id: NodeId, value: TemplateValue) -> ValueIndex {
        let value_index = self.template.values.len(); 
        self.template.values.push(value);

        self.enshure_map_size(node_id);
        self.map_node_id[node_id.0].1 = value_index;
        value_index
    }

    
    pub fn add_value(&mut self, value: TemplateValue) -> ValueIndex {
        let value_index = self.template.values.len(); 
        self.template.values.push(value);
        value_index
    }

    pub fn add_depends_of_value(&mut self, value_index: ValueIndex) {
        if self.building_template_index == TEMPLATE_INDEX_NONE {
            return;
        }

        for hook in ValueHooksIterator::new(&mut self.template.values, value_index) {
            let depends = &mut self.template.nodes[self.building_template_index].depends; 

            if !depends.contains(&hook.template_index) {
                depends.push(hook.template_index);
            }
        }
    }
}

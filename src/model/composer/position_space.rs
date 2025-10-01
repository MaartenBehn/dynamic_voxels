use egui_snarl::OutPinId;
use itertools::Itertools;
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::collapse::CollapseChildKey, util::{number::Nu, vector::Ve}};

use super::{build::BS, data_type::ComposeDataType, nodes::ComposeNodeType, number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, template::{ComposeTemplate, TemplateIndex}, volume::VolumeTemplate, ModelComposer};


#[derive(Debug, Clone)]
pub enum PositionSpaceTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    Grid(GridTemplate<V, V2, V3, T, D>),
    LeafSpread(LeafSpreadTemplate<V, V2, V3, T, D>),
    Path(PathTemplate<V, V2, V3, T, D>)
}

#[derive(Debug, Clone)]
pub struct GridTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    pub volume: VolumeTemplate<V, V2, V3, T, D>,
    pub spacing: NumberTemplate<V2, V3, T>,
}

#[derive(Debug, Clone)]
pub struct LeafSpreadTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    pub volume: VolumeTemplate<V, V2, V3, T, D>,
    pub samples: NumberTemplate<V2, V3, T>,
}

#[derive(Debug, Clone)]
pub struct PathTemplate<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> {
    pub spacing: NumberTemplate<V2, V3, T>,
    pub side_variance: PositionTemplate<V, V2, V3, T, D>,
    pub start: PositionTemplate<V, V2, V3, T, D>,
    pub end: PositionTemplate<V, V2, V3, T, D>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_pos_space<V: Ve<T, D>, const D: usize>(
        &self, 
        pin: OutPinId,
        building_template_index: TemplateIndex,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> PositionSpaceTemplate<V, V2, V3, T, D> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::Grid3D => {
                assert_eq!(D, 3);
                
                let grid = GridTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume3D), building_template_index, template),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), building_template_index, template),
                };
                PositionSpaceTemplate::Grid(grid)
            },
            ComposeNodeType::Grid2D => {
                assert_eq!(D, 2);
                
                let grid = GridTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume2D), building_template_index, template),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), building_template_index, template),                
                };
                PositionSpaceTemplate::Grid(grid)
            },
            ComposeNodeType::LeafSpread3D => {
                assert_eq!(D, 3);
                
                let leaf_spread = LeafSpreadTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume3D), building_template_index, template),
                    samples: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), building_template_index, template),                
                };
                PositionSpaceTemplate::LeafSpread(leaf_spread)
            },
            ComposeNodeType::LeafSpread2D => {
                assert_eq!(D, 2);
                
                let leaf_spread = LeafSpreadTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume2D), building_template_index, template),
                    samples: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), building_template_index, template),                
                };
                PositionSpaceTemplate::LeafSpread(leaf_spread)
            },
            ComposeNodeType::Path3D => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, building_template_index, template),
                    end: self.make_position(node, 1, building_template_index, template),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), building_template_index, template),
                    side_variance: self.make_position(node, 3, building_template_index, template),
                };
                PositionSpaceTemplate::Path(path)
            },
            ComposeNodeType::Path2D => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, building_template_index, template),
                    end: self.make_position(node, 1, building_template_index, template),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), building_template_index, template),
                    side_variance: self.make_position(node, 3, building_template_index, template),
                };
                PositionSpaceTemplate::Path(path)
            },

            _ => unreachable!(),
        }
    }
}

impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> PositionSpaceTemplate<V, V2, V3, T, D> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match &self {
            PositionSpaceTemplate::Grid(grid_volume_data) => {
                grid_volume_data.volume.get_dependend_template_nodes()
                    .chain(grid_volume_data.spacing.get_dependend_template_nodes())
                    .collect_vec()
            },
            PositionSpaceTemplate::LeafSpread(leaf_spread_template) => {
                leaf_spread_template.volume.get_dependend_template_nodes()
                    .collect_vec()
            },
            PositionSpaceTemplate::Path(path) => {
                path.start.get_dependend_template_nodes()
                    .chain(path.end.get_dependend_template_nodes())
                    .chain(path.spacing.get_dependend_template_nodes())
                    .chain(path.side_variance.get_dependend_template_nodes())
                    .collect_vec()
            }
        }.into_iter()  
    }

    pub fn cut_loop(&mut self, to_index: usize) {
        match self {
            PositionSpaceTemplate::Grid(grid_template) => {
                grid_template.volume.cut_loop(to_index);
                grid_template.spacing.cut_loop(to_index);
            },
            PositionSpaceTemplate::LeafSpread(leaf_spread_template) => {
                leaf_spread_template.volume.cut_loop(to_index);
                leaf_spread_template.samples.cut_loop(to_index);
            },
            PositionSpaceTemplate::Path(path_template) => {
                path_template.spacing.cut_loop(to_index);
                path_template.start.cut_loop(to_index);
                path_template.end.cut_loop(to_index);
                path_template.side_variance.cut_loop(to_index);
            },
        }
    }

}

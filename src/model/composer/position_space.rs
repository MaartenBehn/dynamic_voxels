use egui_snarl::OutPinId;
use itertools::Itertools;
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::{collapse::CollapseChildKey}};

use super::{data_type::ComposeDataType, nodes::ComposeNodeType, number_space::NumberSpace, primitive::{Number, Position2D}, template::{ComposeTemplate, TemplateIndex}, volume_2d::Volume2D, volume_3d::Volume3D, ModelComposer};


#[derive(Debug, Clone)]
pub enum PositionSpace {
    GridInVolume(GridVolumeData),
    GridOnPlane(GridOnPlaneData),
    Path(Path)
}

#[derive(Debug, Clone)]
pub struct GridVolumeData {
    pub volume: Volume3D,
    pub spacing: Number,
}

#[derive(Debug, Clone)]
pub struct GridOnPlaneData {
    pub volume: Volume2D,
    pub spacing: Number,
    pub height: Number,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub spacing: Number,
    pub side_variance: Position2D,
    pub start: Position2D,
    pub end: Position2D,
}


impl ModelComposer {
    pub fn make_pos_space(&self, pin: OutPinId, template: &ComposeTemplate) -> PositionSpace {
        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::GridInVolume => {
                
                let grid = GridVolumeData {
                    volume: self.make_volume_3d(self.get_input_node_by_type(node, ComposeDataType::Volume3D), template),
                    spacing: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template),
                };
                PositionSpace::GridInVolume(grid)
            },
            ComposeNodeType::GridOnPlane => {
                
                let grid = GridOnPlaneData {
                    volume: self.make_volume_2d(self.get_input_node_by_type(node, ComposeDataType::Volume2D), template),
                    spacing: self.make_number(node, 1, template),
                    height: self.make_number(node, 2, template),
                };
                PositionSpace::GridOnPlane(grid)
            },
            ComposeNodeType::Path => {
                
                let path = Path {
                    start: self.make_position2d(node, 0, template),
                    end: self.make_position2d(node, 1, template),
                    spacing: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template),
                    side_variance: self.make_position2d(node, 3, template),
                };
                PositionSpace::Path(path)
            },

            _ => unreachable!(),
        }
    }
}

impl PositionSpace {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match &self {
            PositionSpace::GridInVolume(grid_volume_data) => {
                grid_volume_data.volume.get_dependend_template_nodes()
                    .chain(grid_volume_data.spacing.get_dependend_template_nodes())
                    .collect_vec()
            },
            PositionSpace::GridOnPlane(grid_on_plane_data) => {
                grid_on_plane_data.volume.get_dependend_template_nodes()
                    .chain(grid_on_plane_data.spacing.get_dependend_template_nodes())
                    .collect_vec()
            },
            PositionSpace::Path(path) => {
                path.start.get_dependend_template_nodes()
                    .chain(path.end.get_dependend_template_nodes())
                    .chain(path.spacing.get_dependend_template_nodes())
                    .chain(path.side_variance.get_dependend_template_nodes())
                    .collect_vec()
            }      
        }.into_iter()  
    }
}

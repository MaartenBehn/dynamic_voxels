use egui_snarl::OutPinId;
use itertools::Itertools;
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::generation::collapse::CollapseChildKey, util::{number::Nu, vector::Ve}};

use super::{build::BS, data_type::ComposeDataType, nodes::ComposeNodeType, number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, template::{ComposeTemplate, TemplateIndex}, volume::VolumeTemplate, ModelComposer};


#[derive(Debug, Clone)]
pub enum PositionSpaceTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    GridInVolume(GridVolumeTemplate<V2, V3, T>),
    GridOnPlane(GridOnPlaneTemplate<V2, V3, T>),
    Path(PathTemplate<V2, V3, T>)
}

#[derive(Debug, Clone)]
pub struct GridVolumeTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub volume: VolumeTemplate<V3, V2, V3, T, 3>,
    pub spacing: NumberTemplate<V2, V3, T>,
}

#[derive(Debug, Clone)]
pub struct GridOnPlaneTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub volume: VolumeTemplate<V2, V2, V3, T, 2>,
    pub spacing: NumberTemplate<V2, V3, T>,
    pub height: NumberTemplate<V2, V3, T>,
}

#[derive(Debug, Clone)]
pub struct PathTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    pub spacing: NumberTemplate<V2, V3, T>,
    pub side_variance: PositionTemplate<V2, V2, V3, T, 2>,
    pub start: PositionTemplate<V2, V2, V3, T, 2>,
    pub end: PositionTemplate<V2, V2, V3, T, 2>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_pos_space(
        &self, 
        pin: OutPinId, 
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> PositionSpaceTemplate<V2, V3, T> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::GridInVolume => {
                
                let grid = GridVolumeTemplate {
                    volume: self.make_volume(self.get_input_pin_by_type(node, ComposeDataType::Volume3D), template),
                    spacing: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template),
                };
                PositionSpaceTemplate::GridInVolume(grid)
            },
            ComposeNodeType::GridOnPlane => {
                
                let grid = GridOnPlaneTemplate {
                    volume: self.make_volume(self.get_input_pin_by_type(node, ComposeDataType::Volume2D), template),
                    spacing: self.make_number(node, 1, template),
                    height: self.make_number(node, 2, template),
                };
                PositionSpaceTemplate::GridOnPlane(grid)
            },
            ComposeNodeType::Path => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, template),
                    end: self.make_position(node, 1, template),
                    spacing: self.make_number(node, self.get_input_index_by_type(node, ComposeDataType::Number(None)), template),
                    side_variance: self.make_position(node, 3, template),
                };
                PositionSpaceTemplate::Path(path)
            },

            _ => unreachable!(),
        }
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> PositionSpaceTemplate<V2, V3, T> {
    pub fn get_dependend_template_nodes(&self) -> impl Iterator<Item = TemplateIndex> {
        match &self {
            PositionSpaceTemplate::GridInVolume(grid_volume_data) => {
                grid_volume_data.volume.get_dependend_template_nodes()
                    .chain(grid_volume_data.spacing.get_dependend_template_nodes())
                    .collect_vec()
            },
            PositionSpaceTemplate::GridOnPlane(grid_on_plane_data) => {
                grid_on_plane_data.volume.get_dependend_template_nodes()
                    .chain(grid_on_plane_data.spacing.get_dependend_template_nodes())
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
}

use std::iter;

use egui_snarl::OutPinId;
use itertools::{iproduct, Either, Itertools};
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::GetValueData, collapser::Collapser, template_changed::MatchValueData}, composer::{ModelComposer, graph::ComposerGraph, make_template::MakeTemplateData, nodes::{ComposeNode, ComposeNodeType}}, data_types::data_type::{ComposeDataType, T}, template::{Template, value::TemplateValue}}, util::{aabb::AABB, iter_merger::IM3, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::{number::{NumberValue, ValueIndexNumber}, position::{PositionValue, ValueIndexPosition}, volume::{ValueIndexVolume, VolumeValue}};

const LEAF_SPREAD_MAX_SAMPLES_MULTIPLYER: usize = 2;
const PATH_MAX_SAMPLES_MULTIPLYER: usize = 2;

pub type ValueIndexPositionSpace = usize;
pub type ValueIndexPositionSpace2D = usize;
pub type ValueIndexPositionSpace3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionSpaceValue {
    Grid(GridValue),
    LeafSpread(LeafSpreadValue),
    Path(PathValue)
}

#[derive(Debug, Clone, Copy)]
pub struct GridValue {
    pub volume: ValueIndexVolume,
    pub spacing: ValueIndexNumber,
}

#[derive(Debug, Clone, Copy)]
pub struct LeafSpreadValue {
    pub volume: ValueIndexVolume,
    pub samples: ValueIndexNumber,
}

#[derive(Debug, Clone, Copy)]
pub struct PathValue {
    pub spacing: ValueIndexNumber,
    pub side_variance: ValueIndexPosition,
    pub start: ValueIndexPosition,
    pub end: ValueIndexPosition,
}

impl ComposerGraph {
    pub fn make_pos_space(
        &self, 
        original_node: &ComposeNode, 
        in_index: usize, 
        data: &mut MakeTemplateData,
    ) -> ValueIndexPositionSpace {
        let node_id = self.get_input_remote_node_id(original_node, in_index);

        if let Some(value_index) = data.get_value_index_from_node_id(node_id) {
            data.add_depends_of_value(value_index);
            return value_index;
        }

        let node = self.snarl.get_node(node_id).expect("Node of remote not found");
        let value = match &node.t {
            ComposeNodeType::Grid2D => {
                
                let grid = GridValue {
                    volume: self.make_volume(node, 0, data),
                    spacing: self.make_number(node, 1, data),
                };
                TemplateValue::PositionSpace2D(PositionSpaceValue::Grid(grid))
            },
            ComposeNodeType::Grid3D => {
                
                let grid = GridValue {
                    volume: self.make_volume(node, 0, data),
                    spacing: self.make_number(node, 1, data),
                };
                TemplateValue::PositionSpace3D(PositionSpaceValue::Grid(grid))
            },
            ComposeNodeType::LeafSpread2D => {
                let leaf_spread = LeafSpreadValue {
                    volume: self.make_volume(node, 0, data),
                    samples: self.make_number(node, 1,data),                
                };
                TemplateValue::PositionSpace2D(PositionSpaceValue::LeafSpread(leaf_spread))
            },
            ComposeNodeType::LeafSpread3D => {
                let leaf_spread = LeafSpreadValue {
                    volume: self.make_volume(node, 0, data),
                    samples: self.make_number(node, 1,data),                
                };
                TemplateValue::PositionSpace3D(PositionSpaceValue::LeafSpread(leaf_spread))
            },
            ComposeNodeType::Path2D => {
                
                let path = PathValue {
                    start: self.make_position(node, 0, data),
                    end: self.make_position(node, 1, data),
                    spacing: self.make_number(node, 2,  data),
                    side_variance: self.make_position(node, 3, data),
                };
                TemplateValue::PositionSpace2D(PositionSpaceValue::Path(path))
            },
            ComposeNodeType::Path3D => {
                
                let path = PathValue {
                    start: self.make_position(node, 0, data),
                    end: self.make_position(node, 1, data),
                    spacing: self.make_number(node, 2, data),
                    side_variance: self.make_position(node, 3, data),
                };
                TemplateValue::PositionSpace3D(PositionSpaceValue::Path(path))
            },
            
            _ => unreachable!(),
        };

        data.set_value(node_id, value)
    }
}

impl PositionSpaceValue {
    pub fn match_value(
        &self, 
        other: &PositionSpaceValue,
        data: MatchValueData
    ) -> bool {

        match self {
            PositionSpaceValue::Grid(grid_value1) => match other {
                PositionSpaceValue::Grid(grid_value2) => {
                    data.match_two_volumes(grid_value1.volume, grid_value2.volume)
                    && data.match_two_numbers(grid_value1.spacing, grid_value2.spacing)
                },
                _ => false,
            },
            PositionSpaceValue::LeafSpread(leaf_spread_value1) => match other {
                PositionSpaceValue::LeafSpread(leaf_spread_value2) => {
                    data.match_two_volumes(leaf_spread_value1.volume, leaf_spread_value2.volume)
                    && data.match_two_numbers(leaf_spread_value1.samples, leaf_spread_value2.samples)
                },
                _ => false,
            },
            PositionSpaceValue::Path(path_value1) => match other {
                PositionSpaceValue::Path(path_value2) => {
                    data.match_two_positions_check(path_value1.start, path_value1.start)
                    && data.match_two_positions_check(path_value1.end, path_value1.end)
                    && data.match_two_numbers(path_value1.spacing, path_value1.spacing)
                    && data.match_two_positions_check(path_value1.side_variance, path_value2.side_variance)               
                },
                _ => false
            },
        }
    }

    pub fn get_value<V: Ve<T, D>, const D: usize>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser,
    ) -> (Vec<V>, bool) {
        match &self {
            PositionSpaceValue::Grid(grid)  => {
                let (mut volume, r_0) = collapser.template.get_volume_value(grid.volume)
                    .get_value::<V, (), D>(get_value_data, collapser);

                let (spacing, r_1) =  collapser.template.get_number_value(grid.spacing)
                    .get_value(get_value_data, collapser);
                
                volume.calculate_bounds();
                let mut points = vec![];
                
                for spacing in spacing {
                    points.extend(volume.get_grid_positions(spacing))
                }
                
                (points, r_0 || r_1 )
            },
            PositionSpaceValue::LeafSpread(spread) => {
                let (mut volume, r_0) = collapser.template.get_volume_value(spread.volume)
                    .get_value::<V, (), D>(get_value_data, collapser);

                let (samples, r_1) =  collapser.template.get_number_value(spread.samples)
                    .get_value(get_value_data, collapser);

                volume.calculate_bounds();
                let aabb: AABB<V, T, D> = volume.get_bounds();
                let min: V::VectorF = AABB::min(&aabb).to_vecf();
                let size: V::VectorF = aabb.size().to_vecf();
                
                let mut points = vec![];
                
                for samples in samples {

                    let samples = samples.to_usize();
                    let tries = samples * LEAF_SPREAD_MAX_SAMPLES_MULTIPLYER;

                    let mut seq = quasi_rd::Sequence::new(D);
                    let mut fi = iter::from_fn(move || Some(seq.next_f32()));

                    let pos_iter = iter::from_fn(move || {
                        let vf = V::VectorF::from_iter(&mut fi);
                        let v = V::from_vecf(vf * size + min); 
                        Some(v)
                    })
                        .take(tries)
                        .filter(|v| volume.is_position_valid(*v))
                        .take(samples);

                    points.extend(pos_iter);
                }
 
                (points, r_0 || r_1 )
            },
            PositionSpaceValue::Path(path) => {
                 let (start, r_0) =  collapser.template.get_position_value::<V, D>(path.start)
                    .get_value(get_value_data, collapser);

                let (end, r_1) =  collapser.template.get_position_value::<V, D>(path.end)
                    .get_value(get_value_data, collapser);

                let (spacing, r_2) =  collapser.template.get_number_value(path.spacing)
                    .get_value(get_value_data, collapser);

                let (side_variance, r_3) =  collapser.template.get_position_value::<V, D>(path.side_variance)
                    .get_value(get_value_data, collapser);

                let points = iproduct!(start, end, spacing, side_variance)
                    .map(|(start, end, spacing, side_variance)| {

                        let end: V::VectorF = end.to_vecf();
                        let side_variance: V::VectorF = side_variance.to_vecf();
                        let spacing = spacing.to_f32();

                        let mut current: V::VectorF = start.to_vecf();

                        let delta: V::VectorF = end - current;
                        let steps = (delta.length() / spacing) as usize;
                        let tries = steps * PATH_MAX_SAMPLES_MULTIPLYER;

                        iter::once(start).chain(
                            iter::successors(Some((current)), move |current| {
                                let delta: V::VectorF = end - *current;
                                let length = delta.length();

                                if length < spacing {
                                    return None;
                                }

                                let r = V::VectorF::from_iter(&mut iter::from_fn(|| Some(fastrand::f32()))) * 2.0 - 1.0;
                                let side = r * side_variance * length;
                                let dir = (delta + side).normalize();
                                let pos = *current + dir * spacing;

                                Some(pos)
                            })
                                .map(|v| V::from_vecf(v))
                        )
                    })
                    .flatten()
                    .collect_vec();

                (points, r_0 || r_1 || r_2 || r_3)
            } 
        }
    }
}


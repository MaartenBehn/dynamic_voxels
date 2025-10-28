use std::iter;

use egui_snarl::OutPinId;
use itertools::{iproduct, Either, Itertools};
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, ModelComposer}, data_types::data_type::ComposeDataType, template::{update::MakeTemplateData, value::ComposeTemplateValue, ComposeTemplate}}, util::{aabb::AABB, iter_merger::IM3, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::{number::{NumberTemplate, ValueIndexNumber}, position::{PositionTemplate, ValueIndexPosition}, volume::{ValueIndexVolume, VolumeTemplate}};

const LEAF_SPREAD_MAX_SAMPLES_MULTIPLYER: usize = 2;
const PATH_MAX_SAMPLES_MULTIPLYER: usize = 2;

pub type ValueIndexPositionSpace = usize;
pub type ValueIndexPositionSpace2D = usize;
pub type ValueIndexPositionSpace3D = usize;

#[derive(Debug, Clone, Copy)]
pub enum PositionSpaceTemplate {
    Grid(GridTemplate),
    LeafSpread(LeafSpreadTemplate),
    Path(PathTemplate)
}

#[derive(Debug, Clone, Copy)]
pub struct GridTemplate {
    pub volume: ValueIndexVolume,
    pub spacing: ValueIndexNumber,
}

#[derive(Debug, Clone, Copy)]
pub struct LeafSpreadTemplate {
    pub volume: ValueIndexVolume,
    pub samples: ValueIndexNumber,
}

#[derive(Debug, Clone, Copy)]
pub struct PathTemplate {
    pub spacing: ValueIndexNumber,
    pub side_variance: ValueIndexPosition,
    pub start: ValueIndexPosition,
    pub end: ValueIndexPosition,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ModelComposer<V2, V3, T, B> {
    pub fn make_pos_space(
        &self, 
        pin: OutPinId,
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> ValueIndexPositionSpace {
        if let Some(value_index) = data.value_per_node_id.get_value(pin.node) {
            return value_index;
        }

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        let value = match &node.t {
            ComposeNodeType::Grid2D => {
                
                let grid = GridTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                    spacing: self.make_number(node, 1, data),
                };
                ComposeTemplateValue::PositionSpace2D(PositionSpaceTemplate::Grid(grid))
            },
            ComposeNodeType::Grid3D => {
                
                let grid = GridTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                    spacing: self.make_number(node, 1, data),
                };
                ComposeTemplateValue::PositionSpace3D(PositionSpaceTemplate::Grid(grid))
            },
            ComposeNodeType::LeafSpread2D => {
                let leaf_spread = LeafSpreadTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                    samples: self.make_number(node, 1,data),                
                };
                ComposeTemplateValue::PositionSpace2D(PositionSpaceTemplate::LeafSpread(leaf_spread))
            },
            ComposeNodeType::LeafSpread3D => {
                let leaf_spread = LeafSpreadTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_index(node, 0), data),
                    samples: self.make_number(node, 1,data),                
                };
                ComposeTemplateValue::PositionSpace3D(PositionSpaceTemplate::LeafSpread(leaf_spread))
            },
            ComposeNodeType::Path2D => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, data),
                    end: self.make_position(node, 1, data),
                    spacing: self.make_number(node, 2,  data),
                    side_variance: self.make_position(node, 3, data),
                };
                ComposeTemplateValue::PositionSpace2D(PositionSpaceTemplate::Path(path))
            },
            ComposeNodeType::Path3D => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, data),
                    end: self.make_position(node, 1, data),
                    spacing: self.make_number(node, 2, data),
                    side_variance: self.make_position(node, 3, data),
                };
                ComposeTemplateValue::PositionSpace3D(PositionSpaceTemplate::Path(path))
            },
            
            _ => unreachable!(),
        };

        data.set_value(pin.node, value)
    }
}

impl PositionSpaceTemplate { 
    pub fn get_value<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>, const D: usize>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>,
        template: &ComposeTemplate<V2, V3, T, B>
    ) -> (Vec<V>, bool) {
        match &self {
            PositionSpaceTemplate::Grid(grid)  => {
                let (mut volume, r_0) = template.get_volume_value(grid.volume)
                    .get_value::<V, V2, V3, T, B, (), D>(get_value_data, collapser, template, ());

                let (spacing, r_1) =  template.get_number_value(grid.spacing)
                    .get_value(get_value_data, collapser, template);
                
                volume.calculate_bounds();
                let mut points = vec![];
                
                for spacing in spacing {
                    points.extend(volume.get_grid_positions(spacing))
                }
                
                (points, r_0 || r_1 )
            },
            PositionSpaceTemplate::LeafSpread(spread) => {
                let (mut volume, r_0) = template.get_volume_value(spread.volume)
                    .get_value(get_value_data, collapser, template, ());

                let (samples, r_1) =  template.get_number_value(spread.samples)
                    .get_value(get_value_data, collapser, template);

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
            PositionSpaceTemplate::Path(path) => {
                 let (start, r_0) =  template.get_position_value::<V, D>(path.start)
                    .get_value(get_value_data, collapser, template);

                let (end, r_1) =  template.get_position_value::<V, D>(path.end)
                    .get_value(get_value_data, collapser, template);

                let (spacing, r_2) =  template.get_number_value(path.spacing)
                    .get_value(get_value_data, collapser, template);

                let (side_variance, r_3) =  template.get_position_value::<V, D>(path.side_variance)
                    .get_value(get_value_data, collapser, template);

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


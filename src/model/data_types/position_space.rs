use std::iter;

use egui_snarl::OutPinId;
use itertools::{Either, Itertools};
use octa_force::{anyhow::bail, glam::{Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::{collapse::{add_nodes::GetValueData, collapser::Collapser}, composer::{build::BS, nodes::ComposeNodeType, template::{ComposeTemplate, MakeTemplateData, TemplateIndex}, ModelComposer}, data_types::data_type::ComposeDataType}, util::{aabb::AABB, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::{number::NumberTemplate, position::PositionTemplate, volume::VolumeTemplate};

const LEAF_SPREAD_MAX_SAMPLES_MULTIPLYER: usize = 2;
const PATH_MAX_SAMPLES_MULTIPLYER: usize = 2;

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
        data: &mut MakeTemplateData<V2, V3, T, B>,
    ) -> PositionSpaceTemplate<V, V2, V3, T, D> {

        let node = self.snarl.get_node(pin.node).expect("Node of remote not found");
        match &node.t {
            ComposeNodeType::Grid3D => {
                assert_eq!(D, 3);
                
                let grid = GridTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume3D), data),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data),
                };
                PositionSpaceTemplate::Grid(grid)
            },
            ComposeNodeType::Grid2D => {
                assert_eq!(D, 2);
                
                let grid = GridTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume2D), data),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data),                
                };
                PositionSpaceTemplate::Grid(grid)
            },
            ComposeNodeType::LeafSpread3D => {
                assert_eq!(D, 3);
                
                let leaf_spread = LeafSpreadTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume3D), data),
                    samples: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)),data),                
                };
                PositionSpaceTemplate::LeafSpread(leaf_spread)
            },
            ComposeNodeType::LeafSpread2D => {
                assert_eq!(D, 2);
                
                let leaf_spread = LeafSpreadTemplate {
                    volume: self.make_volume(self.get_input_remote_pin_by_type(node, ComposeDataType::Volume2D), data),
                    samples: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data),                
                };
                PositionSpaceTemplate::LeafSpread(leaf_spread)
            },
            ComposeNodeType::Path3D => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, data),
                    end: self.make_position(node, 1, data),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)), data),
                    side_variance: self.make_position(node, 3, data),
                };
                PositionSpaceTemplate::Path(path)
            },
            ComposeNodeType::Path2D => {
                
                let path = PathTemplate {
                    start: self.make_position(node, 0, data),
                    end: self.make_position(node, 1, data),
                    spacing: self.make_number(node, self.get_input_pin_index_by_type(node, ComposeDataType::Number(None)),  data),
                    side_variance: self.make_position(node, 3, data),
                };
                PositionSpaceTemplate::Path(path)
            },

            _ => unreachable!(),
        }
    }
}

impl<V: Ve<T, D>, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, const D: usize> PositionSpaceTemplate<V, V2, V3, T, D> { 
    pub fn get_value<B: BS<V2, V3, T>>(
        &self,
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> (impl Iterator<Item = V>, bool) {
        match &self {
            PositionSpaceTemplate::Grid(template)  => {
                let (mut volume, r_0) = template.volume.get_value(get_value_data, collapser, ());
                volume.calculate_bounds();
                let (spacing, r_1) = template.spacing.get_value(get_value_data, collapser);
                let pos_iter = volume.get_grid_positions(spacing);

                (Either::Left(Either::Left(pos_iter)), r_0 || r_1 )
            },
            PositionSpaceTemplate::LeafSpread(template) => {
                let (mut volume, r_0) = template.volume.get_value(get_value_data, collapser, ());
                volume.calculate_bounds();
                let (samples, r_1) = template.samples.get_value(get_value_data, collapser);

                let aabb = volume.get_bounds();

                let min: V::VectorF = AABB::min(&aabb).to_vecf();
                let size: V::VectorF = aabb.size().to_vecf();

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
                    .filter(move |v| volume.is_position_valid(*v))
                    .take(samples);

                (Either::Left(Either::Right(pos_iter)), r_0 || r_1 )
            },
            PositionSpaceTemplate::Path(template) => {
                let (spacing, r_0) = template.spacing.get_value(get_value_data, collapser); 
                let (side_variance, r_1) = template.side_variance.get_value(get_value_data, collapser); 
                let (start, r_2) = template.start.get_value(get_value_data, collapser); 
                let (end, r_3) = template.end.get_value(get_value_data, collapser);

                let end: V::VectorF = end.to_vecf();
                let side_variance: V::VectorF = side_variance.to_vecf();
                let spacing = spacing.to_f32();

                let mut current: V::VectorF = start.to_vecf();

                let delta: V::VectorF = end - current;
                let steps = (delta.length() / spacing) as usize;
                let tries = steps * PATH_MAX_SAMPLES_MULTIPLYER;

                let pos_iter = iter::once(start).chain(
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
                );

                (Either::Right(pos_iter), r_0 || r_1 || r_2 || r_3)
            } 
        }
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


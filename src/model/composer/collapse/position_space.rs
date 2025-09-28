use std::{fmt, iter, marker::PhantomData, usize};

use itertools::{Either, Itertools};
use octa_force::{anyhow::bail, glam::{IVec3, Mat4, Vec3, Vec3A}, log::warn, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::composer::{build::BS, position_space::PositionSpaceTemplate, template::TemplateIndex}, util::{aabb::AABB, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::{add_nodes::GetValueData, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}};

const LEAF_SPREAD_MAX_SAMPLES_MULTIPLYER: usize = 2;
const PATH_MAX_SAMPLES_MULTIPLYER: usize = 2;

#[derive(Debug, Clone)]
pub struct PositionSpace<V: Ve<T, D>, T: Nu, const D: usize> {
    data: PositionSpaceData<V, T, D>,
    positions: SlotMap<CollapseChildKey, V>,
    new_children: Vec<CollapseChildKey>,
}

#[derive(Debug, Clone)]
enum PositionSpaceData<V: Ve<T, D>, T: Nu, const D: usize> {
    Grid(Grid<V, T, D>),
    LeafSpread(LeafSpread<V, T, D>),
    Path(Path<V, T, D>)
}

#[derive(Clone, Debug)]
struct Grid<V: Ve<T, D>, T: Nu, const D: usize> {
    pub volume: CSGTree<(), V, T, D>,
    pub spacing: T,
}

#[derive(Clone, Debug)]
struct LeafSpread<V: Ve<T, D>, T: Nu, const D: usize> {
    pub volume: CSGTree<(), V, T, D>,
    pub samples: T,
}

#[derive(Clone, Debug)]
struct Path<V: Ve<T, D>, T: Nu, const D: usize> {
    pub spacing: T,
    pub side_variance: V,
    pub start: V,
    pub end: V,
}

impl<V: Ve<T, D>,  T: Nu, const D: usize> PositionSpace<V, T, D> {
    pub fn from_template<V2: Ve<T, 2>, V3: Ve<T, 3>, B: BS<V2, V3, T>>(
        template_space: &PositionSpaceTemplate<V, V2, V3, T, D>, 
        get_value_data: GetValueData,
        collapser: &Collapser<V2, V3, T, B>
    ) -> Self {
        let data = match &template_space {
            PositionSpaceTemplate::Grid(template) 
            => PositionSpaceData::Grid(Grid { 
                volume: template.volume.get_value(get_value_data, collapser, ()), 
                spacing: template.spacing.get_value(get_value_data, collapser),
            }),
            PositionSpaceTemplate::LeafSpread(template) 
            => PositionSpaceData::LeafSpread(LeafSpread { 
                volume: template.volume.get_value(get_value_data, collapser, ()), 
                samples: template.samples.get_value(get_value_data, collapser), 
            }),
            PositionSpaceTemplate::Path(template) 
            => PositionSpaceData::Path(Path {
                spacing: template.spacing.get_value(get_value_data, collapser),
                side_variance: template.side_variance.get_value(get_value_data, collapser),
                start: template.start.get_value(get_value_data, collapser),
                end: template.end.get_value(get_value_data, collapser),
            }),
        };

        Self {
            data,
            positions: Default::default(),
            new_children: Default::default(),
        }
    }

    pub fn get_position(&self, index: CollapseChildKey) -> V {
        self.positions[index]    
    }

    pub fn get_positions(&self) -> impl Iterator<Item = V> {
         self.positions.values().copied()
    }

    pub fn is_child_valid(&self, index: CollapseChildKey) -> bool {
        self.positions.contains_key(index)    
    }

    pub fn update(&mut self) {
        let new_positions = match &self.data {
            PositionSpaceData::Grid(grid_volume) => {
                grid_volume.volume.get_grid_positions(grid_volume.spacing).collect_vec()
            },
            PositionSpaceData::LeafSpread(leaf_spread) => leaf_spread.get_positions(),
            PositionSpaceData::Path(path) => path.get_positions(),
        };

        self.new_children = update_positions(new_positions, &mut self.positions);
    }

    pub fn get_new_children(&self) -> &[CollapseChildKey] {
        &self.new_children
    }
}

fn update_positions<V: Ve<T, D>, T: Nu, const D: usize>(
    mut new_positions: Vec<V>, 
    positions: &mut SlotMap<CollapseChildKey, V>
) -> Vec<CollapseChildKey> {
    positions.retain(|_, p| {
        if let Some(i) = new_positions.iter().position(|t| *t == *p) {
            new_positions.swap_remove(i);
            true
        } else {
            false
        }
    });

    new_positions.iter()
        .map(|p| positions.insert(*p))
        .collect_vec()

}

impl<V: Ve<T, D>, T: Nu, const D: usize> Path<V, T, D> {
    pub fn get_positions(&self) -> Vec<V> {
        let end: V::VectorF = self.end.to_vecf();
        let side_variance: V::VectorF = self.side_variance.to_vecf();
        let spacing = self.spacing.to_f32();
        
        let mut points = vec![self.start];
        let mut current: V::VectorF = self.start.to_vecf();

        let delta: V::VectorF = end - current;
        let steps = (delta.length() / spacing) as usize;
        let tries = steps * PATH_MAX_SAMPLES_MULTIPLYER;

        for i in 0..tries {
            let delta: V::VectorF = end - current;
            let length = delta.length();
            if length < spacing {
                points.push(self.end);
                return points;
            }

            let r = V::VectorF::from_iter(&mut iter::from_fn(|| Some(fastrand::f32()))) * 2.0 - 1.0;
            let side = r * side_variance * length;
            let dir = (delta + side).normalize();
            current = current + dir * spacing;
            points.push(V::from_vecf(current));
        }

        warn!("Path timeout at {:?} did not react end {:?}", current, end);

        points
    }
}

impl<V: Ve<T, D>, T: Nu, const D: usize> LeafSpread<V, T, D> {
    pub fn get_positions(&self) -> Vec<V> {
        let aabb = self.volume.get_bounds();
        let min: V::VectorF = AABB::min(&aabb).to_vecf();
        let size: V::VectorF = aabb.size().to_vecf();

        let samples = self.samples.to_usize();
        let tries = samples * LEAF_SPREAD_MAX_SAMPLES_MULTIPLYER;

        let mut seq = quasi_rd::Sequence::new(D);
        let mut fi = iter::from_fn(|| Some(seq.next_f32()));

        let points = iter::from_fn(|| {
            let vf = V::VectorF::from_iter(&mut fi);
            let v = V::from_vecf(vf * size + min); 
            Some(v)
        })
            .take(tries)
            .filter(|v| self.volume.is_position_valid(*v))
            .take(samples)
            .collect_vec();

        if points.len() < samples {
            warn!("Leaf Spread timeout only {:?} of {:?} samples points created.", points.len(), samples);
        }

        points
    }
}

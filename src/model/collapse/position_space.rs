use std::{fmt, iter, marker::PhantomData, usize};

use itertools::{Either, Itertools};
use octa_force::{anyhow::bail, glam::{IVec3, Mat4, Vec3, Vec3A}, log::warn, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::{composer::{build::BS, template::TemplateIndex}, data_types::position_space::PositionSpaceTemplate}, util::{aabb::AABB, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::{add_nodes::GetValueData, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}};


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
    ) -> (Self, bool) {
        let (data, needs_recompute) = match &template_space {
            PositionSpaceTemplate::Grid(template)  => {
                let (mut volume, r_0) = template.volume.get_value(get_value_data, collapser, ());
                volume.calculate_bounds();
                let (spacing, r_1) = template.spacing.get_value(get_value_data, collapser); 

                (PositionSpaceData::Grid(Grid { 
                    volume, 
                    spacing,
                }), r_0 || r_1 )
            },
            PositionSpaceTemplate::LeafSpread(template) => {
                let (mut volume, r_0) = template.volume.get_value(get_value_data, collapser, ());
                volume.calculate_bounds();
                let (samples, r_1) = template.samples.get_value(get_value_data, collapser); 

                (PositionSpaceData::LeafSpread(LeafSpread { 
                    volume, 
                    samples, 
                }), r_0 || r_1 )
            },
            PositionSpaceTemplate::Path(template) => {
                let (spacing, r_0) = template.spacing.get_value(get_value_data, collapser); 
                let (side_variance, r_1) = template.side_variance.get_value(get_value_data, collapser); 
                let (start, r_2) = template.start.get_value(get_value_data, collapser); 
                let (end, r_3) = template.end.get_value(get_value_data, collapser); 

                (PositionSpaceData::Path(Path {
                    spacing,
                    side_variance,
                    start,
                    end,
                }), r_0 || r_1 || r_2 || r_3)
            } 
        };

        (Self {
            data,
            positions: Default::default(),
            new_children: Default::default(),
        }, needs_recompute)
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





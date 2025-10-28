use std::{fmt, iter, marker::PhantomData, usize};

use itertools::{Either, Itertools};
use octa_force::{anyhow::bail, glam::{IVec3, Mat4, Vec3, Vec3A}, log::warn, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::{composer::{build::BS}, data_types::{number_space::NumberSpaceTemplate, position_space::PositionSpaceTemplate}}, util::{aabb::AABB, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValid}};

use super::{add_nodes::GetValueData, collapser::{CollapseChildKey, CollapseNodeKey, Collapser}};


#[derive(Debug, Clone, Default)]
pub struct PositionPairSet<V: Ve<T, D>, T: Nu, const D: usize> {
    positions: SlotMap<CollapseChildKey, (V, V)>,
    new_children: Vec<CollapseChildKey>,
    p: PhantomData<T>,
}

impl<V: Ve<T, D>,  T: Nu, const D: usize> PositionPairSet<V, T, D> { 
    pub fn get_position_pair(&self, index: CollapseChildKey) -> (V, V) {
        self.positions[index]    
    }

    pub fn get_position_pairs(&self) -> impl Iterator<Item = (V, V)> {
        self.positions.values().into_iter().map(|v| *v)
    }
    
    pub fn is_child_valid(&self, index: CollapseChildKey) -> bool {
        self.positions.contains_key(index)    
    }

    pub fn update(
        &mut self,
        mut new_pairs: Vec<(V, V)>,
    ) {
        self.positions.retain(|_, p| {
            if let Some(i) = new_pairs.iter().position(|t| *t == *p) {
                new_pairs.swap_remove(i);
                true
            } else {
                false
            }
        });

        let new_children = new_pairs.iter()
            .map(|p| self.positions.insert(*p))
            .collect_vec();

        self.new_children = new_children;
    }

    pub fn get_new_children(&self) -> &[CollapseChildKey] {
        &self.new_children
    }
}





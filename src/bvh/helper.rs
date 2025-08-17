use std::mem::MaybeUninit;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

use super::{node::BvhNode, shape::{ShapeIndex, Shapes}, traits::BHShape};


/// Holds the arguments for calling build.
pub struct BvhNodeBuildArgs<'a, S, V: Ve<T, D>, T: Nu, const D: usize> {
    pub(crate) shapes: &'a Shapes<'a, S>,
    pub(crate) indices: &'a mut [ShapeIndex],
    pub(crate) nodes: &'a mut [MaybeUninit<BvhNode<V, T, D>>],
    pub(crate) parent_index: usize,
    pub(crate) depth: u32,
    pub(crate) node_index: usize,
    pub(crate) exit_index: usize,
    pub(crate) aabb_bounds: AABB<V, T, D>,
    pub(crate) centroid_bounds: AABB<V, T, D>,
}

impl<S, V: Ve<T, D>, T: Nu, const D: usize> BvhNodeBuildArgs<'_, S, V, T, D> {
    /// Finish building this portion of the bvh.
    pub fn build(self)
    where
        S: BHShape<V, T, D>,
    {
        BvhNode::<V, T, D>::build(self)
    }

    /// Finish building this portion of the bvh using a custom executor.
    pub fn build_with_executor(
        self,
        executor: impl FnMut(BvhNodeBuildArgs<'_, S, V, T, D>, BvhNodeBuildArgs<'_, S, V, T, D>),
    ) where
        S: BHShape<V, T, D>,
    {
        BvhNode::<V, T, D>::build_with_executor(self, executor)
    }

    /// Returns the number of nodes that are part of this build.
    pub fn node_count(&self) -> usize {
        self.indices.len()
    }

    /// Returns the current depth in the Bvh.
    pub fn depth(&self) -> usize {
        self.depth as usize
    }
}

pub(crate) fn joint_aabb_of_shapes<V: Ve<T, D>, T: Nu, const D: usize, Shape: BHShape<V, T, D>>(
    indices: &[ShapeIndex],
    shapes: &Shapes<Shape>,
) -> (AABB<V, T, D>, AABB<V, T, D>) {
    let mut aabb = AABB::default();
    let mut centroid = AABB::default();
    for index in indices {
        let shape = shapes.get(*index);
        let shape_aabb = shape.aabb();
        aabb.union_mut(shape_aabb);
        centroid.union_point_mut(shape_aabb.center());
    }
    (aabb, centroid)
}

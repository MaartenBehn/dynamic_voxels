use std::mem::MaybeUninit;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

use super::{node::BHNode, shape::{BHShape, Shapes}};


/// Holds the arguments for calling build.
pub struct BvhNodeBuildArgs<'a, S: BHShape<V, T, D>, N: BHNode<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> {
    pub(crate) shapes: &'a Shapes<'a, S, V, T, D>,
    pub(crate) indices: &'a mut [usize],
    pub(crate) nodes: &'a mut [MaybeUninit<N>],
    pub(crate) node_index: usize,
    pub(crate) exit_index: usize,
    pub(crate) aabb_bounds: AABB<V, T, D>,
    pub(crate) centroid_bounds: AABB<V, T, D>,
}

impl<S: BHShape<V, T, D>, N: BHNode<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> BvhNodeBuildArgs<'_, S, N, V, T, D> {
    /// Finish building this portion of the bvh.
    pub fn build(self)
    where
        S: BHShape<V, T, D>,
    {
        N::build(self)
    }

    /// Finish building this portion of the bvh using a custom executor.
    pub fn build_with_executor(
        self,
        executor: impl FnMut(BvhNodeBuildArgs<'_, S, N, V, T, D>, BvhNodeBuildArgs<'_, S, N, V, T, D>),
    ) where
        S: BHShape<V, T, D>,
    {
        N::build_with_executor(self, executor)
    }

    /// Returns the number of nodes that are part of this build.
    pub fn node_count(&self) -> usize {
        self.indices.len()
    }
}

pub(crate) fn joint_aabb_of_shapes<V: Ve<T, D>, T: Nu, const D: usize, Shape: BHShape<V, T, D>>(
    indices: &[usize],
    shapes: &Shapes<Shape, V, T, D>,
) -> (AABB<V, T, D>, AABB<V, T, D>) {
    let mut aabb = AABB::default();
    let mut centroid = AABB::default();
    for index in indices {
        let shape_aabb = shapes.aabb(*index);
        aabb.union_mut(shape_aabb);
        centroid.union_point_mut(shape_aabb.center());
    }
    (aabb, centroid)
}

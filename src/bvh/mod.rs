// Based on 

pub mod shape;
pub mod node;
pub mod helper;
pub mod traits;
pub mod bucket;

use std::{marker::PhantomData, mem::MaybeUninit};

use helper::{joint_aabb_of_shapes, BvhNodeBuildArgs};
use node::{BHNode};
use shape::{BHShape, Shapes};

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

#[derive(Debug, Default, Clone)]
pub struct Bvh<N: BHNode<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> {
    pub nodes: Vec<N>,
    p1: PhantomData<V>,
    p2: PhantomData<T>,
}

impl<N: BHNode<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> Bvh<N, V, T, D> { 

    pub fn build_par<Shape: BHShape<V, T, D> + Send>(shapes: &[Shape], leafs: &mut [usize]) -> Self
    where
        T: Send,
        Self: Sized,
    {
        Self::build_with_executor(shapes, leafs, rayon_executor)
    }

    fn build_with_executor<S: BHShape<V, T, D>>(
        shapes: &[S], 
        indices: &mut [usize], 
        executor: impl FnMut(BvhNodeBuildArgs<S, N, V, T, D>, BvhNodeBuildArgs<S, N, V, T, D>),
    ) -> Bvh<N, V, T, D> {
        if shapes.is_empty() {
            return Bvh { nodes: Vec::new(), ..Default::default() };
        }

        let expected_node_count = shapes.len() * 2 - 1;
        let mut nodes = Vec::with_capacity(expected_node_count);

        let uninit_slice = unsafe {
            std::slice::from_raw_parts_mut(
                nodes.as_mut_ptr() as *mut MaybeUninit<N>,
                expected_node_count,
            )
        };
        let shapes = Shapes::from_slice(shapes);
        let (aabb, centroid) = joint_aabb_of_shapes(&indices, &shapes);

        let tree_len = indices.len() * 2 - 1;

        N::build_with_executor(
            BvhNodeBuildArgs {
                shapes: &shapes,
                indices: indices,
                nodes: uninit_slice,
                node_index: 0,
                exit_index: tree_len,
                aabb_bounds: aabb,
                centroid_bounds: centroid,
            },
            executor,
        );

        // SAFETY
        // The vec is allocated with this capacity above and is only mutated through slice methods so
        // it is guaranteed that the allocated size has not changed.
        unsafe {
            nodes.set_len(expected_node_count);
        }
        Bvh { nodes, ..Default::default() }
    }
}


pub fn rayon_executor<S, N: BHNode<V, T, D>, V: Ve<T, D>, T: Send + Nu, const D: usize>(
    left: BvhNodeBuildArgs<S, N, V, T, D>,
    right: BvhNodeBuildArgs<S, N, V, T, D>,
) where
    S: BHShape<V, T, D> + Send,
{
    // 64 was found experimentally. Calling join() has overhead that makes the build slower without this.
    if left.node_count() + right.node_count() < 64 {
        left.build();
        right.build();
    } else {
        rayon::join(
            || left.build_with_executor(rayon_executor),
            || right.build_with_executor(rayon_executor),
        );
    }
}

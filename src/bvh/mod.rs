// Based on 

pub mod shape;
pub mod node;
pub mod helper;
pub mod traits;
pub mod bucket;

use std::{marker::PhantomData, mem::MaybeUninit};

use helper::{joint_aabb_of_shapes, BvhNodeBuildArgs};
use node::BvhNode;
use shape::{ShapeIndex, Shapes};
use traits::BHShape;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

#[derive(Debug)]
pub struct Bvh<V: Ve<T, D>, T: Nu, const D: usize> {
    nodes: Vec<BvhNode<V, T, D>>,
}

impl<V: Ve<T, D>, T: Nu, const D: usize> Bvh<V, T, D> { 

    pub fn build_par<Shape: BHShape<V, T, D> + Send>(shapes: &mut [Shape], leafs: &[usize]) -> Self
    where
        T: Send,
        Self: Sized,
    {
        Self::build_with_executor(shapes, leafs, rayon_executor)
    }

    fn build_with_executor<S: BHShape<V, T, D>>(
        shapes: &mut [S], 
        leafs: &[usize], 
        executor: impl FnMut(BvhNodeBuildArgs<S, V, T, D>, BvhNodeBuildArgs<S, V, T, D>),
    ) -> Bvh<V, T, D> {
        if shapes.is_empty() {
            return Bvh { nodes: Vec::new() };
        }

        let mut indices = leafs.into_iter()
            .map(|i| ShapeIndex(*i))
            .collect::<Vec<ShapeIndex>>();
        let expected_node_count = shapes.len() * 2 - 1;
        let mut nodes = Vec::with_capacity(expected_node_count);

        let uninit_slice = unsafe {
            std::slice::from_raw_parts_mut(
                nodes.as_mut_ptr() as *mut MaybeUninit<BvhNode<V, T, D>>,
                expected_node_count,
            )
        };
        let shapes = Shapes::from_slice(shapes);
        let (aabb, centroid) = joint_aabb_of_shapes(&indices, &shapes);

        let tree_len = indices.len() * 2 - 1;

        BvhNode::build_with_executor(
            BvhNodeBuildArgs {
                shapes: &shapes,
                indices: &mut indices,
                nodes: uninit_slice,
                parent_index: 0,
                depth: 0,
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
        Bvh { nodes }
    }
}


pub fn rayon_executor<S, V: Ve<T, D>, T: Send + Nu, const D: usize>(
    left: BvhNodeBuildArgs<S, V, T, D>,
    right: BvhNodeBuildArgs<S, V, T, D>,
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

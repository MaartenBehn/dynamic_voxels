// Based on 

pub mod shape;
pub mod node;
pub mod helper;
pub mod bucket;

use std::{marker::PhantomData, mem::MaybeUninit};

use helper::{joint_aabb_of_shapes, BvhNodeBuildArgs};
use node::{BHNode};
use shape::{BHShape, Shapes};

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

#[derive(Debug, Clone)]
pub struct Bvh<N: BHNode<E, V, T, D>, E, V: Ve<T, D>, T: Nu, const D: usize> {
    pub nodes: Vec<N>,
    p0: PhantomData<E>,
    p1: PhantomData<V>,
    p2: PhantomData<T>,
}

impl<N: BHNode<E, V, T, D>, E, V: Ve<T, D>, T: Nu, const D: usize> Bvh<N, E, V, T, D> { 
    pub fn empty() -> Self {
        Bvh { 
            nodes: vec![],
            p0: PhantomData,
            p1: PhantomData,
            p2: PhantomData,
        }
    }

    pub fn build_par<S>(shapes: &[S], leafs: &mut [usize]) -> Self
    where
        T: Send,
        Self: Sized,
        S: BHShape<E, V, T, D> + Send + Sync,
        E: Send + Sync,
        N: Send,
    {
        Self::build_with_executor(shapes, leafs, rayon_executor)
    }

    fn build_with_executor<S: BHShape<E, V, T, D>>(
        shapes: &[S], 
        indices: &mut [usize], 
        executor: impl FnMut(BvhNodeBuildArgs<E, S, N, V, T, D>, BvhNodeBuildArgs<E, S, N, V, T, D>),
    ) -> Bvh<N, E, V, T, D> {
        if shapes.is_empty() {
            return Bvh { 
                nodes: Vec::new(), 
                p0: PhantomData,
                p1: PhantomData,
                p2: PhantomData,
            };
        }

        let tree_len = indices.len() * 2 - 1;
        let mut nodes = Vec::with_capacity(tree_len);

        let uninit_slice = unsafe {
            std::slice::from_raw_parts_mut(
                nodes.as_mut_ptr() as *mut MaybeUninit<N>,
                tree_len,
            )
        };
        let shapes = Shapes::from_slice(shapes);
        let (aabb, centroid) = joint_aabb_of_shapes(&indices, &shapes);

        
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
            nodes.set_len(tree_len);
        }
        Bvh { 
            nodes,
            p0: PhantomData,
            p1: PhantomData,
            p2: PhantomData,
        }
    }
}


pub fn rayon_executor<E, S, N, V: Ve<T, D>, T: Send + Nu, const D: usize>(
    left: BvhNodeBuildArgs<E, S, N, V, T, D>,
    right: BvhNodeBuildArgs<E, S, N, V, T, D>,
) where
    S: BHShape<E, V, T, D> + Send + Sync,
    N: BHNode<E, V, T, D> + Send,
    E: Send + Sync
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

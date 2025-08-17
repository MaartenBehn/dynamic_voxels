use std::cell::RefCell;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

use super::{bucket::{Bucket, BUCKETS, NUM_BUCKETS}, helper::{joint_aabb_of_shapes, BvhNodeBuildArgs}, shape::{ShapeIndex, Shapes}, traits::BHShape};


/// The [`BvhNode`] enum that describes a node in a [`Bvh`].
/// It's either a leaf node and references a shape (by holding its index)
/// or a regular node that has two child nodes.
/// The non-leaf node stores the [`Aabb`]s of its children.
///
/// [`Aabb`]: ../aabb/struct.Aabb.html
/// [`Bvh`]: struct.Bvh.html
/// [`Bvh`]: struct.BvhNode.html
///
#[derive(Debug, Copy, Clone)]
//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct  BvhNode<V: Ve<T, D>, T: Nu, const D: usize> {
    aabb: AABB<V, T, D>,
    exit: u32,
    shape: u32,
}


impl<V: Ve<T, D>, T: Nu, const D: usize> BvhNode<V, T, D> {
    /// Builds a [`BvhNode`] recursively using SAH partitioning.
    ///
    /// [`BvhNode`]: enum.BvhNode.html
    ///
    pub fn build<S: BHShape<V, T, D>>(args: BvhNodeBuildArgs<S, V, T, D>) {
        if let Some((left, right)) = Self::prep_build(args) {
            Self::build(left);
            Self::build(right);
        }
    }

    /// Builds a [`BvhNode`] with a custom executor function using SAH partitioning.
    ///
    /// [`BvhNode`]: enum.BvhNode.html
    ///
    pub fn build_with_executor<S: BHShape<V, T, D>>(
        args: BvhNodeBuildArgs<S, V, T, D>,
        mut executor: impl FnMut(BvhNodeBuildArgs<S, V, T, D>, BvhNodeBuildArgs<S, V, T, D>),
    ) {
        if let Some((left, right)) = Self::prep_build(args) {
            executor(left, right);
        }
    }

    /// Builds a single [`BvhNode`] in the [`Bvh`] heirarchy.
    /// Returns the arguments needed to call this function and build the future
    /// children of this node. If you do not call this function using the arguments
    /// returned then the Bvh will not be completely built.
    ///
    /// [`BvhNode`]: enum.BvhNode.html
    ///
    fn prep_build<S: BHShape<V, T, D>>(
        args: BvhNodeBuildArgs<S, V, T, D>,
    ) -> Option<(BvhNodeBuildArgs<S, V, T, D>, BvhNodeBuildArgs<S, V, T, D>)> {
        let BvhNodeBuildArgs {
            shapes,
            indices,
            nodes,
            parent_index,
            depth,
            node_index,
            exit_index,
            aabb_bounds,
            centroid_bounds,
        } = args;
        // If there is only one element left, don't split anymore
        if indices.len() == 1 {
            let shape_index = indices[0];
            nodes[0].write(BvhNode {
                aabb: aabb_bounds,
                exit: exit_index as _,
                shape: shape_index.0 as _,
            });
            // Let the shape know the index of the node that represents it.
            shapes.set_node_index(shape_index, node_index);
            return None;
        }

        // Find the axis along which the shapes are spread the most.
        let split_axis = centroid_bounds.largest_axis();
        let split_axis_size = centroid_bounds.max()[split_axis] - centroid_bounds.min()[split_axis];

        // The following `if` partitions `indices` for recursively calling `Bvh::build`.
        let (
            (child_l_aabb, child_l_centroid, child_l_indices),
            (child_r_aabb, child_r_centroid, child_r_indices),
        ) = if split_axis_size <= T::EPSILON {
            // In this branch the shapes lie too close together so that splitting them in a
            // sensible way is not possible. Instead we just split the list of shapes in half.
            let (child_l_indices, child_r_indices) = indices.split_at_mut(indices.len() / 2);
            let (child_l_aabb, child_l_centroid) = joint_aabb_of_shapes(child_l_indices, shapes);
            let (child_r_aabb, child_r_centroid) = joint_aabb_of_shapes(child_r_indices, shapes);

            (
                (child_l_aabb, child_l_centroid, child_l_indices),
                (child_r_aabb, child_r_centroid, child_r_indices),
            )
        } else {
            BvhNode::build_buckets(
                shapes,
                indices,
                split_axis,
                split_axis_size,
                &centroid_bounds,
                &aabb_bounds,
            )
        };

        // Since the Bvh is a full binary tree, we can calculate exactly how many indices each side of the tree
        // will occupy with the formula 2 * (num_shapes) - 1.
        let left_len = child_l_indices.len() * 2 - 1;
        let rigth_len = child_r_indices.len() * 2 - 1;

        // Place the left child right after the current node.
        let child_l_index = node_index + 1;
        // Place the right child after all of the nodes in the tree under the left child.
        let child_r_index = child_l_index + left_len;
        // The first node after this subtree.
        let after_subtree_index = child_r_index + rigth_len;
      
        // Construct the actual data structure and replace the dummy node.
        nodes[0].write(BvhNode {
            aabb: aabb_bounds,
            exit: exit_index as _,
            shape: u32::MAX,
        });

        // Remove the current node from the future build steps.
        let next_nodes = &mut nodes[1..];
        // Split the remaining nodes in the slice based on how many nodes we will need for each subtree.
        let (l_nodes, r_nodes) = next_nodes.split_at_mut(left_len);

        Some((
            BvhNodeBuildArgs {
                shapes,
                indices: child_l_indices,
                nodes: l_nodes,
                parent_index: node_index,
                depth: depth + 1,
                node_index: child_l_index,
                exit_index: child_r_index,
                aabb_bounds: child_l_aabb,
                centroid_bounds: child_l_centroid,
            },
            BvhNodeBuildArgs {
                shapes,
                indices: child_r_indices,
                nodes: r_nodes,
                parent_index: node_index,
                depth: depth + 1,
                node_index: child_r_index,
                exit_index: after_subtree_index,
                aabb_bounds: child_r_aabb,
                centroid_bounds: child_r_centroid,
            },
        ))
    }

    #[allow(clippy::type_complexity)]
    fn build_buckets<'a, S: BHShape<V, T, D>>(
        shapes: &Shapes<S>,
        indices: &'a mut [ShapeIndex],
        split_axis: usize,
        split_axis_size: T,
        centroid_bounds: &AABB<V, T, D>,
        aabb_bounds: &AABB<V, T, D>,
    ) -> (
        (AABB<V, T, D>, AABB<V, T, D>, &'a mut [ShapeIndex]),
        (AABB<V, T, D>, AABB<V, T, D>, &'a mut [ShapeIndex]),
    ) {
        // Use fixed size arrays of `Bucket`s, and thread local index assignment vectors.
        BUCKETS.with(move |buckets_ref| {
            let bucket_assignments = &mut *buckets_ref.borrow_mut();
            let mut buckets = [Bucket::empty(); NUM_BUCKETS];
            buckets.fill(Bucket::empty());
            for b in bucket_assignments.iter_mut() {
                b.clear();
            }

            // In this branch the `split_axis_size` is large enough to perform meaningful splits.
            // We start by assigning the shapes to `Bucket`s.
            for idx in indices.iter() {
                let shape = shapes.get(*idx);
                let shape_aabb = shape.aabb();
                let shape_center = shape_aabb.center();

                // Get the relative position of the shape centroid `[0.0..1.0]`.
                let bucket_num_relative =
                    (shape_center[split_axis] - centroid_bounds.min()[split_axis]) / split_axis_size;

                // Convert that to the actual `Bucket` number.
                let bucket_num = (bucket_num_relative
                    * (T::from_usize(NUM_BUCKETS) - T::from_f32(0.01)))
                .to_usize();

                // Extend the selected `Bucket` and add the index to the actual bucket.
                buckets[bucket_num].add_aabb(shape_aabb);
                bucket_assignments[bucket_num].push(*idx);
            }

            // Compute the costs for each configuration and select the best configuration.
            let mut min_bucket = 0;
            let mut min_cost = T::MAX;
            let mut child_l_aabb = AABB::default();
            let mut child_l_centroid = AABB::default();
            let mut child_r_aabb = AABB::default();
            let mut child_r_centroid = AABB::default();
            for i in 0..(NUM_BUCKETS - 1) {
                let (l_buckets, r_buckets) = buckets.split_at(i + 1);
                let child_l = l_buckets.iter().fold(Bucket::empty(), Bucket::join_bucket);
                let child_r = r_buckets.iter().fold(Bucket::empty(), Bucket::join_bucket);

                let cost = (T::from_usize(child_l.size) * child_l.aabb.surface_area()
                    + T::from_usize(child_r.size) * child_r.aabb.surface_area())
                    / aabb_bounds.surface_area();
                if cost < min_cost {
                    min_bucket = i;
                    min_cost = cost;
                    child_l_aabb = child_l.aabb;
                    child_l_centroid = child_l.centroid;
                    child_r_aabb = child_r.aabb;
                    child_r_centroid = child_r.centroid;
                }
            }
            // Join together all index buckets.
            // split input indices, loop over assignments and assign
            let (l_assignments, r_assignments) = bucket_assignments.split_at_mut(min_bucket + 1);

            let mut l_count = 0;
            for group in l_assignments.iter() {
                l_count += group.len();
            }

            let (child_l_indices, child_r_indices) = indices.split_at_mut(l_count);

            for (l_i, shape_index) in l_assignments
                .iter()
                .flat_map(|group| group.iter())
                .enumerate()
            {
                child_l_indices[l_i] = *shape_index;
            }
            for (r_i, shape_index) in r_assignments
                .iter()
                .flat_map(|group| group.iter())
                .enumerate()
            {
                child_r_indices[r_i] = *shape_index;
            }

            (
                (child_l_aabb, child_l_centroid, child_l_indices),
                (child_r_aabb, child_r_centroid, child_r_indices),
            )
        })
    }
 
    /* 
    /// Traverses the [`Bvh`] recursively and returns all shapes whose [`Aabb`] is
    /// intersected by the given [`Ray`].
    ///
    /// [`Aabb`]: ../aabb/struct.Aabb.html
    /// [`Bvh`]: struct.Bvh.html
    /// [`Ray`]: ../ray/struct.Ray.html
    ///
    pub(crate) fn traverse_recursive<Query: IntersectsAabb<T, D>, Shape: Bounded<T, D>>(
        nodes: &[BvhNode<V, T, D>],
        node_index: usize,
        shapes: &[Shape],
        query: &Query,
        indices: &mut Vec<usize>,
    ) {
        match nodes[node_index] {
            BvhNode::Node {
                ref child_l_aabb,
                child_l_index,
                ref child_r_aabb,
                child_r_index,
                ..
            } => {
                if query.intersects_aabb(child_l_aabb) {
                    BvhNode::traverse_recursive(nodes, child_l_index, shapes, query, indices);
                }
                if query.intersects_aabb(child_r_aabb) {
                    BvhNode::traverse_recursive(nodes, child_r_index, shapes, query, indices);
                }
            }
            BvhNode::Leaf { shape_index, .. } => {
                // Either we got to a non-root node recursively, in which case the caller
                // checked our AABB, or we are processing the root node, in which case we
                // need to check the AABB.
                if node_index != 0 || query.intersects_aabb(&shapes[shape_index].aabb()) {
                    indices.push(shape_index);
                }
            }
        }
    }
    */

    /*
    /// Traverses the [`Bvh`] recursively and updates the given `best_candidate` with
    /// the nearest shape found so far.
    ///
    /// [`Aabb`]: ../aabb/struct.Aabb.html
    /// [`Bvh`]: struct.Bvh.html
    ///
    pub(crate) fn nearest_to_recursive<'a, Shape: Bounded<T, D> + PointDistance<T, D>>(
        nodes: &[BvhNode<V, T, D>],
        node_index: usize,
        query: nalgebra::Point<T, D>,
        shapes: &'a [Shape],
        best_candidate: &mut Option<(&'a Shape, T)>,
    ) {
        match nodes[node_index] {
            BvhNode::Node {
                ref child_l_aabb,
                child_l_index,
                ref child_r_aabb,
                child_r_index,
                ..
            } => {
                // Compute the min dist for both children
                let mut children = [
                    (child_l_index, child_l_aabb.min_distance_squared(query)),
                    (child_r_index, child_r_aabb.min_distance_squared(query)),
                ];

                // Sort children to go to the best candidate first and have a better chance of pruning
                if children[0].1 > children[1].1 {
                    children.swap(0, 1);
                }

                // Traverse children
                for (index, child_dist) in children {
                    // Node might contain a better shape: check it.
                    // TODO: to be replaced by `Option::is_none_or` after 2025-10 for 1 year MSRV.
                    #[allow(clippy::unnecessary_map_or)]
                    if best_candidate.map_or(true, |(_, best_dist)| child_dist < best_dist) {
                        Self::nearest_to_recursive(nodes, index, query, shapes, best_candidate);
                    }
                }
            }
            BvhNode::Leaf { shape_index, .. } => {
                // This leaf might contain a better shape: check it directly with its exact distance (squared).
                let dist = shapes[shape_index].distance_squared(query);

                // TODO: to be replaced by `Option::is_none_or` after 2025-10 for 1 year MSRV.
                #[allow(clippy::unnecessary_map_or)]
                if best_candidate.map_or(true, |(_, best_dist)| dist < best_dist) {
                    *best_candidate = Some((&shapes[shape_index], dist));
                }
            }
        }
    }
    */
}

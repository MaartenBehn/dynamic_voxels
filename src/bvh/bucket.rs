use std::cell::RefCell;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

pub const NUM_BUCKETS: usize = 6;

thread_local! {
    /// Thread local for the buckets used while building to reduce allocations during build
    pub static BUCKETS: RefCell<[Vec<usize>; NUM_BUCKETS]> = RefCell::new(Default::default());
}

#[derive(Clone, Copy)]
pub struct Bucket<V: Ve<T, D>, T: Nu, const D: usize> {
    /// The number of shapes in this [`Bucket`].
    pub size: usize,

    /// The joint [`Aabb`] of the shapes in this [`Bucket`].
    pub aabb: AABB<V, T, D>,

    /// The [`Aabb`] of the centers of the shapes in this [`Bucket`]
    pub centroid: AABB<V, T, D>,
}

impl<V: Ve<T, D>, T: Nu, const D: usize> Bucket<V, T, D> {
    /// Returns an empty bucket.
    pub fn empty() -> Bucket<V, T, D> {
        Bucket {
            size: 0,
            aabb: AABB::default(),
            centroid: AABB::default(),
        }
    }

    /// Extend this [`Bucket`] by a shape with the given [`Aabb`].
    pub fn add_aabb(&mut self, aabb: AABB<V, T, D>) {
        self.size += 1;
        self.aabb.union_mut(aabb);
        self.centroid.union_point_mut(aabb.center());
    }

    /// Join the contents of two [`Bucket`]'s.
    pub fn join_bucket(a: Bucket<V, T, D>, b: &Bucket<V, T, D>) -> Bucket<V, T, D> {
        Bucket {
            size: a.size + b.size,
            aabb: a.aabb.union(b.aabb),
            centroid: a.centroid.union(b.centroid),
        }
    }
}


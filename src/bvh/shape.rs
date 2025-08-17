use std::marker::PhantomData;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

pub struct Shapes<'a, S: BHShape<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> {
    shapes: &'a [S],
    p1: PhantomData<V>,
    p2: PhantomData<T>,
}

pub trait BHShape<V: Ve<T, D>, T: Nu, const D: usize>: Send + Sync + Sized {
    fn aabb(&self, shapes: &Shapes<Self, V, T, D>) -> AABB<V, T, D>;
}

impl<S: BHShape<V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> Shapes<'_, S, V, T, D> {
    /// Returns a reference to the Shape found at shape_index.
    pub fn get(&self, shape_index: usize) -> &S {
        &self.shapes[shape_index]
    }

    pub fn aabb(&self, shape_index: usize) -> AABB<V, T, D> {
        self.shapes[shape_index].aabb(self)
    }

    /// Creates a [`Shapes`] that inherits its lifetime from the slice.
    pub(crate) fn from_slice(slice: &[S]) -> Shapes<S, V, T, D>
    {
        Shapes { shapes: &slice, p1: Default::default(), p2: Default::default() }
    }
}


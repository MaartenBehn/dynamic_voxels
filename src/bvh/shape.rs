use std::marker::PhantomData;

use crate::util::{aabb::AABB, number::Nu, vector::Ve};

pub struct Shapes<'a, E, S: BHShape<E, V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> {
    shapes: &'a [S],
    p0: PhantomData<E>,
    p1: PhantomData<V>,
    p2: PhantomData<T>,
}

pub trait BHShape<E, V: Ve<T, D>, T: Nu, const D: usize>: Sized {
    fn aabb(&self, shapes: &Shapes<E, Self, V, T, D>) -> AABB<V, T, D>;
    fn extra_data(&self, shapes: &Shapes<E, Self, V, T, D>) -> E;
}

impl<E, S: BHShape<E, V, T, D>, V: Ve<T, D>, T: Nu, const D: usize> Shapes<'_, E, S, V, T, D> {
    /// Returns a reference to the Shape found at shape_index.
    pub fn get(&self, shape_index: usize) -> &S {
        &self.shapes[shape_index]
    }

    pub fn aabb(&self, shape_index: usize) -> AABB<V, T, D> {
        self.shapes[shape_index].aabb(self)
    }

    /// Creates a [`Shapes`] that inherits its lifetime from the slice.
    pub(crate) fn from_slice(slice: &[S]) -> Shapes<'_, E, S, V, T, D>
    {
        Shapes { 
            shapes: &slice, 
            p0: PhantomData, 
            p1: PhantomData, 
            p2: PhantomData 
        }
    }
}


use std::marker::PhantomData;

use crate::util::{number::Nu, vector::Ve};

use super::traits::BHShape;

/// Shapes holds a mutable ptr to the slice of Shapes passed in to build. It is accessed only through a ShapeIndex.
/// These are a set of unique indices into Shapes that are generated at the start of the build process. Because they
/// are all unique they guarantee that access into Shapes is safe to do in parallel.
pub struct Shapes<'a, S> {
    ptr: *mut S,
    len: usize,
    marker: PhantomData<&'a S>,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
/// ShapeIndex represents an entry into the Shapes struct. It is used to help ensure that we are only accessing Shapes
/// with unique indices.
pub(crate) struct ShapeIndex(pub usize);

impl<S> Shapes<'_, S> {
    /// Calls set_bh_node_index on the Shape found at shape_index.
    pub(crate) fn set_node_index<V: Ve<T, D>,  T: Nu, const D: usize>(
        &self,
        shape_index: ShapeIndex,
        node_index: usize,
    ) where
        S: BHShape<V, T, D>,
    {
        assert!(shape_index.0 < self.len);
        unsafe {
            self.ptr
                .add(shape_index.0)
                .as_mut()
                .unwrap()
                .set_bh_node_index(node_index);
        }
    }

    /// Returns a reference to the Shape found at shape_index.
    pub(crate) fn get<V: Ve<T, D>,  T: Nu, const D: usize>(&self, shape_index: ShapeIndex) -> &S
    where
        S: BHShape<V, T, D>,
    {
        assert!(shape_index.0 < self.len);
        unsafe { self.ptr.add(shape_index.0).as_ref().unwrap() }
    }

    /// Creates a [`Shapes`] that inherits its lifetime from the slice.
    pub(crate) fn from_slice<V: Ve<T, D>,  T: Nu, const D: usize>(slice: &mut [S]) -> Shapes<S>
    where
        S: BHShape<V, T, D>,
    {
        Shapes {
            ptr: slice.as_mut_ptr(),
            len: slice.len(),
            marker: PhantomData,
        }
    }
}

unsafe impl<S: Send> Send for Shapes<'_, S> {}
unsafe impl<S> Sync for Shapes<'_, S> {}

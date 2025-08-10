use octa_force::OctaResult;

pub mod buddy_buffer_allocator;
pub mod cached_vec;
pub mod parallel_vec;

pub trait DataBuffer<T> {
    fn push(&self, values: &[T]) -> OctaResult<u32>;
}


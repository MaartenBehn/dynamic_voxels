use crate::util::{aabb::AABB, number::Nu, vector::Ve};


pub trait BHShape<V: Ve<T, D>, T: Nu, const D: usize> {
    fn set_bh_node_index(&mut self, _: usize);
    fn bh_node_index(&self) -> usize;
    fn aabb(&self) -> AABB<V, T, D>;
}

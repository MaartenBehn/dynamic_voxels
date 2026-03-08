use crate::{csg::csg_tree::tree::CSGTree, util::{number::Nu, vector::Ve}, volume::VolumeChangeBounds};


impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeChangeBounds<V, T, D> for CSGTree<M, V, T, D> {
    fn calculate_change_bounds(&mut self) {
    }

    fn get_change_bounds(&self) -> crate::util::aabb::AABB<V, T, D> {
        self.changed_bounds
    }
}

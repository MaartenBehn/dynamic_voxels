use crate::{csg::{Base, csg_tree::tree::{CSGTree, CSGTreeIndex, CSGTreeNodeData}}, util::{aabb::AABB, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeChangeBounds}};


impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeChangeBounds<V, T, D> for CSGTree<M, V, T, D> {
    fn calculate_change_bounds(&mut self) {
    }

    fn get_change_bounds(&self) -> AABB<V, T, D> {
        self.changed_bounds
    }
}

impl<M: Base + Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {  
    pub fn reset_changed_bounds(&mut self) {
        self.changed_bounds = Default::default();
    }

    pub fn get_mat(&self, index: CSGTreeIndex) -> V::Matrix {
        match &self.nodes[index].data {
            CSGTreeNodeData::Box(csgbox) => csgbox.get_mat(),
            CSGTreeNodeData::Sphere(csgsphere) => csgsphere.get_mat(),
            CSGTreeNodeData::OffsetVoxelGrid(offset_voxel_grid) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
            _ => unreachable!()
        }
    }

    pub fn set_mat(&mut self, index: CSGTreeIndex, mat: V::Matrix) {
        let new_change_bounds = match &mut self.nodes[index].data {
            CSGTreeNodeData::Box(csgbox) => {
                let aabb = csgbox.get_bounds();
                csgbox.set_mat(mat);
                aabb.union(csgbox.get_bounds())
            },
            CSGTreeNodeData::Sphere(csgsphere) => {
                let aabb = csgsphere.get_bounds();
                csgsphere.set_mat(mat);
                aabb.union(csgsphere.get_bounds())
            },
            CSGTreeNodeData::OffsetVoxelGrid(offset_voxel_grid) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
            _ => unreachable!()
        };

        self.changed_bounds = self.changed_bounds.union(new_change_bounds);
        self.calculate_bounds_parents(index);
    }
}

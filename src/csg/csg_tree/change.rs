use crate::{csg::{Base, csg_tree::tree::{CSGTree, CSGTreeIndex, CSGTreeNodeData}}, util::{aabb::AABB, matrix::Ma, number::Nu, vector::Ve}, volume::{VolumeBounds, VolumeChangeBounds}};


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
            CSGTreeNodeData::Box(d) => d.get_mat().cast(),
            CSGTreeNodeData::Sphere(d) => d.get_mat().cast(),
            CSGTreeNodeData::Cylinder(d) => d.get_mat().cast(),
            CSGTreeNodeData::OffsetVoxelGrid(offset_voxel_grid) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
            _ => unreachable!()
        }
    }

    pub fn set_mat(&mut self, index: CSGTreeIndex, mat: V::Matrix) {
        let new_change_bounds = match &mut self.nodes[index].data {
            CSGTreeNodeData::Box(d) => {
                let aabb = d.get_bounds();
                d.set_mat(mat.cast());
                aabb.union(d.get_bounds())
            },
            CSGTreeNodeData::Sphere(d) => {
                let aabb = d.get_bounds();
                d.set_mat(mat.cast());
                aabb.union(d.get_bounds())
            },
            CSGTreeNodeData::Cylinder(d) => {
                let aabb = d.get_bounds();
                d.set_mat(mat.cast());
                aabb.union(d.get_bounds())
            },
            CSGTreeNodeData::OffsetVoxelGrid(offset_voxel_grid) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(shared_voxel_grid) => todo!(),
            _ => unreachable!()
        };

        self.changed_bounds = self.changed_bounds.union(new_change_bounds);
        self.calculate_bounds_parents(index);
    }
}

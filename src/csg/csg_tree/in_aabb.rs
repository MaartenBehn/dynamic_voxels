use crate::{util::{aabb::AABB, math_config::MC}, volume::{VolumeQureyAABB, VolumeQureyAABBResult}, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{remove::CSGTreeRemove, tree::{CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::CSGTreeUnion};

impl<C: MC<D>, const D: usize> VolumeQureyAABB<C::Vector, C::Number, D> for CSGTree<u8, C, D> {
    fn get_aabb_value(&self, aabb: AABB<C::Vector, C::Number, D>) -> VolumeQureyAABBResult {
        self.get_aabb_value_index(self.root, aabb)
    }
}

impl<C: MC<D>, const D: usize> CSGTree<u8, C, D> {
    fn get_aabb_value_index(&self, index: CSGTreeIndex, aabb: AABB<C::Vector, C::Number, D>) -> VolumeQureyAABBResult {

        let node = &self.nodes[index];
        match &node.data {
            CSGTreeNodeData::Union(d) => self.get_aabb_value_union(d, aabb),
            CSGTreeNodeData::Remove(d) => self.get_aabb_value_remove(d, aabb),

            CSGTreeNodeData::Box(d) => d.get_aabb_value(aabb),
            CSGTreeNodeData::Sphere(d) => d.get_aabb_value(aabb),
            CSGTreeNodeData::OffsetVoxelGrid(d) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(d) => todo!(),
        }
    }

    fn get_aabb_value_union(&self, union: &CSGTreeUnion<C, D>, aabb: AABB<C::Vector, C::Number, D>) -> VolumeQureyAABBResult {
        let mut i = 0;
        while i < union.bvh.nodes.len() {
            let b = &union.bvh.nodes[i];
            if b.aabb.collides_aabb(aabb) {
                if let Some(leaf) = b.leaf {
                    let v = self.get_aabb_value_index(leaf, aabb); 
                    if !matches!(v, VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)) {
                        return v;
                    }
                }

                i += 1;
            } else {
                i = b.exit;
            }
        }

        VolumeQureyAABBResult::Full(MATERIAL_ID_NONE)
    }

    fn get_aabb_value_remove(&self, remove: &CSGTreeRemove, aabb: AABB<C::Vector, C::Number, D>) -> VolumeQureyAABBResult {
        let base = self.get_aabb_value_index(remove.base, aabb);
        let remove = self.get_aabb_value_index(remove.remove, aabb);

        if matches!(base, VolumeQureyAABBResult::Mixed) {
            if matches!(remove, VolumeQureyAABBResult::Mixed) {
                return VolumeQureyAABBResult::Mixed;
            } else if remove.get_value() != 0 {
                return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
            } else {
                return VolumeQureyAABBResult::Mixed
            }
        }

        let a = base.get_value();
        if a == 0 {
            return VolumeQureyAABBResult::Full(MATERIAL_ID_NONE);
        }

        if matches!(remove, VolumeQureyAABBResult::Mixed) {
            return VolumeQureyAABBResult::Mixed;
        }

        let b = remove.get_value();
        if b != 0 { VolumeQureyAABBResult::Full(MATERIAL_ID_NONE) }
        else { VolumeQureyAABBResult::Full(a) }
    }
}

use crate::{util::math_config::MC, volume::VolumeQureyPosValue, voxel::palette::palette::MATERIAL_ID_NONE};

use super::{remove::CSGTreeRemove, tree::{CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::CSGTreeUnion};


impl<C: MC<D>, const D: usize> VolumeQureyPosValue<C::Vector, C::Number, D> for CSGTree<u8, C, D> { 
    fn get_value(&self, pos: C::Vector) -> u8 {
        todo!()
    }
}

impl<C: MC<D>, const D: usize> CSGTree<u8, C, D> {
    fn get_value_index(&self, index: CSGTreeIndex, pos: C::Vector) -> u8 {
        let node = &self.nodes[index];
        match &node.data {
            CSGTreeNodeData::Union(d) => self.get_value_union(d, pos),
            CSGTreeNodeData::Remove(d) => self.get_value_remove(d, pos),
            
            CSGTreeNodeData::Box(d) => d.get_value(pos),
            CSGTreeNodeData::Sphere(d) => d.get_value(pos),
            CSGTreeNodeData::OffsetVoxelGrid(d) => d.get_value(pos),
            CSGTreeNodeData::SharedVoxelGrid(d) => d.get_value(pos),
        }
    }

    fn get_value_union(&self, union: &CSGTreeUnion<C, D>, pos: C::Vector) -> u8 {
        let mut i = 0;
        while i < union.flat_bvh.len() {
            let b = &union.flat_bvh[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let v = self.get_value_index(leaf, pos); 
                    if v != MATERIAL_ID_NONE {
                        return v;
                    }
                }

                i += 1;
            } else {
                i = b.exit;
            }
        }

        MATERIAL_ID_NONE
    }

    fn get_value_remove(&self, remove: &CSGTreeRemove, pos: C::Vector) -> u8 {
        let base = self.get_value_index(remove.base, pos);
        let remove = self.get_value_index(remove.remove, pos);

        if remove != MATERIAL_ID_NONE || base == MATERIAL_ID_NONE { 
            MATERIAL_ID_NONE 
        } else { 
            base
        }
    }
}

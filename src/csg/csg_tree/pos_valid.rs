use crate::{util::math_config::MC, volume::VolumeQureyPosValid};

use super::{remove::CSGTreeRemove, tree::{CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::CSGTreeUnion};


impl<V: Send + Sync, C: MC<D>, const D: usize> VolumeQureyPosValid<C::Vector, C::Number, D> for CSGTree<V, C, D> {
    fn is_position_valid(&self, pos: C::Vector) -> bool {
        self.is_position_valid_index(self.root, pos)
    }
}

impl<V: Send + Sync, C: MC<D>, const D: usize> CSGTree<V, C, D> {
    fn is_position_valid_index(&self, index: CSGTreeIndex, pos: C::Vector) -> bool {
        let node = &self.nodes[index];
        match &node.data {
            CSGTreeNodeData::Union(d) => self.is_position_valid_union(d, pos),
            CSGTreeNodeData::Remove(d) => self.is_position_valid_remove(d, pos),
            
            CSGTreeNodeData::Box(d) => d.is_position_valid(pos),
            CSGTreeNodeData::Sphere(d) => d.is_position_valid(pos),
            CSGTreeNodeData::OffsetVoxelGrid(d) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(d) => todo!(),
        }
    }

    fn is_position_valid_union(&self, union: &CSGTreeUnion<C, D>, pos: C::Vector) -> bool {
        let mut i = 0;
        while i < union.bvh.nodes.len() {
            let b = &union.bvh.nodes[i];
            if b.aabb.pos_in_aabb(pos) {
                if let Some(leaf) = b.leaf {
                    let v = self.is_position_valid_index(leaf, pos); 
                    if v {
                        return true;
                    }
                }

                i += 1;
            } else {
                i = b.exit;
            }
        }

        false
    }

    fn is_position_valid_remove(&self, remove: &CSGTreeRemove, pos: C::Vector) -> bool {
        let base = self.is_position_valid_index(remove.base, pos);
        let remove = self.is_position_valid_index(remove.remove, pos);

        base && !remove
    }
}

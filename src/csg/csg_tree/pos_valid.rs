use crate::{util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

use super::{remove::CSGTreeRemove, tree::{CSGTreeNodeData, CSGTree, CSGTreeIndex}, union::CSGTreeUnion};


impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> VolumeQureyPosValid<V, T, D> for CSGTree<M, V, T, D> {
    fn is_position_valid(&self, pos: V) -> bool {
        self.is_position_valid_index(self.root, pos)
    }
}

impl<M: Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {
    fn is_position_valid_index(&self, index: CSGTreeIndex, pos: V) -> bool {
        let node = &self.nodes[index];
        match &node.data {
            CSGTreeNodeData::None => false,
            CSGTreeNodeData::Union(d) => self.is_position_valid_union(d, pos),
            CSGTreeNodeData::Cut(d) => self.is_position_valid_remove(d, pos),
            
            CSGTreeNodeData::Box(d) => d.is_position_valid(pos),
            CSGTreeNodeData::Sphere(d) => d.is_position_valid(pos),
            CSGTreeNodeData::OffsetVoxelGrid(d) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(d) => todo!(),
        }
    }

    fn is_position_valid_union(&self, union: &CSGTreeUnion<V, T, D>, pos: V) -> bool {
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

    fn is_position_valid_remove(&self, remove: &CSGTreeRemove, pos: V) -> bool {
        let base = self.is_position_valid_index(remove.base, pos);
        let remove = self.is_position_valid_index(remove.remove, pos);

        base && !remove
    }
}

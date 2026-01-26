use crate::{csg::csg_tree::{remove::CSGTreeRemove, tree::{CSGTree, CSGTreeIndex, CSGTreeNodeData}, union::CSGTreeUnion}, util::{number::Nu, vector::Ve}, volume::VolumeGradient};

impl<M, V: Ve<T, D>, T: Nu, const D: usize> VolumeGradient<V::VectorF, D> for CSGTree<M, V, T, D> {
    fn get_gradient_at_position(&self, pos: V::VectorF) -> V::VectorF {
        self.get_gradient_at_position_internal(self.root, pos)
    }
}

impl<M, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {
    fn get_gradient_at_position_internal(&self, index: CSGTreeIndex, pos: V::VectorF) -> V::VectorF {
        let node = &self.nodes[index];
        match &node.data {
            CSGTreeNodeData::None => V::VectorF::ZERO,
            CSGTreeNodeData::Union(d) => self.get_gradient_at_position_union(d, pos),
            CSGTreeNodeData::Cut(d) => self.get_gradient_at_position_union_remove(d, pos),
            
            CSGTreeNodeData::Box(d) => d.get_gradient_at_position(pos),
            CSGTreeNodeData::Sphere(d) => d.get_gradient_at_position(pos),
            CSGTreeNodeData::OffsetVoxelGrid(d) => todo!(),
            CSGTreeNodeData::SharedVoxelGrid(d) => todo!(),
        }
    }

    fn get_gradient_at_position_union(&self, union: &CSGTreeUnion<V, T, D>, pos: V::VectorF) -> V::VectorF {

        let mut max_grad = V::VectorF::MAX;
        for index in union.indecies.iter() {
            let grad = self.get_gradient_at_position_internal(*index, pos);

            if grad.length_squared() < max_grad.length_squared() {
                max_grad = grad;
            }
        }

        max_grad
    }

    fn get_gradient_at_position_union_remove(&self, remove: &CSGTreeRemove, pos: V::VectorF) -> V::VectorF {
        todo!()        
    }
}

use super::tree::{CSGNodeData2D, CSGTree2D, CSGTreeKey2D};

impl<T> CSGTree2D<T> {
    pub fn get_id_parents(&self, ids: &[CSGTreeKey2D]) -> Vec<CSGTreeKey2D> {
        self.nodes
            .iter()
            .filter_map(|(i, node)| {
                match node.data {
                    CSGNodeData2D::Union(child1, child2)
                    | CSGNodeData2D::Remove(child1, child2)
                    | CSGNodeData2D::Intersect(child1, child2) => {
                        if ids.contains(&child1) || ids.contains(&child2) {
                            return Some(i);
                        }
                    }
                    _ => {}
                }

                None
            })
            .collect()
    }
}

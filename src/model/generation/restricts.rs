use super::{collapse::{CollapseNodeKey, CollapseOperation, Collapser}, template::TemplateTree, traits::ModelGenerationTypes};

impl<T: ModelGenerationTypes> Collapser<T> {
    pub fn push_restricts_collapse_opperations(&mut self, node_index: CollapseNodeKey, template: &TemplateTree<T>) {
        let node = &self.nodes[node_index];

        for (identifier, index) in node.restricts.iter() {
            self.pending_collapse_opperations.push_back(CollapseOperation::RestrictHook { 
                index: node_index, 
                identifier: node.identifier, 
                restricts_index: *index, 
                restricts_identifier: *identifier, 
            });
        }
    }

}

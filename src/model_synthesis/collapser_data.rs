use slotmap::SlotMap;

use crate::volume::Volume;

use super::{builder::{BU, IT}, collapse::{CollapseNodeKey, CollapseOperation, Collapser, CollapseNode, NodeOperation}, template::TemplateTree};

#[derive(Clone, Debug)]
pub struct CollapserData< I: IT, U: BU> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<I, U>>,
    pub pending_operations: Vec<NodeOperation>,
    pub pending_collapse_opperations: Vec<CollapseOperation<I, U>>,
}

impl<I: IT, U: BU> CollapserData<I, U> {
    pub fn into_collapser<'a, V: Volume>(self, template: &'a TemplateTree<I, V>) -> Collapser<'a, I, U, V> {
        Collapser { 
            template, 
            nodes: self.nodes, 
            pending_operations: self.pending_operations, 
            pending_collapse_opperations: self.pending_collapse_opperations 
        }
    }
}

impl<'a, I: IT, U: BU, V: Volume> Collapser<'a, I, U, V> {
    pub fn into_data(self) -> CollapserData<I, U> {
        CollapserData { 
            nodes: self.nodes, 
            pending_operations: self.pending_operations, 
            pending_collapse_opperations: self.pending_collapse_opperations 
        }
    }
}

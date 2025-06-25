use slotmap::SlotMap;

use crate::volume::VolumeQureyPos;

use super::{builder::{BU, IT}, collapse::{CollapseNode, CollapseNodeKey, CollapseOperation, Collapser}, pending_operations::PendingOperations, template::TemplateTree};

#[derive(Clone, Debug)]
pub struct CollapserData< I: IT, U: BU, V: VolumeQureyPos> {
    pub nodes: SlotMap<CollapseNodeKey, CollapseNode<I, U, V>>,
    pub pending_operations: PendingOperations,
    pub pending_collapse_opperations: Vec<CollapseOperation<I, U>>,
}

impl<I: IT, U: BU, V: VolumeQureyPos> CollapserData<I, U, V> {
    pub fn into_collapser<'a>(self, template: &'a TemplateTree<I, V>) -> Collapser<'a, I, U, V> {
        Collapser { 
            template, 
            nodes: self.nodes, 
            pending_operations: self.pending_operations, 
            pending_collapse_opperations: self.pending_collapse_opperations 
        }
    }
}

impl<'a, I: IT, U: BU, V: VolumeQureyPos> Collapser<'a, I, U, V> {
    pub fn into_data(self) -> CollapserData<I, U, V> {
        CollapserData { 
            nodes: self.nodes, 
            pending_operations: self.pending_operations, 
            pending_collapse_opperations: self.pending_collapse_opperations 
        }
    }
}

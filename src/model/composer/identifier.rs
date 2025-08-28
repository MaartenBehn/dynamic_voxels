use egui_snarl::NodeId;

#[derive(Debug, Clone, Copy)]
pub struct Identifier {
    node_id: NodeId,
}

impl Identifier {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }
}



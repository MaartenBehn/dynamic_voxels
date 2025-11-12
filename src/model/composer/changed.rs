use egui_snarl::{NodeId, OutPinId, Snarl};
use octa_force::OctaResult;

use crate::util::{number::Nu, vector::Ve};

use super::{build::BS, nodes::ComposeNode, viewer::ComposeViewer};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeViewer<V2, V3, T, B> {
    pub fn set_added(&mut self, node_id: NodeId, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        if self.added_nodes.len() <= node_id.0 {
            self.added_nodes.resize(node_id.0 + 1, false);
            self.changed_nodes.resize(node_id.0 + 1, false);
            self.invalid_nodes.resize(node_id.0 + 1, false);
            self.needs_collapse_nodes.resize(node_id.0 + 1, false);
        }

        self.added_nodes.set(node_id.0, true);
        self.set_needs_collapse(node_id, snarl);
    }

    pub fn set_changed(&mut self, node_id: NodeId, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        if self.added_nodes.get(node_id.0).as_deref().copied().unwrap_or(false) {
            return;
        }

        self.changed_nodes.set(node_id.0, true);
        self.set_needs_collapse(node_id, snarl);
    }

    pub fn set_deleted(&mut self, node_id: NodeId) {
        self.added_nodes.set(node_id.0, false);
        self.changed_nodes.set(node_id.0, false);

        if !self.deleted_nodes.contains(&node_id) {
            self.deleted_nodes.push(node_id);
        }
    }

    pub fn set_needs_collapse(&mut self, node_id: NodeId, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
       
        if self.needs_collapse_nodes.len() <= node_id.0 {
            self.added_nodes.resize(node_id.0 + 1, false);
            self.changed_nodes.resize(node_id.0 + 1, false);
            self.invalid_nodes.resize(node_id.0 + 1, false);
            self.needs_collapse_nodes.resize(node_id.0 + 1, false);
        } 

        if *self.needs_collapse_nodes.get(node_id.0).as_deref().unwrap() {
            return;
        }

        self.needs_collapse_nodes.set(node_id.0, true);
        
        let node = snarl.get_node(node_id)
            .expect("NodeId was not valid")
            .to_owned();

        for (i, output) in node.outputs.iter().enumerate() {
            let out_pin = snarl.out_pin(OutPinId { node: node.id, output: i });

            for remote in out_pin.remotes {
                self.set_needs_collapse(remote.node, snarl);
            }       
        }
    }
}

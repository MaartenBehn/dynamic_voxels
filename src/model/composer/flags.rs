use bitvec::vec::BitVec;
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use itertools::Itertools;
use smallvec::SmallVec;

use crate::model::{composer::nodes::{ComposeNode, ComposeNodeInput, ComposeNodeOutput}, data_types::data_type::{ComposeDataType, ComposeNodeType}};

#[derive(Debug)]
pub struct ComposerNodeFlags { 
    changed_nodes: BitVec,
    needs_collapse_nodes: BitVec,
    invalid_nodes: BitVec,
    cam_nodes: SmallVec<[NodeId; 4]>,
}

impl ComposerNodeFlags {
    pub fn new(snarl: &mut Snarl<ComposeNode>) -> Self {
        let mut flags = Self {
            changed_nodes: BitVec::new(),
            needs_collapse_nodes: BitVec::new(),
            invalid_nodes: BitVec::new(),
            cam_nodes: SmallVec::new(),
        };


        for node in snarl.nodes().cloned().collect_vec() {
            let node_id = node.id; 
            flags.enshure_nodes_list_index(node_id.0);

            match node.t {
                ComposeNodeType::CamPosition => {
                    flags.cam_nodes.push(node_id);
                }
                _ => {}
            }

            let valid = flags.validate_node(node, snarl);
            flags.invalid_nodes.set(node_id.0, !valid);
        }

        flags
    }

    pub fn reset_change_flags(&mut self) {
        self.changed_nodes.clear();
        self.needs_collapse_nodes.clear();
    } 

    pub fn enshure_nodes_list_index(&mut self, i: usize) {
        if self.changed_nodes.len() <= i {
            self.changed_nodes.resize(i + 1, false);
            self.invalid_nodes.resize(i + 1, false);
            self.needs_collapse_nodes.resize(i + 1, false);
        }
    }

    pub fn is_changed(&self, node_id: NodeId) -> bool {
        self.changed_nodes.get(node_id.0).as_deref().copied().unwrap_or(false)
    }

    pub fn needs_collapse(&self, node_id: NodeId) -> bool {
        self.needs_collapse_nodes.get(node_id.0).as_deref().copied().unwrap_or(false)
    }

    pub fn iter_changed(&self) -> impl Iterator<Item=NodeId> {
        self.changed_nodes.iter_ones().map(|i| NodeId(i))
    }

    pub fn iter_changed_all(&self) -> impl Iterator<Item=(NodeId, bool)> {
        self.changed_nodes.iter().by_vals().enumerate().map(|(i, v)| (NodeId(i), v))
    }

    pub fn set_changed(&mut self, node_id: NodeId, snarl: &Snarl<ComposeNode>) {
        self.enshure_nodes_list_index(node_id.0);

        self.changed_nodes.set(node_id.0, true);
        self.set_needs_collapse(node_id, snarl);
    }

    pub fn set_needs_collapse(&mut self, node_id: NodeId, snarl: &Snarl<ComposeNode>) {
        self.enshure_nodes_list_index(node_id.0);

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

    pub fn needs_collapse_any(&self) -> bool {
        self.needs_collapse_nodes.any()
    }

    pub fn set_cam_notes_as_changed(&mut self, snarl: &Snarl<ComposeNode>) {
        for node_id in self.cam_nodes.to_owned() {
            self.set_changed(node_id, snarl);
        }
    }

    pub fn add_cam_note(&mut self, node_id: NodeId) {
        if !self.cam_nodes.contains(&node_id) {
            self.cam_nodes.push(node_id);
        }
    }

    pub fn remove_cam_note(&mut self, node_id: NodeId) {
        let res = self.cam_nodes.iter().position(|id| *id == node_id);
        if let Some(i) = res {
            self.cam_nodes.swap_remove(i);
        }
    }

    pub fn are_all_valid(&self) -> bool {
        !self.invalid_nodes.any()
    }

    pub fn check_valid_for_all_nodes(&mut self, snarl: &mut Snarl<ComposeNode>) {
        let nodes = snarl.nodes().cloned().collect_vec();

        for node in nodes {
            let i = node.id.0;
            self.enshure_nodes_list_index(i);

            let valid =  self.validate_node(node, snarl);
            self.invalid_nodes.set(i, !valid);
        }
    }

    pub fn update_valid_for_all_invalid_nodes(&mut self, snarl: &mut Snarl<ComposeNode>) {
        for i in self.invalid_nodes.iter_ones().collect_vec() {
            if let Some(node) = snarl.get_node(NodeId(i)) {
                let node = node.to_owned();

                if self.validate_node(node, snarl) {
                    self.invalid_nodes.set(i, false);
                }             
            } else {
                self.invalid_nodes.set(i, false);
            }
        }
    }

    pub fn update_node_valid(&mut self, node_id: NodeId, snarl: &mut Snarl<ComposeNode>) {
        let node = snarl.get_node(node_id)
            .expect("NodeId was not valid")
            .to_owned();

        let valid = self.validate_node(node, snarl);
        self.invalid_nodes.set(node_id.0, !valid);
    }

    pub fn validate_node(&self, node: ComposeNode, snarl: &mut Snarl<ComposeNode>) -> bool {

        let mut valid = true;
        for (i, input) in node.inputs.iter().enumerate() {
            let in_pin = snarl.in_pin(InPinId { node: node.id, input: i });
            
            if in_pin.remotes.is_empty() {
                let node = snarl.get_node_mut(node.id).unwrap();
                let input: &mut ComposeNodeInput = &mut node.inputs[i];

                match &input.data_type {
                    ComposeDataType::Number(_)
                    | ComposeDataType::Position2D(_)
                    | ComposeDataType::Position3D(_) 
                    | ComposeDataType::Creates 
                    | ComposeDataType::Material(_) => {
                        input.valid = true;
                    },
                    _ => {
                        input.valid = false;
                        valid = false;
                    }
                }
            } else if (in_pin.remotes.len() == 1) {
                let remote = in_pin.remotes[0];
                let out_pin = snarl.out_pin(remote);
                let remote_node = snarl.get_node(remote.node)
                    .expect("Node of Remote not found");

                let input_data_type = input.data_type;
                let output_data_type = remote_node.outputs[remote.output].data_type;

                let node = snarl.get_node_mut(node.id).unwrap();
                let input = &mut node.inputs[i];

                if input_data_type == output_data_type {
                    input.valid = true;
                } else {
                    input.valid = false;
                    valid = false;
                }
            } else {
                let node = snarl.get_node_mut(node.id).unwrap();
                let input: &mut ComposeNodeInput = &mut node.inputs[i];

                input.valid = false;
                valid = false;
            }
        }

        for (i, output) in node.outputs.iter().enumerate() {
            let out_pin = snarl.out_pin(OutPinId { node: node.id, output: i });

            let mut output_valid = true;
            let output_data_type = output.data_type;

            for remote in out_pin.remotes {
                let in_pin = snarl.in_pin(remote);
                let remote_node = snarl.get_node(remote.node)
                    .expect("Node of Remote not found");

                let input_data_type = remote_node.inputs[remote.input].data_type;

                if input_data_type != output_data_type {
                    output_valid = false;
                }
            }      

            let node = snarl.get_node_mut(node.id).unwrap();
            let output: &mut ComposeNodeOutput = &mut node.outputs[i];

            if output_valid {
                output.valid = true;
            } else {
                output.valid = false;
                valid = false;
            } 

        }

        return valid;
    }
}



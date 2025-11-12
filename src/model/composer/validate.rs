use egui_snarl::{ui::{PinWireInfo, WireStyle}, InPin, InPinId, NodeId, OutPinId, Snarl};
use itertools::Itertools;
use octa_force::egui::{self, epaint::{CircleShape, PathShape, PathStroke}, Color32, Shape};

use crate::{model::data_types::data_type::ComposeDataType, util::{number::Nu, vector::Ve}};

use super::{build::BS, nodes::{ComposeNode, ComposeNodeInput, ComposeNodeOutput}, viewer::ComposeViewer, ModelComposer};



impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeViewer<V2, V3, T, B> {
    pub fn check_valid_for_all_nodes(&mut self, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        let nodes = snarl.nodes().cloned().collect_vec();

        for node in nodes {
            let i = node.id.0;
            if self.invalid_nodes.len() <= i {
                self.added_nodes.resize(i + 1, false);
                self.changed_nodes.resize(i + 1, false);
                self.invalid_nodes.resize(i + 1, false);
            }

            let valid =  self.validate_node(node, snarl);
            self.invalid_nodes.set(i, !valid);
        }
    }

    pub fn update_valid_for_all_invalid_nodes(&mut self, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
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

    pub fn update_node_valid(&mut self, node_id: NodeId, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        let node = snarl.get_node(node_id)
            .expect("NodeId was not valid")
            .to_owned();

        let valid =  self.validate_node(node, snarl);
        self.invalid_nodes.set(node_id.0, !valid);
    }

    fn validate_node(&self, node: ComposeNode<B::ComposeType>, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) -> bool {

        let mut valid = true;
        for (i, input) in node.inputs.iter().enumerate() {
            let in_pin = snarl.in_pin(InPinId { node: node.id, input: i });
            
            if in_pin.remotes.is_empty() {
                let node = snarl.get_node_mut(node.id).unwrap();
                let input: &mut ComposeNodeInput = &mut node.inputs[i];

                match &input.data_type {
                    ComposeDataType::Number(_)
                    | ComposeDataType::Position2D(_)
                    | ComposeDataType::Position3D(_) => {
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
                let input: &mut ComposeNodeInput = &mut node.inputs[i];

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



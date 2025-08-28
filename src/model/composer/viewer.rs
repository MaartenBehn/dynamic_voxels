use egui_snarl::{ui::{AnyPins, PinInfo, SnarlViewer}, InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use itertools::Itertools;
use octa_force::{egui::{self, Color32, DragValue, Ui}, glam::{Vec2, Vec3A}};

use super::{data_type::ComposeDataType, nodes::{get_node_templates, ComposeNode, ComposeNodeInput, ComposeNodeOutput, ComposeNodeType}};


#[derive(Debug)]
pub struct ComposeViewer {
    pub node_templates: Vec<ComposeNode>,
}

impl ComposeViewer {
    pub fn new() -> Self {
        let node_templates = get_node_templates();
        Self { node_templates }
    } 
}

impl SnarlViewer<ComposeNode> for ComposeViewer {
    fn title(&mut self, node: &ComposeNode) -> String { 
        node.title() 
    }

    fn inputs(&mut self, node: &ComposeNode) -> usize { 
        node.inputs.len() 
    }

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut octa_force::egui::Ui,
        snarl: &mut egui_snarl::Snarl<ComposeNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let input = &mut snarl[pin.id.node].inputs[pin.id.input];
        
        ui.label(input.name.to_string());

        // Show input fields for number if nothing is connected.
        if pin.remotes.is_empty() {
            match &mut input.data_type {
                ComposeDataType::Number(d) => { 
                    let mut v = d.unwrap_or(0.0);
                    if ui.add(DragValue::new(&mut v)).changed() {
                        (*d) = Some(v);
                    }
                },
                ComposeDataType::Position2D(d) => { 
                    let mut v = d.unwrap_or(Vec2::ZERO);

                    ui.label("x:");
                    if ui.add(DragValue::new(&mut v.x)).changed() {
                        (*d) = Some(v);
                    }
                    ui.label("y:");
                    if ui.add(DragValue::new(&mut v.y)).changed() {
                        (*d) = Some(v);
                    }
                },
                ComposeDataType::Position3D(d) => {
                    let mut v = d.unwrap_or(Vec3A::ZERO);

                    ui.label("x:");
                    if ui.add(DragValue::new(&mut v.x)).changed() {
                        (*d) = Some(v);
                    }

                    ui.label("y:");
                    if ui.add(DragValue::new(&mut v.y)).changed() {
                        (*d) = Some(v);
                    }

                    ui.label("z:");
                    if ui.add(DragValue::new(&mut v.z)).changed() {
                        (*d) = Some(v);
                    }
                },
                _ => {},
            }
        }
        
        input.data_type.get_pin()
    }

    fn outputs(&mut self, node: &ComposeNode) -> usize {
        node.outputs.len()
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut octa_force::egui::Ui,
        snarl: &mut egui_snarl::Snarl<ComposeNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &mut snarl[pin.id.node]; 
        let output = &mut node.outputs[pin.id.output];

        ui.add_space(8.0);
        
        match node.t {
            ComposeNodeType::Number => match &mut output.data_type {
                ComposeDataType::Number(d) => { 
                    let mut v = d.unwrap_or(0.0);
                    if ui.add(DragValue::new(&mut v)).changed() {
                        (*d) = Some(v);
                    }
                },
                _ => unreachable!(),
            },
            ComposeNodeType::Position2D => match &mut output.data_type {
                ComposeDataType::Position2D(d) => { 
                    ui.horizontal(|ui| {
                        let mut v = d.unwrap_or(Vec2::ZERO);

                        if ui.add(DragValue::new(&mut v.y)).changed() {
                            (*d) = Some(v);
                        }
                        ui.label("y:");
                        if ui.add(DragValue::new(&mut v.x)).changed() {
                            (*d) = Some(v);
                        }
                        ui.label("x:");
                    });
                },
                _ => unreachable!(),
            },
            ComposeNodeType::Position3D => match &mut output.data_type {
                ComposeDataType::Position3D(d) => {
                    let mut v = d.unwrap_or(Vec3A::ZERO);

                    if ui.add(DragValue::new(&mut v.z)).changed() {
                        (*d) = Some(v);
                    }
                    ui.label("z:");
                    if ui.add(DragValue::new(&mut v.y)).changed() {
                        (*d) = Some(v);
                    }
                    ui.label("y:");
                    if ui.add(DragValue::new(&mut v.x)).changed() {
                        (*d) = Some(v);
                    }
                    ui.label("x:");
                },
                _ => unreachable!(),
            },
            _ => { ui.label(output.name.to_string()); }
        }

        output.data_type.get_pin()
    }

    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<ComposeNode>) {
        if to_input(snarl, to).data_type == to_output(snarl, from).data_type {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }

            snarl.connect(from.id, to.id);
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<ComposeNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<ComposeNode>) {
        ui.label("Add node");
        for node in self.node_templates.iter() {
            if ui.button(node.title()).clicked() {
                let new_node = snarl.insert_node(pos, node.to_owned());
                snarl.get_node_mut(new_node).unwrap().id = new_node;
                ui.close();
            }
        }    
    }

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<ComposeNode>) -> bool {
        true
    }

    fn show_dropped_wire_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        src_pins: AnyPins,
        snarl: &mut Snarl<ComposeNode>,
    ) {

        ui.label("Add node");

        match src_pins {
            AnyPins::Out(src_pins) => {
                for src_pin in src_pins {
                    let src_data = snarl[src_pin.node].outputs[src_pin.output].to_owned();

                    let mut src_label = src_pins.len() >= 2;
                    for node in self.node_templates.iter() {

                        let to_pins = node.inputs.iter()
                            .enumerate()
                            .filter(|(_, i)| i.data_type == src_data.data_type)
                            .collect_vec();

                        if to_pins.is_empty() {
                            continue;
                        }

                        if src_label {
                            ui.label(format!("> {}",  src_data.name));
                            src_label = false;
                        } 

                        if to_pins.len() >= 2 {
                            ui.label(format!("{:?}",  node.t));

                            for (i, to_data) in to_pins {
                                if ui.button(to_data.get_name()).clicked() {
                                    let new_node = snarl.insert_node(pos, node.to_owned());
                                    snarl.get_node_mut(new_node).unwrap().id = new_node;
                                    let dst_pin = InPinId {
                                        node: new_node,
                                        input: i,
                                    };

                                    snarl.connect(*src_pin, dst_pin);
                                }

                            } 
                        } else if !to_pins.is_empty() {
                            if ui.button(format!("{:?}",  node.t)).clicked() {
                                let new_node = snarl.insert_node(pos, node.to_owned());
                                snarl.get_node_mut(new_node).unwrap().id = new_node;
                                let dst_pin = InPinId {
                                    node: new_node,
                                    input: to_pins[0].0,
                                };

                                snarl.connect(*src_pin, dst_pin);
                            }

                        }

                    }
                }

            }
            AnyPins::In(src_pins) => {
                for src_pin in src_pins {
                    let src_data = snarl[src_pin.node].inputs[src_pin.input].to_owned();
                    

                    let mut src_label = src_pins.len() >= 2;
                    for node in self.node_templates.iter() {

                        let to_pins = node.outputs.iter()
                            .enumerate()
                            .filter(|(_, i)| i.data_type == src_data.data_type)
                            .collect_vec();

                        if to_pins.is_empty() {
                            continue;
                        }

                        if src_label {
                            ui.label(format!("> {}",  src_data.name));
                            src_label = false;
                        } 

                        if to_pins.len() >= 2 {
                            ui.label(format!("{:?}",  node.t));

                            for (i, to_data) in to_pins {
                                if ui.button(to_data.get_name()).clicked() {
                                    let new_node = snarl.insert_node(pos, node.to_owned());
                                    snarl.get_node_mut(new_node).unwrap().id = new_node;
                                    let dst_pin = OutPinId {
                                        node: new_node,
                                        output: i,
                                    };

                                    snarl.drop_inputs(*src_pin);
                                    snarl.connect(dst_pin, *src_pin);
                                }

                            } 
                        } else if !to_pins.is_empty() {
                            if ui.button(format!("{:?}",  node.t)).clicked() {
                                let new_node = snarl.insert_node(pos, node.to_owned());
                                snarl.get_node_mut(new_node).unwrap().id = new_node;
                                let dst_pin = OutPinId {
                                    node: new_node,
                                    output: to_pins[0].0,
                                };

                                snarl.drop_inputs(*src_pin);
                                snarl.connect(dst_pin, *src_pin);
                            }

                        }

                    }
                }

            }
        };
    }

    fn has_node_menu(&mut self, _node: &ComposeNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<ComposeNode>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }
}

pub fn to_input<'a>(snarl: &'a Snarl<ComposeNode>, in_pin: &'a InPin) -> &'a ComposeNodeInput {
    &snarl[in_pin.id.node].inputs[in_pin.id.input]
}

pub fn to_output<'a>(snarl: &'a Snarl<ComposeNode>, out_pin: &'a OutPin) -> &'a ComposeNodeOutput {
    &snarl[out_pin.id.node].outputs[out_pin.id.output]
}




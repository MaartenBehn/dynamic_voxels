use std::marker::PhantomData;

use egui_snarl::{ui::{AnyPins, PinInfo, SnarlViewer}, InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use itertools::Itertools;
use octa_force::{egui::{self, Color32, DragValue, Ui}, glam::{IVec2, IVec3, Vec2, Vec3A}};

use crate::util::{number::Nu, vector::Ve};

use super::{build::{ComposeTypeTrait, BS}, data_type::ComposeDataType, nodes::{get_node_templates, ComposeNode, ComposeNodeInput, ComposeNodeOutput, ComposeNodeType}};


#[derive(Debug)]
pub struct ComposeViewer<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub node_templates: Vec<ComposeNode<B::ComposeType>>,
    p0: PhantomData<V2>,
    p1: PhantomData<V3>,
    p2: PhantomData<T>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeViewer<V2, V3, T, B> {
    pub fn new() -> Self {
        let mut node_templates = get_node_templates();
        node_templates.append(&mut B::compose_nodes());
        Self { 
            node_templates,
            p0: Default::default(),
            p1: Default::default(),
            p2: Default::default(),
        }
    } 
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> SnarlViewer<ComposeNode<B::ComposeType>> for ComposeViewer<V2, V3, T, B> {
    fn title(&mut self, node: &ComposeNode<B::ComposeType>) -> String { 
        node.title() 
    }

    fn inputs(&mut self, node: &ComposeNode<B::ComposeType>) -> usize { 
        node.inputs.len() 
    }

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut octa_force::egui::Ui,
        snarl: &mut egui_snarl::Snarl<ComposeNode<B::ComposeType>>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let input = &mut snarl[pin.id.node].inputs[pin.id.input];
        
        ui.label(input.name.to_string());

        // Show input fields for number if nothing is connected.
        if pin.remotes.is_empty() {
            match &mut input.data_type {
                ComposeDataType::Number(d) => { 
                    let mut v = d.unwrap_or(0);
                    if ui.add(DragValue::new(&mut v)).changed() {
                        (*d) = Some(v);
                    }
                },
                ComposeDataType::Position2D(d) => { 
                    let mut v = d.unwrap_or(IVec2::ZERO);

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
                    let mut v = d.unwrap_or(IVec3::ZERO);

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

    fn outputs(&mut self, node: &ComposeNode<B::ComposeType>) -> usize {
        node.outputs.len()
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut octa_force::egui::Ui,
        snarl: &mut egui_snarl::Snarl<ComposeNode<B::ComposeType>>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &mut snarl[pin.id.node]; 
        let output = &mut node.outputs[pin.id.output];

        ui.add_space(8.0); 
        ui.label(output.name.to_string());

        output.data_type.get_pin()
    }

    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        if to_input(snarl, to).data_type == to_output(snarl, from).data_type {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }

            snarl.connect(from.id, to.id);
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<ComposeNode<B::ComposeType>>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        ui.label("Add node");
        for node in self.node_templates.iter() {
            if ui.button(node.title()).clicked() {
                let new_node = snarl.insert_node(pos, node.to_owned());
                snarl.get_node_mut(new_node).unwrap().id = new_node;
                ui.close();
            }
        }    
    }

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<ComposeNode<B::ComposeType>>) -> bool {
        true
    }

    fn show_dropped_wire_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        src_pins: AnyPins,
        snarl: &mut Snarl<ComposeNode<B::ComposeType>>,
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

    fn has_node_menu(&mut self, _node: &ComposeNode<B::ComposeType>) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<ComposeNode<B::ComposeType>>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }
}

pub fn to_input<'a, CT: ComposeTypeTrait>(
    snarl: &'a Snarl<ComposeNode<CT>>, 
    in_pin: &'a InPin
) -> &'a ComposeNodeInput {
    &snarl[in_pin.id.node].inputs[in_pin.id.input]
}

pub fn to_output<'a, CT: ComposeTypeTrait>(
    snarl: &'a Snarl<ComposeNode<CT>>, 
    out_pin: &'a OutPin
) -> &'a ComposeNodeOutput {
    &snarl[out_pin.id.node].outputs[out_pin.id.output]
}




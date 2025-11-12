use std::{marker::PhantomData, time::Duration};

use bitvec::vec::BitVec;
use egui_snarl::{ui::{AnyPins, NodeLayout, PinInfo, PinPlacement, SnarlStyle, SnarlViewer}, InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use itertools::Itertools;
use octa_force::{egui::{self, Color32, CornerRadius, DragValue, Pos2, Ui}, glam::{IVec2, IVec3, Vec2, Vec3A}};
use smallvec::SmallVec;

use crate::{model::data_types::data_type::ComposeDataType, util::{number::Nu, vector::Ve}};

use super::{build::{ComposeTypeTrait, BS}, nodes::{get_node_templates, ComposeNode, ComposeNodeGroupe, ComposeNodeInput, ComposeNodeOutput, ComposeNodeType}, pin::ComposePin};

const NOT_VALID_ANIMATION_TIME: f32 = 1.5;

#[derive(Debug)]
pub struct ComposeViewer<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub node_templates: Vec<ComposeNode<B::ComposeType>>,
    pub groupe_templates: Vec<Vec<ComposeNode<B::ComposeType>>>,
    pub deleted_nodes: SmallVec<[NodeId; 8]>,
    pub added_nodes: BitVec,
    pub changed_nodes: BitVec,
    pub invalid_nodes: BitVec,
    pub needs_collapse_nodes: BitVec,

    pub not_valid_scale: f32,
    pub offset: Vec2,
    pub hovered_menue_groupe: Option<ComposeNodeGroupe>,
    p0: PhantomData<V2>,
    p1: PhantomData<V3>,
    p2: PhantomData<T>,
}

pub const fn style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::coil()),
        pin_placement: Some(PinPlacement::Edge),
        pin_size: Some(10.0),
        collapsible: Some(false),
        node_frame: Some(egui::Frame {
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin {
                left: 0,
                right: 0,
                top: 0,
                bottom: 4,
            },
            corner_radius: CornerRadius::same(8),
            fill: egui::Color32::from_gray(30),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        bg_frame: Some(egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::same(2),
            corner_radius: CornerRadius::ZERO,
            fill: egui::Color32::from_gray(40),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        centering: Some(true),
        crisp_magnified_text: Some(true),
        ..SnarlStyle::new()
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeViewer<V2, V3, T, B> {
    pub fn new() -> Self {
        let mut node_templates = get_node_templates();
        node_templates.append(&mut B::compose_nodes());
        
        let mut groupe_templates: Vec<Vec<ComposeNode<B::ComposeType>>> = vec![];
        for node in node_templates.iter() {
            let group = groupe_templates.iter_mut()
                .find(|g|  g[0].group == node.group);

            let group = if group.is_none() {
                groupe_templates.push(vec![]);
                groupe_templates.last_mut().unwrap()
            } else {
                group.unwrap()
            };

            group.push(node.clone());
        }

        Self { 
            node_templates,
            groupe_templates,
            invalid_nodes: BitVec::new(),
            deleted_nodes: SmallVec::new(),
            added_nodes: BitVec::new(),
            changed_nodes: BitVec::new(),
            needs_collapse_nodes: BitVec::new(),

            not_valid_scale: 0.0,
            offset: Vec2::default(),
            hovered_menue_groupe: None,
            p0: Default::default(),
            p1: Default::default(),
            p2: Default::default(),
        }
    }

    pub fn update(&mut self, time: Duration) {
        let scale = 0.8 + simple_easing::roundtrip((time.as_secs_f32() % NOT_VALID_ANIMATION_TIME) / NOT_VALID_ANIMATION_TIME) * 0.3;
        self.not_valid_scale = simple_easing::sine_in(scale);
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> SnarlViewer<ComposeNode<B::ComposeType>> for ComposeViewer<V2, V3, T, B> {
    fn title(&mut self, node: &ComposeNode<B::ComposeType>) -> String { 
        node.title() 
    }

    fn show_header(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<ComposeNode<B::ComposeType>>,
    ) {
        ui.horizontal(|ui| {
            ui.label(self.title(&snarl[node]));

            let size = egui::Vec2::new(30.0, 30.0);
            if self.added_nodes.get(node.0).as_deref().copied().unwrap_or(false) {
                ui.add(
                    egui::Image::new(egui::include_image!("../../../assets/plus.svg"))
                        .tint(Color32::GRAY)
                        .fit_to_exact_size(size)
                );            
            }

            if self.changed_nodes.get(node.0).as_deref().copied().unwrap_or(false) {
                ui.add(
                    egui::Image::new(egui::include_image!("../../../assets/pencil.svg"))
                        .tint(Color32::GRAY)
                        .fit_to_exact_size(size)
                );            
            }

            if self.needs_collapse_nodes.get(node.0).as_deref().copied().unwrap_or(false) {
                ui.add(
                    egui::Image::new(egui::include_image!("../../../assets/spin.svg"))
                        .tint(Color32::GRAY)
                        .fit_to_exact_size(size)
                );            
            }
        });
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
        let input: &mut ComposeNodeInput = &mut snarl[pin.id.node].inputs[pin.id.input];
        
        ui.label(input.name.to_string());

        // Show input fields for number if nothing is connected.
        let changed = if pin.remotes.is_empty() {
            match &mut input.data_type {
                ComposeDataType::Number(d) => { 
                    let mut v = d.unwrap_or(0);
                    let res = ui.add(DragValue::new(&mut v));
                    if res.changed() {
                        (*d) = Some(v);
                    }

                    res.lost_focus() || res.dragged()
                },
                ComposeDataType::Position2D(d) => { 
                    let mut v = d.unwrap_or(IVec2::ZERO);

                    ui.label("x:");
                    let res_0 = ui.add(DragValue::new(&mut v.x));
                    if res_0.changed() {
                        (*d) = Some(v);
                    }

                    ui.label("y:");
                    let res_1 = ui.add(DragValue::new(&mut v.y));
                    if res_1.changed() {
                        (*d) = Some(v);
                    }

                    res_0.lost_focus() || res_0.dragged() || res_1.lost_focus() || res_1.dragged()
                },
                ComposeDataType::Position3D(d) => {
                    let mut v = d.unwrap_or(IVec3::ZERO);

                    ui.label("x:");
                    let res_0 = ui.add(DragValue::new(&mut v.x));
                    if res_0.changed() {
                        (*d) = Some(v);
                    }

                    ui.label("y:");
                    let res_1 = ui.add(DragValue::new(&mut v.y));
                    if res_1.changed() {
                        (*d) = Some(v);
                    }
                    
                    ui.label("z:");
                    let res_2 = ui.add(DragValue::new(&mut v.z));
                    if res_2.changed() {
                        (*d) = Some(v);
                    }

                    res_0.lost_focus() || res_0.dragged() || res_1.lost_focus() || res_1.dragged() || res_2.lost_focus() || res_2.dragged()
                },
                _ => false,
            }
        } else {
            false
        };

        let compose_pin = ComposePin::new(input.data_type, input.valid, self.not_valid_scale); 

        if changed {
            self.set_changed(pin.id.node, snarl);
        }

        compose_pin
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
        let output: &mut ComposeNodeOutput = &mut node.outputs[pin.id.output];

        ui.label(output.name.to_string());

        ComposePin::new(output.data_type, output.valid, self.not_valid_scale) 
    }

    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {
        if to_input(snarl, to).data_type == to_output(snarl, from).data_type {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }

            snarl.connect(from.id, to.id);

            self.set_changed(from.id.node, snarl);
            self.set_changed(to.id.node, snarl);

            self.update_node_valid(from.id.node, snarl);
            self.update_node_valid(to.id.node, snarl);
        }
    }

    fn has_graph_menu(&mut self, pos: egui::Pos2, _snarl: &mut Snarl<ComposeNode<B::ComposeType>>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<ComposeNode<B::ComposeType>>) {

        ui.label("Add node");
        for group in self.groupe_templates.iter() {
            let group_type = group[0].group;
           
            let res = egui::CollapsingHeader::new(format!("{:?}", &group_type))
                .open(Some(self.hovered_menue_groupe == Some(group_type)))
                .show(ui, |ui| {
                    for node in group {
                        if ui.button(node.title()).clicked() {
                            ui.close();

                            let new_node = snarl.insert_node(pos, node.to_owned());
                            snarl.get_node_mut(new_node).unwrap().id = new_node;
                            return Some(new_node);
                        }
                    }
                    return None;
                });

            if res.header_response.hovered() || (!res.body_response.is_none() && res.body_response.unwrap().hovered()) {
                self.hovered_menue_groupe = Some(group_type);
            }

            if let Some(Some(new_node)) = res.body_returned {
                self.set_added(new_node, snarl);
                self.update_node_valid(new_node, snarl);
                return;
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
                                    
                                    self.set_added(new_node, snarl);
                                    self.set_changed(src_pin.node, snarl);

                                    self.update_node_valid(new_node, snarl);
                                    self.update_node_valid(src_pin.node, snarl);
                                    return;
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
                                                                    
                                self.set_added(new_node, snarl);
                                self.set_changed(src_pin.node, snarl);

                                self.update_node_valid(new_node, snarl);
                                self.update_node_valid(src_pin.node, snarl);
                                return;
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
                                    
                                    self.set_added(new_node, snarl);
                                    self.set_changed(src_pin.node, snarl);

                                    self.update_node_valid(new_node, snarl);
                                    self.update_node_valid(src_pin.node, snarl);
                                    return;
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

                                self.set_added(new_node, snarl);
                                self.set_changed(src_pin.node, snarl);

                                self.update_node_valid(new_node, snarl);
                                self.update_node_valid(src_pin.node, snarl);
                                return;
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
            ui.close();
            snarl.remove_node(node);
            self.set_deleted(node);
            self.check_valid_for_all_nodes(snarl);
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




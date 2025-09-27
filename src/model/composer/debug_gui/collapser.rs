use octa_force::egui;
use slotmap::Key;

use crate::{model::composer::{build::BS, collapse::collapser::{CollapseNodeKey, Collapser}}, util::{number::Nu, vector::Ve}};


impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Collapser<V2, V3, T, B> { 
    pub fn debug_render(&self, ui: &mut egui::Ui) {
        ui.strong("Pending");
        ui.label(format!("Min Level: {}", self.pending.min_with_value));
        for (i, (collapse, create)) in self.pending.pending_per_level
            .iter()
            .enumerate() {

            if !create.is_empty()  {
                ui.label(format!("- Level {} Create", i));

                for pending in create {
                    ui.label(format!("{:?}", pending));
                }
            } 

            if !collapse.is_empty()  {
                ui.label(format!("- Level {} Collapse", i));

                for pending in collapse {
                    ui.label(format!("{:?}", pending));
                }
            } 
        }

        if !self.nodes.is_empty() {
            self.node(ui, self.get_root_key(), 0);
        } else {
            ui.label("Collapser empty");
        }
    }

    fn node(&self, ui: &mut egui::Ui, index: CollapseNodeKey, mut i: usize) -> usize {
        let node = &self.nodes[index];

        egui::CollapsingHeader::new(format!("Node: {:?}", node.template_index))
            .id_salt(format!("node: {i}"))
            .show(ui, |ui| {
            ui.label(format!("Level: {}", node.level));
            ui.label(format!("Template Index: {}", node.template_index));

            if !node.defined_by.is_null() {
                ui.strong("Defined By:");
                i = self.node(ui, node.defined_by, i+1);
            }
                        
            if !node.children.is_empty() {
                ui.strong("Children:");
                for (_, c) in node.children.iter() {
                    for key in c {
                        i = self.node(ui, *key, i+1);
                    }
                }
            }

            if !node.depends.is_empty() {
                ui.strong("Depends:");
                for (_, c) in node.depends.iter() {
                    for key in c {
                        i = self.node(ui, *key, i+1);
                    }
                }
            }
 
            ui.label(format!("Data: {:#?}", node.data));

            i
        }).body_returned.unwrap_or(i)  
    }
} 

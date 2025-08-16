use octa_force::egui;
use slotmap::Key;

use crate::model::{generation::{collapse::{CollapseNodeKey, Collapser}, traits::ModelGenerationTypes}, worker::CollapserChangeReciver};


#[derive(Debug)]
pub struct CollapserDebugGui<T: ModelGenerationTypes> {
    r: CollapserChangeReciver<T>
}

impl<T: ModelGenerationTypes> CollapserDebugGui<T> {
    pub fn new(r: CollapserChangeReciver<T>) -> Self {
        Self {
            r
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        let collapser = self.r.get_collapser();

        egui::Window::new("Collapser")
            .vscroll(true)
            .show(ctx, |ui| {

                ui.strong("Pending User Opperation");
                for o in collapser.pending_user_opperations.iter() {
                    ui.label(format!("- {:?}", o));
                }
                
                ui.strong("Pending");
                ui.label(format!("Min Level: {}", collapser.pending.min_with_value));
                for (i, (collapse, create)) in collapser.pending.pending_per_level
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

                if !collapser.nodes.is_empty() {
                    Self::node(collapser, ui, collapser.nodes.keys().next().unwrap(), 0);
                } else {
                    ui.label("Collapser empty");
                }
            });
    }

    fn node(collapser: &Collapser<T>, ui: &mut egui::Ui, index: CollapseNodeKey, mut i: usize) -> usize {
        let node = &collapser.nodes[index];

        egui::CollapsingHeader::new(format!("Node: {:?}", node.identifier))
            .id_salt(format!("node: {i}"))
            .show(ui, |ui| {
            ui.label(format!("Level: {}", node.level));
            ui.label(format!("Template Index: {}", node.template_index));

            if !node.defined_by.is_null() {
                ui.strong("Defined By:");
                i = Self::node(collapser, ui, node.defined_by, i+1);
            }
                        
            if !node.children.is_empty() {
                ui.strong("Children:");
                for (_, c) in node.children.iter() {
                    for key in c {
                        i = Self::node(collapser, ui, *key, i+1);
                    }
                }
            }

            if !node.depends.is_empty() {
                ui.strong("Depends:");
                for (_, c) in node.depends.iter() {
                    for key in c {
                        i = Self::node(collapser, ui, *key, i+1);
                    }
                }
            }

            if !node.restricts.is_empty() {
                ui.strong("Restricts:");
                for (_, c) in node.restricts.iter() {
                    for key in c {
                        i = Self::node(collapser, ui, *key, i+1);
                    }
                }
            }

            if !node.knows.is_empty() {
                ui.strong("Knows:");
                for (_, c) in node.knows.iter() {
                    for key in c {
                        i = Self::node(collapser, ui, *key, i+1);
                    }
                }
            }

            if !node.next_reset.is_null() {
                ui.strong("Next Reset");
                i = Self::node(collapser, ui, node.next_reset, i+1);
            }

            ui.label(format!("Data: {:?}", node.data));
            ui.label(format!("Undo Data: {:?}", node.undo_data));

            i
        }).body_returned.unwrap_or(i)  
    }
} 

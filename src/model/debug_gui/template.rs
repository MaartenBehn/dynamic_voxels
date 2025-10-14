use itertools::Itertools;
use octa_force::egui;

use crate::{model::composer::{build::BS, dependency_tree::{DependencyPath, DependencyTree}, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex, TemplateNode}}, util::{number::Nu, vector::Ve}};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> { 
    pub fn debug_render(&self, ui: &mut egui::Ui) {
        if self.nodes.is_empty() {
            ui.label("Template empty");
            return;
        }

        self.node(ui, 0, &mut 0);
    }

    fn node(&self, ui: &mut egui::Ui, index: TemplateIndex, node_counter: &mut usize) {
        let node = &self.nodes[index];
        (*node_counter) += 1;

        egui::CollapsingHeader::new(format!("Node: {:?}", index))
            .id_salt(format!("template node: {node_counter}"))
            .show(ui, |ui| {

            ui.label(format!("Value: {}", match &node.value {
                ComposeTemplateValue::NumberSpace(space) => format!("Number Space: {:#?}",space),
                ComposeTemplateValue::PositionSpace2D(space) => format!("Position Space 2D: {:#?}",space),
                ComposeTemplateValue::PositionSpace3D(space) => format!("Position Space 3D: {:#?}",space),
                ComposeTemplateValue::None => "None".to_string(),
                ComposeTemplateValue::Build(t) => format!("Build: {:#?}", t),
            }));

            ui.label(format!("Level: {}", node.level));

            if !node.creates.is_empty() {
                ui.strong("Creates:");
                for creates in node.creates.iter() {
                    self.node(ui, creates.to_create, node_counter);

                    ui.label(format!("Type: {:#?}", creates.t));

                    if !creates.others.is_empty() {
                        ui.strong("Other Ammounts:");
                        for other in creates.others.iter() {
                            self.node(ui, *other, node_counter);
                        }
                    }
                }
            }

            if !node.depends.is_empty() {
                ui.strong("Depends:");
                for i in node.depends.iter() {
                    self.node(ui, *i, node_counter);
                }
            }
 
            if !node.dependecy_tree.steps.is_empty() {
                ui.strong("Dependency Tree:");
                self.dependecy_tree(&node.dependecy_tree, node, ui, 0);
            } 

            if !node.depends_loop.is_empty() {
                ui.strong("Depends Loop:");
                for (i, path) in node.depends_loop.iter() {
                    self.node(ui, *i, node_counter);
                    self.dependecy_path(path, ui, node_counter);
                }
            }

            if !node.dependend.is_empty() {
                ui.strong("dependend is:");
                for i in node.dependend.iter() {
                    self.node(ui, *i, node_counter);
                }
            }
        });
    }

    fn dependecy_tree(
        &self, 
        tree: &DependencyTree, 
        inital_node: &TemplateNode<V2, V3, T, B>, 
        ui: &mut egui::Ui, 
        index: usize
    ) {
        let step = &tree.steps[index];
        let node = &self.nodes[step.into_index];

        let up_text = if step.up { "up" } else { "down" };
        let leaf_text = if step.leaf.is_some() {
            "leaf".to_string()
        } else {
            "".to_string()
        };

        ui.collapsing(format!("{} {:?} {}", up_text, node.node_id, leaf_text), |ui| {
            for i in step.children.iter() {
                self.dependecy_tree(tree, inital_node, ui, *i);
            }
        });
    }

    fn dependecy_path(
        &self, 
        path: &DependencyPath,
        ui: &mut egui::Ui,
        node_counter: &mut usize,
    ) {
        ui.collapsing("Path", |ui| {
            for step in path.steps.iter() {
                let up_text = if step.up { "up" } else { "down" };
                ui.label(format!("Step: {}", up_text));

                self.node(ui, step.into_index, node_counter);
            }
        });
    }
}

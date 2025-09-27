use itertools::Itertools;
use octa_force::egui;

use crate::{model::composer::{build::BS, dependency_tree::DependencyTree, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex, TemplateNode}}, util::{number::Nu, vector::Ve}};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeTemplate<V2, V3, T, B> { 
    pub fn debug_render(&self, ui: &mut egui::Ui) {
        if self.nodes.is_empty() {
            ui.label("Template empty");
            return;
        }

        self.node(ui, 0, &DependencyTree::default());
    }

    fn node(&self, ui: &mut egui::Ui, index: TemplateIndex, dependency_tree: &DependencyTree) {
        let node = &self.nodes[index];

        ui.collapsing(format!("Node: {:?}", node.node_id), |ui| {
            ui.label(format!("Value: {}", match &node.value {
                ComposeTemplateValue::NumberSpace(space) => format!("Number Space: {:#?}",space),
                ComposeTemplateValue::PositionSpace2D(space) => format!("Position Space 2D: {:#?}",space),
                ComposeTemplateValue::PositionSpace3D(space) => format!("Position Space 3D: {:#?}",space),
                ComposeTemplateValue::None => "None".to_string(),
                ComposeTemplateValue::Build(t) => "Build".to_string(),
            }));

            ui.label(format!("Level: {}", node.level));

            if !node.defines.is_empty() {
                ui.strong("Defines:");
                for ammount in node.defines.iter() {
                    self.node(ui, ammount.template_index, &ammount.dependecy_tree);
                    ui.label(format!("Ammount: {:?}", ammount.t));
                }
            } 

            if !node.depends.is_empty() {
                ui.strong("Depends:");
                for i in node.depends.iter() {
                    let child = &self.nodes[*i];
                    ui.label(format!("{:?}", child.node_id));
                }
            }
 
            if !dependency_tree.steps.is_empty() {
                ui.strong("Dependency Tree:");
                self.relative_path(dependency_tree, node, ui, 0);
            }  

            if !node.dependend.is_empty() {
                ui.strong("dependend is:");
                for i in node.dependend.iter() {
                    let child = &self.nodes[*i];
                    ui.label(format!("{:?}", child.node_id));
                }
            }
        });
    }

    fn relative_path(
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
                self.relative_path(tree, inital_node, ui, *i);
            }
        });
    }
}

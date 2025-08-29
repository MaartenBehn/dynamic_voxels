use itertools::Itertools;
use octa_force::egui;

use crate::model::composer::{dependency_tree::{DependencyTree}, template::{ComposeTemplate, ComposeTemplateValue, TemplateIndex, TemplateNode}};

#[derive(Debug)]
pub struct TemplateDebugGui {
    pub template: ComposeTemplate,
}

impl TemplateDebugGui {
    pub fn new() -> Self {
        Self {
            template: Default::default(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if self.template.nodes.is_empty() {
            ui.label("Template empty");
            return;
        }

        Self::node(&self.template, ui, 0, &DependencyTree::default());
    }

    fn node(template: &ComposeTemplate, ui: &mut egui::Ui, index: TemplateIndex, dependency_tree: &DependencyTree) {
        let node = &template.nodes[index];

        ui.collapsing(format!("Node: {:?}", node.node_id), |ui| {
            ui.label(format!("Value: {}", match &node.value {
                ComposeTemplateValue::NumberSpace(space) => format!("Number Space: {:#?}",space),
                ComposeTemplateValue::PositionSpace(space) => format!("Position Space: {:#?}",space),
                ComposeTemplateValue::None => "None".to_string(),
                ComposeTemplateValue::Object() => "Object".to_string(),
            }));

            ui.label(format!("Level: {}", node.level));

            if !node.defines.is_empty() {
                ui.strong("Defines:");
                for ammount in node.defines.iter() {
                    Self::node(template, ui, ammount.template_index, &ammount.dependecy_tree);
                    ui.label(format!("Ammount: {:?}", ammount.n));
                }
            } 

            if !node.depends.is_empty() {
                ui.strong("Depends:");
                for i in node.depends.iter() {
                    let child = &template.nodes[*i];
                    ui.label(format!("{:?}", child.node_id));
                }
            }
 
            if !dependency_tree.steps.is_empty() {
                ui.strong("Dependency Tree:");
                Self::relative_path(template, dependency_tree, node, ui, 0);
            }  

            if !node.dependend.is_empty() {
                ui.strong("dependend is:");
                for i in node.dependend.iter() {
                    let child = &template.nodes[*i];
                    ui.label(format!("{:?}", child.node_id));
                }
            }
        });
    }

    fn relative_path(
        template: &ComposeTemplate, 
        tree: &DependencyTree, 
        inital_node: &TemplateNode, 
        ui: &mut egui::Ui, 
        index: usize
    ) {
        let step = &tree.steps[index];
        let node = &template.nodes[step.into_index];

        let up_text = if step.up { "up" } else { "down" };
        let leaf_text = if step.leaf.is_some() {
            "leaf".to_string()
        } else {
            "".to_string()
        };

        ui.collapsing(format!("{} {:?} {}", up_text, node.node_id, leaf_text), |ui| {
            for i in step.children.iter() {
                Self::relative_path(template, tree, inital_node, ui, *i);
            }
        });
    }
}

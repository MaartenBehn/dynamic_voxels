use itertools::Itertools;
use octa_force::egui;
use crate::model::{generation::{relative_path::{LeafType, RelativePathTree}, template::{NodeTemplateValue, TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes}, worker::TemplateChangeReciver};

#[derive(Debug)]
pub struct TemplateDebugGui<T: ModelGenerationTypes> {
    r: TemplateChangeReciver<T>
}

impl<T: ModelGenerationTypes> TemplateDebugGui<T> {
    pub fn new(r: TemplateChangeReciver<T>) -> Self {
        Self {
            r
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        let template = self.r.get_template();

        egui::Window::new("Template Tree")
            .vscroll(true)
            .show(ctx, |ui| {
                if template.nodes.is_empty() {
                    ui.label("Template empty");
                    return;
                }

                Self::node(template, ui, 0, &RelativePathTree::default());
            });
    }

    fn node(template: &TemplateTree<T>, ui: &mut egui::Ui, index: TemplateIndex, dependency_tree: &RelativePathTree) {
        let node = &template.nodes[index];

        ui.collapsing(format!("Node: {:?}", node.identifier), |ui| {
            ui.label(format!("Value: {}", match &node.value {
                NodeTemplateValue::Groupe => "Groupe".to_string(),
                NodeTemplateValue::NumberRangeHook => "Number Range Hook".to_string(),
                NodeTemplateValue::NumberRange(number_range) => format!("Number Range: {:?}", number_range.values),
                NodeTemplateValue::PosSetHook => "Pos Set Hook".to_string(),
                NodeTemplateValue::PosSet(position_set) => format!("Pos Set"),
                NodeTemplateValue::BuildHook => "Build".to_string(),
            }));

            ui.label(format!("Level: {}", node.level));

            if !node.defines_n.is_empty() || !node.defines_by_value.is_empty() {
                ui.strong("Defines:");
                for ammount in node.defines_n.iter() {
                    Self::node(template, ui, ammount.index, &ammount.dependecy_tree);
                    ui.label(format!("Ammount: {}", ammount.ammount));
                }

                for by_value in node.defines_by_value.iter() {
                    Self::node(template, ui, by_value.index, &by_value.dependecy_tree);
                    ui.label("Ammount: by value");
                }
            }

            if !node.restricts.is_empty() {
                ui.strong("Restricts:");
                for i in node.restricts.iter() {
                    let child = &template.nodes[*i];
                    ui.label(format!("{:?}", child.identifier));
                }
            }

            if !node.depends.is_empty() {
                ui.strong("Depends:");
                for i in node.depends.iter() {
                    let child = &template.nodes[*i];
                    ui.label(format!("{:?}", child.identifier));
                }
            }

            if !node.knows.is_empty() {
                ui.strong("Knows:");
                for i in node.knows.iter() {
                    let child = &template.nodes[*i];
                    ui.label(format!("{:?}", child.identifier));
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
                    ui.label(format!("{:?}", child.identifier));
                }
            }
        });
    }

    fn relative_path(
        template: &TemplateTree<T>, 
        tree: &RelativePathTree, 
        inital_node: &TemplateNode<T>, 
        ui: &mut egui::Ui, 
        index: usize
    ) {
        let step = &tree.steps[index];
        let node = &template.nodes[step.into_index];

        let up_text = if step.up { "up" } else { "down" };
        let leaf_text = step.leafs.iter()
            .map(|leaf| match leaf {
                LeafType::Restricts(i) => format!("restricts"),
                LeafType::Depends(i) => format!("depends"),
                LeafType::Knows(i) => format!("knows"),
            })
            .join(", ");

        ui.collapsing(format!("{} {:?} {}", up_text, node.identifier, leaf_text), |ui| {
            for i in step.children.iter() {
                Self::relative_path(template, tree, inital_node, ui, *i);
            }
        });
    }
}

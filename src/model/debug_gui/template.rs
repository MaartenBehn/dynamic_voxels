use itertools::Itertools;
use octa_force::egui;

use crate::{model::{composer::build::{BS, TemplateValueTrait}, data_types::{number::NumberTemplate, number_space::NumberSpaceTemplate, position::PositionTemplate, position_pair_set::PositionPairSetTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate}, template::{Template, TemplateIndex, dependency_tree::{DependencyPath, DependencyTree}, nodes::TemplateNode, value::{TemplateValue, ValueIndex}}}, util::{number::Nu, vector::Ve}};

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Template<V2, V3, T, B> { 
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

            self.value(ui, node.value_index);

            ui.label(format!("Level: {}", node.level));

            if !node.creates.is_empty() {
                ui.strong("Creates:");
                for creates in node.creates.iter() {
                    self.node(ui, creates.to_create, node_counter);

                    ui.label(format!("Type: {:#?}", creates.t));
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

    fn value(&self, ui: &mut egui::Ui, value_index: ValueIndex) {
        let value = &self.values[value_index];

        ui.collapsing(format!("{:?}", value), |ui| {
            
            match value {
                TemplateValue::None => unreachable!(),
                TemplateValue::Number(number_template) => {
                    match number_template {
                        NumberTemplate::Const(t) => {},
                        NumberTemplate::Hook(hook) => {},
                        NumberTemplate::SplitPosition2D((i, _)) => self.value(ui, *i),
                        NumberTemplate::SplitPosition3D((i, _)) => self.value(ui, *i),
                        NumberTemplate::Position3DTo2D(i) => self.value(ui, *i),
                    }
                },
                TemplateValue::NumberSet(number_space_template) => {
                    match number_space_template {
                        NumberSpaceTemplate::NumberRange { min, max, step } => {
                            self.value(ui, *min);
                            self.value(ui, *max);
                            self.value(ui, *step);
                        },
                    }
                },
                TemplateValue::Position2D(position_template) => {
                    match position_template {
                        PositionTemplate::Const(_) => {},
                        PositionTemplate::Add((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionTemplate::Sub((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionTemplate::FromNumbers([x, y]) => {
                            self.value(ui, *x);
                            self.value(ui, *y);
                        },
                        PositionTemplate::Position2DTo3D((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionTemplate::Position3DTo2D(i) =>self.value(ui, *i),
                        PositionTemplate::PerPosition(hook) => {},
                        PositionTemplate::PerPair(_) => {},
                        PositionTemplate::Cam => {},
                        PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                    }
                },
                TemplateValue::Position3D(position_template) => {
                    match position_template {
                        PositionTemplate::Const(_) => {},
                        PositionTemplate::Add((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionTemplate::Sub((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionTemplate::FromNumbers([x, y, z]) => {
                            self.value(ui, *x);
                            self.value(ui, *y);
                            self.value(ui, *z);
                        },
                        PositionTemplate::Position2DTo3D((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionTemplate::Position3DTo2D(i) =>self.value(ui, *i),
                        PositionTemplate::PerPosition(hook) => {},
                        PositionTemplate::PerPair(_) => {},
                        PositionTemplate::Cam => {},
                        PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                    }

                },
                TemplateValue::PositionSet2D(position_set_template) => {
                    match position_set_template {
                        PositionSetTemplate::All(i) => self.value(ui, *i),
                    }
                },
                TemplateValue::PositionSet3D(position_set_template) => {
                    match position_set_template {
                        PositionSetTemplate::All(i) => self.value(ui, *i),
                    }
                },
                TemplateValue::PositionPairSet2D(position_pair_set_template) => {
                    match position_pair_set_template {
                        PositionPairSetTemplate::ByDistance((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                    }
                },
                TemplateValue::PositionPairSet3D(position_pair_set_template) => {
                    match position_pair_set_template {
                        PositionPairSetTemplate::ByDistance((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                    }
                },
                TemplateValue::PositionSpace2D(position_space_template) => {
                    match position_space_template {
                        PositionSpaceTemplate::Grid(grid_template) => {
                            self.value(ui, grid_template.volume);
                            self.value(ui, grid_template.spacing);
                        },
                        PositionSpaceTemplate::LeafSpread(leaf_spread_template) => {
                            self.value(ui, leaf_spread_template.volume);
                            self.value(ui, leaf_spread_template.samples); 
                        },
                        PositionSpaceTemplate::Path(path_template) => {
                            self.value(ui, path_template.start);
                            self.value(ui, path_template.end);
                            self.value(ui, path_template.spacing);
                            self.value(ui, path_template.side_variance);
                        },
                    }
                },
                TemplateValue::PositionSpace3D(position_space_template) => {
                    match position_space_template {
                        PositionSpaceTemplate::Grid(grid_template) => {
                            self.value(ui, grid_template.volume);
                            self.value(ui, grid_template.spacing);
                        },
                        PositionSpaceTemplate::LeafSpread(leaf_spread_template) => {
                            self.value(ui, leaf_spread_template.volume);
                            self.value(ui, leaf_spread_template.samples); 
                        },
                        PositionSpaceTemplate::Path(path_template) => {
                            self.value(ui, path_template.start);
                            self.value(ui, path_template.end);
                            self.value(ui, path_template.spacing);
                            self.value(ui, path_template.side_variance);
                        },
                    }
                },
                TemplateValue::Volume2D(volume_template) => {
                    match volume_template {
                        VolumeTemplate::Sphere { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeTemplate::Disk { pos, size, height } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                            self.value(ui, *height);
                        },
                        VolumeTemplate::Box { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeTemplate::Union { a, b } => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        VolumeTemplate::Cut { base, cut } => {
                            self.value(ui, *base);
                            self.value(ui, *cut);
                        },
                        VolumeTemplate::Material { mat, child } => {
                            self.value(ui, *child);
                        },
                    }
                },
                TemplateValue::Volume3D(volume_template) => {
                    match volume_template {
                        VolumeTemplate::Sphere { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeTemplate::Disk { pos, size, height } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                            self.value(ui, *height);
                        },
                        VolumeTemplate::Box { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeTemplate::Union { a, b } => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        VolumeTemplate::Cut { base, cut } => {
                            self.value(ui, *base);
                            self.value(ui, *cut);
                        },
                        VolumeTemplate::Material { mat, child } => {
                            self.value(ui, *child);
                        },
                    }
                },
                TemplateValue::Build(v) => {
                    for v in v.to_owned().value_indecies() {
                        self.value(ui, v);
                    }
                },
            }
        });
    }

    fn dependecy_tree(
        &self, 
        tree: &DependencyTree, 
        inital_node: &TemplateNode, 
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

        ui.collapsing(format!("{} {:?} {}", up_text, node.index, leaf_text), |ui| {
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

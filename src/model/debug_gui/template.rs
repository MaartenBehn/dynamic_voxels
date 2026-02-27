use itertools::Itertools;
use octa_force::egui;

use crate::{model::{ data_types::{number::NumberValue, number_space::NumberSpaceValue, position::PositionValue, position_pair_set::PositionPairSetValue, position_set::PositionSetValue, position_space::PositionSpaceValue, volume::VolumeValue}, template::{Template, TemplateIndex, dependency_tree::{DependencyPath, DependencyTree}, nodes::TemplateNode, value::{TemplateValue, ValueIndex}}}, util::{number::Nu, vector::Ve}};

impl Template { 
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
                        NumberValue::Const(t) => {},
                        NumberValue::Hook(hook) => {},
                        NumberValue::SplitPosition2D((i, _)) => self.value(ui, *i),
                        NumberValue::SplitPosition3D((i, _)) => self.value(ui, *i),
                        NumberValue::Position3DTo2D(i) => self.value(ui, *i),
                    }
                },
                TemplateValue::NumberSet(number_space_template) => {
                    match number_space_template {
                        NumberSpaceValue::NumberRange { min, max, step } => {
                            self.value(ui, *min);
                            self.value(ui, *max);
                            self.value(ui, *step);
                        },
                    }
                },
                TemplateValue::Position2D(position_template) => {
                    match position_template {
                        PositionValue::Const(_) => {},
                        PositionValue::Add((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionValue::Sub((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionValue::FromNumbers([x, y]) => {
                            self.value(ui, *x);
                            self.value(ui, *y);
                        },
                        PositionValue::Position2DTo3D((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionValue::Position3DTo2D(i) =>self.value(ui, *i),
                        PositionValue::PerPosition(hook) => {},
                        PositionValue::PerPair(_) => {},
                        PositionValue::Cam => {},
                        PositionValue::PhantomData(phantom_data) => unreachable!(),
                    }
                },
                TemplateValue::Position3D(position_template) => {
                    match position_template {
                        PositionValue::Const(_) => {},
                        PositionValue::Add((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionValue::Sub((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionValue::FromNumbers([x, y, z]) => {
                            self.value(ui, *x);
                            self.value(ui, *y);
                            self.value(ui, *z);
                        },
                        PositionValue::Position2DTo3D((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        PositionValue::Position3DTo2D(i) =>self.value(ui, *i),
                        PositionValue::PerPosition(hook) => {},
                        PositionValue::PerPair(_) => {},
                        PositionValue::Cam => {},
                        PositionValue::PhantomData(phantom_data) => unreachable!(),
                    }

                },
                TemplateValue::PositionSet2D(position_set_template) => {
                    match position_set_template {
                        PositionSetValue::All(i) => self.value(ui, *i),
                    }
                },
                TemplateValue::PositionSet3D(position_set_template) => {
                    match position_set_template {
                        PositionSetValue::All(i) => self.value(ui, *i),
                    }
                },
                TemplateValue::PositionPairSet2D(position_pair_set_template) => {
                    match position_pair_set_template {
                        PositionPairSetValue::ByDistance((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                    }
                },
                TemplateValue::PositionPairSet3D(position_pair_set_template) => {
                    match position_pair_set_template {
                        PositionPairSetValue::ByDistance((a, b)) => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                    }
                },
                TemplateValue::PositionSpace2D(position_space_template) => {
                    match position_space_template {
                        PositionSpaceValue::Grid(grid_template) => {
                            self.value(ui, grid_template.volume);
                            self.value(ui, grid_template.spacing);
                        },
                        PositionSpaceValue::LeafSpread(leaf_spread_template) => {
                            self.value(ui, leaf_spread_template.volume);
                            self.value(ui, leaf_spread_template.samples); 
                        },
                        PositionSpaceValue::Path(path_template) => {
                            self.value(ui, path_template.start);
                            self.value(ui, path_template.end);
                            self.value(ui, path_template.spacing);
                            self.value(ui, path_template.side_variance);
                        },
                    }
                },
                TemplateValue::PositionSpace3D(position_space_template) => {
                    match position_space_template {
                        PositionSpaceValue::Grid(grid_template) => {
                            self.value(ui, grid_template.volume);
                            self.value(ui, grid_template.spacing);
                        },
                        PositionSpaceValue::LeafSpread(leaf_spread_template) => {
                            self.value(ui, leaf_spread_template.volume);
                            self.value(ui, leaf_spread_template.samples); 
                        },
                        PositionSpaceValue::Path(path_template) => {
                            self.value(ui, path_template.start);
                            self.value(ui, path_template.end);
                            self.value(ui, path_template.spacing);
                            self.value(ui, path_template.side_variance);
                        },
                    }
                },
                TemplateValue::Volume2D(volume_template) => {
                    match volume_template {
                        VolumeValue::Sphere { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeValue::Disk { pos, size, height } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                            self.value(ui, *height);
                        },
                        VolumeValue::Box { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeValue::Union { a, b } => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        VolumeValue::Cut { base, cut } => {
                            self.value(ui, *base);
                            self.value(ui, *cut);
                        },
                        VolumeValue::Material { mat, child } => {
                            self.value(ui, *child);
                        },
                    }
                },
                TemplateValue::Volume3D(volume_template) => {
                    match volume_template {
                        VolumeValue::Sphere { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeValue::Disk { pos, size, height } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                            self.value(ui, *height);
                        },
                        VolumeValue::Box { pos, size } => {
                            self.value(ui, *pos);
                            self.value(ui, *size);
                        },
                        VolumeValue::Union { a, b } => {
                            self.value(ui, *a);
                            self.value(ui, *b);
                        },
                        VolumeValue::Cut { base, cut } => {
                            self.value(ui, *base);
                            self.value(ui, *cut);
                        },
                        VolumeValue::Material { mat, child } => {
                            self.value(ui, *child);
                        },
                    }
                },
                TemplateValue::Voxels(voxel_template) => {
                    self.value(ui, voxel_template.pos);
                    self.value(ui, voxel_template.volume);
                },
                TemplateValue::Mesh(mesh_template) => {
                    self.value(ui, mesh_template.pos);
                    self.value(ui, mesh_template.volume);
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

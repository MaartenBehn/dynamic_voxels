use std::time::Instant;

use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Edge, Graph, GraphView, Node};
use fdg::{fruchterman_reingold::{FruchtermanReingold, FruchtermanReingoldConfiguration}, nalgebra::{Const, OPoint}, Force, ForceGraph};
use octa_force::{controls::Controls, egui::{self, vec2, CollapsingHeader, Context, Pos2, ScrollArea, Ui, Vec2}};
use petgraph::{csr::DefaultIx, graph::{EdgeIndex, NodeIndex}, visit::GraphRef, Directed};

use super::{drawers::{draw_counts_sliders, draw_section_debug, draw_simulation_config_sliders, draw_start_reset_buttons, ValuesConfigButtonsStartReset, ValuesConfigSlidersGraph, ValuesConfigSlidersSimulation, ValuesSectionDebug}, node_shape::NodeShape, settings::{ForceSettings, SettingsNavigation, SimulationSettings}};

const EVENTS_LIMIT: usize = 100;
const SHOW_COOLDOWN: f32 = 0.1;

pub struct WFCRenderer {
    pub g: Graph<(), (), Directed, DefaultIx, NodeShape, DefaultEdgeShape>,
    sim: ForceGraph<f32, 2, Node<(), (), Directed, DefaultIx, NodeShape>, Edge<(), (), Directed, DefaultIx, NodeShape, DefaultEdgeShape>>,
    force: FruchtermanReingold<f32, 2>,

    force_settings: ForceSettings,
    simulation_settings: SimulationSettings,

    settings_navigation: SettingsNavigation,

    last_events: Vec<String>,

    pan: [f32; 2],
    zoom: f32,

    show: bool,
    last_button_click: Instant,
}

impl WFCRenderer {
    pub fn new() -> Self {
        let force_settings = ForceSettings::default();
        let simulation_settings = SimulationSettings::default();

        let g = petgraph::stable_graph::StableGraph::new();
        
        let mut g = Graph::<_, _, _, _, NodeShape, _>::from(&g);

        let mut force = init_force(&force_settings);
        let mut sim = fdg::init_force_graph_uniform(g.g.clone(), 1.0);
        force.apply(&mut sim);
        g.g.node_weights_mut().for_each(|node| {
            let point: fdg::nalgebra::OPoint<f32, fdg::nalgebra::Const<2>> =
                sim.node_weight(node.id()).unwrap().1;
            node.set_location(Pos2::new(point.coords.x, point.coords.y));
        });

        Self {
            g,
            sim,
            force,

            force_settings,
            simulation_settings,

            settings_navigation: SettingsNavigation::default(),

            last_events: Vec::default(),

            pan: [0., 0.],
            zoom: 0.,

            show: false,
            last_button_click: Instant::now(),
        }
    }

    pub fn update(&mut self, controls: &Controls) {
        if controls.f2 && self.last_button_click.elapsed().as_secs_f32() > SHOW_COOLDOWN {
            self.show = !self.show;
            self.last_button_click = Instant::now();
        } 

        if !self.simulation_settings.running {
            return;
        }

        self.sim.node_weights_mut().for_each(|node| {
            let g_node = self.g.g.node_weight_mut(node.0.id()).unwrap();
            if g_node.dragged() {
                node.1.x = g_node.location().x;
                node.1.y = g_node.location().y;
            } else {
                let pos = vec2(node.1.x, node.1.y);
                let force = pos * self.simulation_settings.center_force;
            
                node.1.x = pos.x - force.x;
                node.1.y = pos.y - force.y;
            }

        });

        self.force.apply(&mut self.sim);

        self.g.g.node_weights_mut().for_each(|node| {
        let sim_computed_point: OPoint<f32, Const<2>> =
                self.sim.node_weight(node.id()).unwrap().1;
            node.set_location(Pos2::new(
                sim_computed_point.coords.x,
                sim_computed_point.coords.y,
            ));
        });

    }

    fn draw_section_simulation(&mut self, ui: &mut Ui) { 
        draw_start_reset_buttons(
            ui,
            ValuesConfigButtonsStartReset {
                simulation_stopped: !self.simulation_settings.running,
            },
            |simulation_stopped: bool, reset_pressed: bool| {
                self.simulation_settings.running = !simulation_stopped;
                if reset_pressed {
                    self.reset()
                };
            },
        );

        ui.add_space(10.);

        draw_simulation_config_sliders(
            ui,
            ValuesConfigSlidersSimulation {
                dt: self.force_settings.dt,
                cooloff_factor: self.force_settings.cooloff_factor,
                scale: self.force_settings.scale,
            },
            |delta_dt: f32, delta_cooloff_factor: f32, delta_scale: f32| {
                self.force_settings.dt += delta_dt;
                self.force_settings.cooloff_factor += delta_cooloff_factor;
                self.force_settings.scale += delta_scale;

                self.force = init_force(&self.force_settings);
            },
        );

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.simulation_settings.center_force, 0.0..=1.0).text("center force"));
        });

    }

    fn draw_section_widget(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Navigation")
            .default_open(true)
            .show(ui, |ui| {
                if ui
                    .checkbox(
                        &mut self.settings_navigation.fit_to_screen_enabled,
                        "fit to screen",
                    )
                    .changed()
                    && self.settings_navigation.fit_to_screen_enabled
                {
                    self.settings_navigation.zoom_and_pan_enabled = false
                };

                ui.label("Enable fit to screen to fit the graph to the screen on every frame.");

                ui.add_space(5.);

                if ui.checkbox(&mut self.settings_navigation.zoom_and_pan_enabled,"zoom and pan",)
                    .changed() && self.settings_navigation.zoom_and_pan_enabled {
                    self.settings_navigation.fit_to_screen_enabled = false
                };
            });

        CollapsingHeader::new("Selected")
            .default_open(true)
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(200.)
                    .show(ui, |ui| {
                        self.g.selected_nodes().iter().for_each(|node| {
                            ui.label(format!("{node:?}"));
                        });
                        self.g.selected_edges().iter().for_each(|edge| {
                            ui.label(format!("{edge:?}"));
                        });
                    });
            });

        CollapsingHeader::new("Last Events")
            .default_open(true)
            .show(ui, |ui| {
                if ui.button("clear").clicked() {
                    self.last_events.clear();
                }
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        self.last_events.iter().rev().for_each(|event| {
                            ui.label(event);
                        });
                    });
            });
    }

    fn draw_section_debug(&self, ui: &mut Ui) {
        draw_section_debug(
            ui,
            ValuesSectionDebug {
                zoom: self.zoom,
                pan: self.pan,
                            },
        );
    }

    pub fn reset(&mut self) {
        let mut force = init_force(&self.force_settings);
        let mut sim = fdg::init_force_graph_uniform(self.g.g.clone(), 1.0);
        force.apply(&mut sim);
        self.g.g.node_weights_mut().for_each(|node| {
            let point: fdg::nalgebra::OPoint<f32, fdg::nalgebra::Const<2>> =
                sim.node_weight(node.id()).unwrap().1;
            node.set_location(Pos2::new(point.coords.x, point.coords.y));
        });

        self.sim = sim;
        self.force = force;
    }

    
    pub fn gui_windows(&mut self, ctx: &Context) {
        if !self.show {
            return;
        }

        egui::SidePanel::right("right_panel")
            .min_width(250.)
            .show(ctx, |ui|  {
            ScrollArea::vertical().show(ui, |ui| {
                CollapsingHeader::new("Simulation")
                    .default_open(true)
                    .show(ui, |ui| self.draw_section_simulation(ui));

                ui.add_space(10.);

                egui::CollapsingHeader::new("Debug")
                    .default_open(true)
                    .show(ui, |ui| self.draw_section_debug(ui));

                ui.add_space(10.);

                CollapsingHeader::new("Widget")
                    .default_open(true)
                    .show(ui, |ui| self.draw_section_widget(ui));
                });

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let settings_interaction = &egui_graphs::SettingsInteraction::new()
                .with_dragging_enabled(true);


            let settings_navigation = &egui_graphs::SettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.settings_navigation.zoom_and_pan_enabled)
                .with_fit_to_screen_enabled(self.settings_navigation.fit_to_screen_enabled)
                .with_zoom_speed(self.settings_navigation.zoom_speed);


            let settings_style = &egui_graphs::SettingsStyle::new();

            ui.add(
                &mut GraphView::<_, _, _, _ , NodeShape, _>::new(&mut self.g)
                    .with_interactions(settings_interaction)
                    .with_navigations(settings_navigation)
                    .with_styles(settings_style)
            );        }); 
  
    }
}



fn init_force(settings: &ForceSettings) -> FruchtermanReingold<f32, 2> {
    FruchtermanReingold {
        conf: FruchtermanReingoldConfiguration {
            dt: settings.dt,
            cooloff_factor: settings.cooloff_factor,
            scale: settings.scale,
        },
        ..Default::default()
    }
}



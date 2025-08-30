pub mod nodes;
pub mod viewer;
pub mod data_type;
pub mod collapse;
pub mod template;
pub mod number_space;
pub mod position_space;
pub mod primitive;
pub mod identifier;
pub mod volume;
pub mod ammount;
pub mod dependency_tree;
pub mod debug_gui;
pub mod build;

use std::{fs::{self, File}, io::Write};

use build::BS;
use collapse::collapser::Collapser;
use egui_snarl::{ui::{NodeLayout, PinPlacement, SnarlStyle, SnarlWidget}, Snarl};
use nodes::ComposeNode;
use octa_force::{anyhow::anyhow, egui::{self, CornerRadius, Id}, OctaResult};
use template::ComposeTemplate;
use viewer::ComposeViewer;

use crate::util::{number::Nu, vector::Ve};

const TEMP_SAVE_FILE: &str = "./composer_temp_save.json";

#[derive(Debug)]
pub struct ModelComposer<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    snarl: Snarl<ComposeNode<V2, V3, T, B>>,
    style: SnarlStyle,
    viewer: ComposeViewer<V2, V3, T, B>,
    template: ComposeTemplate<V2, V3, T, B>,
    collapser: Collapser<V2, V3, T, B>,
}

const fn default_style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::coil()),
        pin_placement: Some(PinPlacement::Edge),
        pin_size: Some(7.0),
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
        ..SnarlStyle::new()
    }
}

impl<V2, V3, T, B> ModelComposer<V2, V3, T, B> 
where 
    V2: Ve<T, 2> + serde::Serialize + serde::de::DeserializeOwned, 
    V3: Ve<T, 3> + serde::Serialize + serde::de::DeserializeOwned, 
    T: Nu + serde::Serialize + serde::de::DeserializeOwned, 
    B: BS<V2, V3, T> + serde::Serialize + serde::de::DeserializeOwned 
{
    pub fn new() -> Self {
        let snarl = load_snarl().unwrap_or(Snarl::new());       
        let style = SnarlStyle::new();
        let viewer = ComposeViewer::new();

        ModelComposer {
            snarl,
            style, 
            viewer,
            template: Default::default(),
            collapser: Default::default(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) { 
        egui::SidePanel::right("Right Side")
            .default_width(300.0)
            .show(ctx, |ui| {
                if ui.button("Build Template").clicked() {
                    self.template = ComposeTemplate::new(self);
                }

                self.template.debug_render(ui);

                if ui.button("Build Collapser").clicked() {
                    self.collapser = self.template.get_collapser();
                    self.collapser.run(&self.template);
                }

                self.collapser.debug_render(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            SnarlWidget::new()
                .id(Id::new("snarl-demo"))
                .style(self.style)
                .show(&mut self.snarl, &mut self.viewer, ui);
        });
    }

    pub fn update(&mut self) -> OctaResult<()> {
        let snarl = serde_json::to_string(&self.snarl).unwrap();

        let mut file = File::create(TEMP_SAVE_FILE)?;
        file.write_all(snarl.as_bytes())?;
        
        Ok(())
    }
}

pub fn load_snarl<V2, V3, T, B>() -> OctaResult<Snarl<ComposeNode<V2, V3, T, B>>> 
where 
    V2: Ve<T, 2> + serde::de::DeserializeOwned, 
    V3: Ve<T, 3> + serde::de::DeserializeOwned, 
    T: Nu + serde::de::DeserializeOwned, 
    B: BS<V2, V3, T> + serde::de::DeserializeOwned 
{
    let content = fs::read_to_string(TEMP_SAVE_FILE)?; 
    let snarl = serde_json::from_str(&content)?; 
    Ok(snarl)
}


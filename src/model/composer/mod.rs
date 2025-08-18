use std::{fs::{self, File}, io::Write};

use egui_snarl::{ui::{NodeLayout, PinPlacement, SnarlStyle, SnarlWidget}, Snarl};
use nodes::ComposeNode;
use octa_force::{anyhow::anyhow, egui::{self, CornerRadius, Id}, OctaResult};
use viewer::ComposeViewer;

pub mod nodes;
pub mod viewer;
pub mod data_type;

const TEMP_SAVE_FILE: &str = "./composer_temp_save.json";

#[derive(Debug)]
pub struct ModelComposer {
    snarl: Snarl<ComposeNode>,
    style: SnarlStyle,
    viewer: ComposeViewer,
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

impl ModelComposer {
     pub fn new() -> Self {
        let snarl = load_snarl().unwrap_or(Snarl::new());       
        let style = SnarlStyle::new();
        let viewer = ComposeViewer::new();

        ModelComposer { snarl, style, viewer }
    }

    pub fn render(&mut self, ctx: &egui::Context) { 
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

pub fn load_snarl() -> OctaResult<Snarl<ComposeNode>> {
    let content = fs::read_to_string(TEMP_SAVE_FILE)?; 
    let snarl = serde_json::from_str(&content)?; 
    Ok(snarl)
}


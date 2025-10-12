pub mod nodes;
pub mod viewer;
pub mod template;
pub mod dependency_tree;
pub mod build;
pub mod validate;
pub mod pin;

use std::{fs::{self, File}, io::Write, time::Duration};

use build::{ComposeTypeTrait, BS};
use egui_snarl::{ui::{NodeLayout, PinPlacement, SnarlStyle, SnarlWidget}, Snarl};
use nodes::ComposeNode;
use octa_force::{anyhow::anyhow, egui::{self, Align, CornerRadius, Frame, Id, Layout}, glam::{uvec2, UVec2}, log::debug, OctaResult};
use template::ComposeTemplate;
use viewer::ComposeViewer;

use crate::util::{number::Nu, vector::Ve};

use super::collapse::worker::{CollapserChangeReciver, ComposeCollapseWorker};


const TEMP_SAVE_FILE: &str = "./composer_temp_save.json";

#[derive(Debug)]
pub struct ModelComposer<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub snarl: Snarl<ComposeNode<B::ComposeType>>,
    pub style: SnarlStyle,
    pub viewer: ComposeViewer<V2, V3, T, B>,

    pub template: ComposeTemplate<V2, V3, T, B>,
    pub collapser_worker: ComposeCollapseWorker<V2, V3, T, B>,
    pub collapser_reciver: CollapserChangeReciver<V2, V3, T, B>,

    pub auto_rebuild: bool,
    pub manual_rebuild: bool,

    pub render_panel_changed: bool,
    pub render_panel_size: UVec2,
    pub render_panel_offset: UVec2,
}

const fn default_style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::coil()),
        pin_placement: Some(PinPlacement::Edge),
        pin_size: Some(10.0),
        collapsible: Some(false),
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
    V2: Ve<T, 2>, 
    V3: Ve<T, 3>, 
    T: Nu, 
    B: BS<V2, V3, T>,
    B::ComposeType: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(state: B) -> Self {
        let mut snarl = load_snarl().unwrap_or(Snarl::new());       
        let style = default_style();
        let mut viewer = ComposeViewer::new();
        let template = ComposeTemplate::empty();
        let (collapser_worker, collapser_reciver) = ComposeCollapseWorker::new(template.clone(), state);
       
        viewer.check_valid_for_all_nodes(&mut snarl);

        ModelComposer {
            snarl,
            style, 
            viewer,
            template,
            collapser_worker,
            collapser_reciver,
            auto_rebuild: false,
            manual_rebuild: true,

            render_panel_changed: false,
            render_panel_size: UVec2::ZERO,
            render_panel_offset: UVec2::ZERO,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {  
        let res = egui::TopBottomPanel::bottom("Bottom")
            .default_height(500.0)
            .show(ctx, |ui| {
                SnarlWidget::new()
                    .id(Id::new("snarl-demo"))
                    .style(self.style)
                    .show(&mut self.snarl, &mut self.viewer, ui);
            });

        let max_y = res.response.rect.top();

        let res = egui::SidePanel::right("Right Side")
            .min_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        ui.set_min_width(500.0);

                        div(ui, |ui| {
                            ui.checkbox(&mut self.auto_rebuild, "Auto Rebuild");
                            if ui.button("Rebuild").clicked() {
                                self.manual_rebuild = true;
                            }
                        });

                        egui::CollapsingHeader::new("Template:")
                            .show(ui, |ui| {
                                self.template.debug_render(ui);
                            });

                        egui::CollapsingHeader::new("Collapser:")
                            .show(ui, |ui| {
                                self.collapser_reciver.get_collapser().debug_render(ui);
                            });
                    });
            });

        let max_x = res.response.rect.left();
        let new_render_panel_size = uvec2(max_x as u32, max_y as u32); 

        if new_render_panel_size != self.render_panel_size {
            self.render_panel_size = new_render_panel_size;
            self.render_panel_changed = true;
        }
    }

    pub fn update(&mut self, time: Duration) -> OctaResult<()> {
        self.viewer.update(time);

        if (self.viewer.changed && self.auto_rebuild) || self.manual_rebuild {
            self.viewer.changed = false;
            self.manual_rebuild = false;

            if !self.viewer.invalid_nodes.any() {
                debug!("Rebuilding Template");

                self.template = ComposeTemplate::new(self);
                self.collapser_worker.template_changed(self.template.clone());
            }
        } 

        let snarl = serde_json::to_string(&self.snarl).unwrap();
        let mut file = File::create(TEMP_SAVE_FILE)?;
        file.write_all(snarl.as_bytes())?;
        
        Ok(())
    }
}

pub fn load_snarl<CT: ComposeTypeTrait + serde::de::DeserializeOwned>() -> OctaResult<Snarl<ComposeNode<CT>>> {
    let content = fs::read_to_string(TEMP_SAVE_FILE)?; 
    let snarl = serde_json::from_str(&content)?;
    Ok(snarl)
}


fn div(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::none().show(ui, |ui| {
        ui.with_layout(Layout::left_to_right(Align::TOP), add_contents);
    });
}

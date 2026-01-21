pub mod nodes;
pub mod viewer;
pub mod validate;
pub mod pin;
pub mod graph;
pub mod external_input;
pub mod output_state;

use std::{fs::{self, File}, io::Write, time::{Duration, Instant}};

use egui_snarl::{ui::{NodeLayout, PinPlacement, SnarlStyle, SnarlWidget}, Snarl};
use graph::ComposerGraph;
use nodes::ComposeNode;
use octa_force::{anyhow::anyhow, camera::Camera, egui::{self, Align, CornerRadius, Frame, Id, Layout, Margin}, glam::{uvec2, UVec2, Vec2}, log::{debug, info, warn}, OctaResult};
use viewer::{style, ComposeViewer, ComposeViewerData, ComposeViewerTemplates};

use crate::{model::{collapse::external_input::ExternalInput, composer::output_state::OutputState}, scene::worker::SceneWorkerSend, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

use super::{collapse::worker::{CollapserChangeReciver, ComposeCollapseWorker}, template::Template};


#[derive(Debug)]
pub struct ModelComposer {
    pub graph: ComposerGraph,
    
    pub style: SnarlStyle,
    pub viewer_templates: ComposeViewerTemplates,
    pub viewer_data: ComposeViewerData,

    pub template: Template,
    pub collapser_worker: ComposeCollapseWorker,
    pub collapser_reciver: CollapserChangeReciver,

    pub auto_rebuild: bool,
    pub manual_rebuild: bool,

    pub render_panel_changed: bool,
    pub render_panel_size: UVec2,
    pub render_panel_offset: UVec2,

    pub external_input: ExternalInput,
    pub palette: SharedPalette,
}

impl ModelComposer {
    pub fn new(camera: &Camera, mut palette: SharedPalette, scene: SceneWorkerSend) -> Self {
        let mut graph = ComposerGraph::new();

        let style = style();
        let viewer_templates = ComposeViewerTemplates::new();
        let viewer_data = ComposeViewerData::new();

        let mut template = Template::empty();
        template.update(&graph, &mut palette);

        let state = OutputState::new(scene, camera, palette.clone());
        
        let external_input = ExternalInput::new(camera);
        let (collapser_worker, collapser_reciver) = ComposeCollapseWorker::new(template.clone(), external_input, state); 

        ModelComposer {
            graph,
            style,
            viewer_templates,
            viewer_data,

            template,
            collapser_worker,
            collapser_reciver,
            auto_rebuild: false,
            manual_rebuild: false,

            render_panel_changed: false,
            render_panel_size: UVec2::ZERO,
            render_panel_offset: UVec2::ZERO,

            external_input,
            palette,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) { 
        egui_extras::install_image_loaders(ctx);

        let res = egui::TopBottomPanel::bottom("Bottom")
            .default_height(500.0)
            .show(ctx, |ui| {

                let mut viewer = ComposeViewer {
                    templates: &self.viewer_templates,
                    data: &mut self.viewer_data,
                    flags: &mut self.graph.flags,
                    palette: &mut self.palette,
                };

                SnarlWidget::new()
                    .id(Id::new("snarl-demo"))
                    .style(self.style)
                    .show(&mut self.graph.snarl, &mut viewer, ui);
            });

        let split_y = res.response.rect.top();
        self.viewer_data.offset = Vec2::new(0.0, split_y as f32);

        let res = egui::SidePanel::right("Right Side")
            .min_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        ui.set_min_width(500.0);

                        div(ui, Margin::same(10), |ui| {
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
        let new_render_panel_size = uvec2(max_x as u32, split_y as u32); 

        if new_render_panel_size != self.render_panel_size {
            self.render_panel_size = new_render_panel_size;
            self.render_panel_changed = true;
        }
    }

    pub fn update(&mut self, time: Duration, camera: &Camera) -> OctaResult<()> {
        self.viewer_data.update(time);
        self.update_external_input(camera);

        if self.manual_rebuild || (self.auto_rebuild && self.graph.flags.needs_collapse_nodes.any())  {
            self.manual_rebuild = false;

            if !self.graph.flags.invalid_nodes.any() {
                debug!("Rebuilding Template");
                
                let now = Instant::now();

                let updates = self.template.update(&self.graph, &mut self.palette);
                self.collapser_worker.template_changed(self.template.clone(), updates, self.external_input);

                let elapsed = now.elapsed();
                info!("Template took: {:?}", elapsed);
                
                self.graph.flags.reset_change_flags();

            } else {
                warn!("Cant Rebuild error in graph!");
            }
        } 
       
        self.graph.save()?;

        Ok(())
    }
}


fn div(ui: &mut egui::Ui, margin: Margin, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::NONE
        .outer_margin(margin)
        .show(ui, |ui| {
        ui.with_layout(Layout::left_to_right(Align::TOP), add_contents);
    });
}

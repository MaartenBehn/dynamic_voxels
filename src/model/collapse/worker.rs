use std::time::Instant;

use octa_force::{anyhow::Context, camera::Camera, log::{debug, error, info, trace}, OctaResult};

use crate::{model::{composer::output_state::OutputState, template::{Template}}, scene::worker::SceneWorkerSend, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

use super::{collapser::Collapser, external_input::{self, ExternalInput}};

#[derive(Debug)]
pub struct ComposeCollapseWorker {
    pub task: smol::Task<()>,
    pub update_s: smol::channel::Sender<CollapserUpdate>,
}

enum CollapserUpdate {
    Template(Template),
    ExternalInput(ExternalInput)
}

#[derive(Debug)]
pub struct CollapserChangeReciver {
    r: smol::channel::Receiver<Collapser>,
    collapser: Collapser,
    closed: bool,
}

impl ComposeCollapseWorker {
    pub fn new(
        template: Template,
        external_input: ExternalInput,
        state: OutputState,
    ) -> (Self, CollapserChangeReciver) {
        let collasper = smol::block_on(Collapser::new(template, external_input, state));

        let (update_s, update_r) = 
            smol::channel::bounded(1);

        let (collapser_s, collapser_r) = 
            smol::channel::bounded(1);
         
        let task = smol::spawn(Self::run(
            update_r, 
            collapser_s, 
            collasper.clone()));

        (
            ComposeCollapseWorker {
                task,
                update_s,
            },
            CollapserChangeReciver {
                r: collapser_r,
                collapser: collasper,
                closed: false,
            }
        )
    }

    async fn run(
        update_r: smol::channel::Receiver<CollapserUpdate>,
        collapser_s: smol::channel::Sender<Collapser>, 
        mut collapser: Collapser,
    ) {
        
        let now = Instant::now();
        info!("Start at: {:?}", now);

        collapser.run().await;

        let elapsed = now.elapsed();
        info!("Collapse took: {:?}", elapsed);

        let _ = collapser_s.force_send(collapser.clone());

        loop {
            match update_r.recv().await {
                Ok(up) => {
                    match up {
                        CollapserUpdate::Template(template) => {
                            collapser.template_changed(template);

                            let now = Instant::now();
                            info!("Start at: {:?}", now);
                            collapser.run().await;

                            let elapsed = now.elapsed();
                            info!("Collapse took: {:?}", elapsed);
                            
                            let _ = collapser_s.force_send(collapser.clone());
                        },
                        CollapserUpdate::ExternalInput(external_input) => {
                            collapser.external_input = external_input;
                            collapser.external_input_changed();

                            let now = Instant::now();
                            info!("Start at: {:?}", now);
                            collapser.run().await;

                            let elapsed = now.elapsed();
                            info!("Collapse took: {:?}", elapsed);
                        },
                    }
                },
                Err(e) => break,
            }
        }
    }

    pub fn template_changed(&self, template: Template) {
        let _ = self.update_s.force_send(CollapserUpdate::Template(template));
    }

    pub fn external_input_changed(&self, external_input: ExternalInput) {
        let _ = self.update_s.force_send(CollapserUpdate::ExternalInput(external_input));
    }
}

impl CollapserChangeReciver {
    pub fn get_collapser(&mut self) -> &Collapser {
        if self.closed {
            return &self.collapser
        }

        match self.r.try_recv() {
            Ok(collapser) => {
                self.collapser = collapser;
                &self.collapser
            },
            Err(e) => match e {
                smol::channel::TryRecvError::Empty => {
                    &self.collapser
                },
                smol::channel::TryRecvError::Closed => {
                    error!("Collapser Change Channel closed");
                    self.closed = true;
                    &self.collapser
                },
            },
        }
    }
 
}

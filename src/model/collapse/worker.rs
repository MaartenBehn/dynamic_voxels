use std::time::Instant;

use octa_force::{anyhow::Context, camera::Camera, log::{debug, error, info, trace}, OctaResult};

use crate::{model::{composer::output_state::OutputState, template::{Template, update::TemplateNodeUpdate}}, scene::worker::SceneWorkerSend, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

use super::{collapser::Collapser, external_input::{self, ExternalInput}};


#[derive(Debug)]
pub struct ComposeCollapseWorker {
    pub task: smol::Task<()>,
    pub update_s: smol::channel::Sender<(Template, Vec<TemplateNodeUpdate>, ExternalInput)>,
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
        engine_data: ExternalInput,
        mut state: OutputState,
    ) -> (Self, CollapserChangeReciver) {
        let collasper = smol::block_on(Collapser::new(&template));

        let (update_s, update_r) = 
            smol::channel::bounded(1);

        let (collapser_s, collapser_r) = 
            smol::channel::bounded(1);
         
        let task = smol::spawn(Self::run(
            update_r, 
            collapser_s, 
            template, 
            collasper.clone(), 
            engine_data,
            state));

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
        update_r: smol::channel::Receiver<(Template, Vec<TemplateNodeUpdate>, ExternalInput)>,
        collapser_s: smol::channel::Sender<Collapser>, 
        mut template: Template,
        mut collapser: Collapser,
        engine_data: ExternalInput,
        mut state: OutputState,
    ) {
        let now = Instant::now();

        collapser.run(&template, engine_data, &mut state).await;

        let elapsed = now.elapsed();
        info!("Collapse took: {:?}", elapsed);

        let _ = collapser_s.force_send(collapser.clone());

        loop {
            match update_r.recv().await {
                Ok((new_template, updates, external_input)) => {
                    let now = Instant::now();

                    collapser.template_changed(&new_template, &template, updates);
                    template = new_template;

                    collapser.run(&template, external_input, &mut state).await;

                    let elapsed = now.elapsed();
                    info!("Collapse took: {:?}", elapsed);

                    let _ = collapser_s.force_send(collapser.clone());
                },
                Err(e) => break,
            }
        }
    }

    pub fn template_changed(&self, template: Template, update: Vec<TemplateNodeUpdate>, external_input: ExternalInput) {
        let _ = self.update_s.force_send((template, update, external_input));
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

use std::time::Instant;

use octa_force::{anyhow::Context, camera::Camera, log::{debug, error, info, trace}, OctaResult};

use crate::{model::{composer::build::BS, template::{update::TemplateNodeUpdate, ComposeTemplate}}, scene::worker::SceneWorkerSend, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

use super::{collapser::Collapser, external_input::{self, ExternalInput}};


#[derive(Debug)]
pub struct ComposeCollapseWorker<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub task: smol::Task<()>,
    pub update_s: smol::channel::Sender<(ComposeTemplate<V2, V3, T, B>, Vec<TemplateNodeUpdate>, ExternalInput)>,
}

#[derive(Debug)]
pub struct CollapserChangeReciver<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    r: smol::channel::Receiver<Collapser<V2, V3, T, B>>,
    collapser: Collapser<V2, V3, T, B>,
    closed: bool,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeCollapseWorker<V2, V3, T, B> {
    pub fn new(
        template: ComposeTemplate<V2, V3, T, B>,
        mut state: B, 
        engine_data: ExternalInput,
    ) -> (Self, CollapserChangeReciver<V2, V3, T, B>) {
        let collasper = smol::block_on(Collapser::<V2, V3, T, B>::new(&template, &mut state));

        let (update_s, update_r) = 
            smol::channel::bounded(1);

        let (collapser_s, collapser_r) = 
            smol::channel::bounded(1);
         
        let task = smol::spawn(Self::run(
            update_r, 
            collapser_s, 
            template, 
            collasper.clone(), 
            state, 
            engine_data));

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
        update_r: smol::channel::Receiver<(ComposeTemplate<V2, V3, T, B>, Vec<TemplateNodeUpdate>, ExternalInput)>,
        collapser_s: smol::channel::Sender<Collapser::<V2, V3, T, B>>, 
        mut template: ComposeTemplate<V2, V3, T, B>,
        mut collapser: Collapser<V2, V3, T, B>,
        mut state: B, 
        engine_data: ExternalInput,
    ) {
        let now = Instant::now();

        collapser.run(&template, &mut state, engine_data).await;

        let elapsed = now.elapsed();
        info!("Collapse took: {:?}", elapsed);

        let _ = collapser_s.force_send(collapser.clone());

        loop {
            match update_r.recv().await {
                Ok((new_template, updates, external_input)) => {
                    let now = Instant::now();

                    collapser.template_changed(&new_template, &template, updates, &mut state);
                    template = new_template;

                    collapser.run(&template, &mut state, external_input).await;

                    let elapsed = now.elapsed();
                    info!("Collapse took: {:?}", elapsed);

                    let _ = collapser_s.force_send(collapser.clone());
                },
                Err(e) => break,
            }
        }
    }

    pub fn template_changed(&self, template: ComposeTemplate<V2, V3, T, B>, update: Vec<TemplateNodeUpdate>, external_input: ExternalInput) {
        let _ = self.update_s.force_send((template, update, external_input));
    }
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> CollapserChangeReciver<V2, V3, T, B> {
    pub fn get_collapser(&mut self) -> &Collapser<V2, V3, T, B> {
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

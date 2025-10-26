use octa_force::{anyhow::Context, camera::Camera, log::{debug, error, trace}, OctaResult};

use crate::{model::{composer::build::BS, template::ComposeTemplate}, scene::worker::SceneWorkerSend, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

use super::collapser::Collapser;


#[derive(Debug)]
pub struct ComposeCollapseWorker<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub task: smol::Task<()>,
    pub update_s: smol::channel::Sender<ComposeTemplate<V2, V3, T, B>>,
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
    ) -> (Self, CollapserChangeReciver<V2, V3, T, B>) {
        let collasper = smol::block_on(Collapser::<V2, V3, T, B>::new(&template, &mut state));

        let (update_s, update_r) = 
            smol::channel::bounded(1);

        let (collapser_s, collapser_r) = 
            smol::channel::bounded(1);
         
        let task = smol::spawn(Self::run(update_r, collapser_s, template, collasper.clone(), state));

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
        update_r: smol::channel::Receiver<ComposeTemplate<V2, V3, T, B>>,
        collapser_s: smol::channel::Sender<Collapser::<V2, V3, T, B>>, 
        template: ComposeTemplate<V2, V3, T, B>,
        mut collapser: Collapser<V2, V3, T, B>,
        mut state: B, 
    ) {
        collapser.run(&template, &mut state).await;

        let _ = collapser_s.force_send(collapser.clone());

        loop {
            match update_r.recv().await {
                Ok(template) => {
                    collapser.template_changed(&template, &mut state);
                    collapser.run(&template, &mut state).await;
                    let _ = collapser_s.force_send(collapser.clone());
                },
                Err(e) => break,
            }
        }
    }

    pub fn template_changed(&self, template: ComposeTemplate<V2, V3, T, B>) {
        let _ = self.update_s.force_send(template);
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

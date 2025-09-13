use octa_force::{anyhow::Context, camera::Camera, log::{debug, error, trace}, OctaResult};

use crate::{model::composer::{build::BS, template::ComposeTemplate}, scene::worker::SceneWorkerSend, util::{number::Nu, vector::Ve}, voxel::palette::shared::SharedPalette};

use super::collapser::Collapser;


#[derive(Debug)]
pub struct ComposeCollapseWorker<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub task: smol::Task<()>,
    pub update_s: smol::channel::Sender<ComposeTemplate<V2, V3, T, B>>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> ComposeCollapseWorker<V2, V3, T, B> {
    pub fn new(
        template: ComposeTemplate<V2, V3, T, B>,
        state: B, 
    ) -> Self {
        let (update_s, update_r) = 
            smol::channel::bounded(1);
         
        let task = smol::spawn(Self::run(update_r, template, state));

        ComposeCollapseWorker {
            task,
            update_s,
        }
    }

    async fn run(
        update_r: smol::channel::Receiver<ComposeTemplate<V2, V3, T, B>>, 
        template: ComposeTemplate<V2, V3, T, B>,
        mut state: B, 
    ) {

        let mut collasper = Collapser::<V2, V3, T, B>::new(&template, &mut state).await;
        collasper.run(&template, &mut state).await;

        loop {
            match update_r.recv().await {
                Ok(template) => {
                    collasper.template_changed(&template, &mut state);
                    collasper.run(&template, &mut state).await;
                },
                Err(e) => break,
            }
        }
    }

    pub fn template_changed(&self, template: ComposeTemplate<V2, V3, T, B>) {
        let _ = self.update_s.force_send(template);
    }
}

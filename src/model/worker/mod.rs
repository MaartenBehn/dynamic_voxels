use octa_force::{camera::Camera, log::debug};

use crate::{scene::worker::SceneWorkerSend, voxel::palette::shared::SharedPalette};

use super::generation::{collapse::Collapser, template::TemplateTree, traits::{Model, ModelGenerationTypes}};


#[derive(Debug)]
pub struct ModelWorker<M: Model> {
    pub task: smol::Task<M>,
    pub update_s: smol::channel::Sender<M::UpdateData>,
    pub template_r: smol::channel::Receiver<TemplateTree<M::GenerationTypes>>,
}

#[derive(Debug)]
pub struct ModelChangeSender<T: ModelGenerationTypes> {
    template_s: smol::channel::Sender<TemplateTree<T>>,
}

#[derive(Debug)]
pub struct TemplateChangeReciver<T: ModelGenerationTypes> {
    r: smol::channel::Receiver<TemplateTree<T>>,
    template: TemplateTree<T>,
}

impl<M: Model + 'static> ModelWorker<M> {
    pub fn new(mut palette: SharedPalette, scene: SceneWorkerSend) -> ModelWorker<M> {
        let (update_s, update_r) = smol::channel::bounded(1);
        let (template_s, template_r) = smol::channel::bounded(1);
        let change_s = ModelChangeSender::new(template_s);

        let task = smol::spawn(async move {
            let mut model = M::new(&mut palette, &scene, &change_s).await.expect("Failed to create Model");
            loop {}

            loop {
                let ticked = model.tick(&scene, 10, &change_s).await.expect("Failed to tick");

                if ticked {
                    match update_r.try_recv() {
                        Ok(data) => {
                            model.update(data, &change_s).await.expect("Failed to update Model");
                        },
                        Err(e) => match e {
                            smol::channel::TryRecvError::Empty => {},
                            smol::channel::TryRecvError::Closed => break,
                        },
                    }
                } else {
                    match update_r.recv().await {
                        Ok(data) => {
                            model.update(data, &change_s).await.expect("Failed to update Islands");
                        },
                        Err(e) => break,
                    }
                }
            }

            model
        });

        ModelWorker {
            task,
            update_s,
            template_r,
        }
    }

    pub fn update(&self, update_data: M::UpdateData) {
        let _ = self.update_s.force_send(update_data);
    }

    pub fn stop(self) -> M {
        self.update_s.close();
        smol::block_on(async {
            self.task.await
        })
    }

    pub fn get_template(&self) -> TemplateChangeReciver<M::GenerationTypes> {
        TemplateChangeReciver { 
            r: self.template_r.clone(), 
            template: TemplateTree::default(),
        }
    }
}

impl<T: ModelGenerationTypes> ModelChangeSender<T> {
    pub fn new(template_s: smol::channel::Sender<TemplateTree<T>>) -> Self {
        Self {
            template_s,
        }
    }

    pub fn send_template(&self, template: TemplateTree<T>) {
        let _ = self.template_s.force_send(template);
    }

    pub fn send_collapser(&self, collapser: Collapser<T>) {
        todo!()
    }
}

impl<T: ModelGenerationTypes> TemplateChangeReciver<T> {
    pub fn get_template(&mut self) -> &TemplateTree<T> {
        match self.r.try_recv() {
            Ok(template) => {
                self.template = template;
                &self.template
            },
            Err(e) => match e {
                smol::channel::TryRecvError::Empty => {
                    &self.template
                },
                smol::channel::TryRecvError::Closed => {
                    panic!("Template Change Channel closed");
                },
            },
        }
    }
 
}

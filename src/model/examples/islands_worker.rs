use octa_force::{camera::Camera, log::debug};

use crate::{scene::worker::SceneWorkerSend, voxel::palette::shared::SharedPalette};

use super::islands::{Islands, UpdateData};

#[derive(Debug)]
pub struct IslandsWorker {
    pub task: smol::Task<Islands>,
    pub update_s: smol::channel::Sender<UpdateData>
}

impl IslandsWorker {
    pub fn new(mut palette: SharedPalette, scene: SceneWorkerSend) -> IslandsWorker {
        let (update_s, update_r) = smol::channel::bounded(1); 


        let task = smol::spawn(async move {
            let mut islands = Islands::new(&mut palette, &scene).await.expect("Failed to create Islands");

            loop {
                let ticked = islands.tick(&scene, 10).await.expect("Failed to tick");

                if ticked {
                    match update_r.try_recv() {
                        Ok(data) => {
                            islands.update(data).expect("Failed to update Islands");
                        },
                        Err(e) => match e {
                            smol::channel::TryRecvError::Empty => {},
                            smol::channel::TryRecvError::Closed => break,
                        },
                    }
                } else {
                    match update_r.recv().await {
                        Ok(data) => {
                            islands.update(data).expect("Failed to update Islands");
                        },
                        Err(e) => break,
                    }
                }
            }

            islands
        });

        IslandsWorker {
            task,
            update_s,
        }
    }

    pub fn update(&self, camera: &Camera) {
        self.update_s.force_send(UpdateData {
            pos: camera.get_position_in_meters(),
        }).expect("Islands Worker Update Channel closed");
    }

    pub fn stop(self) -> Islands {
        self.update_s.close();
        smol::block_on(async {
            self.task.await
        })
    }
}

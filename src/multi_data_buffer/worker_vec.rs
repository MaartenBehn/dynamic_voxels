use core::fmt;

use octa_force::{log::debug, OctaResult};
use smallvec::SmallVec;
use smol::future::FutureExt;

use crate::util::worker_message::{WorkerMessage, WorkerMessageHandle};

use super::cached_vec::CachedVec;

pub enum WorkerVecMessage<const N: usize, T> {
    Push(WorkerMessage<SmallVec<[T; N]>, OctaResult<u32>>),
    PushSingle(WorkerMessage<T, OctaResult<u32>>),
}

pub struct WorkerVec<const N: usize, T> {
    task: smol::Task<CachedVec<T>>,
    s: smol::channel::Sender<WorkerVecMessage<N, T>>,
}

impl<T: Send + Copy + Default + fmt::Debug + Eq + std::hash::Hash + 'static> CachedVec<T> {
    pub fn run_worker<const N: usize>(mut self, cap: usize) -> WorkerVec<N, T>{
        let (s, r) = smol::channel::bounded(cap); 

        let task = smol::spawn(async move {
            loop {
                match r.recv().await {
                    Ok(m) => match m {
                        WorkerVecMessage::Push(worker_message) => {
                            let res = self.push(&worker_message.data);
                            worker_message.awnser(res).await;
                        },
                        WorkerVecMessage::PushSingle(worker_message) => {
                            let res = self.push(&[worker_message.data]);
                            worker_message.awnser(res).await;
                        },
                    },
                    Err(_) => {
                        debug!("Got close");
                        break;
                    }, // Channel closed
                }
            } 

            self
        });

        WorkerVec {
            task,
            s,
        }
    }
}

impl<const N: usize, T> WorkerVec<N, T> {
    pub fn push(&self, data: SmallVec<[T; N]>) -> WorkerMessageHandle<OctaResult<u32>> {
        let (msg, res) = WorkerMessage::new(data);

        smol::block_on(async {
            self.s.send(WorkerVecMessage::Push(msg))
                .await.expect("Send channel to worker closed!");
        });

        res
    }

    pub fn push_single(&mut self, data: T) -> WorkerMessageHandle<OctaResult<u32>> {
        let (msg, res) = WorkerMessage::new(data);

        smol::block_on(async {
            self.s.send(WorkerVecMessage::PushSingle(msg))
                .await.expect("Send channel to worker closed!");
        });

        res
    }

    pub fn stop(self) -> CachedVec<T> {
        self.s.close();
        smol::block_on(async {
            self.task.await
        })
    }
}

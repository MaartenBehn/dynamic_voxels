

pub struct WorkerMessage<S, R> {
    pub data: S,
    back_s: oneshot::Sender<R>
}

pub struct WorkerMessageHandle<R> {
    back_r: oneshot::Receiver<R>
}

impl<S, R> WorkerMessage<S, R> {
    pub fn new(data: S) -> (WorkerMessage<S, R>, WorkerMessageHandle<R>) {
        let (back_s, back_r) = oneshot::channel();
        (WorkerMessage { data, back_s }, WorkerMessageHandle { back_r })
    }

    pub async fn awnser(self, reponse: R) {
        self.back_s.send(reponse).expect("Back Channel closed!");
    }
}

impl<R> WorkerMessageHandle<R> {
    pub fn result_blocking(self) -> R {
        self.back_r.recv().unwrap()
    }

    pub async fn result_async(self) -> R {
        self.back_r.await.unwrap()
    }

    pub fn has_result(&self) -> bool {
        self.back_r.has_message()
    }
}



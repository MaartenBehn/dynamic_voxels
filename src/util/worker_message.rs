

pub struct WithRespose<S, R> {
    pub data: S,
    back_s: oneshot::Sender<R>
}

pub struct WorkerRespose<R> {
    back_r: oneshot::Receiver<R>
}

impl<S, R> WithRespose<S, R> {
    pub fn new(data: S) -> (WithRespose<S, R>, WorkerRespose<R>) {
        let (back_s, back_r) = oneshot::channel();
        (WithRespose { data, back_s }, WorkerRespose { back_r })
    }

    pub fn unwarp(self) -> (S, impl FnOnce(R)) {
        (self.data, |res: R| {
            self.back_s.send(res).expect("Repose channel cloesed!");
        })
    }
}

impl<R> WorkerRespose<R> {
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



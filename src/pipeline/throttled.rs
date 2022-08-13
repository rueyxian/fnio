use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;

use crate::func::DynFunc;

///
pub struct Throttled<I, O> {
    func: Arc<DynFunc<I, O>>,
    cap: usize,
    sem: Arc<Semaphore>,
}

///
impl<I, O> Throttled<I, O>
where
    I: Send + 'static,
    O: Send + 'static + Debug,
{

    ///
    pub(super) fn new(cap: usize, func: DynFunc<I, O>) -> Throttled<I, O> {
        let sem = Arc::new(Semaphore::new(cap));
        let func = Arc::new(func);
        Throttled { func, cap, sem }
    }

    ///
    pub fn channel(
        &self,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::mpsc::Sender<I>,
        tokio::sync::mpsc::Receiver<O>,
    ) {
        let (i_send, i_recv) = {
            let (s, r) = tokio::sync::mpsc::channel::<I>(self.cap);
            let r = Arc::new(Mutex::new(r));
            (s, r)
        };

        let (o_send, o_recv) = {
            let (s, r) = tokio::sync::mpsc::channel::<O>(self.cap);
            let s = Arc::new(s);
            (s, r)
        };

        let (jh_send, mut jh_recv) = tokio::sync::mpsc::unbounded_channel();

        for _ in 0..self.cap {
            let func = Arc::clone(&self.func);
            let sem = Arc::clone(&self.sem);
            let i_recv = Arc::clone(&i_recv);
            let o_send = Arc::clone(&o_send);
            let jh_send = jh_send.clone();
            let jh = tokio::spawn(async move {
                loop {
                    let i = {
                        let mut guard = i_recv.lock().await;
                        match guard.recv().await {
                            Some(i) => i,
                            None => break,
                        }
                    };
                    let permit = sem.acquire().await.unwrap();
                    let o = func(i).await;
                    drop(permit);
                    o_send.send(o).await.unwrap();
                }
            });
            jh_send.send(jh).unwrap();
        }

        let jh_out = tokio::spawn(async move {
            while let Some(jh) = jh_recv.recv().await {
                jh.await.unwrap();
            }
        });

        (jh_out, i_send, o_recv)
    }
}

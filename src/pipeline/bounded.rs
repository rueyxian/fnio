use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::func::DynFunc;

///
pub struct Bounded<I, O> {
    func: Arc<DynFunc<I, O>>,
}

///
impl<I, O> Bounded<I, O>
where
    I: Send + 'static,
    O: Send + 'static + Debug,
{
    ///
    pub(super) fn new(func: DynFunc<I, O>) -> Self {
        let func = Arc::new(func);
        Self { func }
    }

    ///
    pub fn channel(
        &self,
        cap: usize,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::mpsc::Sender<I>,
        tokio::sync::mpsc::Receiver<O>,
    ) {
        let (i_send, i_recv) = {
            let (s, r) = tokio::sync::mpsc::channel::<I>(cap);
            let r = Arc::new(Mutex::new(r));
            (s, r)
        };

        let (o_send, o_recv) = {
            let (s, r) = tokio::sync::mpsc::channel::<O>(cap);
            let s = Arc::new(s);
            (s, r)
        };

        let (jh_send, mut jh_recv) = tokio::sync::mpsc::unbounded_channel();

        for _ in 0..cap {
            let func = Arc::clone(&self.func);
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
                    let o = func(i).await;
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

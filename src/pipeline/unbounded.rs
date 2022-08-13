use std::fmt::Debug;
use std::sync::Arc;

use crate::func::DynFunc;

///
pub struct Unbounded<I, O> {
    func: Arc<DynFunc<I, O>>,
}

///
impl<I, O> Unbounded<I, O>
where
    I: Send + 'static,
    O: Send + 'static + Debug,
{
    ///
    pub(super) fn new(func: DynFunc<I, O>) -> Unbounded<I, O> {
        let func = Arc::new(func);
        Unbounded { func }
    }

    ///
    pub fn channel(
        &self,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::mpsc::UnboundedSender<I>,
        tokio::sync::mpsc::UnboundedReceiver<O>,
    ) {
        let (i_send, mut i_recv) = tokio::sync::mpsc::unbounded_channel::<I>();

        let (o_send, o_recv) = {
            let (s, r) = tokio::sync::mpsc::unbounded_channel::<O>();
            let s = Arc::new(s);
            (s, r)
        };

        let (jh_send, mut jh_recv) = {
            let (s, r) = tokio::sync::mpsc::unbounded_channel();
            let s = Arc::new(s);
            (s, r)
        };

        let func = Arc::clone(&self.func);

        let _ = tokio::spawn(async move {
            while let Some(i) = i_recv.recv().await {
                let func = Arc::clone(&func);
                let o_send = Arc::clone(&o_send);
                let jh_send = Arc::clone(&jh_send);
                let jh = tokio::spawn(async move {
                    let o = func(i).await;
                    o_send.send(o).unwrap();
                });
                jh_send.send(jh).unwrap();
            }
        });

        let jh_out = tokio::spawn(async move {
            while let Some(jh) = jh_recv.recv().await {
                jh.await.unwrap();
            }
        });

        (jh_out, i_send, o_recv)
    }
}

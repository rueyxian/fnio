use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::pipeline::PipelineBuilder;

///
pub(crate) type DynFunc<I, O> =
    Box<dyn Fn(I) -> Pin<Box<dyn Future<Output = O> + Send>> + Send + Sync>;

///
pub fn func<F, Fut, I, O>(f: F) -> Func<I, O>
where
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = O> + Send + 'static,
    I: Send + 'static,
    O: Send + 'static + Debug,
{
    let func = {
        let future = move |input: I| {
            let fut = f(input);
            Box::pin(fut) as Pin<Box<dyn Future<Output = O> + Send>>
        };
        Box::new(future) as DynFunc<I, O>
    };
    Func { func }
}

///
pub struct Func<I, O> {
    func: DynFunc<I, O>,
}

///
impl<I, O> Func<I, O>
where
    I: Send + 'static,
    O: Send + 'static + Debug,
{

///
    pub fn pipeline(self) -> PipelineBuilder<I, O> {
        PipelineBuilder::new(self.func)
    }
}

use std::fmt::Debug;

use super::Bounded;
use super::Throttled;
use super::Unbounded;
use crate::func::DynFunc;

///
pub struct PipelineBuilder<I, O> {
    func: DynFunc<I, O>,
}



    ///
impl<I, O> PipelineBuilder<I, O>
where
    I: Send + 'static,
    O: Send + 'static + Debug,
{

    ///
    pub(crate) fn new(func: DynFunc<I, O>) -> Self {
        Self { func }
    }

    ///
    pub fn throttled(self, cap: usize) -> Throttled<I, O> {
        Throttled::<I, O>::new(cap, self.func)
    }

    ///
    pub fn bounded(self) -> Bounded<I, O> {
        Bounded::<I, O>::new(self.func)
    }

    ///
    pub fn unbounded(self) -> Unbounded<I, O> {
        Unbounded::<I, O>::new(self.func)
    }
}

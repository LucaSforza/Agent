use std::{
    fmt,
    marker::PhantomData,
    time::{Duration, Instant},
};

use crate::problem::{CostructSolution, RandomState, Utility};

use super::algorithms::ImprovingAlgorithm;

pub struct AttemptResult<P>
where
    P: CostructSolution,
{
    pub state: P::State,
    pub h: P::Cost,
    pub iterations: usize,
}

impl<P> AttemptResult<P>
where
    P: CostructSolution,
{
    pub fn new(state: P::State, h: P::Cost, iterations: usize) -> Self {
        Self {
            state: state,
            h: h,
            iterations: iterations,
        }
    }
}

pub struct ResolverResult<P>
where
    P: CostructSolution,
{
    pub state: P::State,
    pub h: P::Cost,
    pub iterations: usize,
    pub duration: Duration,
}

impl<P> fmt::Debug for ResolverResult<P>
where
    P: CostructSolution<State: fmt::Debug, Cost: fmt::Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "state:\n{:?}", self.state)?;
        writeln!(f, "h: {:?}", self.h)?;
        writeln!(f, "iterations: {:?}", self.iterations)?;
        write!(f, "duration: {:?}", self.duration)
    }
}

impl<P> ResolverResult<P>
where
    P: CostructSolution,
{
    pub fn from_inner(start: Instant, inner: AttemptResult<P>) -> Self {
        Self {
            state: inner.state,
            h: inner.h,
            iterations: inner.iterations,
            duration: start.elapsed(),
        }
    }
}

pub struct Resolver<I, P>
where
    P: CostructSolution,
    I: ImprovingAlgorithm<P>,
{
    algo: I,
    _problem: PhantomData<P>,
}

impl<I, P> Resolver<I, P>
where
    P: RandomState + Utility,
    I: ImprovingAlgorithm<P>,
{
    pub fn new(algo: I) -> Self {
        Self {
            algo: algo,
            _problem: PhantomData,
        }
    }
}

impl<I, P> Resolver<I, P>
where
    P: CostructSolution,
    I: ImprovingAlgorithm<P>,
{
    pub fn resolve(&mut self, problem: &P) -> ResolverResult<P> {
        let start = Instant::now();
        let inner = self.algo.attempt(problem);
        return ResolverResult::from_inner(start, inner);
    }

    pub fn resolve_restart(&mut self, problem: &P, max_restarts: usize) -> ResolverResult<P> {
        let start = Instant::now();
        let mut result = self.algo.attempt(problem);
        for _ in 1..max_restarts {
            let new_result = self.algo.attempt(problem);
            if new_result.h < result.h {
                result.state = new_result.state;
                result.h = new_result.h;
            }
            result.iterations += new_result.iterations;
        }
        return ResolverResult::from_inner(start, result);
    }
}

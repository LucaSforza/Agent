use core::fmt;
use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use rand::Rng;

use crate::problem::IterativeImprovingProblem;

pub struct InnerResolverResult<P>
where
    P: IterativeImprovingProblem,
{
    pub state: P::State,
    pub h: P::Cost,
    pub iterations: usize,
}

impl<P> InnerResolverResult<P>
where
    P: IterativeImprovingProblem,
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
    P: IterativeImprovingProblem,
{
    pub state: P::State,
    pub h: P::Cost,
    pub iterations: usize,
    pub duration: Duration,
}

impl<P> fmt::Debug for ResolverResult<P>
where
    P: IterativeImprovingProblem<State: fmt::Debug, Cost: fmt::Debug>,
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
    P: IterativeImprovingProblem,
{
    pub fn from_inner(start: Instant, inner: InnerResolverResult<P>) -> Self {
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
    P: IterativeImprovingProblem,
    I: ImprovingAlgorithm<P>,
{
    algo: I,
    _problem: PhantomData<P>,
}

impl<I, P> Resolver<I, P>
where
    P: IterativeImprovingProblem,
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
    P: IterativeImprovingProblem,
    I: ImprovingAlgorithm<P>,
{
    pub fn resolve(&mut self, problem: &P) -> ResolverResult<P> {
        let start = Instant::now();
        let inner = self.algo.inner_resolve(problem);
        return ResolverResult::from_inner(start, inner);
    }

    pub fn resolve_restart(&mut self, problem: &P, max_restarts: usize) -> ResolverResult<P> {
        let start = Instant::now();
        let mut result = self.algo.inner_resolve(problem);
        for _ in 1..max_restarts {
            let new_result = self.algo.inner_resolve(problem);
            if new_result.h <= P::Cost::default() {
                result.state = new_result.state;
                result.h = new_result.h;
                result.iterations += new_result.iterations;
                let result = ResolverResult::from_inner(start, result);
                return result;
            }
            if new_result.h < result.h {
                result.state = new_result.state;
                result.h = new_result.h;
            }
            result.iterations += new_result.iterations;
        }
        return ResolverResult::from_inner(start, result);
    }
}

pub trait ImprovingAlgorithm<P>
where
    P: IterativeImprovingProblem,
{
    fn inner_resolve(&mut self, problem: &P) -> InnerResolverResult<P>;
}

pub struct SteepestDescend<R: Rng> {
    rng: R,
}

impl<R: Rng> SteepestDescend<R> {
    pub fn new(rng: R) -> Self {
        Self { rng: rng }
    }
}

impl<R: Rng, P: IterativeImprovingProblem<State: Clone>> ImprovingAlgorithm<P>
    for SteepestDescend<R>
{
    fn inner_resolve(&mut self, problem: &P) -> InnerResolverResult<P> {
        let mut iterations = 0;
        let mut curr_state = problem.random_state(&mut self.rng);
        let mut curr_h = problem.heuristic(&curr_state);
        loop {
            iterations += 1;
            let mut new_curr_state = curr_state.clone();
            let mut new_curr_h = curr_h;
            for a in problem.executable_actions(&curr_state) {
                let (new_state, _) = problem.result(&curr_state, &a);
                let new_h = problem.heuristic(&new_state);
                if new_h < new_curr_h {
                    new_curr_state = new_state;
                    new_curr_h = new_h;
                }
            }
            if curr_h > new_curr_h {
                curr_state = new_curr_state;
                curr_h = new_curr_h;
            } else {
                let result = InnerResolverResult::new(curr_state, curr_h, iterations);
                return result;
            }
        }
    }
}

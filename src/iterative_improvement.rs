use core::fmt;
use std::{
    marker::PhantomData,
    ops::Sub,
    time::{Duration, Instant},
};

use rand::Rng;
use rand_distr::num_traits::Signed;

use crate::problem::*;

pub struct AttemptResult<P>
where
    P: Problem,
{
    pub state: P::State,
    pub h: P::Cost,
    pub iterations: usize,
}

impl<P> AttemptResult<P>
where
    P: Problem,
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
    P: Problem,
{
    pub state: P::State,
    pub h: P::Cost,
    pub iterations: usize,
    pub duration: Duration,
}

impl<P> fmt::Debug for ResolverResult<P>
where
    P: Problem<State: fmt::Debug, Cost: fmt::Debug>,
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
    P: Problem,
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
    P: Problem,
    I: ImprovingAlgorithm<P>,
{
    algo: I,
    _problem: PhantomData<P>,
}

impl<I, P> Resolver<I, P>
where
    P: RandomizeState + Utility,
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
    P: Problem,
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
            if new_result.h <= P::Cost::default() {
                // TODO: check if it is a goal state
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
    P: Problem,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P>;
}

pub struct SteepestDescend<R: Rng> {
    rng: R,
}

impl<R: Rng> SteepestDescend<R> {
    pub fn new(rng: R) -> Self {
        Self { rng: rng }
    }
}

impl<R, P> ImprovingAlgorithm<P> for SteepestDescend<R>
where
    R: Rng,
    P: ModifyState + Utility + RandomizeState<State: Clone>,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P> {
        let mut iterations = 0;
        let mut curr_state = problem.random_state(&mut self.rng);
        let mut curr_h = problem.heuristic(&curr_state);
        loop {
            iterations += 1;
            let mut new_curr_state = curr_state.clone();
            let mut new_curr_h = curr_h;
            for a in problem.modify_actions(&curr_state) {
                let new_state = problem.modify(&curr_state, &a);
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
                let result = AttemptResult::new(curr_state, curr_h, iterations);
                return result;
            }
        }
    }
}

pub struct HillClimbing<R: Rng> {
    rng: R,
    max_lateral: Option<usize>,
}

impl<R: Rng> HillClimbing<R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng: rng,
            max_lateral: None,
        }
    }

    pub fn with_max_lateral(rng: R, max_lateral: usize) -> Self {
        Self {
            rng: rng,
            max_lateral: max_lateral.into(),
        }
    }
}

impl<R> HillClimbing<R>
where
    R: Rng,
{
    fn get_next_state<P: Utility + ModifyState>(
        lateral: &mut usize,
        problem: &P,
        state: &P::State,
        curr_h: P::Cost,
        max_lateral: Option<usize>,
    ) -> Option<(P::State, P::Cost)> {
        let mut actions = problem.modify_actions(state);
        while let Some(a) = actions.next() {
            let next_state = problem.modify(state, &a);
            let next_h = problem.heuristic(&next_state);
            if max_lateral.map_or(true, |x| x > *lateral) && next_h == curr_h {
                *lateral += 1;
                return (next_state, next_h).into();
            }
            if next_h < curr_h {
                *lateral = 0;
                return (next_state, next_h).into();
            }
        }
        None
    }
}

impl<P, R> ImprovingAlgorithm<P> for HillClimbing<R>
where
    P: ModifyState + Utility + RandomizeState<State: Clone>,
    R: Rng,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P> {
        let mut curr_state = problem.random_state(&mut self.rng);
        let mut curr_h = problem.heuristic(&curr_state);
        let mut iterations = 0;
        let mut lateral = 0;
        loop {
            iterations += 1;
            let to_assign =
                Self::get_next_state(&mut lateral, problem, &curr_state, curr_h, self.max_lateral);
            if let Some((next_state, next_h)) = to_assign {
                curr_state = next_state;
                curr_h = next_h;
            } else {
                return AttemptResult::new(curr_state, curr_h, iterations);
            }
        }
    }
}

pub struct SimulatedAnnealing<R: Rng> {
    rng: R,
    cooling: fn(usize) -> f64,
    precision: f64,
}

impl<R: Rng> SimulatedAnnealing<R> {
    pub fn default_cooling(t: usize) -> f64 {
        1.0 / (t as f64)
    }

    pub fn new(rng: R) -> Self {
        Self {
            rng: rng,
            cooling: Self::default_cooling,
            precision: 10e-6,
        }
    }

    pub fn with_cooling(rng: R, cooling: fn(usize) -> f64) -> Self {
        Self {
            rng: rng,
            cooling: cooling,
            precision: 10e-6,
        }
    }
}

use libm::exp;

impl<R, P> ImprovingAlgorithm<P> for SimulatedAnnealing<R>
where
    P: ModifyRandom + Utility + RandomizeState<Cost: Sub<Output = P::Cost> + Into<f64> + Signed>,
    R: Rng,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P> {
        let mut curr_state = problem.random_state(&mut self.rng);
        let mut curr_h = problem.heuristic(&curr_state);

        for t in 0.. {
            let velocity = (self.cooling)(t);
            if curr_h <= Default::default() {
                return AttemptResult::new(curr_state, curr_h, t + 1);
            }
            if velocity <= self.precision {
                return AttemptResult::new(curr_state, curr_h, t + 1);
            }
            let next_action = problem.random_modify_action(&mut self.rng, &curr_state);
            if let Some(next_action) = next_action {
                let next_state = problem.modify(&curr_state, &next_action);
                let next_h = problem.heuristic(&next_state);
                if next_h <= curr_h {
                    curr_state = next_state;
                    curr_h = next_h;
                } else {
                    let diff: f64 = (curr_h - next_h).abs().into();
                    let r: f64 = self.rng.random();
                    if r <= (1.0 / exp(diff / velocity)) {
                        curr_state = next_state;
                        curr_h = next_h;
                    }
                }
            }
        }

        unreachable!()
    }
}

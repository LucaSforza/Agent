use std::{cmp::Reverse, collections::BinaryHeap, fmt, ops::Sub};

use ordered_float::OrderedFloat;
use rand::Rng;
use rand_distr::{
    num_traits::{Inv, Signed},
    weighted::WeightedIndex,
    Distribution,
};

use crate::problem::*;

pub trait ImprovingAlgorithm<P>
where
    P: CostructSolution,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P>;
}

#[derive(PartialEq, Eq)]
pub enum Verbosity {
    Low,
    Max,
}

pub trait VerbosityLevel<P>: ImprovingAlgorithm<P>
where
    P: CostructSolution,
{
    fn set_verbosity(v: Verbosity);
}

pub struct SteepestDescend<R: Rng> {
    verb: Verbosity,
    rng: R,
}

impl<R: Rng> SteepestDescend<R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng: rng,
            verb: Verbosity::Low,
        }
    }

    pub fn with_verbosity(rng: R, verb: Verbosity) -> Self {
        Self {
            verb: verb,
            rng: rng,
        }
    }
}

impl<R, P> ImprovingAlgorithm<P> for SteepestDescend<R>
where
    R: Rng,
    P: StatePerturbation + Utility + RandomState<State: Clone + fmt::Debug, Cost: fmt::Debug>,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P> {
        let mut iterations = 0;
        let mut curr_state = problem.random_state(&mut self.rng);
        let mut curr_h = problem.heuristic(&curr_state);
        loop {
            iterations += 1;
            if self.verb == Verbosity::Max {
                eprintln!(
                    "\nIteration: {}\ncurrent state:\n{:?}\nh:{:?}",
                    iterations, curr_state, curr_h
                );
            }
            let mut new_curr_state = curr_state.clone();
            let mut new_curr_h = curr_h;
            for a in problem.perturbations(&curr_state) {
                let new_state = problem.perturb(&curr_state, &a);
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
    fn get_next_state<P: Utility + StatePerturbation>(
        lateral: &mut usize,
        problem: &P,
        state: &P::State,
        curr_h: P::Cost,
        max_lateral: Option<usize>,
    ) -> Option<(P::State, P::Cost)> {
        let mut actions = problem.perturbations(state);
        while let Some(a) = actions.next() {
            let next_state = problem.perturb(state, &a);
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
    P: StatePerturbation + Utility + RandomState<State: Clone>,
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

use super::resolver::AttemptResult;

impl<R, P> ImprovingAlgorithm<P> for SimulatedAnnealing<R>
where
    P: RandomPerturbation + Utility + RandomState<Cost: Sub<Output = P::Cost> + Into<f64> + Signed>,
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
            let next_action = problem.random_pertubation(&mut self.rng, &curr_state);
            if let Some(next_action) = next_action {
                let next_state = problem.perturb(&curr_state, &next_action);
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

struct Node<P>(Reverse<P::Cost>, P::State)
where
    P: CostructSolution;

impl<P> PartialEq for Node<P>
where
    P: CostructSolution,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P> Eq for Node<P> where P: CostructSolution {}

impl<P> PartialOrd for Node<P>
where
    P: CostructSolution,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<P> Ord for Node<P>
where
    P: CostructSolution,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
pub struct LocalBeam<R: Rng> {
    rng: R,
    k: usize,
    max_iter: Option<usize>,
}

impl<R: Rng> LocalBeam<R> {
    pub fn from_parts(rng: R, k: usize, max_iter: Option<usize>) -> Self {
        Self {
            rng: rng,
            k: k,
            max_iter: max_iter,
        }
    }
}

impl<R, P> ImprovingAlgorithm<P> for LocalBeam<R>
where
    R: Rng,
    P: Utility + StatePerturbation + RandomState + CostructSolution + InitState,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P> {
        let mut current_pop = Vec::with_capacity(self.k);
        for _ in 0..self.k {
            current_pop.push(problem.random_state(&mut self.rng));
        }
        let mut iter = 0;
        let mut succ: BinaryHeap<Node<P>> = BinaryHeap::new();
        loop {
            iter += 1;
            succ.clear();
            for s in current_pop.iter() {
                for a in problem.perturbations(s) {
                    let next_s = problem.perturb(s, &a);
                    let next_h = problem.heuristic(&next_s);
                    if next_h <= Default::default() {
                        return AttemptResult::new(next_s, next_h, iter);
                    } else {
                        succ.push(Node(Reverse(next_h), next_s));
                    }
                }
            }

            if self.max_iter.map_or(false, |max| max < iter) {
                let node = succ
                    .pop()
                    .unwrap_or(Node(Default::default(), problem.init_state()));
                return AttemptResult::new(node.1, node.0 .0, iter);
            }

            current_pop.clear();
            for _ in 0..self.k {
                let next_s = succ.pop().map(|n| n.1);
                if let Some(next_s) = next_s {
                    current_pop.push(next_s);
                } else {
                    break;
                }
            }
            if current_pop.len() == 0 {
                // TODO: make sure that AttemptResult returns a failure
                return AttemptResult::new(problem.init_state(), Default::default(), iter);
            }
        }
    }
}

pub struct GeneticAlgorithm<R: Rng> {
    rng: R,
    k: usize,
    max_iter: Option<usize>,
    pmut: f64,
}

impl<R: Rng> GeneticAlgorithm<R> {
    pub fn from_parts(rng: R, k: usize, max_iter: Option<usize>, pmut: f64) -> Self {
        Self {
            rng: rng,
            k: k,
            max_iter: max_iter,
            pmut: pmut,
        }
    }
}

impl<R, P> ImprovingAlgorithm<P> for GeneticAlgorithm<R>
where
    R: Rng,
    P: MutateGene + Utility<Cost: From<f64> + Into<f64>> + RandomState + Crossover,
{
    fn attempt(&mut self, problem: &P) -> AttemptResult<P> {
        let mut current_pop = Vec::with_capacity(self.k);
        let mut current_weights: Vec<f64> = Vec::with_capacity(self.k);
        for _ in 0..self.k {
            let state = problem.random_state(&mut self.rng);
            let h = problem.heuristic(&state);
            if h <= Default::default() {
                return AttemptResult::new(state, h, 0);
            }
            current_pop.push(state);
            current_weights.push(h.into().inv()); // TODO: aggiungi reverse
        }
        let mut iter = 0;
        let mut distr =
            WeightedIndex::new(&current_weights).expect("Failed to create WeightedIndex");
        loop {
            let mut new_pop = Vec::with_capacity(self.k);
            let mut new_weights = Vec::with_capacity(self.k);
            iter += 1;
            while new_pop.len() < self.k {
                let parent1 = &current_pop[distr.sample(&mut self.rng)];
                let parent2 = &current_pop[distr.sample(&mut self.rng)];

                let mut child = problem.crossover(&mut self.rng, parent1, parent2);

                let r: f64 = self.rng.random();

                if r <= self.pmut {
                    problem.mutate_gene(&mut self.rng, &mut child);
                }

                let child_h = problem.heuristic(&child);

                if child_h <= Default::default() {
                    return AttemptResult::new(child, child_h, iter);
                }

                new_pop.push(child);
                new_weights.push(child_h.into().inv());
            }

            current_pop = new_pop;
            current_weights = new_weights;
            distr = WeightedIndex::new(&current_weights).unwrap();

            // Stop if max iterations reached
            if self.max_iter.map_or(false, |max| max <= iter) {
                let best_s = current_pop
                    .into_iter()
                    .zip(current_weights.into_iter())
                    .min_by_key(|(_, h)| OrderedFloat(*h))
                    .map(|(x, _)| x)
                    .unwrap();
                let best_h = problem.heuristic(&best_s);

                return AttemptResult::new(best_s, best_h, iter);
            }
        }
    }
}

use core::fmt;
use std::{
    collections::HashSet,
    fmt::Debug,
    hash::Hash,
    time::{Duration, Instant},
};

use bumpalo::Bump;

use crate::problem::*;
use crate::statexplorer::frontier::{
    AStarBackend, BestFirstBackend, DequeBackend, Frontier, FrontierBackend, MinCostBackend,
    StackBackend,
};
use crate::statexplorer::node::Node;

struct InnerResult<State, Action>
where
    Action: Clone,
{
    actions: Option<Vec<Action>>,
    state: Option<State>,
    max_frontier_size: usize,
}

impl<State, Action> InnerResult<State, Action>
where
    Action: Clone,
{
    fn found(state: State, actions: Vec<Action>, max_frontier_size: usize) -> Self {
        Self {
            state: state.into(),
            actions: actions.into(),
            max_frontier_size: max_frontier_size,
        }
    }

    fn not_found(max_frontier_size: usize) -> Self {
        Self {
            state: None,
            actions: None,
            max_frontier_size: max_frontier_size,
        }
    }
}

pub struct SearchResult<State, Action>
where
    Action: Clone,
{
    pub total_time: Duration,
    pub state: Option<State>,
    pub actions: Option<Vec<Action>>,
    pub n_iter: usize,
    pub max_frontier_size: usize,
}

impl<State, Action> SearchResult<State, Action>
where
    Action: Clone,
{
    fn new() -> Self {
        Self {
            total_time: Duration::default(),
            actions: None,
            state: None,
            n_iter: 0,
            max_frontier_size: 0,
        }
    }

    fn from_inner_result(
        start: Instant,
        n_iter: usize,
        inner_result: InnerResult<State, Action>,
    ) -> Self {
        Self {
            state: inner_result.state,
            total_time: start.elapsed(),
            actions: inner_result.actions,
            n_iter: n_iter,
            max_frontier_size: inner_result.max_frontier_size,
        }
    }
}

impl<State, Action> fmt::Display for SearchResult<State, Action>
where
    State: Debug,
    Action: Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.state.is_some() && self.actions.is_some() {
            write!(
                f,
                "state: {:?}\nactions: {:?}\ntime: {:?}\niterations: {}\nmax frontier size: {}",
                self.state.as_ref().unwrap(),
                self.actions.as_ref().unwrap(),
                self.total_time,
                self.n_iter,
                self.max_frontier_size
            )
        } else {
            write!(
                f,
                "no solution found\ntime: {:?}\niterations: {}\nmax frontier size: {}",
                self.total_time, self.n_iter, self.max_frontier_size
            )
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Verbosity {
    None,
    Low,
}

pub struct Explorer<'a, P, Backend>
where
    P: Utility + SuitableState, // TODO: generalize more
    Backend: FrontierBackend<'a, P> + Debug,
{
    verbosity: Verbosity,
    problem: &'a P,
    explored: HashSet<P::State>,
    frontier: Frontier<'a, P, Backend>,
    arena: &'a Bump,
}

impl<'a, P, Backend> Explorer<'a, P, Backend>
where
    P: SuitableState + Utility<State: Eq + Hash + Clone + Debug, Action: Clone, Cost: Debug>,
    Backend: FrontierBackend<'a, P> + Debug,
{
    pub fn with_verbosity(problem: &'a P, arena: &'a Bump, verbosity: Verbosity) -> Self {
        Self {
            problem: problem,
            verbosity: verbosity,
            explored: HashSet::new(),
            frontier: Frontier::new(),
            arena: arena,
        }
    }

    pub fn with_low_v(problem: &'a P, arena: &'a Bump) -> Self {
        Self {
            problem: problem,
            verbosity: Verbosity::Low,
            explored: HashSet::new(),
            frontier: Frontier::new(),
            arena: arena,
        }
    }

    pub fn new(problem: &'a P, arena: &'a Bump) -> Self {
        Self {
            problem: problem,
            verbosity: Verbosity::None,
            explored: HashSet::new(),
            frontier: Frontier::new(),
            arena: arena,
        }
    }

    pub fn iterative_search(
        &mut self,
        init_state: P::State,
        max_limit: usize,
    ) -> SearchResult<P::State, P::Action> {
        let mut lim = 1;
        let mut result = SearchResult::new();
        let start = Instant::now();
        let mut n_iter = 0;
        loop {
            if max_limit < lim {
                result.n_iter = n_iter;
                result.total_time = start.elapsed();
                return result;
            }
            let inner_result = self.inner_search(&mut n_iter, init_state.clone(), lim.into());
            if result.max_frontier_size < inner_result.max_frontier_size {
                result.max_frontier_size = inner_result.max_frontier_size
            }
            if inner_result.actions.is_some() {
                return SearchResult::from_inner_result(start, n_iter, inner_result);
            }
            lim += 1
        }
    }

    pub fn search_with_max_depth(
        &mut self,
        init_state: P::State,
        max_depth: usize,
    ) -> SearchResult<P::State, P::Action> {
        let start = Instant::now();
        let mut n_iter = 0;
        let result = self.inner_search(&mut n_iter, init_state, max_depth.into());
        SearchResult::from_inner_result(start, n_iter, result)
    }

    pub fn search(&mut self, init_state: P::State) -> SearchResult<P::State, P::Action> {
        let start = Instant::now();
        let mut n_iter = 0;
        let result = self.inner_search(&mut n_iter, init_state, None);
        SearchResult::from_inner_result(start, n_iter, result)
    }

    fn inner_search(
        &mut self,
        n_iter: &mut usize,
        init_state: P::State,
        lim: Option<usize>,
    ) -> InnerResult<P::State, P::Action> {
        self.frontier.reset();
        self.explored.clear();
        self.frontier.enqueue_or_replace(Node::in_arena(
            None,
            &self.problem,
            init_state,
            None,
            P::Cost::default(),
            self.arena,
        ));

        let result: InnerResult<P::State, P::Action>;

        let mut max_frontier_size = 0;
        while let Some(curr_node) = self.frontier.dequeue() {
            *n_iter += 1;

            let curr_state = curr_node.get_state();

            self.eprint_status(curr_state, curr_node.get_g_cost(), *n_iter);

            if self.problem.is_suitable(&curr_state) {
                result = InnerResult::<P::State, P::Action>::found(
                    curr_node.get_state().clone(),
                    curr_node.get_plan().into(),
                    max_frontier_size,
                );
                return result;
            } else {
                let depth = curr_node.get_depth();
                if lim.map_or(true, |x| x > depth) {
                    for action in self.problem.executable_actions(curr_state) {
                        let (new_state, cost) = self.problem.result(curr_state, &action);
                        if !self.explored.contains(&new_state) {
                            let new_node = Node::in_arena(
                                Some(curr_node),
                                &self.problem,
                                new_state,
                                Some(action),
                                cost,
                                self.arena,
                            );
                            self.frontier.enqueue_or_replace(new_node);
                        }
                    }
                }
            }
            self.explored.insert(curr_state.clone());
            if max_frontier_size < self.frontier.size() {
                max_frontier_size = self.frontier.size();
            }
        }
        result = InnerResult::<P::State, P::Action>::not_found(max_frontier_size);
        return result;
    }

    fn eprint_status(&self, curr_state: &P::State, cost: P::Cost, n_iter: usize) {
        if self.verbosity == Verbosity::Low {
            eprintln!(
                "I: {} cost: {:?} current state:\n{:?}",
                n_iter, cost, curr_state
            );
        }
    }
}

pub struct TreeExplorer<'a, P, Backend>
where
    P: SuitableState + Utility,
    Backend: FrontierBackend<'a, P>,
{
    problem: &'a P,
    frontier: Backend,
    arena: &'a Bump,
}

impl<'a, P, Backend> TreeExplorer<'a, P, Backend>
where
    P: SuitableState + Utility<State: Copy, Action: Clone>,
    Backend: FrontierBackend<'a, P>,
{
    pub fn new(problem: &'a P, arena: &'a Bump) -> Self {
        Self {
            problem: problem,
            frontier: Default::default(),
            arena: arena,
        }
    }

    pub fn iterative_search(
        &mut self,
        init_state: P::State,
        max_limit: usize,
    ) -> SearchResult<P::State, P::Action> {
        let mut lim = 1;
        let mut result = SearchResult::new();
        let start = Instant::now();
        let mut n_iter = 0;
        loop {
            if max_limit < lim {
                result.n_iter = n_iter;
                result.total_time = start.elapsed();
                return result;
            }
            let inner_result = self.inner_search(&mut n_iter, init_state.clone(), lim.into());
            if result.max_frontier_size < inner_result.max_frontier_size {
                result.max_frontier_size = inner_result.max_frontier_size
            }
            if inner_result.actions.is_some() {
                return SearchResult::from_inner_result(start, n_iter, inner_result);
            }
            lim += 1
        }
    }

    pub fn search_with_max_depth(
        &mut self,
        init_state: P::State,
        max_depth: usize,
    ) -> SearchResult<P::State, P::Action> {
        let start = Instant::now();
        let mut n_iter = 0;
        let result = self.inner_search(&mut n_iter, init_state, max_depth.into());
        SearchResult::from_inner_result(start, n_iter, result)
    }

    pub fn search(&mut self, init_state: P::State) -> SearchResult<P::State, P::Action> {
        let start = Instant::now();
        let mut n_iter = 0;
        let result = self.inner_search(&mut n_iter, init_state, None);
        SearchResult::from_inner_result(start, n_iter, result)
    }

    fn inner_search(
        &mut self,
        n_iter: &mut usize,
        init_state: P::State,
        lim: Option<usize>,
    ) -> InnerResult<P::State, P::Action> {
        self.frontier.reset();
        self.frontier.enqueue(Node::in_arena(
            None,
            &self.problem,
            init_state,
            None,
            P::Cost::default(),
            self.arena,
        ));

        //let mut n_iter = 0;
        let result: InnerResult<P::State, P::Action>;

        let mut max_frontier_size = 0;
        while let Some(curr_node) = self.frontier.dequeue() {
            *n_iter += 1;

            let curr_state = curr_node.get_state();

            if self.problem.is_suitable(&curr_state) {
                result = InnerResult::<P::State, P::Action>::found(
                    *curr_node.get_state(),
                    curr_node.get_plan().into(),
                    max_frontier_size,
                );
                return result;
            } else {
                let depth = curr_node.get_depth();
                if lim.map_or(true, |x| x > depth) {
                    for action in self.problem.executable_actions(curr_state) {
                        let (new_state, cost) = self.problem.result(curr_state, &action);
                        let new_node = Node::in_arena(
                            Some(curr_node),
                            &self.problem,
                            new_state,
                            Some(action),
                            cost,
                            self.arena,
                        );
                        self.frontier.enqueue(new_node);
                    }
                }
            }
            if max_frontier_size < self.frontier.size() {
                max_frontier_size = self.frontier.size();
            }
        }
        result = InnerResult::<P::State, P::Action>::not_found(max_frontier_size);
        return result;
    }
}

pub type BFSExplorer<'a, P> = Explorer<'a, P, DequeBackend<'a, P>>;
pub type DFSExplorer<'a, P> = Explorer<'a, P, StackBackend<'a, P>>;
pub type MinCostExplorer<'a, P> = Explorer<'a, P, MinCostBackend<'a, P>>;
pub type BestFirstGreedyExplorer<'a, P> = Explorer<'a, P, BestFirstBackend<'a, P>>;
pub type AStarExplorer<'a, P> = Explorer<'a, P, AStarBackend<'a, P>>;

pub type BFSTreeExplorer<'a, P> = TreeExplorer<'a, P, DequeBackend<'a, P>>;
pub type DFSTreeExplorer<'a, P> = TreeExplorer<'a, P, StackBackend<'a, P>>;
pub type MinTreeCostExplorer<'a, P> = TreeExplorer<'a, P, MinCostBackend<'a, P>>;
pub type BestFirstGreedyTreeExplorer<'a, P> = TreeExplorer<'a, P, BestFirstBackend<'a, P>>;
pub type AStarTreeExplorer<'a, P> = TreeExplorer<'a, P, AStarBackend<'a, P>>;

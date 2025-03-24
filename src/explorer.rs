use core::fmt;
use std::{
    collections::HashSet,
    fmt::Debug,
    hash::Hash,
    time::{Duration, Instant},
};

use crate::{
    explorer_node::Node,
    frontier::{
        AStarBackend, BestFirstBackend, DequeBackend, Frontier, FrontierBackend, MinCostBackend,
        StackBackend,
    },
    problem::*,
};

struct InnerResult<Action>
where
    Action: Clone,
{
    actions: Option<Vec<Action>>,
    max_frontier_size: usize,
}

impl<Action> InnerResult<Action>
where
    Action: Clone,
{
    fn found(actions: Vec<Action>, max_frontier_size: usize) -> Self {
        Self {
            actions: actions.into(),
            max_frontier_size: max_frontier_size,
        }
    }

    fn not_found(max_frontier_size: usize) -> Self {
        Self {
            actions: None,
            max_frontier_size: max_frontier_size,
        }
    }
}

pub struct SearchResult<Action>
where
    Action: Clone,
{
    pub total_time: Duration,
    pub actions: Option<Vec<Action>>,
    pub n_iter: usize,
    pub max_frontier_size: usize,
}

impl<Action> SearchResult<Action>
where
    Action: Clone,
{
    fn new() -> Self {
        Self {
            total_time: Duration::default(),
            actions: None,
            n_iter: 0,
            max_frontier_size: 0,
        }
    }

    fn from_inner_result(start: Instant, n_iter: usize, inner_result: InnerResult<Action>) -> Self {
        Self {
            total_time: start.elapsed(),
            actions: inner_result.actions,
            n_iter: n_iter,
            max_frontier_size: inner_result.max_frontier_size,
        }
    }
}

impl<Action> fmt::Display for SearchResult<Action>
where
    Action: Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "actions: {:?}\ntime: {:?}\niterations: {}\nmax frontier size: {}",
            self.actions, self.total_time, self.n_iter, self.max_frontier_size
        )
    }
}

#[derive(PartialEq, Eq)]
pub enum Verbosity {
    None,
    Low,
}

pub struct Explorer<P, Backend>
where
    P: StateExplorerProblem,
    Backend: FrontierBackend<P> + Debug,
{
    verbosity: Verbosity,
    problem: P,
    explored: HashSet<P::State>,
    frontier: Frontier<P, Backend>,
}

impl<P, Backend> Explorer<P, Backend>
where
    P: StateExplorerProblem<State: Eq + Hash + Clone + Debug, Action: Clone>,
    Backend: FrontierBackend<P> + Debug,
{
    pub fn with_verbosity(problem: P, verbosity: Verbosity) -> Self {
        Self {
            problem: problem,
            verbosity: verbosity,
            explored: HashSet::new(),
            frontier: Frontier::new(),
        }
    }

    pub fn with_low_v(problem: P) -> Self {
        Self {
            problem: problem,
            verbosity: Verbosity::Low,
            explored: HashSet::new(),
            frontier: Frontier::new(),
        }
    }

    pub fn new(problem: P) -> Self {
        Self {
            problem: problem,
            verbosity: Verbosity::None,
            explored: HashSet::new(),
            frontier: Frontier::new(),
        }
    }

    pub fn iterative_search(
        &mut self,
        init_state: P::State,
        max_limit: usize,
    ) -> SearchResult<P::Action> {
        let mut lim = 1;
        let mut result: SearchResult<P::Action> = SearchResult::new();
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
                result.n_iter = n_iter;
                result.total_time = start.elapsed();
                result.actions = inner_result.actions;
                return result;
            }
            lim += 1
        }
    }

    pub fn search_with_max_depth(
        &mut self,
        init_state: P::State,
        max_depth: usize,
    ) -> SearchResult<P::Action> {
        let start = Instant::now();
        let mut n_iter = 0;
        let result = self.inner_search(&mut n_iter, init_state, max_depth.into());
        SearchResult::from_inner_result(start, n_iter, result)
    }

    pub fn search(&mut self, init_state: P::State) -> SearchResult<P::Action> {
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
    ) -> InnerResult<P::Action> {
        self.frontier.reset();
        self.explored.clear();
        self.frontier.enqueue_or_replace(Node::new(
            None,
            &self.problem,
            init_state,
            None,
            P::Cost::default(),
        ));

        //let mut n_iter = 0;
        let result: InnerResult<P::Action>;

        let mut max_frontier_size = 0;
        self.eprint_status(*n_iter);
        while let Some(curr_node) = self.frontier.dequeue() {
            *n_iter += 1;

            let curr_state = curr_node.get_state();

            if self.problem.is_goal(&curr_state) {
                result =
                    InnerResult::<P::Action>::found(curr_node.get_plan().into(), max_frontier_size);
                return result;
            } else {
                let depth = curr_node.get_depth();
                if lim.map_or(true, |x| x > depth) {
                    for action in self.problem.executable_actions(curr_state) {
                        let (new_state, cost) = self.problem.result(curr_state, &action);
                        if !self.explored.contains(&new_state) {
                            let new_node = Node::new(
                                Some(curr_node.clone()),
                                &self.problem,
                                new_state.clone(),
                                Some(action),
                                cost,
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
            self.eprint_status(*n_iter);
        }
        result = InnerResult::<P::Action>::not_found(max_frontier_size);
        return result;
    }

    fn eprint_status(&self, n_iter: usize) {
        if self.verbosity == Verbosity::Low {
            eprintln!("iter: {} Frontier: {:?}", n_iter + 1, self.frontier,);
            eprintln!("iter: {} Explored: {:?}", n_iter + 1, self.explored,)
        }
    }
}

pub type BFSExplorer<P> = Explorer<P, DequeBackend<P>>;
pub type DFSExplorer<P> = Explorer<P, StackBackend<P>>;
pub type MinCostExplorer<P> = Explorer<P, MinCostBackend<P>>;
pub type BestFirstGreedyExplorer<P> = Explorer<P, BestFirstBackend<P>>;
pub type AStarExplorer<P> = Explorer<P, AStarBackend<P>>;

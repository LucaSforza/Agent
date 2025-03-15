use core::fmt;
use std::{
    collections::HashSet,
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::{
    agent::{Node, WorldState},
    frontier::{
        DequeBackend, Frontier, FrontierBackend, MinFBackend, MinGBackend, MinHBackend,
        StackBackend,
    },
};

struct InnerResult<Action>
where
    Action: Clone,
{
    actions: Option<Vec<Action>>,
    n_iter: usize,
    max_frontier_size: usize,
}

impl<Action> InnerResult<Action>
where
    Action: Clone,
{
    fn found(actions: Vec<Action>, n_iter: usize, max_frontier_size: usize) -> Self {
        Self {
            actions: actions.into(),
            n_iter: n_iter,
            max_frontier_size: max_frontier_size,
        }
    }

    fn not_found(n_iter: usize, max_frontier_size: usize) -> Self {
        Self {
            actions: None,
            n_iter: n_iter,
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

    fn from_inner_result(start: Instant, inner_result: InnerResult<Action>) -> Self {
        Self {
            total_time: start.elapsed(),
            actions: inner_result.actions,
            n_iter: inner_result.n_iter,
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

// TODO: add max depth for the nodes
pub struct Explorer<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action>,
{
    explored: HashSet<State>,
    frontier: Frontier<State, Action, Backend>,
}

impl<State, Action, Backend> Explorer<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action>,
{
    pub fn new() -> Self {
        Self {
            explored: HashSet::new(),
            frontier: Frontier::new(),
        }
    }

    pub fn iterative_search(
        &mut self,
        init_state: State,
        max_limit: usize,
    ) -> SearchResult<Action> {
        let mut lim = 1;
        let mut result: SearchResult<Action> = SearchResult::new();
        let start = Instant::now();
        loop {
            if max_limit < lim {
                result.total_time = start.elapsed();
                return result;
            }
            let inner_result = self.inner_search(init_state.clone(), lim.into());
            result.n_iter += inner_result.n_iter;
            if result.max_frontier_size < inner_result.max_frontier_size {
                result.max_frontier_size = inner_result.max_frontier_size
            }
            if inner_result.actions.is_some() {
                result.total_time = start.elapsed();
                result.actions = inner_result.actions;
                return result;
            }
            lim += 1
        }
    }

    pub fn search_with_max_depth(
        &mut self,
        init_state: State,
        max_depth: usize,
    ) -> SearchResult<Action> {
        let start = Instant::now();
        let result = self.inner_search(init_state, max_depth.into());
        SearchResult::from_inner_result(start, result)
    }

    pub fn search(&mut self, init_state: State) -> SearchResult<Action> {
        let start = Instant::now();
        let result = self.inner_search(init_state, None);
        SearchResult::from_inner_result(start, result)
    }

    fn inner_search(&mut self, init_state: State, lim: Option<usize>) -> InnerResult<Action> {
        self.frontier.reset();
        self.explored.clear();
        self.frontier
            .enqueue_or_replace(Node::new(None, init_state, None, 0.0));

        let mut n_iter = 0;
        let result: InnerResult<Action>;

        let mut max_frontier_size = 0;

        while let Some(curr_node) = self.frontier.dequeue() {
            n_iter += 1;

            let curr_state = curr_node.get_state();
            if curr_state.is_goal() {
                result = InnerResult::<Action>::found(
                    curr_node.get_plan().into(),
                    n_iter,
                    max_frontier_size,
                );
                return result;
            } else {
                let depth = curr_node.get_depth();
                if lim.map_or(true, |x| x > depth) {
                    for action in curr_state.executable_actions() {
                        let (new_state, cost) = curr_state.result(&action);
                        if !self.explored.contains(&new_state) {
                            let new_node = Node::new(
                                Some(curr_node.clone()),
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
        }
        result = InnerResult::<Action>::not_found(n_iter, max_frontier_size);
        return result;
    }
}

pub type BFSExplorer<State, Action> = Explorer<State, Action, DequeBackend<State, Action>>;
pub type DFSExplorer<State, Action> = Explorer<State, Action, StackBackend<State, Action>>;
pub type MinCostExplorer<State, Action> = Explorer<State, Action, MinGBackend<State, Action>>;
pub type BestFirstGreedyExplorer<State, Action> =
    Explorer<State, Action, MinHBackend<State, Action>>;
pub type AStarExplore<State, Action> = Explorer<State, Action, MinFBackend<State, Action>>;

use core::fmt;
use std::{
    collections::{HashMap, HashSet},
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
            max_frontier_size,
        }
    }

    fn not_found(max_frontier_size: usize) -> Self {
        Self {
            state: None,
            actions: None,
            max_frontier_size,
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
            n_iter,
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

/// Graph-search strategy for [`bounded_search`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphSearchAlgorithm {
    AStar,
    UniformCost,
    BreadthFirst,
}

/// Result of a bounded graph search. `expanded` counts states removed from the
/// frontier and checked, including a state that satisfies the goal.
#[derive(Debug, PartialEq, Eq)]
pub enum GraphSearchOutcome<State, Action> {
    Solved {
        state: State,
        actions: Vec<Action>,
        expanded: usize,
    },
    Exhausted {
        expanded: usize,
    },
    LimitReached {
        expanded: usize,
    },
}

/// Searches a finite graph while owning the arena used for search nodes.
///
/// The state budget includes the initial state. A zero budget returns
/// `LimitReached` without evaluating the initial state.
/// A* and uniform-cost search require nonnegative action costs; A* also relies
/// on an admissible heuristic for optimality. A* breaks priority ties by lower
/// h-cost and then FIFO insertion order. Breadth-first search is intended for
/// uniform-cost edges.
pub fn bounded_search<P>(
    problem: &P,
    init_state: P::State,
    algorithm: GraphSearchAlgorithm,
    max_states: usize,
) -> GraphSearchOutcome<P::State, P::Action>
where
    P: SuitableState + Utility<State: Eq + Hash + Clone, Action: Clone>,
{
    if max_states == 0 {
        return GraphSearchOutcome::LimitReached { expanded: 0 };
    }

    let arena = Bump::new();
    match algorithm {
        GraphSearchAlgorithm::AStar => bounded_search_with_backend::<P, AStarBackend<'_, P>>(
            problem, init_state, max_states, &arena,
        ),
        GraphSearchAlgorithm::UniformCost => {
            bounded_search_with_backend::<P, MinCostBackend<'_, P>>(
                problem, init_state, max_states, &arena,
            )
        }
        GraphSearchAlgorithm::BreadthFirst => {
            bounded_search_with_backend::<P, DequeBackend<'_, P>>(
                problem, init_state, max_states, &arena,
            )
        }
    }
}

fn bounded_search_with_backend<'a, P, Backend>(
    problem: &'a P,
    init_state: P::State,
    max_states: usize,
    arena: &'a Bump,
) -> GraphSearchOutcome<P::State, P::Action>
where
    P: SuitableState + Utility<State: Eq + Hash + Clone, Action: Clone>,
    Backend: FrontierBackend<'a, P>,
{
    let mut frontier: Frontier<'a, P, Backend> = Frontier::new();
    let mut best_cost = HashMap::new();
    let initial = Node::in_arena(None, problem, init_state, None, P::Cost::default(), arena);
    best_cost.insert(initial.get_state().clone(), initial.get_g_cost());
    frontier.enqueue_or_replace(initial);
    let mut expanded = 0;

    while let Some(node) = frontier.dequeue() {
        if best_cost
            .get(node.get_state())
            .is_some_and(|cost| *cost < node.get_g_cost())
        {
            continue;
        }

        expanded += 1;
        if problem.is_suitable(node.get_state()) {
            return GraphSearchOutcome::Solved {
                state: node.get_state().clone(),
                actions: node.get_plan(),
                expanded,
            };
        }

        for action in problem.executable_actions(node.get_state()) {
            let (next_state, step_cost) = problem.result(node.get_state(), &action);
            let next_cost = node.get_g_cost() + step_cost;
            if best_cost
                .get(&next_state)
                .is_some_and(|known_cost| *known_cost <= next_cost)
            {
                continue;
            }
            if !best_cost.contains_key(&next_state) && best_cost.len() == max_states {
                return GraphSearchOutcome::LimitReached { expanded };
            }

            let next = Node::in_arena(
                Some(node),
                problem,
                next_state.clone(),
                Some(action),
                step_cost,
                arena,
            );
            best_cost.insert(next_state, next_cost);
            frontier.enqueue_or_replace(next);
        }
    }

    GraphSearchOutcome::Exhausted { expanded }
}

pub struct Explorer<'a, P, Backend>
where
    P: Utility + SuitableState, // TODO: generalize more
    Backend: FrontierBackend<'a, P> + Debug,
{
    verbosity: Verbosity,
    problem: &'a P,
    explored: HashSet<P::State>,
    best_cost: HashMap<P::State, P::Cost>,
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
            problem,
            verbosity,
            explored: HashSet::new(),
            best_cost: HashMap::new(),
            frontier: Frontier::new(),
            arena,
        }
    }

    pub fn with_low_v(problem: &'a P, arena: &'a Bump) -> Self {
        Self {
            problem,
            verbosity: Verbosity::Low,
            explored: HashSet::new(),
            best_cost: HashMap::new(),
            frontier: Frontier::new(),
            arena,
        }
    }

    pub fn new(problem: &'a P, arena: &'a Bump) -> Self {
        Self {
            problem,
            verbosity: Verbosity::None,
            explored: HashSet::new(),
            best_cost: HashMap::new(),
            frontier: Frontier::new(),
            arena,
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
        self.best_cost.clear();
        let initial_node: &'a Node<'a, P> = Node::in_arena(
            None,
            self.problem,
            init_state,
            None,
            P::Cost::default(),
            self.arena,
        );
        self.best_cost
            .insert(initial_node.get_state().clone(), initial_node.get_g_cost());
        self.frontier.enqueue_or_replace(initial_node);

        let result: InnerResult<P::State, P::Action>;

        let mut max_frontier_size = 0;
        while let Some(curr_node) = self.frontier.dequeue() {
            if Backend::REOPENS_IMPROVED_STATES
                && self
                    .best_cost
                    .get(curr_node.get_state())
                    .is_some_and(|cost| *cost < curr_node.get_g_cost())
            {
                continue;
            }
            *n_iter += 1;

            let curr_state = curr_node.get_state();

            self.eprint_status(curr_state, curr_node.get_g_cost(), *n_iter);

            if self.problem.is_suitable(curr_state) {
                result = InnerResult::<P::State, P::Action>::found(
                    curr_node.get_state().clone(),
                    curr_node.get_plan(),
                    max_frontier_size,
                );
                return result;
            } else {
                let depth = curr_node.get_depth();
                if lim.is_none_or(|x| x > depth) {
                    for action in self.problem.executable_actions(curr_state) {
                        let (new_state, cost) = self.problem.result(curr_state, &action);
                        let new_cost = curr_node.get_g_cost() + cost;
                        if Backend::REOPENS_IMPROVED_STATES {
                            if self
                                .best_cost
                                .get(&new_state)
                                .is_some_and(|best| *best <= new_cost)
                            {
                                continue;
                            }
                        } else if self.explored.contains(&new_state) {
                            continue;
                        }
                        let new_node = Node::in_arena(
                            Some(curr_node),
                            self.problem,
                            new_state.clone(),
                            Some(action),
                            cost,
                            self.arena,
                        );
                        if Backend::REOPENS_IMPROVED_STATES {
                            self.best_cost.insert(new_state, new_cost);
                        }
                        self.frontier.enqueue_or_replace(new_node);
                    }
                }
            }
            if !Backend::REOPENS_IMPROVED_STATES {
                self.explored.insert(curr_state.clone());
            }
            if max_frontier_size < self.frontier.size() {
                max_frontier_size = self.frontier.size();
            }
        }
        result = InnerResult::<P::State, P::Action>::not_found(max_frontier_size);
        result
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
            problem,
            frontier: Default::default(),
            arena,
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
            let inner_result = self.inner_search(&mut n_iter, init_state, lim.into());
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
            self.problem,
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

            if self.problem.is_suitable(curr_state) {
                result = InnerResult::<P::State, P::Action>::found(
                    *curr_node.get_state(),
                    curr_node.get_plan(),
                    max_frontier_size,
                );
                return result;
            } else {
                let depth = curr_node.get_depth();
                if lim.is_none_or(|x| x > depth) {
                    for action in self.problem.executable_actions(curr_state) {
                        let (new_state, cost) = self.problem.result(curr_state, &action);
                        let new_node = Node::in_arena(
                            Some(curr_node),
                            self.problem,
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
        result
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

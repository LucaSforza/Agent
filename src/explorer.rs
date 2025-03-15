use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::{
    agent::{Node, WorldState},
    frontier::{DequeBackend, Frontier, FrontierBackend, MinGBackend, StackBackend},
};

pub struct SearchResult<Action>
where
    Action: Clone,
{
    pub total_time: Duration,
    pub actions: Option<Vec<Action>>,
    pub n_iter: usize,
}

impl<Action> SearchResult<Action>
where
    Action: Clone,
{
    pub fn found(start_time: Instant, actions: Vec<Action>, n_iter: usize) -> Self {
        Self {
            total_time: start_time.elapsed(),
            actions: actions.into(),
            n_iter: n_iter,
        }
    }

    pub fn not_found(start_time: Instant, n_iter: usize) -> Self {
        Self {
            total_time: start_time.elapsed(),
            actions: None,
            n_iter: n_iter,
        }
    }
}

// TODO: add max depth for the nodes
pub struct Explorer<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action>,
{
    _action: std::marker::PhantomData<Action>,
    _state: std::marker::PhantomData<State>,
    _backend: std::marker::PhantomData<Backend>,
}

impl<State, Action, Backend> Explorer<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action>,
{
    pub fn new() -> Self {
        Self {
            _action: std::marker::PhantomData,
            _state: std::marker::PhantomData,
            _backend: std::marker::PhantomData,
        }
    }

    pub fn iterative_search(&self, init_state: State) -> SearchResult<Action> {
        let mut lim = 0;
        let mut result;
        loop {
            result = self.inner_search(init_state.clone(), lim.into());
            if result.actions.is_some() {
                return result;
            } // TODO: quando capire che non esiste soluzione
            lim += 1
        }
    }

    pub fn search(&self, init_state: State) -> SearchResult<Action> {
        self.inner_search(init_state, None)
    }

    fn inner_search(&self, init_state: State, lim: Option<usize>) -> SearchResult<Action> {
        let mut frontier = Frontier::<State, Action, Backend>::new();
        let mut explored = HashSet::new();
        frontier.enqueue_or_replace(Node::new(None, init_state, None, 0.0));

        let mut n_iter = 0;
        let result: SearchResult<Action>;

        let start = Instant::now();

        while let Some(curr_node) = frontier.dequeue() {
            if lim.map_or(false, |x| x >= n_iter) {
                result = SearchResult::<Action>::not_found(start, n_iter);
                return result;
            }
            n_iter += 1;

            let curr_state = curr_node.get_state();
            if curr_state.is_goal() {
                result = SearchResult::<Action>::found(start, curr_node.get_plan().into(), n_iter);
                return result;
            } else {
                for action in curr_state.executable_actions() {
                    let (new_state, cost) = curr_state.result(&action);
                    if !explored.contains(&new_state) {
                        let new_node = Node::new(
                            Some(curr_node.clone()),
                            new_state.clone(),
                            Some(action),
                            cost,
                        );
                        frontier.enqueue_or_replace(new_node);
                    }
                }
            }
            explored.insert(curr_state.clone());
        }
        result = SearchResult::<Action>::not_found(start, n_iter);
        return result;
    }
}

pub type BFSExplorer<State, Action> = Explorer<State, Action, DequeBackend<State, Action>>;
pub type DFSExplorer<State, Action> = Explorer<State, Action, StackBackend<State, Action>>;
pub type MinCostExplorer<State, Action> = Explorer<State, Action, MinGBackend<State, Action>>;

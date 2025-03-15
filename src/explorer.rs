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

    pub fn iterative_search(&mut self, init_state: State) -> SearchResult<Action> {
        let mut lim = 0;
        let mut result;
        loop {
            result = self.inner_search(init_state.clone(), lim.into());
            if result.actions.is_some() {
                return result;
            } // TODO: quando capire che non esiste soluzione
              // TODO: salvarsi i tempi per poi sommarli
            lim += 1
        }
    }

    pub fn search(&mut self, init_state: State) -> SearchResult<Action> {
        self.inner_search(init_state, None)
    }

    fn inner_search(&mut self, init_state: State, lim: Option<usize>) -> SearchResult<Action> {
        self.frontier.reset();
        self.explored.clear();
        self.frontier
            .enqueue_or_replace(Node::new(None, init_state, None, 0.0));

        let mut n_iter = 0;
        let result: SearchResult<Action>;

        let start = Instant::now();

        while let Some(curr_node) = self.frontier.dequeue() {
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
            self.explored.insert(curr_state.clone());
        }
        result = SearchResult::<Action>::not_found(start, n_iter);
        return result;
    }
}

pub type BFSExplorer<State, Action> = Explorer<State, Action, DequeBackend<State, Action>>;
pub type DFSExplorer<State, Action> = Explorer<State, Action, StackBackend<State, Action>>;
pub type MinCostExplorer<State, Action> = Explorer<State, Action, MinGBackend<State, Action>>;

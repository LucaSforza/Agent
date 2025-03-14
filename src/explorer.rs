use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::{
    agent::{Node, WorldState},
    frontier::{DequeFrontier, Frontier, StackFrontier},
};

pub struct SearchResult<Action>
where
    Action: Clone,
{
    pub total_time: Duration,
    pub actions: Option<Vec<Action>>,
    pub n_iter: u64,
}

impl<Action> SearchResult<Action>
where
    Action: Clone,
{
    pub fn found(start_time: Instant, actions: Vec<Action>, n_iter: u64) -> Self {
        Self {
            total_time: start_time.elapsed(),
            actions: actions.into(),
            n_iter: n_iter,
        }
    }

    pub fn not_found(start_time: Instant, n_iter: u64) -> Self {
        Self {
            total_time: start_time.elapsed(),
            actions: None,
            n_iter: n_iter,
        }
    }
}

pub struct Explorer<State, Action, Front>
where
    State: WorldState<Action>,
    Action: Clone,
    Front: Frontier<State, Action>,
{
    max_depth: Option<u64>,
    _action: std::marker::PhantomData<Action>,
    _state: std::marker::PhantomData<State>,
    _front: std::marker::PhantomData<Front>,
}

impl<State, Action, Front> Explorer<State, Action, Front>
where
    State: WorldState<Action>,
    Action: Clone,
    Front: Frontier<State, Action>,
{
    pub fn new() -> Self {
        Self {
            max_depth: None,
            _action: std::marker::PhantomData,
            _state: std::marker::PhantomData,
            _front: std::marker::PhantomData,
        }
    }

    pub fn with_max_depth(max_depth: u64) -> Self {
        Self {
            max_depth: max_depth.into(),
            _action: std::marker::PhantomData,
            _state: std::marker::PhantomData,
            _front: std::marker::PhantomData,
        }
    }

    pub fn search(self, init_state: State) -> SearchResult<Action> {
        let mut frontier = Front::new();
        let mut explored = HashSet::new();
        frontier.enqueue_or_replace(Node::new(None, init_state, None, 0.0));

        let mut n_iter = 0;
        let result: SearchResult<Action>;

        let start = Instant::now();

        while let Some(curr_node) = frontier.dequeue() {
            if self.max_depth.map_or(false, |x| x >= n_iter) {
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

pub type BFSExplorer<State, Action> = Explorer<State, Action, DequeFrontier<State, Action>>;
pub type DFSExplorer<State, Action> = Explorer<State, Action, StackFrontier<State, Action>>;

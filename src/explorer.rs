use std::{collections::HashSet, rc::Rc, time::Instant};

use crate::{
    agent::{Node, WorldState},
    frontier::Frontier,
};

pub struct Explorer<State, Action, Front>
where
    State: WorldState<Action>,
    Action: Clone,
    Front: Frontier<State, Action>,
{
    max_depth: Option<usize>,
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

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_depth: max_depth.into(),
            _action: std::marker::PhantomData,
            _state: std::marker::PhantomData,
            _front: std::marker::PhantomData,
        }
    }

    pub fn search(self, init_state: State) -> Option<Vec<Action>>
    where
        State: WorldState<Action>,
        Action: Clone + Default,
        Front: Frontier<State, Action>,
    {
        let mut frontier = Front::new();
        let mut explored = HashSet::new();
        frontier.enqueue(Rc::new(Node::new(None, init_state, Action::default(), 0.0)));
        while let Some(curr_node) = frontier.dequeue() {
            let curr_state = curr_node.get_state();
            if curr_state.is_goal() {
                return Some(curr_node.get_plan());
            } else {
                for action in curr_state.executable_actions() {
                    let (new_state, cost) = curr_state.result(&action);
                    if !explored.iter().any(|x| *x == new_state) {
                        let new_node = Rc::new(Node::new(
                            Some(curr_node.clone()),
                            new_state.clone(),
                            action,
                            cost,
                        )); // TODO: cambiare costo
                        let mut found = false;
                        for existing_node in frontier.mut_elements() {
                            if *existing_node.get_state() == new_state {
                                found = true;
                                if new_node.get_total_cost() < existing_node.get_total_cost() {
                                    *existing_node = new_node.clone(); // Aggiornamento sicuro
                                }
                                break; // Uscita dal loop per evitare iterazioni inutili
                            }
                        }

                        if !found {
                            frontier.enqueue(new_node);
                        }
                    }
                }
            }
            explored.insert(curr_state.clone());
        }
        return None;
    }
}

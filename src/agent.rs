use std::cell::RefCell;
use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::vec::Vec;

pub trait WorldState<Action>: Clone + Eq + Hash + Debug {
    type Iter: Iterator<Item = Action>;

    fn executable_actions(&self) -> Self::Iter;
    fn result(&self, action: &Action) -> (Self, f64);
    fn is_goal(&self) -> bool;
    fn heuristic(&self) -> f64;
}

use ordered_float::OrderedFloat;

#[derive(PartialEq, Eq)]
pub struct Node<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    state: State,
    parent: Option<Rc<Node<State, Action>>>,
    action: Option<Action>,
    total_cost: OrderedFloat<f64>,
    heuristic: OrderedFloat<f64>,
    depth: usize,
    dead: RefCell<bool>,
}

impl<State, Action> Node<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    pub fn new(
        parent: Option<Rc<Node<State, Action>>>,
        state: State,
        action: Option<Action>,
        cost: f64,
    ) -> Self {
        assert!((parent.is_none() && action.is_none()) || (parent.is_some() && action.is_some()));
        let mut total_cost: OrderedFloat<f64> = cost.into();
        let mut depth = 0;
        if let Some(parent_node) = parent.as_ref() {
            total_cost += parent_node.total_cost;
            depth = parent_node.depth + 1;
        }
        let h = state.heuristic();
        Self {
            state: state,
            parent: parent,
            action: action,
            total_cost: total_cost,
            depth: depth,
            heuristic: h.into(),
            dead: false.into(),
        }
    }

    pub fn get_plan(&self) -> Vec<Action> {
        assert!(!self.is_dead());
        let mut result: Vec<Action> = Vec::with_capacity(self.depth);

        if self.action.is_some() {
            result.push(self.action.clone().unwrap());
            let mut current_node: Option<Rc<Node<State, Action>>> = self.parent.clone();

            while let Some(node) = current_node {
                if let Some(action) = &node.action {
                    result.push(action.clone());
                }
                current_node = node.parent.clone();
            }
        }
        result.reverse();
        result
    }

    pub fn get_g_cost(&self) -> OrderedFloat<f64> {
        return self.total_cost;
    }

    pub fn get_h_cost(&self) -> OrderedFloat<f64> {
        return self.heuristic;
    }

    pub fn get_f_cost(&self) -> OrderedFloat<f64> {
        return self.total_cost + self.heuristic;
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn mark_dead(&self) {
        *self.dead.borrow_mut() = true;
    }

    pub fn is_dead(&self) -> bool {
        *self.dead.borrow()
    }
}

impl<State, Action> Debug for Node<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("state", &self.state)
            .field("total_cost", &self.total_cost)
            .field("heuristic", &self.heuristic)
            .field("depth", &self.depth)
            .finish()
    }
}

impl<State, Action> std::hash::Hash for Node<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state);
        self.total_cost.hash(state);
        self.heuristic.hash(state);
        self.depth.hash(state);
    }
}

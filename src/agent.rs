use std::clone::Clone;
use std::hash::Hash;
use std::rc::Rc;
use std::vec::Vec;

pub trait WorldState<Action>: Clone + Eq + Hash {
    type Iter: Iterator<Item = Action>;

    fn executable_actions(&self) -> Self::Iter;
    fn result(&self, action: &Action) -> (Self, f64);
    fn is_goal(&self) -> bool;
}

use ordered_float::OrderedFloat;

#[derive(Hash, PartialEq, Eq)]
pub struct Node<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    state: State,
    parent: Option<Rc<Node<State, Action>>>,
    action: Option<Action>,
    total_cost: OrderedFloat<f64>,
    depth: usize,
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
        Node {
            state: state,
            parent: parent,
            action: action,
            total_cost: total_cost,
            depth: depth,
        }
    }

    pub fn get_plan(&self) -> Vec<Action> {
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

    pub fn get_total_cost(&self) -> OrderedFloat<f64> {
        return self.total_cost;
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}

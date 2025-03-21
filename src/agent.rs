use std::cell::RefCell;
use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::vec::Vec;

pub trait Problem {
    type State;
    type Action;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action>;
    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, f64);
    fn heuristic(&self, state: &Self::State) -> f64;
}

pub trait StateExplorerProblem: Problem {
    fn is_goal(&self, state: &Self::State) -> bool;
}

use ordered_float::OrderedFloat;

#[derive(PartialEq, Eq)]
pub struct Node<P>
where
    P: StateExplorerProblem,
{
    state: P::State,
    parent: Option<Rc<Node<P>>>,
    action: Option<P::Action>,
    total_cost: OrderedFloat<f64>,
    heuristic: OrderedFloat<f64>,
    depth: usize,
    dead: RefCell<bool>,
}

impl<P> Node<P>
where
    P: StateExplorerProblem<Action: Clone>,
{
    pub fn new(
        parent: Option<Rc<Node<P>>>,
        problem: &P,
        state: P::State,
        action: Option<P::Action>,
        cost: f64,
    ) -> Self {
        assert!((parent.is_none() && action.is_none()) || (parent.is_some() && action.is_some()));
        let mut total_cost: OrderedFloat<f64> = cost.into();
        let mut depth = 0;
        if let Some(parent_node) = parent.as_ref() {
            total_cost += parent_node.total_cost;
            depth = parent_node.depth + 1;
        }
        let h = problem.heuristic(&state);
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

    pub fn get_plan(&self) -> Vec<P::Action> {
        assert!(!self.is_dead());
        let mut result: Vec<P::Action> = Vec::with_capacity(self.depth);

        if self.action.is_some() {
            result.push(self.action.clone().unwrap());
            let mut current_node: Option<Rc<Node<P>>> = self.parent.clone();

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

    pub fn get_state(&self) -> &P::State {
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

impl<P> Debug for Node<P>
where
    P: StateExplorerProblem<State: Debug, Action: Clone>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ s: {:?}, g: {}, h:{}, f:{}}}",
            self.state,
            self.total_cost,
            self.heuristic,
            self.get_f_cost()
        )

        /* f.debug_struct("Node")
        .field("state", &self.state)
        .field("total_cost", &self.total_cost)
        .field("heuristic", &self.heuristic)
        .field("depth", &self.depth)
        .finish() */
    }
}

impl<P> std::hash::Hash for Node<P>
where
    P: StateExplorerProblem<State: Hash>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state);
        self.total_cost.hash(state);
        self.heuristic.hash(state);
        self.depth.hash(state);
    }
}

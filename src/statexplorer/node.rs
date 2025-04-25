use std::cell::RefCell;
use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;
use std::vec::Vec;

use bumpalo::Bump;

use crate::problem::*;

#[derive(PartialEq, Eq)]
pub struct Node<'a, P>
where
    P: Utility,
{
    state: P::State,
    parent: Option<&'a Self>,
    action: Option<P::Action>,
    total_cost: P::Cost,
    heuristic: P::Cost,
    depth: usize,
    dead: RefCell<bool>,
}

impl<'a, P> Node<'a, P>
where
    P: Utility<Action: Clone, Cost: Add<Output = P::Cost>>,
{
    pub fn in_arena(
        parent: Option<&'a Node<'a, P>>,
        problem: &P,
        state: P::State,
        action: Option<P::Action>,
        cost: P::Cost,
        arena: &'a Bump,
    ) -> &'a Self {
        arena.alloc(Self::new(parent, problem, state, action, cost))
    }
    pub fn new(
        parent: Option<&'a Node<'a, P>>,
        problem: &P,
        state: P::State,
        action: Option<P::Action>,
        cost: P::Cost,
    ) -> Self {
        assert!((parent.is_none() && action.is_none()) || (parent.is_some() && action.is_some()));
        let mut total_cost = cost;
        let mut depth = 0;
        if let Some(parent_node) = parent.as_ref() {
            total_cost = total_cost + parent_node.total_cost;
            depth = parent_node.depth + 1;
        }
        let h = problem.heuristic(&state);
        Self {
            state: state,
            parent: parent,
            action: action,
            total_cost: total_cost,
            depth: depth,
            heuristic: h,
            dead: false.into(),
        }
    }

    pub fn get_plan(&self) -> Vec<P::Action> {
        assert!(!self.is_dead());
        let mut result: Vec<P::Action> = Vec::with_capacity(self.depth);

        if self.action.is_some() {
            result.push(self.action.clone().unwrap());
            let mut current_node = self.parent;

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

    pub fn get_g_cost(&self) -> P::Cost {
        return self.total_cost;
    }

    pub fn get_h_cost(&self) -> P::Cost {
        return self.heuristic;
    }

    pub fn get_f_cost(&self) -> P::Cost {
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

impl<P> Debug for Node<'_, P>
where
    P: Utility<State: Debug, Action: Clone, Cost: Debug + Add<Output = P::Cost>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ s: {:?}, g: {:?}, h:{:?}, f:{:?}}}",
            self.state,
            self.total_cost,
            self.heuristic,
            self.get_f_cost()
        )
    }
}

impl<P> std::hash::Hash for Node<'_, P>
where
    P: Utility<State: Hash, Cost: Hash>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state);
        self.total_cost.hash(state);
        self.heuristic.hash(state);
        self.depth.hash(state);
    }
}

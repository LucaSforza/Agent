use std::clone::Clone;
use std::hash::Hash;
use std::rc::Rc;
use std::vec::Vec;

pub trait WorldState<Action>: Clone + Eq + Hash {
    type Iter: Iterator<Item = Action>;

    fn executable_actions(&self) -> Self::Iter;
    fn result(&self, action: &Action) -> (Self, f32);
    fn is_goal(&self) -> bool;
}

pub struct Node<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    state: State,
    parent: Option<Rc<Node<State, Action>>>,
    action: Action,
    total_cost: f32,
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
        action: Action,
        cost: f32,
    ) -> Self {
        let mut total_cost = cost;
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
        result.push(self.action.clone());
        let mut parent: Option<Rc<Node<State, Action>>> = self.parent.clone();
        while let Some(node) = parent.as_ref() {
            let new_node = node.clone();
            parent = new_node.parent.clone();
            if new_node.parent.is_some() {
                result.push(new_node.action.clone());
            }
        }
        return result;
    }

    pub fn get_total_cost(&self) -> f32 {
        return self.total_cost;
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}

/*
pub struct Strategy<State, Action, Target>
where
    State: WorldState<Action>,
    Action: Clone,
    Front: Frontier<State, Action>,
{
    init_state: State,
    goal: Target,
}

impl<State, Action, Front, Target> Strategy<State, Action, Front, Target>
where
    State: WorldState<Action>,
    Action: Clone,
    Front: Frontier<State, Action>,
    Target: Goal<State>,
{
    pub fn new(init_state: State, goal: Target) -> Self {
        Self {
            init_state: init_state,
            goal: goal,
        }
    }

    pub fn search(&mut self)
} */

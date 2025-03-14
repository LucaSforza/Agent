use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::agent::{Node, WorldState};

pub trait FrontierBackend<State, Action>: Default
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn enqueue(&mut self, item: Rc<Node<State, Action>>);
    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>>;
    fn delete(&mut self, state: &State) -> bool;
}

pub struct Frontier<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action>,
{
    collection: Backend,
    get_node: HashMap<State, Rc<Node<State, Action>>>,
}

impl<State, Action, Backend> Frontier<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action>,
{
    pub fn new() -> Self {
        Self {
            collection: Backend::default(),
            get_node: HashMap::new(),
        }
    }

    // TODO: change bool into an enum
    // TODO: change if the cost is less than the actual node
    pub fn enqueue_or_replace(&mut self, item: Node<State, Action>) -> bool {
        let mut to_remove = None;
        if let Some(old_node) = self.get_node.get(item.get_state()) {
            if old_node.get_total_cost() > item.get_total_cost() {
                to_remove = old_node.get_state().clone().into();
            } else {
                return false;
            }
        }

        if let Some(to_remove) = to_remove {
            self.collection.delete(&to_remove);
            self.get_node.remove(&to_remove);
        }

        let state = item.get_state().clone();
        let to_insert = Rc::new(item);
        assert!(self.get_node.insert(state, to_insert.clone()).is_none());
        self.collection.enqueue(to_insert);

        true
    }

    pub fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        let result = self.collection.dequeue();
        if result.is_some() {
            let node = result.clone().unwrap();
            self.get_node.remove(node.get_state());
        }
        result
    }
}

pub type DequeBackend<State, Action> = VecDeque<Rc<Node<State, Action>>>;

impl<State, Action> FrontierBackend<State, Action> for DequeBackend<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        self.pop_front()
    }

    fn enqueue(&mut self, item: Rc<Node<State, Action>>) {
        self.push_back(item);
    }

    fn delete(&mut self, state: &State) -> bool {
        let mut index = None;
        for (i, node) in self.iter().enumerate() {
            if node.get_state() == state {
                index = i.into()
            }
        }
        if let Some(i) = index {
            self.remove(i);
            return true;
        }
        return false;
    }
}

pub type StackBackend<State, Action> = Vec<Rc<Node<State, Action>>>;

impl<State, Action> FrontierBackend<State, Action> for StackBackend<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn enqueue(&mut self, item: Rc<Node<State, Action>>) {
        self.push(item);
    }

    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        self.pop()
    }

    fn delete(&mut self, state: &State) -> bool {
        let mut index = None;
        for (i, node) in self.iter().enumerate() {
            if node.get_state() == state {
                index = i.into()
            }
        }
        if let Some(i) = index {
            self.remove(i);
            return true;
        }
        return false;
    }
}

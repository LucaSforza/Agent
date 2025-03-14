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
        if self.get_node.contains_key(item.get_state()) {
            return false;
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
}

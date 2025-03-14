use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::agent::{Node, WorldState};

pub trait Frontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn new() -> Self;
    fn enqueue_or_replace(&mut self, item: Node<State, Action>) -> bool; // TODO: change bool into an enum
    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>>;
}

pub struct DequeFrontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    collection: VecDeque<Rc<Node<State, Action>>>,
    get_node: HashMap<State, Rc<Node<State, Action>>>,
}

impl<State, Action> Frontier<State, Action> for DequeFrontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn new() -> Self {
        Self {
            collection: VecDeque::new(),
            get_node: HashMap::new(),
        }
    }

    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        let result = self.collection.pop_front();
        if result.is_some() {
            let node = result.clone().unwrap();
            assert!(self.get_node.remove(node.get_state()).is_some());
        }
        result
    }

    fn enqueue_or_replace(&mut self, item: Node<State, Action>) -> bool {
        let mut to_swap = false;
        if let Some(old_node) = self.get_node.get(item.get_state()) {
            if old_node.get_total_cost() < item.get_total_cost() {
                to_swap = true
            } else {
                return false; // se il costo è più alto allora ignora item
            }
        }

        if to_swap {
            self.get_node.remove(item.get_state());
            let mut index = None;
            for (i, n) in self.collection.iter().enumerate() {
                if n.get_state() == item.get_state() {
                    index = i.into();
                }
            }
            self.collection.remove(index.unwrap());
        }
        let state = item.get_state().clone();
        let to_insert = Rc::new(item);
        assert!(self.get_node.insert(state, to_insert.clone()).is_none());
        self.collection.push_back(to_insert);

        true
    }
}

pub struct StackFrontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    collection: Vec<Rc<Node<State, Action>>>,
    get_node: HashMap<State, Rc<Node<State, Action>>>,
}

impl<State, Action> Frontier<State, Action> for StackFrontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn new() -> Self {
        Self {
            collection: Vec::new(),
            get_node: HashMap::new(),
        }
    }

    fn enqueue_or_replace(&mut self, item: Node<State, Action>) -> bool {
        let mut to_swap = false;
        if let Some(old_node) = self.get_node.get(item.get_state()) {
            if old_node.get_total_cost() < item.get_total_cost() {
                to_swap = true
            } else {
                return false; // se il costo è più alto allora ignora item
            }
        }

        if to_swap {
            self.get_node.remove(item.get_state());
        }
        let state = item.get_state().clone();
        let to_insert = Rc::new(item);
        self.get_node.insert(state, to_insert.clone());
        self.collection.push(to_insert);

        return true;
    }

    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        let result = self.collection.pop();
        if result.is_some() {
            let node = result.clone().unwrap();
            self.get_node.remove(node.get_state());
        }
        result
    }
}

use std::{
    cmp::Reverse,
    collections::{HashMap, VecDeque},
    hash::Hash,
    ops::{Deref, DerefMut},
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
    fn reset(&mut self);
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
            if old_node.get_g_cost() > item.get_g_cost() {
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

    pub fn reset(&mut self) {
        self.collection.reset();
        self.get_node.clear();
    }

    pub fn size(&self) -> usize {
        self.get_node.len()
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

    fn reset(&mut self) {
        self.clear();
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

    fn reset(&mut self) {
        self.clear();
    }
}
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

macro_rules! create_backend {
    ($name:ident, $cost_fn:ident) => {
        pub struct $name<State, Action>(
            PriorityQueue<Rc<Node<State, Action>>, Reverse<OrderedFloat<f64>>>,
        )
        where
            State: WorldState<Action>,
            Action: Clone;

        impl<State, Action> Deref for $name<State, Action>
        where
            State: WorldState<Action>,
            Action: Clone,
        {
            type Target = PriorityQueue<Rc<Node<State, Action>>, Reverse<OrderedFloat<f64>>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<State, Action> DerefMut for $name<State, Action>
        where
            State: WorldState<Action>,
            Action: Clone,
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<State, Action> Default for $name<State, Action>
        where
            State: WorldState<Action>,
            Action: Clone + Hash + Eq,
        {
            fn default() -> Self {
                Self(Default::default())
            }
        }

        impl<State, Action> FrontierBackend<State, Action> for $name<State, Action>
        where
            State: WorldState<Action>,
            Action: Clone + Hash + Eq,
        {
            fn enqueue(&mut self, item: Rc<Node<State, Action>>) {
                let cost = item.$cost_fn();
                self.push(item, Reverse(OrderedFloat::from(cost)));
            }

            fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
                self.pop().map(|(node, _)| node)
            }

            fn delete(&mut self, state: &State) -> bool {
                let to_remove = self.iter().find_map(|(node, _)| {
                    if node.get_state() == state {
                        Some(node.clone())
                    } else {
                        None
                    }
                });

                if let Some(node) = to_remove {
                    self.remove(&node);
                    true
                } else {
                    false
                }
            }

            fn reset(&mut self) {
                self.clear();
            }
        }
    };
}

// Genera le strutture specifiche utilizzando la macro
create_backend!(MinGBackend, get_g_cost);
create_backend!(MinHBackend, get_h_cost);
create_backend!(MinFBackend, get_f_cost);

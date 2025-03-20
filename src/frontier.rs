use std::{
    cmp::Reverse,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
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
        let mut to_remove: Option<&State> = None;
        if let Some(old_node) = self.get_node.get(item.get_state()) {
            if old_node.get_g_cost() > item.get_g_cost() {
                to_remove = old_node.get_state().into();
                old_node.mark_dead();
            } else {
                return false;
            }
        }

        if let Some(to_remove) = to_remove {
            self.get_node.remove(&to_remove.clone());
        }

        let state = item.get_state().clone();
        let to_insert = Rc::new(item);
        assert!(self.get_node.insert(state, to_insert.clone()).is_none());
        self.collection.enqueue(to_insert);

        true
    }

    pub fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        let mut result = self.collection.dequeue();
        while result.clone().map_or(false, |n| n.is_dead()) {
            result = self.collection.dequeue()
        }
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

impl<State, Action, Backend> Debug for Frontier<State, Action, Backend>
where
    State: WorldState<Action>,
    Action: Clone,
    Backend: FrontierBackend<State, Action> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.collection)
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

    fn reset(&mut self) {
        self.clear();
    }
}

use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

pub trait NodeCost<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn cost(node: &Node<State, Action>) -> OrderedFloat<f64>;
}

pub struct AStarPolicy {}

impl<State, Action> NodeCost<State, Action> for AStarPolicy
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn cost(node: &Node<State, Action>) -> OrderedFloat<f64> {
        node.get_f_cost()
    }
}

pub struct BestFirstPolicy {}

impl<State, Action> NodeCost<State, Action> for BestFirstPolicy
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn cost(node: &Node<State, Action>) -> OrderedFloat<f64> {
        node.get_h_cost()
    }
}

pub struct MinCostPolicy {}

impl<State, Action> NodeCost<State, Action> for MinCostPolicy
where
    State: WorldState<Action>,
    Action: Clone,
{
    fn cost(node: &Node<State, Action>) -> OrderedFloat<f64> {
        node.get_g_cost()
    }
}

pub struct PriorityBackend<State, Action, Policy>
where
    State: WorldState<Action>,
    Action: Clone,
    Policy: NodeCost<State, Action>,
{
    collection: PriorityQueue<Rc<Node<State, Action>>, Reverse<OrderedFloat<f64>>>,
    policy: PhantomData<Policy>,
}

impl<State, Action, Policy> Default for PriorityBackend<State, Action, Policy>
where
    State: WorldState<Action>,
    Action: Clone + Hash + Eq,
    Policy: NodeCost<State, Action>,
{
    fn default() -> Self {
        Self {
            collection: Default::default(),
            policy: PhantomData,
        }
    }
}

impl<State, Action, Policy> FrontierBackend<State, Action>
    for PriorityBackend<State, Action, Policy>
where
    State: WorldState<Action>,
    Action: Clone + Hash + Eq,
    Policy: NodeCost<State, Action>,
{
    fn enqueue(&mut self, item: Rc<Node<State, Action>>) {
        let cost = Policy::cost(item.as_ref());
        self.collection.push(item, Reverse(cost));
    }

    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        self.collection.pop().map(|(x, _)| x)
    }

    fn reset(&mut self) {
        self.collection.clear()
    }
}

impl<State, Action, Policy> Debug for PriorityBackend<State, Action, Policy>
where
    State: WorldState<Action>,
    Action: Clone,
    Policy: NodeCost<State, Action>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for item in self.collection.iter() {
            item.fmt(f)?;
            write!(f, ",")?;
        }
        write!(f, "}}")
    }
}

// Genera le strutture specifiche utilizzando la macro
pub type MinCostBackend<State, Action> = PriorityBackend<State, Action, MinCostPolicy>;
pub type BestFirstBackend<State, Action> = PriorityBackend<State, Action, BestFirstPolicy>;
pub type AStarBackend<State, Action> = PriorityBackend<State, Action, AStarPolicy>;

use std::{
    cmp::Reverse,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

use crate::agent::{Node, StateExplorerProblem};

pub trait FrontierBackend<P>: Default
where
    P: StateExplorerProblem,
{
    fn enqueue(&mut self, item: Rc<Node<P>>);
    fn dequeue(&mut self) -> Option<Rc<Node<P>>>;
    fn reset(&mut self);
}

pub struct Frontier<P, Backend>
where
    P: StateExplorerProblem,
    Backend: FrontierBackend<P>,
{
    collection: Backend,
    get_node: HashMap<P::State, Rc<Node<P>>>,
}

impl<P, Backend> Frontier<P, Backend>
where
    P: StateExplorerProblem<State: Eq + Hash + Clone, Action: Clone>,
    Backend: FrontierBackend<P>,
{
    pub fn new() -> Self {
        Self {
            collection: Backend::default(),
            get_node: HashMap::new(),
        }
    }

    // TODO: change bool into an enum
    // TODO: change if the cost is less than the actual node
    pub fn enqueue_or_replace(&mut self, item: Node<P>) -> bool {
        let mut to_remove: Option<&P::State> = None;
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

    pub fn dequeue(&mut self) -> Option<Rc<Node<P>>> {
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

impl<P, Backend> Debug for Frontier<P, Backend>
where
    P: StateExplorerProblem,
    Backend: FrontierBackend<P> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.collection)
    }
}

pub type DequeBackend<P> = VecDeque<Rc<Node<P>>>;

impl<P> FrontierBackend<P> for DequeBackend<P>
where
    P: StateExplorerProblem,
{
    fn dequeue(&mut self) -> Option<Rc<Node<P>>> {
        self.pop_front()
    }

    fn enqueue(&mut self, item: Rc<Node<P>>) {
        self.push_back(item);
    }

    fn reset(&mut self) {
        self.clear();
    }
}

pub type StackBackend<P> = Vec<Rc<Node<P>>>;

impl<P> FrontierBackend<P> for StackBackend<P>
where
    P: StateExplorerProblem,
{
    fn enqueue(&mut self, item: Rc<Node<P>>) {
        self.push(item);
    }

    fn dequeue(&mut self) -> Option<Rc<Node<P>>> {
        self.pop()
    }

    fn reset(&mut self) {
        self.clear();
    }
}

use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

pub trait NodeCost<P>
where
    P: StateExplorerProblem,
{
    fn cost(node: &Node<P>) -> OrderedFloat<f64>;
}

pub struct AStarPolicy {}

impl<P> NodeCost<P> for AStarPolicy
where
    P: StateExplorerProblem<Action: Clone>,
{
    fn cost(node: &Node<P>) -> OrderedFloat<f64> {
        node.get_f_cost()
    }
}

pub struct BestFirstPolicy {}

impl<P> NodeCost<P> for BestFirstPolicy
where
    P: StateExplorerProblem<Action: Clone>,
{
    fn cost(node: &Node<P>) -> OrderedFloat<f64> {
        node.get_h_cost()
    }
}

pub struct MinCostPolicy {}

impl<P> NodeCost<P> for MinCostPolicy
where
    P: StateExplorerProblem<Action: Clone>,
{
    fn cost(node: &Node<P>) -> OrderedFloat<f64> {
        node.get_g_cost()
    }
}

pub struct PriorityBackend<P, Policy>
where
    P: StateExplorerProblem<Action: Clone>,
    Policy: NodeCost<P>,
{
    collection: PriorityQueue<Rc<Node<P>>, Reverse<OrderedFloat<f64>>>,
    policy: PhantomData<Policy>,
}

impl<P, Policy> Default for PriorityBackend<P, Policy>
where
    P: StateExplorerProblem<State: Eq + Hash, Action: Eq + Clone> + Eq,
    Policy: NodeCost<P>,
{
    fn default() -> Self {
        Self {
            collection: Default::default(),
            policy: PhantomData,
        }
    }
}

impl<P, Policy> FrontierBackend<P> for PriorityBackend<P, Policy>
where
    P: StateExplorerProblem<State: Eq + Hash, Action: Eq + Clone> + Eq,
    Policy: NodeCost<P>,
{
    fn enqueue(&mut self, item: Rc<Node<P>>) {
        let cost = Policy::cost(item.as_ref());
        self.collection.push(item, Reverse(cost));
    }

    fn dequeue(&mut self) -> Option<Rc<Node<P>>> {
        self.collection.pop().map(|(x, _)| x)
    }

    fn reset(&mut self) {
        self.collection.clear()
    }
}

impl<P, Policy> Debug for PriorityBackend<P, Policy>
where
    P: StateExplorerProblem<State: Debug, Action: Clone>,
    Policy: NodeCost<P>,
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
pub type MinCostBackend<P> = PriorityBackend<P, MinCostPolicy>;
pub type BestFirstBackend<P> = PriorityBackend<P, BestFirstPolicy>;
pub type AStarBackend<P> = PriorityBackend<P, AStarPolicy>;

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    fmt::{Debug, Pointer},
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

use crate::problem::*;
use crate::statexplorer::node::Node;

pub trait FrontierBackend<'a, P>: Default
where
    P: Utility,
{
    fn enqueue(&mut self, item: &'a Node<'a, P>);
    fn dequeue(&mut self) -> Option<&'a Node<'a, P>>;
    fn reset(&mut self);
    fn size(&self) -> usize;
}

pub struct Frontier<'a, P, Backend>
where
    P: Utility,
    Backend: FrontierBackend<'a, P>,
{
    collection: Backend,
    get_node: HashMap<P::State, &'a Node<'a, P>>,
}

impl<'a, P, Backend> Frontier<'a, P, Backend>
where
    P: Utility<State: Eq + Hash + Clone, Action: Clone>,
    Backend: FrontierBackend<'a, P>,
{
    pub fn new() -> Self {
        Self {
            collection: Backend::default(),
            get_node: HashMap::new(),
        }
    }

    pub fn enqueue_or_replace(&mut self, item: &'a Node<'a, P>) -> bool {
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
        assert!(self.get_node.insert(state, item).is_none());
        self.collection.enqueue(item);

        true
    }

    pub fn dequeue(&mut self) -> Option<&'a Node<'a, P>> {
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

impl<'a, P, Backend> Debug for Frontier<'a, P, Backend>
where
    P: Utility,
    Backend: FrontierBackend<'a, P> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.collection)
    }
}

pub type DequeBackend<'a, P> = VecDeque<&'a Node<'a, P>>;

impl<'a, P> FrontierBackend<'a, P> for DequeBackend<'a, P>
where
    P: Utility,
{
    fn dequeue(&mut self) -> Option<&'a Node<'a, P>> {
        self.pop_front()
    }

    fn enqueue(&mut self, item: &'a Node<'a, P>) {
        self.push_back(item);
    }

    fn reset(&mut self) {
        self.clear();
    }

    fn size(&self) -> usize {
        self.len()
    }
}

pub type StackBackend<P> = Vec<Rc<Node<P>>>;

impl<P> FrontierBackend<P> for StackBackend<P>
where
    P: Utility,
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

    fn size(&self) -> usize {
        self.len()
    }
}

pub trait NodeCost<P>
where
    P: Utility,
{
    fn cost(node: &Node<P>) -> P::Cost;
}

pub struct AStarPolicy {}

impl<P> NodeCost<P> for AStarPolicy
where
    P: Utility<Action: Clone>,
{
    fn cost(node: &Node<P>) -> P::Cost {
        node.get_f_cost()
    }
}

pub struct BestFirstPolicy {}

impl<P> NodeCost<P> for BestFirstPolicy
where
    P: Utility<Action: Clone>,
{
    fn cost(node: &Node<P>) -> P::Cost {
        node.get_h_cost()
    }
}

pub struct MinCostPolicy {}

impl<P> NodeCost<P> for MinCostPolicy
where
    P: Utility<Action: Clone>,
{
    fn cost(node: &Node<P>) -> P::Cost {
        node.get_g_cost()
    }
}

pub struct NodeAndCost<P>(Rc<Node<P>>, Reverse<P::Cost>)
where
    P: Utility;

impl<P> NodeAndCost<P>
where
    P: Utility,
{
    pub fn new(node: Rc<Node<P>>, cost: P::Cost) -> Self {
        Self(node, Reverse(cost))
    }
}

impl<P> Ord for NodeAndCost<P>
where
    P: Utility,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

impl<P> PartialOrd for NodeAndCost<P>
where
    P: Utility,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<P> PartialEq for NodeAndCost<P>
where
    P: Utility,
{
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<P> Eq for NodeAndCost<P> where P: Utility {}

impl<P> Debug for NodeAndCost<P>
where
    P: Utility,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct PriorityBackend<P, Policy>
where
    P: Utility,
    Policy: NodeCost<P>,
{
    collection: BinaryHeap<NodeAndCost<P>>,
    policy: PhantomData<Policy>,
}

impl<P, Policy> Default for PriorityBackend<P, Policy>
where
    P: Utility,
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
    P: Utility,
    Policy: NodeCost<P>,
{
    fn enqueue(&mut self, item: Rc<Node<P>>) {
        let cost = Policy::cost(item.as_ref());
        self.collection.push(NodeAndCost::new(item, cost));
    }

    fn dequeue(&mut self) -> Option<Rc<Node<P>>> {
        self.collection.pop().map(|x| x.0)
    }

    fn reset(&mut self) {
        self.collection.clear()
    }

    fn size(&self) -> usize {
        self.collection.len()
    }
}

impl<P, Policy> Debug for PriorityBackend<P, Policy>
where
    P: Utility<State: Debug, Action: Clone, Cost: Debug>,
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

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    fmt::{Debug, Pointer},
    hash::Hash,
    marker::PhantomData,
};

use crate::problem::*;
use crate::statexplorer::node::Node;

pub trait FrontierBackend<'a, P>: Default
where
    P: Utility,
{
    const REOPENS_IMPROVED_STATES: bool = false;

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
        let mut replaced_old = false;
        if let Some(old_node) = self.get_node.get(item.get_state()) {
            if old_node.get_g_cost() > item.get_g_cost() {
                old_node.mark_dead();
                replaced_old = true;
            } else {
                return false;
            }
        }

        if replaced_old {
            self.get_node.remove(item.get_state());
        }

        let state = item.get_state().clone();
        assert!(self.get_node.insert(state, item).is_none());
        self.collection.enqueue(item);

        true
    }

    pub fn dequeue(&mut self) -> Option<&'a Node<'a, P>> {
        let mut result = self.collection.dequeue();
        while result.is_some_and(|n| n.is_dead()) {
            result = self.collection.dequeue()
        }
        if let Some(node) = result {
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

impl<'a, P, Backend> Default for Frontier<'a, P, Backend>
where
    P: Utility<State: Eq + Hash + Clone, Action: Clone>,
    Backend: FrontierBackend<'a, P>,
{
    fn default() -> Self {
        Self::new()
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

pub type StackBackend<'a, P> = Vec<&'a Node<'a, P>>;

impl<'a, P> FrontierBackend<'a, P> for StackBackend<'a, P>
where
    P: Utility,
{
    fn enqueue(&mut self, item: &'a Node<'a, P>) {
        self.push(item);
    }

    fn dequeue(&mut self) -> Option<&'a Node<'a, P>> {
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
    const REOPENS_IMPROVED_STATES: bool = false;

    fn cost(node: &Node<P>) -> P::Cost;

    /// Secondary ordering for nodes with the same primary cost.
    ///
    /// The default preserves the previous behavior. A* overrides it to favor
    /// nodes closer to a goal when their f-cost is tied.
    fn tie_break(_node: &Node<P>) -> P::Cost {
        P::Cost::default()
    }
}

pub struct AStarPolicy {}

impl<P> NodeCost<P> for AStarPolicy
where
    P: Utility<Action: Clone>,
{
    const REOPENS_IMPROVED_STATES: bool = true;

    fn cost(node: &Node<P>) -> P::Cost {
        node.get_f_cost()
    }

    fn tie_break(node: &Node<P>) -> P::Cost {
        node.get_h_cost()
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
    const REOPENS_IMPROVED_STATES: bool = true;

    fn cost(node: &Node<P>) -> P::Cost {
        node.get_g_cost()
    }
}

pub struct NodeAndCost<'a, P>(&'a Node<'a, P>, Reverse<(P::Cost, P::Cost, u64)>)
where
    P: Utility;

impl<'a, P> NodeAndCost<'a, P>
where
    P: Utility,
{
    pub fn new(node: &'a Node<P>, cost: P::Cost, tie_break: P::Cost) -> Self {
        Self::with_sequence(node, cost, tie_break, 0)
    }

    fn with_sequence(node: &'a Node<P>, cost: P::Cost, tie_break: P::Cost, sequence: u64) -> Self {
        Self(node, Reverse((cost, tie_break, sequence)))
    }
}

impl<P> Ord for NodeAndCost<'_, P>
where
    P: Utility,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

impl<P> PartialOrd for NodeAndCost<'_, P>
where
    P: Utility,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<P> PartialEq for NodeAndCost<'_, P>
where
    P: Utility,
{
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<P> Eq for NodeAndCost<'_, P> where P: Utility {}

impl<P> Debug for NodeAndCost<'_, P>
where
    P: Utility,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct PriorityBackend<'a, P, Policy>
where
    P: Utility,
    Policy: NodeCost<P>,
{
    collection: BinaryHeap<NodeAndCost<'a, P>>,
    next_sequence: u64,
    policy: PhantomData<Policy>,
}

impl<'a, P, Policy> Default for PriorityBackend<'a, P, Policy>
where
    P: Utility,
    Policy: NodeCost<P>,
{
    fn default() -> Self {
        Self {
            collection: Default::default(),
            next_sequence: 0,
            policy: PhantomData,
        }
    }
}

impl<'a, P, Policy> FrontierBackend<'a, P> for PriorityBackend<'a, P, Policy>
where
    P: Utility,
    Policy: NodeCost<P>,
{
    const REOPENS_IMPROVED_STATES: bool = Policy::REOPENS_IMPROVED_STATES;

    fn enqueue(&mut self, item: &'a Node<'a, P>) {
        let cost = Policy::cost(item);
        let tie_break = Policy::tie_break(item);
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.collection
            .push(NodeAndCost::with_sequence(item, cost, tie_break, sequence));
    }

    fn dequeue(&mut self) -> Option<&'a Node<'a, P>> {
        self.collection.pop().map(|x| x.0)
    }

    fn reset(&mut self) {
        self.collection.clear();
        self.next_sequence = 0;
    }

    fn size(&self) -> usize {
        self.collection.len()
    }
}

impl<P, Policy> Debug for PriorityBackend<'_, P, Policy>
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

pub type MinCostBackend<'a, P> = PriorityBackend<'a, P, MinCostPolicy>;
pub type BestFirstBackend<'a, P> = PriorityBackend<'a, P, BestFirstPolicy>;
pub type AStarBackend<'a, P> = PriorityBackend<'a, P, AStarPolicy>;

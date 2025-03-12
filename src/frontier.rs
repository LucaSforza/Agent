use std::{collections::VecDeque, rc::Rc};

use crate::agent::{Node, WorldState};

pub trait Frontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    type Iter<'a>: Iterator<Item = &'a mut Rc<Node<State, Action>>>
    where
        State: 'a,
        Action: 'a,
        Self: 'a;

    fn new() -> Self;
    fn enqueue(&mut self, item: Rc<Node<State, Action>>);
    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>>;
    fn mut_elements(&mut self) -> Self::Iter<'_>;
}

pub type DequeFrontier<State, Action> = VecDeque<Rc<Node<State, Action>>>;

impl<State, Action> Frontier<State, Action> for DequeFrontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    type Iter<'a>
        = std::collections::vec_deque::IterMut<'a, Rc<Node<State, Action>>>
    where
        State: 'a,
        Action: 'a;

    fn new() -> Self {
        Self::new()
    }

    fn enqueue(&mut self, item: Rc<Node<State, Action>>) {
        self.push_back(item);
    }

    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        self.pop_front()
    }

    fn mut_elements(&mut self) -> Self::Iter<'_> {
        self.iter_mut()
    }
}

pub type StackFrontier<State, Action> = Vec<Rc<Node<State, Action>>>;

impl<State, Action> Frontier<State, Action> for StackFrontier<State, Action>
where
    State: WorldState<Action>,
    Action: Clone,
{
    type Iter<'a>
        = std::slice::IterMut<'a, Rc<Node<State, Action>>>
    where
        State: 'a,
        Action: 'a;

    fn new() -> Self {
        Self::new()
    }

    fn enqueue(&mut self, item: Rc<Node<State, Action>>) {
        self.push(item);
    }

    fn dequeue(&mut self) -> Option<Rc<Node<State, Action>>> {
        self.pop()
    }

    fn mut_elements(&mut self) -> Self::Iter<'_> {
        self.iter_mut()
    }
}

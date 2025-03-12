use std::clone::Clone;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::vec::Vec;

pub trait Goal<State> {
    fn is_goal(&self, state: &State) -> bool;
}

pub trait WorldState<Action>: Clone + Eq + Hash {
    type Iter: Iterator<Item = Action>;

    fn executable_actions(&self) -> Self::Iter;
    fn result(&self, action: &Action) -> Self;
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
    fn new(
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

    fn get_plan(&self) -> Vec<Action> {
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

    fn get_total_cost(&self) -> f32 {
        return self.total_cost;
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}

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
    fn add(&mut self, item: Rc<Node<State, Action>>);
    fn pop(&mut self) -> Option<Rc<Node<State, Action>>>;
    fn delete(&mut self, state: &State);
    fn iter_mut(&mut self) -> Self::Iter<'_>;
}

pub fn search<State, Action, Front>(
    init_state: State,
    goal: impl Goal<State>,
) -> Option<Vec<Action>>
where
    State: WorldState<Action>,
    Action: Clone + Default,
    Front: Frontier<State, Action>,
{
    let mut frontier = Front::new();
    let mut explored = HashSet::new();
    frontier.add(Rc::new(Node::new(None, init_state, Action::default(), 0.0)));
    while let Some(curr_node) = frontier.pop() {
        let curr_state = &curr_node.state;
        if goal.is_goal(curr_state) {
            return Some(curr_node.get_plan());
        } else {
            for action in curr_state.executable_actions() {
                let new_state = curr_state.result(&action);
                if !explored.iter().any(|x| *x == new_state) {
                    let new_node = Rc::new(Node::new(
                        Some(curr_node.clone()),
                        new_state.clone(),
                        action,
                        1.0,
                    )); // TODO: cambiare costo
                    let mut found = false;
                    for existing_node in frontier.iter_mut() {
                        if existing_node.state == new_state {
                            found = true;
                            if new_node.get_total_cost() < existing_node.get_total_cost() {
                                *existing_node = new_node.clone(); // Aggiornamento sicuro
                            }
                            break; // Uscita dal loop per evitare iterazioni inutili
                        }
                    }

                    if !found {
                        frontier.add(new_node);
                    }
                }
            }
        }
        explored.insert(curr_state.clone());
    }
    return None;
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

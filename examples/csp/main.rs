use agent::improve::algorithms::Verbosity;
use agent::{
    improve::{algorithms::SteepestDescend, resolver::Resolver},
    problem::{self, Problem, StatePerturbation, SuitableState, Utility},
};
use std::cmp::max;

/*
struct Domain {
    low: u32,
    up: u32,
}

impl Domain {
    fn from_parts(low: u32, up: u32) -> Self {
        Self { low: low, up: up }
    }

    fn constains(&self, x: u32) -> bool {
        self.low <= x && x <= self.up
    }
} */

struct Csp {
    n: usize,
    // dom: Domain,
}

impl Csp {
    fn new(n: usize) -> Self {
        Self {
            n: n,
            // dom: Domain::from_parts(1, 5),
        }
    }
}

impl Problem for Csp {
    type State = Vec<i32>;
}

impl problem::CostructSolution for Csp {
    type Action = i32;
    type Cost = i32;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        let mut actions: Vec<i32> = Vec::with_capacity(1);

        let val = match state.len() {
            0 => Some(3),
            1 => Some(1),
            2 => Some(4),
            3 => Some(5),
            4 => Some(2),
            _ => None,
        };
        if let Some(val) = val {
            actions.push(val)
        }

        actions.into_iter()
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        let mut new_state = state.clone();
        new_state.push(*action);
        (new_state, 1)
    }
}

impl Utility for Csp {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        if state.len() != self.n {
            panic!()
        }

        let mut cost = 0;

        // C_3
        cost += max((state[2] * state[2] + state[3] * state[3]) - 15, 0);
        // C_2
        cost += max(state[1] - state[2], 0);
        // C_1
        cost += max(1 - (state[0] - state[2]), 0);
        // C_4
        cost += max(3 - state[4], 0);
        // C_5
        cost += max(3 - (state[0] + state[4]), 0);
        cost
    }
}

struct ChangeVariabile {
    i: usize,
    new_val: i32,
}

impl ChangeVariabile {
    fn from_parts(i: usize, new_val: i32) -> Self {
        Self {
            i: i,
            new_val: new_val,
        }
    }
}

impl SuitableState for Csp {
    fn is_suitable(&self, state: &Self::State) -> bool {
        state.len() == self.n
    }
}

impl StatePerturbation for Csp {
    type Perturbation = ChangeVariabile;

    fn perturbations(&self, state: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
        let mut actions = Vec::with_capacity(self.n * 5);

        for i in 0..self.n {
            for j in 1..=5 {
                if j != state[i] {
                    actions.push(ChangeVariabile::from_parts(i, j));
                }
            }
        }

        actions.into_iter()
    }

    fn perturb(&self, state: &Self::State, action: &Self::Perturbation) -> Self::State {
        let mut new_state = state.clone();
        new_state[action.i] = action.new_val;
        new_state
    }
}

fn main() {
    let mut resolver = Resolver::new(SteepestDescend::with_verbosity(rand::rng(), Verbosity::Max));
    let csp = Csp::new(5);
    let sol = resolver.resolve(&csp);
    println!("{:?}", sol)
}

use std::ops::Add;

use rand::Rng;

pub trait Problem {
    type State;
    type Action;
    type Cost: Copy + Default + Eq + Ord + Add<Output = Self::Cost> + PartialOrd + PartialEq;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action>;
    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost);
    fn heuristic(&self, state: &Self::State) -> Self::Cost;
}

pub trait StateExplorerProblem: Problem {
    fn is_goal(&self, state: &Self::State) -> bool;
}

pub trait IterativeImprovingProblem: Problem {
    fn random_state<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::State;
}

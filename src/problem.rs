use rand::Rng;
use rand_distr::num_traits::Num;

pub trait Problem {
    type State;
    type Action;
    type Cost: Default + Copy + Ord + Num;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action>;
    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost);
}

pub trait Utility: Problem {
    fn heuristic(&self, state: &Self::State) -> Self::Cost;
}

pub trait WithSolution: Problem {
    fn is_goal(&self, state: &Self::State) -> bool;
}

pub trait RandomizeState: Problem {
    fn random_state<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::State;
}

pub trait StateExplorerProblem: Problem + Utility + WithSolution {}

pub trait IterativeImprovingProblem: Problem + Utility + RandomizeState {}

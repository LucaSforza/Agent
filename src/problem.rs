use rand::{seq::IteratorRandom, Rng};
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

pub trait ModifyState: Problem {
    type ModifyAction;

    fn modify_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::ModifyAction>;
    fn modify(&self, state: &Self::State, action: &Self::ModifyAction) -> Self::State;
}

pub trait ModifyRandom: ModifyState {
    fn random_modify_action<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::ModifyAction>;
}

impl<T> ModifyRandom for T
where
    T: ModifyState,
{
    fn random_modify_action<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::ModifyAction> {
        self.modify_actions(state).choose(rng)
    }
}

pub trait RandomizeState: Problem {
    fn random_action<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::Action>;

    fn random_state<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::State;
}

impl<T> RandomizeState for T
where
    T: WithSolution + Problem<State: Default>,
{
    fn random_action<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::Action> {
        self.executable_actions(state).choose(rng)
    }

    fn random_state<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::State {
        // TODO: What if the while loop runs indefinitely?
        let mut state = Default::default();
        while !self.is_goal(&state) {
            if let Some(action) = self.random_action(rng, &state) {
                state = self.result(&state, &action).0;
            } else {
                state = Default::default()
            }
        }

        state
    }
}

pub trait Genetic: Problem {
    fn crossover<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
        other: &Self::State,
    ) -> Self::State;

    fn mutate_gene<R: Rng + ?Sized>(&self, rng: &mut R, state: &mut Self::State);
}

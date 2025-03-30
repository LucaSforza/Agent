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

pub trait StatePerturbation: Problem {
    type Perturbation;

    fn perturbations(&self, state: &Self::State) -> impl Iterator<Item = Self::Perturbation>;
    fn perturb(&self, state: &Self::State, action: &Self::Perturbation) -> Self::State;
}

pub trait RandomPerturbation: StatePerturbation {
    fn random_pertubation<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::Perturbation>;
}

impl<T> RandomPerturbation for T
where
    T: StatePerturbation,
{
    fn random_pertubation<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::Perturbation> {
        self.perturbations(state).choose(rng)
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

pub trait Crossover: Problem {
    fn crossover<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
        other: &Self::State,
    ) -> Self::State;
}

pub trait MutateGene: Problem {
    fn mutate_gene<R: Rng + ?Sized>(&self, rng: &mut R, state: &Self::State) -> Self::State;
}

impl<T> MutateGene for T
where
    T: RandomPerturbation<State: Clone>,
{
    fn mutate_gene<R: Rng + ?Sized>(&self, rng: &mut R, state: &Self::State) -> Self::State {
        let action = self.random_pertubation(rng, state);
        if let Some(action) = action {
            self.perturb(state, &action)
        } else {
            state.clone()
        }
    }
}

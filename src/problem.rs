use rand::{seq::IteratorRandom, Rng};
use rand_distr::num_traits::Num;

pub trait Problem {
    type State;
}

pub trait InitState: Problem {
    fn init_state(&self) -> Self::State;
}

impl<T> InitState for T
where
    T: Problem<State: Default>,
{
    fn init_state(&self) -> Self::State {
        Default::default()
    }
}

pub trait CostructSolution: Problem {
    type Action;
    type Cost: Default + Copy + Ord + Num;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action>;
    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost);
}

pub trait Utility: CostructSolution {
    fn heuristic(&self, state: &Self::State) -> Self::Cost;
}

pub trait SuitableState: Problem {
    fn is_suitable(&self, state: &Self::State) -> bool;
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

pub trait RandomAction: CostructSolution {
    fn random_action<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::Action>;
}

impl<T> RandomAction for T
where
    T: CostructSolution,
{
    fn random_action<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
    ) -> Option<Self::Action> {
        self.executable_actions(state).choose(rng)
    }
}

pub trait RandomState: CostructSolution {
    fn random_state<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::State;
}

impl<T> RandomState for T
where
    T: SuitableState + RandomAction + InitState,
{
    fn random_state<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::State {
        // TODO: What if the while loop runs indefinitely?
        let mut state = self.init_state();
        while !self.is_suitable(&state) {
            if let Some(action) = self.random_action(rng, &state) {
                state = self.result(&state, &action).0;
            } else {
                state = self.init_state();
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

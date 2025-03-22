use rand::Rng;

use crate::problem::IterativeImprovingProblem;

pub trait Resolver<P>
where
    P: IterativeImprovingProblem,
{
    fn resolve(&mut self, problem: &P) -> P::State;

    fn resolve_restart(&mut self, problem: &P, max_restarts: usize) -> P::State {
        let mut result = self.resolve(problem);
        let mut old_h = problem.heuristic(&result);
        for _ in 1..max_restarts {
            let new_result = self.resolve(problem);
            let new_h = problem.heuristic(&new_result);
            if new_h <= P::Cost::default() {
                return new_result;
            }
            if new_h < old_h {
                result = new_result;
                old_h = new_h;
            }
        }
        return result;
    }
}

pub struct SteepestDescend<R: Rng> {
    rng: R,
}

impl<R: Rng> SteepestDescend<R> {
    pub fn new(rng: R) -> Self {
        Self { rng: rng }
    }
}

impl<R: Rng, P: IterativeImprovingProblem<State: Clone>> Resolver<P> for SteepestDescend<R> {
    fn resolve(&mut self, problem: &P) -> <P>::State {
        let mut curr_state = problem.random_state(&mut self.rng);
        let mut curr_h = problem.heuristic(&curr_state);
        loop {
            let mut new_curr_state = curr_state.clone();
            let mut new_curr_h = curr_h;
            for a in problem.executable_actions(&curr_state) {
                let (new_state, _) = problem.result(&curr_state, &a);
                let new_h = problem.heuristic(&new_state);
                if new_h < new_curr_h {
                    new_curr_state = new_state;
                    new_curr_h = new_h;
                }
            }
            if curr_h > new_curr_h {
                curr_state = new_curr_state;
                curr_h = new_curr_h;
            } else {
                return curr_state;
            }
        }
    }
}

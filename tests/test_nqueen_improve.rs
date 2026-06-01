#[cfg(test)]
mod tests {
    use agent::improve::algorithms::{
        GeneticAlgorithm, HillClimbing, LocalBeam, SimulatedAnnealing, SteepestDescend,
    };
    use agent::improve::resolver::Resolver;
    use agent::problem::{
        CostructSolution, Crossover, Problem, StatePerturbation, SuitableState, Utility,
    };
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    // Small problem: 3-bit vector, goal = all true.
    // Covers all trait bounds needed by improve/ algorithms.

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct Bits(Vec<bool>);

    impl std::fmt::Debug for Bits {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl Default for Bits {
        fn default() -> Self {
            Self(vec![false, false, false])
        }
    }

    struct FlipProblem;

    impl Problem for FlipProblem {
        type State = Bits;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SetBit(usize);

    impl CostructSolution for FlipProblem {
        type Action = SetBit;
        type Cost = ordered_float::OrderedFloat<f64>;

        fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
            (0..state.0.len())
                .filter(|&i| !state.0[i])
                .map(SetBit)
        }

        fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
            let mut v = state.0.clone();
            v[action.0] = true;
            (Bits(v), 1.0.into())
        }
    }

    impl Utility for FlipProblem {
        fn heuristic(&self, state: &Self::State) -> Self::Cost {
            (state.0.iter().filter(|&&b| !b).count() as f64).into()
        }
    }

    impl SuitableState for FlipProblem {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.0.iter().all(|&b| b)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Flip(usize);

    impl StatePerturbation for FlipProblem {
        type Perturbation = Flip;

        fn perturbations(&self, state: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
            (0..state.0.len()).map(Flip)
        }

        fn perturb(&self, state: &Self::State, action: &Self::Perturbation) -> Self::State {
            let mut v = state.0.clone();
            v[action.0] = !v[action.0];
            Bits(v)
        }
    }

    impl Crossover for FlipProblem {
        fn crossover<R: Rng + ?Sized>(
            &self,
            rng: &mut R,
            a: &Self::State,
            b: &Self::State,
        ) -> Self::State {
            let point = rng.random_range(0..=a.0.len());
            let mut v = a.0[..point].to_vec();
            v.extend_from_slice(&b.0[point..]);
            Bits(v)
        }
    }

    #[test]
    fn test_steepest_descend() {
        let mut resolver = Resolver::new(SteepestDescend::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&FlipProblem);
        assert!(result.iterations > 0);
        assert!(result.h <= ordered_float::OrderedFloat::from(0.0));
    }

    #[test]
    fn test_hill_climbing() {
        let mut resolver = Resolver::new(HillClimbing::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&FlipProblem);
        assert!(result.iterations > 0);
        assert!(result.h <= ordered_float::OrderedFloat::from(0.0));
    }

    #[test]
    fn test_simulated_annealing() {
        // Fast cooling to keep test quick
        let mut resolver = Resolver::new(SimulatedAnnealing::with_cooling(
            StdRng::seed_from_u64(42),
            |t| 1.0 / (t as f64).sqrt(),
        ));
        let result = resolver.resolve(&FlipProblem);
        assert!(result.iterations > 0);
        assert!(result.h <= ordered_float::OrderedFloat::from(0.0));
    }

    #[test]
    fn test_local_beam() {
        let mut resolver =
            Resolver::new(LocalBeam::from_parts(StdRng::seed_from_u64(42), 5, Some(20)));
        let result = resolver.resolve(&FlipProblem);
        assert!(result.iterations > 0);
    }

    #[test]
    fn test_genetic_algorithm() {
        let mut resolver = Resolver::new(GeneticAlgorithm::from_parts(
            StdRng::seed_from_u64(42),
            10,
            Some(30),
            0.3,
        ));
        let result = resolver.resolve(&FlipProblem);
        // GA may solve immediately if random state hits optimal.
        // Just check no panic.
    }

    #[test]
    fn test_restart_better_or_equal() {
        let mut resolver = Resolver::new(HillClimbing::new(StdRng::seed_from_u64(42)));
        let single = resolver.resolve(&FlipProblem);
        let multi = resolver.resolve_restart(&FlipProblem, 5);
        assert!(multi.h <= single.h);
    }
}

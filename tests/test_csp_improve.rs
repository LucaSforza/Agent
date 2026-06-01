#[cfg(test)]
mod tests {
    use agent::improve::algorithms::{
        GeneticAlgorithm, HillClimbing, LocalBeam, SimulatedAnnealing, SteepestDescend,
    };
    use agent::improve::resolver::Resolver;
    use agent::problem::{
        CostructSolution, Crossover, Problem, StatePerturbation, SuitableState, Utility,
    };
    use ordered_float::OrderedFloat;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    // Tiny CSP: 3 variables, domain 1..2.
    // Constraint: all values equal (h = count of differing).
    // State space small, solve fast.

    struct TinyCsp;

    #[derive(Clone, PartialEq, Eq, Hash, Default)]
    struct Assign(Vec<i32>);

    impl std::fmt::Debug for Assign {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl Problem for TinyCsp {
        type State = Assign;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct PickVar(usize, i32);

    impl CostructSolution for TinyCsp {
        type Action = PickVar;
        type Cost = OrderedFloat<f64>;

        fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
            let mut actions = Vec::new();
            if state.0.len() < 3 {
                let idx = state.0.len();
                for val in [1, 2] {
                    actions.push(PickVar(idx, val));
                }
            }
            actions.into_iter()
        }

        fn result(&self, state: &Self::State, a: &Self::Action) -> (Self::State, Self::Cost) {
            let mut v = state.0.clone();
            v.push(a.1);
            (Assign(v), 1.0.into())
        }
    }

    impl Utility for TinyCsp {
        fn heuristic(&self, state: &Self::State) -> Self::Cost {
            if state.0.len() < 3 {
                return 2.0.into();
            }
            let mut cost = 0.0;
            if state.0[0] != state.0[1] { cost += 1.0; }
            if state.0[1] != state.0[2] { cost += 1.0; }
            cost.into()
        }
    }

    impl SuitableState for TinyCsp {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.0.len() == 3
        }
    }

    #[derive(Clone, Copy)]
    struct SwapVar(usize, i32);

    impl StatePerturbation for TinyCsp {
        type Perturbation = SwapVar;

        fn perturbations(&self, state: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
            let mut actions = Vec::new();
            for i in 0..state.0.len() {
                for &val in &[1, 2] {
                    if val != state.0[i] {
                        actions.push(SwapVar(i, val));
                    }
                }
            }
            actions.into_iter()
        }

        fn perturb(&self, state: &Self::State, a: &Self::Perturbation) -> Self::State {
            let mut v = state.0.clone();
            v[a.0] = a.1;
            Assign(v)
        }
    }

    impl Crossover for TinyCsp {
        fn crossover<R: Rng + ?Sized>(
            &self,
            rng: &mut R,
            a: &Self::State,
            b: &Self::State,
        ) -> Self::State {
            let p = rng.random_range(1..3);
            let mut v = a.0[..p].to_vec();
            v.extend_from_slice(&b.0[p..]);
            Assign(v)
        }
    }

    #[test]
    fn test_csp_steepest_descend() {
        let mut resolver = Resolver::new(SteepestDescend::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&TinyCsp);
        assert!(result.h <= 0.0.into(), "h={}", result.h);
    }

    #[test]
    fn test_csp_hill_climbing() {
        let mut resolver = Resolver::new(HillClimbing::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&TinyCsp);
        assert!(result.h <= 0.0.into(), "h={}", result.h);
    }

    #[test]
    fn test_csp_simulated_annealing() {
        let mut resolver = Resolver::new(SimulatedAnnealing::with_cooling(
            StdRng::seed_from_u64(42),
            |t| 1.0 / (t as f64).sqrt(),
        ));
        let result = resolver.resolve(&TinyCsp);
        assert!(result.h <= 0.0.into(), "h={}", result.h);
    }

    #[test]
    fn test_csp_local_beam() {
        let mut resolver =
            Resolver::new(LocalBeam::from_parts(StdRng::seed_from_u64(42), 5, Some(20)));
        let result = resolver.resolve(&TinyCsp);
        assert!(result.h <= 0.0.into(), "h={}", result.h);
    }

    #[test]
    fn test_csp_genetic_algorithm() {
        let mut resolver = Resolver::new(GeneticAlgorithm::from_parts(
            StdRng::seed_from_u64(42),
            10,
            Some(20),
            0.3,
        ));
        let result = resolver.resolve(&TinyCsp);
        assert!(result.h <= 0.0.into(), "h={}", result.h);
    }

    #[test]
    fn test_csp_restart_improves() {
        let mut resolver = Resolver::new(HillClimbing::new(StdRng::seed_from_u64(42)));
        let single = resolver.resolve(&TinyCsp);
        let multi = resolver.resolve_restart(&TinyCsp, 5);
        assert!(multi.h <= single.h, "single={} multi={}", single.h, multi.h);
    }
}

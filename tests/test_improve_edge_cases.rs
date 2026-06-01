#[cfg(test)]
mod tests {
    use agent::improve::algorithms::{
        HillClimbing, LocalBeam, SimulatedAnnealing, SteepestDescend,
    };
    use agent::improve::resolver::Resolver;
    use agent::problem::{
        CostructSolution, Problem, StatePerturbation, SuitableState, Utility,
    };
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    // Problem where initial state is already optimal (h=0).
    #[derive(Clone)]
    struct TrivialOptimal;

    #[derive(Clone, PartialEq, Eq, Hash, Default)]
    struct ZeroState;

    impl std::fmt::Debug for ZeroState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "ZeroState")
        }
    }

    impl Problem for TrivialOptimal {
        type State = ZeroState;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Noop;

    impl CostructSolution for TrivialOptimal {
        type Action = Noop;
        type Cost = i32;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            std::iter::empty()
        }

        fn result(&self, _: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (ZeroState, 0)
        }
    }

    impl Utility for TrivialOptimal {
        fn heuristic(&self, _: &Self::State) -> Self::Cost {
            0
        }
    }

    impl SuitableState for TrivialOptimal {
        fn is_suitable(&self, _: &Self::State) -> bool {
            true
        }
    }

    impl StatePerturbation for TrivialOptimal {
        type Perturbation = Noop;

        fn perturbations(&self, _: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
            std::iter::empty()
        }

        fn perturb(&self, state: &Self::State, _: &Self::Perturbation) -> Self::State {
            state.clone()
        }
    }

    #[test]
    fn test_trivial_optimal_steepest_descend() {
        let mut resolver = Resolver::new(SteepestDescend::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&TrivialOptimal);
        assert_eq!(result.h, 0);
        eprintln!("Trivial SteepestDescend: iter={}", result.iterations);
    }

    #[test]
    fn test_trivial_optimal_hill_climbing() {
        let mut resolver = Resolver::new(HillClimbing::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&TrivialOptimal);
        assert_eq!(result.h, 0);
    }

    #[test]
    fn test_trivial_optimal_simulated_annealing() {
        let mut resolver = Resolver::new(SimulatedAnnealing::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&TrivialOptimal);
        assert_eq!(result.h, 0);
    }

    #[test]
    fn test_trivial_optimal_local_beam() {
        let mut resolver =
            Resolver::new(LocalBeam::from_parts(StdRng::seed_from_u64(42), 5, Some(10)));
        let result = resolver.resolve(&TrivialOptimal);
        assert_eq!(result.h, 0);
    }

    // Problem with a flat landscape: all perturbations same heuristic.
    #[derive(Clone)]
    struct FlatLandscape;

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct FlatState(i32);

    impl std::fmt::Debug for FlatState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Flat({})", self.0)
        }
    }

    impl Default for FlatState {
        fn default() -> Self {
            Self(0)
        }
    }

    impl Problem for FlatLandscape {
        type State = FlatState;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Bump;

    impl CostructSolution for FlatLandscape {
        type Action = Bump;
        type Cost = i32;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            (0..5).map(|_| Bump)
        }

        fn result(&self, state: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (FlatState(state.0 + 1), 1)
        }
    }

    impl Utility for FlatLandscape {
        fn heuristic(&self, _: &Self::State) -> Self::Cost {
            1
        }
    }

    impl SuitableState for FlatLandscape {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.0 >= 100
        }
    }

    impl StatePerturbation for FlatLandscape {
        type Perturbation = Bump;

        fn perturbations(&self, _: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
            (0..3).map(|_| Bump)
        }

        fn perturb(&self, state: &Self::State, _: &Self::Perturbation) -> Self::State {
            FlatState(state.0 + 1)
        }
    }

    #[test]
    fn test_flat_landscape_steepest_descend_terminates() {
        let mut resolver = Resolver::new(SteepestDescend::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&FlatLandscape);
        assert!(result.h > 0, "should not find goal in flat landscape");
        eprintln!(
            "Flat SteepestDescend: h={} iter={}",
            result.h, result.iterations
        );
    }

    #[test]
    fn test_flat_landscape_hill_climbing_terminates() {
        let mut resolver =
            Resolver::new(HillClimbing::with_max_lateral(StdRng::seed_from_u64(42), 10));
        let result = resolver.resolve(&FlatLandscape);
        eprintln!(
            "Flat HillClimbing: h={} iter={}",
            result.h, result.iterations
        );
    }

    #[test]
    fn test_flat_landscape_simulated_annealing_terminates() {
        let mut resolver = Resolver::new(SimulatedAnnealing::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&FlatLandscape);
        eprintln!(
            "Flat SimulatedAnnealing: h={} iter={}",
            result.h, result.iterations
        );
    }

    // Single perturbation only.
    #[derive(Clone)]
    struct SingleNeighbor;

    #[derive(Clone, PartialEq, Eq, Hash, Default)]
    struct SingleState {
        count: i32,
    }

    impl std::fmt::Debug for SingleState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Single({})", self.count)
        }
    }

    impl Problem for SingleNeighbor {
        type State = SingleState;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Inc;

    impl CostructSolution for SingleNeighbor {
        type Action = Inc;
        type Cost = i32;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            std::iter::once(Inc)
        }

        fn result(&self, state: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (SingleState {
                count: state.count + 1,
            }, 1)
        }
    }

    impl Utility for SingleNeighbor {
        fn heuristic(&self, state: &Self::State) -> Self::Cost {
            state.count
        }
    }

    impl SuitableState for SingleNeighbor {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.count >= 5
        }
    }

    impl StatePerturbation for SingleNeighbor {
        type Perturbation = Inc;

        fn perturbations(&self, _: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
            std::iter::once(Inc)
        }

        fn perturb(&self, state: &Self::State, _: &Self::Perturbation) -> Self::State {
            SingleState {
                count: state.count + 1,
            }
        }
    }

    #[test]
    fn test_single_perturbation_steepest_descend() {
        let mut resolver = Resolver::new(SteepestDescend::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&SingleNeighbor);
        eprintln!(
            "SingleNeighbor SteepestDescend: h={} iter={}",
            result.h, result.iterations
        );
    }

    #[test]
    fn test_single_perturbation_hill_climbing() {
        let mut resolver = Resolver::new(HillClimbing::new(StdRng::seed_from_u64(42)));
        let result = resolver.resolve(&SingleNeighbor);
        eprintln!(
            "SingleNeighbor HillClimbing: h={} iter={}",
            result.h, result.iterations
        );
    }

    // Local beam edge: k=1 (degrades to hill-climbing)
    #[test]
    fn test_local_beam_k1_terminates() {
        let mut resolver =
            Resolver::new(LocalBeam::from_parts(StdRng::seed_from_u64(42), 1, Some(20)));
        let result = resolver.resolve(&SingleNeighbor);
        eprintln!("LocalBeam(k=1): h={}, iter={}", result.h, result.iterations);
    }
}

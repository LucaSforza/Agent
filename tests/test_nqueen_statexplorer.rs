#[cfg(test)]
mod tests {
    use agent::problem::{CostructSolution, Problem, SuitableState, Utility};
    use agent::statexplorer::resolver::{
        AStarExplorer, BFSExplorer, BestFirstGreedyExplorer, DFSExplorer, MinCostExplorer,
    };
    use bumpalo::Bump;

    // Tiny problem: count from 0 to target.
    // Small state space (target=4, depth up to 4), fast search.

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct Counter(i32);

    impl std::fmt::Debug for Counter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Default for Counter {
        fn default() -> Self {
            Self(0)
        }
    }

    struct CountTo {
        target: i32,
    }

    impl Problem for CountTo {
        type State = Counter;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Inc;

    impl CostructSolution for CountTo {
        type Action = Inc;
        type Cost = i32;

        fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
            if state.0 < self.target {
                vec![Inc].into_iter()
            } else {
                vec![].into_iter()
            }
        }

        fn result(&self, state: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (Counter(state.0 + 1), 1)
        }
    }

    impl Utility for CountTo {
        fn heuristic(&self, state: &Self::State) -> Self::Cost {
            self.target - state.0
        }
    }

    impl SuitableState for CountTo {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.0 >= self.target
        }
    }

    fn run(explorer: &mut BFSExplorer<CountTo>) {
        let init = Counter(0);
        let result = explorer.search(init);
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        assert_eq!(actions.len() as i32, 4);
        assert!(result.state.is_some());
        eprintln!("CountTo BFS: iter={} t={:?}", result.n_iter, result.total_time);
    }

    #[test]
    fn test_count_bfs() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);
        run(&mut explorer);
    }

    #[test]
    fn test_count_dfs() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&problem, &arena);
        let result = explorer.search(Counter(0));
        assert!(result.actions.is_some());
        assert_eq!(result.actions.unwrap().len(), 4);
    }

    #[test]
    fn test_count_min_cost() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = MinCostExplorer::new(&problem, &arena);
        let result = explorer.search(Counter(0));
        assert!(result.actions.is_some());
        assert_eq!(result.actions.unwrap().len(), 4);
    }

    #[test]
    fn test_count_best_first() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = BestFirstGreedyExplorer::new(&problem, &arena);
        let result = explorer.search(Counter(0));
        assert!(result.actions.is_some());
        assert_eq!(result.actions.unwrap().len(), 4);
    }

    #[test]
    fn test_count_a_star() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = AStarExplorer::new(&problem, &arena);
        let result = explorer.search(Counter(0));
        assert!(result.actions.is_some());
        assert_eq!(result.actions.unwrap().len(), 4);
    }

    #[test]
    fn test_count_dfs_iterative() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&problem, &arena);
        let result = explorer.iterative_search(Counter(0), 10);
        assert!(result.actions.is_some());
        assert_eq!(result.actions.unwrap().len(), 4);
    }

    #[test]
    fn test_count_max_depth_exact() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);
        let result = explorer.search_with_max_depth(Counter(0), 4);
        assert!(result.actions.is_some());
    }

    #[test]
    fn test_count_max_depth_too_shallow() {
        let problem = CountTo { target: 4 };
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);
        let result = explorer.search_with_max_depth(Counter(0), 2);
        assert!(result.actions.is_none());
    }
}

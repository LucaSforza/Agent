#[cfg(test)]
mod tests {
    use agent::problem::{CostructSolution, Problem, SuitableState, Utility};
    use agent::statexplorer::resolver::{
        AStarExplorer, BFSExplorer, DFSExplorer, MinCostExplorer,
    };
    use bumpalo::Bump;

    // Problem: already at goal (empty actions).
    #[derive(Clone, PartialEq, Eq, Hash)]
    struct EmptyState;

    impl std::fmt::Debug for EmptyState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Empty")
        }
    }

    impl Default for EmptyState {
        fn default() -> Self {
            Self
        }
    }

    struct AlreadyAtGoal;

    impl Problem for AlreadyAtGoal {
        type State = EmptyState;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct DoNothing;

    impl CostructSolution for AlreadyAtGoal {
        type Action = DoNothing;
        type Cost = i32;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            std::iter::empty()
        }

        fn result(&self, state: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (EmptyState, 0)
        }
    }

    impl Utility for AlreadyAtGoal {
        fn heuristic(&self, _: &Self::State) -> Self::Cost {
            0
        }
    }

    impl SuitableState for AlreadyAtGoal {
        fn is_suitable(&self, _: &Self::State) -> bool {
            true
        }
    }

    #[test]
    fn test_already_at_goal_bfs() {
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&AlreadyAtGoal, &arena);
        let result = explorer.search(EmptyState);
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        assert!(actions.is_empty(), "goal state should give empty plan");
        assert!(result.state.is_some());
        eprintln!("AlreadyAtGoal BFS: iter={}", result.n_iter);
    }

    #[test]
    fn test_already_at_goal_dfs() {
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&AlreadyAtGoal, &arena);
        let result = explorer.search(EmptyState);
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        assert!(actions.is_empty());
    }

    #[test]
    fn test_already_at_goal_min_cost() {
        let arena = Bump::new();
        let mut explorer = MinCostExplorer::new(&AlreadyAtGoal, &arena);
        let result = explorer.search(EmptyState);
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        assert!(actions.is_empty());
    }

    #[test]
    fn test_already_at_goal_a_star() {
        let arena = Bump::new();
        let mut explorer = AStarExplorer::new(&AlreadyAtGoal, &arena);
        let result = explorer.search(EmptyState);
        assert!(result.actions.is_some());
        assert!(result.actions.unwrap().is_empty());
    }

    // Problem with zero actions from initial state (not a goal).
    struct NoActions;

    #[derive(Clone, PartialEq, Eq, Hash, Default)]
    struct Stuck;

    impl std::fmt::Debug for Stuck {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Stuck")
        }
    }

    impl Problem for NoActions {
        type State = Stuck;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Impossible;

    impl CostructSolution for NoActions {
        type Action = Impossible;
        type Cost = i32;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            std::iter::empty()
        }

        fn result(&self, _: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (Stuck, 0)
        }
    }

    impl Utility for NoActions {
        fn heuristic(&self, _: &Self::State) -> Self::Cost {
            10
        }
    }

    impl SuitableState for NoActions {
        fn is_suitable(&self, _: &Self::State) -> bool {
            false
        }
    }

    #[test]
    fn test_no_actions_bfs_returns_none() {
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&NoActions, &arena);
        let result = explorer.search(Stuck);
        assert!(result.actions.is_none(), "no path to goal");
        assert!(result.state.is_none());
        eprintln!("NoActions BFS: iter={}", result.n_iter);
    }

    #[test]
    fn test_no_actions_dfs_returns_none() {
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&NoActions, &arena);
        let result = explorer.search(Stuck);
        assert!(result.actions.is_none());
    }

    // Single state problem (auto-satisfying).
    struct SingleState;

    #[derive(Clone, PartialEq, Eq, Hash, Default)]
    struct OnlyState;

    impl std::fmt::Debug for OnlyState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Only")
        }
    }

    impl Problem for SingleState {
        type State = OnlyState;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Nothing;

    impl CostructSolution for SingleState {
        type Action = Nothing;
        type Cost = i32;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            std::iter::empty()
        }

        fn result(&self, _: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
            (OnlyState, 0)
        }
    }

    impl Utility for SingleState {
        fn heuristic(&self, _: &Self::State) -> Self::Cost {
            0
        }
    }

    impl SuitableState for SingleState {
        fn is_suitable(&self, _: &Self::State) -> bool {
            true
        }
    }

    #[test]
    fn test_single_state_search_returns_immediately() {
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&SingleState, &arena);
        let result = explorer.search(OnlyState);
        assert!(result.actions.is_some());
        assert!(result.actions.unwrap().is_empty());
        assert!(result.state.is_some());
        eprintln!("SingleState BFS: iter={}", result.n_iter);
    }

    // Multi-path graph: two paths to goal, cheaper one is longer.
    // State: position index 0..4. Goal: position 4.
    // Path 1: 0 -> 1 -> 4 (cost 10 per step = total 20)
    // Path 2: 0 -> 2 -> 3 -> 4 (cost 1 per step = total 3)
    // A*/MinCost should find path 2 (cheaper total cost).

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct Pos(usize);

    impl std::fmt::Debug for Pos {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Pos({})", self.0)
        }
    }

    impl Default for Pos {
        fn default() -> Self {
            Self(0)
        }
    }

    struct CheapLongPath;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Edge {
        A,
        B,
    }

    impl Problem for CheapLongPath {
        type State = Pos;
    }

    impl CostructSolution for CheapLongPath {
        type Action = Edge;
        type Cost = i32;

        fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
            match state.0 {
                0 => vec![Edge::A, Edge::B].into_iter(),
                1 => vec![Edge::A].into_iter(),
                2 => vec![Edge::A].into_iter(),
                3 => vec![Edge::A].into_iter(),
                _ => vec![].into_iter(),
            }
        }

        fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
            match (state.0, action) {
                (0, Edge::A) => (Pos(1), 10),
                (0, Edge::B) => (Pos(2), 1),
                (1, Edge::A) => (Pos(4), 10),
                (2, Edge::A) => (Pos(3), 1),
                (3, Edge::A) => (Pos(4), 1),
                _ => (state.clone(), 0),
            }
        }
    }

    impl Utility for CheapLongPath {
        fn heuristic(&self, state: &Self::State) -> Self::Cost {
            if state.0 == 4 {
                0
            } else {
                1
            }
        }
    }

    impl SuitableState for CheapLongPath {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.0 == 4
        }
    }

    #[test]
    fn test_min_cost_picks_cheaper_path() {
        let arena = Bump::new();
        let mut explorer = MinCostExplorer::new(&CheapLongPath, &arena);
        let result = explorer.search(Pos(0));
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        // Cheaper path: B -> A -> A (0->2->3->4), 3 actions
        assert_eq!(
            actions.len(),
            3,
            "MinCost should pick the cheaper 3-step path, got {:?}",
            actions
        );
        eprintln!("MinCost multi-path: actions={:?}", actions);
    }

    #[test]
    fn test_a_star_picks_cheaper_path() {
        let arena = Bump::new();
        let mut explorer = AStarExplorer::new(&CheapLongPath, &arena);
        let result = explorer.search(Pos(0));
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        assert_eq!(
            actions.len(),
            3,
            "A* should pick the cheaper 3-step path, got {:?}",
            actions
        );
        eprintln!("AStar multi-path: actions={:?}", actions);
    }

    #[test]
    fn test_bfs_picks_shortest_path() {
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&CheapLongPath, &arena);
        let result = explorer.search(Pos(0));
        assert!(result.actions.is_some());
        let actions = result.actions.unwrap();
        // BFS picks shortest path: A -> A (0->1->4), 2 actions
        assert_eq!(actions.len(), 2, "BFS should pick shortest path");
        eprintln!("BFS multi-path: actions={:?}", actions);
    }

    // Cyclic graph: edge cases for explored set.
    #[derive(Clone, PartialEq, Eq, Hash)]
    struct Cyclic(i32);

    impl std::fmt::Debug for Cyclic {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "C({})", self.0)
        }
    }

    impl Default for Cyclic {
        fn default() -> Self {
            Self(0)
        }
    }

    struct CyclicProblem;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Move {
        Fwd,
        Back,
    }

    impl Problem for CyclicProblem {
        type State = Cyclic;
    }

    impl CostructSolution for CyclicProblem {
        type Action = Move;
        type Cost = i32;

        fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
            if state.0 < 0 || state.0 > 5 {
                vec![].into_iter()
            } else {
                vec![Move::Fwd, Move::Back].into_iter()
            }
        }

        fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
            match action {
                Move::Fwd => (Cyclic(state.0 + 1), 1),
                Move::Back => (Cyclic(state.0 - 1), 1),
            }
        }
    }

    impl Utility for CyclicProblem {
        fn heuristic(&self, state: &Self::State) -> Self::Cost {
            (5 - state.0).abs()
        }
    }

    impl SuitableState for CyclicProblem {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.0 == 5
        }
    }

    #[test]
    fn test_cyclic_graph_bfs_terminates() {
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&CyclicProblem, &arena);
        let result = explorer.search(Cyclic(0));
        assert!(result.actions.is_some(), "BFS should find path through cycle");
        eprintln!("Cyclic BFS: actions={:?}", result.actions.unwrap());
    }

    #[test]
    fn test_cyclic_graph_dfs_terminates() {
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&CyclicProblem, &arena);
        let result = explorer.search(Cyclic(0));
        assert!(result.actions.is_some(), "DFS should find path through cycle");
        eprintln!("Cyclic DFS: iter={}", result.n_iter);
    }
}

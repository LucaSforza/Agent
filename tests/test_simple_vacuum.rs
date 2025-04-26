#[cfg(test)]
mod tests {
    use std::vec;

    use agent::{
        problem::{CostructSolution, Problem, SuitableState, Utility},
        statexplorer::resolver::{BFSExplorer, DFSExplorer},
    };
    // use frontier::DequeFrontier;

    #[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
    enum Action {
        Left,
        Right,
        Suck,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
    enum Position {
        Left,
        Right,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
    enum TailState {
        Clean,
        Dirty,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct HouseState {
        pos: Position,
        right_state: TailState,
        left_state: TailState,
    }

    impl HouseState {
        fn from_parts(pos: Position, right_state: TailState, left_state: TailState) -> Self {
            HouseState {
                pos,
                right_state: right_state,
                left_state: left_state,
            }
        }
    }

    use bumpalo::Bump;
    use ordered_float::OrderedFloat;

    struct CleanProblem {}

    impl CleanProblem {
        fn new() -> Self {
            Self {}
        }
    }

    impl Problem for CleanProblem {
        type State = HouseState;
    }

    impl CostructSolution for CleanProblem {
        type Action = Action;
        type Cost = OrderedFloat<f64>;

        fn executable_actions(&self, _: &Self::State) -> impl Iterator<Item = Self::Action> {
            vec![Action::Left, Action::Right, Action::Suck].into_iter()
        }

        fn result(
            &self,
            state: &Self::State,
            action: &Self::Action,
        ) -> (Self::State, OrderedFloat<f64>) {
            let result_state = match action {
                Action::Left => {
                    HouseState::from_parts(Position::Left, state.right_state, state.left_state)
                }
                Action::Right => {
                    HouseState::from_parts(Position::Right, state.right_state, state.left_state)
                }
                Action::Suck => match state.pos {
                    Position::Left => {
                        HouseState::from_parts(state.pos, state.right_state, TailState::Clean)
                    }
                    Position::Right => {
                        HouseState::from_parts(state.pos, TailState::Clean, state.left_state)
                    }
                },
            };
            (result_state, 1.into())
        }
    }

    impl Utility for CleanProblem {
        fn heuristic(&self, state: &Self::State) -> OrderedFloat<f64> {
            let mut result = 0;

            if state.left_state == TailState::Clean {
                result += 1;
            }

            if state.right_state == TailState::Clean {
                result += 1;
            }

            return result.into();
        }
    }

    impl SuitableState for CleanProblem {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.left_state == TailState::Clean && state.right_state == TailState::Clean
        }
    }

    #[test]
    fn test_bfs_clean_left_dirty_right() {
        let problem = CleanProblem::new();
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);
        let init_state = HouseState::from_parts(Position::Left, TailState::Clean, TailState::Dirty);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![Action::Suck]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_bfs_dirty_left_clean_right() {
        let problem = CleanProblem::new();
        let init_state =
            HouseState::from_parts(Position::Right, TailState::Dirty, TailState::Clean);
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);

        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![Action::Suck]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_bfs_both_dirty() {
        let problem = CleanProblem::new();
        let init_state = HouseState::from_parts(Position::Left, TailState::Dirty, TailState::Dirty);
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![Action::Suck, Action::Right, Action::Suck]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_bfs_both_clean() {
        let problem = CleanProblem::new();
        let init_state =
            HouseState::from_parts(Position::Right, TailState::Clean, TailState::Clean);
        let arena = Bump::new();
        let mut explorer = BFSExplorer::new(&problem, &arena);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_dfs_clean_left_dirty_right() {
        let problem = CleanProblem::new();
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&problem, &arena);
        let init_state = HouseState::from_parts(Position::Left, TailState::Clean, TailState::Dirty);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![Action::Suck]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_dfs_dirty_left_clean_right() {
        let problem = CleanProblem::new();
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&problem, &arena);
        let init_state =
            HouseState::from_parts(Position::Right, TailState::Dirty, TailState::Clean);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![Action::Suck]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_dfs_both_dirty() {
        let problem = CleanProblem::new();
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&problem, &arena);
        let init_state = HouseState::from_parts(Position::Left, TailState::Dirty, TailState::Dirty);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![Action::Suck, Action::Right, Action::Suck]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_dfs_both_clean() {
        let problem = CleanProblem::new();
        let arena = Bump::new();
        let mut explorer = DFSExplorer::new(&problem, &arena);
        let init_state =
            HouseState::from_parts(Position::Right, TailState::Clean, TailState::Clean);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        assert_eq!(res, vec![]);
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }
}

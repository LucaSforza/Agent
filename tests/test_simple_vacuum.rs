#[cfg(test)]
mod tests {
    use agent::agent::WorldState;
    use agent::explorer::{BFSExplorer, DFSExplorer};
    // use frontier::DequeFrontier;

    #[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
    enum Action {
        Left,
        Right,
        Suck,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Copy)]
    enum Position {
        Left,
        Right,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Copy)]
    enum TailState {
        Clean,
        Dirty,
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
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

    impl WorldState<Action> for HouseState {
        type Iter = std::vec::IntoIter<Action>;

        fn executable_actions(&self) -> Self::Iter {
            vec![Action::Left, Action::Right, Action::Suck].into_iter()
        }

        fn result(&self, action: &Action) -> (Self, f64) {
            let result_state = match action {
                Action::Left => Self::from_parts(Position::Left, self.right_state, self.left_state),
                Action::Right => {
                    Self::from_parts(Position::Right, self.right_state, self.left_state)
                }
                Action::Suck => match self.pos {
                    Position::Left => {
                        Self::from_parts(self.pos, self.right_state, TailState::Clean)
                    }
                    Position::Right => {
                        Self::from_parts(self.pos, TailState::Clean, self.left_state)
                    }
                },
            };
            (result_state, 1.0)
        }

        fn is_goal(&self) -> bool {
            return self.left_state == TailState::Clean && self.right_state == TailState::Clean;
        }

        fn heuristic(&self) -> f64 {
            let mut result = 0.0;

            if self.left_state == TailState::Clean {
                result += 1.0;
            }

            if self.right_state == TailState::Clean {
                result += 1.0;
            }

            return result;
        }
    }

    #[test]
    fn test_bfs_clean_left_dirty_right() {
        let mut explorer = BFSExplorer::<HouseState, Action>::new();
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
        let init_state =
            HouseState::from_parts(Position::Right, TailState::Dirty, TailState::Clean);
        let mut explorer = BFSExplorer::<HouseState, Action>::new();

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
        let init_state = HouseState::from_parts(Position::Left, TailState::Dirty, TailState::Dirty);
        let mut explorer = BFSExplorer::<HouseState, Action>::new();
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
        let init_state =
            HouseState::from_parts(Position::Right, TailState::Clean, TailState::Clean);
        let mut explorer = BFSExplorer::<HouseState, Action>::new();
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
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
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
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
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
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
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
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use agent::{
        problem::{CostructSolution, Problem, SuitableState, Utility},
        statexplorer::resolver::{
            AStarExplorer, BFSExplorer, BestFirstGreedyExplorer, DFSExplorer, MinCostExplorer,
        },
    };
    // use frontier::DequeFrontier;

    #[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
    enum Action {
        Left,
        Right,
        Down,
        Up,
        Suck,
        Nothing,
    }

    impl Default for Action {
        fn default() -> Self {
            Self::Nothing
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct Pos {
        x: usize,
        y: usize,
    }

    impl Pos {
        fn new(x: usize, y: usize) -> Self {
            Self { x: x, y: y }
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct HouseState {
        pos: Pos,
        where_dirty: Rc<Vec<Pos>>,
    }

    impl HouseState {
        fn with_dirty(x: usize, y: usize, where_dirty: Vec<Pos>) -> Self {
            Self {
                pos: Pos { x: x, y: y },
                where_dirty: where_dirty.into(),
            }
        }

        fn new_position(&self, x: usize, y: usize) -> Self {
            Self {
                pos: Pos { x: x, y: y },
                where_dirty: self.where_dirty.clone(),
            }
        }

        fn clean(&self, new_dirty: Vec<Pos>) -> Self {
            Self {
                pos: self.pos.clone(),
                where_dirty: new_dirty.into(),
            }
        }

        fn is_dirty(&self) -> bool {
            let mut result = false;
            for pos in self.where_dirty.as_ref() {
                if *pos == self.pos {
                    result = true;
                    break;
                }
            }
            return result;
        }
    }

    #[derive(PartialEq, Eq)]
    struct CleanProblem {
        rows: usize,
        cols: usize,
    }

    impl CleanProblem {
        fn new(rows: usize, cols: usize) -> Self {
            Self {
                rows: rows,
                cols: cols,
            }
        }
    }

    use ordered_float::OrderedFloat;

    impl Problem for CleanProblem {
        type State = HouseState;
    }

    impl CostructSolution for CleanProblem {
        type Action = Action;
        type Cost = OrderedFloat<f64>;

        fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
            let mut actions = Vec::with_capacity(6); // TODO: change this
            if self.is_suitable(state) {
                actions.push(Action::Nothing);
            }
            if state.is_dirty() {
                actions.push(Action::Suck);
            }
            if state.pos.x != 0 {
                actions.push(Action::Left);
            }
            if state.pos.x < self.rows - 1 {
                actions.push(Action::Right);
            }
            if state.pos.y != 0 {
                actions.push(Action::Up);
            }
            if state.pos.y < self.cols - 1 {
                actions.push(Action::Down);
            }
            actions.into_iter()
        }

        fn result(
            &self,
            state: &Self::State,
            action: &Self::Action,
        ) -> (Self::State, OrderedFloat<f64>) {
            let result_state = match action {
                Action::Left => state.new_position(state.pos.x - 1, state.pos.y),
                Action::Right => state.new_position(state.pos.x + 1, state.pos.y),
                Action::Suck => {
                    let new_dirty = state.where_dirty.as_ref().clone();
                    let new_dirty = new_dirty.into_iter().filter(|e| *e != state.pos).collect();
                    state.clean(new_dirty)
                }
                Action::Nothing => state.clone(),
                Action::Down => state.new_position(state.pos.x, state.pos.y + 1),
                Action::Up => state.new_position(state.pos.x, state.pos.y - 1),
            };
            (result_state, 1.into())
        }
    }

    impl Utility for CleanProblem {
        fn heuristic(&self, state: &Self::State) -> OrderedFloat<f64> {
            (state.where_dirty.len() as f64).into()
        }
    }

    impl SuitableState for CleanProblem {
        fn is_suitable(&self, state: &Self::State) -> bool {
            state.where_dirty.is_empty()
        }
    }

    #[test]
    fn test_vacuum_bfs() {
        let problem = CleanProblem::new(32, 32);
        let init_state = HouseState::with_dirty(
            3,
            2,
            vec![
                Pos::new(10, 15),
                Pos::new(29, 14),
                Pos::new(13, 15),
                Pos::new(1, 29),
                Pos::new(31, 31),
            ],
        );
        let mut explorer = BFSExplorer::new(problem);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_vacuum_bfs_clean() {
        let problem = CleanProblem::new(32, 32);
        let init_state = HouseState::with_dirty(3, 2, vec![]);
        let mut explorer = BFSExplorer::new(problem);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_vacuum_dfs() {
        let problem = CleanProblem::new(32, 32);
        let mut explorer = DFSExplorer::new(problem);
        let init_state = HouseState::with_dirty(
            3,
            2,
            vec![
                Pos::new(10, 15),
                Pos::new(29, 14),
                Pos::new(13, 15),
                Pos::new(1, 29),
                Pos::new(31, 31),
            ],
        );
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_vacuum_dfs_clean() {
        let problem = CleanProblem::new(32, 32);
        let mut explorer = DFSExplorer::new(problem);
        let init_state = HouseState::with_dirty(3, 2, vec![]);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }

    #[test]
    fn test_vacuum_dfs_esposito() {
        let pos = vec![
            Pos::new(0, 1),
            Pos::new(0, 2),
            Pos::new(0, 3),
            Pos::new(1, 1),
            Pos::new(1, 3),
            Pos::new(2, 3),
            Pos::new(2, 4),
            Pos::new(3, 0),
            Pos::new(3, 1),
            Pos::new(3, 2),
            Pos::new(3, 3),
            Pos::new(4, 0),
            Pos::new(4, 2),
        ];
        let problem = CleanProblem::new(5, 5);
        let init_state = HouseState::with_dirty(4, 3, pos);
        let mut explorer = DFSExplorer::new(problem);
        let sresult = explorer.search(init_state);
        assert!(sresult.actions.is_some());
        // let actions = sresult.actions.clone().unwrap();
        eprintln!("{}", sresult);
    }

    #[test]
    fn test_vacuum_bfs_esposito() {
        let pos = vec![
            Pos::new(0, 1),
            Pos::new(0, 2),
            Pos::new(0, 3),
            Pos::new(1, 1),
            Pos::new(1, 3),
            Pos::new(2, 0),
            Pos::new(2, 1),
            Pos::new(3, 1),
            Pos::new(3, 2),
            Pos::new(3, 3),
            Pos::new(3, 4),
            Pos::new(4, 2),
            Pos::new(4, 4),
        ];
        let expected_result = vec![
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Left,
            Action::Up,
            Action::Suck,
            Action::Left,
            Action::Left,
            Action::Suck,
            Action::Left,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Right,
            Action::Down,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Right,
            Action::Suck,
        ];
        let problem = CleanProblem::new(5, 5);
        let init_state = HouseState::with_dirty(3, 4, pos);
        let mut explorer = BFSExplorer::new(problem);
        let sresult = explorer.search(init_state);
        assert!(sresult.actions.is_some());
        let actions = sresult.actions.clone().unwrap();
        assert_eq!(actions, expected_result);
        eprintln!("{}", sresult);
    }

    #[test]
    fn test_vacuum_min_cost_esposito() {
        let pos = vec![
            Pos::new(0, 1),
            Pos::new(0, 2),
            Pos::new(0, 3),
            Pos::new(1, 1),
            Pos::new(1, 3),
            Pos::new(2, 0),
            Pos::new(2, 1),
            Pos::new(3, 1),
            Pos::new(3, 2),
            Pos::new(3, 3),
            Pos::new(3, 4),
            Pos::new(4, 2),
            Pos::new(4, 4),
        ];

        let expected_result = vec![
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Up,
            Action::Up,
            Action::Suck,
            Action::Left,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Up,
            Action::Up,
            Action::Suck,
            Action::Left,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Down,
            Action::Left,
            Action::Suck,
            Action::Left,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Right,
            Action::Suck,
        ];
        let problem = CleanProblem::new(5, 5);
        let init_state = HouseState::with_dirty(3, 4, pos);
        let mut explorer = MinCostExplorer::new(problem);
        let sresult = explorer.search(init_state);
        assert!(sresult.actions.is_some());
        let actions = sresult.actions.clone().unwrap();
        assert_eq!(actions, expected_result);
        eprintln!("{}", sresult);
    }

    #[test]
    fn test_vacuum_iterative_esposito() {
        let pos = vec![
            Pos::new(0, 1),
            Pos::new(0, 2),
            Pos::new(0, 3),
            Pos::new(1, 1),
            Pos::new(1, 3),
            Pos::new(2, 0),
            Pos::new(2, 1),
            Pos::new(3, 1),
            Pos::new(3, 2),
            Pos::new(3, 3),
            Pos::new(3, 4),
            Pos::new(4, 2),
            Pos::new(4, 4),
        ];

        let expected_result = vec![
            Action::Up,
            Action::Up,
            Action::Right,
            Action::Suck,
            Action::Down,
            Action::Down,
            Action::Suck,
            Action::Up,
            Action::Up,
            Action::Left,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Down,
            Action::Left,
            Action::Left,
            Action::Up,
            Action::Up,
            Action::Up,
            Action::Left,
            Action::Suck,
            Action::Down,
            Action::Down,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Down,
            Action::Down,
            Action::Right,
            Action::Right,
            Action::Right,
            Action::Suck,
            Action::Up,
            Action::Up,
            Action::Up,
            Action::Suck,
            Action::Left,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Down,
            Action::Left,
            Action::Suck,
            Action::Down,
            Action::Down,
            Action::Suck,
        ];
        let problem = CleanProblem::new(5, 5);
        let init_state = HouseState::with_dirty(3, 4, pos);
        let mut explorer = DFSExplorer::new(problem);
        let sresult = explorer.iterative_search(init_state, 300);
        assert!(sresult.actions.is_some());
        let actions = sresult.actions.clone().unwrap();
        assert_eq!(actions, expected_result);
        eprintln!("{}", sresult);
    }

    #[test]
    fn test_vacuum_best_first_esposito() {
        let pos = vec![
            Pos::new(0, 1),
            Pos::new(0, 2),
            Pos::new(0, 3),
            Pos::new(1, 1),
            Pos::new(1, 3),
            Pos::new(2, 0),
            Pos::new(2, 1),
            Pos::new(3, 1),
            Pos::new(3, 2),
            Pos::new(3, 3),
            Pos::new(3, 4),
            Pos::new(4, 2),
            Pos::new(4, 4),
        ];
        let problem = CleanProblem::new(5, 5);
        let init_state = HouseState::with_dirty(3, 4, pos);
        let mut explorer = BestFirstGreedyExplorer::new(problem);
        let sresult = explorer.search(init_state);
        assert!(sresult.actions.is_some());
        // let actions = sresult.actions.clone().unwrap();
        // assert_eq!(actions, expected_result);
        eprintln!("{}", sresult);
    }

    #[test]
    fn test_vacuum_a_star_esposito() {
        let pos = vec![
            Pos::new(0, 1),
            Pos::new(0, 2),
            Pos::new(0, 3),
            Pos::new(1, 1),
            Pos::new(1, 3),
            Pos::new(2, 0),
            Pos::new(2, 1),
            Pos::new(3, 1),
            Pos::new(3, 2),
            Pos::new(3, 3),
            Pos::new(3, 4),
            Pos::new(4, 2),
            Pos::new(4, 4),
        ];

        let expected_result = vec![
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Left,
            Action::Left,
            Action::Suck,
            Action::Left,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Up,
            Action::Suck,
            Action::Right,
            Action::Down,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Right,
            Action::Suck,
            Action::Down,
            Action::Down,
            Action::Suck,
        ];
        let problem = CleanProblem::new(5, 5);
        let init_state = HouseState::with_dirty(3, 4, pos);
        let mut explorer = AStarExplorer::new(problem);
        let sresult = explorer.search(init_state);
        assert!(sresult.actions.is_some());
        let actions = sresult.actions.clone().unwrap();
        assert_eq!(actions.len(), expected_result.len());
        eprintln!("{}", sresult);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use agent::agent::WorldState;
    use agent::explorer::{
        AStarExplorer, BFSExplorer, BestFirstGreedyExplorer, DFSExplorer, MinCostExplorer,
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
        rows: usize,
        cols: usize,
        where_dirty: Rc<Vec<Pos>>,
    }

    impl HouseState {
        fn with_dirty(
            x: usize,
            y: usize,
            cols_len: usize,
            rows_len: usize,
            where_dirty: Vec<Pos>,
        ) -> Self {
            assert!(x < cols_len && y < rows_len);
            Self {
                pos: Pos { x: x, y: y },
                rows: rows_len,
                cols: cols_len,
                where_dirty: where_dirty.into(),
            }
        }

        fn new_position(&self, x: usize, y: usize) -> Self {
            assert!(x < self.rows && y < self.cols);
            Self {
                pos: Pos { x: x, y: y },
                rows: self.rows,
                cols: self.cols,
                where_dirty: self.where_dirty.clone(),
            }
        }

        fn clean(&self, new_dirty: Vec<Pos>) -> Self {
            Self {
                pos: self.pos.clone(),
                rows: self.rows,
                cols: self.rows,
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

    impl WorldState<Action> for HouseState {
        type Iter = std::vec::IntoIter<Action>;

        fn executable_actions(&self) -> Self::Iter {
            let mut actions = Vec::with_capacity(6); // TODO: change this
            if self.is_goal() {
                actions.push(Action::Nothing);
            }
            if self.is_dirty() {
                actions.push(Action::Suck);
            }
            if self.pos.x != 0 {
                actions.push(Action::Left);
            }
            if self.pos.x < self.rows - 1 {
                actions.push(Action::Right);
            }
            if self.pos.y != 0 {
                actions.push(Action::Up);
            }
            if self.pos.y < self.cols - 1 {
                actions.push(Action::Down);
            }
            actions.into_iter()
        }

        fn result(&self, action: &Action) -> (Self, f64) {
            let result_state = match action {
                Action::Left => self.new_position(self.pos.x - 1, self.pos.y),
                Action::Right => self.new_position(self.pos.x + 1, self.pos.y),
                Action::Suck => {
                    let new_dirty = self.where_dirty.as_ref().clone();
                    let new_dirty = new_dirty.into_iter().filter(|e| *e != self.pos).collect();
                    self.clean(new_dirty)
                }
                Action::Nothing => self.clone(),
                Action::Down => self.new_position(self.pos.x, self.pos.y + 1),
                Action::Up => self.new_position(self.pos.x, self.pos.y - 1),
            };
            (result_state, 1.0)
        }

        fn is_goal(&self) -> bool {
            return self.where_dirty.is_empty();
        }

        fn heuristic(&self) -> f64 {
            let mut result = 0.0;
            let mut pos: &Pos = &self.pos;
            for dirty_pos in self.where_dirty.iter() {
                result += ((dirty_pos.x as isize - pos.x as isize).abs()
                    + (dirty_pos.y as isize - pos.y as isize).abs()
                    + 1) as f64;
                pos = dirty_pos;
            }
            return result;
        }
    }

    #[test]
    fn test_vacuum_bfs() {
        let init_state = HouseState::with_dirty(
            3,
            2,
            32,
            32,
            vec![
                Pos::new(10, 15),
                Pos::new(29, 14),
                Pos::new(13, 15),
                Pos::new(1, 29),
                Pos::new(31, 31),
            ],
        );
        let mut explorer = BFSExplorer::<HouseState, Action>::new();
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
        let init_state = HouseState::with_dirty(3, 2, 32, 32, vec![]);
        let mut explorer = BFSExplorer::<HouseState, Action>::new();
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
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
        let init_state = HouseState::with_dirty(
            3,
            2,
            32,
            32,
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
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
        let init_state = HouseState::with_dirty(3, 2, 32, 32, vec![]);
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

        let init_state = HouseState::with_dirty(4, 3, 5, 5, pos);
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
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

        let init_state = HouseState::with_dirty(3, 4, 5, 5, pos);
        let mut explorer = BFSExplorer::<HouseState, Action>::new();
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
            Action::Left,
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
            Action::Down,
            Action::Right,
            Action::Suck,
            Action::Down,
            Action::Suck,
            Action::Right,
            Action::Suck,
        ];

        let init_state = HouseState::with_dirty(3, 4, 5, 5, pos);
        let mut explorer = MinCostExplorer::<HouseState, Action>::new();
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

        let init_state = HouseState::with_dirty(3, 4, 5, 5, pos);
        let mut explorer = DFSExplorer::<HouseState, Action>::new();
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

        let init_state = HouseState::with_dirty(3, 4, 5, 5, pos);
        let mut explorer = BestFirstGreedyExplorer::<HouseState, Action>::new();
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

        let init_state = HouseState::with_dirty(3, 4, 5, 5, pos);
        let mut explorer = AStarExplorer::<HouseState, Action>::new();
        let sresult = explorer.search(init_state);
        assert!(sresult.actions.is_some());
        let actions = sresult.actions.clone().unwrap();
        assert_eq!(actions, expected_result);
        eprintln!("{}", sresult);
    }
}

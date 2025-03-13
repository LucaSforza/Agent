#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use agent::agent::WorldState;
    use agent::explorer::{BFSExplorer, DFSExplorer};
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

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct Pos {
        x: usize,
        y: usize,
    }

    impl Pos {
        fn new(x: usize, y: usize) -> Self {
            Self { x: x, y: y }
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
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

        fn result(&self, action: &Action) -> (Self, f32) {
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
            return self.where_dirty.len() == 0;
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
        let explorer = BFSExplorer::<HouseState, Action>::new();
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
        let explorer = BFSExplorer::<HouseState, Action>::new();
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
        let explorer = DFSExplorer::<HouseState, Action>::new();
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
        let explorer = DFSExplorer::<HouseState, Action>::new();
        let init_state = HouseState::with_dirty(3, 2, 32, 32, vec![]);
        let result = explorer.search(init_state);
        assert!(result.actions.is_some());
        let res = result.actions.unwrap();
        eprintln!(
            "Result: {:?}, time: {:?}, n_iter: {}",
            res, result.total_time, result.n_iter
        );
    }
}

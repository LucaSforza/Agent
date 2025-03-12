mod agent;

#[cfg(test)]
mod tests {
    use super::*;

    use agent::search;
    use agent::{Frontier, Goal, Node};
    use std::collections::VecDeque;
    use std::rc::Rc;

    #[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
    enum Action {
        Left,
        Right,
        Suck,
        Nothing,
    }

    impl Default for Action {
        fn default() -> Self {
            Self::Nothing
        }
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

    impl agent::WorldState<Action> for HouseState {
        type Iter = std::vec::IntoIter<Action>;

        fn executable_actions(&self) -> Self::Iter {
            vec![Action::Left, Action::Right, Action::Suck, Action::Nothing].into_iter()
        }

        fn result(&self, action: &Action) -> Self {
            match action {
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
                Action::Nothing => self.clone(),
            }
        }
    }

    struct HouseGoal {}

    impl Goal<HouseState> for HouseGoal {
        fn is_goal(&self, state: &HouseState) -> bool {
            return state.left_state == TailState::Clean && state.right_state == TailState::Clean;
        }
    }

    impl Frontier<HouseState, Action> for VecDeque<Rc<Node<HouseState, Action>>> {
        fn new() -> Self {
            Self::new()
        }

        fn add(&mut self, item: std::rc::Rc<agent::Node<HouseState, Action>>) {
            self.push_back(item);
        }

        fn pop(&mut self) -> Option<std::rc::Rc<agent::Node<HouseState, Action>>> {
            self.pop_front()
        }

        fn delete(&mut self, state: &HouseState) {
            if let Some(pos) = self.iter().position(|node| node.get_state() == state) {
                self.remove(pos);
            }
        }

        fn iter_mut(&mut self) -> Self::Iter<'_> {
            self.iter_mut()
        }

        type Iter<'a> = std::collections::vec_deque::IterMut<'a, Rc<Node<HouseState, Action>>>;
    }

    #[test]
    fn test_clean_left_dirty_right() {
        let result = search::<HouseState, Action, VecDeque<Rc<Node<HouseState, Action>>>>(
            HouseState::from_parts(Position::Left, TailState::Clean, TailState::Dirty),
            HouseGoal {},
        );
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res, vec![Action::Suck]);
        eprintln!("{:?}", res);
    }

    #[test]
    fn test_dirty_left_clean_right() {
        let result = search::<HouseState, Action, VecDeque<Rc<Node<HouseState, Action>>>>(
            HouseState::from_parts(Position::Right, TailState::Dirty, TailState::Clean),
            HouseGoal {},
        );
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res, vec![Action::Suck]);
        eprintln!("{:?}", res);
    }

    #[test]
    fn test_both_dirty() {
        let result = search::<HouseState, Action, VecDeque<Rc<Node<HouseState, Action>>>>(
            HouseState::from_parts(Position::Left, TailState::Dirty, TailState::Dirty),
            HouseGoal {},
        );
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res, vec![Action::Suck, Action::Right, Action::Suck]);
        eprintln!("{:?}", res);
    }

    #[test]
    fn test_both_clean() {
        let result = search::<HouseState, Action, VecDeque<Rc<Node<HouseState, Action>>>>(
            HouseState::from_parts(Position::Right, TailState::Clean, TailState::Clean),
            HouseGoal {},
        );
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res, vec![Action::Nothing]);
        eprintln!("{:?}", res);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use agent::{
        agent::WorldState,
        explorer::{
            AStarExplorer, BFSExplorer, BestFirstGreedyExplorer, DFSExplorer, MinCostExplorer,
        },
    };
    use ordered_float::OrderedFloat;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Action<'a> {
        goto: &'a str,
        cost: OrderedFloat<f64>,
    }

    impl<'a> Action<'a> {
        fn new(name: &'a str, cost: f64) -> Self {
            Self {
                goto: name,
                cost: cost.into(),
            }
        }
    }

    #[derive(Hash, PartialEq, Eq, Clone, Copy)]
    struct State<'a> {
        name: &'a str,
    }

    impl<'a> State<'a> {
        fn new(name: &'a str) -> Self {
            Self { name: name }
        }
    }

    impl Debug for State<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    impl<'a> WorldState<Action<'a>> for State<'a> {
        type Iter = std::vec::IntoIter<Action<'a>>;

        fn executable_actions(&self) -> Self::Iter {
            let mut result = Vec::with_capacity(3);
            match self.name {
                "A" => result.push(Action::new("B", 4.0)),
                "B" => {
                    result.push(Action::new("C", 3.0));
                    result.push(Action::new("G1", 9.0));
                }
                "C" => {
                    result.push(Action::new("F", 2.0));
                    result.push(Action::new("J", 5.0));
                    result.push(Action::new("S", 1.0));
                }
                "D" => {
                    result.push(Action::new("C", 3.0));
                    result.push(Action::new("E", 3.0));
                    result.push(Action::new("S", 8.0));
                }
                "E" => {
                    result.push(Action::new("G2", 7.0));
                }
                "F" => {
                    result.push(Action::new("D", 1.0));
                    result.push(Action::new("G2", 4.0));
                }
                "G1" => {}
                "G2" => {}
                "J" => {
                    result.push(Action::new("G1", 3.0));
                }
                "S" => {
                    result.push(Action::new("A", 2.0));
                    result.push(Action::new("B", 7.0));
                    result.push(Action::new("D", 5.0));
                }
                _ => panic!(),
            };
            result.into_iter()
        }

        fn result(&self, action: &Action<'a>) -> (Self, f64) {
            (Self { name: action.goto }, action.cost.into())
        }

        fn is_goal(&self) -> bool {
            self.name == "G1" || self.name == "G2"
        }

        fn heuristic(&self) -> f64 {
            match self.name {
                "A" => 9.0,
                "B" => 3.0,
                "C" => 2.0,
                "D" => 4.0,
                "E" => 5.0,
                "F" => 3.0,
                "G1" => 0.0,
                "G2" => 0.0,
                "J" => 1.0,
                "S" => 7.0,
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_a31_dfs() {
        let init_state = State::new("S");
        let mut explorer = DFSExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a31_bfs() {
        let init_state = State::new("S");
        let mut explorer = BFSExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a31_iter() {
        let init_state = State::new("S");
        let mut explorer = DFSExplorer::<State, Action>::with_low_v();
        let result = explorer.iterative_search(init_state, 40);
        println!("{}", result);
    }
    #[test]
    fn test_a31_min_cost() {
        let init_state = State::new("S");
        let mut explorer = MinCostExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a31_a_star() {
        let init_state = State::new("S");
        let mut explorer = AStarExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a31_best_first() {
        let init_state = State::new("S");
        let mut explorer = BestFirstGreedyExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
}

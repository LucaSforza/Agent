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
                "A" => {
                    result.push(Action::new("E", 1.0));
                    result.push(Action::new("H", 8.0));
                }
                "B" => {
                    result.push(Action::new("C", 2.0));
                    result.push(Action::new("I", 3.0));
                    result.push(Action::new("J", 5.0));
                }
                "C" => {
                    result.push(Action::new("S", 1.0));
                    result.push(Action::new("G2", 18.0));
                }
                "D" => {
                    result.push(Action::new("C", 2.0));
                }
                "E" => {
                    result.push(Action::new("D", 2.0));
                    result.push(Action::new("H", 7.0));
                }
                "G1" => {
                    result.push(Action::new("E", 2.0));
                }
                "G2" => {
                    result.push(Action::new("B", 15.0));
                }
                "J" => {
                    result.push(Action::new("G2", 12.0));
                }
                "S" => {
                    result.push(Action::new("A", 3.0));
                    result.push(Action::new("B", 3.0));
                    result.push(Action::new("D", 3.0));
                }
                "H" => {
                    result.push(Action::new("G1", 9.0));
                }
                "I" => {
                    result.push(Action::new("H", 4.0));
                    result.push(Action::new("A", 1.0));
                }
                x => panic!("{} not exists", x),
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
                "A" => 16.0,
                "B" => 16.0,
                "C" => 14.0,
                "D" => 17.0,
                "E" => 15.0,
                "G1" => 0.0,
                "G2" => 0.0,
                "I" => 12.0,
                "H" => 8.0,
                "J" => 10.0,
                "S" => 20.0,
                x => panic!("{} not exists", x),
            }
        }
    }

    #[test]
    fn test_a32_dfs() {
        let init_state = State::new("S");
        let mut explorer = DFSExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a32_bfs() {
        let init_state = State::new("S");
        let mut explorer = BFSExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a32_iter() {
        let init_state = State::new("S");
        let mut explorer = DFSExplorer::<State, Action>::with_low_v();
        let result = explorer.iterative_search(init_state, 40);
        println!("{}", result);
    }
    #[test]
    fn test_a32_min_cost() {
        let init_state = State::new("S");
        let mut explorer = MinCostExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a32_a_star() {
        let init_state = State::new("S");
        let mut explorer = AStarExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
    #[test]
    fn test_a32_best_first() {
        let init_state = State::new("S");
        let mut explorer = BestFirstGreedyExplorer::<State, Action>::with_low_v();
        let result = explorer.search(init_state);
        println!("{}", result);
    }
}

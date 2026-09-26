use agent::problem::{CostructSolution, Problem, SuitableState, Utility};
use agent::statexplorer::resolver::{bounded_search, GraphSearchAlgorithm, GraphSearchOutcome};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct State(u8);

struct ReopenProblem;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    ToA,
    ToB,
    BToA,
    AToGoal,
}

impl Problem for ReopenProblem {
    type State = State;
}

impl CostructSolution for ReopenProblem {
    type Action = Action;
    type Cost = i32;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        match state.0 {
            0 => vec![Action::ToA, Action::ToB],
            1 => vec![Action::AToGoal],
            2 => vec![Action::BToA],
            _ => vec![],
        }
        .into_iter()
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        match (state.0, action) {
            (0, Action::ToA) => (State(1), 5),
            (0, Action::ToB) => (State(2), 1),
            (1, Action::AToGoal) => (State(3), 10),
            (2, Action::BToA) => (State(1), 1),
            _ => unreachable!(),
        }
    }
}

impl Utility for ReopenProblem {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        match state.0 {
            // This admissible but inconsistent heuristic makes A pop before B.
            2 => 10,
            _ => 0,
        }
    }
}

impl SuitableState for ReopenProblem {
    fn is_suitable(&self, state: &Self::State) -> bool {
        state.0 == 3
    }
}

#[test]
fn astar_reopens_state_when_a_cheaper_path_arrives() {
    let result = bounded_search(&ReopenProblem, State(0), GraphSearchAlgorithm::AStar, 4);
    assert_eq!(
        result,
        GraphSearchOutcome::Solved {
            state: State(3),
            actions: vec![Action::ToB, Action::BToA, Action::AToGoal],
            expanded: 5,
        }
    );
}

#[test]
fn bounded_search_counts_initial_state_and_reports_exhaustion() {
    let result = bounded_search(&ReopenProblem, State(0), GraphSearchAlgorithm::AStar, 2);
    assert_eq!(result, GraphSearchOutcome::LimitReached { expanded: 1 });
}

#[test]
fn zero_budget_returns_limit_without_goal_check() {
    let result = bounded_search(&ReopenProblem, State(3), GraphSearchAlgorithm::AStar, 0);
    assert_eq!(result, GraphSearchOutcome::LimitReached { expanded: 0 });
}

struct ExhaustedProblem;

impl Problem for ExhaustedProblem {
    type State = State;
}

impl CostructSolution for ExhaustedProblem {
    type Action = ();
    type Cost = i32;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        (state.0 == 0).then_some(()).into_iter()
    }

    fn result(&self, _: &Self::State, _: &Self::Action) -> (Self::State, Self::Cost) {
        (State(1), 1)
    }
}

impl Utility for ExhaustedProblem {
    fn heuristic(&self, _: &Self::State) -> Self::Cost {
        0
    }
}

impl SuitableState for ExhaustedProblem {
    fn is_suitable(&self, _: &Self::State) -> bool {
        false
    }
}

#[test]
fn bounded_search_reports_exhaustion_separately_from_limit() {
    let result = bounded_search(
        &ExhaustedProblem,
        State(0),
        GraphSearchAlgorithm::UniformCost,
        2,
    );
    assert_eq!(result, GraphSearchOutcome::Exhausted { expanded: 2 });
}

#[path = "../examples/protein_folding/formulation.rs"]
#[allow(dead_code)]
mod formulation;

use agent::{
    problem::{CostructSolution, InitState},
    statexplorer::{
        frontier::{AStarBackend, FrontierBackend, MinCostBackend},
        resolver::TreeExplorer,
    },
};
use bumpalo::Bump;
use formulation::{
    h_lookahead3, h_lookahead3_parity, h_lookahead4_parity, AminoAcid, Dir, ProteinFolding,
};

fn sequence(input: &str) -> Vec<AminoAcid> {
    input
        .chars()
        .map(|residue| match residue {
            'H' => AminoAcid::H,
            'P' => AminoAcid::P,
            _ => panic!("invalid test sequence"),
        })
        .collect()
}

fn search_cost<'a, B: FrontierBackend<'a, ProteinFolding<'a>>>(
    problem: &'a ProteinFolding<'a>,
    arena: &'a Bump,
) -> u32 {
    let mut explorer = TreeExplorer::<_, B>::new(problem, arena);
    let result = explorer.search(problem.init_state());
    let actions = result.actions.expect("folding exists");
    assert_eq!(actions.len() + 1, problem.aminoacids.len());

    let mut state = problem.init_state();
    actions.iter().fold(0, |cost, action: &Dir| {
        let (next, step_cost) = problem.result(&state, action);
        state = next;
        cost + step_cost
    })
}

#[test]
fn test_protein_astar_matches_uniform_cost() {
    for input in [
        "HHPHPPHP",
        "HHPHPPHHHPPPPHH",
        "HHHHHHHH",
        "HPHPHPHP",
        "PPPPPPPP",
    ] {
        let residues = sequence(input);
        let problem_arena = Bump::new();
        let problem =
            ProteinFolding::with_heuristic(residues.clone(), &problem_arena, h_lookahead3);
        let astar_arena = Bump::new();
        let astar_cost =
            search_cost::<AStarBackend<'_, ProteinFolding<'_>>>(&problem, &astar_arena);

        let parity_problem_arena = Bump::new();
        let parity_problem = ProteinFolding::with_heuristic(
            residues.clone(),
            &parity_problem_arena,
            h_lookahead3_parity,
        );
        let parity_arena = Bump::new();
        let parity_cost =
            search_cost::<AStarBackend<'_, ProteinFolding<'_>>>(&parity_problem, &parity_arena);

        let parity4_problem_arena = Bump::new();
        let parity4_problem = ProteinFolding::with_heuristic(
            residues.clone(),
            &parity4_problem_arena,
            h_lookahead4_parity,
        );
        let parity4_arena = Bump::new();
        let parity4_cost =
            search_cost::<AStarBackend<'_, ProteinFolding<'_>>>(&parity4_problem, &parity4_arena);

        let uniform_problem_arena = Bump::new();
        let uniform_problem = ProteinFolding::new(residues, &uniform_problem_arena);
        let uniform_arena = Bump::new();
        let uniform_cost =
            search_cost::<MinCostBackend<'_, ProteinFolding<'_>>>(&uniform_problem, &uniform_arena);
        assert_eq!(astar_cost, uniform_cost, "{input}");
        assert_eq!(parity_cost, uniform_cost, "{input}");
        assert_eq!(parity4_cost, uniform_cost, "{input}");
    }
}

#[test]
fn test_parity_heuristic_dominates_count_bound() {
    let arena = Bump::new();
    let problem = ProteinFolding::with_heuristic(
        sequence("HHPHPHHHPPPPHHPHPHPP"),
        &arena,
        h_lookahead3_parity,
    );
    let mut states = vec![problem.init_state()];
    for _ in 0..5 {
        let mut children = Vec::new();
        for state in states {
            assert!(h_lookahead3_parity(&problem, state) >= h_lookahead3(&problem, state));
            for action in problem.executable_actions(&state) {
                children.push(problem.result(&state, &action).0);
            }
        }
        states = children;
    }
}

#[test]
fn test_parity_bounds_do_not_exceed_remaining_optimum() {
    for input in ["HHPPHHHH", "HPHPHPHP", "HHPHPPHP"] {
        let arena = Bump::new();
        let problem = ProteinFolding::new(sequence(input), &arena);
        let mut states = vec![problem.init_state()];
        for _ in 0..4 {
            let mut children = Vec::new();
            for state in states {
                let mut explorer =
                    TreeExplorer::<_, MinCostBackend<ProteinFolding>>::new(&problem, &arena);
                if let Some(actions) = explorer.search(state).actions {
                    let mut current = state;
                    let mut optimum = 0;
                    for action in actions {
                        let (next, cost) = problem.result(&current, &action);
                        current = next;
                        optimum += cost;
                    }
                    assert!(h_lookahead3_parity(&problem, state) <= optimum, "{input}");
                    assert!(h_lookahead4_parity(&problem, state) <= optimum, "{input}");
                }
                for action in problem.executable_actions(&state) {
                    children.push(problem.result(&state, &action).0);
                }
            }
            states = children;
        }
    }
}

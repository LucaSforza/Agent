#[path = "../examples/protein_folding/formulation.rs"]
mod formulation;

use agent::{
    problem::{CostructSolution, InitState},
    statexplorer::{
        frontier::{AStarBackend, FrontierBackend, MinCostBackend},
        resolver::TreeExplorer,
    },
};
use bumpalo::Bump;
use formulation::{h_lookahead3, AminoAcid, Dir, ProteinFolding};

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
    for input in ["HHPHPPHP", "HHPHPPHHHPPPPHH"] {
        let residues = sequence(input);
        let problem_arena = Bump::new();
        let problem =
            ProteinFolding::with_heuristic(residues.clone(), &problem_arena, h_lookahead3);
        let astar_arena = Bump::new();
        let astar_cost =
            search_cost::<AStarBackend<'_, ProteinFolding<'_>>>(&problem, &astar_arena);

        let uniform_problem_arena = Bump::new();
        let uniform_problem = ProteinFolding::new(residues, &uniform_problem_arena);
        let uniform_arena = Bump::new();
        let uniform_cost =
            search_cost::<MinCostBackend<'_, ProteinFolding<'_>>>(&uniform_problem, &uniform_arena);
        assert_eq!(astar_cost, uniform_cost, "{input}");
    }
}

mod formulation;

use std::collections::{HashMap, HashSet};

use agent::{
    explorer::Explorer,
    frontier::{
        AStarBackend, BestFirstBackend, DequeBackend, FrontierBackend, MinCostBackend, StackBackend,
    },
};
use formulation::{AminoAcid, Board, Direction, ProteinFolding};

fn run_example<B: FrontierBackend<ProteinFolding> + std::fmt::Debug>(protein: &Vec<AminoAcid>) {
    let problem = ProteinFolding::new(protein.clone());

    let mut resolver = Explorer::<ProteinFolding, B>::new(problem);
    let init_state = Board::init_state();

    let r = resolver.search(init_state);
    println!("{}", r);
    print_solution(protein, r.actions.unwrap());
}

type MinCost = MinCostBackend<ProteinFolding>;
type DFS = StackBackend<ProteinFolding>;
type BFS = DequeBackend<ProteinFolding>;
type AStar = AStarBackend<ProteinFolding>;
type BestFirst = BestFirstBackend<ProteinFolding>;

fn print_solution(protein: &Vec<AminoAcid>, solution: Vec<Direction>) {
    // Genera le posizioni originali degli aminoacidi
    let mut positions = vec![(0, 0)];
    let mut current_pos = (0, 0);
    for dir in solution {
        current_pos = match dir {
            Direction::Up => (current_pos.0, current_pos.1 + 1),
            Direction::Down => (current_pos.0, current_pos.1 - 1),
            Direction::Left => (current_pos.0 - 1, current_pos.1),
            Direction::Right => (current_pos.0 + 1, current_pos.1),
        };
        positions.push(current_pos);
    }

    // Mappa posizioni con indici
    let pos_to_index: HashMap<(i32, i32), usize> = positions
        .iter()
        .enumerate()
        .map(|(i, pos)| (*pos, i))
        .collect();

    // Conta H adiacenti non collegati
    let mut adjacency_pairs = HashSet::new();
    for (i, aa) in protein.iter().enumerate() {
        if *aa == AminoAcid::H {
            let (x, y) = positions[i];
            // Controlla tutte e quattro le direzioni
            for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let neighbor_pos = (x + dx, y + dy);
                if let Some(&j) = pos_to_index.get(&neighbor_pos) {
                    if protein[j] == AminoAcid::H && j != i {
                        // Escludi coppie consecutive
                        if (i as i32 - j as i32).abs() != 1 {
                            let pair = if i < j { (i, j) } else { (j, i) };
                            adjacency_pairs.insert(pair);
                        }
                    }
                }
            }
        }
    }

    // Crea griglia scalata per legami
    let mut grid: HashMap<(i32, i32), char> = HashMap::new();
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);

    // Aggiungi aminoacidi alla griglia
    for (i, &(x, y)) in positions.iter().enumerate() {
        let scaled_x = x * 2;
        let scaled_y = y * 2;
        grid.insert(
            (scaled_x, scaled_y),
            match protein[i] {
                AminoAcid::H => 'H',
                AminoAcid::P => 'P',
            },
        );
        min_x = min_x.min(scaled_x);
        max_x = max_x.max(scaled_x);
        min_y = min_y.min(scaled_y);
        max_y = max_y.max(scaled_y);
    }

    // Aggiungi legami alla griglia
    for i in 0..positions.len() - 1 {
        let (x1, y1) = positions[i];
        let (x2, y2) = positions[i + 1];
        let (sx1, sy1) = (x1 * 2, y1 * 2);
        let (sx2, sy2) = (x2 * 2, y2 * 2);

        if x1 != x2 {
            grid.insert((sx1 + (sx2 - sx1).signum(), sy1), '-');
        } else {
            grid.insert((sx1, sy1 + (sy2 - sy1).signum()), '|');
        }
    }

    // Stampa la griglia
    for y in (min_y..=max_y).rev() {
        let mut line = String::new();
        for x in min_x..=max_x {
            line.push(*grid.get(&(x, y)).unwrap_or(&' '));
        }
        println!("{}", line);
    }

    println!("\nEnergy: {}", -(adjacency_pairs.len() as isize));
}

fn run_all(protein: &Vec<AminoAcid>) {
    println!("MinCost:");
    run_example::<MinCost>(protein);
    println!("BFS:");
    run_example::<BFS>(protein);
    println!("DFS:");
    run_example::<DFS>(protein);
    println!("AStar:");
    run_example::<AStar>(protein);
    println!("BestFirst:");
    run_example::<BestFirst>(protein);
    println!("Iterative:");
    let problem = ProteinFolding::new(protein.clone());

    let mut resolver = Explorer::<ProteinFolding, DFS>::new(problem);
    let init_state = Board::init_state();

    let r = resolver.iterative_search(init_state, 300);
    println!("{}", r);
    print_solution(protein, r.actions.unwrap());
}

fn main() {
    let protein = vec![
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::P,
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::P,
    ];

    run_all(&protein);
    /*
    let protein = vec![
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::H,
    ];
    run_all(&protein); */
}

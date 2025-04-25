mod formulation;

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use agent::{
    problem::InitState,
    statexplorer::{
        frontier::{
            AStarBackend, BestFirstBackend, DequeBackend, FrontierBackend, MinCostBackend,
            StackBackend,
        },
        resolver::TreeExplorer,
    },
};
use bumpalo::Bump;
use formulation::{AminoAcid, Dir, ProteinFolding};
use rand::seq::SliceRandom;

fn run_example<'a, B: FrontierBackend<ProteinFolding<'a>> + std::fmt::Debug>(
    protein: &Vec<AminoAcid>,
    arena: &'a Bump,
) -> Duration {
    let problem = ProteinFolding::new(protein.clone(), arena);

    let init_state = problem.init_state();
    let mut resolver = TreeExplorer::<ProteinFolding, B>::new(problem);

    let r = resolver.search(init_state);
    println!("{}", r);
    print_solution(protein, r.actions.unwrap());
    r.total_time
}

type MinCost<'a> = MinCostBackend<ProteinFolding<'a>>;
type AStar<'a> = AStarBackend<ProteinFolding<'a>>;
type BestFirst<'a> = BestFirstBackend<ProteinFolding<'a>>;
type BFS<'a> = DequeBackend<ProteinFolding<'a>>;
type DFS<'a> = StackBackend<ProteinFolding<'a>>;

fn print_solution(protein: &Vec<AminoAcid>, solution: Vec<Dir>) -> i32 {
    // Genera le posizioni originali degli aminoacidi
    let mut positions = vec![(0, 0)];
    let mut current_pos = (0, 0);
    for dir in solution {
        current_pos = match dir {
            Dir::Up => (current_pos.0, current_pos.1 + 1),
            Dir::Down => (current_pos.0, current_pos.1 - 1),
            Dir::Left => (current_pos.0 - 1, current_pos.1),
            Dir::Right => (current_pos.0 + 1, current_pos.1),
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
    -(adjacency_pairs.len() as i32)
}

fn run_all(protein: &Vec<AminoAcid>) {
    println!("MinCost:");
    let arena = Bump::new();
    run_example::<MinCost>(protein, &arena);
    drop(arena);
    let arena = Bump::new();
    println!("AStar:");
    run_example::<AStar>(protein, &arena);
    // println!("BestFirst:");
    // run_example::<BestFirst>(protein);
    // println!("DFS:");
    // run_example::<DFS>(protein);
    // println!("BFS:");
    // run_example::<BFS>(protein);
}

use AminoAcid::*;

fn random_protein(n: usize, h_number: usize) -> Vec<AminoAcid> {
    assert!(n >= h_number);

    let mut result: Vec<AminoAcid> = vec![H; h_number];
    result.extend(vec![P; n - h_number]);
    let mut rng = rand::rng();
    result.shuffle(&mut rng);

    result
}
/*
fn random_test(n: usize, iters: usize) {
    for i in 19..=n {
        let mut max_ratio: f64 = 0.0;
        let mut max_time = Duration::default();
        for j in 0..=i {
            let mut med = Duration::default();
            for _ in 0..iters {
                let r = random_protein(i, j);
                let d = run_example::<AStar>(&r);
                med += d / iters as u32;
            }
            let ratio = j as f64 / i as f64;
            println!(
                "ratio: {}\nprotein lenght: {}\nh number: {}\ntime: {:?}\n",
                ratio, i, j, med
            );
            if med > max_time {
                max_time = med;
                max_ratio = ratio;
            }
        }
        println!("protein lenght: {}\n Max ratio: {}", i, max_ratio);
    }
}*/

fn main() {
    //let protein = vec![P, H, H, P, H, P, P, H, P];

    // let protein = vec![H, H, P, H, P, P, H, H, H, P, P, P, P, H, H, P];

    let protein = vec![
        H, H, P, H, P, P, H, H, H, P, P, P, P, H, H, P, H, P, H, P, P, H, P, H, P, H,
    ];

    // let protein = vec![
    //     H, H, P, H, P, P, H, H, H, P, P, P, P, H, H, P, H, P, H, P, P, H, P, H, P, H, H,
    // ];

    // let protein = vec![H, H, H, H, H, H, H, H, H, P, H, H, H, H, H, H, H, H, H];

    run_all(&protein);

    // let mut rng = rand::rng();
    // let r = random_protein(20, rng.random_range(0..=20));
    // println!("{:?}", r);
    // let s1 = run_example::<MinCost>(&r);
    // let s2 = run_example::<AStar>(&r);
    // assert_eq!(s1, s2);

    // random_test(20, 5);
}

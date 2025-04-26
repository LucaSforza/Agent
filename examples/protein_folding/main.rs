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

fn run_example<'a, B: FrontierBackend<'a, ProteinFolding<'a>> + std::fmt::Debug>(
    arena: &'a Bump,
    problem: &'a ProteinFolding<'a>,
) {
    let init_state = problem.init_state();
    let mut resolver = TreeExplorer::<'a, ProteinFolding, B>::new(problem, arena);

    let r = resolver.search(init_state);
    println!("{}", r);
    print_solution(&problem.aminoacids, r.actions.unwrap());
}

fn run_example_get_time<'a, B: FrontierBackend<'a, ProteinFolding<'a>>>(
    arena: &'a Bump,
    problem: &'a ProteinFolding<'a>,
) -> Duration {
    let init_state = problem.init_state();
    let mut resolver = TreeExplorer::<'a, ProteinFolding, B>::new(problem, arena);

    let r = resolver.search(init_state);
    r.total_time
}

type MinCost<'a> = MinCostBackend<'a, ProteinFolding<'a>>;
type AStar<'a> = AStarBackend<'a, ProteinFolding<'a>>;
type BestFirst<'a> = BestFirstBackend<'a, ProteinFolding<'a>>;
type BFS<'a> = DequeBackend<'a, ProteinFolding<'a>>;
type DFS<'a> = StackBackend<'a, ProteinFolding<'a>>;

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

fn run_all(protein: Vec<AminoAcid>) {
    {
        let arena_problem = Bump::new();
        let problem = ProteinFolding::new(protein.clone(), &arena_problem);
        println!("MinCost:");
        let arena_explorer = Bump::new();
        run_example::<MinCost>(&arena_explorer, &problem);
    }
    {
        let arena_problem = Bump::new();
        let problem = ProteinFolding::new(protein, &arena_problem);
        println!("AStar:");
        let arena_explorer = Bump::new();
        run_example::<AStar>(&arena_explorer, &problem);
    }
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

fn random_test(n: usize, iters: usize) {
    let mut arena = Bump::new();

    for i in 1..=n {
        let mut max_ratio: f64 = 0.0;
        let mut max_time = Duration::default();
        for j in 0..=i {
            let mut med = Duration::default();
            for _ in 0..iters {
                let r = random_protein(i, j);
                let problem = ProteinFolding::new(r, &arena);
                let d = run_example_get_time::<AStar>(&arena, &problem);
                arena.reset();
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
}

use clap::Parser;

#[derive(Debug, Clone)]
struct AminoAcidSequence(Vec<AminoAcid>);

// Implement FromStr to parse a string into AminoAcidSequence
impl std::str::FromStr for AminoAcidSequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::with_capacity(s.len());
        for c in s.chars() {
            match c {
                'H' => result.push(AminoAcid::H),
                'P' => result.push(AminoAcid::P),
                _ => return Err(format!("Invalid aminoacid '{}'", c)),
            }
        }
        Ok(AminoAcidSequence(result))
    }
}

#[derive(Parser)]
enum Commands {
    RunProtein {
        aminoacids: AminoAcidSequence,
    },
    RandTest {
        #[clap(short, long)]
        len: usize,
        #[clap(short, long)]
        iters: usize,
    },
}

/*
Example values:
    PHHPHPPHP
    HHPHPPHHHPPPPHH
    HHPHPHHHPPPPHHPP
    HHPHPHHHPPPPHHPHPHPPHPHPH da controllare
    HHPHPPHHHPPPPHHPHPHPPHPHPHH
*/

fn main() {
    let args = Commands::parse();

    match args {
        Commands::RunProtein { aminoacids } => run_all(aminoacids.0),
        Commands::RandTest { len, iters } => random_test(len, iters),
    }

    // let mut rng = rand::rng();
    // let r = random_protein(20, rng.random_range(0..=20));
    // println!("{:?}", r);
    // let s1 = run_example::<MinCost>(&r);
    // let s2 = run_example::<AStar>(&r);
    // assert_eq!(s1, s2);

    // random_test(20, 5);
}

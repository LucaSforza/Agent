#[path = "protein_folding/formulation.rs"]
mod formulation;

use std::{env, process};

use agent::{
    problem::InitState,
    statexplorer::{frontier::AStarBackend, resolver::TreeExplorer},
};
use bumpalo::Bump;
use formulation::{h_lookahead3, AminoAcid, Dir, ProteinFolding};

fn parse_sequence(input: &str) -> Result<Vec<AminoAcid>, String> {
    if input.is_empty() {
        return Err("sequence must not be empty".into());
    }

    input
        .chars()
        .map(|residue| match residue {
            'H' => Ok(AminoAcid::H),
            'P' => Ok(AminoAcid::P),
            _ => Err(format!("invalid residue {residue:?}; use only H and P")),
        })
        .collect()
}

fn energy(sequence: &[AminoAcid], actions: &[Dir]) -> i32 {
    let mut positions = Vec::with_capacity(sequence.len());
    let (mut x, mut y) = (0i32, 0i32);
    positions.push((x, y));

    for direction in actions {
        match direction {
            Dir::Up => x -= 1,
            Dir::Down => x += 1,
            Dir::Left => y -= 1,
            Dir::Right => y += 1,
        }
        positions.push((x, y));
    }

    let mut contacts = 0;
    for i in 0..sequence.len() {
        if sequence[i] != AminoAcid::H {
            continue;
        }
        for j in (i + 2)..sequence.len() {
            if sequence[j] == AminoAcid::H
                && (positions[i].0 - positions[j].0).abs() + (positions[i].1 - positions[j].1).abs()
                    == 1
            {
                contacts += 1;
            }
        }
    }
    -contacts
}

fn run(sequence: &[AminoAcid]) -> Result<(u128, usize, i32), String> {
    let arena = Bump::new();
    let problem = ProteinFolding::with_heuristic(sequence.to_vec(), &arena, h_lookahead3);
    let mut explorer = TreeExplorer::<_, AStarBackend<ProteinFolding>>::new(&problem, &arena);
    let result = explorer.search(problem.init_state());
    let actions = result
        .actions
        .as_deref()
        .ok_or_else(|| "search found no solution".to_owned())?;

    Ok((
        result.total_time.as_nanos(),
        result.n_iter,
        energy(sequence, actions),
    ))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 1 || args.len() > 2 {
        eprintln!("usage: cargo run --release --example protein_bench -- SEQUENCE [REPETITIONS]");
        process::exit(2);
    }

    let sequence = match parse_sequence(&args[0]) {
        Ok(sequence) => sequence,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };
    let repetitions = match args.get(1).map_or(Ok(5usize), |value| value.parse()) {
        Ok(count) if count > 0 => count,
        _ => {
            eprintln!("repetitions must be a positive integer");
            process::exit(2);
        }
    };

    println!("sequence,run,duration_ns,iterations,energy");
    for run_index in 1..=repetitions {
        match run(&sequence) {
            Ok((duration_ns, iterations, score)) => println!(
                "{},{},{},{},{}",
                args[0], run_index, duration_ns, iterations, score
            ),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    }
}

#[path = "protein_folding/formulation.rs"]
#[allow(dead_code)]
mod formulation;

use std::{env, process};

use agent::{
    problem::InitState,
    statexplorer::{frontier::AStarBackend, resolver::TreeExplorer},
};
use bumpalo::Bump;
use formulation::{
    h_lookahead2, h_lookahead3, h_lookahead3_parity, h_lookahead4_parity, old_heuristic, AminoAcid,
    Dir, ProteinFolding,
};

const HEURISTICS: &[&str] = &[
    "legacy",
    "one_step",
    "lookahead2",
    "lookahead3",
    "lookahead3_parity",
    "lookahead4_parity",
];

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

fn run(sequence: &[AminoAcid], heuristic: &str) -> Result<(u128, usize, i32), String> {
    let arena = Bump::new();
    let problem = match heuristic {
        "legacy" => ProteinFolding::with_heuristic(sequence.to_vec(), &arena, old_heuristic),
        "one_step" => ProteinFolding::new(sequence.to_vec(), &arena),
        "lookahead2" => ProteinFolding::with_heuristic(sequence.to_vec(), &arena, h_lookahead2),
        "lookahead3" => ProteinFolding::with_heuristic(sequence.to_vec(), &arena, h_lookahead3),
        "lookahead3_parity" => {
            ProteinFolding::with_heuristic(sequence.to_vec(), &arena, h_lookahead3_parity)
        }
        "lookahead4_parity" => {
            ProteinFolding::with_heuristic(sequence.to_vec(), &arena, h_lookahead4_parity)
        }
        _ => return Err(format!("unknown heuristic {heuristic:?}")),
    };
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
    if args.len() != 3 {
        eprintln!("usage: protein_heuristic_bench SEQUENCE HEURISTIC REPETITIONS");
        eprintln!("heuristics: {}", HEURISTICS.join(", "));
        process::exit(2);
    }
    let sequence = match parse_sequence(&args[0]) {
        Ok(sequence) => sequence,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };
    if !HEURISTICS.contains(&args[1].as_str()) {
        eprintln!(
            "unknown heuristic {:?}; expected one of {}",
            args[1],
            HEURISTICS.join(", ")
        );
        process::exit(2);
    }
    let repetitions = match args[2].parse::<usize>() {
        Ok(count) if count > 0 => count,
        _ => {
            eprintln!("repetitions must be a positive integer");
            process::exit(2);
        }
    };

    println!("heuristic,sequence,run,duration_ns,iterations,energy");
    for run_index in 1..=repetitions {
        match run(&sequence, &args[1]) {
            Ok((duration_ns, iterations, score)) => println!(
                "{},{},{},{},{},{}",
                args[1], args[0], run_index, duration_ns, iterations, score
            ),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    }
}

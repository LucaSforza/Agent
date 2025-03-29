use std::fmt;
use std::time::Duration;

use agent::explorer::{AStarExplorer, BFSExplorer};
use agent::iterative_improvement::{
    HillClimbing, ImprovingAlgorithm, Resolver, SimulatedAnnealing, SteepestDescend,
};
use agent::problem::{Problem, RandomizeState, Utility, WithSolution};
use rand::seq::IteratorRandom;
use rand_distr::uniform::{UniformSampler, UniformUsize};

type NextQueenPos = usize;

struct NextQueenIterator {
    i: usize,
    n: usize,
}

impl NextQueenIterator {
    fn new(n: usize) -> Self {
        Self { i: 0, n: n }
    }

    fn void_iter(n: usize) -> Self {
        Self { i: n, n: n }
    }
}

impl Iterator for NextQueenIterator {
    type Item = NextQueenPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.n {
            let to_return = self.i;
            self.i += 1;
            return Some(to_return);
        }
        return None;
    }
}

struct MoveQueen {
    col: usize,
    new_row: usize,
}

impl MoveQueen {
    fn new(col: usize, new_row: usize) -> Self {
        Self {
            col: col,
            new_row: new_row,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct DeploymentQueens {
    pos: Vec<usize>,
}

impl fmt::Debug for DeploymentQueens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        let n = self.pos.len();
        for row in 0..n {
            for col in 0..n {
                if self.pos[col] == row {
                    write!(f, "Q ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl DeploymentQueens {
    fn new(pos: Vec<usize>) -> Self {
        Self { pos: pos }
    }

    fn push_queen(&self, col: NextQueenPos) -> Self {
        let mut new_pos = self.pos.clone();
        new_pos.push(col);
        Self::new(new_pos)
    }

    fn move_queen(&self, dir: &MoveQueen) -> Self {
        let mut new_pos = self.pos.clone();
        new_pos[dir.col] = dir.new_row;
        Self::new(new_pos)
    }
}

impl Default for DeploymentQueens {
    fn default() -> Self {
        Self { pos: Vec::new() }
    }
}

struct NQueen {
    n: usize,
    distr: UniformUsize,
}

impl NQueen {
    fn new(n: usize) -> Self {
        Self {
            n: n,
            distr: UniformUsize::new(0, n).unwrap(),
        }
    }
}

use ordered_float::OrderedFloat;

impl Problem for NQueen {
    type State = DeploymentQueens;
    type Action = NextQueenPos;
    type Cost = OrderedFloat<f64>;
    type ActionIterator = NextQueenIterator;

    fn executable_actions(&self, state: &Self::State) -> Self::ActionIterator {
        if self.is_goal(state) {
            NextQueenIterator::void_iter(self.n)
        } else {
            NextQueenIterator::new(self.n)
        }
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        let new_state = state.push_queen(*action);
        (new_state, 1.into())
    }
}

impl Utility for NQueen {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        let mut result: f64 = 0.0;

        for i in 0..state.pos.len() {
            for j in (i + 1)..state.pos.len() {
                if state.pos[i] == state.pos[j]
                    || state.pos[i].abs_diff(state.pos[j]) == i.abs_diff(j)
                {
                    result += 1.0;
                }
            }
        }

        if !self.is_goal(state) {
            result += (self.n - state.pos.len()) as f64
        }

        return result.into();
    }
}

impl WithSolution for NQueen {
    fn is_goal(&self, state: &Self::State) -> bool {
        self.n == state.pos.len()
    }
}

fn resolve_nqueen<A: ImprovingAlgorithm<NQueen>>(
    problem: &NQueen,
    resolver: &mut Resolver<A, NQueen>,
    iterations: u32,
) {
    let mut total_duration: Duration = Duration::default();
    let mut correct = 0;

    for _ in 0..iterations {
        let result = resolver.resolve(problem);
        total_duration += result.duration;
        if result.h <= 0.0.into() {
            correct += 1;
        }
    }
    println!("\tOne restart:");
    println!("\t  Correctness: {}", (correct as f64) / iterations as f64);
    println!("\t  Total Duration: {:?}", total_duration);
    println!("\t  Mean time: {:?}", total_duration / iterations);
}

fn resolve_restart_nqueen<A: ImprovingAlgorithm<NQueen>>(
    problem: &NQueen,
    resolver: &mut Resolver<A, NQueen>,
    iterations: u32,
    n_restarts: usize,
) {
    let mut total_duration: Duration = Duration::default();
    let mut correct = 0;

    for _ in 0..iterations {
        let result = resolver.resolve_restart(problem, n_restarts);
        total_duration += result.duration;
        if result.h <= 0.0.into() {
            correct += 1;
        }
    }
    println!("\tNumber restarts:{}", n_restarts);
    println!("\t  Correctness: {}", (correct as f64) / iterations as f64);
    println!("\t  Total Duration: {:?}", total_duration);
    println!("\t  Mean time: {:?}", total_duration / iterations);
}

fn run_one_time_nqueen_algo<A: ImprovingAlgorithm<NQueen>>(
    problem: &NQueen,
    resolver: &mut Resolver<A, NQueen>,
    n_restarts: usize,
) {
    let r = resolver.resolve(problem);
    println!("{:?}", r);

    let r = resolver.resolve_restart(problem, n_restarts);

    println!("{:?}", r)
}

fn run_one_time_nqueen(size: usize, n_restarts: usize) {
    let problem = NQueen::new(size);

    println!("Steepest Descend:");
    let mut resolver = Resolver::new(SteepestDescend::new(rand::rng()));
    run_one_time_nqueen_algo(&problem, &mut resolver, n_restarts);

    println!("\nHill Climbing:");
    let mut resolver = Resolver::new(HillClimbing::with_max_lateral(rand::rng(), 100));
    run_one_time_nqueen_algo(&problem, &mut resolver, n_restarts);

    println!("\nSimulated Annealing:");
    let mut resolver = Resolver::new(SimulatedAnnealing::new(rand::rng()));
    run_one_time_nqueen_algo(&problem, &mut resolver, n_restarts);
}

fn run_nqueen(n: usize, iterations: u32, restarts: usize) {
    let problem = NQueen::new(n);

    println!("Steepest Descend:");
    let mut resolver = Resolver::new(SteepestDescend::new(rand::rng()));
    resolve_nqueen(&problem, &mut resolver, iterations);
    resolve_restart_nqueen(&problem, &mut resolver, iterations, restarts);

    println!("Hill Climbing:");
    let mut resolver = Resolver::new(HillClimbing::with_max_lateral(rand::rng(), 100));
    resolve_nqueen(&problem, &mut resolver, iterations);
    resolve_restart_nqueen(&problem, &mut resolver, iterations, restarts);

    println!("Simulated Annealing:");
    let mut resolver = Resolver::new(SimulatedAnnealing::new(rand::rng()));
    resolve_nqueen(&problem, &mut resolver, iterations);
    resolve_restart_nqueen(&problem, &mut resolver, iterations, restarts);
}

fn run_nqueen_explorer(n: usize) {
    let problem = NQueen::new(n);

    let mut explorer = AStarExplorer::new(problem);

    let result = explorer.search(DeploymentQueens::default());

    println!("{}", result);
}

fn main() {
    // run_nqueen(8, 2500, 100);
    // run_one_time_nqueen(8, 100);

    run_nqueen_explorer(100);
}

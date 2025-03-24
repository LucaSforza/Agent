use std::fmt;
use std::time::Duration;

use agent::iterative_improvement::{
    HillClimbing, ImprovingAlgorithm, Resolver, SimulatedAnnealing, SteepestDescend,
};
use agent::problem::{IterativeImprovingProblem, Problem, RandomizeState, Utility};
use rand_distr::uniform::{UniformSampler, UniformUsize};

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

#[derive(Clone)]
struct DeploymentQueens {
    pos: Vec<usize>,
}

impl fmt::Debug for DeploymentQueens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn move_queen(&self, dir: &MoveQueen) -> Self {
        let mut new_pos = self.pos.clone();
        new_pos[dir.col] = dir.new_row;
        Self::new(new_pos)
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
    type Action = MoveQueen;
    type Cost = OrderedFloat<f64>;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        let mut actions = Vec::with_capacity(self.n * (self.n - 1));

        for i in 0..self.n {
            for j in 0..self.n {
                if state.pos[i] != j {
                    actions.push(MoveQueen::new(i, j))
                }
            }
        }

        actions.into_iter()
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        let new_state = state.move_queen(action);
        (new_state, 0.into())
    }
}

impl Utility for NQueen {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        let mut result = 0.0;

        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if state.pos[i] == state.pos[j]
                    || state.pos[i].abs_diff(state.pos[j]) == i.abs_diff(j)
                {
                    result += 1.0;
                }
            }
        }

        return result.into();
    }
}

impl RandomizeState for NQueen {
    fn random_state<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::State {
        let mut pos = Vec::with_capacity(self.n);
        for _ in 0..self.n {
            pos.push(self.distr.sample(rng));
        }

        DeploymentQueens::new(pos)
    }
}

impl IterativeImprovingProblem for NQueen {}

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

fn main() {
    run_nqueen(32, 10, 10);
    run_one_time_nqueen(32, 10);
}

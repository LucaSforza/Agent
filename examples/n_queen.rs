use std::fmt;
use std::time::Duration;

use agent::iterative_improvement::{
    HillClimbing, ImprovingAlgorithm, Resolver, SimulatedAnnealing, SteepestDescend,
};
use agent::problem::{IterativeImprovingProblem, Problem};
use rand_distr::uniform::{UniformSampler, UniformUsize};

enum Direction {
    Up,
    Down,
}

struct MoveQueen {
    col: usize,
    direction: Direction,
}

impl MoveQueen {
    fn new(col: usize, dir: Direction) -> Self {
        Self {
            col: col,
            direction: dir,
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

    fn move_queen(&self, problem: &NQueen, dir: &MoveQueen) -> Self {
        let mut new_pos = self.pos.clone();
        match dir.direction {
            Direction::Up => {
                assert!(new_pos[dir.col] != 0);
                new_pos[dir.col] -= 1;
            }
            Direction::Down => {
                assert!(new_pos[dir.col] + 1 < problem.n);
                new_pos[dir.col] += 1;
            }
        }
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
        let mut actions = Vec::with_capacity(self.n * 2);

        for i in 0..self.n {
            if state.pos[i] + 1 < self.n {
                actions.push(MoveQueen::new(i, Direction::Down));
            }
            if state.pos[i] != 0 {
                actions.push(MoveQueen::new(i, Direction::Up));
            }
        }

        actions.into_iter()
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        let new_state = state.move_queen(self, action);
        (new_state, 0.into())
    }

    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        let mut result = 0.0;

        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if state.pos[i] == state.pos[j] {
                    result += 1.0;
                    // break;
                }
            }

            for j in (i + 1)..self.n {
                if state.pos[i].abs_diff(state.pos[j]) == i.abs_diff(j) {
                    result += 1.0;
                    // break;
                }
            }
        }

        return result.into();
    }
}

impl IterativeImprovingProblem for NQueen {
    fn random_state<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::State {
        let mut pos = Vec::with_capacity(self.n);
        for _ in 0..self.n {
            pos.push(self.distr.sample(rng));
        }

        DeploymentQueens::new(pos)
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
    run_nqueen(8, 2500, 100);
}

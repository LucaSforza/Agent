use std::fmt;
use std::time::Duration;

use agent::improve::{
    algorithms::GeneticAlgorithm, algorithms::HillClimbing, algorithms::ImprovingAlgorithm,
    algorithms::LocalBeam, algorithms::SimulatedAnnealing, algorithms::SteepestDescend,
    resolver::Resolver,
};
use agent::problem::{
    CostructSolution, Crossover, InitState, Problem, StatePerturbation, SuitableState, Utility,
};
use agent::statexplorer::frontier::{
    AStarBackend, BestFirstBackend, FrontierBackend, MinCostBackend,
};
use agent::statexplorer::resolver::Explorer;

use bumpalo::Bump;
use ordered_float::OrderedFloat;

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

    fn potential_conficts(&self, next_row: NextQueenPos) -> usize {
        let mut conficts = 0;
        let next_col = self.pos.len();
        for (col, row) in self.pos.iter().enumerate() {
            if *row == next_row || row.abs_diff(next_row) == col.abs_diff(next_col) {
                conficts += 1;
            }
        }
        conficts
    }

    fn push_queen(&self, next_row: NextQueenPos) -> (Self, OrderedFloat<f64>) {
        let cost = self.potential_conficts(next_row);
        let mut new_pos = self.pos.clone();
        new_pos.push(next_row);
        let result = Self::new(new_pos);
        (result, (cost as f64).into())
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

#[derive(Clone)]
struct NQueen {
    n: usize,
}

impl NQueen {
    fn new(n: usize) -> Self {
        Self { n: n }
    }
}

impl Problem for NQueen {
    type State = DeploymentQueens;
}

impl CostructSolution for NQueen {
    type Action = NextQueenPos;
    type Cost = OrderedFloat<f64>;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        if self.is_suitable(state) {
            NextQueenIterator::void_iter(self.n)
        } else {
            NextQueenIterator::new(self.n)
        }
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        state.push_queen(*action)
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

        return result.into();
    }
}

impl SuitableState for NQueen {
    fn is_suitable(&self, state: &Self::State) -> bool {
        self.n == state.pos.len()
    }
}

struct MoveQueenIterator {
    i: usize,
    j: usize,
    n: usize,
}

impl MoveQueenIterator {
    fn new(n: usize) -> Self {
        Self { i: 0, j: 0, n: n }
    }
}

impl Iterator for MoveQueenIterator {
    type Item = MoveQueen;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.n {
            let to_move = MoveQueen::new(self.j, self.i);
            self.i += 1;

            return to_move.into();
        } else if self.j >= self.n {
            return None;
        } else {
            let to_move = MoveQueen::new(self.j, self.i);
            self.j += 1;
            if self.j >= self.n {
                // esauriamo l'iteratore se finiamo le colonne da esplorare
                self.i = self.n;
            } else {
                self.i = 0;
            }
            return to_move.into();
        }
    }
}

impl StatePerturbation for NQueen {
    type Perturbation = MoveQueen;

    fn perturbations(&self, _state: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
        MoveQueenIterator::new(self.n)
    }

    fn perturb(&self, state: &Self::State, action: &Self::Perturbation) -> Self::State {
        state.move_queen(action)
    }
}

impl Crossover for NQueen {
    fn crossover<R: rand::Rng + ?Sized>(
        &self,
        rng: &mut R,
        state: &Self::State,
        other: &Self::State,
    ) -> Self::State {
        // TODO: move the board in such a way as to minimize attacks
        let crossover_point = rng.random_range(0..self.n);
        let mut new_pos = state.pos[..crossover_point].to_vec();
        new_pos.extend_from_slice(&other.pos[crossover_point..]);
        Self::State::new(new_pos)
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

    println!("Local Beam:");
    let mut resolver = Resolver::new(LocalBeam::from_parts(rand::rng(), 20, 100.into()));
    run_one_time_nqueen_algo(&problem, &mut resolver, n_restarts);

    println!("Genetic Algorithm:");

    let mut resolver = Resolver::new(GeneticAlgorithm::from_parts(
        rand::rng(),
        20,
        100.into(),
        0.8,
    ));

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

    println!("Local Beam:");
    let mut resolver = Resolver::new(LocalBeam::from_parts(rand::rng(), 20, 100.into()));
    resolve_nqueen(&problem, &mut resolver, iterations);

    println!("Genetic Algorithm:");
    let mut resolver = Resolver::new(GeneticAlgorithm::from_parts(
        rand::rng(),
        20,
        100.into(),
        0.8,
    ));
    resolve_nqueen(&problem, &mut resolver, iterations);
}

fn run_nqueen_explorer<'a, Backend: FrontierBackend<'a, NQueen> + fmt::Debug>(
    problem: &'a NQueen,
    arena: &'a Bump,
) {
    let mut explorer = Explorer::<'a, NQueen, Backend>::new(problem, arena);

    let result = explorer.search(problem.init_state());

    println!("result:\n{}", result);
    if let Some(state) = result.state {
        println!("h: {}", problem.heuristic(&state));
    } else {
        println!("NO SOLUTION FOUND");
    }
}

fn run_nqueens_explorer(n: usize) {
    let problem = NQueen::new(n);

    let mut arena = Bump::new();

    println!("A*:");
    run_nqueen_explorer::<AStarBackend<NQueen>>(&problem, &arena);

    arena.reset();

    println!("MinCost:");

    run_nqueen_explorer::<MinCostBackend<NQueen>>(&problem, &arena);

    arena.reset();

    println!("Best First:");
    run_nqueen_explorer::<BestFirstBackend<NQueen>>(&problem, &arena);

    arena.reset();
}

use clap::Parser;

#[derive(Parser)]
#[command(author = "Luca Sforza", version, about)]
enum Command {
    StateExploration {
        n: usize,
    },
    OneTime {
        #[clap(short)]
        n: usize,
        #[clap(short, long)]
        restarts: usize,
    },
    Perform {
        #[clap(short)]
        n: usize,
        #[clap(short, long)]
        iterations: u32,
        #[clap(short, long)]
        restarts: usize,
    },
}

fn main() {
    let args = Command::parse();

    match args {
        Command::StateExploration { n } => run_nqueens_explorer(n),
        Command::OneTime { n, restarts } => run_one_time_nqueen(n, restarts),
        Command::Perform {
            n,
            iterations,
            restarts,
        } => run_nqueen(n, iterations, restarts),
    }
}

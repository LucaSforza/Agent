use std::fmt;

use agent::iterative_improvement::{Resolver, SteepestDescend};
use agent::problem::{IterativeImprovingProblem, Problem};
use rand_distr::uniform::{UniformSampler, UniformUsize};
use rand_distr::Uniform;

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

impl Problem for NQueen {
    type State = DeploymentQueens;
    type Action = MoveQueen;
    type Cost = u64;

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
        (new_state, 0)
    }

    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        let mut result = 0;

        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if state.pos[i] == state.pos[j] {
                    result += 1;
                    break;
                }
            }

            for j in 1..=(self.n - i - 1) {
                if (state.pos[i + j] as isize - j as isize) < 0 {
                    continue;
                }
                if state.pos[i] == (state.pos[i + j] - j) {
                    result += 1;
                    break;
                }
            }

            for j in 1..=(self.n - i - 1) {
                if state.pos[i] == (state.pos[i + j] + j) {
                    result += 1;
                    break;
                }
            }
        }

        return result;
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

fn main() {
    let problem = NQueen::new(8);
    let k_1 = DeploymentQueens::new(vec![0, 1, 0, 2, 5, 3, 3, 6]);
    let k_2 = DeploymentQueens::new(vec![0, 1, 0, 2, 5, 7, 3, 6]);
    let c_1 = problem.heuristic(&k_1);
    let c_2 = problem.heuristic(&k_2);
    assert_eq!(7, c_1);
    assert_eq!(5, c_2);

    let mut resolver = SteepestDescend::new(rand::rng());
    let state = resolver.resolve(&problem);
    println!("state:\n{:?}\ncost:{}", state, problem.heuristic(&state));

    let state = resolver.resolve_restart(&problem, 1000);
    println!("state:\n{:?}\ncost:{}", state, problem.heuristic(&state))
}

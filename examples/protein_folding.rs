use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use agent::problem::{Problem, Utility};
use ordered_float::OrderedFloat;
use petgraph::{data::Build, graph::NodeIndex};

#[derive(PartialEq, Eq)]
enum AminoAcid {
    H,
    P,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Pos {
    id: usize,
    x: usize,
    y: usize,
    dir: Direction,
}

impl Pos {
    fn move_dir(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.x -= 1,
            Direction::Down => self.x += 1,
            Direction::Left => self.y -= 1,
            Direction::Right => self.y += 1,
        }
    }

    fn clone_move(&self, dir: Direction) -> Self {
        let mut new_pos = self.clone();
        new_pos.move_dir(dir);
        return new_pos;
    }

    fn old_pos(&self, n: usize) -> Self {
        let x;
        let y;

        match self.dir {
            Direction::Up => {
                assert!(self.x != 0);
                x = self.x - 1;
                y = self.y;
            }
            Direction::Down => {
                assert!((self.x + 1) as usize != n);
                x = self.x + 1;
                y = self.y;
            }
            Direction::Left => {
                assert!(self.y != 0);
                x = self.x;
                y = self.y - 1;
            }
            Direction::Right => {
                assert!((self.y + 1) as usize != n);
                x = self.x;
                y = self.y + 1;
            }
        };

        // old_pos.dir = Some(*self);

        Pos {
            id: self.id - 1,
            x: x,
            y: y,
            dir: self.dir, // TODO: change
        }
    }
}

use petgraph::{Graph, Undirected};

#[derive(Clone)]
struct Board {
    protein: Graph<Pos, Direction, Undirected, u32>,
    index: Vec<NodeIndex>,
}

impl Board {
    fn new() -> Self {
        Self {
            protein: Graph::new_undirected(),
            index: Vec::new(),
        }
    }
}

impl Board {
    fn get_last_index(&self) -> &NodeIndex {
        self.index
            .last()
            .expect("Chiamato solo nel passo induttivo")
    }

    fn get_last_aminoacid(&self) -> &Pos {
        let index = self.get_last_index();
        &self.protein[index]
    }

    fn add_pos(
        &mut self,
        problem: &ProteinFolding,
        dir: Direction,
    ) -> <ProteinFolding as Problem>::Cost {
        if self.index.len() == 0 {
            // Caso base
            // Controllare che la direzione iniziale non sia verso l'alto
            let mut old_init_pos = problem.next_firts_attempt.borrow_mut();

            let mut new_init_pos = *old_init_pos;

            if old_init_pos.x + 1 < problem.n {
                old_init_pos.move_dir(Direction::Left);
            } else if old_init_pos.y < problem.n {
                old_init_pos.x = 0;
                old_init_pos.move_dir(Direction::Down);
            } else {
                // fare attenzione a resettarlo a 0 quando finisco
                panic!("Combinations are ended"); // TODO: traslate better
            }
            new_init_pos.id = 0;
            new_init_pos.dir = dir;
            self.protein.add_node(new_init_pos);

            return 0.0.into();
        } else {
            // Passo induttivo
            let last_amin = self.get_last_aminoacid();
            let mut new_pos = last_amin.clone_move(dir);
            new_pos.id = last_amin.id + 1;
            new_pos.dir = dir;
            let b = self.protein.add_node(new_pos);
            self.protein.add_edge(*self.get_last_index(), b, dir);
            return 1.0.into(); // TODO: change
        }
    }
}

impl Deref for Board {
    type Target = Graph<Pos, Direction, Undirected>;

    fn deref(&self) -> &Self::Target {
        &self.protein
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.protein
    }
}

type NextPos = Pos;

struct ProteinFolding {
    aminoacids: Vec<AminoAcid>,
    n: usize,
    next_firts_attempt: RefCell<Pos>, // quando non ho piazzato nessun aminoacido allora provo tutte le n*n combinazioni
}

impl Problem for ProteinFolding {
    type State = Board;
    type Action = Direction;
    type Cost = OrderedFloat<f64>;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        todo!()
    }

    fn result(&self, board: &Self::State, direction: &Self::Action) -> (Self::State, Self::Cost) {
        let mut new_board = board.clone();
        let cost = new_board.add_pos(self, *direction);

        (new_board, cost)
    }
}

impl Utility for ProteinFolding {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        // Calcolare la distanza euclidiana dagli aminoacidi H non consecutivi e sottrai per gli aminoacidi
        todo!()
    }
}

fn main() {
    println!("Hello World!")
}

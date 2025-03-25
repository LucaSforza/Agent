use std::ops::{Deref, DerefMut};

use agent::{
    explorer::MinCostExplorer,
    problem::{Problem, StateExplorerProblem, Utility, WithSolution},
};
use ordered_float::OrderedFloat;
use petgraph::graph::NodeIndex;

#[derive(PartialEq, Eq)]
enum AminoAcid {
    H,
    P,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Hash)]
struct Pos {
    id: usize,
    x: isize,
    y: isize,
    dir: Direction,
}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Pos {
    fn new() -> Self {
        Self {
            id: 0,
            x: 0,
            y: 0,
            dir: Direction::Up,
        } // la direzione iniziale non importa perché poi la sostituisco
    }

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
}

use petgraph::{Graph, Undirected};

#[derive(Clone)]
struct Board {
    protein: Graph<Pos, Direction, Undirected, u32>,
    index: Vec<NodeIndex>,
}

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for i in self.index.iter() {
            self.protein[*i].hash(state);
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        if self.index.len() != other.index.len() {
            return false;
        }

        for (i, j) in self.index.iter().zip(other.index.iter()) {
            if self.protein[*i] != other.protein[*j] {
                return false;
            }
        }
        return true;
    }
}

impl Eq for Board {}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index.len())
    }
}

impl Board {
    fn new() -> Self {
        Self {
            protein: Default::default(),
            index: Vec::new(),
        }
    }

    fn init_state() -> Self {
        let mut board = Self::new();
        board.add_pos(Direction::Down); // Non importa quale sia la direzione iniziale, tanto non farà neanche parte del risultato
        board
    }

    fn suitable(&self, pos: &Pos, n: isize) -> bool {
        if pos.x >= n || pos.y >= n || pos.x <= -n || pos.y <= -n {
            return false;
        }

        for i in self.index.iter() {
            let a = &self.protein[*i];
            if a.x == pos.x && a.y == pos.y {
                return false;
            }
        }
        return true;
    }

    fn get_last_index(&self) -> &NodeIndex {
        self.index
            .last()
            .expect("Chiamato solo nel passo induttivo")
    }

    fn get_last_aminoacid(&self) -> &Pos {
        let index = self.get_last_index();
        &self.protein[*index]
    }

    fn search_for_contacts(&self, pos: &Pos) -> <ProteinFolding as Problem>::Cost {
        let mut contacts = 0.0;

        for i in self.index.iter() {
            let amin = &self.protein[*i];
            assert!(amin.x != pos.x || amin.y != pos.y);
            if ((amin.x - pos.x).abs() + (amin.y - pos.y).abs()) == 1 {
                contacts += 1.0;
            }
        }
        return contacts.into();
    }

    fn add_pos(
        &mut self,
        // problem: &ProteinFolding,
        dir: Direction,
    ) -> <ProteinFolding as Problem>::Cost {
        if self.index.len() == 0 {
            // Caso base

            let mut init_pos = Pos::new();

            init_pos.dir = dir;
            let index = self.protein.add_node(init_pos);
            self.index.push(index);

            return 0.0.into();
        } else {
            // Passo induttivo
            // assumiamo che il nuovo aminoacido non sia sopra ad un altro (già controllo in executable_actions)
            // e comunque ci stanno degli assert che controllano questo requisito
            let last_amin = self.get_last_aminoacid();
            let mut new_pos = last_amin.clone_move(dir);
            new_pos.id = last_amin.id + 1;
            new_pos.dir = dir;
            let b = self.protein.add_node(new_pos);
            let contancts = self.search_for_contacts(&new_pos);
            self.protein.add_edge(*self.get_last_index(), b, dir);
            self.index.push(b);
            return (2.0 - contancts.0).into();
            // Posso fare al piu due contatti per aminoacido H
            // quindi se voglio minimizzare il costo per massimizzare i contatti
            // allora sottraggo a 2 con il numero effettivo di contatti
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

struct ProteinFolding {
    aminoacids: Vec<AminoAcid>, // len is n
}

impl ProteinFolding {
    fn new(aminoacid: Vec<AminoAcid>) -> Self {
        Self {
            aminoacids: aminoacid,
        }
    }
}

impl Problem for ProteinFolding {
    type State = Board;
    type Action = Direction;
    type Cost = OrderedFloat<f64>;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        let last_aminoacid;
        if state.index.len() == 0 {
            return vec![
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ]
            .into_iter();
        } else {
            last_aminoacid = state.get_last_aminoacid();
        }

        let mut actions = Vec::with_capacity(3);
        let n = self.aminoacids.len() as isize;

        if state.suitable(&last_aminoacid.clone_move(Direction::Down), n) {
            actions.push(Direction::Down);
        }
        if state.suitable(&last_aminoacid.clone_move(Direction::Up), n) {
            actions.push(Direction::Up);
        }
        if state.suitable(&last_aminoacid.clone_move(Direction::Left), n) {
            actions.push(Direction::Left);
        }
        if state.suitable(&last_aminoacid.clone_move(Direction::Right), n) {
            actions.push(Direction::Right);
        }

        actions.into_iter()
    }

    fn result(&self, board: &Self::State, direction: &Self::Action) -> (Self::State, Self::Cost) {
        let mut new_board = board.clone();
        let cost = new_board.add_pos(*direction);

        (new_board, cost)
    }
}

impl WithSolution for ProteinFolding {
    fn is_goal(&self, state: &Self::State) -> bool {
        self.aminoacids.len() == state.index.len()
    }
}

impl Utility for ProteinFolding {
    fn heuristic(&self, _state: &Self::State) -> Self::Cost {
        // Calcolare la distanza euclidiana dagli aminoacidi H non consecutivi e sottrai per gli aminoacidi
        0.0.into()
    }
}

impl StateExplorerProblem for ProteinFolding {}

fn main() {
    let problem = ProteinFolding::new(vec![
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::P,
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::P,
    ]);

    let mut resolver = MinCostExplorer::new(problem);

    let init_state = Board::init_state();

    let r = resolver.search(init_state);

    println!("{}", r)
}

use std::ops::{Deref, DerefMut};

use agent::problem::{CostructSolution, Problem, SuitableState, Utility};
use petgraph::graph::NodeIndex;

#[derive(PartialEq, Eq, Clone)]
pub enum AminoAcid {
    H,
    P,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Hash, PartialEq)]
pub struct Pos {
    x: isize,
    y: isize,
}

impl Pos {
    fn new() -> Self {
        Self { x: 0, y: 0 }
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
pub struct Board {
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

    pub fn init_state(problem: &ProteinFolding) -> Self {
        let mut board = Self::new();
        board.add_pos(problem, Direction::Down); // Non importa quale sia la direzione iniziale, tanto non farà neanche parte del risultato
        board
    }

    fn suitable(&self, pos: &Pos) -> bool {
        for i in self.index.iter() {
            let a = &self.protein[*i];
            if a == pos {
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

    fn contacts(&self, problem: &ProteinFolding) -> u32 {
        let mut total_contacts = 0;

        for (i, index) in self.index.iter().enumerate() {
            let amin = &self.protein[*index];
            if problem.aminoacids[i] == AminoAcid::H {
                for dir in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    let neighbor_pos = amin.clone_move(dir);
                    for (j, other_index) in self.index.iter().enumerate() {
                        if i != j
                            && self.protein[*other_index] == neighbor_pos
                            && problem.aminoacids[j] == AminoAcid::H
                            && self
                                .protein
                                .find_edge_undirected(*index, *other_index)
                                .is_none()
                        {
                            total_contacts += 1;
                        }
                    }
                }
            }
        }

        total_contacts // Each contact is counted twice, so divide by 2
    }

    fn add_pos(
        &mut self,
        problem: &ProteinFolding,
        dir: Direction,
    ) -> <ProteinFolding as CostructSolution>::Cost {
        if self.index.len() == 0 {
            // Caso base

            let init_pos = Pos::new();

            let index = self.protein.add_node(init_pos);
            self.index.push(index);

            return 0;
        } else {
            // Passo induttivo
            // assumiamo che il nuovo aminoacido non sia sopra ad un altro (già controllo in executable_actions)
            // e comunque ci stanno degli assert che controllano questo requisito
            let last_amin = self.get_last_aminoacid();
            let new_pos = last_amin.clone_move(dir);
            let b = self.protein.add_node(new_pos);
            self.protein.add_edge(*self.get_last_index(), b, dir);
            self.index.push(b);

            let final_cost;
            if self.index.len() == problem.aminoacids.len() {
                let max_contacts = problem.h_numer * 3;
                let contacts = self.contacts(problem);
                assert!(max_contacts as i64 - contacts as i64 >= 0);
                final_cost = max_contacts - contacts;
            } else {
                final_cost = 0;
            }

            return 1 + final_cost;
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

pub struct ProteinFolding {
    aminoacids: Vec<AminoAcid>, // len is n
    h_numer: u32,
}

impl ProteinFolding {
    pub fn new(aminoacid: Vec<AminoAcid>) -> Self {
        let h_number = aminoacid
            .iter()
            .map(|x| if *x == AminoAcid::H { 1 } else { 0 })
            .sum();
        Self {
            aminoacids: aminoacid,
            h_numer: h_number,
        }
    }
}

impl Problem for ProteinFolding {
    type State = Board;
}

impl CostructSolution for ProteinFolding {
    type Action = Direction;
    type Cost = u32;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        if state.index.len() == 1 {
            // non importa dove vado la prima volta
            return vec![Direction::Up].into_iter();
        }

        let last_aminoacid;

        last_aminoacid = state.get_last_aminoacid();

        let mut actions = Vec::with_capacity(4);

        for dir in vec![
            Direction::Left,
            Direction::Down,
            Direction::Up,
            Direction::Right,
        ] {
            if state.suitable(&last_aminoacid.clone_move(dir)) {
                actions.push(dir);
            }
        }

        actions.into_iter()
    }

    fn result(&self, board: &Self::State, direction: &Self::Action) -> (Self::State, Self::Cost) {
        let mut new_board = board.clone();
        let cost = new_board.add_pos(self, *direction);

        (new_board, cost)
    }
}

impl SuitableState for ProteinFolding {
    fn is_suitable(&self, state: &Self::State) -> bool {
        self.aminoacids.len() == state.index.len()
    }
}

impl Utility for ProteinFolding {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        // Calcolare la distanza euclidiana dagli aminoacidi H non consecutivi e sottrai per gli aminoacidi H presenti
        let mut cost = 0.0;

        for (i, a) in state.index.iter().zip(self.aminoacids.iter()) {
            // se l'aminoacido è H allora controllo la distanza minima rispetto ad un altro aminoacido H che non sia adiacente
            if *a == AminoAcid::H {
                let amin = &state.protein[*i];
                let mut min_distrance = f64::INFINITY;
                for (j, b) in state.index.iter().zip(self.aminoacids.iter()) {
                    if i != j && *b == AminoAcid::H && state.find_edge_undirected(*i, *j).is_none()
                    {
                        // calcola la distanza euclidiana e aggiungila al costo
                        let other_amin = &state.protein[*j];
                        let distance = ((amin.x - other_amin.x).pow(2)
                            + (amin.y - other_amin.y).pow(2))
                            as f64;
                        let distance = distance.sqrt();
                        if distance < min_distrance {
                            min_distrance = distance;
                        }
                    }
                }
                // se l'ho trovato lo aggiungo al costo
                if min_distrance.is_finite() {
                    cost += min_distrance;
                }
            }
        }

        // le distanze sono duplicate, divido per 2
        let mut cost = (cost / 2.0).floor() as <ProteinFolding as CostructSolution>::Cost;

        // aggiungo al costo tutte le H non ancora posizionate, cosi quando sottraggo il risultato è consistente
        cost += (self
            .aminoacids
            .iter()
            .filter(|x| **x == AminoAcid::H)
            .count()) as u32;

        // sottraggo al costo il numero di H posizionati
        // questo perché vorrei che la soluzione ottima abbia 0 come euristica.
        // Se ogni H è stato posizionato con successo allora le loro distanze euclidiane sono 1
        // vengono sommate al costo e poi sottratte qua.
        cost - self.h_numer
    }
}

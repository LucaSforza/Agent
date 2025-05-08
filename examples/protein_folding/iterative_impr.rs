use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use crate::formulation::AminoAcid;

use agent::problem::{
    CostructSolution, InitState, Problem, StatePerturbation, SuitableState, Utility,
};

#[derive(Copy, Clone, Debug)]
pub enum Dir {
    Forward,
    Left,
    Right,
}

impl Dir {
    fn rotate(&self, other: Dir) -> Self {
        match other {
            Dir::Forward => panic!("cannot perturbate the state with the forward perturbation"),
            Dir::Left => match self {
                Dir::Forward => Dir::Left,
                Dir::Left => Dir::Right,
                Dir::Right => Dir::Forward,
            },
            Dir::Right => match self {
                Dir::Forward => Dir::Right,
                Dir::Left => Dir::Forward,
                Dir::Right => Dir::Left,
            },
        }
    }
}

impl Dir {
    fn next_orientation(&self, last_orientation: Orientation) -> Orientation {
        match self {
            Self::Forward => last_orientation,
            Self::Left => match last_orientation {
                Orientation::North => Orientation::Ovest,
                Orientation::Sud => Orientation::East,
                Orientation::East => Orientation::North,
                Orientation::Ovest => Orientation::Sud,
            },
            Self::Right => match last_orientation {
                Orientation::North => Orientation::East,
                Orientation::Sud => Orientation::Ovest,
                Orientation::East => Orientation::Sud,
                Orientation::Ovest => Orientation::North,
            },
        }
    }
}

enum Orientation {
    North,
    Sud,
    East,
    Ovest,
}

#[derive(Clone, Debug)]
pub struct Protein {
    placement: Vec<Dir>,
}

impl Protein {
    pub fn from_parts(placement: Vec<Dir>) -> Self {
        Self {
            placement: placement,
        }
    }

    fn perturb(&mut self, perturbation: &ProteinPerturbation) {
        let dir = self.placement[perturbation.index];
        self.placement[perturbation.index] = dir.rotate(perturbation.direction);
    }
}

struct AminoacidSet {
    h_number: i32,
    p_number: i32,
}

impl AminoacidSet {
    fn new() -> Self {
        Self {
            h_number: 0,
            p_number: 0,
        }
    }

    fn insert(&mut self, aminoacid: AminoAcid) {
        match aminoacid {
            AminoAcid::H => self.h_number += 1,
            AminoAcid::P => self.p_number += 1,
        }
    }
}

impl From<AminoAcid> for AminoacidSet {
    fn from(value: AminoAcid) -> Self {
        let mut result = Self::new();
        result.insert(value);
        result
    }
}

pub struct ProteinFolding {
    aminoacids: Vec<AminoAcid>,
    visited: RefCell<HashMap<(i32, i32), AminoacidSet>>,
}

impl ProteinFolding {
    pub fn new(aminoacids: Vec<AminoAcid>) -> Self {
        let a_number = aminoacids.len();
        Self {
            aminoacids: aminoacids,
            visited: HashMap::with_capacity(a_number).into(),
        }
    }
}

impl InitState for ProteinFolding {
    fn init_state(&self) -> Self::State {
        Protein::from_parts(vec![])
    }
}

impl Problem for ProteinFolding {
    type State = Protein;
}

impl CostructSolution for ProteinFolding {
    type Action = Dir;
    type Cost = i64;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        if state.placement.len() == 0 {
            vec![Dir::Forward].into_iter()
        } else {
            // TODO: aggiungere il fatto che se non ho mai svoltato allora posso andare solo a destra e non a sinistra
            vec![Dir::Forward, Dir::Left, Dir::Right].into_iter()
        }
    }

    fn result(&self, state: &Self::State, action: &Self::Action) -> (Self::State, Self::Cost) {
        let mut placements = state.placement.clone();
        placements.push(*action);
        let new_state = Protein::from_parts(placements);
        (new_state, 0)
    }
}

impl SuitableState for ProteinFolding {
    fn is_suitable(&self, state: &Self::State) -> bool {
        state.placement.len() == self.aminoacids.len()
    }
}

fn count_contacts(protein: &HashMap<(i32, i32), AminoacidSet>) -> i64 {
    let mut contacts: i64 = 0;

    let mut considerated = HashSet::new();

    for ((x, y), set) in protein.iter() {
        considerated.insert((*x, *y));
        if set.h_number > 0 {
            if considerated.get(&(x - 1, *y)).is_none() {
                if let Some(set_i) = protein.get(&(x - 1, *y)) {
                    if set_i.h_number > 0 {
                        contacts += 1 // set_i.h_number as i64 * set.h_number as i64;
                    }
                }
            }
            if considerated.get(&(x + 1, *y)).is_none() {
                if let Some(set_i) = protein.get(&(x + 1, *y)) {
                    if set_i.h_number > 0 {
                        contacts += 1 //set_i.h_number as i64 * set.h_number as i64;
                    }
                }
            }
            if considerated.get(&(*x, y - 1)).is_none() {
                if let Some(set_i) = protein.get(&(*x, y - 1)) {
                    if set_i.h_number > 0 {
                        contacts += 1 // set_i.h_number as i64 * set.h_number as i64;
                    }
                }
            }
            if considerated.get(&(*x, y + 1)).is_none() {
                if let Some(set_i) = protein.get(&(*x, y + 1)) {
                    if set_i.h_number > 0 {
                        contacts += 1 // set_i.h_number as i64 * set.h_number as i64;
                    }
                }
            }
        }
    }

    -contacts
}

impl Utility for ProteinFolding {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        let mut total_cost: i64 = 0;

        let mut visited_b = self.visited.borrow_mut();
        visited_b.clear();

        visited_b.insert((0, 0), self.aminoacids[0].into());

        let mut last_dir;
        let mut orientation = Orientation::North;
        let mut last_pos = (0, 0);

        for (i, dir) in state.placement.iter().enumerate().skip(1) {
            match orientation {
                Orientation::North => {
                    last_pos = (last_pos.0, last_pos.1 - 1);
                }
                Orientation::Sud => {
                    last_pos = (last_pos.0, last_pos.1 + 1);
                }
                Orientation::East => {
                    last_pos = (last_pos.0 + 1, last_pos.1);
                }
                Orientation::Ovest => {
                    last_pos = (last_pos.0 - 1, last_pos.1);
                }
            }
            last_dir = *dir;
            orientation = last_dir.next_orientation(orientation);
            if let Some(aminoacid_set) = visited_b.get_mut(&last_pos) {
                aminoacid_set.insert(self.aminoacids[i]);
                total_cost += 20; // penalty
            } else {
                visited_b.insert(last_pos, self.aminoacids[i].into());
            }
        }

        total_cost + count_contacts(&visited_b)
    }
}

pub struct ProteinPerturbation {
    index: usize,
    direction: Dir,
}

struct IteratorPerturbation {
    index: usize,
    end: usize,
    right: bool,
}

impl IteratorPerturbation {
    fn new(n: usize) -> Self {
        Self {
            index: 0,
            end: n,
            right: true,
        }
    }
}

impl Iterator for IteratorPerturbation {
    type Item = ProteinPerturbation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            None
        } else {
            let index = self.index;
            let direction = if self.right {
                // alla prima iterazione
                self.right = false;
                Dir::Right
            } else {
                // seconda iterazione
                self.right = true;
                self.index += 1;
                Dir::Left
            };
            Some(ProteinPerturbation {
                index: index,
                direction: direction,
            })
        }
    }
}

impl StatePerturbation for ProteinFolding {
    type Perturbation = ProteinPerturbation;

    fn perturbations(&self, _state: &Self::State) -> impl Iterator<Item = Self::Perturbation> {
        IteratorPerturbation::new(self.aminoacids.len())
    }

    fn perturb(&self, state: &Self::State, action: &Self::Perturbation) -> Self::State {
        let mut new_state = state.clone();
        new_state.perturb(action);
        new_state
    }
}

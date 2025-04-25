use std::rc::Rc;

use agent::problem::{CostructSolution, Problem, SuitableState, Utility};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AminoAcid {
    H,
    P,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Pos {
    x: isize,
    y: isize,
}

impl Pos {
    fn move_dir(&mut self, dir: Dir) {
        match dir {
            Dir::Up => self.x -= 1,
            Dir::Down => self.x += 1,
            Dir::Left => self.y -= 1,
            Dir::Right => self.y += 1,
        }
    }

    fn clone_move(&self, dir: Dir) -> Self {
        let mut new_pos = self.clone();
        new_pos.move_dir(dir);
        return new_pos;
    }
}

#[derive(Clone, Default)]
pub struct Board {
    last: Option<Rc<Self>>,
    pos: Pos,
    depth: usize,
    has_turned: bool,
    total_contacs: u32,
}

pub struct BoardIterator {
    head: Option<Rc<Board>>,
}

impl BoardIterator {
    fn from_parts(head: Option<Rc<Board>>) -> Self {
        Self { head: head }
    }

    fn new(board: Rc<Board>) -> Self {
        Self { head: board.into() }
    }

    fn void_iter() -> Self {
        Self { head: None }
    }
}

impl Iterator for BoardIterator {
    type Item = Rc<Board>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(board) = self.head.clone() {
            self.head = board.last.clone();
            Some(board)
        } else {
            None
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        if self.depth != other.depth {
            return false;
        }
        if self.pos != other.pos {
            return false;
        }
        let mut curr = self.last.clone();
        let mut curr_other = other.last.clone();

        while let (Some(c), Some(c_other)) = (curr, curr_other) {
            if c.pos != c_other.pos {
                return false;
            }
            curr = c.last.clone();
            curr_other = c_other.last.clone();
        }
        return true;
    }
}

impl Eq for Board {}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "contacts: {}", self.total_contacs)
    }
}

impl Board {
    fn suitable(&self, pos: &Pos) -> bool {
        if self.pos == *pos {
            return false;
        }

        let mut last = self.last.clone();

        while let Some(l) = last {
            if l.pos == *pos {
                return false;
            }
            last = l.last.clone();
        }
        return true;
    }

    fn iter(self: Rc<Self>) -> BoardIterator {
        BoardIterator {
            head: self.clone().into(),
        }
    }
}

fn default_heuristic(problem: &ProteinFolding, state: &Rc<Board>) -> u32 {
    problem.h_number - state.total_contacs
}

fn default_cost_f(problem: &ProteinFolding, state: &Rc<Board>, new_pos: &Pos) -> u32 {
    if problem.aminoacids[state.depth + 1] != AminoAcid::H {
        return 0;
    }
    // assume the aminoacid is H
    let max_attacts = 3;
    let mut attacts = 0;

    let mut last = state.last.clone();

    while let Some(l) = last {
        if problem.aminoacids[l.depth] == AminoAcid::H {
            if l.pos.x.abs_diff(new_pos.x) + l.pos.y.abs_diff(new_pos.y) == 1 {
                attacts += 1;
            }
        }
        last = l.last.clone()
    }
    max_attacts - attacts
}

pub struct ProteinFolding {
    aminoacids: Vec<AminoAcid>, // len is n
    h_number: u32,
    heuristic: fn(&ProteinFolding, &Rc<Board>) -> u32,
    cost_f: fn(&ProteinFolding, &Rc<Board>, &Pos) -> u32,
}

impl ProteinFolding {
    pub fn new(aminoacid: Vec<AminoAcid>) -> Self {
        let h_number = aminoacid
            .iter()
            .map(|x| if *x == AminoAcid::H { 1 } else { 0 })
            .sum();
        Self {
            aminoacids: aminoacid,
            h_number: h_number,
            heuristic: default_heuristic,
            cost_f: default_cost_f,
        }
    }
}

impl Problem for ProteinFolding {
    type State = Rc<Board>;
}

impl CostructSolution for ProteinFolding {
    type Action = Dir;
    type Cost = u32;

    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        if state.depth == 0 {
            // non importa dove vado la prima volta
            return vec![Dir::Up].into_iter();
        }

        let mut actions;
        if state.has_turned {
            actions = Vec::with_capacity(3);
            for dir in vec![Dir::Left, Dir::Down, Dir::Up, Dir::Right] {
                if state.suitable(&state.pos.clone_move(dir)) {
                    actions.push(dir);
                }
            }
        } else {
            // come prima svolta considerare solo la destra
            actions = Vec::with_capacity(2);
            for dir in vec![Dir::Down, Dir::Up, Dir::Right] {
                if state.suitable(&state.pos.clone_move(dir)) {
                    actions.push(dir);
                }
            }
        }
        actions.into_iter()
    }

    fn result(&self, board: &Self::State, dir: &Self::Action) -> (Self::State, Self::Cost) {
        let mut new_board = Board {
            last: board.clone().into(),
            depth: board.depth + 1,
            has_turned: board.has_turned,
            pos: board.pos.clone_move(*dir),
            total_contacs: board.total_contacs,
        };
        if *dir == Dir::Left || *dir == Dir::Right {
            new_board.has_turned = true;
        }

        let cost = (self.cost_f)(self, board, &new_board.pos);
        if self.aminoacids[board.depth + 1] == AminoAcid::H {
            if cost != 3 {
                new_board.total_contacs += 1;
            }
        }

        (Rc::new(new_board), cost)
    }
}

impl SuitableState for ProteinFolding {
    fn is_suitable(&self, state: &Self::State) -> bool {
        self.aminoacids.len() - 1 == state.depth
    }
}

impl Utility for ProteinFolding {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        (self.heuristic)(self, state)
    }
}

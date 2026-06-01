use agent::problem::{CostructSolution, InitState, Problem, SuitableState, Utility};

use bumpalo::Bump;

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
pub struct Board<'a> {
    last: Option<&'a Self>,
    pos: Pos,
    depth: usize,
    has_turned: bool,
    total_contacs: u32,
}

pub struct BoardIterator<'a> {
    head: Option<&'a Board<'a>>,
}

impl<'a> BoardIterator<'a> {
    fn from_parts(head: Option<&'a Board<'a>>) -> Self {
        Self { head: head }
    }

    fn new(board: &'a Board) -> Self {
        Self { head: board.into() }
    }

    fn void_iter() -> Self {
        Self { head: None }
    }
}

impl<'a> Iterator for BoardIterator<'a> {
    type Item = &'a Board<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(board) = self.head.clone() {
            self.head = board.last.clone();
            Some(board)
        } else {
            None
        }
    }
}

impl PartialEq for Board<'_> {
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

impl Eq for Board<'_> {}

impl std::fmt::Debug for Board<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "contacts: {}", self.total_contacs)
    }
}

impl<'a> Board<'a> {
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

    fn iter(self: &'a Self) -> BoardIterator<'a> {
        BoardIterator { head: self.into() }
    }
}

fn count_contacts_chain(chain: &[(Pos, bool)], pos: &Pos) -> u32 {
    let mut c = 0u32;
    for (p, is_h) in chain {
        if *is_h && (p.x - pos.x).abs() + (p.y - pos.y).abs() == 1 {
            c += 1;
        }
    }
    c
}

fn build_chain<'a>(state: &'a Board<'a>, problem: &ProteinFolding) -> Vec<(Pos, bool)> {
    let mut chain = Vec::new();
    let mut curr = Some(state);
    while let Some(b) = curr {
        chain.push((b.pos, problem.aminoacids[b.depth] == AminoAcid::H));
        curr = b.last;
    }
    chain.reverse();
    chain
}

// 1-step lookahead + relaxed count bound
fn h_lookahead1<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    let n = problem.aminoacids.len();
    let d = state.depth;
    let next = d + 1;
    if next >= n { return 0; }

    let chain = build_chain(state, problem);
    let tip = chain.last().unwrap().0;
    let dirs = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

    let mut result = 0u32;

    if problem.aminoacids[next] == AminoAcid::H {
        let mut min_cost = 3u32;
        for dir in dirs {
            let p = tip.clone_move(dir);
            if chain.iter().any(|(pos, _)| *pos == p) { continue; }
            let cost = 3u32.saturating_sub(count_contacts_chain(&chain, &p).min(3));
            if cost < min_cost { min_cost = cost; }
        }
        result += min_cost;
    }

    let mut h_count = chain.iter().filter(|(_, h)| *h).count() as u32;
    if problem.aminoacids[next] == AminoAcid::H { h_count += 1; }

    for i in (next + 1)..n {
        if problem.aminoacids[i] == AminoAcid::H {
            if h_count < 3 { result += 3 - h_count; }
            h_count += 1;
        }
    }
    result
}

// 2-step lookahead + relaxed count bound
pub fn h_lookahead2<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    let n = problem.aminoacids.len();
    let d = state.depth;
    if d + 1 >= n { return 0; }

    let chain = build_chain(state, problem);
    let tip = chain.last().unwrap().0;
    let dirs = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];
    let step1_h = problem.aminoacids[d + 1] == AminoAcid::H;
    let step2_exists = d + 2 < n;
    let step2_h = step2_exists && problem.aminoacids[d + 2] == AminoAcid::H;

    let mut min_total = u32::MAX;

    for dir1 in dirs {
        let p1 = tip.clone_move(dir1);
        if chain.iter().any(|(pos, _)| *pos == p1) { continue; }

        let cost1 = if step1_h {
            3u32.saturating_sub(count_contacts_chain(&chain, &p1).min(3))
        } else { 0 };

        if !step2_exists {
            min_total = min_total.min(cost1);
            continue;
        }

        let mut min_cost2 = 3u32;
        for dir2 in dirs {
            let p2 = p1.clone_move(dir2);
            if chain.iter().any(|(pos, _)| *pos == p2) { continue; }
            if p2 == p1 { continue; }

            if step2_h {
                let c2 = count_contacts_chain(&chain, &p2)
                    + if step1_h && (p1.x - p2.x).abs() + (p1.y - p2.y).abs() == 1 { 1 } else { 0 };
                min_cost2 = min_cost2.min(3u32.saturating_sub(c2.min(3)));
            } else {
                min_cost2 = 0;
            }
        }
        if min_cost2 < 3 || !step2_h {
            min_total = min_total.min(cost1 + min_cost2);
        }
    }

    let mut result = if min_total == u32::MAX { 0 } else { min_total };

    let mut h_count = chain.iter().filter(|(_, h)| *h).count() as u32;
    if step1_h { h_count += 1; }
    if step2_h { h_count += 1; }

    for i in (d + 3)..n {
        if problem.aminoacids[i] == AminoAcid::H {
            if h_count < 3 { result += 3 - h_count; }
            h_count += 1;
        }
    }
    result
}

fn min_k_steps(
    chain: &mut Vec<(Pos, bool)>,
    problem: &ProteinFolding,
    next_depth: usize,
    remaining: usize,
) -> u32 {
    if remaining == 0 || next_depth >= problem.aminoacids.len() {
        return 0;
    }
    let tip = chain.last().unwrap().0;
    let is_h = problem.aminoacids[next_depth] == AminoAcid::H;
    let dirs = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

    let mut best = u32::MAX;
    for dir in dirs {
        let new_pos = tip.clone_move(dir);
        if chain.iter().any(|(p, _)| *p == new_pos) { continue; }
        let contacts = count_contacts_chain(chain, &new_pos);
        let cost = if is_h { 3u32.saturating_sub(contacts.min(3)) } else { 0 };
        chain.push((new_pos, is_h));
        let future = min_k_steps(chain, problem, next_depth + 1, remaining - 1);
        chain.pop();
        let total = cost + future;
        if total < best { best = total; }
    }
    if best == u32::MAX { 0 } else { best }
}

// 3-step lookahead + relaxed count bound
pub fn h_lookahead3<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    let n = problem.aminoacids.len();
    let d = state.depth;
    if d + 1 >= n { return 0; }

    let mut chain = build_chain(state, problem);
    let k = 3;
    let mut result = min_k_steps(&mut chain, problem, d + 1, k);

    let end = n.min(d + 1 + k);
    let mut h_count = chain.iter().filter(|(_, h)| *h).count() as u32;
    for i in (d + 1)..end {
        if problem.aminoacids[i] == AminoAcid::H { h_count += 1; }
    }
    for i in end..n {
        if problem.aminoacids[i] == AminoAcid::H {
            if h_count < 3 { result += 3 - h_count; }
            h_count += 1;
        }
    }
    result
}

fn default_heuristic<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    h_lookahead1(problem, state)
}

pub fn old_heuristic<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    problem.h_number - state.total_contacs
}

fn default_cost_f<'a>(problem: &ProteinFolding, state: &'a Board<'a>, new_pos: &Pos) -> u32 {
    if problem.aminoacids[state.depth + 1] != AminoAcid::H {
        return 0;
    }
    // assume the aminoacid is H
    let max_attacts = 3;
    let mut attacts = 0;

    let mut last = state.last.clone();

    while let Some(l) = last {
        if problem.aminoacids[l.depth] == AminoAcid::H {
            let dx = (l.pos.x - new_pos.x) as f64;
            let dy = (l.pos.y - new_pos.y) as f64;
            let distance = (dx * dx + dy * dy).sqrt();
            if (distance - 1.0).abs() < f64::EPSILON {
                attacts += 1;
            }
        }
        last = l.last.clone()
    }
    max_attacts - attacts
}

pub struct ProteinFolding<'a> {
    pub aminoacids: Vec<AminoAcid>, // len is n
    h_number: u32,
    heuristic: fn(&ProteinFolding, &'a Board<'a>) -> u32,
    cost_f: fn(&ProteinFolding, &'a Board<'a>, &Pos) -> u32,
    arena: &'a Bump,
}

impl<'a> ProteinFolding<'a> {
    pub fn new(aminoacid: Vec<AminoAcid>, arena: &'a Bump) -> Self {
        let h_number = aminoacid
            .iter()
            .map(|x| if *x == AminoAcid::H { 1 } else { 0 })
            .sum();
        Self {
            aminoacids: aminoacid,
            h_number: h_number,
            heuristic: default_heuristic,
            cost_f: default_cost_f,
            arena: arena,
        }
    }

    pub fn with_heuristic(
        aminoacid: Vec<AminoAcid>,
        arena: &'a Bump,
        heuristic: fn(&ProteinFolding, &'a Board<'a>) -> u32,
    ) -> Self {
        let h_number = aminoacid
            .iter()
            .map(|x| if *x == AminoAcid::H { 1 } else { 0 })
            .sum();
        Self {
            aminoacids: aminoacid,
            h_number: h_number,
            heuristic: heuristic,
            cost_f: default_cost_f,
            arena: arena,
        }
    }
}

impl<'a> Problem for ProteinFolding<'a> {
    type State = &'a Board<'a>;
}

impl<'a> CostructSolution for ProteinFolding<'a> {
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
        let mut new_board: Board<'a> = Board {
            last: (*board).into(),
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

        (self.arena.alloc(new_board), cost)
    }
}

impl SuitableState for ProteinFolding<'_> {
    fn is_suitable(&self, state: &Self::State) -> bool {
        self.aminoacids.len() - 1 == state.depth
    }
}

impl Utility for ProteinFolding<'_> {
    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        (self.heuristic)(self, state)
    }
}

impl<'a> InitState for ProteinFolding<'a> {
    fn init_state(&self) -> Self::State {
        self.arena.alloc(Default::default())
    }
}

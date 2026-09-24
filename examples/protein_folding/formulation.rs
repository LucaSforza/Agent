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

fn build_chain<'a>(state: &'a Board<'a>, problem: &ProteinFolding) -> Vec<(Pos, bool)> {
    // This chain contains exactly one entry per residue already placed.
    let mut chain = Vec::with_capacity(state.depth + 1);
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
    if next >= n {
        return 0;
    }

    let chain = build_chain(state, problem);
    let tip = chain.last().unwrap().0;
    let dirs = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

    let mut result = 0u32;

    if problem.aminoacids[next] == AminoAcid::H {
        let mut min_cost = 3u32;
        for dir in dirs {
            let p = tip.clone_move(dir);
            let (occupied, contacts) = scan_chain_for_move(&chain, &p, true);
            if occupied {
                continue;
            }
            let cost = 3u32.saturating_sub(contacts.min(3));
            if cost < min_cost {
                min_cost = cost;
            }
        }
        result += min_cost;
    }

    let mut h_count = chain.iter().filter(|(_, h)| *h).count() as u32;
    if problem.aminoacids[next] == AminoAcid::H {
        h_count += 1;
    }

    for i in (next + 1)..n {
        if problem.aminoacids[i] == AminoAcid::H {
            if h_count < 3 {
                result += 3 - h_count;
            }
            h_count += 1;
        }
    }
    result
}

// 2-step lookahead + relaxed count bound
pub fn h_lookahead2<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    let n = problem.aminoacids.len();
    let d = state.depth;
    if d + 1 >= n {
        return 0;
    }

    let chain = build_chain(state, problem);
    let tip = chain.last().unwrap().0;
    let dirs = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];
    let step1_h = problem.aminoacids[d + 1] == AminoAcid::H;
    let step2_exists = d + 2 < n;
    let step2_h = step2_exists && problem.aminoacids[d + 2] == AminoAcid::H;

    let mut min_total = u32::MAX;

    for dir1 in dirs {
        let p1 = tip.clone_move(dir1);
        let (occupied, contacts1) = scan_chain_for_move(&chain, &p1, step1_h);
        if occupied {
            continue;
        }

        let cost1 = if step1_h {
            3u32.saturating_sub(contacts1.min(3))
        } else {
            0
        };

        if !step2_exists {
            min_total = min_total.min(cost1);
            continue;
        }

        let mut min_cost2 = 3u32;
        for dir2 in dirs {
            let p2 = p1.clone_move(dir2);
            let (occupied, contacts2) = scan_chain_for_move(&chain, &p2, step2_h);
            if occupied {
                continue;
            }
            if p2 == p1 {
                continue;
            }

            if step2_h {
                min_cost2 = min_cost2.min(3u32.saturating_sub(contacts2.min(3)));
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
    if step1_h {
        h_count += 1;
    }
    if step2_h {
        h_count += 1;
    }

    for i in (d + 3)..n {
        if problem.aminoacids[i] == AminoAcid::H {
            if h_count < 3 {
                result += 3 - h_count;
            }
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
        let (occupied, contacts) = scan_chain_for_move(chain, &new_pos, is_h);
        if occupied {
            continue;
        }
        let cost = if is_h {
            3u32.saturating_sub(contacts.min(3))
        } else {
            0
        };
        chain.push((new_pos, is_h));
        let future = min_k_steps(chain, problem, next_depth + 1, remaining - 1);
        chain.pop();
        let total = cost + future;
        if total < best {
            best = total;
        }
    }
    if best == u32::MAX {
        0
    } else {
        best
    }
}

// Check self-avoidance and count non-bonded H contacts in one chain walk.
// The last chain entry is the covalent parent of `pos`.
fn scan_chain_for_move(chain: &[(Pos, bool)], pos: &Pos, count_contacts: bool) -> (bool, u32) {
    let mut contacts = 0;
    let last_index = chain.len().saturating_sub(1);
    for (index, (chain_pos, is_h)) in chain.iter().enumerate() {
        if *chain_pos == *pos {
            return (true, contacts);
        }
        if count_contacts
            && index != last_index
            && *is_h
            && (chain_pos.x - pos.x).abs() + (chain_pos.y - pos.y).abs() == 1
        {
            contacts += 1;
        }
    }
    (false, contacts)
}

fn exact_steps<'a>(problem: &ProteinFolding, state: &'a Board<'a>, lookahead: usize) -> u32 {
    let mut chain = Vec::with_capacity(state.depth + 1 + lookahead);
    let mut curr = Some(state);
    while let Some(board) = curr {
        chain.push((board.pos, problem.aminoacids[board.depth] == AminoAcid::H));
        curr = board.last;
    }
    chain.reverse();
    min_k_steps(&mut chain, problem, state.depth + 1, lookahead)
}

// 3-step lookahead + relaxed count bound
pub fn h_lookahead3<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    let n = problem.aminoacids.len();
    let d = state.depth;
    if d + 1 >= n {
        return 0;
    }

    let mut result = exact_steps(problem, state, 3);
    let placed_h = problem.h_prefix[d + 1];
    let mut h_count = placed_h[0] + placed_h[1];

    let end = n.min(d + 4);
    for i in (d + 1)..end {
        if problem.aminoacids[i] == AminoAcid::H {
            h_count += 1;
        }
    }
    for i in end..n {
        if problem.aminoacids[i] == AminoAcid::H {
            if h_count < 3 {
                result += 3 - h_count;
            }
            h_count += 1;
        }
    }
    result
}

// The square lattice is bipartite: residue i can touch only residues of the
// opposite index parity. The covalent predecessor never counts as a contact.
// This suffix bound dominates the old H-count bound and costs one lookup.
pub fn h_lookahead3_parity<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    h_lookahead_parity(problem, state, 3)
}

pub fn h_lookahead4_parity<'a>(problem: &ProteinFolding, state: &'a Board<'a>) -> u32 {
    h_lookahead_parity(problem, state, 4)
}

fn h_lookahead_parity<'a>(problem: &ProteinFolding, state: &'a Board<'a>, lookahead: usize) -> u32 {
    if state.depth + 1 >= problem.aminoacids.len() {
        return 0;
    }
    let placed_h = problem.h_prefix[state.depth + 1];
    let suffix_start = (state.depth + 1 + lookahead).min(problem.aminoacids.len());

    // Every future H-H contact uses one even and one odd residue. Previously
    // placed H can offer at most two non-bonded sides, or three at chain ends.
    let mut past_capacity = [2 * placed_h[0], 2 * placed_h[1]];
    if problem.aminoacids[0] == AminoAcid::H {
        past_capacity[0] += 1;
    }
    if state.depth > 0 && problem.aminoacids[state.depth] == AminoAcid::H {
        past_capacity[state.depth % 2] += 1;
    }
    let future_h = [
        problem.h_by_parity[0] - placed_h[0],
        problem.h_by_parity[1] - placed_h[1],
    ];
    let future_base_cost = 3 * (future_h[0] + future_h[1]);
    let max_future_contacts = (3 * future_h[0] + past_capacity[0])
        .min(3 * future_h[1] + past_capacity[1])
        .min(future_base_cost);
    let capacity_bound = future_base_cost - max_future_contacts;

    let exact_h = problem.h_prefix[suffix_start][0] + problem.h_prefix[suffix_start][1]
        - placed_h[0]
        - placed_h[1];
    let suffix_bound = problem.parity_tail_bound[suffix_start];
    // Exact lookahead cannot exceed three per H. Skip it when the global
    // capacity bound already dominates even that optimistic upper limit.
    if capacity_bound >= 3 * exact_h + suffix_bound {
        return capacity_bound;
    }
    let lookahead_bound = exact_steps(problem, state, lookahead) + suffix_bound;
    lookahead_bound.max(capacity_bound)
}

fn parity_tail_bound(aminoacids: &[AminoAcid]) -> Vec<u32> {
    let mut bounds = vec![0; aminoacids.len() + 1];
    let mut preceding_h = [0u32; 2];
    for (index, acid) in aminoacids.iter().enumerate() {
        if *acid == AminoAcid::H {
            if index > 0 {
                let opposite_h = preceding_h[1 - index % 2];
                // If H, the parent is included in opposite_h but bonded.
                let bonded_parent = u32::from(aminoacids[index - 1] == AminoAcid::H);
                bounds[index] = 3u32.saturating_sub(opposite_h - bonded_parent);
            }
            preceding_h[index % 2] += 1;
        }
    }
    for index in (0..aminoacids.len()).rev() {
        bounds[index] += bounds[index + 1];
    }
    bounds
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
            if (l.pos.x - new_pos.x).abs() + (l.pos.y - new_pos.y).abs() == 1 {
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
    h_by_parity: [u32; 2],
    h_prefix: Vec<[u32; 2]>,
    parity_tail_bound: Vec<u32>,
    heuristic: fn(&ProteinFolding, &'a Board<'a>) -> u32,
    cost_f: fn(&ProteinFolding, &'a Board<'a>, &Pos) -> u32,
    arena: &'a Bump,
}

impl<'a> ProteinFolding<'a> {
    pub fn new(aminoacid: Vec<AminoAcid>, arena: &'a Bump) -> Self {
        Self::with_heuristic(aminoacid, arena, default_heuristic)
    }

    pub fn with_heuristic(
        aminoacid: Vec<AminoAcid>,
        arena: &'a Bump,
        heuristic: fn(&ProteinFolding, &'a Board<'a>) -> u32,
    ) -> Self {
        let mut h_prefix = Vec::with_capacity(aminoacid.len() + 1);
        h_prefix.push([0; 2]);
        for (index, acid) in aminoacid.iter().enumerate() {
            let mut counts = *h_prefix.last().unwrap();
            if *acid == AminoAcid::H {
                counts[index % 2] += 1;
            }
            h_prefix.push(counts);
        }
        let h_by_parity = *h_prefix.last().unwrap();
        Self {
            parity_tail_bound: parity_tail_bound(&aminoacid),
            aminoacids: aminoacid,
            h_number: h_by_parity[0] + h_by_parity[1],
            h_by_parity,
            h_prefix,
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
        let directions = if state.depth == 0 {
            // The first move fixes rotational symmetry.
            [Some(Dir::Up), None, None, None]
        } else if state.has_turned {
            [
                Some(Dir::Left),
                Some(Dir::Down),
                Some(Dir::Up),
                Some(Dir::Right),
            ]
        } else {
            // Before the first turn, keep only one mirrored turn direction.
            [Some(Dir::Down), Some(Dir::Up), Some(Dir::Right), None]
        };

        directions
            .into_iter()
            .filter_map(move |dir| dir.filter(|dir| state.suitable(&state.pos.clone_move(*dir))))
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

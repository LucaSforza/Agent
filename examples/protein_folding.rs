use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use agent::{
    explorer::Explorer,
    frontier::{
        AStarBackend, BestFirstBackend, DequeBackend, FrontierBackend, MinCostBackend, StackBackend,
    },
    problem::{Problem, StateExplorerProblem, Utility, WithSolution},
};
use ordered_float::OrderedFloat;
use petgraph::graph::NodeIndex;

#[derive(PartialEq, Eq, Clone)]
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
    h_numer: u32,
}

impl ProteinFolding {
    fn new(aminoacid: Vec<AminoAcid>) -> Self {
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

        let mut h_numer = self.h_numer;

        for aminoacid in self.aminoacids.iter().skip(state.index.len()) {
            // per ogni aminoacido rimanente non ancora posizionato, se non è H allora
            // sono sicuro che il suo costo sarà 2, altrimento lo sottraggo ad h_number
            // che contiene il numero di aminoacidi H
            if *aminoacid != AminoAcid::H {
                cost += 2.0;
            } else {
                h_numer -= 1;
            }
        }

        // sottraggo al costo il numero di H posizionati
        // questo perché vorrei che la soluzione ottima abbia 0 come euristica.
        // Se ogni H è stato posizionato con successo allora le loro distanze euclidiane sono 1
        // vengono sommate al costo e poi sottratte qua.
        (cost - h_numer as f64).into()
    }
}

impl StateExplorerProblem for ProteinFolding {}

fn run_example<B: FrontierBackend<ProteinFolding> + std::fmt::Debug>(protein: &Vec<AminoAcid>) {
    let problem = ProteinFolding::new(protein.clone());

    let mut resolver = Explorer::<ProteinFolding, B>::new(problem);
    let init_state = Board::init_state();

    let r = resolver.search(init_state);
    println!("{}", r);
    print_solution(protein, r.actions.unwrap());
}

type MinCost = MinCostBackend<ProteinFolding>;
type DFS = StackBackend<ProteinFolding>;
type BFS = DequeBackend<ProteinFolding>;
type AStar = AStarBackend<ProteinFolding>;
type BestFirst = BestFirstBackend<ProteinFolding>;

fn print_solution(protein: &Vec<AminoAcid>, solution: Vec<Direction>) {
    // Genera le posizioni originali degli aminoacidi
    let mut positions = vec![(0, 0)];
    let mut current_pos = (0, 0);
    for dir in solution {
        current_pos = match dir {
            Direction::Up => (current_pos.0, current_pos.1 + 1),
            Direction::Down => (current_pos.0, current_pos.1 - 1),
            Direction::Left => (current_pos.0 - 1, current_pos.1),
            Direction::Right => (current_pos.0 + 1, current_pos.1),
        };
        positions.push(current_pos);
    }

    // Mappa posizioni -> indice
    let pos_to_index: HashMap<(i32, i32), usize> = positions
        .iter()
        .enumerate()
        .map(|(i, pos)| (*pos, i))
        .collect();

    // Conta H adiacenti non collegati
    let mut adjacency_pairs = HashSet::new();
    for (i, aa) in protein.iter().enumerate() {
        if *aa == AminoAcid::H {
            let (x, y) = positions[i];
            // Controlla tutte e quattro le direzioni
            for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let neighbor_pos = (x + dx, y + dy);
                if let Some(&j) = pos_to_index.get(&neighbor_pos) {
                    if protein[j] == AminoAcid::H && j != i {
                        // Escludi coppie consecutive
                        if (i as i32 - j as i32).abs() != 1 {
                            let pair = if i < j { (i, j) } else { (j, i) };
                            adjacency_pairs.insert(pair);
                        }
                    }
                }
            }
        }
    }

    // Crea griglia scalata per legami
    let mut grid: HashMap<(i32, i32), char> = HashMap::new();
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);

    // Aggiungi aminoacidi alla griglia
    for (i, &(x, y)) in positions.iter().enumerate() {
        let scaled_x = x * 2;
        let scaled_y = y * 2;
        grid.insert(
            (scaled_x, scaled_y),
            match protein[i] {
                AminoAcid::H => 'H',
                AminoAcid::P => 'P',
            },
        );
        min_x = min_x.min(scaled_x);
        max_x = max_x.max(scaled_x);
        min_y = min_y.min(scaled_y);
        max_y = max_y.max(scaled_y);
    }

    // Aggiungi legami alla griglia
    for i in 0..positions.len() - 1 {
        let (x1, y1) = positions[i];
        let (x2, y2) = positions[i + 1];
        let (sx1, sy1) = (x1 * 2, y1 * 2);
        let (sx2, sy2) = (x2 * 2, y2 * 2);

        if x1 != x2 {
            grid.insert((sx1 + (sx2 - sx1).signum(), sy1), '-');
        } else {
            grid.insert((sx1, sy1 + (sy2 - sy1).signum()), '|');
        }
    }

    // Stampa la griglia
    for y in (min_y..=max_y).rev() {
        let mut line = String::new();
        for x in min_x..=max_x {
            line.push(*grid.get(&(x, y)).unwrap_or(&' '));
        }
        println!("{}", line);
    }

    // Stampa il conteggio
    println!("\nEnergy: {}", -(adjacency_pairs.len() as isize));
}

fn main() {
    let protein = vec![
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::H,
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::P,
        AminoAcid::P,
        AminoAcid::H,
        AminoAcid::P,
    ];

    println!("MinCost:");
    run_example::<MinCost>(&protein);
    println!("BFS:");
    run_example::<BFS>(&protein);
    println!("DFS:");
    run_example::<DFS>(&protein);
    println!("AStar:");
    run_example::<AStar>(&protein);
    println!("BestFirst:");
    run_example::<BestFirst>(&protein);
    println!("Iterative:");
    let problem = ProteinFolding::new(protein.clone());

    let mut resolver = Explorer::<ProteinFolding, DFS>::new(problem);
    let init_state = Board::init_state();

    let r = resolver.iterative_search(init_state, 300);
    println!("{}", r);
    print_solution(&protein, r.actions.unwrap());
}

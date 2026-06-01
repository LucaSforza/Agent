# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
cargo build                       # debug build
cargo build --release             # release build
cargo test                        # all tests
cargo test --release              # tests in release (faster)
cargo t --release                 # shorthand
cargo run --example n_queen --release -- perform -n 8 -i 100 -r 100
cargo run --example protein_folding --release -- run-protein HHPHPPHHHPPPPHHPP
cargo run --example csp --release
# run.sh wraps example execution:
./run.sh n_queen perform -n 8 -i 100 -r 100
```

Run single test:
```bash
cargo test test_vacuum_bfs -- --nocapture
```

## Architecture

Problem-solving framework. Two families of algorithms share core trait definitions in `src/problem.rs`.

### Traits (`src/problem.rs`)

- **`Problem`** — marker trait, associated `State` type
- **`CostructSolution`** — defines actions, transition `result()`, and `Cost` type
- **`Utility`** — `heuristic()` for A*/greedy
- **`SuitableState`** — goal/suitability check
- **`StatePerturbation` / `RandomPerturbation`** — neighborhood generation for iterative improvement
- **`Crossover` / `MutateGene`** — genetic algorithm operators (blanket impl of `MutateGene` via `RandomPerturbation`)

### Module tree

```
src/
├── lib.rs                     # pub mod improve, problem, statexplorer
├── problem.rs                 # core traits (no algorithm code)
├── improve/
│   ├── mod.rs
│   ├── algorithms.rs          # SteepestDescend, HillClimbing, SimulatedAnnealing, LocalBeam, GeneticAlgorithm
│   └── resolver.rs            # Resolver<I, P> wraps an ImprovingAlgorithm, adds restart logic
└── statexplorer/
    ├── mod.rs
    ├── node.rs                # Node with arena allocation (bumpalo), dead-node marking, plan reconstruction
    ├── frontier.rs            # Backend trait + Deque/Stack/PriorityBackends, AStar/MinCost/BestFirst policies
    └── resolver.rs            # Explorer (graph search with explored set) + TreeExplorer (no explored set)
```

### State space search (`statexplorer/`)

- `Explorer<P, Backend>` — graph search, tracks explored states via `HashSet`
- `TreeExplorer<P, Backend>` — tree search, no explored set
- Backends: `DequeBackend` (BFS), `StackBackend` (DFS), `MinCostBackend` (UCS), `BestFirstBackend` (greedy), `AStarBackend` (A*)
- Type aliases at bottom of `resolver.rs`: `BFSExplorer`, `DFSExplorer`, `AStarExplorer`, etc.
- Nodes allocated in `bumpalo::Bump` arena — reset arena between searches
- Supports iterative deepening search via `iterative_search(max_limit)`

### Iterative improvement (`improve/`)

- `ImprovingAlgorithm<P>` trait — single `attempt()` call
- Algorithms: `SteepestDescend`, `HillClimbing` (with optional lateral moves), `SimulatedAnnealing` (pluggable cooling function), `LocalBeam`, `GeneticAlgorithm`
- `Resolver<I, P>` wraps algorithm, provides `resolve()` (single attempt) and `resolve_restart()` (multiple restarts, tracks best)

### Examples

- **n_queen** — N-Queens, demonstrates both search and improvement families. CLI via clap: `perform` (stats), `one-time` (single run), `state-exploration` (search).
- **protein_folding** — Protein folding on 2D grid (HP model) using tree search. CLI: `run-protein <SEQUENCE>` or `rand-test`.
  - `ProteinFolding` stores heuristic + cost as function pointers, swappable via `with_heuristic()`
  - Heuristic functions in `formulation.rs`:
    - `old_heuristic` — `h_total - total_contacts`. Inadmissible. Fast, suboptimal.
    - `h_lookahead1` — 1-step geometric lookahead + relaxed count bound. Admissible.
    - `h_lookahead2` — 2-step lookahead + relaxed bound. Admissible, ~36% MinCost iterations.
    - `h_lookahead3` — 3-step recursive lookahead + relaxed bound. Admissible, ~19% MinCost iterations.
  - Multi-step lookahead uses recursive `min_k_steps()` with push/pop on `Vec<(Pos, bool)>` chain (no arena needed for heuristic).
- **csp** — Constraint satisfaction with steepest descent.

### Tests

- `test_simple_vacuum` — 2-position vacuum, BFS/DFS with exact action sequences
- `test_vacuum` — 32x32 grid vacuum, all search algorithms, iterative deepening, Esposito layout

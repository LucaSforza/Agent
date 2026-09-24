# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2021 problem-solving framework. Core traits live in
`src/problem.rs`; public modules are declared by `src/lib.rs`. Search algorithms
are under `src/statexplorer/`, while iterative-improvement algorithms are under
`src/improve/`. Runnable demonstrations are in `examples/` (`n_queen`, `csp`,
and `protein_folding`). Keep integration tests in `tests/`, with each test file
defining its small problem formulation inline. Supporting notes and assignment
write-ups are Markdown files at the repository root.

## Build, Test, and Development Commands

Use the standard Cargo workflow:

```bash
cargo build                         # debug build
cargo build --release               # optimized build
cargo test                          # run all integration tests
cargo test test_vacuum_bfs -- --nocapture  # run one test with output
cargo run --example csp --release   # run the CSP example
./run.sh n_queen perform -n 8 -i 100 -r 100
```

`run.sh` is a convenience wrapper for release-mode examples. `build.sh` builds
the Docker image and `run_docker.sh` runs an example inside it.

## Coding Style & Naming Conventions

Write idiomatic Rust and format changes with `cargo fmt` before submitting.
Use four-space indentation, `snake_case` for modules, functions, variables, and
test names, and `UpperCamelCase` for types and traits. Keep algorithm code in
the appropriate family module; put shared problem contracts in `problem.rs`.
Prefer focused, generic interfaces consistent with the existing trait-based
design rather than example-specific library APIs.

## Testing Guidelines

Add or update an integration test in `tests/` for behavior changes. Name test
functions descriptively, e.g. `test_already_at_goal_bfs`. Cover normal behavior
and relevant edge cases such as empty actions, immediate goals, cycles, or
restart behavior. Use seeded `StdRng` where randomness affects assertions so
tests remain reproducible. Run `cargo test` (and `cargo test --release` when
performance-sensitive) before opening a PR.

## Commit & Pull Request Guidelines

Recent commits use brief, lowercase imperative summaries, sometimes in Italian
(for example, `add TODO.md with bumpalo encapsulation analysis`). Keep commits
small and scoped. PRs should explain the behavioral change, identify affected
algorithms/examples, link an issue when applicable, and include command output
or screenshots only when they clarify user-visible behavior.

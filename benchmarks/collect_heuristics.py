"""Benchmark current protein-folding heuristics on identical sequences."""

import argparse
import csv
import io
import json
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
SEQUENCES = (
    "PHHPHPPHP",
    "HHPHPPHHHPPPPHH",
    "HHPHPHHHPPPPHHPHPHPP",
)
HEURISTICS = (
    "legacy",
    "one_step",
    "lookahead2",
    "lookahead3",
    "lookahead3_parity",
    "lookahead4_parity",
)
FIELDS = ("heuristic", "sequence", "run", "duration_ns", "iterations", "energy")


def command(args, **kwargs):
    return subprocess.run(args, check=True, text=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repetitions", type=int, default=7)
    parser.add_argument("--sequences", nargs="+", default=SEQUENCES)
    parser.add_argument(
        "--output", type=Path, default=ROOT / "benchmarks" / "heuristics.csv"
    )
    args = parser.parse_args()
    if args.repetitions < 7:
        parser.error("--repetitions must be at least 7")
    if not args.sequences or any(not s or set(s) - {"H", "P"} for s in args.sequences):
        parser.error("--sequences must contain nonempty H/P strings")

    source_commit = command(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True).stdout.strip()
    if command(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True).stdout.strip():
        parser.error("working tree must be clean before benchmark measurements")

    rows = []
    with tempfile.TemporaryDirectory(prefix="protein-heuristics-") as scratch:
        target = Path(scratch) / "target"
        command(
            [
                "cargo",
                "build",
                "--offline",
                "--locked",
                "--release",
                "--example",
                "protein_heuristic_bench",
                "--target-dir",
                str(target),
            ],
            cwd=ROOT,
        )
        binary = target / "release" / "examples" / "protein_heuristic_bench"
        for sequence in args.sequences:
            energies = {}
            for heuristic in HEURISTICS:
                print(f"{sequence} {heuristic}", flush=True)
                output = command(
                    [str(binary), sequence, heuristic, str(args.repetitions)],
                    cwd=ROOT,
                    capture_output=True,
                ).stdout
                run_rows = list(csv.DictReader(io.StringIO(output)))
                rows.extend(run_rows)
                energies[heuristic] = {int(row["energy"]) for row in run_rows}

            if any(len(values) != 1 for values in energies.values()):
                raise RuntimeError(f"inconsistent energies across runs for {sequence}: {energies}")
            best_energy = min(next(iter(values)) for values in energies.values())
            for heuristic, values in energies.items():
                score = next(iter(values))
                if score > best_energy:
                    print(
                        f"SUBOPTIMAL {sequence} {heuristic}: energy {score}; best {best_energy}",
                        flush=True,
                    )

    current_commit = command(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True).stdout.strip()
    if current_commit != source_commit or command(
        ["git", "status", "--porcelain"], cwd=ROOT, capture_output=True
    ).stdout.strip():
        raise RuntimeError("working tree changed during benchmark measurements; results discarded")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=FIELDS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    metadata = {
        "source_commit": source_commit,
        "working_tree_dirty": False,
        "sequences": args.sequences,
        "heuristics": HEURISTICS,
        "repetitions": args.repetitions,
        "machine": platform.platform(),
        "processor": platform.processor(),
        "rustc": command(["rustc", "--version"], cwd=ROOT, capture_output=True).stdout.strip(),
        "cargo": command(["cargo", "--version"], cwd=ROOT, capture_output=True).stdout.strip(),
        "measurement": "SearchResult.total_time; release build; A*; same sequence for each heuristic",
    }
    args.output.with_suffix(".json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Saved {args.output}")


if __name__ == "__main__":
    main()

"""Build and measure the same protein-folding harness at three revisions."""

import argparse
import csv
import io
import json
from pathlib import Path
import platform
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
REVISIONS = (
    "5842cd21f0ff8db55fdea543826f5c78e30d3fdc",
    "60443c0bec1cc607366c679179f305928ed93111",
)
SEQUENCES = (
    "PHHPHPPHP",
    "HHPHPPHHHPPPPHH",
    "HHPHPHHHPPPPHHPHPHPP",
)


def command(args, **kwargs):
    return subprocess.run(args, check=True, text=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repetitions", type=int, default=7)
    parser.add_argument("--sequences", nargs="+", default=SEQUENCES)
    parser.add_argument("--output", type=Path, default=ROOT / "benchmarks" / "results.csv")
    args = parser.parse_args()
    if args.repetitions < 1:
        parser.error("--repetitions must be positive")
    if not args.sequences or any(not s or set(s) - {"H", "P"} for s in args.sequences):
        parser.error("--sequences must contain nonempty H/P strings")

    current = command(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True).stdout.strip()
    if command(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True).stdout.strip():
        parser.error("working tree must be clean so HEAD identifies measured code")

    harness = (ROOT / "examples" / "protein_bench.rs").read_bytes()
    rows = []
    with tempfile.TemporaryDirectory(prefix="protein-benchmark-") as scratch:
        for revision in (*REVISIONS, current):
            short = revision[:8]
            checkout = Path(scratch) / short
            checkout.mkdir()
            archive = Path(scratch) / f"{short}.tar"
            command(["git", "archive", "--format=tar", "-o", str(archive), revision], cwd=ROOT)
            command(["tar", "-xf", str(archive), "-C", str(checkout)])
            (checkout / "examples" / "protein_bench.rs").write_bytes(harness)

            target = Path(scratch) / f"target-{short}"
            command(
                ["cargo", "build", "--offline", "--locked", "--release", "--example", "protein_bench", "--target-dir", str(target)],
                cwd=checkout,
            )
            binary = target / "release" / "examples" / "protein_bench"
            for sequence in args.sequences:
                print(f"{short} {sequence}", flush=True)
                output = command(
                    [str(binary), sequence, str(args.repetitions)],
                    cwd=checkout,
                    capture_output=True,
                ).stdout
                for row in csv.DictReader(io.StringIO(output)):
                    rows.append({"revision": revision, **row})

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="") as file:
        writer = csv.DictWriter(
            file,
            fieldnames=("revision", "sequence", "run", "duration_ns", "iterations", "energy"),
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(rows)

    metadata = {
        "revisions": [*REVISIONS, current],
        "sequences": args.sequences,
        "repetitions": args.repetitions,
        "machine": platform.platform(),
        "processor": platform.processor(),
        "rustc": command(["rustc", "--version"], capture_output=True).stdout.strip(),
        "cargo": command(["cargo", "--version"], capture_output=True).stdout.strip(),
        "measurement": "SearchResult.total_time; release build; same harness; median across runs",
    }
    args.output.with_suffix(".json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Saved {args.output}")


if __name__ == "__main__":
    main()

# /// script
# requires-python = ">=3.10"
# dependencies = ["matplotlib==3.10.8"]
# ///
"""Plot median runtime and search work for three protein-folding revisions."""

import argparse
import csv
from collections import defaultdict
import json
from pathlib import Path
from statistics import median

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt


HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=HERE / "results.csv")
    parser.add_argument("--output", type=Path, default=HERE / "performance.png")
    args = parser.parse_args()
    metadata = json.loads(args.input.with_suffix(".json").read_text())
    revisions = metadata["revisions"]
    sequences = metadata["sequences"]
    groups = defaultdict(list)
    with args.input.open(newline="") as file:
        for row in csv.DictReader(file):
            groups[row["revision"], row["sequence"]].append(row)

    summary = {}
    for sequence in sequences:
        for revision in revisions:
            rows = groups[revision, sequence]
            if len(rows) != metadata["repetitions"]:
                raise ValueError(f"missing measurements: {revision} {sequence}")
            iterations = {int(row["iterations"]) for row in rows}
            energies = {int(row["energy"]) for row in rows}
            if len(iterations) != 1 or len(energies) != 1:
                raise ValueError(f"inconsistent search results: {revision} {sequence}")
            summary[revision, sequence] = (
                median(int(row["duration_ns"]) for row in rows) / 1_000_000,
                iterations.pop(),
                energies.pop(),
            )

    colors = ["#607d8b", "#e49c44", "#287c55"]
    labels = [f"{revision[:8]}" for revision in revisions]
    xs = list(range(len(sequences)))
    width = 0.25
    fig, (ax_time, ax_work) = plt.subplots(2, 1, figsize=(11, 8), layout="constrained")
    for index, revision in enumerate(revisions):
        offsets = [x + (index - 1) * width for x in xs]
        times = [summary[revision, sequence][0] for sequence in sequences]
        work = [summary[revision, sequence][1] for sequence in sequences]
        ax_time.bar(offsets, times, width, label=labels[index], color=colors[index])
        ax_work.bar(offsets, work, width, color=colors[index])

    ax_time.set_title("Protein folding · A* con lookahead a 3 passi")
    ax_time.set_ylabel("Tempo mediano ricerca (ms; scala log)")
    ax_time.set_yscale("log")
    ax_time.legend(title="Commit")
    ax_work.set_ylabel("Nodi estratti dalla frontiera")
    ax_work.set_xlabel("Sequenza · lunghezza")
    for axis in (ax_time, ax_work):
        axis.set_xticks(xs, [f"{sequence}\n{len(sequence)} residui" for sequence in sequences])
        axis.grid(axis="y", alpha=0.2)
        axis.set_axisbelow(True)
    fig.savefig(args.output, dpi=180)
    plt.close(fig)

    lines = [
        "# Risultati benchmark",
        "",
        f"Misura: {metadata['measurement']}. Ripetizioni: {metadata['repetitions']}.",
        f"Macchina: {metadata['machine']}; {metadata['rustc']}.",
        "",
        "| Sequenza | Commit | Tempo mediano (ms) | Iterazioni | Energia |",
        "|---|---|---:|---:|---:|",
    ]
    for sequence in sequences:
        for revision in revisions:
            runtime, iterations, energy = summary[revision, sequence]
            lines.append(f"| `{sequence}` | `{revision[:8]}` | {runtime:.3f} | {iterations:,} | {energy} |")
    lines += ["", "Il tempo esclude compilazione, avvio del processo e disegno della soluzione."]
    report = args.input.with_name("results.md")
    report.write_text("\n".join(lines) + "\n")
    print(f"Saved {args.output} and {report}")


if __name__ == "__main__":
    main()

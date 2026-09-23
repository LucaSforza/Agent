# /// script
# requires-python = ">=3.10"
# dependencies = ["matplotlib==3.10.8"]
# ///
"""Compare runtime, search work, and energy for protein-folding heuristics."""

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
ORDER = [
    "legacy",
    "one_step",
    "lookahead2",
    "lookahead3",
    "lookahead3_parity",
    "lookahead4_parity",
]
LABELS = {
    "legacy": "legacy",
    "one_step": "default 1-step",
    "lookahead2": "lookahead 2",
    "lookahead3": "lookahead 3",
    "lookahead3_parity": "lookahead 3 parity",
    "lookahead4_parity": "lookahead 4 parity",
}
OLD = set(ORDER[:4])
COLORS = {
    "legacy": "#9aa8b2",
    "one_step": "#718996",
    "lookahead2": "#4e7589",
    "lookahead3": "#285e78",
    "lookahead3_parity": "#2b8c62",
    "lookahead4_parity": "#75b798",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=HERE / "heuristics.csv")
    parser.add_argument("--output", type=Path, default=HERE / "heuristics.png")
    parser.add_argument("--report", type=Path, default=HERE / "heuristics.md")
    args = parser.parse_args()
    metadata_path = args.input.with_suffix(".json")
    metadata = json.loads(metadata_path.read_text()) if metadata_path.exists() else {}

    measurements = defaultdict(list)
    with args.input.open(newline="") as file:
        for row in csv.DictReader(file):
            heuristic = row["heuristic"]
            if heuristic not in ORDER:
                raise ValueError(f"unknown heuristic: {heuristic}")
            measurements[heuristic, row["sequence"]].append(
                (int(row["duration_ns"]), int(row["iterations"]), int(row["energy"]))
            )

    sequences = sorted({sequence for _, sequence in measurements}, key=lambda value: (len(value), value))
    if not sequences:
        raise ValueError(f"no measurements in {args.input}")
    summary = {}
    run_counts = set()
    for sequence in sequences:
        for heuristic in ORDER:
            runs = measurements[heuristic, sequence]
            if not runs:
                raise ValueError(f"missing measurements: {heuristic} {sequence}")
            run_counts.add(len(runs))
            summary[heuristic, sequence] = (
                median(item[0] for item in runs) / 1_000_000,
                median(item[1] for item in runs),
                median(item[2] for item in runs),
            )

    count = len(ORDER)
    xs = list(range(len(sequences)))
    width = 0.16
    fig, axes = plt.subplots(3, 1, figsize=(15, 10), layout="constrained", sharex=True)
    ax_time, ax_iterations, ax_energy = axes
    for index, heuristic in enumerate(ORDER):
        offsets = [x + (index - (count - 1) / 2) * width for x in xs]
        times = [summary[heuristic, sequence][0] for sequence in sequences]
        iterations = [summary[heuristic, sequence][1] for sequence in sequences]
        energies = [summary[heuristic, sequence][2] for sequence in sequences]
        legend_label = LABELS[heuristic]
        ax_time.bar(offsets, times, width, label=legend_label, color=COLORS[heuristic])
        ax_iterations.bar(offsets, iterations, width, color=COLORS[heuristic])
        ax_energy.bar(offsets, energies, width, color=COLORS[heuristic])

    ax_time.set_title("Protein folding · confronto euristiche")
    ax_time.set_ylabel("Tempo mediano di ricerca (ms; scala log)")
    ax_time.set_yscale("log")
    ax_time.legend(ncols=3, title="Storiche (blu/grigio) e nuove (verde)")
    ax_iterations.set_ylabel("Iterazioni mediane")
    ax_energy.set_ylabel("Energia mediana (più bassa è meglio)")
    ax_energy.set_xlabel("Sequenza")
    ax_energy.axhline(0, color="#555555", linewidth=0.7)
    for axis in axes:
        axis.grid(axis="y", alpha=0.2)
        axis.set_axisbelow(True)
    ax_energy.set_xticks(xs, [f"{len(sequence)} residui" for sequence in sequences])
    fig.savefig(args.output, dpi=180)
    plt.close(fig)

    lines = [
        "# Benchmark delle euristiche",
        "",
        f"Mediane su {', '.join(map(str, sorted(run_counts)))} esecuzioni per configurazione; input `{args.input.name}`.",
        "Le barre grigio-blu indicano euristiche presenti nei commit storici; il verde indica le nuove `h_lookahead3_parity` e `h_lookahead4_parity`.",
    ]
    if metadata:
        lines += [
            f"Codice misurato: `{metadata.get('source_commit', 'sconosciuto')}`; working tree {'dirty' if metadata.get('working_tree_dirty') else 'pulito'}.",
            f"Ambiente: {metadata.get('machine', 'sconosciuto')}; {metadata.get('rustc', 'rustc sconosciuto')}; {metadata.get('measurement', 'misura non specificata')}.",
        ]
    lines += [
        "",
        "| Sequenza | Euristica | Tempo mediano (ms) | Iterazioni mediane | Energia mediana |",
        "|---|---|---:|---:|---:|",
    ]
    warnings = []
    for sequence in sequences:
        best_energy = min(summary[heuristic, sequence][2] for heuristic in ORDER)
        for heuristic in ORDER:
            runtime, iterations, energy = summary[heuristic, sequence]
            marker = " ⚠ subottima" if heuristic in OLD and energy > best_energy else ""
            if marker:
                warnings.append(f"`{heuristic}` su `{sequence}`: energia {energy}, migliore {best_energy}.")
            lines.append(
                f"| `{sequence}` | {LABELS[heuristic]}{marker} | {runtime:.3f} | {iterations:,.0f} | {energy:.0f} |"
            )
    lines += [
        "",
        "Energia più bassa corrisponde a più contatti H. Un'euristica che accelera la ricerca ma restituisce energia peggiore non è un miglioramento della qualità della soluzione.",
    ]
    if warnings:
        lines += ["", "## Avvertenze sulle soluzioni", ""]
        lines.extend(f"- {warning}" for warning in warnings)
    else:
        lines += ["", "Nessuna euristica storica ha prodotto energia peggiore della migliore misurata per le sequenze incluse."]
    args.report.write_text("\n".join(lines) + "\n")
    print(f"Saved {args.output} and {args.report}")


if __name__ == "__main__":
    main()

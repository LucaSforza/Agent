# Benchmark protein folding

Il confronto usa tre revisioni: `5842cd21f0ff8db55fdea543826f5c78e30d3fdc`,
`60443c0bec1cc607366c679179f305928ed93111` e il commit delle migliorie.
Il medesimo `examples/protein_bench.rs` viene copiato nei checkout storici e
compilato in release. Misura solo la ricerca A* con euristica a tre passi;
input fissi, esecuzioni ripetute e mediane limitano il rumore. Sono registrati
anche iterazioni ed energia, per distinguere lavoro svolto e velocità della CPU.

Prima si fa commit del codice, poi si misurano le tre revisioni da un working
tree pulito:

```bash
python3 benchmarks/collect.py --repetitions 7
UV_CACHE_DIR=/tmp/protein-uv-cache MPLCONFIGDIR=/tmp/protein-mpl \
  uv run --script benchmarks/plot.py
```

`collect.py` salva dati grezzi in `benchmarks/results.csv` e ambiente in
`benchmarks/results.json`; `plot.py` crea `benchmarks/performance.png` e
`benchmarks/results.md`. Il tempo è `SearchResult.total_time`: esclude
compilazione, avvio del processo e calcolo dell'energia per l'output.

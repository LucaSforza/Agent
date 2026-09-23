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

## Risultati misurati

[Grafico](../benchmarks/performance.png) · [tabella](../benchmarks/results.md) ·
[CSV grezzo](../benchmarks/results.csv).

Sulla sequenza da 20 residui, il commit `081ef55f` impiega 31,688 ms mediani:
3,18 volte più veloce di `5842cd21` e 1,16 volte di `60443c0b`.
`60443c0b` e `081ef55f` estraggono entrambi 13.154 nodi; qui il guadagno
aggiuntivo viene dal lavoro svolto per nodo. Tutte le revisioni producono
energia -7 su questa sequenza. Le tre sequenze e le sette misure per caso
sono nel CSV; i tempi molto brevi della sequenza da 9 residui sono più
suscettibili a rumore di sistema.

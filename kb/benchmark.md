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

## Confronto delle euristiche

Il benchmark euristico misura sul codice finale quattro varianti già presenti
nei commit `5842cd21`/`60443c0b` (`legacy`, `one_step`, `lookahead2`,
`lookahead3`) e le nuove `h_lookahead3_parity` e `h_lookahead4_parity`. Tutte ricevono gli stessi input
e vengono misurate con lo stesso harness release, incluse sequenze da 9, 15, 20
e 25 residui. Il caso più lungo rende visibile anche la qualità della soluzione:
la variante `legacy` può essere subottima; tabella e grafico annotano l'energia
peggiore rispetto alla migliore misurata sullo stesso input. Il grafico mostra le mediane
di tempo e iterazioni, più l'energia ottenuta; il gruppo blu indica le varianti
storiche e il verde le nuove. Energia più bassa significa più contatti H. Una
euristica storica con energia maggiore della migliore per quella sequenza viene
segnalata come subottima: velocità e qualità della soluzione vanno interpretate
insieme.

```bash
python3 benchmarks/collect_heuristics.py --repetitions 7
UV_CACHE_DIR=/tmp/protein-uv-cache MPLCONFIGDIR=/tmp/protein-mpl \
  uv run --script benchmarks/plot_heuristics.py
```

Il CSV `benchmarks/heuristics.csv` conserva ogni esecuzione; i metadati di
macchina e compilatore sono in `benchmarks/heuristics.json`. Il grafico e la
tabella vengono salvati in `benchmarks/heuristics.png` e
`benchmarks/heuristics.md`.

[Secondo grafico](../benchmarks/heuristics.png) ·
[tabella euristiche](../benchmarks/heuristics.md) ·
[CSV euristiche](../benchmarks/heuristics.csv).

Sui 20 residui, `h_lookahead3_parity` impiega 25,799 ms mediani contro
28,998 ms di `h_lookahead3` (1,12 volte più veloce); entrambe danno energia
-7. La variante a quattro passi estrae meno nodi (9.482 contro 13.224), ma
richiede 46,004 ms: calcolo dell'euristica più costoso. Sui 25 residui,
`legacy` impiega 21,868 ms ma trova energia -10; le euristiche ammissibili
trovano -11. Le conclusioni valgono per queste sequenze e questa macchina.

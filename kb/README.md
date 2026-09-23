# Knowledge base

Questa cartella riassume struttura e scelte del progetto. Le API e gli esempi nel
codice restano la fonte di verità.

- [Architettura](architecture.md): contratti dei problemi e famiglie di algoritmi.
- [Protein folding](protein-folding.md): modello HP, ricerca e criteri di correttezza.
- [Benchmark](benchmark.md): protocollo e risultati delle tre revisioni.

## Comandi rapidi

```bash
cargo test
cargo run --release --example protein_folding -- solve HHPHPPHHHPPPPHH
cargo run --release --example protein_bench -- HHPHPPHHHPPPPHH
```

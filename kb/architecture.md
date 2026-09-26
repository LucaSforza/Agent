# Architettura

Il crate Rust `agent` separa **formulazione del problema** e **algoritmo**. Una
formulazione definisce stato, azioni, transizione, costo, euristica e condizione
di arrivo tramite i trait in `src/problem.rs`.

## Ricerca nello spazio degli stati

`src/statexplorer/` implementa BFS, DFS, costo uniforme, Best First e A*.
`resolver.rs` espone due risolutori: `Explorer` tiene un insieme di stati già
esplorati, mentre `TreeExplorer` visita un albero senza deduplicare gli stati.
`frontier.rs` gestisce coda, pila e heap di priorità. `node.rs` conserva stato,
genitore, azione, costo cumulativo ed euristica; i nodi sono allocati in un
arena `bumpalo` per ridurre il costo delle allocazioni. Per A*, la priorità è
`f = g + h`; a parità di `f` la revisione `60443c0` preferisce `h` minore.

### Contratto di integrazione con pddl_fol

La ricerca su grafo espone anche un punto d'ingresso pubblico che crea e
possiede l'arena: i client non devono importare `bumpalo`. Il chiamante può
imporre un massimo positivo di stati distinti memorizzati; il risultato
distingue soluzione, esaurimento e limite raggiunto. Il conteggio di stati
espansi rimane disponibile. Le API esistenti dell'`Explorer` restano valide.

Per A* e costo uniforme, un percorso più economico verso uno stato già visto
deve poter sostituire il precedente anche dopo l'espansione. Il controllo dei
duplicati confronta il miglior costo `g` noto, non soltanto l'appartenenza a un
insieme degli esplorati. La terminazione avviene al prelievo dell'obiettivo
dalla frontiera. A parità di priorità l'ordine è deterministico. I costi delle
azioni usati da queste ricerche devono essere non negativi; `pddl_fol` usa
sempre costo unitario.

## Miglioramento iterativo

`src/improve/` comprende discesa ripida, hill climbing e simulated annealing.
Il risolutore può eseguire un tentativo o più restart. Le formulazioni
esemplificative sono in `examples/n_queen.rs` e `examples/csp/`.

## Verifica

I test di integrazione in `tests/` definiscono piccoli problemi locali per
controllare algoritmi e casi limite. Per modifiche al protein folding, verificare
anche energia della soluzione, numero di residui collocati e regressioni sui
tempi release: solo la durata non prova la correttezza.

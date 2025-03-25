## Agent Framework

L'idea di questo framework è quella di creare un unica interfaccia per modellare
il proprio "problema" da risolvere e con la stessa formulazione del problema
permette la sua risoluzione con una vasta gamma di algoritmi.

## Algoritmi a miglioramento iterativo

Per eseguire l'esempio n_queen eseguire:

```bash
./run.sh n_queen
```

Darà in output la percentuale (tra 0 e 1) delle volte in cui i vari algoritmi a miglioramento iterativo  (steesp descend, hill climbing, simulated annealing) hanno trovato il risultato ottimo per il problema NQueen.

Esempio output:

```
Steepest Descend:
        One restart:
          Correctness: 0.004
          Total Duration: 1.644453ms
          Mean time: 6.577µs
        Number restarts:100
          Correctness: 0.304
          Total Duration: 136.813983ms
          Mean time: 547.255µs
Hill Climbing:
        One restart:
          Correctness: 0.008
          Total Duration: 12.513636ms
          Mean time: 50.054µs
        Number restarts:100
          Correctness: 0.46
          Total Duration: 911.79205ms
          Mean time: 3.647168ms
Simulated Annealing:
        One restart:
          Correctness: 0.06
          Total Duration: 33.157589ms
          Mean time: 132.63µs
        Number restarts:100
          Correctness: 0.992
          Total Duration: 712.095673ms
          Mean time: 2.848382ms
```

Per maggiori dettagli degli output e dell'implementazione del problema e degli algoritmi
vedere [a_4_1.md](a_4_1.md).

# Protein Folding

Nella cartella esempi è presente come esempio il problema del Protein Folding.
Per maggiori informazioni cliccare [qui](examples/protein_folding/README.md).

## Esplorazione di Spazi degli stati

Per eseguire gli esempi:

```bash
cargo t --release
```


vedere Markdown [a_3_1.md](a_3_1.md) e [a_3_2.md](a_3_2.md) per la soluzione degli esercizi.


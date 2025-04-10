# Protein Folding

Per formulare questo problema mi ero soffermato a ragionare su cos'è il costo e sul cos'è l'azione
per questo problema.

Il problema è che quando piazziamo un aminoacido H non possiamo sapere finché non abbiamo costruito la soluzione completa
se quell'aminoacido ci contribuirà tanto o poco per la massimizzazione dei contatti.

Quindi l'idea per la formulazione del problema è: Piazza il primo aminoacido sulla posizione (0,0) e questo sarà lo stato iniziale. Alle prossime iterazioni piazza il prossimo aminoacido verso una direzione (sopra, sotto, sinistra, destra) che sia legale (ovvero evitando che due aminoacidi finiscano sulla stessa posizione).

Il costo delle azioni è 3 - [numero di nuovi contatti generati]. 
In questo modo MinCost quando minimizzerà le azioni, massimizzerà il numero di contatti generati.

Le azioni quindi sono la DIREZIONE in cui deve essere piazzato il prossimo aminoacido.
Dalle direzioni si può ricostruire tutta la forma della proteina e calcolarne l'energia.

## Modellazione del problema

```rust
pub struct ProteinFolding {
    aminoacids: Vec<AminoAcid>, // len is n
    h_numer: u32,
}
```
Questra struttura rappresenta il problema. Mi salvo in cache anche il numero di proteine H (mi servirà piu' avanti).

```rust
#[derive(Clone, Default)]
pub struct Board {
    last: Option<Rc<Board>>,
    pos: Pos,
    depth: usize,
    has_turned: bool,
    total_contacs: u32,
}
```

Questo è lo stato del problema. Per risparmiare memoria la `Board` è una linked list. In `pos` è contenuta la posizione
dell'ultimo aminoacido piazziato. In `last` è contenuta la proteina generata fino all'aminoacido precedente (se ho piazzato
solo un aminoacido `last` è `None`).

L'attributo `depth` identifica la profondità della linked list e serve per sapere che tipo di aminoacido è quello piazzato. Può essere usato come indice nel vettore `aminoacids` all'interno di `ProteinFolding`.

L'attributo `has_turned` ci serve per evitare simmetrie (vedere azioni possibili).

## Azioni possibili

```rust
fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
    if state.depth == 0 {
        // non importa dove vado la prima volta
        return vec![Dir::Up].into_iter();
    }

    let mut actions;
    if state.has_turned {
        actions = Vec::with_capacity(3);
        for dir in vec![Dir::Left, Dir::Down, Dir::Up, Dir::Right] {
            if state.suitable(&state.pos.clone_move(dir)) {
                actions.push(dir);
            }
        }
    } else {
        // come prima svolta considerare solo la destra
        actions = Vec::with_capacity(2);
        for dir in vec![Dir::Down, Dir::Up, Dir::Right] {
            if state.suitable(&state.pos.clone_move(dir)) {
                actions.push(dir);
            }
        }
    }
    actions.into_iter()
}
```

Le azioni eseguibili sono semplicemente tutte le direzioni che informano dove deve essere piazzato.

Però per evitare simmetrie adottiamo due strategie:
- il prossimo aminoacido tale che non infrangano i requisiti (descritti nel metodo `suitable`).
- finché non abbiamo svoltato per la prima volta a destra o a sinistra, consideriamo solo una direzione possibile per svoltare.

## Goal

Il goal è semplicemente quando ho piazzato tutti gli aminoacidi sulla griglia.

## Euristica

```rust
fn heuristic(&self, state: &Self::State) -> Self::Cost {
    self.h_numer - state.total_contacs
}
```

Questa euristica è molto naive, ma sperimentalmente ho notato che si comporta molto bene (vedere le Conclusioni).

L'idea è che per fare piu' contatti possibili devo fare in modo che le H siano "ammucchiate".
`total_contacts` è il numero di aminoacidi H a cui ho "accopiato" almeno un altro aminoacido H.

## Conclusioni

Vorrei concludere mostrando come si comporta questa modellazione.

I test mostrati fanno affidamento alla proteina: H, H, P, H, P, P, H, H, H, P, P, P, P, H, H, P.

Il risultato ottimo è -6. (se l'implementazione è corretta... comunque ho provato con l'esempio sulla traccia e trova lo stesso risultato).

### MinCost

```
MinCost:
contacts: 5
actions: [Up, Right, Down, Right, Down, Left, Left, Left, Left, Left, Up, Right, Right, Up, Left]
time: 52.0823ms
iterations: 28209
max frontier size: 47449
  P-H H-P  
    | | |  
P-P-H H H-P
|         |
P-P-H-H-H-P

Energy: -6
```

Min Cost trova sempre l'ottimo.

### BFS

```
BFS:
contacts: 0
actions: [Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up]
time: 934.642473ms
iterations: 471364
max frontier size: 802075
P
|
H
|
H
...
```

BFS non trova l'ottimo, questo perché all'ultima iterazione mette tutti gli stati finali in frontiera, ma per via della regola della BFS prende il primo che ha inserito (First In Firts Out).
Quindi dato che la prima mossa che l'algoritmo considera è andare a sinistra (vedere funzione `executable_actions`) allora chiaramente il primo che ha inserito è la soluzione dove piazza aminoacidi sempre a sinistra (apparte il primo aminoacido che viene posizionato sempre in alto).

### DFS

```
DFS:
actions: [Up, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right]
time: 83.21µs
iterations: 16
max frontier size: 28
H-P-H-P-P-H-H-H-P-P-P-P-H-H-P
|                            
H

Energy: 0
```

Ovviamente DFS non trova l'ottimo generalmente, ma dato che segue la regola Last In First Out è interessante notare come la direzione che sceglie sempre è quella di andare a destra che è esattamente l'ultima direzione che viene considerata nella funzione `exectuable_actions`. (tranne il primo ovviamente)

### AStar

```
AStar:
contacts: 5
actions: [Up, Right, Down, Right, Down, Left, Left, Left, Left, Left, Up, Right, Right, Up, Left]
time: 3.894715ms
iterations: 515
max frontier size: 850
  P-H H-P  
    | | |  
P-P-H H H-P
|         |
P-P-H-H-H-P

Energy: -6
```

A* trova sempre l'ottimo quando l'euristica è consistente.

### BestFirst

```
BestFirst:
contacts: 3
actions: [Up, Right, Down, Down, Right, Up, Up, Up, Up, Left, Down, Left, Left, Down, Left]
time: 67.62µs
iterations: 38
max frontier size: 60
      P-P
      | |
  H-P-P H
  |     |
P-H H-P H
    | | |
    H H H
      | |
      P-P

Energy: -3
```

Generalmente non trova l'ottimo. Però è curioso notare come l'euristica cerchi di raggruppare le H il piu possibile. Dato
che la mia euristica si basa sulle distanze euclidiane delle H.

### Iterative

```
Iterative:
actions: [Up, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right, Right]
time: 1.424581952s
iterations: 747821
max frontier size: 28
H-P-H-P-P-H-H-H-P-P-P-P-H-H-P
|                            
H                            

Energy: 0
```

Ha lo stesso problema della DFS.

### Critical Ratio

Ho provato anche a calcolare la critical ratio per questo problema.

La ratio che ho preso in considerazione è (numero di H)/(numero totale di aminoacidi).

Le potreine le genero a caso. Fissato il numero di H e di aminoacidi totali piazzo in modo randomico le H per tutta la sequenza
ed eseguo 300 test (ogni test ha una proteina differente, ma il numero di H è sempre lo stesso). Poi mi calcolo il tempo medio
e mi salvo la ratio che massimizza il tempo medio.

Per potreine piu' corte di 8 non sembra esserci un pattern, ma con potreine sufficientemente grandi il risultato sembra convergere
sul 0.10.

```
protein lenght: 1
 Max ratio: 0
protein lenght: 2
 Max ratio: 0.5
protein lenght: 3
 Max ratio: 1
protein lenght: 4
 Max ratio: 0
protein lenght: 5
 Max ratio: 1
protein lenght: 6
 Max ratio: 0.5
protein lenght: 7
 Max ratio: 1
protein lenght: 8
 Max ratio: 0.375
protein lenght: 9
 Max ratio: 0.3333333333333333
protein lenght: 10
 Max ratio: 0.2
protein lenght: 11
 Max ratio: 0.18181818181818182
protein lenght: 12
 Max ratio: 0.16666666666666666
protein lenght: 13
 Max ratio: 0.15384615384615385
protein lenght: 14
 Max ratio: 0.14285714285714285
protein lenght: 15
 Max ratio: 0.13333333333333333
protein lenght: 16
 Max ratio: 0.125
protein lenght: 17
 Max ratio: 0.11764705882352941
```
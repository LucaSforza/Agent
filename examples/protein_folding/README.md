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
Questra struttura rappresenta il mio problema. Mi salvo in cache anche il numero di proteine H (mi servirà piu' avanti).

```rust
pub struct Board {
    protein: Graph<Pos, Direction, Undirected, u32>,
    index: Vec<NodeIndex>,
    has_turned: bool,
}
```

Questo è lo stato del problema. Per risparmiare memoria non ho optato per una matrice, ma invece per un grafo usando il crate `petgraph`. Il grafo che uso utilizza le liste di adiacenza come struttura dati.

Mi tengo anche un vettore `index` dove nell'i-esima posizione contiene l'indice all'interno della struttura di adiagenza dove è memoriazzato l'i-esimo aminoacido. Un aminoacio è rappresentato semplicemente come la sua posizione e gli archi sono la direzione (in realtà questo dato negli archi non lo uso mai, gli archi mi servono solo a capire se due aminoacidi sono adicenti).

Nello stato non mi serve tenermi anche quale aminoacido si trova in una certa posizione. Se voglio quale aminoacido in quale posizione si trova allora mi basta conoscere la sua posizione dal vettore `aminoacids` dentro la struttura `ProteinFolder` e usare il vettore `index` per recuperare l'indice all'interno della lista di adiacenza.

L'attributo `has_turned` ci serve per evitare simmetrie (vedere azioni possibili).

## Azioni possibili

```rust
    fn executable_actions(&self, state: &Self::State) -> impl Iterator<Item = Self::Action> {
        if state.index.len() == 1 {
            // non importa dove vado la prima volta
            return vec![Dir::Up].into_iter();
        }

        let last_aminoacid;

        last_aminoacid = state.get_last_aminoacid();

        let mut actions;
        if state.has_turned {
            actions = Vec::with_capacity(3);
            for dir in vec![Dir::Left, Dir::Down, Dir::Up, Dir::Right] {
                if state.suitable(&last_aminoacid.clone_move(dir)) {
                    actions.push(dir);
                }
            }
        } else {
            // alla prima svolta considerare solo la destra
            actions = Vec::with_capacity(2);
            for dir in vec![Dir::Down, Dir::Up, Dir::Right] {
                if state.suitable(&last_aminoacid.clone_move(dir)) {
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
        // Calcolare la distanza euclidiana dagli aminoacidi H non consecutivi e sottrai per gli aminoacidi H presenti
        let mut cost = 0.0;

        for (i, a) in state.index.iter().zip(self.aminoacids.iter()) {
            // se l'aminoacido è H allora controllo la distanza minima rispetto ad un altro aminoacido H che non sia adiacente
            if *a == AminoAcid::H {
                let amin = &state.protein[*i];
                let mut min_distrance = f64::INFINITY;
                for (j, b) in state.index.iter().zip(self.aminoacids.iter()) {
                    if i != j && *b == AminoAcid::H && state.find_edge_undirected(*i, *j).is_none()
                    {
                        // calcola la distanza euclidiana e aggiungila al costo
                        let other_amin = &state.protein[*j];
                        let distance = ((amin.x - other_amin.x).pow(2)
                            + (amin.y - other_amin.y).pow(2))
                            as f64;
                        let distance = distance.sqrt();
                        if distance < min_distrance {
                            min_distrance = distance;
                        }
                    }
                }
                // se l'ho trovato lo aggiungo al costo
                if min_distrance.is_finite() {
                    cost += min_distrance;
                }
            }
        }

        // le distanze sono duplicate, divido per 2
        let mut cost = (cost / 2.0).floor() as <ProteinFolding as Problem>::Cost;

        // aggiungo al costo tutte le H non ancora posizionate, cosi quando sottraggo il risultato è consistente
        cost += (self
            .aminoacids
            .iter()
            .filter(|x| **x == AminoAcid::H)
            .count()) as u32;

        // sottraggo al costo il numero di H posizionati
        // questo perché vorrei che la soluzione ottima abbia 0 come euristica.
        // Se ogni H è stato posizionato con successo allora le loro distanze euclidiane sono 1
        // vengono sommate al costo e poi sottratte qua.
        cost - self.h_numer
    }
```

L'idea dell'euristica è quella di dividerla in due parti:
La prima è di considerare gli aminoacidi che ho già piazzato e quelli che non ho piazzato.

Per quelli che ho piazzato ignoro gli aminoacidi P, invece gli H trovo la distanza euclidiana
minima rispetto ad un altro H che non sia adiacente e aggiungo queste distanze al valore finale dell'euristica.

Questo perché voglio penalizzare stati in cui gli aminoacidi H sono troppo lontani tra di loro.

La soluzione migliore di tutte dovrebbe essere quella in cui l'euristica vale 0, perciò sottraggo al valore dell'euristica il numero di H nella proteina. Così facendo se tutte le distanze minime erano 1 allora l'euristica varrà 0.

Questo però solo se ho posizionato tutti gli aminoacidi, ma potrei avere ancora degli aminoacidi H da piazzare che potrebbero ridurre il costo. Quindi per fare l'euristica dovrei essere ottimista e quindi assumo che troverò la posizione perfetta per loro e quindi con distanza euclidiana ad 1.

## Conclusione

Vorrei concludere mostrando come si comporta questa modellazione.

I test mostrati fanno affidamento alla proteina: HHPHPPHHHPPPPHHP

Il risultato ottimo è -6. (se l'implementazione è corretta... comunque ho provato con l'esempio sulla traccia e trova lo stesso risultato).

### MinCost

```
MinCost:
actions: [Up, Right, Down, Right, Down, Left, Left, Left, Left, Left, Up, Right, Right, Up, Left]
time: 138.334369ms
iterations: 28209
max frontier size: 47449

  P-H H-P  
    | | |  
P-P-H H H-P
|         |
P-P-H-H-H-P

Energy: -6
```

Min Cost trova sempre l'ottimo

### BFS

```
BFS:
state: TODO: fare la stampa

actions: [Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up, Up]
time: 2.600552147s
iterations: 471364
max frontier size: 802075
P
|
H
|
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
actions: [Up, Right, Down, Right, Down, Left, Left, Left, Left, Left, Up, Right, Right, Up, Left]
time: 87.863347ms
iterations: 24124
max frontier size: 40144

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
actions: [Up, Right, Up, Right, Down, Down, Left, Down, Left, Left, Down, Right, Right, Right, Right]
time: 12.565534ms
iterations: 3531
max frontier size: 5691

    H-P  
    | |  
  H-P P  
  |   |  
  H H-H  
    |    
P-P-H    
|        
P-P-H-H-P

Energy: -2
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
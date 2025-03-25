# Protein Folding

Per formulare questo problema mi ero soffermato a ragionare su cos'è il costo e sul cos'è l'azione
per questo problema.

Ho subito notato che un aminoacido H può fare al piu' due connessioni. Quindi se l'obbiettivo è 
massimizzare il numero di contatti allora il costo di un azione deve essere: 2 - numero di contatti.

Quindi l'idea per la formulazione del problema è: Piazza il primo aminoacido sulla posizione (0,0) e questo sarà lo stato iniziale. Alle prossime iterazioni piazza il prossimo aminoacido, se esso è P allora il costo è per forza 2, altrimenti conta il numero di nuovi contatti e sottrailo a 2. Per esempio se piazzando il nuovo aminoacido ho 2 nuovi contatti allora il costo di quell'azione è 0.

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
}
```

Questo è lo stato del problema. Per risparmiare memoria non ho optato per una matrice, ma invece per un grafo usando il crate `petgraph`. Il grafo che uso utilizza le liste di adiacenza come struttura dati.

Però mi tengo anche un vettore `index` dove nell'i-esima posizione contiene l'indice all'interno della struttura di adiagenza dove è memoriazzato l'i-esimo aminoacido. Un aminoacio è rappresentato semplicemente come la sua posizione e gli archi sono la direzione (in realtà questo dato negli archi non lo uso mai, gli archi mi servono solo a capire se due aminoacidi sono adicenti).

Nello stato non mi serve tenermi anche quale aminoacido si trova in una certa posizione. Se voglio quale aminoacido in quale posizione si trova allora mi basta conoscere la sua posizione dal vettore `aminoacids` dentro la struttura `ProteinFolder` e usare il vettore `index` per recuperare l'indice all'interno della lista di adiacenza.

## Goal

Il goal è semplicemente quando ho piazzato tutti gli aminoacidi sulla griglia.
# Protein folding HP

`examples/protein_folding/formulation.rs` rappresenta una catena di residui H
(idrofobici) e P (polari) su reticolo quadrato. Ogni stato `Board` contiene
posizione dell'ultimo residuo e riferimento allo stato precedente. Ogni azione
aggiunge un residuo in una casella libera adiacente. Prima mossa e prima svolta
sono vincolate per eliminare rotazioni e riflessioni equivalenti.

L'energia è il negativo del numero di contatti fra H adiacenti sul reticolo ma
non consecutivi nella catena. Collocare un nuovo H ha costo `3 - nuovi_contatti`;
collocare P ha costo zero. Per sequenza fissata, minimizzare il costo totale
equivale a massimizzare i contatti. L'esempio permette confronto fra più
algoritmi; `solve` usa A* con lookahead di tre passi.

L'euristica a tre passi prova le mosse legali dei prossimi tre residui e somma
un limite inferiore rilassato per quelli restanti. Le nuove euristiche di
parità rafforzano quel limite: nel reticolo quadrato solo residui di parità
opposta nella catena possono entrare in contatto, e ogni H già collocato ha
un numero limitato di lati liberi. Si usa il massimo fra limite globale e
lookahead, evitando di sommarli due volte. Le varianti a tre e quattro passi
sono ammissibili; `solve` usa quella a tre passi, scelta sul tempo misurato.

Il primo benchmark storico usa sempre la stessa selezione di euristica e la
stessa sequenza, così le revisioni differiscono soltanto nel codice del
progetto. La revisione `5842cd2` precede una correzione del conteggio dei
contatti nell'euristica; il confronto delle prestazioni va letto insieme
all'energia ottenuta.

Dettagli storici e output dimostrativi: `examples/protein_folding/README.md`.

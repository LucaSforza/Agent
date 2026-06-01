# TODO

## Incapsulamento Bumpalo

Bumpalo é attualmente esposto in tutta l'API pubblica: `Node<'a, P>`, `Explorer<'a, P, Backend>`, `TreeExplorer<'a, P, Backend>`, `Frontier<'a, P, Backend>` portano `'a` dall'arena. Ogni utente deve importare `bumpalo` come dipendenza e creare `Bump::new()` prima di usare qualunque explorer.

49x `Bump::new()` sparsi tra test ed esempi. Il problema principale é in:

- `src/statexplorer/node.rs:13` — `Node<'a, P>` con `parent: Option<&'a Self>`
- `src/statexplorer/resolver.rs:120,283` — `Explorer` e `TreeExplorer` portano `&'a Bump` come campo
- `src/statexplorer/frontier.rs:12` — `FrontierBackend<'a, P>` trait contaminato da `'a`
- `examples/protein_folding/formulation.rs:337` — `ProteinFolding<'a>` con `arena: &'a Bump`

### Possibili approcci

#### A) Self-referential struct (self_cell / ouroboros)

`Explorer` crea `Bump` internamente, usa `self_cell` crate per self-reference: Node punta all'arena dentro Explorer stesso.

```
+ API pulita, nessun lifetime esposto
+/- Dipendenza extra (self_cell)
- Macro opache, compile time maggiore, debugging complesso
```

#### B) Allocator trait strategy

```rust
trait NodeAllocator<P> {
    fn alloc(&mut self, node: Node<P>) -> *const Node<P>;
}
```

Impl per `BumpAllocator`, `BoxAllocator`, `VecAllocator`. Explorer prende `impl NodeAllocator<P>`.

```
+ Flessibile, bumpalo diventa opt-in
+/- Dispatch statico (generic) o dinamico (Box<dyn>)
- Complessitá: un parametro tipo in piú su Explorer
- Serve decidere default
```

#### C) Box default, bumpalo opt-in

`Explorer::new(problem)` usa `Box::new(Node)`. `Explorer::with_arena(problem, arena)` usa bumpalo.
Node.parent = `Option<Box<Node<P>>>`  o `Rc<Node<P>>`.

```
+ Utente base non vede bumpalo
+/- Rc/Box overhead per utenza che non usa bumpalo (atomic increment su parent walk)
- Due code path allocazione, Node cambia struttura
- Box::new per ogni nodo => OOM su 5M nodi protein folding
```

#### D) `NonNull<Node<P>>` + Explorer possiede Bump

```rust
struct Node<P: Utility> {
    parent: Option<NonNull<Node<P>>>,  // niente lifetime
    ...
}
struct Explorer<P, Backend> {
    arena: Pin<Box<Bump>>,  // owned, non esposto
    ...
}
```

```
+ API pulita, niente 'a sulle struct pubbliche
+ Bumpalo nascosto dentro Explorer
- unsafe per dereferenziare NonNull (ma safety locale: Explorer vive > Node)
- Cambiamento strutturale a Node
```

#### E) Convenience constructor con unsafe interno

`Explorer::new_owned(problem)` crea Bump in `Pin<Box<Bump>>`, estende lifetime con unsafe per i Node puntatori.

```
+ Minime modifiche al codice explorer esistente
+ Coesiste con Explorer::new() attuale (backward compat)
- unsafe in due punti
- Due modi di creare Explorer mantenere
```

#### F) Vec-backed slab (nessun bumpalo)

```rust
struct Node<P: Utility> {
    parent: Option<usize>,  // indice nella slab
    ...
}
struct Explorer<P, Backend> {
    arena: Vec<Node<P>>,  // owned, niente lifetime
    ...
}
```

I nodi sono in un `Vec<Node<P>>`. Parent punta all'indice, non a un pointer. La Vec rialloca ma gli indici rimangono validi.

```
+ Zero unsafe
+ Zero dip extra (bumpalo rimosso da Cargo.toml)
+ Nessun lifetime sull'API pubblica
+ Indici stabili anche dopo realloc Vec
+ reset = Vec::clear(), O(1)
- Vec::push ha overhead di capacity check vs bumpalo bump-pointer
- Node leggermente piú grande (padding tra campi)
- Cambiamento strutturale a Node e Explorer
```

#### G) Status quo documentato

Tenere tutto com'è. Aggiungere documentazione che spiega perché bumpalo é necessario e come usarlo.

```
+ Zero cambiamenti
- Utente deve comunque importare bumpalo
- Lifetime 'a continua a contaminare API
```

### Raccomandazione personale

**F (Vec-backed slab)** é la migliore per UX: niente unsafe, niente dip extra, niente lifetime. Va valutato se la differenza di performance rispetto a bumpalo é accettabile (Vec::push ha capacity check, bumpalo é bump pointer). Per la dimensione media dei problemi gestiti, probabilmente trascurabile.

**D (NonNull)** é meglio se si vuole mantenere bumpalo per performance: nasconde l'arena dentro Explorer, l'utente non lo vede, ma richiede unsafe.

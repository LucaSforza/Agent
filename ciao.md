```Rust
fn heuristic(&self) -> f64 {
    let mut result = 0.0;
    let mut pos: &Pos = &self.pos;
    for dirty_pos in self.where_dirty.iter() {
        result += ((dirty_pos.x as isize - pos.x as isize).abs()
            + (dirty_pos.y as isize - pos.y as isize).abs()
            + 1) as f64;
        pos = dirty_pos;
    }
    return result;
}
```

Ho cambiato di nuovo l'euristica, adesso per ogni possizione aggiungo un "1" che rappresenta
il fatto di "pulire", le performance sono aumentate in modo assurdo

```
time: 162.042µs
iterations: 36
max frontier size: 74
```
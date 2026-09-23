# Risultati benchmark

Misura: SearchResult.total_time; release build; same harness; median across runs. Ripetizioni: 7.
Macchina: Linux-7.2.5-200.fc44.x86_64-x86_64-with-glibc2.43; rustc 1.92.0 (ded5c06cf 2025-12-08).

| Sequenza | Commit | Tempo mediano (ms) | Iterazioni | Energia |
|---|---|---:|---:|---:|
| `PHHPHPPHP` | `5842cd21` | 0.040 | 33 | -2 |
| `PHHPHPPHP` | `60443c0b` | 0.024 | 20 | -2 |
| `PHHPHPPHP` | `081ef55f` | 0.018 | 20 | -2 |
| `HHPHPPHHHPPPPHH` | `5842cd21` | 9.615 | 3,475 | -6 |
| `HHPHPPHHHPPPPHH` | `60443c0b` | 5.908 | 2,537 | -6 |
| `HHPHPPHHHPPPPHH` | `081ef55f` | 5.238 | 2,537 | -6 |
| `HHPHPHHHPPPPHHPHPHPP` | `5842cd21` | 100.747 | 30,407 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | `60443c0b` | 36.770 | 13,154 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | `081ef55f` | 31.688 | 13,154 | -7 |

Il tempo esclude compilazione, avvio del processo e disegno della soluzione.

# Benchmark delle euristiche

Mediane su 7 esecuzioni per configurazione; input `heuristics.csv`.
Le barre grigio-blu indicano euristiche presenti nei commit storici; il verde indica le nuove `h_lookahead3_parity` e `h_lookahead4_parity`.
Codice misurato: `08bfb290dc6c599bebf299b89e266b984922db67`; working tree pulito.
Ambiente: Linux-7.2.5-200.fc44.x86_64-x86_64-with-glibc2.43; rustc 1.92.0 (ded5c06cf 2025-12-08); SearchResult.total_time; release build; A*; same sequence for each heuristic.

| Sequenza | Euristica | Tempo mediano (ms) | Iterazioni mediane | Energia mediana |
|---|---|---:|---:|---:|
| `PHHPHPPHP` | legacy | 0.009 | 31 | -2 |
| `PHHPHPPHP` | default 1-step | 0.013 | 61 | -2 |
| `PHHPHPPHP` | lookahead 2 | 0.018 | 31 | -2 |
| `PHHPHPPHP` | lookahead 3 | 0.018 | 20 | -2 |
| `PHHPHPPHP` | lookahead 3 parity | 0.012 | 13 | -2 |
| `PHHPHPPHP` | lookahead 4 parity | 0.015 | 10 | -2 |
| `HHPHPPHHHPPPPHH` | legacy | 0.069 | 431 | -6 |
| `HHPHPPHHHPPPPHH` | default 1-step | 5.057 | 9,808 | -6 |
| `HHPHPPHHHPPPPHH` | lookahead 2 | 9.817 | 10,931 | -6 |
| `HHPHPPHHHPPPPHH` | lookahead 3 | 4.762 | 2,537 | -6 |
| `HHPHPPHHHPPPPHH` | lookahead 3 parity | 3.930 | 2,400 | -6 |
| `HHPHPPHHHPPPPHH` | lookahead 4 parity | 5.020 | 1,577 | -6 |
| `HHPHPHHHPPPPHHPHPHPP` | legacy | 2.252 | 8,883 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | default 1-step | 31.885 | 43,423 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | lookahead 2 | 34.437 | 29,343 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | lookahead 3 | 28.998 | 13,154 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | lookahead 3 parity | 25.799 | 13,224 | -7 |
| `HHPHPHHHPPPPHHPHPHPP` | lookahead 4 parity | 46.004 | 9,482 | -7 |
| `HHPHPHHHPPPPHHPHPHPPHPHPH` | legacy ⚠ subottima | 21.868 | 54,575 | -10 |
| `HHPHPHHHPPPPHHPHPHPPHPHPH` | default 1-step | 2268.714 | 2,351,966 | -11 |
| `HHPHPHHHPPPPHHPHPHPPHPHPH` | lookahead 2 | 2440.517 | 1,444,813 | -11 |
| `HHPHPHHHPPPPHHPHPHPPHPHPH` | lookahead 3 | 2328.762 | 781,854 | -11 |
| `HHPHPHHHPPPPHHPHPHPPHPHPH` | lookahead 3 parity | 2303.622 | 781,043 | -11 |
| `HHPHPHHHPPPPHHPHPHPPHPHPH` | lookahead 4 parity | 2819.917 | 454,097 | -11 |

Energia più bassa corrisponde a più contatti H. Un'euristica che accelera la ricerca ma restituisce energia peggiore non è un miglioramento della qualità della soluzione.

## Avvertenze sulle soluzioni

- `legacy` su `HHPHPHHHPPPPHHPHPHPPHPHPH`: energia -10, migliore -11.

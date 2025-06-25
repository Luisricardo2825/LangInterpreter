| Lib                  | Aloca múltiplos tipos?  | Remoção segura | Zero `unsafe`        | Ideal para...                    |
| -------------------- | ----------------------- | -------------- | -------------------- | -------------------------------- |
| `bumpalo`            | ✅ (via manual control) | ❌             | ⚠️ `unsafe` possível | Compiladores, performance máxima |
| `typed-arena`        | ❌ (1 tipo por arena)   | ❌             | ✅                   | ASTs, uso simples                |
| `slotmap`            | ✅                      | ✅             | ✅                   | Jogos, ASTs, engine systems      |
| `generational-arena` | ✅                      | ✅             | ✅                   | Estruturas mutáveis seguras      |
| `gc_arena`           | ✅                      | ✅             | ✅                   | Interpreters avançados           |

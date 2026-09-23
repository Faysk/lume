# Orçamento de desempenho

Este documento define os alvos de desempenho do Lume até a 1.0.

Eles são **orçamentos verificáveis**, não promessas de benchmark universal. Hardware,
filesystem, codec e mídia real variam; regressões devem ser comparadas no mesmo ambiente.

## Cenário de referência

O benchmark sintético do catálogo usa **250.000 registros** e não depende de mídia pessoal.

Execução manual:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml catalog_250k_fixture_stays_paginated -- --ignored --nocapture
```

O teste registra:

- tempo de uma página no meio do catálogo;
- planos de consulta para filtro por tipo;
- ordenação por data;
- ordenação por tamanho.

## Budgets para a 1.0

| Área | Alvo |
|---|---|
| Abrir catálogo existente | interface utilizável em até **2 s** em SSD local |
| Página comum do catálogo | **p95 ≤ 150 ms** no fixture de 250k |
| Busca/filtro comum | **p95 ≤ 200 ms** no fixture de 250k |
| Primeiro lote durante scan | itens visíveis em até **2 s** após o scanner encontrar mídia suportada |
| Paginação | no máximo **500 registros** por chamada; UI usa 240 |
| Scanner | batch de **128** mídias por transação |
| Thumbnails | até **4** decodificações concorrentes; fila pendente curta |
| Memória | objetivo de **≤ 500 MB** durante navegação normal no catálogo de 250k |

## Regras

- não carregar o catálogo inteiro na UI;
- não gerar thumbnails de toda a biblioteca antecipadamente;
- a viewport tem prioridade sobre trabalho que saiu da tela;
- alterações de filtro descartam resultados antigos;
- toda ordenação paginada precisa de tie-break determinístico;
- índices só entram quando correspondem a consulta real;
- cache pode ser apagado e reconstruído sem perda;
- originais nunca são usados como área de trabalho.

## Limites conhecidos

A paginação atual ainda usa `OFFSET`. Ela é simples e determinística, mas offsets muito
altos podem ficar mais caros. A issue #49 permanece aberta para medir e, se necessário,
migrar as ordenações críticas para cursor/keyset pagination.

O orçamento de memória precisa de medição física do processo Windows antes da 1.0.

# Roadmap

O backlog executável do Lume vive nas GitHub Issues.

Índice central: [Backlog mestre — Lume 0.1 → 1.0](https://github.com/Faysk/lume/issues/97)

O roadmap descreve **releases testáveis**, não datas artificiais.

## Regra

Depois da fundação, nenhuma versão existe apenas para trabalho interno. Cada release precisa resultar em uma build que possa ser aberta e testada como produto.

```text
0.1 → 0.2 → 0.3 → 0.4 → 0.5 → 0.6 → 0.7 → 0.8 → 0.9 → 1.0
```

## 0.1 — Primeira versão visualmente funcional

Epic: [#2](https://github.com/Faysk/lume/issues/2)

A 0.1 já precisa parecer o Lume:

- app Tauri/React funcional;
- SQLite e catálogo persistente;
- selecionar uma pasta;
- scan incremental;
- thumbnails básicos;
- galeria visual responsiva e virtualizada;
- viewer básico de foto/vídeo;
- anterior/próxima;
- estados de loading/erro;
- reabrir mantendo catálogo.

Uma lista técnica de caminhos não satisfaz a 0.1.

## 0.2 — Navegação e descoberta

Epic: [#3](https://github.com/Faysk/lume/issues/3)

- grid/lista;
- pesquisa por nome;
- filtros;
- ordenação;
- densidade das miniaturas;
- scroll/contexto preservado;
- atalhos.

## 0.3 — Múltiplas fontes

Epic: [#4](https://github.com/Faysk/lume/issues/4)

- vários discos/pastas;
- gerenciamento de fontes;
- offline;
- re-scan;
- reconciliação;
- cancelamento;
- paths/permissões;
- progresso por fonte.

## 0.4 — Experiência de mídia

Epic: [#5](https://github.com/Faysk/lume/issues/5)

- dimensões/orientação;
- duração de vídeo;
- ffprobe/ffmpeg controlados;
- poster de vídeo;
- zoom/pan;
- playback e fallback externo;
- GIF/WebP animado;
- painel de informações.

## 0.5 — Escala e performance

Epic: [#6](https://github.com/Faysk/lume/issues/6)

- benchmark de 250 mil registros;
- índices SQLite medidos;
- paginação em escala;
- batching/backpressure;
- thumbnails priorizados pela viewport;
- cache/invalidação;
- 4K e densidades extremas;
- orçamento de desempenho para 1.0.

## 0.6 — UX e acabamento visual

Epic: [#7](https://github.com/Faysk/lume/issues/7)

- sistema visual básico;
- claro/escuro;
- responsividade;
- acessibilidade;
- primeira experiência;
- skeletons;
- mensagens de erro;
- microinterações leves.

## 0.7 — Preferências e conveniências

Epic: [#8](https://github.com/Faysk/lume/issues/8)

- persistência de view/sort/densidade;
- tema;
- tamanho/posição da janela;
- abrir externamente;
- abrir no Explorer;
- copiar path/nome;
- inspecionar/limpar cache.

## 0.8 — Robustez e recuperação

Epic: [#9](https://github.com/Faysk/lume/issues/9)

- disco removido durante scan;
- fechar app durante trabalho;
- arquivos corrompidos;
- permissões;
- migrations;
- integridade do catálogo;
- cache corrompido;
- logs locais;
- suite sintética de falhas.

## 0.9 — Release candidate Windows

Epic: [#10](https://github.com/Faysk/lume/issues/10)

- identidade/versionamento;
- instalador;
- CI;
- artifacts por SHA;
- revisão de scopes;
- licenças;
- instalação limpa;
- upgrade;
- estratégia de assinatura;
- checklist de RC.

## 1.0 — Release estável

Epic: [#11](https://github.com/Faysk/lume/issues/11)

- teste com biblioteca real grande;
- aprovação do orçamento de desempenho;
- regressão completa;
- guia de instalação;
- README final;
- política local-first;
- zero bug bloqueador conhecido;
- tag, artifacts e release notes.

## Gestão do trabalho

- Issues são o backlog oficial.
- Cada issue deve ter entrega e critérios de aceite claros.
- Bugs encontrados ganham issue própria e entram na versão afetada.
- Não puxar problema de uma versão futura para a atual sem necessidade concreta.
- Originais continuam somente leitura durante toda a 1.0.
- O trabalho pode ocorrer diretamente na `main` nesta fase inicial do projeto.

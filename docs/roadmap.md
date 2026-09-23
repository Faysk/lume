# Roadmap

O roadmap descreve ordem de valor, não calendário.

## F0 — Fundação

Estado: **em andamento**

Objetivos:

- visão do produto;
- escopo fechado da V1;
- decisão de tecnologia;
- arquitetura mínima;
- princípios de engenharia;
- estrutura inicial do repositório.

Saída:

- qualquer mudança futura consegue responder claramente se pertence à V1.

## F1 — Vertical slice

Objetivo: provar o caminho completo mais fino possível.

Entregar:

- shell Tauri;
- React/Vite;
- comando para escolher diretório;
- criação de `library.db`;
- cadastro de uma fonte;
- scanner básico;
- persistência de arquivos suportados;
- consulta paginada;
- lista textual simples.

Saída:

> escolher uma pasta e ver seus arquivos cadastrados na tela.

Sem thumbnails nessa fase.

## F2 — Galeria

Entregar:

- grid responsivo;
- virtualização;
- geração de thumbnails;
- cache;
- placeholder;
- tamanho ajustável;
- list view;
- pesquisa por nome;
- filtros básicos;
- ordenação.

Saída:

> usar o Lume já é melhor que navegar manualmente pelas pastas para encontrar mídia.

## F3 — Viewer

Entregar:

- imagem;
- vídeo comum;
- anterior/próxima;
- teclado;
- fullscreen;
- zoom/pan para imagem;
- fallback “abrir externamente” para formatos não reproduzíveis.

Saída:

> uma sessão de navegação não precisa sair do Lume.

## F4 — Robustez

Entregar:

- múltiplas fontes;
- fonte offline;
- re-scan;
- itens removidos;
- cancelamento;
- progresso;
- recuperação após fechar durante scan;
- testes com catálogo grande;
- otimizações guiadas por benchmark.

Saída:

> bibliotecas reais grandes podem ser usadas diariamente.

## V1.0

Critério de release:

- instalação simples em Windows;
- nenhuma ferramenta de desenvolvimento necessária;
- fluxo completo sem terminal;
- originais permanecem intocados;
- desempenho aceitável em catálogo grande;
- documentação operacional curta.

## Depois da V1

Candidatos, sem compromisso de ordem:

- favoritos;
- coleções virtuais;
- metadata fotográfica melhor;
- timeline;
- suporte a mais formatos;
- watcher;
- drag-and-drop de fontes;
- rede local.

IA não tem versão reservada. Ela só entra quando existir um produto básico suficientemente bom para continuar útil com IA desligada.

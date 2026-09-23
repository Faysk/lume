# Arquitetura

## Objetivo arquitetural

O Lume deve permanecer um aplicativo desktop simples: um shell, um frontend, um core nativo e um banco local.

A arquitetura evita servidor HTTP local, microserviços e múltiplos runtimes enquanto não existir uma necessidade concreta.

## Visão

```text
React + TypeScript
        │
        │ Tauri IPC
        ▼
Rust core
 ├─ source service
 ├─ scanner
 ├─ catalog
 ├─ media metadata
 ├─ thumbnail service
 └─ settings
        │
        ├───────────── SQLite: library.db
        ├───────────── cache de thumbnails
        └───────────── fontes somente leitura
```

## Fronteiras

### Frontend

Responsável por interface, navegação, grid/lista, filtros, viewer, acessibilidade e atalhos.

Não varre filesystem, não executa SQL, não decide permissões de paths, não gera thumbnails e não chama binários arbitrários.

### Core Rust

Responsável por validar fontes, acessar filesystem, scan incremental, SQLite, paginação, metadados, thumbnails e integração controlada com sidecars.

Qualquer caminho manipulado deve pertencer a uma fonte cadastrada ou a um diretório interno do Lume.

### SQLite

Um único banco: `library.db`.

O frontend não acessa o banco diretamente. Queries são implementadas pelo core para preservar contratos e permitir evolução do schema.

WAL é a configuração inicial preferida para permitir consultas enquanto a indexação grava novos itens.

## Modelo inicial

### sources

```text
id
root_path
display_name
created_at
last_scan_started_at
last_scan_finished_at
last_seen_at
status
```

### media

```text
id
source_id
relative_path
file_name
extension
media_type
size_bytes
created_at_fs
modified_at_fs
width
height
duration_ms
metadata_state
thumbnail_state
first_seen_at
last_seen_scan_id
```

Não existe SHA-256 obrigatório.

Na V1, identidade lógica pode ser baseada em `source_id + relative_path`. Renomear ou mover um arquivo entre scans pode produzir um novo item. Isso é aceitável até existir um caso real que justifique identidade de conteúdo.

## Scan

```text
usuário adiciona fonte
      ↓
validação de acesso
      ↓
scanner percorre diretórios
      ↓
batch pequeno de itens
      ↓
transação SQLite
      ↓
evento de progresso
      ↓
UI passa a enxergar os itens
```

O scanner não espera metadata avançada nem thumbnails para cadastrar um item.

## Metadata progressiva

Primeira passagem:

- path;
- nome;
- extensão;
- tamanho;
- timestamps;
- tipo básico.

Depois, conforme necessário:

- dimensões;
- duração;
- metadata técnica;
- thumbnail.

Isso prioriza tempo até o primeiro conteúdo visível.

## Miniaturas

Miniaturas são derivados descartáveis.

Regras:

- cache fora das fontes;
- geração sob demanda;
- prioridade para viewport;
- pequeno prefetch adiante/atrás;
- falha por item;
- nenhum warmup obrigatório da biblioteca inteira.

## Consultas da biblioteca

A UI nunca pede toda a biblioteca de uma vez.

O core expõe consultas paginadas, por exemplo:

```text
query_media(filters, sort, cursor/offset, limit)
```

Paginação limita memória/IPC; virtualização limita DOM. As duas serão usadas.

## Arquivos no WebView

A exposição de arquivos locais deve ocorrer somente para fontes selecionadas pelo usuário.

No Tauri, o caminho preferido é asset protocol com escopo persistido para as fontes autorizadas, sem acesso genérico ao filesystem.

## Vídeo

O primeiro caminho é playback suportado pelo WebView para formatos/codecs comuns.

Quando metadata ou poster exigirem processamento externo, `ffprobe`/`ffmpeg` podem ser executados como sidecars controlados pelo core.

Codec não suportado internamente não bloqueia o catálogo: a V1 pode oferecer “Abrir no aplicativo padrão”.

## Concorrência

A V1 não precisa de supervisor geral de jobs.

Precisamos apenas de scanner em background, fila limitada de thumbnails, cancelamento simples, progresso observável e transações pequenas no catálogo.

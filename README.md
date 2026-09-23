# Lume

> Toda a sua mídia, sem tirar nada do lugar.

Lume é um visualizador local-first para reunir fotos, vídeos e GIFs espalhados por discos e pastas em uma única biblioteca visual, sem reorganizar, mover, renomear ou importar os arquivos originais.

O produto nasce de uma regra simples: **resolver muito bem o problema principal antes de aumentar o escopo**.

## Estado

🚧 **Fundação / planejamento da V1**

Os projetos anteriores `signal-archive` e `Memoria-Local` são referências de aprendizado, não bases de código do Lume.

## V1 em uma frase

Selecionar uma ou mais fontes locais e começar a navegar pelas mídias imediatamente.

### Entra na V1

- múltiplos discos e diretórios;
- varredura recursiva somente leitura;
- fotos, vídeos e GIFs;
- galeria em grid virtualizado e visualização em lista;
- miniaturas com tamanho ajustável;
- pesquisa por nome;
- filtros por tipo, extensão, data, tamanho e fonte;
- ordenação;
- viewer de imagem e vídeo com anterior/próxima;
- catálogo SQLite local;
- cache descartável de miniaturas;
- re-scan manual;
- comportamento explícito para fontes offline;
- interface responsiva.

### Não entra na V1

Deduplicação, hashing global, reorganização física, importação, backup, cloud, login, compartilhamento de rede, app móvel, edição e supervisor complexo de jobs.

Se uma funcionalidade não ajuda diretamente o fluxo **abrir → adicionar fonte → ver mídia → filtrar → abrir → navegar**, ela provavelmente não pertence à V1.

## Princípios

1. Originais são somente leitura.
2. Lume não exige reorganizar o disco.
3. A aplicação abre na biblioteca, não em um dashboard.
4. A interface deve continuar fluida com bibliotecas grandes.
5. Indexação é incremental: o que já foi encontrado já pode aparecer.
6. Cache é descartável e nunca é a única cópia de nada.
7. O catálogo pode ser reconstruído a partir das fontes.
8. Uma aplicação, um processo principal, um banco.
9. Toda nova complexidade precisa justificar o problema real que resolve.

## Stack proposta

| Camada | Tecnologia |
|---|---|
| Desktop shell | Tauri 2 |
| UI | React + TypeScript + Vite |
| Core local | Rust |
| Catálogo | SQLite |
| Acesso SQLite | rusqlite |
| Virtualização | TanStack Virtual |
| Imagens | decodificação/thumbnail em Rust quando possível |
| Vídeo | FFmpeg/ffprobe como sidecar quando necessário |
| Plataforma inicial | Windows 11 |

A decisão detalhada e as alternativas consideradas ficam em `docs/architecture/technology.md`.

## Arquitetura inicial

```text
React + TypeScript
        │
        │ Tauri IPC
        ▼
Rust core
 ├─ Sources
 ├─ Scanner
 ├─ Catalog
 ├─ Media
 └─ Thumbnail cache
        │
        ├─ library.db (SQLite)
        ├─ cache/
        └─ fontes somente leitura
```

## Documentação

- [Visão do produto](docs/product/vision.md)
- [Escopo fechado da V1](docs/product/v1-scope.md)
- [Arquitetura](docs/architecture/overview.md)
- [Escolha de tecnologia](docs/architecture/technology.md)
- [Roadmap](docs/roadmap.md)
- [Princípios de engenharia](docs/engineering/principles.md)
- [Decisões arquiteturais](docs/decisions/README.md)

## Critério de sucesso da V1

```text
instalar
→ abrir
→ adicionar uma fonte
→ ver mídias surgindo enquanto a varredura continua
→ pesquisar e filtrar
→ ajustar miniaturas
→ abrir uma foto ou vídeo
→ navegar anterior/próxima
→ fechar
→ abrir novamente e continuar usando a biblioteca
```

Sem terminal, sem Docker, sem setup manual de banco, sem mover arquivos e sem tutorial obrigatório.

---

**Lume** — toda a sua mídia, sem tirar nada do lugar.

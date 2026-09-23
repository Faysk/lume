# Escolha de tecnologia

## Decisão

Stack inicial do Lume:

- **Tauri 2** para o aplicativo desktop;
- **React + TypeScript + Vite** para interface;
- **Rust** para core local;
- **SQLite** como catálogo;
- **rusqlite** para acesso ao banco a partir do core;
- **TanStack Virtual** para virtualização;
- processamento nativo de imagens em Rust quando possível;
- **FFmpeg/ffprobe** como sidecar apenas onde entregar valor real.

## Critérios

A escolha prioriza:

1. bom acesso a filesystem local;
2. baixo overhead permanente;
3. UI moderna e responsiva;
4. capacidade de trabalhar com bibliotecas grandes;
5. distribuição simples em Windows;
6. possibilidade futura de macOS/Linux;
7. um único aplicativo, sem servidor local obrigatório;
8. isolamento claro entre UI e operações privilegiadas;
9. compatibilidade com ferramentas de mídia consolidadas.

## Opções avaliadas

| Opção | Pontos fortes | Custos para o Lume |
|---|---|---|
| Tauri 2 + Rust + React | leve, WebView nativo, core compilado, IPC, permissions/scopes, sidecars, Windows installer | Rust tem curva maior |
| Electron + React | ecossistema enorme, desenvolvimento simples, APIs maduras | embute Chromium + Node, footprint maior para um app que deve ficar leve |
| Wails + Go + React | simples, leve, Go excelente para I/O, usa WebView nativo | ecossistema desktop menor; Wails v3 ainda está em beta na data da decisão |
| Avalonia + .NET | excelente desktop nativo, C#, renderer próprio, Windows forte | grid/media UX e reutilização web exigem mais trabalho; React já é uma vantagem conhecida |

## Por que Tauri

Tauri entrega a separação desejada:

```text
WebView
  ↕ IPC
Rust
  ↕
filesystem / SQLite / sidecars
```

O WebView não precisa ganhar acesso indiscriminado ao computador. Capacidades e scopes podem restringir operações, inclusive arquivos expostos pelo asset protocol.

Também existe suporte oficial para binários auxiliares, MSI/NSIS no Windows, asset protocol e escopos de filesystem.

## Por que React

A galeria é o centro do produto.

React oferece ecossistema maduro, componentes acessíveis, fluxo conhecido, integração direta com TanStack Virtual e responsividade simples.

Não usaremos Next.js. Lume não precisa de SSR, rotas de servidor ou backend Node.

Vite é suficiente.

## Por que Rust no core

O trabalho principal do backend é local:

- filesystem;
- diretórios grandes;
- metadata;
- filas;
- SQLite;
- arquivos binários;
- integração com ferramentas nativas.

Rust permite fazer isso sem manter um runtime adicional em produção.

A regra é importante: **não escrever Rust complexo apenas porque Rust permite**.

## SQLite

SQLite é adequado porque o catálogo é local, existe um único usuário, não precisamos de servidor de banco e consultas estruturadas de filtros/ordenação são naturais.

Configuração inicial:

- SQLite bundled com o aplicativo;
- WAL;
- foreign keys habilitadas;
- migrations versionadas;
- índices somente para consultas reais.

### Biblioteca Rust

Escolha inicial: `rusqlite`.

Não usaremos o plugin SQL do Tauri como API direta do frontend. O banco pertence ao core.

## Virtualização

Escolha inicial: `@tanstack/react-virtual`.

A UI deve virtualizar grid e lista, mas isso não substitui paginação.

```text
SQLite query limitada
      ↓
página de resultados
      ↓
virtualizador
      ↓
somente itens visíveis no DOM
```

## Mídia

### Imagem

Primeiro tentar bibliotecas Rust para formatos comuns, dimensões, resize e geração de thumbnail.

### Vídeo

FFmpeg/ffprobe são candidatos para poster, duração, streams/codecs e metadata técnica.

A integração deve ser feita pelo core, não pelo frontend.

Distribuição de FFmpeg exige atenção à licença e à configuração usada no build. Antes de embutir binários em release, a conformidade LGPL/GPL deve ser documentada.

## Acesso aos originais

Tauri possui asset protocol para transformar paths locais autorizados em URLs consumíveis pelo WebView.

Como as fontes são escolhidas em runtime, o desenho deve usar escopo persistente para manter acesso somente às raízes aprovadas pelo usuário.

Não configurar `**/*` global apenas para facilitar desenvolvimento.

## Estado de UI

Não escolher Redux/Zustand/etc. antecipadamente.

Começar com estado React local, hooks e contexto pequeno quando necessário.

Adicionar biblioteca de estado somente se o fluxo ficar objetivamente difícil de manter.

## Testes

Core:

- `cargo test`;
- filesystem sintético em diretórios temporários;
- banco temporário;
- fixtures pequenas, sem mídia pessoal.

Frontend:

- Vitest;
- React Testing Library;
- testes de filtros, navegação e estados.

Performance:

- fixture sintética com dezenas/centenas de milhares de registros no SQLite;
- arquivos físicos sintéticos quando o comportamento do scanner exigir.

## Referências oficiais

- Tauri architecture: https://v2.tauri.app/concept/architecture/
- Tauri filesystem: https://v2.tauri.app/plugin/file-system/
- Tauri asset protocol: https://v2.tauri.app/security/asset-protocol/
- Tauri sidecars: https://v2.tauri.app/develop/sidecar/
- Tauri Windows installer: https://v2.tauri.app/distribute/windows-installer/
- SQLite WAL: https://sqlite.org/wal.html
- rusqlite: https://docs.rs/rusqlite/
- TanStack Virtual: https://tanstack.com/virtual/latest/docs/introduction
- Electron: https://www.electronjs.org/docs/latest
- Wails: https://wails.io/docs/introduction/
- Avalonia: https://docs.avaloniaui.net/
- FFmpeg legal: https://ffmpeg.org/legal.html

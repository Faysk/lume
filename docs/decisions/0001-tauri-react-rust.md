# ADR-0001 — Tauri + React + Rust

Status: **Aceita**

Data: 2026-09-23

## Contexto

O Lume precisa de uma interface desktop moderna e de operações locais eficientes sobre filesystem, SQLite e mídia.

A aplicação deve evitar um servidor local e múltiplos runtimes permanentes.

## Decisão

Usar:

- Tauri 2 como shell;
- React + TypeScript + Vite para UI;
- Rust como core;
- IPC do Tauri como fronteira entre frontend e operações privilegiadas.

## Alternativas

### Electron

Vantagens:

- ecossistema grande;
- JavaScript ponta a ponta.

Não escolhido porque embute Chromium e Node e adiciona footprint permanente que não traz benefício suficiente para o objetivo enxuto do Lume.

### Wails

Vantagens:

- Go excelente para filesystem;
- WebView nativo;
- modelo simples.

Não escolhido porque Tauri oferece um modelo de capabilities/scopes muito alinhado à necessidade de limitar acesso aos arquivos e Rust integra naturalmente ao seu ecossistema. Wails v3 ainda está em beta na data desta decisão.

### Avalonia

Vantagens:

- stack .NET forte;
- desktop nativo;
- bom suporte Windows.

Não escolhido porque React oferece maior velocidade para construir a galeria responsiva, TanStack Virtual resolve uma parte central da UI e a abordagem web mantém aberta a reutilização visual futura.

## Consequências

Positivas:

- binário desktop compacto comparado a Electron;
- core nativo;
- sem backend HTTP;
- UI moderna;
- isolamento claro;
- sidecars possíveis quando necessários.

Negativas:

- Rust aumenta a curva de desenvolvimento;
- WebView depende do motor disponível no SO;
- formatos de vídeo reproduzíveis podem variar conforme codecs da plataforma.

## Regra de revisão

Reavaliar somente se existir evidência de que Tauri/WebView impede um requisito central do produto.

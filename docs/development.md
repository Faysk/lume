# Desenvolvimento

## Stack

- Node.js moderno compatível com Vite 8;
- Rust stable;
- Tauri 2;
- toolchain Windows exigida pelo Tauri/WebView2.

## Instalação

```powershell
npm install
```

## Executar o aplicativo desktop

```powershell
npm run tauri:dev
```

O Vite é iniciado automaticamente pelo Tauri. Não existe backend HTTP do Lume.

## Validar frontend

```powershell
npm run build
```

## Validar core Rust

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Dados locais

Em desenvolvimento e produção, o Lume usa os diretórios de dados/cache fornecidos pelo Tauri para o aplicativo.

O catálogo se chama:

```text
library.db
```

Thumbnails ficam abaixo do diretório de cache do aplicativo.

Nenhum banco, thumbnail ou artefato do Lume deve ser criado dentro das fontes escolhidas pelo usuário.

## Primeiro fluxo da 0.1

```text
npm run tauri:dev
→ Adicionar pasta
→ escolher diretório com JPG/PNG/WebP/GIF/MP4/MOV/WebM
→ scanner registra os itens em batches
→ galeria aparece progressivamente
→ imagens visíveis recebem thumbnails sob demanda
→ clique abre o viewer
```

## Segurança durante desenvolvimento

Não amplie o asset protocol para `**/*` para “fazer funcionar”.

Fontes escolhidas pelo usuário são adicionadas ao scope em runtime pelo core Rust; o cache do próprio Lume é autorizado separadamente no `tauri.conf.json`.

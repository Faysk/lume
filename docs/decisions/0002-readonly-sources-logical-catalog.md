# ADR-0002 — Fontes somente leitura e catálogo lógico

Status: **Aceita**

Data: 2026-09-23

## Contexto

O problema principal do Lume é visualizar mídias existentes em várias fontes, não reorganizar o armazenamento físico.

Permitir escrita nos originais desde a primeira versão aumentaria risco e escopo.

## Decisão

Na V1:

- fontes são tratadas como somente leitura;
- Lume não move, renomeia nem exclui originais;
- organização é lógica no catálogo;
- banco e cache ficam em diretórios próprios da aplicação;
- remover uma fonte do Lume remove a referência do catálogo, não os arquivos.

## Identidade

A identidade inicial de uma ocorrência é baseada na fonte e no caminho relativo.

Não existe hashing criptográfico obrigatório.

## Consequências

Positivas:

- risco de perda muito menor;
- scan e viewer ficam mais simples;
- usuário não precisa confiar no Lume para reorganizar anos de arquivos;
- banco é reconstruível.

Limitações aceitas:

- rename pode parecer remoção + novo item;
- duplicatas não são identificadas globalmente;
- arquivos idênticos em dois paths aparecem duas vezes.

Essas limitações são preferíveis à complexidade de resolver problemas que a V1 ainda não precisa resolver.

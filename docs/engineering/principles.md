# Princípios de engenharia

## 1. Produto antes da infraestrutura

Nenhuma infraestrutura existe apenas para deixar a arquitetura “completa”.

Primeiro implementar o caminho real do usuário. Generalizar depois de existir repetição ou problema medido.

## 2. Orçamento de complexidade

Antes de adicionar processo, banco, worker, fila, cache, daemon, framework, runtime ou serviço, perguntar se a mesma necessidade pode ser resolvida dentro do core existente.

## 3. Originais imutáveis na V1

O Lume pode abrir, ler e extrair metadata.

O Lume não pode mover, renomear, sobrescrever ou apagar originais.

## 4. Cache é descartável

Qualquer arquivo em cache deve poder sumir e ser recriado.

Nunca guardar informação exclusiva no cache.

## 5. Catálogo reconstruível

`library.db` melhora experiência e performance, mas não é a fonte última dos arquivos.

Uma corrupção do catálogo não pode significar perda dos originais.

## 6. Tempo até primeira mídia

Preferir:

```text
descobrir
→ persistir básico
→ mostrar
→ enriquecer depois
```

Evitar:

```text
descobrir tudo
→ analisar tudo
→ gerar tudo
→ só então mostrar
```

## 7. Não bloquear UI

Operações de disco, SQLite pesado, thumbnails e ferramentas externas não rodam no thread responsável pela interface.

## 8. Paginação e virtualização

Biblioteca grande exige as duas:

- banco não retorna tudo;
- frontend não renderiza tudo.

## 9. Medir antes de otimizar

Não introduzir paralelismo agressivo, memory mapping customizado, GPU, cache em vários discos ou hashing distribuído sem evidência de gargalo.

## 10. Falha localizada

Um arquivo quebrado não derruba scan, fonte, galeria ou aplicação.

Erros são por item sempre que possível.

## 11. Fixtures sintéticas

Testes automatizados não usam acervo pessoal.

Criar diretórios temporários, imagens mínimas, vídeos sintéticos pequenos, nomes estranhos e paths Unicode.

## 12. Segurança simples e explícita

O frontend não recebe permissão ampla de filesystem.

Toda operação privilegiada passa pelo core, valida a fonte, limita paths e evita execução arbitrária de comandos.

## 13. Documentar decisões, não narrar cada commit

Documentação permanente explica por quê, contrato, limite e consequência.

Detalhes temporários pertencem a issues e PRs.

## 14. Definition of Done

Uma feature precisa considerar, conforme aplicável: teste, erro, estado vazio, loading, teclado, fonte offline, biblioteca grande e documentação se alterar contrato.

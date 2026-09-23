# Escopo fechado da V1

Este documento existe para proteger a V1 de expansão prematura de escopo.

## Objetivo

Entregar uma aplicação desktop local que permita escolher múltiplas fontes, descobrir mídias e navegar por elas de forma rápida e agradável sem modificar os arquivos originais.

## Jornada obrigatória

```text
abrir
→ adicionar fonte
→ iniciar descoberta
→ ver itens aparecerem
→ navegar
→ pesquisar/filtrar
→ abrir mídia
→ anterior/próxima
→ fechar
→ abrir novamente sem reconstruir tudo
```

Se essa jornada não estiver boa, adicionar novas áreas de produto não é prioridade.

## Requisitos funcionais P0

### Fontes

- adicionar diretório ou raiz de volume;
- remover a fonte do catálogo sem apagar arquivos;
- permitir múltiplas fontes;
- identificar claramente fonte online/offline;
- não falhar a biblioteca inteira quando uma fonte estiver ausente.

### Descoberta

- scan recursivo;
- originais abertos somente para leitura;
- persistência incremental;
- item descoberto pode aparecer antes do fim da varredura;
- re-scan manual;
- detectar arquivos removidos desde o scan anterior;
- renome ou movimentação pode ser tratado como remoção + novo item na V1.

### Tipos iniciais

Imagens comuns:

- JPEG/JPG;
- PNG;
- WebP;
- GIF.

Vídeos comuns:

- MP4;
- MOV;
- WebM;
- outros formatos podem ser catalogados progressivamente quando o backend conseguir extrair dados de forma confiável.

Um formato catalogado não precisa obrigatoriamente ter playback interno se o codec não for suportado pelo ambiente. O fallback aceitável da V1 é abrir o arquivo no aplicativo padrão do sistema.

### Catálogo

Guardar somente dados necessários para a experiência:

- fonte;
- caminho relativo;
- nome;
- extensão;
- tipo;
- tamanho;
- datas do sistema de arquivos;
- largura/altura quando disponível;
- duração quando disponível;
- estado da miniatura;
- estado de presença da fonte.

Hash criptográfico global não é requisito da V1.

### Galeria e lista

- grid responsivo;
- virtualização;
- slider de tamanho de miniatura;
- loading individual;
- placeholder quando thumbnail não existir;
- manter posição ao voltar do viewer;
- visualização compacta em lista.

### Pesquisa, filtros e ordenação

Pesquisa:

- nome do arquivo.

Filtros:

- tipo;
- extensão;
- fonte;
- intervalo de data;
- intervalo de tamanho.

Ordenação:

- data mais recente/antiga;
- nome A-Z/Z-A;
- maior/menor.

### Viewer

Imagem:

- fit-to-window;
- tamanho real;
- zoom;
- pan.

Vídeo:

- play/pause;
- seek;
- volume;
- fullscreen quando suportado.

Navegação:

- anterior;
- próxima;
- setas do teclado;
- ESC fecha.

### Persistência

Ao reabrir:

- fontes continuam cadastradas;
- catálogo continua disponível;
- thumbnails existentes continuam reutilizáveis;
- fontes offline continuam representadas;
- nenhum scan completo obrigatório apenas para abrir a interface.

## Requisitos não funcionais P0

- nenhuma operação da V1 move, renomeia ou exclui um original;
- scan não pode bloquear a interface;
- consultas devem ser paginadas;
- o frontend não recebe a biblioteca inteira em uma chamada;
- o DOM contém somente uma janela dos itens visíveis;
- falha de thumbnail de um item não interrompe o restante;
- banco e cache ficam em dados da aplicação, nunca dentro das fontes;
- cache pode ser apagado sem perder informação original.

## Fora da V1

Explicitamente fora:

- deduplicação e SHA-256 global;
- favoritos, coleções e tags;
- timeline avançada e mapa;
- importação e organização física;
- watcher em tempo real;
- acesso por celular, servidor LAN e autenticação;
- cloud e sincronização;
- updater automático;
- plugins e API pública.

Algumas dessas capacidades podem chegar depois. Nenhuma deve bloquear a primeira versão utilizável.

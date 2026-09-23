# Visão do produto

## Problema

Fotos, vídeos e GIFs acumulados durante anos acabam espalhados entre discos, pastas, câmeras, celulares, backups e diretórios antigos.

Os arquivos existem, mas revisitar esse conteúdo normalmente exige lembrar onde ele está fisicamente. Mover tudo para uma estrutura nova traz risco, duplicação e trabalho que não resolve o problema principal: **ver o que já existe**.

## Proposta

Lume cria uma camada visual sobre os arquivos existentes.

O usuário escolhe uma ou mais fontes e passa a navegar por todo o conteúdo suportado em uma interface única, sem exigir reorganização física.

A organização do Lume é lógica. O disco continua sendo do usuário.

## Promessa

> Escolha suas fontes e veja sua mídia.

A primeira experiência precisa ser simples:

1. abrir o Lume;
2. adicionar um disco ou diretório;
3. ver itens surgindo enquanto a varredura continua;
4. filtrar ou pesquisar;
5. abrir uma mídia;
6. navegar para anterior ou próxima.

## Para quem

O foco inicial é uma pessoa com uma biblioteca local grande, distribuída em um ou mais discos, que quer reencontrar fotos e vídeos sem migrar tudo para outra plataforma.

O produto não depende de uma organização prévia perfeita dos diretórios.

## O que Lume é

- um visualizador local-first;
- uma biblioteca lógica sobre arquivos existentes;
- um catálogo reconstruível;
- uma experiência de navegação para bibliotecas grandes;
- uma aplicação desktop simples de instalar e abrir.

## O que Lume não é

Na V1, Lume não é backup, DAM profissional, editor, ferramenta de migração, organizador físico automático, serviço em nuvem, servidor doméstico ou plataforma de IA.

## Norte de UX

A aplicação abre na biblioteca.

Não existe uma tela operacional entre o usuário e as mídias. Status de varredura, fonte offline ou geração de miniatura deve aparecer no contexto da própria experiência, sem transformar a aplicação em um painel de infraestrutura.

## Princípio de simplicidade

Cada capacidade nova deve responder a duas perguntas:

1. qual problema real do uso diário ela resolve?
2. esse problema precisa ser resolvido agora?

Se a resposta da segunda pergunta for não, a capacidade fica fora da versão atual.

## Aprendizado dos projetos anteriores

`signal-archive` e `Memoria-Local` provaram conceitos importantes:

- preservar originais;
- separar organização lógica da física;
- trabalhar com múltiplas fontes;
- usar catálogo local;
- tratar cache como descartável.

O Lume não herda a complexidade operacional desses projetos. Eles são fonte de aprendizados, não uma base obrigatória de arquitetura ou código.

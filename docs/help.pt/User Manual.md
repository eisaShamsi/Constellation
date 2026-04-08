# Manual do Usuario do Constellation

**Versao 0.3.4 | Marco 2026**

Constellation e um aplicativo de desktop para Gestao do Conhecimento Pessoal (PKM) que permite gerenciar bibliotecas de notas em Markdown. Desenvolvido com Tauri v2, SvelteKit e Rust, funciona nativamente no Windows, macOS e Linux com suporte completo para arabe e escrita RTL.

---

## Sumario

1. [Primeiros Passos](#primeiros-passos)
2. [Universo e Bibliotecas](#universo-e-bibliotecas)
3. [Criar e Editar Notas](#criar-e-editar-notas)
4. [Pesquisa](#pesquisa)
5. [Vista Estelar (GraphMind)](#vista-estelar-graphmind)
6. [Visualizacao Dividida](#visualizacao-dividida)
7. [Indice](#indice)
8. [Segunda Tela](#segunda-tela)
9. [Propriedades e Frontmatter](#propriedades-e-frontmatter)
10. [Modelos](#modelos)
11. [Tabelas](#tabelas)
12. [Tarefas](#tarefas)
13. [Importador](#importador)
14. [Calendario](#calendario)
15. [Lens](#lens)
16. [Configuracoes](#configuracoes)
17. [Atalhos de Teclado](#atalhos-de-teclado)
18. [Suporte RTL e Arabe](#suporte-rtl-e-arabe)
19. [Seguranca e Privacidade](#seguranca-e-privacidade)
20. [Mapa do conhecimento](#mapa-do-conhecimento)
21. [Motor Cognitivo](#motor-cognitivo)

---

## 1. Primeiros Passos

### Instalacao

Baixe o instalador mais recente na [pagina de versoes do Constellation](https://github.com/eisaShamsi/Constellation/releases):

- **Windows**: Instalador `.exe` (NSIS) ou `.msi`
- **macOS**: Imagem de disco `.dmg`
- **Linux**: Pacote `.AppImage` ou `.deb`

### Primeira Execucao

Ao abrir o Constellation pela primeira vez, o **Assistente de Configuracao do Universo** guia voce atraves de:

1. **Escolha seu idioma** — 15 idiomas disponiveis
2. **Crie ou importe uma biblioteca** — aponte para uma pasta existente com arquivos Markdown, ou comece do zero
3. **Nomeie seu universo** — o universo e o contêiner de todas as suas bibliotecas

### Visao Geral da Interface

| Elemento | Descricao |
|----------|-----------|
| **Barra Lateral (Ribbon)** | Botoes de navegacao: Arvore de arquivos, Busca, Vista Estelar, Calendario, Modelos, Configuracoes |
| **Arvore de Arquivos** | Navegue por notas e pastas dentro das suas bibliotecas |
| **Editor** | Leia e edite suas notas em Markdown |
| **Barra de Abas** | Abra multiplas notas em abas |
| **Barra de Status** | Contagem de palavras, caracteres e tempo de leitura |

---

## 2. Universo e Bibliotecas

### O que e um Universo?

Um **Universo** e o contêiner de nivel superior que abriga todas as suas bibliotecas. Pense nele como seu espaco de trabalho ou colecao de bibliotecas.

### O que e uma Biblioteca?

Uma **Biblioteca** e uma pasta no seu computador contendo arquivos Markdown (`.md`). Voce pode ter multiplas bibliotecas em um unico universo — por exemplo, uma para notas de trabalho e outra para notas pessoais.

### Gerenciar Bibliotecas

- **Adicionar uma biblioteca**: Configuracoes > Bibliotecas > Adicionar Biblioteca, ou arraste uma pasta para o aplicativo
- **Remover uma biblioteca**: Configuracoes > Bibliotecas > clique no botao de remover ao lado do nome da biblioteca
- **Configuracoes da biblioteca**: Cada biblioteca pode ter suas proprias configuracoes de aparencia (fontes, cores)

### Universos Secundarios

Voce pode aninhar universos dentro de outros universos. Um **Universo Secundario** e outra pasta de universo referenciada pelo seu universo principal. As notas dos universos secundarios aparecem na Vista Estelar junto com suas proprias notas, com links entre bibliotecas exibidos como linhas tracejadas.

### Reabertura automática

O Constellation lembra o último universo ativo e o reabre automaticamente ao iniciar. Se o universo foi movido ou seu caminho mudou, o Constellation detecta e corrige o caminho automaticamente.

### Universos portáteis

Os universos do Constellation sao totalmente portáteis. Voce pode mover a pasta do universo para qualquer local — uma unidade diferente, pen drive USB ou outro computador — e o Constellation detectara e corrigira automaticamente todos os caminhos internos ao reabri-lo.

Para mover um universo:
1. Feche o Constellation
2. Mova ou copie a pasta do universo para o novo local
3. Abra o Constellation → a tela de boas-vindas aparece (o caminho antigo nao e mais valido)
4. Escolha **Abrir Universo Existente** e aponte para o novo local
5. Todas as notas e bibliotecas aparecem imediatamente — os caminhos sao corrigidos automaticamente

A estrutura de pastas do universo segue o modelo Obsidian: as notas ficam diretamente na pasta raiz, a configuracao reside em `.constellation/`.

---

## 3. Criar e Editar Notas

### Criar uma Nota

| Metodo | Acao |
|--------|------|
| **Teclado** | `Ctrl+N` |
| **Arvore de Arquivos** | Clique com botao direito em uma pasta > Nova Nota |
| **Mission Control** | `Ctrl+P` > "Nova nota" |

### Modos do Editor

Constellation oferece dois modos de editor, selecionaveis em **Configuracoes > Editor > Tipo de editor**:

#### Editor Markdown (CodeMirror)

O editor padrao para usuarios avancados. Escreva Markdown diretamente com:

- **Pre-visualizacao ao Vivo** — renderiza a formatacao em linha enquanto voce digita
- **Modo Fonte** — mostra a sintaxe Markdown bruta
- **Barra de formatacao** — aparece ao selecionar texto
- **Comandos com barra** — digite `/` para insercoes rapidas
- **Autocompletar de Wikilinks** — digite `[[` para vincular notas
- **Cursores multiplos** — `Alt+Click` ou `Ctrl+D`

#### Editor de Documentos (TipTap)

Uma experiencia WYSIWYG estilo processador de texto com barra visual:

- Negrito, Italico, Sublinhado, Tachado, Realce
- Titulos (H1–H3), Alinhamento de texto
- Listas com marcadores, Listas numeradas, Listas de tarefas
- Citacoes, Blocos de codigo, Linhas horizontais
- Tabelas (inserir, adicionar/remover linhas e colunas)
- Links e Imagens

Ambos os editores salvam como arquivos Markdown padrao. Voce pode alternar entre eles a qualquer momento sem perda de dados.

### Callouts (Destaques)

Crie blocos de destaque estilizados para notas, avisos, dicas e outras indicacoes:

```markdown
> [!note] Informacao importante
> O conteudo do callout vai aqui.

> [!warning] Tenha cuidado
> Esta acao nao pode ser desfeita.

> [!tip]- Clique para expandir
> Conteudo de callout recolhivel.
```

Tipos suportados: `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Cada tipo tem uma cor e icone distintos. Adicione `-` apos o tipo para torna-lo recolhivel (inicia recolhido), ou `+` (inicia expandido).

### Sintaxe de Realce

Envolva o texto com duplo sinal de igual para realca-lo:

```markdown
Este e ==texto realcado== na sua nota.
```

Na Visualizacao ao Vivo, as marcas `==` ficam ocultas e o texto aparece com fundo amarelo.

### Blocos de Codigo

Blocos de codigo delimitados sao exibidos com cor de fundo e rotulo de linguagem:

````markdown
```javascript
const greeting = "Hello, world!";
```
````

O nome da linguagem aparece como um selo acima do bloco de codigo.

### Incorporacao de Imagens

Incorpore imagens diretamente nas suas notas:

```markdown
![Texto alternativo](https://example.com/image.png)   — URL externa
![[photo.jpg]]                                          — arquivo local da biblioteca
```

Na Visualizacao ao Vivo, as imagens sao renderizadas em linha. Imagens locais devem estar na pasta da sua biblioteca. Imagens externas requerem conexao com a internet.

### Barra de Ferramentas de Tabela

Quando o cursor esta dentro de uma tabela markdown, uma barra de ferramentas flutuante aparece com:

- **+ Linha / + Coluna** — adicionar linhas ou colunas
- **- Linha / - Coluna** — remover linhas ou colunas
- **Alinhamento** — alinhamento a esquerda, centro ou direita por coluna
- **Ordenar** — ordenar linhas em ordem crescente ou decrescente pela coluna atual
- **Tab / Shift+Tab** — navegar entre celulas da tabela

### Atalhos de Formatacao de Texto

| Atalho | Acao |
|--------|------|
| `Ctrl+B` | Negrito |
| `Ctrl+I` | Italico |
| `Ctrl+Shift+S` | Tachado |
| `Ctrl+Shift+H` | Realce |
| `Ctrl+K` | Inserir wikilink |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Shift+Z` | Refazer |

### Vincular Notas

Digite `[[` para abrir o autocompletar de notas. Comece a digitar o nome de uma nota e selecione entre as sugestoes. Os links aparecem como wikilinks clicaveis: `[[Nome da Nota]]`.

Voce tambem pode vincular a titulos especificos: `[[Nome da Nota#Titulo]]`.

---

## 4. Pesquisa

Constellation inclui um motor de busca hibrido multilingue baseado em SQLite FTS5 com classificacao BM25, filtros de consulta estruturados e normalizacao otimizada para arabe. A pesquisa e acessivel pela barra lateral.

### Como pesquisar

Clique no icone de pesquisa na barra lateral ou use `Ctrl+Shift+F` para ativar o modo de pesquisa. Digite sua consulta e os resultados aparecem apos um breve atraso (300ms). Pressione `Escape` ou clique em `×` para limpar a pesquisa e voltar a arvore de arquivos.

### Sintaxe de pesquisa

| Sintaxe | Exemplo | O que encontra |
|---------|---------|----------------|
| Texto livre | `gestao de projetos` | Notas contendo essas palavras no titulo ou corpo |
| Filtro de tag | `#pesquisa` | Notas com a tag `#pesquisa` |
| Filtro de propriedade | `status=ativo` | Notas com propriedade frontmatter `status` igual a `ativo` |
| Filtro de wikilink | `links to [[Clima]]` | Notas com link para `[[Clima]]` |
| Escopo de biblioteca | `in:MinhaBiblioteca` | Restringe resultados a uma biblioteca especifica |
| Combinado | `#pesquisa status=ativo economia` | Todos os filtros aplicados juntos |

### Badges de tipo de correspondencia

Cada resultado exibe um badge colorido indicando como a correspondencia foi encontrada. O badge mostra uma letra localizada para acessibilidade (seguro para daltonicos):

| Badge | Cor | Significado |
|-------|-----|-------------|
| **T** | Azul | Correspondencia de titulo — o termo aparece no nome da nota |
| **C** | Verde | Correspondencia de conteudo — o termo aparece no corpo da nota |
| **S** | Roxo | Correspondencia semantica — conceitualmente relacionado (requer modelo de embedding) |
| **P** | Ambar | Correspondencia de propriedade — encontrado via filtro de propriedade frontmatter |
| **#** | Rosa | Correspondencia de tag — encontrado via filtro de tag |
| **W** | Azul claro | Correspondencia de wikilink — encontrado via filtro de wikilink |

As letras dos badges sao localizadas para todos os 15 idiomas suportados.

### Resultados fixados (Navegar entre resultados)

Os resultados permanecem visiveis apos clicar em um deles. A nota aberta e destacada na lista de resultados para que voce saiba qual resultado esta visualizando. Clique em outro resultado para navegar ate ele sem pesquisar novamente.

Para limpar a pesquisa, pressione `Escape` ou clique em `×`.

### Navegacao por teclado

| Tecla | Acao |
|-------|------|
| `Seta para baixo` | Selecionar proximo resultado |
| `Seta para cima` | Selecionar resultado anterior |
| `Enter` | Abrir resultado selecionado |
| `Escape` | Limpar pesquisa e voltar a arvore de arquivos |

### Destaque do termo de pesquisa

Ao abrir uma nota dos resultados, todas as ocorrencias do termo sao destacadas no editor. Funciona com deteccao de diacriticos arabes — pesquisar "ادارة" destacara "إدارة" e todas as variantes diacriticas.

### Historico de pesquisa

Clique no campo de pesquisa quando estiver vazio para ver suas pesquisas recentes (ultimas 20 consultas). Cada entrada mostra o texto da consulta e ha quanto tempo foi realizada. Clique em qualquer entrada para executar essa pesquisa novamente. Use o link "Limpar historico" na parte inferior para apagar todo o historico.

O historico de pesquisa e armazenado localmente no seu dispositivo e persiste entre reinicializacoes.

### Search Hub

O Search Hub e uma experiencia de pesquisa em tela cheia. Clique no icone da lupa na barra do dock para abri-lo. Ambas as barras laterais se recolhem para dar espaco maximo. Digite qualquer termo e o Constellation pesquisa em todos os lugares simultaneamente, agrupando resultados em 5 categorias: Titulos, Conteudos, Tags, Propriedades e Wikilinks. Cada categoria tem uma secao recolhivel com um badge de contagem. Clique em qualquer resultado para abri-lo no editor com todas as ocorrencias destacadas. Um botao "Voltar ao Search Hub" aparece para que voce possa voltar sem pesquisar novamente.

### Operadores de link

O Constellation suporta 6 operadores de pesquisa de topologia de links:

| Sintaxe | O que encontra |
|---------|----------------|
| `links to [[X]]` | Notas que linkam para X (backlinks) |
| `links from [[X]]` | Notas para as quais X linka (links de saida) |
| `mutual [[X]]` | Notas linkadas a X E X linka de volta (bidirecional) |
| `mentions [[X]]` | Notas contendo o nome de X sem um [[wikilink]] |
| `orphans` | Notas sem links de entrada ou saida |
| `links between [[X]] and [[Y]]` | Notas que linkam para X e Y |

Ao digitar qualquer operador de link, o autocomplete `[[` mostra todas as notas no universo. Apos selecionar uma nota, digite `#` para completar titulos ou `|type:` para completar o tipo de link.

---

## 5. Vista Estelar (GraphMind)

A Vista Estelar visualiza suas notas como um grafo 3D interativo alimentado pelo motor **GraphMind** (Pixi.js WebGL).

### Abrir a Vista Estelar

- Clique no icone de grafo na barra lateral
- Pressione `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Vista Estelar"

### Navegacao

| Entrada | Acao |
|---------|------|
| **Clique + arrastar** | Deslocar o grafo |
| **Scroll** | Aproximar/afastar |
| **Clique em um no** | Abrir a nota |
| **Clique direito em um no** | Menu de contexto (Abrir, Focar, Fixar, Ocultar) |
| **Clique do meio + arrastar** | Rotacionar em 3D |
| **W/A/S/D** | Voar pelo espaco 3D |
| **0** | Redefinir rotacao para 2D |
| **Ctrl+F** | Buscar e destacar |
| **Space** | Alternar modo foco |

### Modos de Layout

Pressione `Ctrl+L` para alternar entre:

- **Organico** — layout dirigido por forcas onde agrupamentos surgem naturalmente
- **Hierarquico** — layout em arvore de cima para baixo
- **Temporal** — notas organizadas por data de criacao em uma linha do tempo

### Modo Foco

Clique com botao direito em um no > **Focar** para ver apenas sua vizinhanca. Ajuste:

- **Profundidade** (1–5 saltos) — quantos niveis de conexoes exibir
- **Direcao** (↔/←/→) — todos os links, apenas de entrada, ou apenas de saida

### Navegacao 3D

Clique com o botao do meio e arraste para rotacionar. Use W/A/S/D/Q/E para voar pelo campo estelar. Um indicador de eixos XYZ no canto mostra sua orientacao. Pressione `0` para redefinir.

### Configuracoes

Clique no icone de engrenagem para:

- **Aparencia**: Tamanho do no, visibilidade dos rotulos, tamanho da fonte, espessura dos links, exibir orfaos
- **Fisica**: Forca de repulsao, forca de ligacao, distância de ligacao
- **IA**: Limiar de links semanticos (Fase 2)

### Legenda

A legenda no canto inferior direito mostra as cores de biblioteca/pasta com caixas de selecao para alternar a visibilidade.

### Estratos de Conhecimento

A Vista Estelar classifica automaticamente suas notas em oito estratos de conhecimento com base no nivel de abstracao:

| Estrato | Descricao |
|---------|-----------|
| **Instantaneo** | Notas rapidas e efemeras |
| **Registro** | Eventos datados e entradas de diario |
| **Topico** | Conceitos atomicos sobre uma unica ideia |
| **Mapa** | Notas organizativas que conectam outros topicos |
| **Estrutura** | Modelos e estruturas de pensamento |
| **Principio** | Regras e axiomas verificados |
| **Conviccao** | Valores e crencas fundamentais |
| **Artefato** | Obras concluidas e definitivas |

O estrato e determinado automaticamente a partir do frontmatter, estrutura e links da nota. Voce pode substituir a classificacao manualmente adicionando uma propriedade `stratum` no frontmatter.

### Ciclo de Maturidade

Cada nota passa por um ciclo de maturidade que reflete seu grau de desenvolvimento:

- **Semente** — Ideia inicial ou rascunho bruto
- **Muda** — A nota comeca a tomar forma e tem alguns links
- **Perene** — Nota madura, revisada e bem conectada
- **Canonica** — Referencia definitiva e autoritativa em seu tema

O nivel de maturidade e atualizado automaticamente com base no numero de links, data de revisao e frequencia de edicao. Voce tambem pode defini-lo manualmente atraves da propriedade `maturity` no frontmatter.

---

## 6. Visualizacao Dividida

A visualizacao dividida permite editar multiplas notas lado a lado na janela principal.

### Abrir a Visualizacao Dividida

- **Paleta de Comandos**: `Ctrl+P` e digite "Split View"
- **Atalho de teclado**: Use o atalho atribuido para alternar entre os modos
- **Ciclo**: Desativado → Vertical (lado a lado) → Horizontal (acima e abaixo) → Desativado

### Editar na Visualizacao Dividida

Cada painel e um editor totalmente independente com:
- Barra de ferramentas completa (negrito, italico, titulos, alinhamento, etc.)
- Navegacao por breadcrumb (biblioteca / nome da nota)
- Painel de propriedades e menu suspenso de estagio
- Suporte a salvamento (`Ctrl+S` salva o painel focado)
- Edicao de titulo e renomeacao de arquivo

### Redimensionar Paineis

Arraste o divisor entre os paineis para redimensiona-los. Cada divisor e independente — com 3 ou mais notas abertas, voce pode redimensionar qualquer par adjacente sem afetar os demais. Funciona nos modos vertical e horizontal.

### Foco

Clique em qualquer painel para foca-lo. O painel focado recebe os atalhos de teclado e e rastreado pelos paineis da barra lateral direita (Propriedades, Retrolinks, etc.).

---

## 7. Indice

O Indice e um glossario abrangente de termos de todas as suas bibliotecas — cada palavra significativa, ordenada alfabeticamente com contagens de ocorrencias.

### Abrir o Indice

- **Botao do dock**: Clique no icone do Indice (livro) no dock esquerdo
- **Paleta de Comandos**: `Ctrl+P` e digite "Index"

### Pipeline NLP Multilingue

O Indice processa o texto atraves de um pipeline consciente do idioma antes da indexacao:

- **Arabe**: Algoritmo Lucene Light10 — remove tashkeel, unifica hamza, remove artigo definido (الـ), remove sufixos gramaticais
- **Hebraico**: Remocao de prefixos (ב/ל/מ/ה/ו/כ/ש)
- **Ingles**: Stemming tipo Porter (plurais, formas verbais, sufixos)
- **Frances/Espanhol/Portugues/Alemao**: Remocao de sufixos especificos do idioma
- **Russo/Turco/Hindi/Persa**: Remocao de sufixos morfologicos
- **Todos os 15 idiomas**: Filtragem de palavras vazias (artigos, preposicoes, conjuncoes)

### Navegacao

- **Abas de idioma**: Alterne entre Todos, Arabe, Hebraico, Ingles ou # (caracteres especiais)
- **Barra alfabetica**: Clique em uma letra para filtrar termos que comecam com essa letra — a contagem de termos e atualizada para mostrar quantos correspondem
- **Clique na mesma letra novamente** para limpar o filtro e mostrar todos os termos
- **Modos de ordenacao**: Alfabetico (padrao) ou por frequencia (mais comuns primeiro)

### Editar a partir do Indice

Clique em qualquer nota nas referencias de um termo para abri-la em um painel de pre-visualizacao dividido ao lado do Indice. O painel de pre-visualizacao e um editor completo — voce pode editar, salvar, alterar propriedades e promover o estagio. O termo de pesquisa e destacado na nota e rolado automaticamente.

Pressione `Ctrl+Clique` para abrir a nota como uma aba regular. Um botao "Voltar ao Indice" aparece na barra de abas — clique nele para retornar exatamente onde voce parou no Indice.

### Integracao com a Segunda Tela

Quando a Segunda Tela esta aberta:
- **Clique em um termo** → A Segunda Tela mostra todas as notas contendo esse termo em uma visualizacao dividida (lista de notas + editor)
- **Ctrl+Clique em varios termos** → A Segunda Tela mostra o modo de comparacao com cada termo em sua propria coluna

---

## 8. Segunda Tela

A Segunda Tela e uma janela complementar baseada em modos que se adapta ao modo atual da sua barra lateral.

- **Abrir**: Clique no icone de segunda tela na barra lateral, ou `Ctrl+Shift+2`
- **Fechamento automatico**: Quando voce fecha a janela principal, a segunda tela fecha automaticamente

### Complemento Baseado em Modos

A segunda tela altera seu conteudo com base no modo ativo da barra lateral na janela principal:

| Modo da Barra Lateral | A Segunda Tela Mostra |
|---|---|
| **Explorador de Arquivos** | Painel do Universo — estatisticas, detalhamento de bibliotecas, universos secundarios, tags, notas editadas/abertas recentemente |
| **Navegador** | Visualizacao completa do Navegador para navegar notas |
| **Vista do Ceu** | Arvore da Vista do Ceu com estrutura de diretorios |
| **Vista Estelar** | Complemento da Vista Estelar com retrolinks, links para frente, tags e grafo local |

### Painel do Universo (Modo Explorador de Arquivos)

Quando a janela principal esta no modo Explorador de Arquivos, a segunda tela exibe um painel com:

- **Cartoes de estatisticas** — Nome do universo, contagem de universos secundarios, total de bibliotecas, pastas e notas
- **Universos Secundarios** — Cada universo secundario com suas bibliotecas vinculadas e contagens de pastas/notas
- **Bibliotecas** — Cada biblioteca com contagens de pastas/notas em caixas de estatisticas codificadas por cores
- **Editadas Recentemente** — Notas que voce modificou na sessao atual (rastreadas ao salvar)
- **Abertas Recentemente** — Notas que voce abriu mas nao editou na sessao atual
- **Tags** — Todas as tags de todas as bibliotecas ordenadas por quantidade; clique em uma tag para ver todas as notas que a utilizam

### Interacao do painel

Quando o painel esta ativo na janela principal, clicar nos itens os envia para a segunda tela:

- **Editadas/Abertas Recentemente**: Clique em uma nota para abri-la como editor completo na segunda tela
- **Tags**: Clique em uma tag para mostrar todas as notas que a utilizam em uma visualizacao dividida — lista de notas a esquerda, editor completo a direita

Todas as edicoes na segunda tela sao sincronizadas automaticamente com a janela principal.

### Edicao de Notas na Segunda Tela

A segunda tela suporta edicao completa de notas — digite, salve, renomeie e altere propriedades da mesma forma que na janela principal. As alteracoes sao sincronizadas automaticamente com a janela principal.

### Sincronizacao de Configuracoes

Todas as configuracoes visuais se propagam instantaneamente para a segunda tela — sem necessidade de reiniciar:

- **Idioma**: Alteracoes de idioma da interface se aplicam imediatamente
- **Tema**: O modo claro/escuro/sistema alterna instantaneamente
- **Fontes**: Fonte de interface, fonte de texto, fonte monoespaco e fontes especificas por script
- **Tamanho da fonte**: Tamanhos de fonte de interface e editor
- **Editor**: Comprimento de linha legivel, numeros de linha, barra de ferramentas flutuante
- **Cor de destaque**: Alteracoes na cor de destaque do tema

---

## 9. Propriedades e Frontmatter

As notas podem ter frontmatter YAML no topo:

```yaml
---
tags: [projeto, ativo]
date: 2026-03-19
status: em-andamento
---
```

Constellation detecta os tipos de propriedades automaticamente:

| Tipo | Exemplo |
|------|---------|
| **Texto** | `author: Joao` |
| **Numero** | `priority: 5` |
| **Data** | `date: 2026-03-19` |
| **Lista** | `tags: [a, b, c]` |
| **Caixa de selecao** | `done: true` |
| **Link** | `related: [[Outra Nota]]` |

Alterne a exibicao de propriedades em **Configuracoes > Editor > Propriedades no documento** (Visivel / Oculto / Fonte).

---

## 10. Modelos

Crie modelos de notas reutilizaveis:

1. Crie uma pasta para modelos na sua biblioteca
2. Defina o caminho da pasta de modelos em **Configuracoes > Modelos**
3. Ao criar uma nova nota, escolha um modelo no seletor de modelos

Os modelos suportam variaveis:

| Variavel | Substituida por |
|----------|-----------------|
| `{{date}}` | Data atual |
| `{{time}}` | Hora atual |
| `{{title}}` | Titulo da nota |
| `{{clipboard}}` | Conteudo da area de transferencia |

---

## 11. Tabelas

### Tabelas Markdown

Digite uma tabela Markdown manualmente ou use o comando de barra `/table`:

```markdown
| Cabecalho 1 | Cabecalho 2 |
|-------------|-------------|
| Celula 1    | Celula 2    |
```

### Barra de Ferramentas de Tabela

Quando o cursor esta dentro de uma tabela, uma barra flutuante aparece com:

- Adicionar/remover linhas e colunas
- Alinhar colunas (esquerda, centro, direita)
- Navegar entre celulas com `Tab` / `Shift+Tab`

### Tabelas no Editor de Documentos

O editor de Documentos (TipTap) oferece uma experiencia visual de tabelas:

- Clique no botao de tabela para inserir
- Use o menu suspenso para gerenciar linhas/colunas
- Redimensione colunas arrastando as bordas

---

## 12. Tarefas

Constellation suporta caixas de selecao de tarefas nas notas:

```markdown
- [ ] Tarefa incompleta
- [x] Tarefa concluida
```

No modo de Pre-visualizacao ao Vivo, as caixas de selecao sao clicaveis. As tarefas podem ser buscadas e filtradas em todas as suas bibliotecas.

---

## 13. Importador

Importe notas de outras ferramentas PKM:

- **Obsidian** — importa vaults com compatibilidade completa de wikilinks
- **Pastas Markdown** — importe qualquer pasta de arquivos `.md`
- **Outros formatos** — HTML, arquivos de texto

Va para **Configuracoes > Importador** para iniciar uma importacao.

---

## 14. Calendario

A visualizacao do Calendario mostra as notas organizadas por data:

- Notas com uma propriedade `date` aparecem nos seus respectivos dias
- Notas diarias podem ser criadas para qualquer data
- Navegue entre meses com os botoes de seta

Abra o Calendario na barra lateral.

---

## 15. Lens

Lens fornece visualizacoes filtradas das suas notas:

- Filtre por etiquetas, pastas, propriedades
- Ordene por nome, data ou propriedades personalizadas
- Salve configuracoes do Lens para acesso rapido

---

## 16. Configuracoes

Acesse as Configuracoes pelo icone de engrenagem na barra lateral ou `Ctrl+,`.

### Geral

- Idioma (15 idiomas)
- Tema (Claro / Escuro)
- Fonte da interface, Fonte de texto, Fonte monoespcada, Tamanho da fonte
- Tema de fonte — combinacoes de fontes predefinidas (Maquina de escrever, Classico, Moderno, etc.) para troca rapida

### Editor

- Tipo de editor (Markdown / Documento)
- Visualizacao padrao (Leitura / Edicao)
- Modo de Pre-visualizacao ao Vivo
- Numeros de linha, Guias de indentacao, Verificacao ortografica
- Auto-fechar parenteses, Listas inteligentes

### Bibliotecas

- Adicionar/remover bibliotecas
- Configuracoes de aparencia por biblioteca
- Localizacao da pasta de anexos

### Atualizacoes

- Verificar atualizacoes
- Token do GitHub para atualizacoes de repositorios privados

---

## 17. Atalhos de Teclado

### Globais

| Atalho | Acao |
|--------|------|
| `Ctrl+N` | Nova nota |
| `Ctrl+O` | Star Jump (abertura rapida) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Abrir Vista Estelar |
| `Ctrl+,` | Configuracoes |
| `Ctrl+Shift+F` | Buscar na biblioteca |
| `Ctrl+Shift+N` | Segunda tela |

### Editor

| Atalho | Acao |
|--------|------|
| `Ctrl+B` | Negrito |
| `Ctrl+I` | Italico |
| `Ctrl+K` | Inserir wikilink |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Shift+Z` | Refazer |
| `Ctrl+D` | Selecionar proxima ocorrencia |
| `Ctrl+/` | Alternar comentario |
| `Tab` | Indentar / proxima celula da tabela |

### Vista Estelar

| Atalho | Acao |
|--------|------|
| `Ctrl+F` | Buscar e destacar |
| `Ctrl+L` | Alternar modo de layout |
| `Space` | Alternar modo foco |
| `0` | Redefinir rotacao 3D |
| `W/A/S/D/Q/E` | Voar em 3D |
| `Escape` | Fechar Vista Estelar |

---

## 18. Suporte RTL e Arabe

Constellation oferece suporte de primeira classe para arabe, hebraico, persa, urdu e outros idiomas com escrita RTL:

- **Deteccao automatica**: A direcao da nota e detectada automaticamente a partir do conteudo
- **Interface**: Interface RTL completa quando o idioma arabe/hebraico e selecionado
- **Editor**: Edicao de texto RTL com movimento de cursor e selecao corretos
- **Vista Estelar**: Rotulos em arabe sao renderizados da direita para a esquerda com fallback de fonte adequado
- **Legenda**: Os itens invertem a ordem ponto/texto com base no idioma do conteudo
- **Fontes de escrita**: Configure fontes para arabe, hebraico e CJK independentemente nas Configuracoes

### Configuracao para Arabe

1. Va para **Configuracoes > Geral > Idioma** e selecione Arabe
2. Opcionalmente, defina uma fonte dedicada para arabe em **Configuracoes > Geral > Fontes de escrita**
3. Notas com conteudo em arabe serao renderizadas automaticamente em RTL

---

## 19. Seguranca e Privacidade

- **Todos os dados permanecem locais** — sem sincronizacao na nuvem, sem telemetria, sem rastreamento
- **Arquivos Markdown** — suas notas sao arquivos de texto simples que pertencem totalmente a voce
- **Sem conta necessaria** — Constellation funciona completamente offline
- **Atualizacoes opcionais** — verifique atualizacoes manualmente nas Configuracoes
- **Codigo aberto** — inspecione o codigo em [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 20. Mapa do conhecimento

O Mapa do conhecimento e uma visualizacao radial (sunburst) que mostra a estrutura, densidade e maturidade de todo o seu universo de conhecimento.

### Abrir o Mapa

- **Botao do dock**: Clique no icone do Mapa do conhecimento na barra lateral esquerda
- **Paleta de comandos**: `Ctrl+P` e digite "Constellation Map"

### O que voce ve

- **Centro**: Nome do seu Universo com total de notas e palavras
- **Primeiro anel**: Bibliotecas (cada uma com sua cor). Se seu universo tem universos filhos, eles aparecem aqui tambem.
- **Aneis mais profundos**: Pastas e subpastas dentro de cada biblioteca
- **Segmentos externos**: Notas individuais

### Modos de cor

Alterne entre tres modos pelo menu:
- **Maturidade**: semente (cinza) → muda (verde claro) → perene (verde) → canonico (dourado) → murcho
- **Estrato**: L1 (azul) → L8 (vermelho) — mostra complexidade do conhecimento
- **Biblioteca**: todos os segmentos herdam a cor da sua biblioteca

### Navegacao por aprofundamento

Clique em qualquer segmento de pasta para ampliar. Uma trilha de migalhas mostra seu caminho. Clique em qualquer item da trilha para voltar, ou pressione Escape. Clique em um segmento de nota para abri-la no editor.

### Voltar ao Mapa

Apos abrir uma nota a partir do Mapa, um botao "Voltar ao Mapa" aparece na barra de abas. Clique para retornar exatamente onde estava — mesmo nivel de aprofundamento preservado.

---

## 21. Motor Cognitivo

O Motor Cognitivo e o sistema de inteligencia integrado do Constellation que analisa suas notas e revela padroes ocultos e relacoes entre suas ideias. Sua filosofia fundamental:

> "A quantidade dos seus dados nao importa. Nao se trata de quantas fontes voce armazena, mas de como voce formula seu conhecimento a partir delas e o conecta em uma consciencia unica e significativa."

O Motor Cognitivo e composto por nove ferramentas integradas: Links tipados, Estratos de conhecimento, Ciclo de maturidade, Detector de tensoes, Cadeia de proveniencia, Motor de externalizacao, Pulso de revisao, Trilhas e Visoes multi-lente.

---

### 17.1 Links tipados

#### O que e?

Links tipados sao wikilinks que carregam um tipo de relacao descrevendo como duas notas se conectam. Em vez de simplesmente escrever `[[nota]]`, voce escreve `[[nota|tipo-de-relacao]]` para expressar a natureza do vinculo — e derivada dela? Contradiz? Estende?

#### Por que importa?

Um link comum diz "ha uma conexao" sem dizer qual. Links tipados transformam sua rede de notas de um amontoado de referencias em um verdadeiro mapa de conhecimento que torna visiveis as estruturas de pensamento, dependencias e raciocínios entre ideias.

#### Como usar

1. Abra uma nota no editor
2. Escreva um wikilink com tipo de relacao: `[[Nota destino|derives-from]]`
3. Tipos suportados: `derives-from` (derivado de), `supports` (apoia), `contradicts` (contradiz), `extends` (estende), `exemplifies` (exemplifica), `questions` (questiona)
4. Voce tambem pode adicionar tipos atraves das propriedades da nota na barra lateral direita

#### Onde aparece?

- **Vista Estelar (GraphMind)**: Como linhas coloridas e rotuladas entre os nos
- **Barra lateral direita**: Na aba "Backlinks" com indicacao do tipo de cada link
- **Aba Proveniencia**: Usada para construir a arvore genealogica do conhecimento

---

### 17.2 Estratos de conhecimento

#### O que e?

O Motor Cognitivo classifica automaticamente cada nota em um dos oito estratos: Instantaneo, Registro, Topico, Mapa, Estrutura, Principio, Conviccao, Artefato. A classificacao baseia-se na estrutura, conteudo e numero de links da nota.

#### Por que importa?

Conhecer o tipo de cada nota revela o equilibrio do conhecimento na sua biblioteca. A maioria das suas notas sao meros instantaneos fugaces ou evoluiram para principios e estruturas? Essa consciencia sobre a natureza do conteudo e o primeiro passo para construir conhecimento real em vez de apenas acumular informacoes.

#### Como usar

1. A classificacao ocorre automaticamente — nao e necessario fazer nada
2. Para substituir a classificacao automatica, adicione a propriedade `stratum` no frontmatter:
   ```yaml
   ---
   stratum: framework
   ---
   ```
3. Valores disponiveis: `snapshot`, `log`, `topic`, `map`, `framework`, `principle`, `conviction`, `artifact`

#### Onde aparece?

- **Barra lateral direita**: Na secao de propriedades sob "Estrato"
- **Vista Estelar**: Como cores diferentes dos nos conforme o estrato
- **Configuracoes > Motor Cognitivo**: Para ativar ou desativar a classificacao automatica

---

### 17.3 Ciclo de maturidade

#### O que e?

O motor acompanha o nivel de maturidade de cada nota em quatro estagios: **Semente** → **Muda** → **Perene** → **Canonica**. Cada nota comeca como semente e cresce gradualmente com mais conteudo, links e revisoes.

#### Por que importa?

A maturidade distingue uma ideia bruta de conhecimento refinado. A semente de hoje pode se tornar referencia amanha se voce dedicar a atencao necessaria. O rastreamento de maturidade ajuda a identificar notas que merecem mais desenvolvimento e atencao.

#### Como usar

1. A maturidade muda automaticamente com base em: contagem de palavras, numero de links de entrada e saida e data da ultima modificacao
2. Para definir a maturidade manualmente, adicione a propriedade `maturity` no frontmatter:
   ```yaml
   ---
   maturity: evergreen
   ---
   ```
3. Valores disponiveis: `seed` (Semente), `sapling` (Muda), `evergreen` (Perene), `canonical` (Canonica)

#### Onde aparece?

- **Barra lateral direita**: Um icone ao lado do titulo mostra o estagio de maturidade atual
- **Vista Estelar**: Como tamanho do no — quanto mais madura a nota, maior o no
- **Configuracoes > Motor Cognitivo**: Para ativar ou desativar o rastreamento de maturidade

---

### 17.4 Detector de tensoes

#### O que e?

O Detector de tensoes examina notas vinculadas e alerta quando ha afirmacoes ou conclusoes contraditorias entre duas ou mais notas. Baseia-se na analise de links tipados `contradicts` e na similaridade tematica entre notas.

#### Por que importa?

Tensoes nao sao necessariamente erros — sao convites para pensar mais profundamente. Quando duas ideias na sua biblioteca se contradizem, isso significa que sua compreensao evoluiu ou que existe uma complexidade que vale a pena explorar. Detectar tensoes protege voce de construir conhecimento inconscientemente sobre bases contraditorias.

#### Como usar

1. Adicione um link tipado `contradicts` entre notas conflitantes: `[[Outra nota|contradicts]]`
2. O motor tambem detecta tensoes implicitas por analise de conteudo
3. Consulte a lista de tensoes detectadas na barra lateral

#### Onde aparece?

- **Barra lateral direita**: Na aba "Tensoes" quando contradicoes sao detectadas
- **Vista Estelar**: Como linhas vermelhas tracejadas entre nos conflitantes
- **Painel de notificacoes**: Alertas quando uma nova tensao e detectada

---

### 17.5 Cadeia de proveniencia

#### O que e?

A Cadeia de proveniencia rastreia a origem de cada ideia — de onde veio e do que foi derivada. Utiliza links `[[nota|derives-from]]` para construir uma arvore genealogica que mostra o caminho de evolucao do conhecimento desde a fonte original ate a formulacao atual.

#### Por que importa?

Saber de onde vem suas ideias distingue conhecimento recebido (de livros, artigos, palestras) de conhecimento descoberto (suas proprias conclusoes e reflexoes). Essa consciencia sobre a origem do conhecimento ajuda a avaliar a confiabilidade das suas ideias e entender como seu pensamento se formou ao longo do tempo.

#### Como usar

1. Ao criar uma nota derivada de uma fonte, adicione um link: `[[Fonte original|derives-from]]`
2. Cadeias de multiplos niveis podem ser construidas: nota ← derivada de ← derivada de ← fonte original
3. Classifique fontes externas adicionando `source-type: received` no frontmatter

#### Onde aparece?

- **Barra lateral direita**: A aba "Proveniencia" exibe a arvore genealogica completa
- **Vista Estelar**: Como direcao das setas nos links (da fonte para o derivado)
- **Propriedades da nota**: Classificacao como "recebido" ou "descoberto" com base na cadeia de proveniencia

### 17.6 Motor de externalizacao

#### O que e?

Um pipeline de formalizacao progressiva que rastreia como suas notas amadurecem de capturas brutas a insights cristalizados. Cada nota pode ser atribuida a um de quatro estagios:

| Estagio | Icone | Significado |
|---------|-------|-------------|
| Fugaz | 🌱 | Captura rapida, pensamento passageiro |
| Literatura | 📖 | Reescrita de uma fonte em suas proprias palavras |
| Permanente | 🔗 | Ideia atomica, um conceito, conectada ao seu grafo |
| Sintese | ✨ | Insight original combinando multiplas notas permanentes |

#### Por que importa?

A maioria dos apps trata todas as notas igualmente. O Motor de externalizacao torna a distincao visivel — voce pode ver de relance quanto da sua biblioteca e captura bruta versus compreensao genuina.

#### Como usar

1. Na barra de navegacao (acima do editor), use o menu suspenso de estagios para selecionar um estagio.
2. Ou expanda as Propriedades e use o menu suspenso de estagios la. Ambos sincronizam instantaneamente com a arvore de arquivos.
3. Para promover uma nota, mude o menu suspenso de um estagio para o proximo. No modo Foco, clique em "Promover para Permanente" na parte inferior.
4. Para remover um estagio, selecione "— Estagio —" no menu suspenso.

#### Onde aparece?

- **Barra de navegacao**: menu suspenso com emoji + nome do estagio
- **Painel de propriedades**: menu suspenso quando a propriedade `stage` existe
- **Arvore de arquivos**: icone emoji ao lado do nome da nota
- **Rodape do modo Foco**: botao "Promover para Permanente"

### 17.7 Pulso de revisao

#### O que e?

O Pulso de revisao e um sistema de ressurgimento espacado que traz notas de volta a sua atencao em intervalos crescentes: 1 dia, depois 3, depois 7, depois 14, depois 30 dias apos a ultima revisao. Tambem monitora notas com as tags `#assumption` ou `#model` como pontos de verificacao de modelos mentais, e mantem uma fila "Nunca revisadas" para notas capturadas mas nunca revisitadas.

#### Por que importa?

O conhecimento se dissipa sem revisitacao. Voce escreve uma nota hoje e em tres semanas esquece que ela existe. A repeticao espacada e a tecnica mais estabelecida na ciencia cognitiva para combater essa degradacao. O Pulso de revisao aplica esse principio as suas notas reais.

#### Como usar

1. Clique na aba **Pulso de revisao** na barra lateral esquerda. Voce vera tres secoes: Pendentes de revisao, Pontos de verificacao de modelos mentais (`#assumption` / `#model`), e Nunca revisadas.
2. Clique em qualquer nota para abri-la e ler.
3. Escolha uma das tres acoes:
   - **Revisada** (marca de verificacao) — agenda a proxima revisao no proximo intervalo (1 → 3 → 7 → 14 → 30 dias).
   - **Adiar 7d** (icone de olho) — adia a nota por 7 dias sem avancar o intervalo.
   - **Descartar** (icone de arquivo) — remove a nota da fila de revisao permanentemente.
4. Abra a Paleta de Comandos e digite "Review due notes" para ir direto as notas pendentes.

#### Onde aparece?

- **Barra lateral esquerda**: Aba Pulso de revisao com contador de notas pendentes
- **Paleta de Comandos**: Comando "Review due notes" para acesso rapido

### 17.8 Trilhas

#### O que e?

Trilhas sao sequencias nomeadas e ordenadas de notas — como capitulos de um livro ou paradas em um tour guiado pelo seu conhecimento. Sao definidas adicionando `trail: true` ao frontmatter de uma nota, e listando wikilinks em ordem no corpo da nota.

#### Por que importa?

O conhecimento nem sempre e uma rede. As vezes e um caminho — uma sequencia de aprendizado, uma progressao de argumentos, uma narrativa. As Trilhas capturam essa ordem explicitamente, adicionando uma dimensao linear a sua biblioteca nao linear.

#### Como usar

1. Crie uma nova nota com `trail: true` no frontmatter.
2. No corpo da nota, liste wikilinks na ordem desejada.
3. Ao abrir qualquer nota que pertenca a uma trilha, a barra de navegacao mostra um indicador com o nome da trilha e a posicao (ex. "Minha Trilha 2/5"). Setas de navegacao permitem ir para a nota anterior e proxima.
4. Abra a Paleta de Comandos e digite "Open Trail" para ver todas as trilhas.

#### Onde aparece?

- **Barra de navegacao**: Indicador da trilha com nome, posicao e setas de navegacao
- **Paleta de Comandos**: Comando "Open Trail" lista todas as trilhas

### 17.9 Visoes multi-lente

#### O que e?

Visoes multi-lente permitem visualizar sua biblioteca atraves de diferentes esquemas de classificacao — sem alterar a estrutura de pastas ou duplicar notas. Uma "lente" e um agrupamento virtual que reorganiza notas com base em uma propriedade ou tag. Lentes integradas: "Por estagio" (Fugaz/Literatura/Permanente/Sintese) e "Por topico" (agrupamento por tags). Lentes personalizadas podem ser criadas nas Configuracoes.

#### Por que importa?

Estruturas de pastas impoem uma unica hierarquia, mas o conhecimento nao cabe em uma unica arvore. Visoes multi-lente permitem alternar entre perspectivas sem mover arquivos. As mesmas notas, vistas atraves de diferentes lentes organizacionais.

#### Como usar

1. Na barra lateral, encontre o **seletor de lentes** no topo da arvore de arquivos (padrao "Pastas").
2. Selecione uma lente: "Por estagio", "Por topico" ou uma lente personalizada. A barra lateral se reorganiza instantaneamente.
3. Selecione "Pastas" para retornar a arvore de arquivos padrao.
4. Para criar uma lente personalizada: abra **Configuracoes > Gestao de conhecimento**, clique em **Criar lente**, nomeie e escolha a propriedade frontmatter para agrupamento.
5. Ou use a Paleta de Comandos: digite "Create Lens".

#### Onde aparece?

- **Seletor na barra lateral**: Seletor de lentes no topo da arvore de arquivos
- **Configuracoes > Gestao de conhecimento**: Criar, editar e excluir lentes personalizadas
- **Paleta de Comandos**: Comando "Create Lens"

### Configuracoes do Motor Cognitivo

Todas as ferramentas do Motor Cognitivo podem ser configuradas em **Configuracoes > Motor Cognitivo**:

- **Classificacao de estratos** — Ativar ou desativar a classificacao automatica
- **Rastreamento de maturidade** — Ativar ou desativar o rastreamento do ciclo de maturidade
- **Links tipados** — Ajustar o limiar de sensibilidade para deteccao de links (0.0 – 1.0)
- **Detector de tensoes** — Ativar ou desativar a deteccao automatica de tensoes
- **Substituicao manual** — Adicione propriedades `stratum` e `maturity` no frontmatter para substituir a classificacao automatica

---

*Manual do Usuario do Constellation — Versao 0.3.4 — Marco 2026*
*uconstellation.world*

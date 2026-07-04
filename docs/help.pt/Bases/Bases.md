---
translation_status: AI-generated 2026-05-30 — native-speaker review recommended
language: pt
source: docs/help.uConstellation.World/Bases/Bases.md
aliases:
  - Bases
  - Base Constelação
  - Tabelas de notas
  - Vistas estruturadas
  - Arquivos de base
description: Aprenda a usar a Base Constelação — uma tabela ao vivo das suas notas, uma linha por nota e uma coluna por propriedade, que você pode ordenar, editar e remodelar sem nunca mover um arquivo.
---

# Bases

Uma **Base** transforma um conjunto das suas notas em uma tabela ao vivo: **uma linha por nota, uma coluna por propriedade**. Nada é copiado ou movido — a tabela lê suas notas no lugar e as reflete exatamente como estão agora.

> [!tip] Forte mas simples, por padrão
> Uma Base abre com aparência familiar e despojada — apenas os nomes das suas notas e os campos que lhe importam. As colunas mais profundas e cognitivas do Constellation estão sempre a **um clique de distância**, mas nunca lotam a primeira tela. Você decide quanta estrutura trazer.

> [!info] Não destrutivo
> Uma Base nunca altera suas notas por conta própria. É um pequeno arquivo `.base` que guarda uma consulta — "mostre estas notas, com estas colunas, nesta ordem." Seus arquivos Markdown permanecem exatamente onde estão.

---

## Duas formas de usar uma Base

**1. Como uma aba completa.** Abra um arquivo `.base` e ele preenche a aba como uma tabela interativa.

**2. Dentro de uma nota.** Insira um bloco de código cercado em qualquer nota e ele é renderizado em linha:

````markdown
```base
view: table
```
````

Ambas são movidas pelo mesmo motor, portanto se comportam de forma idêntica.

---

## Criando uma Base

Use **Nova Base** na barra lateral (a ação "+" / Nova Base). O Constellation escreve um pequeno arquivo **YAML** `.base` para você:

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

| Campo | Significado |
|-------|-------------|
| `schema` | Versão do formato (atualmente `1`). |
| `lens` | O nome exibido no topo da tabela. |
| `scope.libraries` | `all`, ou uma lista de bibliotecas específicas a incluir. |
| `scope.federation` | `auto` — inclui também notas de quaisquer Universos vinculados (cUniversos). |
| `columns` | As colunas a exibir. Uma nova Base começa apenas com o **Nome** da nota. |
| `view` | `table` (a tabela é a vista da Base). |

Você raramente precisa editar isto à mão — os próprios controles da tabela (abaixo) escrevem cada alteração de volta no arquivo para você.

---

## A tabela

- **Coluna Nome** — sempre primeiro. Clique no nome de uma nota para abri-la.
- **Cada nota correspondente vira uma linha.** Não há **limite de linhas**. A tabela é *virtualizada* — ela só desenha as linhas atualmente na tela — de modo que uma Base sobre milhares de notas abre instantaneamente e rola suavemente.
- **Direção por célula** — cada valor detecta seu próprio sentido da esquerda-para-a-direita ou da direita-para-a-esquerda, de modo que tabelas multilíngues são lidas corretamente.
- O rodapé mostra quanto tempo a consulta levou.

---

## Encontrando uma nota na tabela

### Pesquisar nesta base

A **caixa de pesquisa** no cabeçalho filtra a tabela conforme você digita. Ela corresponde tanto ao **nome** de uma nota *quanto* ao texto de **cada coluna visível**, de modo que você pode encontrar uma linha por qualquer valor que consiga ver. O selo de contagem ao lado do título mostra **`correspondentes / total`** enquanto você filtra (por exemplo `4/7684`), e volta ao total simples quando você a limpa.

A pesquisa funciona em **qualquer escrita** — digite em árabe para encontrar títulos em árabe, e assim por diante. Como cada linha já está na memória, a filtragem é instantânea mesmo em uma Base de milhares de notas.

### O trilho de letras

Quando uma Base tem **50 linhas ou mais**, uma faixa fina de letras aparece ao longo da borda da tabela. Suas letras são construídas a partir das **primeiras letras dos títulos reais das suas notas** — assim ela mostra **A–Z** para títulos em português e inglês, **أ ب ت …** para o árabe, e as letras corretas para qualquer outra escrita que você tenha. (Em interfaces da direita para a esquerda, o trilho fica automaticamente no lado correto.)

**Clique em uma letra para saltar** para a primeira nota que começa com ela. Se a tabela ainda não estiver ordenada por Nome, clicar em uma letra a **ordena por Nome primeiro** e depois salta — de modo que as letras sempre significam o que você espera.

### Clicar com o botão direito em uma linha

**Clique com o botão direito em qualquer linha** para abrir o menu padrão da nota: **Abrir**, **Abrir em nova aba**, **Marcar**, **Copiar caminho** / **Copiar nome**, **Revelar na árvore de arquivos**, **Abrir no app padrão**, **Mostrar no explorador do sistema** e **Estilo…**. Renomear, mover e excluir *não* são oferecidos aqui de propósito — faça isso pela árvore de arquivos, onde a lista se atualiza com segurança.

---

## Colunas — adicionar, remover, reordenar

### Adicionar uma coluna

Clique em **+ Adicionar coluna**. O seletor é agrupado em dois:

- **Seus campos** — as propriedades de frontmatter que o Constellation encontrou nas suas notas (por exemplo `status`, `maturity`, `author`). Esses são *seus* dados.
- **Constelação** — campos internos que o app sempre conhece: **Nome**, **Caminho**, **Criado** e **Resumo**.

Comece a digitar para filtrar a lista. Campos já presentes na tabela são marcados para que você não os adicione duas vezes.

### Remover uma coluna

Passe o cursor sobre um cabeçalho de coluna e clique no **×**.

### Reordenar colunas

**Pressione e arraste um cabeçalho de coluna para os lados.** A coluna inteira se eleva (ela esmaece e o cabeçalho mostra um contorno de pega), e uma linha vertical marca onde ela vai cair. Solte para movê-la. A coluna Nome permanece fixa como a primeira coluna.

Cada adição, remoção e reordenação é salva de volta no arquivo `.base` automaticamente.

---

## Ordenação

**Clique em um cabeçalho de coluna para ordenar por ele.** Cada clique percorre **crescente → decrescente → desligado** (uma seta mostra a direção atual).

Para ordenar por mais de uma coluna, abra o painel **Ordenar**:

- Adicione várias colunas — a primeira é a ordenação principal, as seguintes desempatam.
- Inverta qualquer nível entre crescente e decrescente.
- Mova níveis para cima ou para baixo para alterar a prioridade, ou remova-os.

---

## Editando uma nota a partir da tabela

Clique duas vezes em uma célula de uma das **suas** colunas de frontmatter para editá-la:

- **Campos de texto livre** — digite o novo valor; **Enter** salva, **Escape** cancela.
- **Campos do tipo lista** (como `maturity`) — surge um **menu suspenso** com os valores válidos **em sua ordem natural** (para `maturity`: *seed → sapling → evergreen → canonical*). Escolha um, ou digite o seu.

A alteração é escrita diretamente no frontmatter YAML daquela nota no disco, e a tabela se atualiza no lugar.

> [!note] Colunas somente leitura
> **Nome** e **Criado** (e as outras colunas internas da Constelação) são calculadas para você, portanto não são editáveis. Apenas seus próprios campos de frontmatter podem ser alterados aqui.

---

## Abrindo uma Base mais antiga

Se você migrar do Obsidian, ou de uma versão anterior do Constellation, seus arquivos `.base` existentes usam um formato mais antigo.

**Seu arquivo nunca é tocado.** Quando o Constellation abre um deles, ele exibe um aviso tranquilo explicando que o formato é mais antigo, e oferece um botão **Converter em Base Constelação**. A conversão acontece **somente quando você clica nele** — ela atualiza o arquivo no lugar para o novo formato YAML (carregando o que puder: o nome, as colunas e filtros de texto simples). Até você optar por converter, o arquivo original é deixado exatamente como estava.

---

## Federação

Uma Base é ciente do Universo. Com `federation: auto`, ela inclui notas de quaisquer Universos vinculados (cUniversos) junto com as suas. Notas que vivem em um Universo vinculado são somente leitura — você pode visualizá-las e ordená-las na Base, mas a edição é reservada às notas que você possui.

---

## Local-first & arquivo-sobre-app

As Bases não guardam dados próprios. Cada valor que você vê vem de um arquivo `.md` real no seu disco, lido ao vivo. Exclua o arquivo `.base` e suas notas permanecem completamente intactas — uma Base é apenas uma lente que você aponta para notas que você já tem.

---

## Teclado & mouse

| Ação | O que faz |
|------|-----------|
| **Digitar** na caixa de pesquisa | Filtrar as linhas por nome e qualquer coluna visível (qualquer escrita) |
| **Clicar** em uma letra no trilho | Saltar para a primeira nota que começa com ela (ordena por nome primeiro, se preciso) |
| **Clicar com o botão direito** em uma linha | Menu da nota: abrir · marcar · copiar · revelar · estilo |
| **Clicar** em um cabeçalho de coluna | Ordenar por ele (crescente → decrescente → desligado) |
| **Arrastar** um cabeçalho de coluna | Reordenar essa coluna |
| **Clicar** no × de um cabeçalho | Remover essa coluna |
| **Clicar duas vezes** em uma célula de frontmatter | Editá-la (menu suspenso para campos do tipo lista) |
| **Enter** | Salvar a edição |
| **Escape** | Cancelar a edição |
| **Clicar** no nome de uma nota | Abrir a nota |

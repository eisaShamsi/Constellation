---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: pt
source: docs/help.uConstellation.World/The Cataloger/The Cataloger.md
aliases:
  - The Cataloger
  - Cataloger
  - Classify notes
  - Classification home
  - CECE home
  - Scan library
  - Classificador
  - Classificar notas
  - Início da classificação
description: O Classificador é o lar de todo o universo para classificar suas notas. É a visão de página inteira na doca onde você executa o Constellation Epistemic Content Engine (CECE) sobre sua Biblioteca, classifica qualquer nota sob demanda, gera resumos das notas e trabalha na fila de revisão. Se a Revisão de Origem é o cartão sobre o qual você age, o Classificador é a sala onde você o faz.
---

# Classificador

> *"Classifique cada nota pelo seu tipo de conhecimento e pela sua fonte."*

O **Classificador** é o lar de todo o universo para a classificação. É uma visão de página inteira, aberta a partir da doca à esquerda, que reúne tudo de que você precisa para ler suas notas através da taxonomia de conhecimento do Constellation em um só lugar: um controle para varrer a Biblioteca inteira, uma forma de classificar qualquer nota individual sob demanda, um botão para gerar resumos das notas, e a fila de revisão ao vivo onde você Aceita, Edita, Rejeita ou desambigua cada sugestão.

Se você já usou o painel **Revisão de Origem** na barra lateral à direita, você já conhece os cartões. O Classificador é o mesmo motor e os mesmos cartões, promovidos para fora de uma aba estreita da barra lateral e recebendo a janela inteira — além de duas coisas que a aba da barra lateral nunca teve: um seletor de notas e um botão "Gerar todos os resumos".

---

## "O Classificador" vs "os catalogadores" — uma palavra rápida sobre nomes

Esses dois nomes parecem semelhantes de propósito, mas significam coisas diferentes:

- **O Classificador** (com *C* maiúsculo, esta visão) é o *lugar* — a sala de página inteira onde a classificação acontece.
- **os catalogadores** (minúsculo, plural) são as *seis lentes* dentro do motor — frontmatter, citações, raízes de palavras, notas vinculadas, notas similares e julgamento de IA — cada uma das quais lê uma nota e vota. Cinco das seis estão ativas hoje; a sexta (julgamento de IA) está construída, mas ainda não foi ligada.

Então: você abre **o Classificador**, e dentro dele **os catalogadores** fazem a leitura. A maquinaria das seis lentes é explicada em detalhe completo no tópico **Revisão de Origem** — este tópico é sobre a sala.

---

## O que é

O Classificador responde a uma pergunta: **"Como cada nota no meu universo está classificada — e o que ainda precisa da minha decisão?"**

Ele é construído em torno de quatro coisas empilhadas de cima para baixo:

1. **Um cabeçalho com três ações** — *Classificar uma nota…*, *Gerar todos os resumos* e *Iniciar varredura*.
2. **Uma faixa de progresso** — aparece apenas enquanto uma varredura da Biblioteca está em execução, mostrando até onde chegou.
3. **A fila de revisão** — os mesmos cartões Aceitar / Editar / Rejeitar / Desambiguar do painel Revisão de Origem, agora em largura total.
4. **Um resumo da nota sob cada cartão** — um breve resumo em linguagem simples da nota para que você possa decidir sem abri-la (veja *Resumos das notas* abaixo, e o tópico dedicado **Resumos das Notas**).

Tudo é executado **no seu dispositivo**. Nenhuma nota jamais sai do Constellation.

---

## Por que importa

A classificação é como o Constellation transforma uma pilha de arquivos `.md` em um corpo de conhecimento *moldado* — cada nota colocada em dois eixos (de onde o conhecimento veio, e que tipo de conhecimento é). Essa forma é o que alimenta o **Constellation Sight**, o painel de **Metadados Epistêmicos** e a busca consciente da taxonomia.

Mas a classificação é um trabalho carregado de decisões. Quando você tem centenas de notas não classificadas, fazê-lo a partir de uma aba estreita da barra lateral — uma nota de cada vez, sem forma de invocar uma nota específica — é lento. O Classificador existe para tornar o trabalho *sentável*: abra-o uma vez, dê-lhe a tela inteira, e trabalhe na sua Biblioteca em uma única sessão focada. O seletor de notas permite que você traga qualquer nota pelo nome; os resumos permitem que você julgue um cartão sem sair da sala; o controle de varredura semeia a fila em massa.

---

## Como abri-lo

1. Na **doca à esquerda** (a faixa vertical de ícones na borda extrema da janela), clique no **ícone de cartões empilhados** — três pequenos cartões em camadas um sobre o outro. Ele fica entre os outros ícones de espaço de trabalho, como o olho do Sight e o neurônio do Nervous System.
2. O Classificador abre como uma **visão de página inteira**, assumindo a área de conteúdo.
3. Para fechá-lo: clique no **(×)** no canto superior direito do cabeçalho, ou pressione **Esc**. Você retorna para onde estava.

> **Nota sobre o Esc:** se o popover de busca *Classificar uma nota…* estiver aberto, pressionar **Esc** fecha apenas o popover e deixa o Classificador aberto. Pressione **Esc** novamente (com o popover fechado) para fechar o próprio Classificador.

---

## O que você vê

### O cabeçalho — três ações

No topo do Classificador, três controles ficam lado a lado:

| Controle | O que faz |
|---|---|
| **Classificar uma nota…** | Abre uma pequena caixa de busca. Digite algumas letras do título de qualquer nota, escolha-a entre os resultados, e o motor a classifica na hora — sem necessidade de abrir a nota primeiro. A nova sugestão aparece na fila abaixo. |
| **Gerar todos os resumos** | Pré-computa um breve resumo para cada nota que ainda não tem um. Roda silenciosamente em segundo plano; o progresso aparece na barra de status na parte inferior da janela; você pode cancelar a qualquer momento. (Detalhado no tópico **Resumos das Notas**.) |
| **Iniciar varredura** | Executa o motor através da sua **Biblioteca ativa inteira** de uma vez, enfileirando uma sugestão para cada nota que ainda não está classificada. Enquanto executa, o botão lê *Executando…* e uma faixa de progresso aparece abaixo do cabeçalho. |

### A faixa de progresso

Diretamente sob o cabeçalho, uma faixa fina aparece **apenas enquanto uma varredura da Biblioteca está em execução**. Ela mostra quantas notas foram processadas e permite que você acompanhe a varredura concluir. Quando nenhuma varredura está em execução, a faixa fica oculta e a fila fica logo abaixo do cabeçalho.

### A fila de revisão

A maior parte do Classificador é a **fila de revisão** — os mesmos cartões que você vê no painel Revisão de Origem, apenas em largura total. Cada cartão mostra uma nota, a leitura do motor sobre como ela se encaixa na sua taxonomia (Source × Content Type), os seis pequenos pontos dos catalogadores, e as ações que você pode tomar:

- **Accept** — escreve a sugestão do motor na nota e limpa o cartão.
- **Edit** — escolha você mesmo os valores a partir de uma árvore.
- **Reject** — limpa o cartão sem escrever nada.
- **Disambiguate** — em um cartão "split", escolha o valor correto entre os chips candidatos.

A mecânica completa dos cartões — os pontos coloridos, os regimes de confiança, a Sibling Disambiguation, os chips de filtro da fila, "Approve all", e a calibração por Biblioteca — está documentada no tópico **Revisão de Origem**. O Classificador usa exatamente esse painel; nada sobre os cartões muda entre a barra lateral e a visão de página inteira.

### O resumo da nota sob cada cartão

Sob o título de cada cartão fica uma breve linha de **Resumo** — algumas frases que dizem do que a nota trata, para que você possa julgar o cartão sem abrir a nota. Isto é produzido pelo **Note Summary Creator (NSC)**; veja a próxima seção e o tópico **Resumos das Notas**.

---

## Classificando uma única nota — o seletor de notas

O botão *Classificar uma nota…* resolve um problema simples: na aba da barra lateral, você só podia classificar a nota que tinha aberta no momento. O Classificador não tem "nota aberta", então ele lhe dá uma forma de invocar qualquer nota pelo nome.

**Para classificar uma nota:**

1. Clique em **Classificar uma nota…**. Uma caixa de busca desce com o espaço reservado *Pesquisar notas…*.
2. Comece a digitar o título da nota. Após uma breve pausa, notas correspondentes aparecem em uma lista (até dez).
3. Clique na nota que você quer. O motor a classifica, o popover fecha, e um cartão novo para essa nota aparece na fila abaixo.
4. Se algo der errado (um raro erro do motor), a mensagem aparece dentro do popover para que você saiba que a classificação não foi executada.

Você não precisa abrir a nota, e você não perde seu lugar na fila. Esta é a forma mais rápida de classificar uma nota específica que você tem em mente.

---

## Resumos das Notas (NSC) dentro do Classificador

Cada cartão na fila carrega um breve **Resumo** da sua nota, mostrado sob o título. O resumo é produzido pelo **Note Summary Creator (NSC)** e segue uma regra firme: **se você escreveu um resumo, o motor usa o seu; ele só gera um quando você não escreveu.**

A ordem de precedência é:

1. **Seu resumo de frontmatter** — um campo `summary:`, `description:`, `abstract:` ou `excerpt:` nas propriedades da nota. Usado exatamente como você o escreveu.
2. **Seu callout de resumo** — um bloco `> [!summary]`, `> [!abstract]` ou `> [!tldr]` no corpo da nota. Usado exatamente como você o escreveu, com diacríticos e tudo.
3. **Um resumo gerado** — apenas se você não escreveu nenhum dos acima. O Constellation lê a nota, encontra suas frases mais centrais, e mostra as três principais em sua ordem original.

O motor **nunca escreve um resumo gerado de volta na sua nota** — seus arquivos `.md` são a fonte da verdade e o Classificador apenas os *lê*.

O botão **Gerar todos os resumos** pré-computa os resumos para a Biblioteca inteira em segundo plano, para que os cartões mostrem seu resumo instantaneamente em vez de preenchê-lo conforme você rola. O detalhe completo — incluindo como os resumos gerados são produzidos e o que fazer se um resumo parecer errado — está no tópico **Resumos das Notas**.

---

## O que o Classificador *não* faz

- **Ele não classifica automaticamente em segundo plano por padrão.** As varreduras são algo que você *inicia*. (Há um modo opcional de segundo plano em Configurações → Inteligência → CECE, desativado por padrão — veja **Revisão de Origem**.)
- **Ele não chama nenhum serviço de nuvem.** Os cinco catalogadores ativos são heurísticos e locais. A sexta lente (julgamento de IA, um modelo de linguagem local) está incorporada ao design, mas ainda não foi ligada, então permanece silenciosa em cada cartão hoje.
- **Ele não altera o texto das suas notas.** Aceitar um cartão escreve *propriedades* de classificação (os campos de frontmatter `sources:` e `content_type:`). Ele nunca edita sua prosa, e nunca escreve um resumo gerado no arquivo.

---

## Fluxos de trabalho comuns

**"Acabei de abrir o Classificador pela primeira vez — por onde começo?"**
Clique em **Iniciar varredura** para enfileirar uma sugestão para cada nota não classificada na Biblioteca. Observe a faixa de progresso preencher. Depois trabalhe descendo a fila, aceitando as que o motor acertou e desambiguando as divididas. Os resumos sob cada cartão permitem que você decida rapidamente.

**"Quero classificar uma nota específica, não a Biblioteca inteira."**
Clique em **Classificar uma nota…**, digite seu título, clique nela. Um cartão aparece na fila. Aceite-o ou edite-o.

**"Meus cartões demoram um momento para mostrar seus resumos."**
Clique em **Gerar todos os resumos** uma vez. Ele pré-computa o resumo de cada nota em segundo plano (progresso na barra de status). Depois que termina, os resumos aparecem instantaneamente.

**"A fila tem centenas de cartões — como eu foco?"**
Use os chips de filtro acima da fila (documentados em **Revisão de Origem**): comece com *Catalogers agreed* e *Approve all* para limpar os fáceis, depois enfrente os cartões divididos.

---

## Tópicos relacionados

- **Revisão de Origem** — os próprios cartões: os seis catalogadores, os pontos coloridos, os regimes de confiança, a Sibling Disambiguation, os filtros da fila, "Approve all", e a calibração por Biblioteca. O Classificador embute este painel.
- **Resumos das Notas** — como a linha de Resumo sob cada cartão é produzida, a precedência que prioriza o autor, e o backfill *Gerar todos os resumos*.
- **Cognitive Engine** — a filosofia mais ampla de formulação do conhecimento na qual a classificação se encaixa.
- **Metadados Epistêmicos** — as propriedades `sources:` e `content_type:` que a classificação escreve, e como lê-las.
- **Constellation Sight** — a visão espacial que a classificação Source × Content Type alimenta.

---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: pt
source: docs/help.uConstellation.World/Note Summaries/Note Summaries.md
aliases:
  - Note Summaries
  - Note Summary
  - Summary
  - NSC
  - Note Summary Creator
  - Build all summaries
  - Resumos das Notas
  - Resumo da nota
  - Gerar todos os resumos
description: Os Resumos das Notas lhe dão um breve resumo em linguagem simples de uma nota para que você possa julgá-la sem abri-la. O Constellation sempre honra um resumo que você mesmo escreveu — no frontmatter ou em um callout de resumo — e só gera um quando você não escreveu. Os resumos gerados são extrativos (as próprias frases mais centrais da nota), somente leitura (nunca escritos de volta no seu arquivo), e computados inteiramente no seu dispositivo.
---

# Resumos das Notas

> *Se você escreveu um resumo, o Constellation usa o seu. Ele só escreve um quando você não escreveu — e mesmo assim, nunca no seu arquivo.*

Um **Resumo da Nota** é um breve resumo de uma nota — algumas frases que lhe dizem do que a nota trata num relance. Os resumos são produzidos pelo **Note Summary Creator (NSC)**. Você os verá em três lugares: sob o título de cada cartão na fila do **Classificador** / **Revisão de Origem** (onde você decide como classificar um cartão sem abrir a nota por trás dele), como uma linha tênue em itálico sob cada resultado nos **Resultados de busca** (que lhe diz *sobre o que* uma nota é, ao lado do trecho que mostra por que ela correspondeu), e como uma faixa fina acima da nota quando você a abre no **Editor** (para que a essência da nota esteja em contexto enquanto você lê ou escreve).

Este tópico explica de onde vêm os resumos, a ordem estrita de precedência que sempre prefere *as suas* palavras às da máquina, como os resumos gerados são construídos, e como pré-computá-los para uma Biblioteca inteira de uma vez.

---

## Por que os resumos existem

Quando você está trabalhando em uma fila de revisão de centenas de cartões, o título sozinho muitas vezes não é suficiente para lembrar o que uma nota realmente diz. Abrir cada nota para refrescar sua memória quebra seu fluxo. Um resumo num relance sob o título conserta isso: você lê três frases, você lembra a nota, você toma a decisão, você segue em frente.

Mas um resumo também é um pequeno ato de autoria. Se você já destilou uma nota em suas próprias palavras — em um campo `summary:` ou em um callout `> [!summary]` — então *esse* é o resumo que deve aparecer, não o palpite de uma máquina. A primeira regra do Constellation para os resumos é, portanto, uma regra sobre respeito pela sua escrita: **o seu vence.**

---

## De onde vem um resumo — a ordem de precedência

Para qualquer nota, o Constellation escolhe o resumo percorrendo esta lista e parando no primeiro que existe:

1. **Seu resumo de frontmatter.** Se as propriedades da nota contêm um campo `summary:`, `description:`, `abstract:` ou `excerpt:` (verificados nessa ordem), seu texto é usado **exatamente como você o escreveu**.
2. **Seu callout de resumo.** Se o corpo da nota contém um callout `> [!summary]`, `> [!abstract]` ou `> [!tldr]`, seu texto é usado **exatamente como você o escreveu** — incluindo diacríticos e pontuação, preservados literalmente.
3. **Um resumo gerado.** Apenas se você não escreveu nenhum dos acima é que o Constellation gera um — lendo a nota e extraindo suas frases mais centrais (veja abaixo).
4. **Um recurso ao texto de abertura.** Para uma nota que o motor não consegue dividir em frases (por exemplo, texto em uma escrita sem pontuação clara de frases), ele mostra as linhas de abertura da nota em vez de um resumo ranqueado.

> **A única regra que mais importa:** os passos 1 e 2 significam que um resumo que você escreveu *nunca* é sobrescrito. Se você vê um resumo gerado em uma nota que você pensava ter resumido, isso significa que o motor não encontrou seu resumo onde ele procura — verifique se seu campo de frontmatter é um dos quatro nomes acima, ou se seu callout é um dos três tipos acima.

---

## Como um resumo gerado é construído

Quando o Constellation tem de gerar um resumo (porque você não escreveu um), ele faz sumarização **extrativa** — ele seleciona frases que já estão na sua nota, em vez de inventar nova prosa. O método é bem estabelecido (TextRank, Mihalcea & Tarau 2004):

1. **Dividir em frases.** O corpo da nota é segmentado em frases usando o padrão Unicode para limites de frases, então funciona através de idiomas e escritas.
2. **Ler o significado de cada frase.** Cada frase é transformada em uma pequena "impressão digital de significado" numérica (um embedding) usando um modelo compacto no dispositivo.
3. **Ranquear por centralidade.** As frases que são mais similares em significado à maioria das *outras* frases pontuam mais alto — essas são as frases que melhor representam a nota como um todo.
4. **Pegar as três principais, em ordem.** As três frases mais bem ranqueadas são mostradas **na ordem em que aparecem na nota**, para que o resumo se leia naturalmente em vez de fora de sequência.

Notas muito longas são tratadas com cuidado — o motor limita quanto do corpo ele escaneia e quantas frases ranqueia, então resumir uma nota enorme nunca atrasa o aplicativo nem arrisca uma falha.

Por ser extrativo, um resumo gerado é sempre feito de frases que você de fato escreveu. Ele nunca colocará palavras na sua boca.

---

## Os resumos são somente leitura — File-Over-App

O Constellation **nunca escreve um resumo gerado de volta na sua nota.** Seus arquivos `.md` são a fonte da verdade; o resumo que você vê em um cartão é computado na hora e armazenado em cache separadamente, não salvo no texto ou no frontmatter do arquivo.

Isto é deliberado, e segue o princípio *File-Over-App* do Constellation: o aplicativo é uma janela para seus arquivos, não um editor que os altera silenciosamente. Se você quer que um resumo viva *na* nota, escreva um você mesmo (um campo `summary:` ou um callout `[!summary]`) — e então, pela regra de precedência acima, o Constellation mostrará o seu e parará de gerar.

Tudo é computado **no seu dispositivo.** Nenhum texto de nota é jamais enviado a lugar nenhum para ser resumido.

---

## Onde os resumos aparecem, e como eles se preenchem

Os resumos surgem por todo o Constellation onde quer que uma nota apareça:

- **Fila do Classificador / Revisão de Origem** — sob o título de cada cartão (a superfície original — veja *O Classificador*).
- **Resultados de busca** — uma linha tênue em itálico sob cada resultado, abaixo do trecho. O trecho mostra *por que* um resultado correspondeu à sua consulta; a linha de resumo mostra *sobre o que* a nota é. Juntos, eles lhe permitem percorrer os resultados sem abrir nada.
- **Editor** — uma faixa fina e suave acima do corpo da nota quando você abre uma nota, para que a essência da nota esteja em contexto enquanto você lê ou escreve. A faixa se esconde quando ainda não há resumo (uma nota recém-criada ou uma cujo resumo ainda está sendo computado).

Por padrão os resumos se preenchem **preguiçosamente e gentilmente**: à medida que os cartões rolam para dentro da visão, à medida que os resultados de busca aparecem, ou quando você abre uma nota, o Constellation computa os resumos ausentes alguns de cada vez, pausando sempre que uma varredura de classificação da Biblioteca está em execução para que os dois nunca competam por recursos. Isto mantém o aplicativo responsivo — você pode ver brevemente um cartão / resultado / nota aberta antes que seu resumo apareça, e então o resumo surge um momento depois.

Se você preferir ter cada resumo pronto com antecedência — para que cada superfície mostre os resumos instantaneamente — use **Gerar todos os resumos**.

---

## Gerar todos os resumos — pré-computando a Biblioteca inteira

O botão **Gerar todos os resumos** (no cabeçalho do **Classificador**) pré-computa um resumo para **cada nota que ainda não tem um atual**, para que os cartões mostrem seu resumo instantaneamente em vez de preenchê-lo conforme você rola.

**Para usá-lo:**

1. Abra o **Classificador** (o ícone de cartões empilhados na doca à esquerda).
2. Clique em **Gerar todos os resumos** no cabeçalho. O botão muda para *Gerando resumos das notas…*.
3. O progresso aparece na **barra de status** na parte inferior da janela — você pode continuar trabalhando enquanto ele roda.
4. Para parar mais cedo, use o controle **Cancelar** na faixa de progresso da barra de status. Uma execução parcial está bem; ela retoma de onde parou na próxima vez.

Algumas coisas que vale a pena saber:

- Ele roda **apenas quando você pede** — ele nunca começa sozinho, então nunca pode atrasar a inicialização do aplicativo.
- Ele roda **em segundo plano** em uma thread separada; a digitação e a navegação permanecem instantâneas.
- Ele é **retomável** — se você o cancelar, ou fechar o aplicativo no meio de uma execução, a próxima execução continua de onde parou em vez de começar do zero.
- Ele só computa resumos que estão **faltando ou desatualizados** — notas cujo resumo já está atual são puladas, então uma segunda execução é rápida.

---

## Garantindo que seu próprio resumo seja usado

Em um cartão, o resumo aparece sob um único rótulo de **Resumo** — o cartão não sinaliza se o texto veio de você ou do motor. O que decide isso é a precedência acima: se uma nota tem um dos campos de frontmatter ou um dos callouts de resumo, o Constellation mostra *esse* e nunca gera um.

Então se uma nota mostra um resumo que parece que a máquina escolheu, essa nota não tem nem um resumo de frontmatter nem um callout de resumo — e a correção é adicionar um:

- Adicione um campo `summary:` (ou `description:` / `abstract:` / `excerpt:`) ao frontmatter da nota, **ou**
- Adicione um callout `> [!summary]` (ou `[!abstract]` / `[!tldr]`) ao corpo.

Na próxima vez que o resumo dessa nota for computado — quando seu cartão carregar a seguir, ou depois que você executar **Gerar todos os resumos** — suas palavras assumem o comando.

---

## Fluxos de trabalho comuns

**"Uma nota mostra um resumo da máquina, mas eu escrevi um."**
O Constellation não encontrou seu resumo onde ele procura. Certifique-se de que seu campo de frontmatter se chama `summary`, `description`, `abstract` ou `excerpt`, **ou** que seu callout é `[!summary]`, `[!abstract]` ou `[!tldr]`. Depois reabra o Classificador (ou clique em *Gerar todos os resumos*) para atualizar.

**"Quero que cada cartão mostre seu resumo no instante em que abro o Classificador."**
Clique em **Gerar todos os resumos** uma vez e deixe terminar. Depois disso, os resumos estão pré-computados e aparecem imediatamente.

**"Quero que o resumo faça parte da própria nota, em disco."**
Escreva-o você mesmo — adicione um campo de frontmatter `summary:` ou um callout `> [!summary]`. O Constellation mostrará então a sua versão (e parará de gerar uma), e suas palavras vivem no arquivo onde qualquer outro aplicativo também pode lê-las.

---

## Tópicos relacionados

- **Classificador** — o lar de página inteira onde os resumos aparecem sob cada cartão, e onde *Gerar todos os resumos* vive.
- **Revisão de Origem** — os cartões de classificação sobre os quais os resumos ficam.
- **Properties** — os campos de frontmatter `summary:` / `description:` / `abstract:` / `excerpt:`, e como adicioná-los.
- **Editing and Formatting** — como escrever um callout `> [!summary]` em uma nota.

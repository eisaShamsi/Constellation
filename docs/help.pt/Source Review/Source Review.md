# Source Review

> **Nota de tradução:** Este tópico de ajuda é uma tradução gerada
> por IA a partir da versão canônica em inglês em
> `help.uConstellation.World/Source Review/Source Review.md`. Revisão
> por falantes nativos pendente. Por favor, envie correções através
> do repositório do projeto.

*(Constellation Epistemic Content Engine — CECE)*

O painel Source Review é onde a Constellation pede que você revise as classificações produzidas pelo **Constellation Epistemic Content Engine** (CECE). Cada cartão na fila mostra uma nota + a leitura do motor sobre como essa nota se encaixa em sua taxonomia de conhecimento. Você Aceita, Edita, Rejeita ou escolhe um chip de Sibling Disambiguation — e ao longo do tempo o motor aprende a forma da sua Library.

Este tópico explica cada parte de um cartão Source Review, o que os pontos coloridos significam, quando confiar no motor e como navegar por centenas de cartões sem rolar para sempre.

---

## O que o CECE realmente faz

Quando você classifica uma nota (clique direito → "Sugerir fontes e tipo de conteúdo", ou via Configurações → botão Executar varredura), o CECE executa **seis catalogadores independentes** sobre a nota. Cada catalogador lê a nota através de sua própria lente — frontmatter, citações, raízes de palavras, notas vinculadas, notas similares, julgamento de IA — e vota em duas perguntas:

- **Source (eixo horizontal)**: de onde *veio* este conhecimento? Exemplos: testemunho (alguém me contou), percepção (eu vi), inferência (eu deduzi), revelação (texto sagrado) e mais oito.
- **Content Type (eixo vertical)**: que *tipo* de conhecimento é este? Exemplos: estado epistêmico (dúvida / certeza / crença), conteúdo semântico (conceito / proposição / fato / teoria), entrada sensorial, entidade simbólica, construto de ordem superior (cosmovisão / doutrina).

Os dois eixos são **independentes**. Uma nota sobre "Eu duvido do pouso na Lua" é testemunho (alguém relatou) no eixo source + estados-epistêmicos/dúvida (a postura do usuário em relação a isso) no eixo content-type.

Após os catalogadores votarem, uma **camada de síntese** combina seus votos em uma única classificação por eixo, com um de três regimes de confiança:

- **Unanimous** — todos os catalogadores que se manifestaram concordaram
- **Strong majority** — a maioria concordou, um discordou (o cartão mostra o nome do dissidente)
- **Split** — sem maioria clara; o motor "se recusou a atribuir" e pede que *você* escolha

Tudo é executado **no seu dispositivo**. Nenhuma nota jamais sai da Constellation.

---

## Os dois eixos em linguagem simples

### Source — *de onde veio este conhecimento?*

Onze valores possíveis mais *não classificável*:

- **Percepção** — observação sensorial em primeira mão
- **Inferência** — raciocínio a partir de premissas (dedução, indução, analogia)
- **Testemunho** — relato de outra pessoa (uma citação, uma referência, uma fonte citada)
- **Transmissão em massa** — relatos convergentes de muitas testemunhas independentes (sunita *al-tawatur*)
- **Comparação** — conhecimento por analogia a um caso conhecido (jurídico *qiyās*, analogias científicas)
- **Postulação** — inferência para a melhor explicação (*arthapatti*)
- **Não-apreensão** — conhecimento da ausência
- **Memória** — recordação de experiência passada
- **Disposição inata** — conhecer pré-experiencial (*fitrah*)
- **Inspiração** — apreensão mística ou criativa (*kashf*)
- **Revelação** — transmissão de texto sagrado ou profético (*al-wahy*)
- **Não classificável** — optar por sair desta classificação

### Content Type — *que tipo de conhecimento é este?*

Cinco ramos de nível superior com sub-ramos:

- **Entradas sensoriais** — sinais brutos (visuais, acústicos, químicos, …)
- **Entidades simbólicas** — sinais, símbolos, códigos
- **Conteúdos semânticos** — conceitos, proposições, fatos, ideias, informações
- **Estados epistêmicos** — dúvida, crença, opinião, certeza, conhecimento, ilusão
- **Construtos de ordem superior** — teorias, doutrinas, cosmovisões, paradigmas

Ambos os eixos têm várias camadas de refinamento sob cada valor de nível superior (por exemplo, *epistemic-states/knowledge/by-content/propositional* é uma folha).

---

## Os seis catalogadores

Cada catalogador é uma *lente* através da qual o CECE lê uma nota. O cartão Source Review os mostra como **seis pequenos pontos coloridos** no canto superior direito. Passe o cursor sobre qualquer ponto para ver seu nome + status.

| Ponto | Catalogador | O que lê |
|---|---|---|
| 🔵 azul | **Seu frontmatter** (Autoridade do Usuário) | Os campos `sources:` e `content_type:` que você já definiu. Se você classificou a nota você mesmo, esta lente tem *autoridade absoluta* — a síntese adota sua escolha e pula as outras. |
| 🌹 rosa | **Citações e estrutura** (Estrutural) | Citações, blockquotes, blocos de código, marcadores de teorema, frases de definição ("o conceito de X é definido como…"), referências a figuras. Lê a forma estrutural da nota. |
| 🟡 âmbar | **Raízes e léxico** (Linguístico) | Análise de raízes árabes (CAE), correspondência de palavras-chave de superfície, equivalência de termos entre idiomas (Bridge). Captura classificação consciente do árabe que embeddings puros perdem. |
| 🟢 azul-petróleo | **Notas vinculadas** (Grafo) | Living Links tipados (`[[Note\|supports]]`, `[[Note\|contradicts]]`, etc.) para outras notas classificadas. Herda a classificação dos vizinhos quando se agrupam. |
| 🟣 violeta | **Notas similares** (Semântico) | Similaridade de embeddings com suas notas já classificadas (k-Nearest-Neighbor). Traz o consenso quando o vetor de conteúdo desta nota se agrupa com notas classificadas. |
| 🟢 verde | **Julgamento de IA** (Raciocínio) | Um LLM local (Qwen3-4B Q5_K_M) executando inferência restrita por gramática. *Ainda não ativo* — fiação do modelo adiada para uma versão posterior. O ponto permanece silencioso em cada cartão hoje. |

### Status do ponto

- **Preenchido** — manifestou-se + concorda com a síntese
- **Em anel** — manifestou-se + discorda da síntese (esta lente escolheu algo diferente)
- **Contorno tracejado** — silencioso (sem sinal nesta lente para esta nota)

O agrupamento de pontos é o indicador de saúde do conjunto em um relance. Um cartão com todos os seis pontos preenchidos é a classificação mais forte possível do motor (raro). Um cartão com um ou dois pontos em anel está mostrando seu raciocínio honestamente — as lentes discordaram.

---

## Os três regimes de confiança

Após os catalogadores votarem, o CECE rotula cada eixo com um de três regimes:

- **Unanimous** — todos os catalogadores que se manifestaram escolheram o mesmo valor primário. O cartão não tem pílula especial.
- **Strong majority (uma discordância)** — a maioria concordou; um dissidente é mostrado pelo nome. O cartão tem uma pílula roxa "Strong majority" no cabeçalho.
- **Split** — sem maioria clara. O cartão tem uma pílula dourada "Catalogers split — needs your call", **uma borda esquerda dourada** e um formulário de Sibling Disambiguation com chips para você escolher.

Cada eixo recebe seu próprio regime independentemente. Um cartão pode ser Unanimous na horizontal + Split na vertical (ou vice-versa). A pílula do cabeçalho resume o pior regime entre ambos os eixos.

---

## Sibling Disambiguation

Quando um eixo é Split, o CECE se recusa a adivinhar e em vez disso traz à tona os valores candidatos como **chips de rádio** sob um prompt:

> *"Os catalogadores se dividiram entre estes candidatos. Escolha qual se encaixa melhor na nota:"*

Você clica em um chip → o motor escreve essa escolha no frontmatter da nota, remove o cartão da fila e atualiza os dados de confiabilidade por Library.

Se o OUTRO eixo estava resolvido (Unanimous ou Strong majority), o CECE *também* escreve o valor desse eixo ao mesmo tempo — então um único clique no chip termina ambos os eixos, não apenas o que você escolheu. O mesmo cartão nunca pergunta a você duas vezes.

Se ambos os eixos forem Split, você escolhe um chip por eixo (dois cliques).

---

## A trilha de raciocínio

Cada cartão tem um alternador **"▸ Por que esta classificação?"** (ou "▾ Ocultar raciocínio" se aberto). Expandi-lo mostra uma linha por catalogador que se manifestou:

- **Ponto na cor da lente** correspondente ao agrupamento de pontos
- **Rótulo do catalogador** (por exemplo, "Raízes e léxico")
- **Confiança auto-relatada** entre colchetes: `[high]` `[medium]` `[low]`
- **Raciocínio de uma linha** explicando o que disparou (por exemplo, *"Linguistic match: vertical → semantic-contents/concept (weight 0.80)"*)
- **Chips de regra amigáveis** abaixo do raciocínio, como `Surface keyword match`, `Side-channel preference rule`, `Arabic root match (CAE)` — estas são as regras específicas que cada catalogador acionou

Durante suas **primeiras 50 revisões** a trilha se expande automaticamente em cada cartão para que você possa construir intuição sobre quando confiar no motor. Depois disso, a trilha se recolhe sob demanda em cartões Unanimous e permanece auto-expandida em cartões Strong majority + Split (onde a discordância é informativa).

Você pode substituir esse padrão a qualquer momento em Configurações → Intelligence → CECE → Visibilidade da trilha de raciocínio:

- **Sempre mostrar** — aberto em cada cartão
- **Apenas em discordância (padrão)** — aberto em cartões Split + Strong majority, além das primeiras 50 revisões
- **Sempre ocultar** — clique manual necessário para expandir

---

## O filtro de composição da fila

Acima da barra de contagem há **cinco chips** que dividem sua fila pelo tipo de decisão que cada cartão precisa de você:

| Chip | Mostra |
|---|---|
| **All** *(padrão)* | a fila completa |
| **Both axes need your call** | cartões onde TANTO horizontal QUANTO vertical são Split |
| **Source needs your call** | cartões onde horizontal é Split + vertical está resolvido |
| **Content type needs your call** | cartões onde vertical é Split + horizontal está resolvido |
| **Catalogers agreed** | cartões onde nenhum eixo é Split — candidatos rápidos para carimbar |

Cada chip mostra a contagem do seu balde (por exemplo, *"Source needs your call (43)"*). Baldes vazios são esmaecidos e desativados. Clicar em um chip re-renderiza os cartões visíveis; a barra de contagem e a matemática Approve All sempre operam na fila **completa** independentemente do filtro ativo, então você sempre pode ver os totais reais.

O filtro resolve o problema da agulha no palheiro quando sua fila tem centenas de cartões. Quer limpar todos os candidatos a carimbar primeiro? Clique em **Catalogers agreed** e depois clique em **Approve all**. Quer focar nos casos mais difíceis? Clique em **Both axes need your call**.

---

## Ações por cartão

Cada cartão tem quatro ações na parte inferior (ou três em cartões Split onde Disambig substitui Accept/Edit):

- **Accept** — escreve o valor primário da síntese do motor em ambos os eixos no frontmatter da nota, remove o cartão da fila. Atualiza a confiabilidade por catalogador.
- **Edit** — abre um seletor de árvore para ambos os eixos; você escolhe os valores manualmente. Mesma atualização de confiabilidade.
- **Reject** — limpa o cartão sem escrever nada. O motor sugerirá novamente se você reclassificar mais tarde. (Rejeição NÃO atualiza a confiabilidade — o usuário "não quer nenhum desses" é ambíguo como sinal de feedback.)
- **Chip de Sibling Disambiguation** — em cartões Split, clique em um dos chips candidatos. Escreve o valor escolhido (e escreve automaticamente o outro eixo se estava resolvido).

---

## O período de calibração de confiança

Suas **primeiras 50 revisões** de cartões classificados pelo CECE são um *período de calibração de confiança*. Durante este tempo, a trilha de raciocínio se expande automaticamente em cada cartão (independentemente do regime), e um banner discreto no topo do painel lembra você: *"Showing reasoning trails until you review N more cards — helps you learn when to trust the catalogers."*

Após 50 revisões o banner desaparece e as trilhas recolhem para o comportamento padrão sob demanda. Você pode substituir via Configurações se quiser mantê-las sempre abertas ou sempre fechadas.

O propósito do período de calibração: o CECE é um sistema probabilístico que melhora à medida que você o corrige (confiabilidade por Library). Ver *por que* cada catalogador votou como votou durante as primeiras 50 revisões permite que você construa sua própria intuição sobre quando suas conclusões são confiáveis no conteúdo específico desta Library.

---

## Calibração por Library

Configurações → Intelligence → CECE → **Per-Library calibration** abre uma tabela somente leitura mostrando a precisão por eixo de cada catalogador na Library ativa:

```
Cataloger          Horizontal      Vertical
─────────          ──────────      ────────
Your frontmatter   12/12 (100%)    4/4 (100%)
Citations          18/22 (82%)     6/8 (75%)
Wordstems          24/28 (86%)     20/26 (77%)
Linked notes       3/4 (uniform)   2/3 (uniform)
Similar notes      14/19 (74%)     12/19 (63%)
AI judgment        — (not running) — (not running)
```

Os números são contagens corretos/total. A porcentagem é mostrada após um catalogador ter 20+ correções nessa Library × eixo (o limiar para dados de precisão estáveis). Abaixo do limiar, o rótulo mostra **(uniform)** — o catalogador contribui com votos de peso uniforme até que dados suficientes se acumulem.

Diferentes Libraries podem ter precisões por catalogador muito diferentes. O catalogador Linguístico se destaca em Libraries pesadas em árabe; o catalogador de Grafo se destaca em Libraries densamente vinculadas. A camada de síntese usa os dados de calibração por Library para ponderar votos — então um catalogador que esteve errado 70% do tempo *nesta* Library tem seus votos com peso reduzido na próxima rodada de síntese.

---

## Classificação em segundo plano

A fila Source Review pode crescer de duas maneiras:

1. **Manual** (padrão) — você clica com o botão direito em uma nota → "Sugerir fontes e tipo de conteúdo", ou aciona Configurações → Executar varredura de classificação.
2. **Segundo plano** — Configurações → Intelligence → CECE → Classificação em segundo plano. Dois modos:
   - **On note save** — auto-classifica cada nota ~1,5 segundos depois que você para de digitar (pega carona no salvamento debounced existente; nunca dispara por tecla pressionada).
   - **On app start** — varre notas não classificadas uma vez por inicialização.

A classificação em segundo plano está **desativada por padrão**. Ambos os modos em segundo plano são executados em uma thread de segundo plano + emitem eventos de progresso; a digitação permanece instantânea; você pode cancelar a partir do cabeçalho do painel Source Review.

---

## Fluxos de trabalho comuns

**"Acabei de instalar o CECE — por onde começo?"**
Abra o painel Source Review. Clique com o botão direito em 5-10 notas da sua árvore de arquivos → "Sugerir fontes e tipo de conteúdo" para semear a fila. Clique pelos cartões um de cada vez. A trilha de raciocínio se expande automaticamente durante suas primeiras 50 revisões — leia-a. Após 5-10 cartões você começará a ver quais catalogadores são confiáveis no seu conteúdo.

**"Minha fila tem 1.200 cartões — onde foco?"**
Use os chips de filtro. Comece com **Catalogers agreed** (candidatos a carimbar) → clique em Approve all para limpá-los. Depois **Source needs your call** + **Content type needs your call** para casos Split que precisam de uma decisão cada. **Both axes need your call** é o conjunto mais difícil; deixe para o final.

**"Como sei quando escolher Accept vs Reject vs Edit vs Disambig?"**
- **Accept** quando o valor primário da síntese corresponde à sua leitura da nota.
- **Reject** quando nenhuma das sugestões se encaixa (por exemplo, o motor perdeu algo que você sabe sobre a nota).
- **Edit** quando você quer um valor que não está em nenhuma das sugestões.
- **Chip de Sibling Disambiguation** quando o cartão é Split e um dos candidatos está correto.

**"Como vejo em quais catalogadores mais confio?"**
Abra Configurações → Intelligence → CECE → Per-Library calibration. A tabela mostra a precisão por catalogador através das correções que você fez nesta Library.

---

## Tópicos relacionados

- **Cognitive Engine** — a filosofia mais ampla de formulação do conhecimento na qual o CECE se encaixa.
- **Properties** — os campos de frontmatter `sources:` e `content_type:` nos quais o CECE escreve.
- **Knowledge Hierarchy** — como Source × Content Type se encaixa na estrutura Universe / Library / Folder / Note.

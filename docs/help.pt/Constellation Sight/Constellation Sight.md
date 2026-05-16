---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: pt
source: docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md
aliases:
  - Constellation Sight
  - Visões Coordenadas
  - Domo de Âncora
  - Mini-Domos
description: Constellation Sight visualiza todo o seu universo de conhecimento como um domo de âncora estratificado com quatro mini-domos coordenados que recodificam as mesmas notas através de diferentes canais (Confiança, Estágio, Atos, Procedência). Passe o cursor sobre qualquer estrela para vê-la em todos os lugares; clique nos chips da barra lateral ou Shift+clique nas estrelas para filtrar; promova qualquer mini-domo para inspeção em tamanho completo com zoom.
---

# Constellation Sight

## O que é?

**Constellation Sight** é o **instrumento diagnóstico** para o seu universo de conhecimento. Um **domo de âncora** central mostra cada nota posicionada por **estrato** (profundidade do pensamento) e **tempo** (quando escrita), com quatro **mini-domos** ao lado que recodificam o mesmo universo através de diferentes canais: **Confiança**, **Estágio**, **Atos**, **Procedência**.

Responde a uma pergunta com cinco lentes complementares: **"Como meu Conteúdo Epistêmico está moldado e organizado?"**

Passe o cursor sobre qualquer estrela em qualquer domo e a mesma nota acende em todas as cinco superfícies — anel dourado na estrela, tom dourado nos chips correspondentes da barra lateral. Clique em um chip da barra lateral e todas as cinco visões se reduzem. Shift+clique em uma estrela no mini Estágio filtra o universo instantaneamente para aquele estágio do ciclo de vida. Clique no espaço vazio em qualquer mini-domo e ele "promove" para o slot principal em tamanho completo, enquanto o principal anterior desce para o slot mini desocupado.

## Por que importa?

A maioria dos apps de notas mostra o que você escreveu. Constellation Sight mostra a **forma** do que você sabe.

- Onde seu pensamento está **concentrado**? (gradiente de densidade no âncora)
- O que ainda está em **estágio inicial** vs **fundação estável**? (gradiente de cor do mini Estágio)
- Quais notas são **portantes** vs **isoladas**? (codificação de tamanho do mini Atos)
- De onde veio cada ideia — seu próprio pensamento, leitura, audição, tradição? (layout setorial do mini Procedência)
- Quão **confiante** você está em suas conclusões? (gradiente de opacidade do mini Confiança)

Uma nota no centro do âncora (alta conectividade → portante) mas em ciano no mini Estágio (`spark` — recém-iniciada) diz algo diagnóstico: uma ideia portante que ainda não amadureceu.

## Como abrir

1. Clique no **ícone de olho** na doca à esquerda do Constellation.
2. O domo de âncora renderiza em 2–5 segundos para a maioria dos universos.
3. Para fechar: clique em **(×)** no canto superior direito, ou pressione **Esc**.

## O que você vê

### A Faixa de Cabeçalho

Topo da visão Sight, da esquerda para a direita:
- **"Constellation Sight"** — o título.
- **"v6.1 — Coordinated Views (Phase 2)"** — subtítulo da versão.
- **Distintivo "X / Y notes"** dourado — visível apenas quando um filtro está ativo.
- **Distintivo "EXTENDED"** em maiúsculas pequenas douradas — visível apenas quando a visão Estendida está ativa.
- **Botão "Reset View"** — visível apenas quando o layout foi mudado.
- **(×)** botão fechar — sempre presente.

### O Domo de Âncora (Slot Principal)

Grande domo creme-em-escuro no centro:
- **Anéis de estrato** — 5 círculos concêntricos. Mais interno = suas notas mais fundamentais; borda externa = suas faíscas mais recentes.
- **Borda de calendário** — 12 rótulos de mês ao redor do exterior.
- **Rótulos de estrato** — texto em itálico no topo.
- **Estrelas** — cada nota desenhada como pequeno ponto creme, posicionada por estrato × tempo.
- **Linhas de conexão** — bordas de links tipados entre notas, baixa opacidade sob as estrelas.
- **Anel de hover** — círculo dourado ao redor de qualquer estrela sobre a qual o cursor passa.

### Os Quatro Mini-Domos

Lado direito, grade 2×2. Ocultos por padrão; revelados com **Ctrl+D** (apenas sessão) ou **Ctrl+Shift+D** (persistente — ver visão Estendida).

Cada mini renderiza o mesmo universo através de uma codificação:

1. **CONFIDENCE — opacidade.** Notas mais confiantes mais brilhantes; tentativas desvanecem.
2. **STAGE — cor (disco completo).** Cor categórica por estágio do ciclo de vida:
   - **Ciano** = `spark` (ideia recém-acendida)
   - **Laranja** = `birth` (tomando forma)
   - **Violeta** = `growth` (em movimento ativo)
   - **Verde** = `maturity` (totalmente formada)
   - **Amarelo** = `renewal` (recentemente revisitada)
   - **Cinza** = `dormancy` / `archival` (inativa / fechada)
3. **ACTS — tamanho (decil superior).** Top 10% por contagem de links = pontos maiores; restante pequeno.
4. **PROVENANCE — 5 setores.** Estrelas reposicionadas em 5 setores angulares: **Self / Read / Heard / Reasoned / Tradition**.

### O Âncora Rebaixado (Quando um Mini é Promovido)

Se promover qualquer mini para o slot principal, o âncora desce para o slot mini desocupado. Lá é renderizado como **pontos creme neutros** com o título **"UNIVERSE — primary view"**.

### A Barra Lateral de Facetas (Borda Esquerda)

Painel colapsável com **6 grupos de filtro de facetas**, cada um exibindo categorias com contagens ao vivo:

- **Folder** — hierarquia de pastas
- **Library** — nomes de bibliotecas
- **Stratum** — Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing
- **Confidence** — Hypothesis / Evidence / Established / Contested
- **Stage** — Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival
- **Provenance** — Self / Read / Heard / Reasoned / Tradition

Clique em **▶** na borda para expandir. Clique em qualquer chip para alternar como filtro.

## Interação

| Gesto | Efeito |
|---|---|
| **Hover sobre estrela** | Anel dourado na mesma estrela em todas as 5 superfícies + chips correspondentes tom dourado. |
| **Clique simples em estrela** | Abre a nota no editor. Botão **"Return to Sight"** aparece. |
| **Shift+clique em estrela** em mini Stage / Confidence / Provenance | Alterna filtro na categoria daquela estrela. |
| **Shift+clique em estrela** em Acts ou âncora | Sem efeito. |
| **Clique em área vazia de um mini** | Aquele mini promove para o slot principal. |
| **Zoom de roda (principal)** | Zoom em direção ao cursor. Faixa: 0,5× a 24×. |
| **Clique+arrastar área vazia** | Pan da visão. |
| **Ctrl+0 / Cmd+0** | Reseta zoom + pan. |
| **Ctrl+D / Cmd+D** | Alterna visibilidade dos mini-domos — **apenas sessão**. |
| **Ctrl+Shift+D / Cmd+Shift+D** | Alterna **visão Estendida** — persistente. |
| **Clique em chip da barra lateral** | Alterna categoria de faceta no conjunto de filtros. |
| **Botão Reset View** | Retorna ao âncora principal no zoom 1.0. |
| **Esc** | Fecha Sight. |

## Modo Fantasma — Seleção Múltipla do Domo

Quando um filtro está ativo, estrelas não-correspondentes permanecem visíveis mas em **baixa opacidade (15%)** em vez de desaparecer. Isto significa:

- Você ainda pode VER onde as estrelas não-correspondentes estão.
- Pode passar o cursor sobre elas (anel dourado aparece).
- Pode **Shift+clique para ADICIONAR a categoria delas ao filtro**.

## Modo Densidade

Quando a contagem de estrelas visíveis (correspondentes) excede o limite de densidade (padrão **5.000**), os mini-domos mudam para um **renderizado de densidade perceptual**.

## Visão Estendida

Pressionar **Ctrl+Shift+D** (ou **Cmd+Shift+D** no Mac) alterna a "visão Estendida" — quando ativa, os mini-domos são visíveis por padrão toda vez que você abre Sight. O estado persiste através de fechamentos do Sight, reinicializações do app e reboots.

## Quando Sight é Mais Útil

- **Auditar a forma do seu conhecimento** — abrir Sight após uma sessão de escrita.
- **Encontrar pontos cegos** — setores do domo com poucas notas podem ser áreas para explorar.
- **Detectar fraqueza portante** — nota posicionada centralmente em cor de estágio inicial.
- **Filtrar e inspecionar** — Shift+clique reduz o universo; promova um mini para estudar um canal em tamanho completo.
- **Rastrear procedência epistêmica** — promova Provenance para ver como seu conhecimento se origina.

## Superfícies Relacionadas

- **Constellation Nervous System (CNS)** — visualização complementar (ícone de neurônio ao lado do olho do Sight).
- **Constellation Map** — visualização de raios solares.
- **Sky View** — visualização de links baseada em grafo.
- **Painel Index** — navegador de termos.

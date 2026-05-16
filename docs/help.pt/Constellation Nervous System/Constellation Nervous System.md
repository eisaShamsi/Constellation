---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: pt
source: docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md
aliases:
  - Constellation Nervous System
  - CNS
  - Sistema Nervoso Constellation
description: Constellation Nervous System (CNS) é a visão de travessia-de-conexões do seu universo. Analisa o grafo de links entre suas notas e traz à tona métricas de Saúde do Universo, comunidades, pontes principais entre clusters, e "Pontos Cegos" de lacunas estruturais. CNS é a visão complementar ao Constellation Sight — se Sight é a forma sensorial do seu universo, CNS são suas conexões neurais.
---

# Constellation Nervous System (CNS)

## O que é?

**Constellation Nervous System** é a visão de **travessia-de-conexões** do seu universo. Enquanto Constellation Sight mostra a *forma* das suas notas (estrato × tempo × codificação de canal), CNS mostra a *fiação* — o grafo de links tipados que as conecta e os padrões estruturais ocultos nesse grafo.

Responde: **"Como as ideias no meu universo estão conectadas, e onde estão as lacunas?"**

A visão é construída em torno de quatro superfícies analíticas:
- **Saúde do Universo** — pontuações globais e por métrica de quão conectado, equilibrado e modular é seu conhecimento.
- **Comunidades** — grupos de notas densamente interconectadas ("clusters ideológicos").
- **Pontes Principais** — as poucas notas que ligam comunidades de outra forma separadas ("conectores portantes").
- **Pontos Cegos** — lacunas estruturais onde você esperaria conexões mas ainda não tem.

O nome "Nervous System" é anatômico: nervos são vias de conexão carregando sinais entre partes distantes de um organismo. CNS trata seu grafo de links tipados da mesma forma.

## Por que importa?

A maioria dos apps de notas trata links como encanamento (saltar daqui para lá). Constellation os trata como **arquitetura de conhecimento**:

- Uma nota com muitos links de entrada é **portante** — muitas ideias dependem dela.
- Uma nota que faz ponte entre duas comunidades é um **ponto de síntese**.
- Uma comunidade com ligação interna fraca é **frágil**.
- Um "Ponto Cego" é um lugar onde a estrutura DEVERIA ter uma conexão mas não tem — uma hipótese para explorar.

## Como abrir

1. Clique no **ícone de neurônio** (pequena forma de célula nervosa ramificada — corpo celular no meio com três ramos dendríticos e terminais sinápticos) na doca à esquerda.
2. CNS abre em sobreposição de janela completa, estilo poço de gravidade — grafo direcionado por força onde cada nota é um nó e cada link tipado uma aresta.
3. Para fechar: clique em **(×)** no topo, ou pressione **Esc**.

## O que você vê

### O Cartão de Saúde do Universo

Painel de resumo mostrando a saúde de conectividade global do seu universo, com roundel dourado de uma pontuação composta (e.g., **91 / 100**) e quatro métricas:

- **Modularity** — quão limpamente suas notas se agrupam em comunidades distintas.
- **Dominance** — se uma comunidade domina o universo.
- **Entropy** — variedade de tamanhos de comunidades.
- **Connectivity** — links médios por nota.

Cada métrica tem uma pílula de status colorida: **HEALTHY** (verde) / **CAUTION** (amarelo) / **IMBALANCED** (vermelho).

### O Poço de Gravidade

Visualização principal: notas flutuam como nós, links as puxam juntos, repulsão as separa. Comunidades se auto-organizam em clusters.

- **Tamanho do nó** = contagem de links.
- **Cor do nó** = pertencimento à comunidade.
- **Aresta** = link tipado entre duas notas.

### Pontes Principais

Lista das notas que ligam as comunidades mais distintas — esses são seus pontos de síntese.

### Comunidades

Lista de clusters de notas detectados.

### Pontos Cegos (Lacunas Estruturais)

Conexões faltantes sugeridas — pares de notas que o algoritmo do grafo pensa que DEVERIAM estar linkadas.

## Interação

CNS usa um padrão **clique-simples-pré-visualiza / duplo-clique-abre** (diferente do clique-simples-abre do Sight):

| Gesto | Efeito |
|---|---|
| **Clique simples em nó** | Seleciona-o. Painel lateral direito desliza com título, comunidade, ranque de centralidade, links de entrada/saída. A nota NÃO é aberta. |
| **Duplo clique em nó** | Abre a nota no editor. Botão **"Return to CNS"** aparece. |
| **Hover em nó** | Tooltip com título. |
| **Clique em área vazia** | Limpa a seleção. |
| **Roda do mouse** | Zoom in/out. |
| **Clique + arrastar** | Pan. |
| **Clique em comunidade na lista** | Destaca notas dessa comunidade no poço. |
| **Clique em entrada de Ponte Principal** | Foca na nota ponte. |
| **Esc** | Fecha CNS. |

O clique-simples-pré-visualiza é deliberado: permite que você escaneie detalhes de muitas notas (e suas conexões) sem se comprometer a abrir cada uma no editor.

## Quando CNS é Mais Útil

- **Auditar sua densidade de conexão** — Universe Health dá uma leitura num relance.
- **Encontrar seus pontos de síntese** — Top Bridges mostra as notas fazendo o trabalho arquitetônico.
- **Descobrir comunidades que você não sabia que existiam** — clusters emergindo do grafo.
- **Remendar Pontos Cegos** — quando o grafo sugere duas notas DEVERIAM estar linkadas mas não estão.
- **Planejar reorganização** — comunidades mapeiam naturalmente para estrutura de pastas.

## CNS vs Sight — Quando Usar Qual

- **Sight** = "Como meu universo está MOLDADO?" Análise espacial / categórica.
- **CNS** = "Como meu universo está CONECTADO?" Análise de rede / topológica.

São complementares: Sight lê a superfície; CNS lê a fiação por baixo.

## Superfícies Relacionadas

- **Constellation Sight** — a visualização irmã (ícone olho na doca).
- **Sky View** — também visão de grafo, mas construída diferentemente.
- **Painéis Backlinks / Outgoing Links** — listas de conexão por nota.

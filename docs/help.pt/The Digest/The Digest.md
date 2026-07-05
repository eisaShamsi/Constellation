---
translation_status: AI-generated 2026-05-24 — native-speaker review recommended
language: pt
source: docs/help.uConstellation.World/The Digest/The Digest.md
aliases:
  - The Digest
  - Universe Digest
  - Digest
  - Digest pane
  - Resumo do Universo
  - Resumo
  - Painel do Resumo
description: O Resumo do Universo é um painel no dock esquerdo que mostra cada nota da sua base de conhecimento no nível de manchete-de-resumo — escalonado Library → Pasta → Nota — para que você possa percorrer todo o Universo sem abrir nada. Clique em uma linha para expandi-la e ver o resumo completo em linha. O filtro restringe toda a lista; a ordenação alterna entre recência (padrão) e alfabética. Lê os mesmos resumos que você vê em todos os outros lugares; sem computação extra; sem espaço em disco extra.
---

# Resumo do Universo

> *Pense no Resumo como um sumário para a sua mente — não uma lista de arquivos, uma lista de ideias.*

O **Resumo do Universo** é o lugar para percorrer toda a sua base de conhecimento no nível do *significado*. Em vez de uma árvore de arquivos (apenas nomes) ou a Vista Estelar (apenas formas), o Resumo mostra a você, sob cada nota, **a única frase que diz do que se trata a nota**. Toque em uma linha e o resumo completo de várias frases se expande em linha. Você pode ler o conteúdo de cinquenta notas em um minuto, sem nunca abrir uma.

Ele vive no seu **dock esquerdo**, ao lado da árvore de Arquivos (o Explorador de Arquivos) e da Vista Estelar — uma das maneiras que o Constellation lhe oferece para navegar.

---

## Por que o Resumo existe

Uma árvore de arquivos lhe diz o que você *tem*. Uma busca lhe diz o que você *pediu*. O Resumo lhe diz o que você *sabe*.

Quando o seu Universo cresce além de algumas centenas de notas, "abrir cada uma para lembrar o que ela diz" se torna impossível. Você precisa de uma maneira de ler a **essência** de cada nota na velocidade do scroll — e uma maneira de expandir qualquer essência no resumo completo no momento em que você quiser pensar nele mais cuidadosamente. Esse é o Resumo.

É o terceiro pilar do Core Plug-In **Note Summary Creator (NSC)**:
- **Pilar 1**: um motor de resumos (Fase 1 / MIG-043).
- **Pilar 2**: um serviço que coloca o resumo onde quer que uma nota apareça (Fase 2 / MIG-044 — Classificador, Resultados de busca, faixa do Editor, Backlinks, Outgoing Links, o Índice, tooltip de hover da Vista Estelar).
- **Pilar 3**: esta vista — o Resumo do Universo (Fase 3 / MIG-045).

---

## Abrir o Resumo

Na **barra lateral esquerda**, clique no **ícone do Resumo do Universo** (uma pequena lista com um círculo no canto) — na fila, ao lado da árvore de Arquivos (o Explorador de Arquivos) / Vista Estelar. A barra lateral muda para o painel do Resumo.

Para voltar, clique em qualquer um dos outros ícones (ou pressione **Escape**).

---

## O que você vê

De cima para baixo:

1. **Barra de ferramentas.** Um campo de busca + um pequeno ícone de relógio (o alternador de ordenação, padrão "por recência").
2. **Cabeçalhos de Library.** Barras roxas em maiúsculas — uma por library no seu Universo. Cada uma mostra o nome da library e uma contagem de quantas notas ela contém.
3. **Cabeçalhos de pasta.** Pequenos rótulos atenuados — um por pasta *que contém notas*. Notas que vivem na raiz da library não recebem cabeçalho de pasta.
4. **Linhas de notas.** Cada linha tem:
   - Um chevron (▶) à esquerda — clique nele para expandir a linha.
   - O **nome da nota** na cor de destaque interativa — clique para **abrir a nota** no editor.
   - Uma linha tênue em itálico abaixo do nome — a **manchete do resumo** (a mesma que aparece em qualquer outra superfície de Fase 1/2).

---

## Expandir uma linha para ler o resumo completo

Clique no **chevron** (▶) à esquerda de uma linha — ou clique na **linha da manchete em itálico** propriamente. O chevron gira para ▼ e o **resumo completo de várias frases** aparece em linha abaixo da manchete, envolvendo-se naturalmente em quantas linhas precisar.

Clique no chevron (ou manchete) novamente para recolher.

A divisão "clique no chevron para expandir, clique no nome para abrir" mantém os dois gestos distintos: você pode expandir para *ler sobre* uma nota, depois continuar rolando além dela; somente quando você clica no nome a nota realmente se abre e toma o foco.

---

## Filtragem

Digite no **campo de busca** no topo. A lista se restringe enquanto você digita — apenas notas cujo **nome, manchete ou resumo completo** contenha sua consulta permanecem visíveis. Cabeçalhos de Library e cabeçalhos de pasta com zero notas correspondentes desaparecem inteiramente (sem cabeçalhos vazios).

Limpe o campo (botão × ou backspace) para restaurar a lista completa.

O filtro é **instantâneo** — o Constellation não acessa seu disco nem o banco de dados. Ele lê os resumos já em memória, então mesmo um Universo de 10.000 notas filtra na velocidade de digitação.

---

## Ordenação: recência ou alfabética

Clique no **ícone de relógio** na barra de ferramentas para alternar entre dois modos de ordenação:

- **Recência** (padrão) — dentro de cada pasta, as notas aparecem em ordem de **tempo de criação, mais recentes primeiro**. As pastas dentro de uma library são ordenadas pela nota mais recente que contêm (para que a pasta mais ativa apareça primeiro). Esse é o padrão porque revela *no que você esteve trabalhando recentemente*.
- **Alfabética** — pastas ordenadas por nome, notas dentro de cada pasta ordenadas por nome. Clique novamente para voltar à recência.

O alternador é por sessão; feche e reabra o Resumo e ele volta à recência.

---

## Federação: Universos filhos aparecem em linha

Se o seu Universo tem **Universos filhos vinculados** (cUniverses), cada library de um Universo filho aparece no Resumo como **seu próprio cabeçalho de Library par**, ao lado das libraries do Universo pai. O Resumo é uma vista unificada de tudo o que pode ser alcançado deste Universo, não apenas das libraries que vivem fisicamente aqui.

(Uma futura atualização do Constellation adicionará um alternador liga/desliga para ocultar libraries de Universos filhos temporariamente do Resumo; por enquanto elas sempre aparecem.)

---

## Como o Resumo permanece rápido em Universos enormes

O Resumo é **virtualizado**: ele renderiza apenas as linhas atualmente visíveis em sua janela de scroll, não a árvore inteira. Um Universo de 10.000 notas rola tão suavemente quanto um de 50. À medida que as linhas rolam para a vista, seus resumos são buscados em lotes do cache em memória do Constellation (o mesmo cache que alimenta todas as outras superfícies de Fase 1/2 — sem trabalho separado, sem armazenamento separado).

O Resumo nunca relê suas notas do disco. Nunca recalcula resumos. É uma vista de **leitura** sobre a mesma tabela `note_summaries` que o motor preenche desde a Fase 1.

---

## Fluxos de trabalho comuns

**"Quero ver no que trabalhei esta semana."**
Abra o Resumo com ordenação = Recência (padrão). As notas criadas mais recentemente aparecem no topo de cada library/pasta. Escaneie as manchetes.

**"Estou procurando uma nota meio lembrada sobre X."**
Abra o Resumo. Digite X (uma palavra que apareceria no título, manchete ou resumo completo da nota). A lista se restringe a candidatos. Clique nos chevrons para ler resumos completos; clique no nome para abrir o vencedor.

**"Quero escrever uma revisão de cima para baixo da minha Library."**
Abra o Resumo, ordenação = Alfabética. Percorra as manchetes em ordem. Clique nos chevrons para ler resumos mais completos quando algo te prender. Use isso como espinha dorsal de uma nova nota MOC (Map of Content).

**"Estou explorando um cUniverse federado pela primeira vez."**
Abra o Resumo. Role além das suas próprias libraries até as libraries do cUniverse — elas são linhas pares. Leia as manchetes para aprender o que o Universo vinculado contém, sem abrir nada dele.

---

## O que NÃO está no Resumo

- **Menu de contexto com clique direito** nas linhas — abrir em uma nova aba, arquivar, etc. (Para v1, as ações primárias são clique-nome-para-abrir e clique-chevron-para-expandir. Uma futura atualização adicionará um menu de contexto.)
- **Agrupamentos personalizados** — Library → Pasta é a única estratificação para v1. (Sem "agrupar por tag" ou "agrupar por estágio" ainda.)
- **Arrastar para reordenar** — o Resumo é somente leitura; a ordenação vem de regras, não de ordenação manual.
- **Controles de classificação tipo Classificador** — o Resumo é uma vista de *navegação*; a classificação vive no **Classificador** (painel separado).

---

## Tópicos relacionados

- **Resumos das Notas** — de onde vêm os resumos, a regra de precedência (o seu vence), e a lista completa de superfícies que os exibem.
- **O Classificador** — o lar de *Gerar todos os resumos* (pré-calcular cada resumo na sua Library de uma vez para que o Resumo se preencha instantaneamente).
- **Vista Estelar** — a vista da *forma* do seu conhecimento (bolhas + links); o Resumo é sua vista complementar do *significado*.
- **Formulação do Conhecimento** — por que o Constellation organiza conhecimento por *conexão* e *resumo*, não apenas por armazenamento de arquivos.

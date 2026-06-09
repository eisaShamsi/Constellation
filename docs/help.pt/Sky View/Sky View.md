---
aliases:
  - Sky View
  - Sky View
  - GraphMind
  - Sky View
  - Visão de estrelas de links
  - Visão de rede
  - Conexões de notas
  - Grafo 3D
description: Visualize e explore as conexões entre suas notas usando o Sky View interativo do Constellation, alimentado pelo motor GraphMind.
---

# Sky View

O Sky View exibe suas notas como uma rede interativa de nós e links, alimentada pelo motor **GraphMind** (Pixi.js WebGL). Cada nó é uma nota, e cada linha representa um `[[wikilink]]` entre notas. Quanto mais conexões uma nota tem, maior aparece o seu nó.

## Abrindo o Sky View

| Método | Ação |
|--------|--------|
| **Mission Control** | Pressione `Ctrl+P`, digite "star view" |
| **Teclado** | `Ctrl+G` |

Pressione `Escape` para fechar o Sky View.

> [!note]
> O ícone do Sky View na faixa foi removido do dock esquerdo. O Sky View agora é acessível pelo atalho de teclado ou pelo Mission Control. O modo Sky View (Organograma) está disponível como uma aba na barra lateral de Gestão de Notas.

---

## Interagindo com o grafo

### Interações básicas

| Entrada | Comportamento |
|-------|----------|
| **Mover (Pan)** | Clique e arraste em um espaço vazio |
| **Zoom** | Roda de rolagem (2D) ou `Ctrl+Rolagem` (3D) |
| **Arrastar nós** | Clique e arraste qualquer nó para reposicioná-lo |
| **Passar o cursor** | Mostra o nome da nota na barra de status e destaca os nós e arestas conectados |
| **Clicar em um nó** | Abre aquela nota no editor |
| **Clique duplo em um nó** | Aproxima o zoom e centraliza naquele nó |
| **Clique direito em um nó** | Abre o menu de contexto |

### Menu de contexto

Clique com o botão direito em qualquer nó para acessar:

| Ação | Descrição |
|--------|-------------|
| **Abrir** | Abre a nota no editor |
| **Focar** | Entra no modo foco centralizado neste nó |
| **Fixar** | Trava o nó na sua posição atual. Clique novamente para desafixar. |
| **Ocultar** | Oculta o nó do grafo. Use "Mostrar tudo" na barra de ferramentas para revelar os nós ocultos. |

---

## Navegação 3D

O Sky View oferece suporte completo à navegação 3D — voe pelas suas notas como se estivesse navegando entre as estrelas.

### Entrando no modo 3D

**Clique com o botão do meio e arraste** (ou **Alt+clique e arraste**) para girar o grafo no espaço 3D. Uma vez girado, os controles de navegação 3D tornam-se ativos.

### Controles 3D

| Entrada | Ação |
|-------|--------|
| **Arrastar com botão do meio** | Girar em torno dos eixos X e Y |
| **Shift+arrastar com botão do meio** | Girar em torno do eixo Z |
| **W / Seta para cima** | Voar para frente (para dentro da tela) |
| **S / Seta para baixo** | Voar para trás |
| **A / Seta para a esquerda** | Deslocar para a esquerda |
| **D / Seta para a direita** | Deslocar para a direita |
| **Q** | Mover para baixo |
| **E** | Mover para cima |
| **Ctrl+Rolagem** | Zoom (mudar o campo de visão) |
| **Rolagem normal** | Voar para frente/trás na direção da câmera |
| **0** | Redefinir a rotação de volta à visão 2D plana |
| **Botão Redefinir** (ícone ↺) | Mesmo que pressionar `0` |

### Guia de eixos XYZ (gizmo)

No modo 3D, um guia de eixos com código de cores aparece no canto inferior esquerdo:

| Eixo | Cor | Direção |
|------|-------|-----------|
| **X** | Vermelho | Esquerda–Direita |
| **Y** | Verde | Cima–Baixo |
| **Z** | Azul | Frente–Trás (profundidade) |

O gizmo gira junto com a câmera para que você sempre saiba a sua orientação.

### Passar o cursor e clicar em 3D

Você pode passar o cursor e clicar nos nós enquanto navega em 3D. O nome da nota aparece na barra de status, e clicar abre a nota — exatamente como no modo 2D.

---

## Modos de layout

O Sky View oferece três algoritmos de layout. Alterne entre eles pressionando `Ctrl+L` ou usando o botão de layout na barra de ferramentas.

| Modo | Descrição | Melhor para |
|------|-------------|----------|
| **Orgânico** | Layout dirigido por forças. Os clusters surgem naturalmente da densidade de links. | Exploração geral — o modo padrão. |
| **Hierárquico** | Grafo acíclico dirigido (DAG) de cima para baixo. | Bibliotecas estruturadas com relações pai–filho. |
| **Temporal** | Nós dispostos ao longo de um eixo de tempo horizontal pela data de criação. | Ver quando as notas foram criadas e como a biblioteca cresceu. |

Alternar os modos dispara uma transição animada suave que preserva sua orientação espacial.

> [!tip]
> O modo Hierárquico é especialmente útil para notas que seguem uma estrutura em árvore (por exemplo, MOCs que ligam a subtópicos). O modo Temporal revela sua linha do tempo intelectual — quando clusters de notas relacionadas foram criados.

---

## Modo foco

O modo foco mostra apenas uma nota específica e sua vizinhança. É um grafo local dinâmico e interativo.

### Entrando no modo foco

- **Clique com o botão direito em um nó** → **Focar**
- **Pressione Espaço** para alternar o modo foco na nota ativa no momento

### Controles do foco

No modo foco, uma barra de controle aparece no topo:

| Controle | Descrição |
|---------|-------------|
| **Controle deslizante de profundidade** (1–5) | Quantos saltos de conexões mostrar. 1 = apenas links diretos, 5 = cinco níveis de profundidade. |
| **Filtro de direção** (↔ / ← / →) | Mostrar todos os links, apenas recebidos ou apenas enviados. |
| **Botão de saída** (×) | Voltar ao Sky View completo |

### Trilha de navegação (breadcrumb)

Conforme você clica pelos nós no modo foco, uma trilha de navegação aparece no topo mostrando seu caminho de navegação. Clique em qualquer item da trilha para voltar ao grafo local daquela nota.

> [!tip]
> Combine o modo foco com o controle deslizante de profundidade para explorar progressivamente a vizinhança de uma nota. Comece na profundidade 1 para ver as conexões diretas, depois aumente para descobrir relações de segundo e terceiro grau.

---

## Pesquisar-para-destacar

Pressione `Ctrl+F` para abrir a barra de pesquisa. Digite uma consulta para destacar as notas correspondentes.

Ao contrário de um filtro, a pesquisa-para-destacar **atenua** os nós não correspondentes sem removê-los. Você mantém toda a estrutura do grafo e o contexto espacial enquanto os nós correspondentes são destacados.

> [!tip]
> A pesquisa funciona tanto no grafo completo quanto no modo foco. Você também pode pesquisar enquanto está no modo 3D.

---

## Painel de configurações

Clique no ícone de engrenagem (⚙) na barra de ferramentas para abrir o painel de configurações. Ele tem três abas:

### Aparência do grafo

| Controle | Descrição | Padrão |
|---------|-------------|---------|
| **Tamanho do nó** | Escalar todos os nós para maiores ou menores | 1.5 |
| **Visibilidade dos rótulos** | Quando os rótulos aparecem: Ao passar o cursor, Sempre ou Nunca | Ao passar o cursor |
| **Tamanho da fonte do rótulo** | Tamanho dos rótulos com o nome das notas | 12 |
| **Espessura do link** | Largura das linhas das arestas | 1 |
| **Mostrar notas órfãs** | Incluir notas sem links | Ativado |

> **Cor de fundo da tela.** A cor por trás das bolhas é definida em **Configurações → Designer de estilo → Sky View → Tela → Fundo** (não neste painel). É independente das suas barras laterais/painéis, então você pode dar ao grafo o seu próprio plano de fundo — uma cor profunda para fazer as bolhas se destacarem, por exemplo — sem alterar o resto da interface. Se não for definida, a tela acompanha a superfície do painel. Veja *Aparência e Temas → Tela do Sky View*.

### Física

| Controle | Descrição | Padrão |
|---------|-------------|---------|
| **Repulsão** | Com que força os nós se afastam | 50 |
| **Força do link** | Com que força os nós ligados se atraem | 0.05 |
| **Distância do link** | Distância alvo entre nós ligados | 30 |
| **Reaquecer a simulação** | Reiniciar o layout de forças a partir do estado atual | — |

### IA

Configurações para links semânticos de IA (Fase 2 — requer modelo de embedding local).

| Controle | Descrição |
|---------|-------------|
| **Mostrar links semânticos** | Alternar as arestas tracejadas detectadas pela IA |
| **Limite de confiança** | Controle deslizante para filtrar os links semânticos pela pontuação de similaridade |

---

## Legenda

A legenda aparece no canto inferior direito e mostra as atribuições de cores para suas bibliotecas.

### Alternância do modo de cor

Clique nos botões **Biblioteca** ou **Pasta** no topo da legenda para alternar como os nós são coloridos:

| Modo | Coloração |
|------|----------|
| **Biblioteca** | Cada biblioteca recebe uma cor única |
| **Pasta** | Cada pasta de nível superior recebe uma cor única |

### Caixas de seleção de visibilidade

Cada entrada da legenda tem uma caixa de seleção. Desmarque uma biblioteca ou pasta para ocultar seus nós do grafo. Isso permite focar em subconjuntos específicos da sua base de conhecimento.

> [!tip]
> No modo Pasta, a contagem de pastas é exibida entre parênteses. Listas longas de pastas podem ser roladas.

---

## Barra de status

A barra de status no canto inferior esquerdo mostra:

- **Contagem de nós** — total de nós visíveis
- **Contagem de arestas** — total de arestas visíveis
- **Contagem de MOCs** — número de Mapas de Conteúdo (notas-hub de alta conectividade)
- **Nome da nota sob o cursor** — aparece quando você passa o cursor sobre um nó

---

## Atalhos de teclado

| Atalho | Ação |
|----------|--------|
| `Ctrl+G` | Abrir o Sky View |
| `Escape` | Fechar o Sky View |
| `Ctrl+F` | Alternar pesquisar-para-destacar |
| `Ctrl+L` | Alternar entre os modos de layout (Orgânico → Hierárquico → Temporal) |
| `Space` | Alternar o modo foco na nota ativa |
| `0` | Redefinir a rotação 3D para a 2D plana |
| `W/A/S/D` | Voar pelo espaço 3D (quando girado) |
| `Q/E` | Mover para baixo/cima no espaço 3D |

---

## Suporte a RTL

O Sky View oferece suporte de primeira classe para árabe, hebraico e outros scripts RTL (da direita para a esquerda):

- **Rótulos dos nós** detectam automaticamente a direção do script — títulos em árabe são renderizados da direita para a esquerda
- **Itens da legenda** invertem a ordem do ponto/texto com base no idioma do conteúdo
- **Dicas e painéis** respeitam o layout RTL
- **Fonte árabe de reserva** — os rótulos usam fontes árabes do sistema (Noto Naskh Arabic, Segoe UI) quando a fonte principal não tem cobertura de glifos árabes

---

## Sobreposição Picture-in-Picture (PiP)

Quando o Sky View está aberto e você clica em um universo filho, biblioteca ou pasta na barra lateral de Gestão de Notas, uma janela **Picture-in-Picture (PiP)** aparece como uma sobreposição redimensionável sobre o grafo principal.

### O que o PiP mostra

O PiP exibe um subgrafo filtrado contendo apenas os nós que pertencem ao escopo selecionado. Por exemplo, clicar em uma biblioteca mostra apenas as notas dessa biblioteca e suas interconexões.

### Recursos do PiP

| Recurso | Descrição |
|---------|-------------|
| **Grafo filtrado** | Apenas os nós do escopo selecionado aparecem |
| **Legenda filtrada** | O PiP tem sua própria legenda mostrando apenas as entradas relevantes |
| **Redimensionável** | Arraste as bordas ou os cantos para redimensionar a janela do PiP |
| **Reposicionável** | Arraste a barra de título para mover o PiP para qualquer lugar da tela |

### Sincronização de seleção entre modos

Clicar em um universo filho, biblioteca, pasta ou nota em qualquer modo da barra lateral (Árvore, Lista ou Organograma) destaca os nós correspondentes no grafo do Sky View. Essa sincronização bidirecional ajuda você a manter a consciência espacial enquanto navega na barra lateral.

---

## Estratos de Conhecimento

O Sky View dimensiona automaticamente os nós com base no seu nível de conhecimento (1-8):

- Pontos pequenos: notas simples (Dado, Informação)
- Nós médios: notas conectadas (Proposição, Conceito)
- Hubs grandes e brilhantes: notas de síntese (Teoria, Paradigma, Visão de mundo)

Os nós de nível mais alto têm um halo de brilho em cor complementar para contraste visual. Isso é ativado quando uma biblioteca tem mais de 20 notas.

---

## Maturidade da Nota

Os nós exibem um anel colorido indicando a maturidade:

- Sem anel: Semente (nota nova)
- Anel verde-claro: Broto (em crescimento)
- Anel verde-rico: Perene (bem estabelecida)
- Anel dourado: Canônica (referência autoritativa)

A maturidade também é mostrada na árvore de arquivos (borda esquerda) e na barra de abas (ponto colorido).

---

## Brilho de Proveniência

Os nós no Sky View mostram um sutil brilho de cor indicando a origem do conhecimento:

- **Brilho azul**: Conhecimento recebido — a cadeia de fontes da nota remonta a uma referência externa (uma nota com url, autor ou doi no seu frontmatter)
- **Brilho âmbar**: Conhecimento descoberto — a cadeia de fontes da nota origina-se das próprias notas do usuário

---

## Notas técnicas

O Sky View é alimentado pelo motor **GraphMind**, um renderizador Pixi.js WebGL com uma simulação d3-force executada em um Web Worker dedicado. Essa arquitetura garante:

- **Renderização a 60fps** mesmo com milhares de nós
- **Layout não bloqueante** — a simulação de forças nunca congela a interface
- **Passar o cursor é apenas visual** — passar o cursor nunca dispara o recálculo da física
- **A simulação para após se estabilizar** — uma vez que os nós encontram suas posições, o motor de física para completamente. Apenas arrastar um nó ou mudar as configurações o reinicia.

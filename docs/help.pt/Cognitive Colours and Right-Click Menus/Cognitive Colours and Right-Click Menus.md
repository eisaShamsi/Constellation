---
aliases:
  - Cores cognitivas
  - Estilização das tags de propriedade
  - Estilização das pílulas de taxonomia
  - Cores de maturidade
  - Cores de confiança
  - Cores de origem
  - Cores de estágio
  - Cores da categoria de correspondência
  - Menu de clique direito
  - Menu de contexto
  - Clique direito no corpo da nota
  - Clique direito na propriedade
  - Clique direito no resultado de busca
  - Unificar sob demanda
description: Reestilize as tags de propriedade do frontmatter e as pílulas de taxonomia, defina uma cor compartilhada para cada estado cognitivo (Maturidade, Confiança, Origem, Estágio, Categoria de correspondência) para que todas as superfícies se unifiquem sob demanda, e use os menus de clique direito de todo o aplicativo no corpo da nota, no painel de Propriedades e nos resultados de busca.
---

# Cores Cognitivas e Menus de Clique Direito

Este tópico cobre duas coisas que chegaram juntas: **duas novas categorias do Designer de estilo** — **Propriedades** (reestilizar as pequenas tags no seu frontmatter) e **Cores cognitivas** (um controle de cor por estado cognitivo, compartilhado por todo o aplicativo) — e os **menus de clique direito de todo o aplicativo** que deixam as ações certas a um clique de distância no corpo da nota, em uma propriedade do frontmatter e em um resultado de busca.

> O Designer de estilo é o estúdio de design em tela cheia que você abre em **Configurações → Aparência → "✦ Abrir Designer de estilo,"** ou a partir da sua própria aba **Designer de estilo** na barra lateral de Configurações. As duas categorias abaixo ficam na lista à esquerda de *Superfícies* junto a Interface, Editor, Links e o resto. Para o comportamento geral do Designer — Inspecionar, Manter / Descartar / Redefinir, Estilos salvos — veja [[Appearance and Themes]].

---

## Designer de estilo → Propriedades

A categoria **Propriedades** reestiliza as pequenas tags que aparecem dentro do **frontmatter** de uma nota (o seu bloco de propriedades YAML) — as pílulas que você vê para `tags`, `aliases` e afins no painel de Propriedades e no topo da nota. Até agora elas eram fixas; agora são suas para moldar.

Abra o **Designer de estilo** e clique em **Propriedades** na lista à esquerda. O centro mostra uma pré-visualização ao vivo das pílulas de propriedade; clique em um controle à direita e a pré-visualização se reestiliza enquanto você edita. Dois elementos:

### Tags de propriedade

As pílulas comuns de tag do frontmatter (por exemplo, cada valor em uma lista `tags`). Quatro controles:

- **Fundo da tag** — a cor de preenchimento da pílula.
- **Texto da tag** — a cor do texto dentro da pílula.
- **Raio da tag** — quão arredondados são os cantos da pílula (0 px = quadrado, até 20 px = totalmente arredondado).
- **Altura** — a altura da pílula em pixels (14–32 px).

### Pílulas de taxonomia

As pílulas usadas para valores no estilo de taxonomia. Três controles:

- **Fundo** — a cor de preenchimento da pílula.
- **Texto** — a cor do texto dentro da pílula.
- **Raio** — arredondamento dos cantos (0–20 px).

> **Nada muda até você tocar em um controle.** Cada valor começa exatamente com a aparência que você tem hoje, então a categoria Propriedades deixa as suas notas com aparência idêntica até que você escolha deliberadamente uma cor ou arraste um deslizador. Clique em **Manter** para salvar a aparência neste Universo.

---

## Designer de estilo → Cores cognitivas

O Constellation pinta o seu **vocabulário cognitivo** em cores — a *maturidade* de uma nota, a *confiança* de um link, de onde uma ideia *veio*, em que *estágio* da vida ela está e *por que* um resultado de busca correspondeu. O problema era que cada uma dessas cores era decidida separadamente em cada superfície: uma nota "murcha" podia ser de um verde na árvore de arquivos e de um verde diferente na Vista do Céu. A categoria **Cores cognitivas** oferece **um controle de cor por estado**, e tudo que mostra aquele estado o segue.

Abra o **Designer de estilo** e clique em **Cores cognitivas** na lista à esquerda. O centro mostra uma legenda de cores do conjunto que você estiver editando; escolha um controle à direita e a legenda se atualiza ao vivo. São cinco conjuntos.

### Maturidade — quão assentada está uma ideia

Cinco estados, do mais jovem ao mais assentado: **Semente**, **Broto**, **Perene**, **Canônica**, **Murcha**. Cada um recebe uma cor, usada pelos pontos de nota na árvore de arquivos, pelo marcador de maturidade na aba e pelo inspetor da nota.

### Confiança — quão certo é um link

Quatro estados: **Hipótese**, **Evidência**, **Estabelecido**, **Contestado**. Uma cor para cada.

### Origem — de onde veio uma ideia

Quatro estados: **Recebido** (retirado de uma fonte), **Descoberto** (seu próprio), **Misto** e **Nenhum**. Uma cor para cada.

### Estágio — onde a nota se situa na sua vida

Seis estados, em ordem: **Faísca**, **Nascimento**, **Crescimento**, **Maturidade**, **Dormência**, **Arquivamento**. Uma cor para cada.

### Categoria de correspondência — por que um resultado de busca correspondeu

Sete tipos de correspondência: **Título**, **Conteúdo**, **Tag**, **Wikilink**, **Propriedade**, **Semântica** (uma correspondência por significado, não por palavra exata) e **Estruturado** (uma correspondência por consulta de propriedade). A cor que você define aqui é compartilhada pelo destaque de busca dentro do editor, pelo selo de correspondência e pelo destaque na linha do resultado no painel de busca.

### "Unificar sob demanda" — a regra que torna isso seguro

As cores cognitivas seguem uma regra deliberada: **nada muda até você escolher uma cor.** Cada superfície mantém a cor que tem hoje como seu próprio padrão de reserva. No momento em que você define a cor de um estado aqui, **todas** as superfícies que mostram aquele estado passam para a sua cor de uma só vez — árvore de arquivos, abas, o inspetor, destaques de busca e assim por diante. Defina "Perene" uma vez, e cada marcador Perene em todo o aplicativo concorda. Deixe um estado intocado e ele fica exatamente como estava antes.

É por isso que a categoria pode ser lançada sem alterar uma única aparência existente: ela unifica *sob demanda*, nunca por padrão. Clique em **Manter** para salvar as suas cores neste Universo.

---

## Menus de clique direito por todo o aplicativo

O Constellation agora oferece um menu completo de clique direito (menu de contexto) nos três lugares onde você mais costuma querer um: o **corpo da nota**, uma **propriedade do frontmatter** e um **resultado de busca**. Cada menu oferece apenas ações que façam sentido onde você clicou.

### Clique direito no corpo da nota

Clique com o botão direito em qualquer lugar do texto de uma nota para obter o menu de edição:

- **Adicionar link** / **Adicionar link externo** — envolve a seleção (ou insere no cursor) como um `[[wikilink]]` ou um link `[texto](url)`.
- **Formatar ▸** — um submenu suspenso: Negrito, Itálico, Sublinhado, Tachado, Realce, Código em linha, Matemática, Alternar comentário, Sobrescrito, Subscrito, Limpar formatação.
- **Parágrafo ▸** — um suspenso: Lista com marcadores, Lista numerada, Lista de tarefas, os níveis de título **H1–H6** e **Corpo**, e Citação em bloco.
- **Inserir ▸** — um suspenso: Nota de rodapé, Tabela, Callout, Régua horizontal, Bloco de código, Bloco de matemática, Imagem.
- **Área de transferência** — Recortar, Copiar, Colar, Colar como texto simples, Selecionar tudo.
- **Estilo…** — salta direto para o **Designer de estilo** focado na categoria **Editor**, para que você possa reestilizar exatamente aquilo em que clicou com o botão direito.

### Clique direito em uma propriedade do frontmatter

Clique com o botão direito em uma **linha** de propriedade no painel de Propriedades (ou no bloco de propriedades no topo da nota) e você obtém ações de propriedade além do menu de edição completo:

- **Copiar valor** — copia o valor da propriedade para a área de transferência.
- **Copiar nome** — copia a chave da propriedade.
- **Remover propriedade** — exclui aquela linha de propriedade.
- **Adicionar propriedade** — adiciona uma nova linha de propriedade vazia.
- …seguidos pelos mesmos itens **Formatar / Parágrafo / Inserir / área de transferência** do corpo da nota, e um item **Estilo…** que abre o Designer de estilo focado na categoria **Propriedades** — de modo que "Estilo…" em uma tag de propriedade estiliza tags de propriedade, não o corpo da nota.

### Clique direito em um resultado de busca

Clique com o botão direito em um resultado no painel de busca para um conjunto **seguro** de ações de nota — aquelas que nunca colocam os seus arquivos em risco:

- **Abrir** — abre a nota.
- **Abrir em nova aba** — abre-a ao lado do que você já tem.
- **Revelar na árvore** — destaca a nota na árvore de arquivos para você ver onde ela vive.
- **Copiar link** / **Copiar caminho** — copia um wikilink para a nota, ou o seu caminho de arquivo.
- **Marcar** — adiciona a nota aos seus marcadores.
- **Mostrar no explorador** — revela o arquivo no gerenciador de arquivos do seu sistema operacional.
- **Abrir no aplicativo padrão** — abre o arquivo no aplicativo que o seu sistema usa para Markdown.
- **Estilo…** — abre o Designer de estilo focado na categoria **Cores cognitivas** (onde vivem as cores de correspondência de busca).

> **Por design, o menu de resultado de busca não tem Renomear, Mover ou Excluir.** Um painel de busca mostra resultados de todo o seu Universo e não mantém a sua própria cópia atualizada ao segundo da árvore de arquivos, então uma ação destrutiva ali poderia agir sobre uma visão desatualizada. O Constellation mantém essas operações na árvore de arquivos (e no Navegador de Notas), onde a visão está sempre atual. O menu de busca serve para *chegar até* uma nota com segurança, não para reestruturar a sua biblioteca.

---

## Bom saber

- **Local e privado.** Tudo isso é calculado a partir das suas próprias notas e configurações no seu dispositivo. Nada é enviado a lugar nenhum.
- **Fala o seu idioma.** Cada item de menu, cada nome de categoria, cada rótulo de estado aparece no idioma de interface escolhido e espelha corretamente para idiomas da direita para a esquerda. As próprias cores dos estados cognitivos são universais — uma cor significa o mesmo estado em todos os idiomas.
- **"Estilo…" sempre cai na superfície certa.** Cada entrada "Estilo…" abre o Designer de estilo focado na categoria do que você clicou com o botão direito: o corpo da nota → **Editor**, uma propriedade → **Propriedades**, um resultado de busca → **Cores cognitivas**. Você nunca precisa caçar os controles certos.

---

## Relacionado

- [[Appearance and Themes]] — o comportamento geral do Designer de estilo, temas, fontes e Estilos salvos
- [[Properties]] — visualizar e editar as propriedades do frontmatter cujas tags você reestiliza aqui
- [[Search]] — o painel de busca cujos resultados carregam o menu de clique direito
- [[Cognitive Engine]] — o que Maturidade, Confiança, Origem e Estágio significam como medidas de conhecimento
- [[Knowledge Formulation]] — os níveis de confiança do link vivo que as cores de Confiança representam

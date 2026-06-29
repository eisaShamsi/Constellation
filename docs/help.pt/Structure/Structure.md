# Estrutura

*(A espinha compositiva — onde esta nota se situa na obra inteira)*

O Constellation já lhe oferece oito **links de pensamento** — *apoia, contradiz, causa, exemplifica, generaliza, deriva-de, parte-de, substitui* — o vocabulário que usa para relacionar uma ideia com outra. Os **links estruturais** são um tipo deliberadamente diferente. Não relacionam ideia com ideia; dispõem a **forma ordenada de uma obra** que está a construir a partir das suas notas: Livro → Parte → Capítulo → Cena, ou qualquer esquema de Mapa de Conteúdo. O painel **Estrutura** é onde você lê essa forma.

A única pergunta que a Estrutura responde é: **"Onde é que esta nota se situa na obra inteira?"** — *não* "como é que esta ideia se relaciona com aquela." Essa segunda pergunta pertence aos painéis Backlinks e Links de Saída, e a Estrutura mantém-se fora do caminho deles.

---

## Por que os links estruturais são mantidos separados do seu pensamento

Uma colocação estrutural é **autoria, não uma afirmação a ser julgada**. Colocar uma cena sob um capítulo, ou um capítulo sob um livro, é uma decisão sobre a *forma do seu manuscrito* — não é evidência, não é um argumento, não é algo que possa ser contradito ou tornar-se mais certo com o tempo.

Por isso os links estruturais são deliberadamente invisíveis a toda medida de pensamento, maturidade e conexão:

- **Não** contam como conexões nos backlinks ou links de saída de uma nota.
- **Não** elevam a maturidade de uma nota.
- **Não** aparecem na Vista Estelar nem no grafo.

Um índice não deveria fazer uma nota parecer mais "conectada" do que é. Os seus links de pensamento e o esquema do seu manuscrito são duas coisas separadas, e o Constellation mantém-nas assim.

---

## Os dois tipos — você só digita um dos lados

Você declara a estrutura a partir de qualquer extremidade que seja conveniente, e o Constellation descobre o inverso por você. Nunca tem de manter ambas as extremidades.

| Propriedade | O que significa |
|---|---|
| **`parent`** | O lugar *desta nota* sob um único pai. (Um capítulo diz a que parte pertence.) |
| **`contains`** | A lista ordenada de filhos *desta nota*. (Um livro lista as suas partes, na ordem de leitura.) |

Declarar o `parent` de um filho e listá-lo numa lista `contains` são duas formas de dizer a mesma coisa. Use a que melhor se ajustar à forma como pensa — de cima para baixo (um livro que *contém* as suas partes) ou de baixo para cima (um capítulo que nomeia o seu *pai*).

---

## Criar um link estrutural — passo a passo

Você cria a estrutura nas **Propriedades** de uma nota — a aba Propriedades na barra lateral direita, ou o bloco de propriedades no topo da nota.

1. Clique em **+ Adicionar propriedade**.
2. Para a chave, digite **`parent`** ou **`contains`**.
3. No valor, digite o **nome da nota de destino** — apenas o nome, por exemplo `Part I - The Cartographer`. **Você não digita os colchetes.** O Constellation envolve o nome num `[[link]]` por você automaticamente. (Se colar um nome que já tenha colchetes, ele é limpo para um único `[[name]]` — nunca um duplo `[[[ ]]]`.)
4. Para **`contains`**, adicione cada filho como o seu próprio chip — digite um nome, prima Enter, digite o seguinte. **A ordem em que os adiciona é a ordem de leitura** do esquema.

> **Renomeiam-se com segurança.** Renomeie um capítulo e o seu lugar na estrutura acompanha automaticamente — o link aponta para a própria nota, não para um trecho de texto congelado. Nunca tem de caçar e corrigir um esquema depois de renomear.

---

## Ler o painel Estrutura

Abra a aba **Estrutura** na barra lateral direita — logo após a aba Backlinks.

- **O esquema.** Encabeçado por **OUTLINE** com uma contagem, o painel mostra a **obra inteira** como uma árvore indentada com marcadores em verde-azulado — todos os descendentes da obra, em ordem — não apenas os filhos da própria nota aberta. Assim, mesmo quando está sobre uma única cena, vê o livro inteiro à sua volta.
- **"Você está aqui."** A nota que está a visualizar atualmente é **realçada** dentro do esquema, para que saiba sempre onde se encontra.
- **O breadcrumb.** Ao longo do topo, um breadcrumb em verde-azulado mostra o caminho ao longo da espinha — por exemplo *The Atlas of Lost Places › Part I › Chapter 1*. Clique em qualquer migalha (ou em qualquer linha do esquema) para saltar diretamente para essa nota.
- **Whole work ⇄ This note.** Um alternador no canto superior direito alterna entre a obra inteira e apenas o ramo da própria nota aberta. Só aparece quando a nota tem um pai (caso contrário, as duas vistas seriam idênticas).

> **Um ciclo nunca o trava.** Se a estrutura acidentalmente formar um círculo sobre si mesma — o pai da nota A é B, e o pai de B é A — o esquema desenha a cadeia e depois para de forma limpa, marcando o ponto de corte com um pequeno **↻**. Passe o rato por cima para uma explicação de uma linha.

---

## Quando duas notas reivindicam o mesmo filho — "Contested"

A estrutura deve ser uma árvore limpa, por isso um filho deve ter exatamente um pai. Se duas notas reivindicarem ambas o mesmo filho — uma através do próprio **`parent`** do filho, a outra através da sua lista **`contains`** — o Constellation **não** escolhe silenciosamente uma e descarta a outra. Em vez disso, essa linha é sinalizada como **Contested** com um selo âmbar **⚠** nomeando o outro reivindicante, para que possa ver o conflito e decidir.

Dois botões de um clique resolvem-no:

- **Keep** — manter o pai declarado do próprio filho. (Esta nota abdica da sua reivindicação sobre o filho.)
- **Move here** — aceitar esta nota como o pai. (O `parent` do filho passa para esta nota.)

Qualquer das escolhas atualiza os ficheiros das notas diretamente e atualiza o esquema. **Nada é jamais alterado sem o seu clique** — o Constellation sinaliza o conflito e aguarda pela sua decisão.

---

## Bom saber

- **Local e privado.** O esquema é lido das suas próprias notas a pedido; nada é enviado para lugar nenhum.
- **Rápido em obras grandes.** Esquemas longos (acima de cerca de 50 linhas) ganham a sua própria barra de rolagem e renderizam apenas as linhas no ecrã, por isso um manuscrito grande abre e rola suavemente.
- **Fala a sua língua.** As etiquetas do painel, o breadcrumb e os botões de resolução aparecem todos no idioma de interface que escolheu e refletem-se corretamente para idiomas da direita para a esquerda. As *chaves* de propriedade `parent` / `contains` permanecem em inglês canónico no ficheiro (para que a estrutura se leia da mesma forma em todos os idiomas), enquanto as suas etiquetas de pílula no ecrã são localizadas.

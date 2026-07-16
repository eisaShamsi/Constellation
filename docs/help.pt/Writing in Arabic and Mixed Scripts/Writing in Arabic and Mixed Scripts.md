# Escrever em árabe e em escritas mistas

O editor do Constellation foi construído com o idioma em primeiro lugar: árabe, hebraico, persa, urdu e notas bilíngues não são um acréscimo — o cursor, a seleção e a direção de cada parágrafo seguem as mesmas regras que o Microsoft Word usa no Windows, então a sua memória muscular funciona desde o primeiro momento. Este tópico cobre tudo sobre *escrever* em texto da direita para a esquerda e em texto misto: como o cursor se move, como selecionar por palavra, frase, linha, parágrafo ou tela, e como forçar a direção de um parágrafo quando a detecção automática não é o que você quer.

(Para saber como o Constellation *entende* o árabe — raízes, busca e o motor morfológico — veja o tópico **Motor árabe**.)

---

## Como o cursor se move

- **As setas movem o cursor um caractere do texto de cada vez, na ordem de leitura** — nunca uma posição na tela. Em árabe puro ou inglês puro, isso se parece exatamente com a seta que você pressionou. Numa costura entre árabe e inglês (uma frase em árabe contendo uma palavra em inglês, por exemplo), o cursor percorre cada caractere na ordem de escrita e visivelmente "salta" através da costura — esse salto é correto; é ele que impede o cursor de parecer preso na fronteira.
- **Home** vai para o **início** de leitura da linha — a borda *direita* de uma linha em árabe. **End** vai para o **fim** de leitura — a borda *esquerda*. Segure **Shift** com qualquer um deles para selecionar até essa borda.
- **Enter** numa linha em árabe coloca o cursor da nova linha à **direita** — a posição natural de escrita.
- Uma **palavra em alfabeto latino no fim de uma linha em árabe** mantém um cursor nítido e estável, em vez de perder a direção.

Todas as regras acima funcionam de forma idêntica no editor padrão, no modo Foco e na vista de mesclagem de conflitos.

---

## Selecionar por unidade

Cada unidade de texto tem um seletor rápido, em qualquer idioma e qualquer mistura:

| Unidade | Como |
|---|---|
| **Palavra** | Clique duplo nela |
| **Frase** | **Ctrl+clique** em qualquer ponto dela — ou pressione **Ctrl+Shift+S** com o cursor dentro dela |
| **Linha** | **Ctrl+L** |
| **Parágrafo** | **Ctrl+Shift+L** — ou clique triplo nele |
| **Uma tela inteira** | **Shift+Page Down** / **Shift+Page Up** |
| **Tudo** | **Ctrl+A** |

Detalhes que vale a pena conhecer:

- **A seleção de frases entende a pontuação árabe.** Ela encerra uma frase em **؟ ۔ !** e no ponto final — mas o ponto e vírgula árabe **؛** é uma pausa *dentro* de uma frase, então a seleção corretamente continua além dele. Números decimais como 3.14 nunca dividem uma frase.
- Um **parágrafo** é um bloco de texto com uma linha vazia acima e outra abaixo — exatamente como no Word. As seleções de linha e de parágrafo abraçam o texto: numa linha em árabe, o destaque para nas palavras em vez de se estender pelo lado esquerdo vazio.
- O Ctrl+clique *substitui* o antigo gesto de "adicionar outro cursor" nessa tecla — agora, o clique faz a seleção de frase.

## Mover por parágrafo

- **Ctrl+↓** salta para o início do **próximo** parágrafo; **Ctrl+↑** para o início do parágrafo **atual** (pressione de novo para ir ao anterior). Adicione **Shift** para selecionar parágrafo por parágrafo enquanto salta. Esta é a convenção do Word, e "próximo" significa simplesmente mais abaixo na página — funciona de forma idêntica em notas em árabe, em inglês e mistas.

---

## Forçar a direção de um parágrafo

O Constellation detecta a direção de cada linha automaticamente a partir das suas primeiras letras. Em geral, isso acerta — mas às vezes você quer passar por cima da detecção: um parágrafo em árabe que começa com um nome de marca em inglês, ou um parágrafo majoritariamente em inglês que você quer ler da direita para a esquerda.

**Pressione e solte Ctrl+Shift do lado DIREITO do teclado** → o parágrafo em que o cursor está se torna **100% da direita para a esquerda**.
**Pressione e solte Ctrl+Shift do lado ESQUERDO** → **100% da esquerda para a direita**.

Esta é a convenção do Microsoft Word. O que você precisa saber:

- **O comando dispara ao soltar as teclas** — pressione as duas teclas juntas, solte, e não pressione mais nada no meio. É por isso que Ctrl+Shift+S, Ctrl+Shift+L e todos os outros atalhos continuam funcionando normalmente: no momento em que uma terceira tecla entra, a troca de direção é cancelada.
- **É uma imposição rígida** — vence a detecção automática e se aplica ao parágrafo inteiro (ou a todos os parágrafos que uma seleção tocar).
- **Fica salva dentro do próprio texto**, como um caractere de direção invisível — então sobrevive ao fechamento da nota, à reinicialização do aplicativo e à sincronização, e até viaja com o texto se você colá-lo no Word ou no Obsidian.
- **Um único Ctrl+Z desfaz.** Pressionar o mesmo lado duas vezes não faz nada além disso.
- **O Markdown permanece seguro.** Listas continuam listas, títulos continuam títulos, citações continuam citações. Blocos de código, tabelas e linhas horizontais são deliberadamente deixados intactos. Uma linha que *começa* com uma #tag mantém a sua direção automática (uma marca forçada ali quebraria a tag) — o resto do parágrafo ainda muda de direção.

---

## Fontes e a interface

- **Fontes de escrita**: configure fontes para árabe, hebraico e CJK de forma independente em **Configurações → Idioma**.
- **Barras de ferramentas de escrita**: botões de símbolos e de pontuação específicos de cada idioma.
- **Destaque de tashkeel**: ative ou desative o destaque dos diacríticos árabes pela barra de ferramentas do editor.
- Selecionar árabe ou hebraico como idioma da interface inverte todo o aplicativo para RTL.

---

## Glossário

- **Ordem de leitura** — a ordem em que os caracteres são escritos e lidos, independentemente de onde aparecem na tela.
- **Costura** — a fronteira entre um trecho da direita para a esquerda e um trecho da esquerda para a direita na mesma linha.
- **Imposição rígida** — uma direção explícita definida por você, que vence a detecção automática pela primeira letra.
- **Marca de direção** — o caractere invisível (RLM/LRM) que guarda a sua imposição dentro do próprio texto.

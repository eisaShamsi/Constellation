---
aliases:
  - Temas
  - Designer de estilo
  - Tema personalizado
  - Importar tema do Obsidian
  - Excluir tema
  - Estilos salvos
description: Personalize cada parte visível do Constellation — todo o estilo (cores, tipografia, componentes, cores dos tipos de link e estilos salvos) vive no Designer de estilo; os temas vivem em Aparência.
---

# Aparência e Temas

A aparência do Constellation é controlada a partir de **Configurações**:

1. **Aparência** — escolha ou crie um tema, importe temas do registro da comunidade do Obsidian e ajuste o alinhamento do título e o ciclo de vida dos Links Vivos.
2. **Designer de estilo (Style Setter)** — sua própria aba na barra lateral de Configurações e agora o **único lar de todo o estilo**: cada cor, tamanho, fonte e elemento da interface e do editor, as cores dos tipos de link e os estilos salvos. (A antiga aba **Style Settings** foi aposentada e mesclada por completo aqui.)

Juntos, eles permitem remodelar o aplicativo para combinar com seu fluxo de trabalho, tamanho de tela e gosto pessoal — sem editar uma única linha de CSS.

## Temas

Um **tema** é um pacote nomeado de cores, configurações e CSS que define a aparência do Constellation. O Constellation vem com seis temas integrados (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark), todos emparelhados entre modos claro e escuro.

### Escolher um tema

1. Abra **Configurações → Aparência**.
2. Clique em qualquer cartão na grade **Temas**. O tema é aplicado imediatamente.
3. O cartão ativo é destacado com uma borda de acento.

### Criar um tema personalizado

1. Na grade de temas, clique no cartão tracejado **+ Novo Tema**.
2. Dê um nome, escolha claro ou escuro e selecione cinco cores (fundo, superfície, texto, acento, borda).
3. Clique em **Salvar**. Seu tema agora aparece na grade.

Todas as outras variáveis (estados hover, sombras, texto atenuado) são derivadas automaticamente de suas cinco cores usando matemática HSL, então você só controla o que importa.

### Editar ou excluir um tema personalizado

Passe o mouse sobre qualquer cartão de tema personalizado:
- **✏️ (lápis)** — abre o editor para alterar nome, tipo ou as cinco cores principais.
- **✕ (X vermelho)** — exclui o tema após confirmação. Temas integrados não podem ser excluídos. Se você excluir o tema ativo, o Constellation reverte para o padrão.

### Importar um tema da comunidade do Obsidian

Clique em **🟣 Temas do Obsidian** para navegar por mais de 200 temas comunitários:
1. Pesquise por nome ou autor.
2. Clique em **Visualizar** para ver uma maquete do layout e a paleta de cinco cores.
3. Clique em **Importar** — o CSS do tema é baixado, adaptado para o Constellation (shim de seletores + extração de variáveis + cores de sintaxe do CodeMirror) e adicionado aos seus temas personalizados.
4. Se o tema suportar **Style Settings**, a contagem é mostrada em seu cartão; essas opções aparecem no Designer de estilo após a importação.

## Style Settings → agora dentro do Designer de estilo

> **Nota:** a aba independente **Style Settings** foi aposentada. Cada controle que ela tinha agora vive dentro do **Designer de estilo** (sua própria aba na barra lateral de Configurações) — que cobre todos eles e mais (a trilha de navegação, o resumo da nota, o painel do Universo, as fontes por sistema de escrita). A descrição a seguir detalha essa superfície de estilo, agora aberta a partir do Designer de estilo.

Essa superfície de estilo é o painel de controle nativo e independente de tema do Constellation. Cobre cada peça visível do chrome, além do editor, e funciona com qualquer tema (integrado, personalizado ou importado).

### Como está organizado

As seções estão recolhidas por padrão. Clique no chevron para expandir:

- **Constellation — Cores** — fundo e superfícies, texto, acento
- **Constellation — Tipografia** — tamanhos de fonte de interface/nota/código, tamanhos H1–H6, peso dos títulos, alturas de linha, espaçamento entre parágrafos
- **Constellation — Layout e Forma** — cantos (raios pequeno/médio/grande), larguras de borda, sombras, comprimento de linha do editor, margens laterais
- **Constellation — Componentes** — dock de faixa, barra de ações lateral, barra de layout (alternadores de painel), barra superior/faixa de abas, barra de status, explorador de arquivos, barra lateral direita, botões, tags, callouts
- **Constellation — Editor** — links, código e blocos, citação em bloco, cursor e seleção

### Alterar um valor

- **Seletores de cor** — clique na amostra, escolha uma cor. O hex é mostrado ao lado.
- **Deslizadores** — arraste para ajustar. O valor numérico aparece na unidade (px, %, etc.).
- **Interruptores** — clique para alternar classes (principalmente para temas importados).
- **Menus suspensos** — escolha uma opção (estilo de decoração de link, etc.).
- **Seta de redefinição (↺)** — aparece ao passar o mouse no final de cada linha. Clicar apaga sua substituição e restaura o padrão do tema.

### Como funciona o salvamento

- As alterações são salvas automaticamente nos **styleSettingsValues** do tema ativo.
- Se você alterar uma Style Setting enquanto um tema integrado estiver ativo, o Constellation **clona automaticamente** o integrado em seus temas personalizados (como `{Nome} (custom)`), depois salva suas alterações lá. O integrado permanece intacto.
- O rótulo **Salvo em:** na parte inferior da aba mostra qual tema contém atualmente suas substituições.
- Clique em **Redefinir tudo para padrões** para apagar todas as substituições no tema ativo.

### Importar / Exportar Style Settings

Barra de ferramentas no topo da aba Style Settings:

- **📋 Colar da área de transferência** — um clique: lê a área de transferência e mescla JSON válido no tema ativo.
- **⬆️ Importar / Colar** — abre uma área de texto; cole JSON manualmente. Escolha **Mesclar** (adiciona/substitui) ou **Substituir tudo** (apaga, usa apenas o colado).
- **📄 De arquivo** — abra um arquivo `.json` exportado do plugin Style Settings do Obsidian ou outra instalação Constellation.
- **📋 Copiar** — copia os valores atuais para a área de transferência como JSON formatado.
- **⬇️ Exportar** — salva os valores como `{theme-name}-style-settings.json`.

O formato JSON corresponde exatamente ao plugin Style Settings do Obsidian — um objeto plano que mapeia IDs de configuração para valores de string:

```json
{
  "h1-size": "36",
  "interactive-accent": "#7c3aed",
  "my-themed-color@@light": "#ffffff",
  "my-themed-color@@dark": "#1e1e2e"
}
```

Isso significa que você pode copiar suas Style Settings do Obsidian e colá-las diretamente no Constellation, ou vice-versa.

## O que você pode controlar

Cada configuração fica sob um dos cinco blocos acima. Destaques:

### Tipografia

- **Tamanho de fonte da interface** — barra lateral, barras de ferramentas, menus
- **Tamanho de fonte da nota** — texto do corpo no editor
- **Tamanho de fonte de código** — código em linha e blocos de código
- **Tamanhos H1 – H6** — cada nível de título individualmente
- **Peso dos títulos** — leveza ou negrito de todos os títulos
- **Alturas de linha** — normal (corpo) e apertada (títulos e UI densa)
- **Espaçamento entre parágrafos** — lacuna entre parágrafos

### Componentes da casca

- **Dock de faixa (ícones à esquerda)** — largura, tamanho do botão, tamanho do ícone, raio, cores
- **Barra de ações lateral** — ícones nova nota/tabela/pasta — tamanho, cor, altura, fundo
- **Barra de layout (alternadores de painel)** — alternadores de barra lateral esquerda/divisão/direita — tamanho do botão, tamanho do ícone, cores, cor do estado ativo
- **Barra superior / faixa de abas** — visível apenas quando há notas abertas em abas; controla altura da faixa, fundo, altura/fonte/raio da aba, cores da aba ativa e inativa
- **Barra de status** — altura, tamanho de fonte, fundo, cor do texto
- **Barra lateral direita (inspetor)** — fundo, altura da linha de abas, tamanho do ícone da aba, cores
- **Explorador de arquivos (barra lateral esquerda)** — linha de notas do Universo, linhas de universos filhos (cUniverse), nomes de bibliotecas, pastas, notas — cada um com tamanho, peso e cor independentes; mais espaçamento vertical de linhas

### Editor

- **Tamanhos de títulos** (H1–H6) e peso
- **Altura de linha** no corpo da nota
- **Código em linha** fundo, cor do texto, raio, tamanho de fonte
- **Cor do link** (padrão + hover) e estilo de decoração (nenhum/sublinhado/pontilhado)
- **Largura da barra de callout** e **raio do callout**
- **Cor do cursor** e **fundo de seleção**

### Cores (cada cor no aplicativo)

- Fundo (primário/alt), superfícies, fundo hover, bordas, fundo de entrada
- Texto (normal/atenuado/fraco/sobre acento), estados erro/aviso/sucesso
- Acento (acento interativo + hover), texto sobre acento

## O Designer de estilo

O **Designer de estilo** (Style Setter) é um estúdio de design em tela cheia para toda a sua interface. Em vez de ajustar as configurações uma a uma e imaginar o resultado, você altera um controle e observa o seu **aplicativo real** se reestilizar enquanto trabalha. É a sua própria aba na barra lateral de Configurações.

**Abra-o:** clique em **Designer de estilo** na barra lateral de Configurações. Você pode **redimensionar o painel** — arraste a pequena alça no canto inferior direito; ele lembra o tamanho na próxima vez.

**Escolha o que estilizar — a lista à esquerda.** Na lateral esquerda ficam as *Superfícies* que você pode estilizar:

- **Interface** — a árvore de arquivos, a barra de status e a barra do Universo.
- **Componentes** — o dock de faixa, as barras de ferramentas, a barra superior e abas, botões, tags e callouts.
- **Editor** — a própria nota: a linha de **trilha de navegação**, títulos, negrito, itálico, links, código em linha, citações em bloco e o **resumo da nota** (a linha em itálico sob o título).
- **Global** — tons de fundo e texto, tons de acento, tipografia e espaçamento, cantos e bordas, e fontes por sistema de escrita.
- **Links** — as cores dos tipos de link e como elas são exibidas.
- **Vista do Céu / OrgChart / Índice / Cataloger / Shell** — as superfícies dos plug-ins.

Abaixo delas estão os seus **estilos salvos** — clique em um para aplicar todo aquele visual de uma vez (veja *Salvar um visual como um Estilo nomeado*, abaixo). *(Os temas integrados são escolhidos em Configurações → Aparência, não aqui.)*

**A mira de inspeção.** Acima da engrenagem de Configurações no dock há uma **mira de inspeção**. Clique nela e depois passe o cursor e clique em qualquer parte do aplicativo — o Designer de estilo abre direto nos controles daquele elemento. É a forma mais rápida de pular do que você vê para o que ajusta.

**Duas formas de ver as suas alterações:**

- **A categoria Editor** mostra uma **pré-visualização da nota no centro.** Clique em um título, em negrito, em um link ou na página e os controles aparecem à direita; a pré-visualização se atualiza no mesmo instante.
- **Todas as outras categorias** encostam o painel em um dos lados e ficam translúcidas, e as suas edições aparecem no **aplicativo real, ao vivo.** Altere a cor da barra de status ou a largura do dock e a barra lateral, o dock, as abas e a barra de status reais se reestilizam **enquanto você arrasta.** Uma etiqueta verde **● live** na barra superior lembra que você está editando a coisa real.

**A categoria Links** mantém num só lugar as cores dos tipos de link, os interruptores de exibição dos links tipados e os controles de forma das pílulas. Cada um dos oito tipos (supports, contradicts, …) é mostrado como a sua **pílula** colorida real — **clique numa pílula para recoloria-la,** e a mudança reflete ao vivo em todo lugar (os links do editor e as pílulas de Backlinks / Saídas). Acima da lista ficam os interruptores — **Colorir links tipados** e **Mostrar rótulos de tipo** — e a **forma da pílula** (raio do canto, altura, peso do rótulo). Uma paleta de **Cores salvas** lembra cada cor que você escolhe para reutilizar em qualquer elemento.

**Manter (Keep), Descartar (Discard), Redefinir (Reset).** Quando gostar do que vê, clique em **Manter** (canto superior direito) para salvar o visual **para este Universo** — ele sobrevive a uma reinicialização. **Descartar** (ou simplesmente fechar com **✕** ou **Esc**) joga fora as suas edições não salvas e o aplicativo real volta para o visual salvo. **Redefinir** limpa tudo de volta ao tema simples. Nada é gravado em disco até você Manter.

**Salve um visual como um Estilo nomeado.** Para reutilizar um visual, salve-o sob um nome: digite um nome no campo **"draft:"** no topo e clique em **"+ Salvar atual como um estilo"** (canto inferior esquerdo). Ele entra na sua lista de **estilos salvos** — global do aplicativo (reutilizável em todos os Universos) e captura o visual que você desenhou no Designer, não apenas um tema. **Clique num estilo salvo para aplicá-lo.** Passe o cursor sobre a linha de um estilo salvo para ver as suas ações: **↻ Atualizar** (sobrescreve aquele estilo com o seu visual *atual* — mantém o nome), **⤓ Exportar** (compartilha como um `.constellation-style.json`), **✎ Renomear** e **✕ Excluir**.

> Os temas integrados (Midnight, Daylight, …) ficam em **Configurações → Aparência**, não no Designer — o Designer guarda os seus **estilos salvos** e o visual ao vivo por Universo.

## Perguntas frequentes

### Posso estilizar a barra de título do Windows ("Constellation v0.3.4 — …")?

Não — essa barra é desenhada pelo sistema operacional (Windows/macOS/Linux). O Constellation não tem acesso CSS a ela. Tudo abaixo é totalmente estilizável.

### Como eu mudo a largura da barra lateral?

Arraste a borda da barra lateral (o manipulador de arrasto na borda) para redimensioná-la. Não existe um deslizador para isso — ele foi removido para evitar fontes de verdade conflitantes.

### Onde vivem as minhas configurações de estilo?

O visual ao vivo que você desenha no Designer de estilo é salvo **por Universo** quando você clica em **Manter** — ele viaja com o seu Universo, então se você sincronizar o diretório do Universo entre dispositivos, o seu estilo vem junto. Os **estilos salvos** nomeados, por outro lado, são globais do aplicativo (reutilizáveis em todos os Universos).

### Posso compartilhar um tema com alguém?

Sim:
- **Tema completo** — no editor de tema, clique em **Exportar**. Compartilhe o arquivo `.json`. O destinatário clica em **↓ Importar** na grade de temas e o seleciona.
- **Um visual completo (Estilo)** — no Designer de estilo, passe o cursor sobre um estilo salvo e clique em **⤓ Exportar** para obter um arquivo `.constellation-style.json`. Diferente de um tema, ele carrega o visual inteiro (tema, fontes, cores dos links, forma), e o destinatário o adiciona como um novo estilo salvo. Útil para compartilhar a sua aparência pessoal por completo.

### Um tema do Obsidian importado parece quebrado. E agora?

Temas do Obsidian podem ser complexos. Casos conhecidos:
- Temas que usam **cores HSL divididas** (como o Minimal) — suportados no Constellation a partir desta versão.
- Temas que dependem da estrutura DOM específica do Obsidian podem renderizar parcialmente. O Constellation inclui um shim de classes que mapeia os seletores mais comuns, mas temas muito estruturais podem exigir ajuste das cinco cores principais ou correção manual dos valores de Style Settings para compensar.

## Relacionado

- [[Universe]] — onde temas e valores de Style Settings são armazenados
- [[Libraries]] — acentos de cor por biblioteca (definidos nas configurações da biblioteca, independentes dos temas)
- [[Importer]] — para importar notas, não temas (a importação de temas está em Aparência)

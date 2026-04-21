---
aliases:
  - Temas
  - Style Settings
  - Tema personalizado
  - Importar tema do Obsidian
  - Excluir tema
  - Exportar ajustes de estilo
description: Personalize cada parte visível do Constellation — temas, cores, tipografia e estilo de componentes via Aparência e a aba nativa Style Settings.
---

# Aparência e Temas

A aparência do Constellation é controlada a partir de dois locais em **Configurações**:

1. **Aparência** — escolha ou crie um tema, importe temas do registro da comunidade do Obsidian e ajuste preferências globais de fonte e layout.
2. **Style Settings** — uma aba dedicada que expõe cada peça visível da interface do Constellation como um controle ajustável ao vivo (deslizadores, seletores de cor, menus suspensos). As alterações são aplicadas instantaneamente e salvas no tema ativo.

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
4. Se o tema suportar **Style Settings**, a contagem é mostrada em seu cartão; essas opções aparecem na aba Style Settings após a importação.

## Style Settings

A aba **Style Settings** é o painel de controle nativo e independente de tema do Constellation. Cobre cada peça visível do chrome, além do editor, e funciona com qualquer tema (integrado, personalizado ou importado).

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

## Perguntas frequentes

### Posso estilizar a barra de título do Windows ("Constellation v0.3.4 — …")?

Não — essa barra é desenhada pelo sistema operacional (Windows/macOS/Linux). O Constellation não tem acesso CSS a ela. Tudo abaixo é totalmente estilizável.

### Por que o deslizador de largura da barra lateral não funciona?

A largura da barra lateral é controlada pelo manipulador de arrasto na borda da barra (arraste para redimensionar). Deliberadamente não duplicamos esse controle em Style Settings para evitar fontes de verdade conflitantes.

### Onde vivem minhas Style Settings?

Dentro de `Universe/settings.json` em `customThemes[i].styleSettingsValues`, com escopo por tema. Elas viajam com seu Universo — se você sincronizar seu diretório de Universo entre dispositivos, seu estilo vem junto.

### Posso compartilhar um tema com alguém?

Sim:
- **Tema completo** — no editor de tema, clique em **Exportar**. Compartilhe o arquivo `.json`. O destinatário clica em **↓ Importar** na grade de temas e o seleciona.
- **Apenas valores de Style Settings** — na aba Style Settings, clique em **Exportar** para exportar apenas os valores de deslizadores/cores (não a estrutura do tema). Útil para aplicar seus ajustes pessoais sobre o tema de outra pessoa.

### Um tema do Obsidian importado parece quebrado. E agora?

Temas do Obsidian podem ser complexos. Casos conhecidos:
- Temas que usam **cores HSL divididas** (como o Minimal) — suportados no Constellation a partir desta versão.
- Temas que dependem da estrutura DOM específica do Obsidian podem renderizar parcialmente. O Constellation inclui um shim de classes que mapeia os seletores mais comuns, mas temas muito estruturais podem exigir ajuste das cinco cores principais ou correção manual dos valores de Style Settings para compensar.

## Relacionado

- [[Universe]] — onde temas e valores de Style Settings são armazenados
- [[Libraries]] — acentos de cor por biblioteca (definidos nas configurações da biblioteca, independentes dos temas)
- [[Importer]] — para importar notas, não temas (a importação de temas está em Aparência)

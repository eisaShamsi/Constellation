# Manual do Usuario do Constellation

**Versao 0.3.4 | Marco 2026**

Constellation e um aplicativo de desktop para Gestao do Conhecimento Pessoal (PKM) que permite gerenciar bibliotecas de notas em Markdown. Desenvolvido com Tauri v2, SvelteKit e Rust, funciona nativamente no Windows, macOS e Linux com suporte completo para arabe e escrita RTL.

---

## Sumario

1. [Primeiros Passos](#primeiros-passos)
2. [Universo e Bibliotecas](#universo-e-bibliotecas)
3. [Criar e Editar Notas](#criar-e-editar-notas)
4. [Vista Estelar (GraphMind)](#vista-estelar-graphmind)
5. [Segunda Tela](#segunda-tela)
6. [Propriedades e Frontmatter](#propriedades-e-frontmatter)
7. [Modelos](#modelos)
8. [Tabelas](#tabelas)
9. [Tarefas](#tarefas)
10. [Importador](#importador)
11. [Calendario](#calendario)
12. [Lens](#lens)
13. [Configuracoes](#configuracoes)
14. [Atalhos de Teclado](#atalhos-de-teclado)
15. [Suporte RTL e Arabe](#suporte-rtl-e-arabe)
16. [Seguranca e Privacidade](#seguranca-e-privacidade)
17. [Motor Cognitivo](#motor-cognitivo)

---

## 1. Primeiros Passos

### Instalacao

Baixe o instalador mais recente na [pagina de versoes do Constellation](https://github.com/eisaShamsi/Constellation/releases):

- **Windows**: Instalador `.exe` (NSIS) ou `.msi`
- **macOS**: Imagem de disco `.dmg`
- **Linux**: Pacote `.AppImage` ou `.deb`

### Primeira Execucao

Ao abrir o Constellation pela primeira vez, o **Assistente de Configuracao do Universo** guia voce atraves de:

1. **Escolha seu idioma** — 15 idiomas disponiveis
2. **Crie ou importe uma biblioteca** — aponte para uma pasta existente com arquivos Markdown, ou comece do zero
3. **Nomeie seu universo** — o universo e o contêiner de todas as suas bibliotecas

### Visao Geral da Interface

| Elemento | Descricao |
|----------|-----------|
| **Barra Lateral (Ribbon)** | Botoes de navegacao: Arvore de arquivos, Busca, Vista Estelar, Calendario, Modelos, Configuracoes |
| **Arvore de Arquivos** | Navegue por notas e pastas dentro das suas bibliotecas |
| **Editor** | Leia e edite suas notas em Markdown |
| **Barra de Abas** | Abra multiplas notas em abas |
| **Barra de Status** | Contagem de palavras, caracteres e tempo de leitura |

---

## 2. Universo e Bibliotecas

### O que e um Universo?

Um **Universo** e o contêiner de nivel superior que abriga todas as suas bibliotecas. Pense nele como seu espaco de trabalho ou colecao de bibliotecas.

### O que e uma Biblioteca?

Uma **Biblioteca** e uma pasta no seu computador contendo arquivos Markdown (`.md`). Voce pode ter multiplas bibliotecas em um unico universo — por exemplo, uma para notas de trabalho e outra para notas pessoais.

### Gerenciar Bibliotecas

- **Adicionar uma biblioteca**: Configuracoes > Bibliotecas > Adicionar Biblioteca, ou arraste uma pasta para o aplicativo
- **Remover uma biblioteca**: Configuracoes > Bibliotecas > clique no botao de remover ao lado do nome da biblioteca
- **Configuracoes da biblioteca**: Cada biblioteca pode ter suas proprias configuracoes de aparencia (fontes, cores)

### Universos Secundarios

Voce pode aninhar universos dentro de outros universos. Um **Universo Secundario** e outra pasta de universo referenciada pelo seu universo principal. As notas dos universos secundarios aparecem na Vista Estelar junto com suas proprias notas, com links entre bibliotecas exibidos como linhas tracejadas.

### Reabertura automática

O Constellation lembra o último universo ativo e o reabre automaticamente ao iniciar. Se o universo foi movido ou seu caminho mudou, o Constellation detecta e corrige o caminho automaticamente.

### Universos portáteis

Os universos do Constellation sao totalmente portáteis. Voce pode mover a pasta do universo para qualquer local — uma unidade diferente, pen drive USB ou outro computador — e o Constellation detectara e corrigira automaticamente todos os caminhos internos ao reabri-lo.

Para mover um universo:
1. Feche o Constellation
2. Mova ou copie a pasta do universo para o novo local
3. Abra o Constellation → a tela de boas-vindas aparece (o caminho antigo nao e mais valido)
4. Escolha **Abrir Universo Existente** e aponte para o novo local
5. Todas as notas e bibliotecas aparecem imediatamente — os caminhos sao corrigidos automaticamente

A estrutura de pastas do universo segue o modelo Obsidian: as notas ficam diretamente na pasta raiz, a configuracao reside em `.constellation/`.

---

## 3. Criar e Editar Notas

### Criar uma Nota

| Metodo | Acao |
|--------|------|
| **Teclado** | `Ctrl+N` |
| **Arvore de Arquivos** | Clique com botao direito em uma pasta > Nova Nota |
| **Mission Control** | `Ctrl+P` > "Nova nota" |

### Modos do Editor

Constellation oferece dois modos de editor, selecionaveis em **Configuracoes > Editor > Tipo de editor**:

#### Editor Markdown (CodeMirror)

O editor padrao para usuarios avancados. Escreva Markdown diretamente com:

- **Pre-visualizacao ao Vivo** — renderiza a formatacao em linha enquanto voce digita
- **Modo Fonte** — mostra a sintaxe Markdown bruta
- **Barra de formatacao** — aparece ao selecionar texto
- **Comandos com barra** — digite `/` para insercoes rapidas
- **Autocompletar de Wikilinks** — digite `[[` para vincular notas
- **Cursores multiplos** — `Alt+Click` ou `Ctrl+D`

#### Editor de Documentos (TipTap)

Uma experiencia WYSIWYG estilo processador de texto com barra visual:

- Negrito, Italico, Sublinhado, Tachado, Realce
- Titulos (H1–H3), Alinhamento de texto
- Listas com marcadores, Listas numeradas, Listas de tarefas
- Citacoes, Blocos de codigo, Linhas horizontais
- Tabelas (inserir, adicionar/remover linhas e colunas)
- Links e Imagens

Ambos os editores salvam como arquivos Markdown padrao. Voce pode alternar entre eles a qualquer momento sem perda de dados.

### Callouts (Destaques)

Crie blocos de destaque estilizados para notas, avisos, dicas e outras indicacoes:

```markdown
> [!note] Informacao importante
> O conteudo do callout vai aqui.

> [!warning] Tenha cuidado
> Esta acao nao pode ser desfeita.

> [!tip]- Clique para expandir
> Conteudo de callout recolhivel.
```

Tipos suportados: `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Cada tipo tem uma cor e icone distintos. Adicione `-` apos o tipo para torna-lo recolhivel (inicia recolhido), ou `+` (inicia expandido).

### Sintaxe de Realce

Envolva o texto com duplo sinal de igual para realca-lo:

```markdown
Este e ==texto realcado== na sua nota.
```

Na Visualizacao ao Vivo, as marcas `==` ficam ocultas e o texto aparece com fundo amarelo.

### Blocos de Codigo

Blocos de codigo delimitados sao exibidos com cor de fundo e rotulo de linguagem:

````markdown
```javascript
const greeting = "Hello, world!";
```
````

O nome da linguagem aparece como um selo acima do bloco de codigo.

### Incorporacao de Imagens

Incorpore imagens diretamente nas suas notas:

```markdown
![Texto alternativo](https://example.com/image.png)   — URL externa
![[photo.jpg]]                                          — arquivo local da biblioteca
```

Na Visualizacao ao Vivo, as imagens sao renderizadas em linha. Imagens locais devem estar na pasta da sua biblioteca. Imagens externas requerem conexao com a internet.

### Barra de Ferramentas de Tabela

Quando o cursor esta dentro de uma tabela markdown, uma barra de ferramentas flutuante aparece com:

- **+ Linha / + Coluna** — adicionar linhas ou colunas
- **- Linha / - Coluna** — remover linhas ou colunas
- **Alinhamento** — alinhamento a esquerda, centro ou direita por coluna
- **Ordenar** — ordenar linhas em ordem crescente ou decrescente pela coluna atual
- **Tab / Shift+Tab** — navegar entre celulas da tabela

### Atalhos de Formatacao de Texto

| Atalho | Acao |
|--------|------|
| `Ctrl+B` | Negrito |
| `Ctrl+I` | Italico |
| `Ctrl+Shift+S` | Tachado |
| `Ctrl+Shift+H` | Realce |
| `Ctrl+K` | Inserir wikilink |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Shift+Z` | Refazer |

### Vincular Notas

Digite `[[` para abrir o autocompletar de notas. Comece a digitar o nome de uma nota e selecione entre as sugestoes. Os links aparecem como wikilinks clicaveis: `[[Nome da Nota]]`.

Voce tambem pode vincular a titulos especificos: `[[Nome da Nota#Titulo]]`.

---

## 4. Vista Estelar (GraphMind)

A Vista Estelar visualiza suas notas como um grafo 3D interativo alimentado pelo motor **GraphMind** (Pixi.js WebGL).

### Abrir a Vista Estelar

- Clique no icone de grafo na barra lateral
- Pressione `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Vista Estelar"

### Navegacao

| Entrada | Acao |
|---------|------|
| **Clique + arrastar** | Deslocar o grafo |
| **Scroll** | Aproximar/afastar |
| **Clique em um no** | Abrir a nota |
| **Clique direito em um no** | Menu de contexto (Abrir, Focar, Fixar, Ocultar) |
| **Clique do meio + arrastar** | Rotacionar em 3D |
| **W/A/S/D** | Voar pelo espaco 3D |
| **0** | Redefinir rotacao para 2D |
| **Ctrl+F** | Buscar e destacar |
| **Space** | Alternar modo foco |

### Modos de Layout

Pressione `Ctrl+L` para alternar entre:

- **Organico** — layout dirigido por forcas onde agrupamentos surgem naturalmente
- **Hierarquico** — layout em arvore de cima para baixo
- **Temporal** — notas organizadas por data de criacao em uma linha do tempo

### Modo Foco

Clique com botao direito em um no > **Focar** para ver apenas sua vizinhanca. Ajuste:

- **Profundidade** (1–5 saltos) — quantos niveis de conexoes exibir
- **Direcao** (↔/←/→) — todos os links, apenas de entrada, ou apenas de saida

### Navegacao 3D

Clique com o botao do meio e arraste para rotacionar. Use W/A/S/D/Q/E para voar pelo campo estelar. Um indicador de eixos XYZ no canto mostra sua orientacao. Pressione `0` para redefinir.

### Configuracoes

Clique no icone de engrenagem para:

- **Aparencia**: Tamanho do no, visibilidade dos rotulos, tamanho da fonte, espessura dos links, exibir orfaos
- **Fisica**: Forca de repulsao, forca de ligacao, distância de ligacao
- **IA**: Limiar de links semanticos (Fase 2)

### Legenda

A legenda no canto inferior direito mostra as cores de biblioteca/pasta com caixas de selecao para alternar a visibilidade.

### Estratos de Conhecimento

A Vista Estelar classifica automaticamente suas notas em oito estratos de conhecimento com base no nivel de abstracao:

| Estrato | Descricao |
|---------|-----------|
| **Instantaneo** | Notas rapidas e efemeras |
| **Registro** | Eventos datados e entradas de diario |
| **Topico** | Conceitos atomicos sobre uma unica ideia |
| **Mapa** | Notas organizativas que conectam outros topicos |
| **Estrutura** | Modelos e estruturas de pensamento |
| **Principio** | Regras e axiomas verificados |
| **Conviccao** | Valores e crencas fundamentais |
| **Artefato** | Obras concluidas e definitivas |

O estrato e determinado automaticamente a partir do frontmatter, estrutura e links da nota. Voce pode substituir a classificacao manualmente adicionando uma propriedade `stratum` no frontmatter.

### Ciclo de Maturidade

Cada nota passa por um ciclo de maturidade que reflete seu grau de desenvolvimento:

- **Semente** — Ideia inicial ou rascunho bruto
- **Muda** — A nota comeca a tomar forma e tem alguns links
- **Perene** — Nota madura, revisada e bem conectada
- **Canonica** — Referencia definitiva e autoritativa em seu tema

O nivel de maturidade e atualizado automaticamente com base no numero de links, data de revisao e frequencia de edicao. Voce tambem pode defini-lo manualmente atraves da propriedade `maturity` no frontmatter.

---

## 5. Segunda Tela

Abra uma janela separada para visualizacao de notas lado a lado.

- **Abrir**: Clique no icone de segunda tela na barra lateral, ou `Ctrl+Shift+N`
- **Sincronizacao**: As notas abrem na segunda tela de forma independente. As configuracoes de fontes e tema se aplicam a ambas as janelas.
- **Largura da nota**: Ajustavel atraves do controle deslizante na barra de ferramentas

---

## 6. Propriedades e Frontmatter

As notas podem ter frontmatter YAML no topo:

```yaml
---
tags: [projeto, ativo]
date: 2026-03-19
status: em-andamento
---
```

Constellation detecta os tipos de propriedades automaticamente:

| Tipo | Exemplo |
|------|---------|
| **Texto** | `author: Joao` |
| **Numero** | `priority: 5` |
| **Data** | `date: 2026-03-19` |
| **Lista** | `tags: [a, b, c]` |
| **Caixa de selecao** | `done: true` |
| **Link** | `related: [[Outra Nota]]` |

Alterne a exibicao de propriedades em **Configuracoes > Editor > Propriedades no documento** (Visivel / Oculto / Fonte).

---

## 7. Modelos

Crie modelos de notas reutilizaveis:

1. Crie uma pasta para modelos na sua biblioteca
2. Defina o caminho da pasta de modelos em **Configuracoes > Modelos**
3. Ao criar uma nova nota, escolha um modelo no seletor de modelos

Os modelos suportam variaveis:

| Variavel | Substituida por |
|----------|-----------------|
| `{{date}}` | Data atual |
| `{{time}}` | Hora atual |
| `{{title}}` | Titulo da nota |
| `{{clipboard}}` | Conteudo da area de transferencia |

---

## 8. Tabelas

### Tabelas Markdown

Digite uma tabela Markdown manualmente ou use o comando de barra `/table`:

```markdown
| Cabecalho 1 | Cabecalho 2 |
|-------------|-------------|
| Celula 1    | Celula 2    |
```

### Barra de Ferramentas de Tabela

Quando o cursor esta dentro de uma tabela, uma barra flutuante aparece com:

- Adicionar/remover linhas e colunas
- Alinhar colunas (esquerda, centro, direita)
- Navegar entre celulas com `Tab` / `Shift+Tab`

### Tabelas no Editor de Documentos

O editor de Documentos (TipTap) oferece uma experiencia visual de tabelas:

- Clique no botao de tabela para inserir
- Use o menu suspenso para gerenciar linhas/colunas
- Redimensione colunas arrastando as bordas

---

## 9. Tarefas

Constellation suporta caixas de selecao de tarefas nas notas:

```markdown
- [ ] Tarefa incompleta
- [x] Tarefa concluida
```

No modo de Pre-visualizacao ao Vivo, as caixas de selecao sao clicaveis. As tarefas podem ser buscadas e filtradas em todas as suas bibliotecas.

---

## 10. Importador

Importe notas de outras ferramentas PKM:

- **Obsidian** — importa vaults com compatibilidade completa de wikilinks
- **Pastas Markdown** — importe qualquer pasta de arquivos `.md`
- **Outros formatos** — HTML, arquivos de texto

Va para **Configuracoes > Importador** para iniciar uma importacao.

---

## 11. Calendario

A visualizacao do Calendario mostra as notas organizadas por data:

- Notas com uma propriedade `date` aparecem nos seus respectivos dias
- Notas diarias podem ser criadas para qualquer data
- Navegue entre meses com os botoes de seta

Abra o Calendario na barra lateral.

---

## 12. Lens

Lens fornece visualizacoes filtradas das suas notas:

- Filtre por etiquetas, pastas, propriedades
- Ordene por nome, data ou propriedades personalizadas
- Salve configuracoes do Lens para acesso rapido

---

## 13. Configuracoes

Acesse as Configuracoes pelo icone de engrenagem na barra lateral ou `Ctrl+,`.

### Geral

- Idioma (15 idiomas)
- Tema (Claro / Escuro)
- Fonte da interface, Fonte de texto, Fonte monoespcada, Tamanho da fonte
- Tema de fonte — combinacoes de fontes predefinidas (Maquina de escrever, Classico, Moderno, etc.) para troca rapida

### Editor

- Tipo de editor (Markdown / Documento)
- Visualizacao padrao (Leitura / Edicao)
- Modo de Pre-visualizacao ao Vivo
- Numeros de linha, Guias de indentacao, Verificacao ortografica
- Auto-fechar parenteses, Listas inteligentes

### Bibliotecas

- Adicionar/remover bibliotecas
- Configuracoes de aparencia por biblioteca
- Localizacao da pasta de anexos

### Atualizacoes

- Verificar atualizacoes
- Token do GitHub para atualizacoes de repositorios privados

---

## 14. Atalhos de Teclado

### Globais

| Atalho | Acao |
|--------|------|
| `Ctrl+N` | Nova nota |
| `Ctrl+O` | Star Jump (abertura rapida) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Abrir Vista Estelar |
| `Ctrl+,` | Configuracoes |
| `Ctrl+Shift+F` | Buscar na biblioteca |
| `Ctrl+Shift+N` | Segunda tela |

### Editor

| Atalho | Acao |
|--------|------|
| `Ctrl+B` | Negrito |
| `Ctrl+I` | Italico |
| `Ctrl+K` | Inserir wikilink |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Shift+Z` | Refazer |
| `Ctrl+D` | Selecionar proxima ocorrencia |
| `Ctrl+/` | Alternar comentario |
| `Tab` | Indentar / proxima celula da tabela |

### Vista Estelar

| Atalho | Acao |
|--------|------|
| `Ctrl+F` | Buscar e destacar |
| `Ctrl+L` | Alternar modo de layout |
| `Space` | Alternar modo foco |
| `0` | Redefinir rotacao 3D |
| `W/A/S/D/Q/E` | Voar em 3D |
| `Escape` | Fechar Vista Estelar |

---

## 15. Suporte RTL e Arabe

Constellation oferece suporte de primeira classe para arabe, hebraico, persa, urdu e outros idiomas com escrita RTL:

- **Deteccao automatica**: A direcao da nota e detectada automaticamente a partir do conteudo
- **Interface**: Interface RTL completa quando o idioma arabe/hebraico e selecionado
- **Editor**: Edicao de texto RTL com movimento de cursor e selecao corretos
- **Vista Estelar**: Rotulos em arabe sao renderizados da direita para a esquerda com fallback de fonte adequado
- **Legenda**: Os itens invertem a ordem ponto/texto com base no idioma do conteudo
- **Fontes de escrita**: Configure fontes para arabe, hebraico e CJK independentemente nas Configuracoes

### Configuracao para Arabe

1. Va para **Configuracoes > Geral > Idioma** e selecione Arabe
2. Opcionalmente, defina uma fonte dedicada para arabe em **Configuracoes > Geral > Fontes de escrita**
3. Notas com conteudo em arabe serao renderizadas automaticamente em RTL

---

## 16. Seguranca e Privacidade

- **Todos os dados permanecem locais** — sem sincronizacao na nuvem, sem telemetria, sem rastreamento
- **Arquivos Markdown** — suas notas sao arquivos de texto simples que pertencem totalmente a voce
- **Sem conta necessaria** — Constellation funciona completamente offline
- **Atualizacoes opcionais** — verifique atualizacoes manualmente nas Configuracoes
- **Codigo aberto** — inspecione o codigo em [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 17. Motor Cognitivo

O Motor Cognitivo e o sistema de inteligencia integrado do Constellation que analisa suas notas e revela padroes ocultos e relacoes entre suas ideias. Sua filosofia fundamental:

> "A quantidade dos seus dados nao importa. Nao se trata de quantas fontes voce armazena, mas de como voce formula seu conhecimento a partir delas e o conecta em uma consciencia unica e significativa."

O Motor Cognitivo e composto por cinco ferramentas integradas: Links tipados, Estratos de conhecimento, Ciclo de maturidade, Detector de tensoes e Cadeia de proveniencia.

---

### 17.1 Links tipados

#### O que e?

Links tipados sao wikilinks que carregam um tipo de relacao descrevendo como duas notas se conectam. Em vez de simplesmente escrever `[[nota]]`, voce escreve `[[nota|tipo-de-relacao]]` para expressar a natureza do vinculo — e derivada dela? Contradiz? Estende?

#### Por que importa?

Um link comum diz "ha uma conexao" sem dizer qual. Links tipados transformam sua rede de notas de um amontoado de referencias em um verdadeiro mapa de conhecimento que torna visiveis as estruturas de pensamento, dependencias e raciocínios entre ideias.

#### Como usar

1. Abra uma nota no editor
2. Escreva um wikilink com tipo de relacao: `[[Nota destino|derives-from]]`
3. Tipos suportados: `derives-from` (derivado de), `supports` (apoia), `contradicts` (contradiz), `extends` (estende), `exemplifies` (exemplifica), `questions` (questiona)
4. Voce tambem pode adicionar tipos atraves das propriedades da nota na barra lateral direita

#### Onde aparece?

- **Vista Estelar (GraphMind)**: Como linhas coloridas e rotuladas entre os nos
- **Barra lateral direita**: Na aba "Backlinks" com indicacao do tipo de cada link
- **Aba Proveniencia**: Usada para construir a arvore genealogica do conhecimento

---

### 17.2 Estratos de conhecimento

#### O que e?

O Motor Cognitivo classifica automaticamente cada nota em um dos oito estratos: Instantaneo, Registro, Topico, Mapa, Estrutura, Principio, Conviccao, Artefato. A classificacao baseia-se na estrutura, conteudo e numero de links da nota.

#### Por que importa?

Conhecer o tipo de cada nota revela o equilibrio do conhecimento na sua biblioteca. A maioria das suas notas sao meros instantaneos fugaces ou evoluiram para principios e estruturas? Essa consciencia sobre a natureza do conteudo e o primeiro passo para construir conhecimento real em vez de apenas acumular informacoes.

#### Como usar

1. A classificacao ocorre automaticamente — nao e necessario fazer nada
2. Para substituir a classificacao automatica, adicione a propriedade `stratum` no frontmatter:
   ```yaml
   ---
   stratum: framework
   ---
   ```
3. Valores disponiveis: `snapshot`, `log`, `topic`, `map`, `framework`, `principle`, `conviction`, `artifact`

#### Onde aparece?

- **Barra lateral direita**: Na secao de propriedades sob "Estrato"
- **Vista Estelar**: Como cores diferentes dos nos conforme o estrato
- **Configuracoes > Motor Cognitivo**: Para ativar ou desativar a classificacao automatica

---

### 17.3 Ciclo de maturidade

#### O que e?

O motor acompanha o nivel de maturidade de cada nota em quatro estagios: **Semente** → **Muda** → **Perene** → **Canonica**. Cada nota comeca como semente e cresce gradualmente com mais conteudo, links e revisoes.

#### Por que importa?

A maturidade distingue uma ideia bruta de conhecimento refinado. A semente de hoje pode se tornar referencia amanha se voce dedicar a atencao necessaria. O rastreamento de maturidade ajuda a identificar notas que merecem mais desenvolvimento e atencao.

#### Como usar

1. A maturidade muda automaticamente com base em: contagem de palavras, numero de links de entrada e saida e data da ultima modificacao
2. Para definir a maturidade manualmente, adicione a propriedade `maturity` no frontmatter:
   ```yaml
   ---
   maturity: evergreen
   ---
   ```
3. Valores disponiveis: `seed` (Semente), `sapling` (Muda), `evergreen` (Perene), `canonical` (Canonica)

#### Onde aparece?

- **Barra lateral direita**: Um icone ao lado do titulo mostra o estagio de maturidade atual
- **Vista Estelar**: Como tamanho do no — quanto mais madura a nota, maior o no
- **Configuracoes > Motor Cognitivo**: Para ativar ou desativar o rastreamento de maturidade

---

### 17.4 Detector de tensoes

#### O que e?

O Detector de tensoes examina notas vinculadas e alerta quando ha afirmacoes ou conclusoes contraditorias entre duas ou mais notas. Baseia-se na analise de links tipados `contradicts` e na similaridade tematica entre notas.

#### Por que importa?

Tensoes nao sao necessariamente erros — sao convites para pensar mais profundamente. Quando duas ideias na sua biblioteca se contradizem, isso significa que sua compreensao evoluiu ou que existe uma complexidade que vale a pena explorar. Detectar tensoes protege voce de construir conhecimento inconscientemente sobre bases contraditorias.

#### Como usar

1. Adicione um link tipado `contradicts` entre notas conflitantes: `[[Outra nota|contradicts]]`
2. O motor tambem detecta tensoes implicitas por analise de conteudo
3. Consulte a lista de tensoes detectadas na barra lateral

#### Onde aparece?

- **Barra lateral direita**: Na aba "Tensoes" quando contradicoes sao detectadas
- **Vista Estelar**: Como linhas vermelhas tracejadas entre nos conflitantes
- **Painel de notificacoes**: Alertas quando uma nova tensao e detectada

---

### 17.5 Cadeia de proveniencia

#### O que e?

A Cadeia de proveniencia rastreia a origem de cada ideia — de onde veio e do que foi derivada. Utiliza links `[[nota|derives-from]]` para construir uma arvore genealogica que mostra o caminho de evolucao do conhecimento desde a fonte original ate a formulacao atual.

#### Por que importa?

Saber de onde vem suas ideias distingue conhecimento recebido (de livros, artigos, palestras) de conhecimento descoberto (suas proprias conclusoes e reflexoes). Essa consciencia sobre a origem do conhecimento ajuda a avaliar a confiabilidade das suas ideias e entender como seu pensamento se formou ao longo do tempo.

#### Como usar

1. Ao criar uma nota derivada de uma fonte, adicione um link: `[[Fonte original|derives-from]]`
2. Cadeias de multiplos niveis podem ser construidas: nota ← derivada de ← derivada de ← fonte original
3. Classifique fontes externas adicionando `source-type: received` no frontmatter

#### Onde aparece?

- **Barra lateral direita**: A aba "Proveniencia" exibe a arvore genealogica completa
- **Vista Estelar**: Como direcao das setas nos links (da fonte para o derivado)
- **Propriedades da nota**: Classificacao como "recebido" ou "descoberto" com base na cadeia de proveniencia

### Configuracoes do Motor Cognitivo

Todas as ferramentas do Motor Cognitivo podem ser configuradas em **Configuracoes > Motor Cognitivo**:

- **Classificacao de estratos** — Ativar ou desativar a classificacao automatica
- **Rastreamento de maturidade** — Ativar ou desativar o rastreamento do ciclo de maturidade
- **Links tipados** — Ajustar o limiar de sensibilidade para deteccao de links (0.0 – 1.0)
- **Detector de tensoes** — Ativar ou desativar a deteccao automatica de tensoes
- **Substituicao manual** — Adicione propriedades `stratum` e `maturity` no frontmatter para substituir a classificacao automatica

---

*Manual do Usuario do Constellation — Versao 0.3.4 — Marco 2026*
*uconstellation.world*

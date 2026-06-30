# Manual do Usuario do Constellation

**Versao 0.1.0 | Marco 2026**

Constellation e um aplicativo de desktop para Gestao do Conhecimento Pessoal (PKM) que permite gerenciar bibliotecas de notas em Markdown. Desenvolvido com Tauri v2, SvelteKit e Rust, funciona nativamente no Windows, macOS e Linux com suporte completo para arabe e escrita RTL.

---

## Sumario

1. [Primeiros Passos](#primeiros-passos)
2. [Universo e Bibliotecas](#universo-e-bibliotecas)
3. [Criar e Editar Notas](#criar-e-editar-notas)
4. [Pesquisa](#pesquisa)
5. [Vista Estelar (GraphMind)](#vista-estelar-graphmind)
6. [Visualizacao Dividida](#visualizacao-dividida)
7. [Indice](#indice)
8. [Constellation Sight](#constellation-sight)
9. [Segunda Tela](#segunda-tela)
10. [Propriedades e Frontmatter](#propriedades-e-frontmatter)
10b. [Revisão de Origem (CECE)](#10b-revisão-de-origem-constellation-epistemic-content-engine--cece)
11. [Modelos](#modelos)
12. [Tabelas](#tabelas)
13. [Tarefas](#tarefas)
14. [Importador](#importador)
15. [Calendario](#calendario)
16. [Lens](#lens)
17. [Configuracoes](#configuracoes)
18. [Atalhos de Teclado](#atalhos-de-teclado)
19. [Suporte RTL e Arabe](#suporte-rtl-e-arabe)
20. [Seguranca e Privacidade](#seguranca-e-privacidade)
21. [Mapa do conhecimento](#mapa-do-conhecimento)
22. [Motor Cognitivo](#motor-cognitivo)

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

## 4. Pesquisa

Constellation inclui um motor de busca hibrido multilingue baseado em SQLite FTS5 com classificacao BM25, filtros de consulta estruturados e normalizacao otimizada para arabe. A pesquisa e acessivel pela barra lateral.

### Como pesquisar

Clique no icone de pesquisa na barra lateral ou use `Ctrl+Shift+F` para ativar o modo de pesquisa. Digite sua consulta e os resultados aparecem apos um breve atraso (300ms). Pressione `Escape` ou clique em `×` para limpar a pesquisa e voltar a arvore de arquivos.

### Sintaxe de pesquisa

| Sintaxe | Exemplo | O que encontra |
|---------|---------|----------------|
| Texto livre | `gestao de projetos` | Notas contendo essas palavras no titulo ou corpo |
| Filtro de tag | `#pesquisa` | Notas com a tag `#pesquisa` |
| Filtro de propriedade | `status=ativo` | Notas com propriedade frontmatter `status` igual a `ativo` |
| Filtro de wikilink | `links to [[Clima]]` | Notas com link para `[[Clima]]` |
| Escopo de biblioteca | `in:MinhaBiblioteca` | Restringe resultados a uma biblioteca especifica |
| Combinado | `#pesquisa status=ativo economia` | Todos os filtros aplicados juntos |

### Badges de tipo de correspondencia

Cada resultado exibe um badge colorido indicando como a correspondencia foi encontrada. O badge mostra uma letra localizada para acessibilidade (seguro para daltonicos):

| Badge | Cor | Significado |
|-------|-----|-------------|
| **T** | Azul | Correspondencia de titulo — o termo aparece no nome da nota |
| **C** | Verde | Correspondencia de conteudo — o termo aparece no corpo da nota |
| **S** | Roxo | Correspondencia semantica — conceitualmente relacionado (requer modelo de embedding) |
| **P** | Ambar | Correspondencia de propriedade — encontrado via filtro de propriedade frontmatter |
| **#** | Rosa | Correspondencia de tag — encontrado via filtro de tag |
| **W** | Azul claro | Correspondencia de wikilink — encontrado via filtro de wikilink |

As letras dos badges sao localizadas para todos os 15 idiomas suportados.

### Resultados fixados (Navegar entre resultados)

Os resultados permanecem visiveis apos clicar em um deles. A nota aberta e destacada na lista de resultados para que voce saiba qual resultado esta visualizando. Clique em outro resultado para navegar ate ele sem pesquisar novamente.

Para limpar a pesquisa, pressione `Escape` ou clique em `×`.

### Navegacao por teclado

| Tecla | Acao |
|-------|------|
| `Seta para baixo` | Selecionar proximo resultado |
| `Seta para cima` | Selecionar resultado anterior |
| `Enter` | Abrir resultado selecionado |
| `Escape` | Limpar pesquisa e voltar a arvore de arquivos |

### Destaque do termo de pesquisa

Ao abrir uma nota dos resultados, todas as ocorrencias do termo sao destacadas no editor. Funciona com deteccao de diacriticos arabes — pesquisar "ادارة" destacara "إدارة" e todas as variantes diacriticas.

### Historico de pesquisa

Clique no campo de pesquisa quando estiver vazio para ver suas pesquisas recentes (ultimas 20 consultas). Cada entrada mostra o texto da consulta e ha quanto tempo foi realizada. Clique em qualquer entrada para executar essa pesquisa novamente. Use o link "Limpar historico" na parte inferior para apagar todo o historico.

O historico de pesquisa e armazenado localmente no seu dispositivo e persiste entre reinicializacoes.

### Search Hub

O Search Hub e uma experiencia de pesquisa em tela cheia. Clique no icone da lupa na barra do dock para abri-lo. Ambas as barras laterais se recolhem para dar espaco maximo. Digite qualquer termo e o Constellation pesquisa em todos os lugares simultaneamente, agrupando resultados em 5 categorias: Titulos, Conteudos, Tags, Propriedades e Wikilinks. Cada categoria tem uma secao recolhivel com um badge de contagem. Clique em qualquer resultado para abri-lo no editor com todas as ocorrencias destacadas. Um botao "Voltar ao Search Hub" aparece para que voce possa voltar sem pesquisar novamente.

### Operadores de link

O Constellation suporta 6 operadores de pesquisa de topologia de links:

| Sintaxe | O que encontra |
|---------|----------------|
| `links to [[X]]` | Notas que linkam para X (backlinks) |
| `links from [[X]]` | Notas para as quais X linka (links de saida) |
| `mutual [[X]]` | Notas linkadas a X E X linka de volta (bidirecional) |
| `mentions [[X]]` | Notas contendo o nome de X sem um [[wikilink]] |
| `orphans` | Notas sem links de entrada ou saida |
| `links between [[X]] and [[Y]]` | Notas que linkam para X e Y |

Ao digitar qualquer operador de link, o autocomplete `[[` mostra todas as notas no universo. Apos selecionar uma nota, digite `#` para completar titulos ou `|type:` para completar o tipo de link.

---

## 5. Vista Estelar (GraphMind)

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

## 6. Visualizacao Dividida

A visualizacao dividida permite editar multiplas notas lado a lado na janela principal.

### Abrir a Visualizacao Dividida

- **Paleta de Comandos**: `Ctrl+P` e digite "Split View"
- **Atalho de teclado**: Use o atalho atribuido para alternar entre os modos
- **Ciclo**: Desativado → Vertical (lado a lado) → Horizontal (acima e abaixo) → Desativado

### Editar na Visualizacao Dividida

Cada painel e um editor totalmente independente com:
- Barra de ferramentas completa (negrito, italico, titulos, alinhamento, etc.)
- Navegacao por breadcrumb (biblioteca / nome da nota)
- Painel de propriedades e menu suspenso de estagio
- Suporte a salvamento (`Ctrl+S` salva o painel focado)
- Edicao de titulo e renomeacao de arquivo

### Redimensionar Paineis

Arraste o divisor entre os paineis para redimensiona-los. Cada divisor e independente — com 3 ou mais notas abertas, voce pode redimensionar qualquer par adjacente sem afetar os demais. Funciona nos modos vertical e horizontal.

### Foco

Clique em qualquer painel para foca-lo. O painel focado recebe os atalhos de teclado e e rastreado pelos paineis da barra lateral direita (Propriedades, Retrolinks, etc.).

---

## 7. Indice

O Indice e um glossario abrangente de termos de todas as suas bibliotecas — cada palavra significativa, ordenada alfabeticamente com contagens de ocorrencias.

### Abrir o Indice

- **Botao do dock**: Clique no icone do Indice (livro) no dock esquerdo
- **Paleta de Comandos**: `Ctrl+P` e digite "Index"

### Pipeline NLP Multilingue

O Indice processa o texto atraves de um pipeline consciente do idioma antes da indexacao:

- **Arabe**: Algoritmo Lucene Light10 — remove tashkeel, unifica hamza, remove artigo definido (الـ), remove sufixos gramaticais
- **Hebraico**: Remocao de prefixos (ב/ל/מ/ה/ו/כ/ש)
- **Ingles**: Stemming tipo Porter (plurais, formas verbais, sufixos)
- **Frances/Espanhol/Portugues/Alemao**: Remocao de sufixos especificos do idioma
- **Russo/Turco/Hindi/Persa**: Remocao de sufixos morfologicos
- **Todos os 15 idiomas**: Filtragem de palavras vazias (artigos, preposicoes, conjuncoes)

### Navegacao

- **Abas de idioma**: Alterne entre Todos, Arabe, Hebraico, Ingles ou # (caracteres especiais)
- **Barra alfabetica**: Clique em uma letra para filtrar termos que comecam com essa letra — a contagem de termos e atualizada para mostrar quantos correspondem
- **Clique na mesma letra novamente** para limpar o filtro e mostrar todos os termos
- **Modos de ordenacao**: Alfabetico (padrao) ou por frequencia (mais comuns primeiro)

### Editar a partir do Indice

Clique em qualquer nota nas referencias de um termo para abri-la em um painel de pre-visualizacao dividido ao lado do Indice. O painel de pre-visualizacao e um editor completo — voce pode editar, salvar, alterar propriedades e promover o estagio. O termo de pesquisa e destacado na nota e rolado automaticamente.

Pressione `Ctrl+Clique` para abrir a nota como uma aba regular. Um botao "Voltar ao Indice" aparece na barra de abas — clique nele para retornar exatamente onde voce parou no Indice.

### Integracao com a Segunda Tela

Quando a Segunda Tela esta aberta:
- **Clique em um termo** → A Segunda Tela mostra todas as notas contendo esse termo em uma visualizacao dividida (lista de notas + editor)
- **Ctrl+Clique em varios termos** → A Segunda Tela mostra o modo de comparacao com cada termo em sua propria coluna

---

## 8. Constellation Sight

O Constellation Sight visualiza todo o seu sistema de conhecimento como um grafico de poco gravitacional. Ele responde: **"Como e meu conhecimento e quao saudavel ele esta?"**

### Abrir o Sight

Clique no **botao Sight** (icone de olho) na faixa esquerda. O grafico de poco gravitacional aparece. Clique em x para fechar.

### O Grafico de Poco Gravitacional

As notas sao organizadas em aneis concentricos por importancia (centralidade). As notas mais conectadas ficam no centro; notas perifericas nas bordas. Dentro de cada anel, as notas sao agrupadas por biblioteca (sua organizacao). Cor do no = biblioteca.

| Elemento | Significado |
|----------|-------------|
| **No grande** | Alta centralidade — conecta diferentes areas de conhecimento |
| **No pequeno** | Periferico — dentro de uma area |
| **Cor do no** | Pertencimento a biblioteca |
| **Linha solida** | Link entre duas notas |
| **Setas de direcao** | Pequenas setas mostrando a direcao do link |
| **Espessura da linha** | Nivel de confianca (grossa = estabelecido, fina = hipotese) |

### Interacao

- **Clique simples** em um no: destaca sua vizinhanca (todas as notas conectadas). Todo o resto escurece.
- **Duplo clique**: abre a nota no editor.
- **Clique em espaco vazio**: limpa o destaque.
- **Rolagem**: zoom. **Arrastar**: panoramica. **Ajustar a tela**: botao da barra de ferramentas.

### Pesquisa no Sight

Clique na lupa. Suporta todos os operadores: `links to [[X]]`, `links from [[X]]`, `mutual [[X]]`, `orphans`, `supports [[X]]`, `contradicts [[X]]`, `#tag`, texto livre e pesquisa semantica. Os resultados mostram cores direcionais: verde (entrada), vermelho (saida).

### Painel de Analise (SightPanel)

Clique no icone de grade para abrir a barra lateral. Mostra: pontuacao de Saude do Universo (0-100), contadores de notas/links/orfaos, barras de tipo de link e confianca, top 10 pontes e Insights de Conhecimento (evidencia mais forte, fundacoes fracas, tensoes, estagnados, mais conectados, lacunas de conhecimento).

### Configuracoes

Icone de engrenagem: ajuste a espessura do traco do link, opacidade e tamanho da seta. As configuracoes persistem entre sessoes.

### 8a. Campos de tradicao por nota (MIG-029)

O chip de tradicao no canto superior esquerdo de Sight permite reenquadrar a cupula atraves de 24 tradicoes academicas em 10 familias epistemicas. Para nove dessas tradicoes (as de forma setorial / concentrica / em escada), cada nota pode ser **classificada explicitamente** via um campo no frontmatter. Notas sem o campo caem num cubo padrao sensato especifico da tradicao; notas COM o campo caem no cubo que voce nomeou.

Adicione o campo ao frontmatter YAML de uma nota:

```yaml
---
masadir_source: sunnah
---
```

Mude para o chip dessa tradicao → sua nota cairá no setor nomeado dela em vez do padrao.

**Campos permitidos e valores:**

| Tradicao | Campo frontmatter | Valores permitidos | Padrao se ausente |
|---|---|---|---|
| **masādir** (uṣūl al-fiqh sunita) | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` |
| **pramāṇa** (Nyāya indiano) | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` |
| **Burhān de Ibn Rushd** | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `shir` (anel mais externo) |
| **PaRDeS** (hermeneutica judaica) | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` |
| **Peirce** (3 categorias faneroscopicas) | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` |
| **Habermas** (3 interesses de conhecimento) | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` |
| **Brotos mencianos** (4 brotos morais) | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` |
| **Sān biǎo moísta** (3 padroes) | `mohist_zone` | `ben` / `yuan` / `yong` | distribuido por hash em 3 zonas |
| **Sŏngnihak coreano** (debate Quatro-Sete) | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` |

**Comportamento:**
- Se voce escrever um valor que a tradicao nao reconhece (erro de digitacao ou inventado), a nota cai no cubo padrao. Sem crash, sem falha de renderizacao.
- Mudancas no frontmatter propagam automaticamente — salve a nota → a proxima renderizacao da cupula refletira a mudanca.
- O mesmo campo so e lido pela tradicao com seu nome. Definir `masadir_source: sunnah` numa nota nao tem efeito quando voce muda para PaRDeS ou Peirce — cada tradicao le seu proprio campo de forma independente.
- Esta e a forma mais explicita de controlar a gramatica espacial da cupula. Sem esses campos, a geometria esta correta mas cada nota cai no mesmo cubo padrao; com eles, o chip torna-se analiticamente significativo.

**Tradicoes sem campos por nota** (atualmente agrupam todas as estrelas por outros meios — pasta / biblioteca / hash):

- Aristotélica (padrao, sem remapeamento)
- Polanyi (nevoa gradiente; sem setorizacao)
- Husserl, Longino, Maqāṣid de al-Shāṭibī, Profecia maimonideana, 13 middot talmúdicas, Wang Yangming, Pluriversal de Mignolo, Transmodernidade de Dussel, Maldonado-Torres, Akan de Wiredu, ʿUmrān de Ibn Khaldūn, Ibuanyidanda

(Migracoes futuras podem adicionar campos frontmatter por nota para estas a medida que a demanda dos usuarios surgir.)

---

## 9. Segunda Tela

A Segunda Tela e uma janela complementar baseada em modos que se adapta ao modo atual da sua barra lateral.

- **Abrir**: Clique no icone de segunda tela na barra lateral, ou `Ctrl+Shift+2`
- **Fechamento automatico**: Quando voce fecha a janela principal, a segunda tela fecha automaticamente

### Complemento Baseado em Modos

A segunda tela altera seu conteudo com base no modo ativo da barra lateral na janela principal:

| Modo da Barra Lateral | A Segunda Tela Mostra |
|---|---|
| **Explorador de Arquivos** | Painel do Universo — estatisticas, detalhamento de bibliotecas, universos secundarios, tags, notas editadas/abertas recentemente |
| **Navegador** | Visualizacao completa do Navegador para navegar notas |
| **Vista do Ceu** | Arvore da Vista do Ceu com estrutura de diretorios |
| **Vista Estelar** | Complemento da Vista Estelar com retrolinks, links para frente, tags e grafo local |

### Painel do Universo (Modo Explorador de Arquivos)

Quando a janela principal esta no modo Explorador de Arquivos, a segunda tela exibe um painel com:

- **Cartoes de estatisticas** — Nome do universo, contagem de universos secundarios, total de bibliotecas, pastas e notas
- **Universos Secundarios** — Cada universo secundario com suas bibliotecas vinculadas e contagens de pastas/notas
- **Bibliotecas** — Cada biblioteca com contagens de pastas/notas em caixas de estatisticas codificadas por cores
- **Editadas Recentemente** — Notas que voce modificou na sessao atual (rastreadas ao salvar)
- **Abertas Recentemente** — Notas que voce abriu mas nao editou na sessao atual
- **Tags** — Todas as tags de todas as bibliotecas ordenadas por quantidade; clique em uma tag para ver todas as notas que a utilizam

### Interacao do painel

Quando o painel esta ativo na janela principal, clicar nos itens os envia para a segunda tela:

- **Editadas/Abertas Recentemente**: Clique em uma nota para abri-la como editor completo na segunda tela
- **Tags**: Clique em uma tag para mostrar todas as notas que a utilizam em uma visualizacao dividida — lista de notas a esquerda, editor completo a direita

Todas as edicoes na segunda tela sao sincronizadas automaticamente com a janela principal.

### Edicao de Notas na Segunda Tela

A segunda tela suporta edicao completa de notas — digite, salve, renomeie e altere propriedades da mesma forma que na janela principal. As alteracoes sao sincronizadas automaticamente com a janela principal.

### Sincronizacao de Configuracoes

Todas as configuracoes visuais se propagam instantaneamente para a segunda tela — sem necessidade de reiniciar:

- **Idioma**: Alteracoes de idioma da interface se aplicam imediatamente
- **Tema**: O modo claro/escuro/sistema alterna instantaneamente
- **Fontes**: Fonte de interface, fonte de texto, fonte monoespaco e fontes especificas por script
- **Tamanho da fonte**: Tamanhos de fonte de interface e editor
- **Editor**: Comprimento de linha legivel, numeros de linha, barra de ferramentas flutuante
- **Cor de destaque**: Alteracoes na cor de destaque do tema

### Estrutura (links estruturais)

O painel **Estrutura** mostra onde a nota aberta se situa dentro de uma *obra* maior — um livro, um argumento, um curso, um Mapa de Conteúdo. Responde a uma pergunta diferente da dos painéis Backlinks e Links de Saída. Esses respondem *"como é que esta ideia se relaciona com outra ideia?"* (os links de pensamento — apoia, contradiz, causa…). A Estrutura responde *"onde é que esta nota se situa na obra inteira que estou a compor?"* — Livro → Parte → Capítulo → Cena.

Esta é a **espinha compositiva** de uma obra: o índice, o esquema ordenado. É mantida deliberadamente **fora** de toda medida de pensamento, maturidade e conexão — colocar uma nota "sob um Livro" nunca altera a maturidade dessa nota, as suas contagens de conexões nem a sua presença na Vista Estelar. Um índice é autoria, não uma afirmação a ser julgada.

**Os dois tipos de link estrutural** (você só digita um lado — o Constellation deduz o inverso por você):

- **`parent`** — o lugar *desta nota* sob um único pai (por exemplo, um capítulo declara a parte a que pertence).
- **`contains`** — a lista ordenada de filhos *desta nota* (por exemplo, um livro lista as suas partes na ordem de leitura).

**Criar um link estrutural** — abra as **Propriedades** da nota (a aba Propriedades na barra lateral direita, ou o bloco de propriedades no topo da nota):

1. Clique em **+ Adicionar propriedade** e digite a chave `parent` ou `contains`.
2. No valor, digite o **nome da nota de destino** — apenas o nome, por exemplo `Part I - The Cartographer`. O Constellation envolve-o num `[[link]]` por você; você **não** digita os colchetes. (Se colar um nome que já tenha colchetes, ele ainda é armazenado de forma limpa como um único `[[name]]` — nunca com colchetes duplos.)
3. Para `contains`, adicione cada filho como o seu próprio chip, na ordem em que quer que sejam lidos — essa ordem torna-se a ordem do esquema.

Os links estruturais **renomeiam-se com segurança**: renomeie um capítulo e o seu lugar na estrutura acompanha automaticamente, porque o link aponta para a nota, não para um trecho de texto congelado.

**Ler o painel Estrutura** — abra a aba **Estrutura** na barra lateral direita (logo após Backlinks):

- O painel mostra a **obra inteira** como um esquema indentado (marcadores em verde-azulado), encabeçado por **OUTLINE** com uma contagem dos descendentes — não apenas os filhos da própria nota aberta.
- A nota que está a visualizar atualmente é **realçada** ("você está aqui") dentro desse esquema.
- Um **breadcrumb** ao longo do topo mostra o caminho ao longo da espinha (por exemplo, *The Atlas of Lost Places › Part I › Chapter 1*). Clique em qualquer migalha — ou em qualquer linha do esquema — para saltar para essa nota.
- Um alternador **Whole work ⇄ This note** (no canto superior direito do painel) alterna entre a obra inteira e apenas a subárvore da própria nota aberta. Só aparece quando a nota tem de facto um pai, de modo que as duas vistas sejam diferentes.
- Se a estrutura acidentalmente formar um ciclo sobre si mesma (o pai da nota A é B, e o pai de B é A), o esquema desenha a cadeia e depois para de forma limpa, marcando o ponto de corte com um pequeno **↻**. Nunca trava.

**Resolver um conflito (Contested).** Se duas notas reivindicarem o mesmo filho — uma através do próprio `parent` do filho, a outra através de uma lista `contains` — o painel sinaliza essa linha como **Contested** (um selo âmbar ⚠ nomeando o outro reivindicante) em vez de descartá-la silenciosamente. Dois botões de um clique resolvem-no:

- **Keep** — manter o pai declarado do próprio filho (esta nota abdica da sua reivindicação sobre o filho).
- **Move here** — aceitar esta nota como o pai (o `parent` do filho passa para esta nota).

Qualquer um dos botões atualiza os ficheiros das notas diretamente e atualiza o esquema. Nada é jamais alterado sem o seu clique.

---

## 10. Propriedades e Frontmatter

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

## 10b. Revisão de Origem (Constellation Epistemic Content Engine — CECE)

> *(Nota de tradução: tradução gerada por IA do capítulo V3-§10.F; revisão por falante nativo pendente.)*

Duas das propriedades de frontmatter mais importantes — `sources:` e `content_type:` — descrevem *como você veio a saber* algo e *que tipo de conhecimento* é. O **Epistemic Content Engine** (CECE) do Constellation classifica cada nota ao longo destes dois eixos automaticamente usando um conjunto de 6 catalogadores. O painel **Revisão de Origem** é onde você revisa e corrige essas classificações.

### O que o motor faz

Quando você classifica uma nota (clique direito → «Sugerir origem e tipo de conteúdo», ou via Configurações > Executar varredura, ou automaticamente via o interruptor de varredura em segundo plano), o CECE executa seis catalogadores independentes contra a nota. Cada um lê a nota através de uma lente diferente e vota em duas perguntas:

- **Origem** (eixo horizontal) — de onde *veio* este conhecimento? Onze valores possíveis: percepção, inferência, testemunho, transmissão-em-massa, comparação, postulação, não-apreensão, memória, disposição-inata, inspiração, revelação. Mais *não classificável*.
- **Tipo de conteúdo** (eixo vertical) — que *espécie* de conhecimento é este? Cinco ramos principais: entradas sensoriais, entidades simbólicas, conteúdos semânticos, estados epistêmicos, construtos de ordem superior.

Os dois eixos são independentes. Uma nota «Eu duvido do pouso na lua» é testemunho (alguém relatou) na origem + estados-epistêmicos/dúvida (sua postura) no tipo de conteúdo.

O motor roda **no seu dispositivo** — nenhuma nota jamais sai do Constellation.

### Os seis catalogadores

Cada catalogador é uma lente. O cartão de Revisão de Origem os mostra como seis pequenos pontos coloridos no canto superior direito de cada cartão:

- **Seu frontmatter** (azul) — adota o que você já definiu, com autoridade absoluta
- **Citações e estrutura** (rosa) — citações, citações em bloco, marcadores de teorema, frases de definição
- **Radicais e léxico** (âmbar) — análise de raízes árabes + equivalência de termos entre idiomas
- **Notas vinculadas** (verde-azulado) — Living Links tipados a outras notas classificadas
- **Notas semelhantes** (violeta) — similaridade por embeddings com suas notas já classificadas
- **Julgamento de IA** (verde) — um LLM local (Qwen3-4B; *ainda não ativo*, adiado para uma versão futura)

Um ponto preenchido significa que esse catalogador se manifestou e concorda com a síntese. Um ponto com anel significa que se manifestou mas discordou. Um ponto com contorno tracejado significa que ficou em silêncio (sem sinal nesta lente).

### Três regimes de confiança

Depois que os catalogadores votam, cada eixo cai em um de três regimes:

- **Unânime** — todos os catalogadores que se manifestaram concordaram
- **Maioria forte (uma divergência)** — a maioria concordou; um divergente identificado
- **Dividido** — sem maioria clara; o motor se recusa a adivinhar e pede que você escolha

Cada eixo recebe seu próprio regime independentemente — um cartão pode ser Unânime no horizontal + Dividido no vertical, etc.

### Sibling Disambiguation

Quando um eixo está Dividido, o motor exibe os valores candidatos como **chips** sob um aviso: *«Escolha qual se ajusta melhor à nota.»* Clique em um chip → o motor escreve essa escolha no frontmatter da nota e remove o cartão da fila. Se o OUTRO eixo estava resolvido (Unânime ou Maioria forte), o motor *também* escreve o valor desse eixo ao mesmo tempo — um clique finaliza ambos os eixos quando apenas um estava Dividido.

### A trilha de raciocínio

Cada cartão tem um interruptor *«▸ Por que esta classificação?»*. Ao expandi-lo, mostra-se uma linha por catalogador que se manifestou com o raciocínio, a confiança autorrelatada e chips de regra amigáveis («Correspondência de palavra-chave de superfície», «Correspondência de raiz árabe (CAE)», «Marcador de definição», etc.) — estas são as regras específicas que cada catalogador acionou.

Durante suas **primeiras 50 revisões**, a trilha se expande automaticamente em cada cartão (um *período de calibração de confiança*) para que você possa desenvolver intuição sobre quando confiar no motor. Depois disso, as trilhas se recolhem para sob demanda em cartões Unânimes. Substitua a qualquer momento em **Configurações > Inteligência > CECE > Visibilidade da trilha de raciocínio**.

### O filtro de composição da fila

Acima da barra de contagem, cinco chips fatiam a fila pelo tipo de decisão que cada cartão precisa:

- **Todos** — a fila completa
- **Ambos os eixos precisam da sua decisão** — ambos os eixos Divididos
- **Origem precisa da sua decisão** — horizontal Dividido + vertical resolvido
- **Tipo de conteúdo precisa da sua decisão** — vertical Dividido + horizontal resolvido
- **Catalogadores concordaram** — nenhum eixo Dividido (candidatos a carimbo automático)

Cada chip mostra sua contagem de balde. O filtro é um fatiador de camada de renderização — a matemática de Aceitar Tudo sempre opera sobre a fila completa, independentemente de qual filtro esteja ativo.

### Ações por cartão

- **Aceitar** — escreve a síntese do motor como primária em ambos os eixos; remove o cartão. Atualiza a confiabilidade por catalogador.
- **Editar** — abre um seletor de árvore para ambos os eixos; escolha manualmente. Mesma atualização de confiabilidade.
- **Rejeitar** — limpa o cartão sem escrever.
- **Chip de Sibling Disambiguation** — apenas em cartões Divididos.

### Calibração por Biblioteca

**Configurações > Inteligência > CECE > Calibração por Biblioteca** abre uma tabela somente leitura mostrando a precisão de cada catalogador por eixo na Biblioteca ativa. Diferentes Bibliotecas têm precisões por catalogador diferentes — Linguístico se sobressai em Bibliotecas com muito árabe, Grafo se sobressai nas densamente vinculadas. A camada de síntese usa esses dados de calibração para ponderar votos.

Um catalogador precisa de **20 correções** antes que sua razão de precisão seja mostrada. Abaixo desse limite, o rótulo lê *«(uniforme)»* — o catalogador contribui com votos de peso uniforme até que dados suficientes se acumulem.

### Classificação em segundo plano

Por padrão, o CECE classifica notas apenas quando você pede (clique direito ou botão de varredura nas Configurações). Você pode optar pela classificação automática em **Configurações > Inteligência > CECE > Classificação em segundo plano**:

- **Ao salvar a nota** — classifica cada nota ~1,5 segundos depois que você para de digitar (cavalga sobre o salvamento debounced existente; nunca dispara por tecla pressionada; a digitação permanece instantânea)
- **Ao iniciar o aplicativo** — escaneia notas não classificadas uma vez por lançamento

### O Classificador — o lar de página inteira

Os mesmos cartões também vivem em uma visão de página inteira chamada **Classificador**, aberta a partir do **ícone de cartões empilhados na doca à esquerda**. É o mesmo motor e a mesma fila, recebendo a janela inteira em vez de uma aba estreita da barra lateral — e adiciona dois controles que a aba da barra lateral nunca teve:

- **Classificar uma nota…** — uma caixa de busca que permite classificar *qualquer* nota pelo nome, sem abri-la primeiro. Digite algumas letras, escolha a nota, e um cartão novo aparece na fila.
- **Gerar todos os resumos** — pré-computa o resumo da nota (veja abaixo) para cada nota que não tem um, em segundo plano, com progresso na barra de status.

Um botão **Iniciar varredura** (a mesma varredura de todo o universo que as Configurações) e uma faixa de progresso ao vivo completam o cabeçalho. Feche o Classificador com o **(×)** ou **Esc**. (Quando a caixa de busca *Classificar uma nota…* está aberta, o primeiro **Esc** fecha apenas essa caixa.)

Uma nota sobre nomenclatura: **o Classificador** é a *sala* (a visão de página inteira); **os catalogadores** são as *seis lentes* dentro do motor que votam em cada cartão. Não confunda os dois.

### Resumos das notas

Sob o título de cada cartão fica um breve **Resumo** — algumas frases que dizem do que a nota trata, para que você possa classificá-la sem abri-la. O Constellation sempre prefere um resumo que *você* escreveu e só gera um quando você não escreveu:

1. Um **campo de frontmatter** `summary:` / `description:` / `abstract:` / `excerpt:`, usado literalmente.
2. Um **callout** `> [!summary]` / `[!abstract]` / `[!tldr]` no corpo, usado literalmente.
3. Caso contrário, um resumo **gerado** — as três frases mais centrais da nota, extraídas (nunca inventadas) e mostradas na ordem original.

Os resumos gerados são **somente leitura** — o Constellation nunca escreve um de volta na sua nota (File-Over-App), e tudo é computado **no seu dispositivo**. Se você quer que um resumo viva no arquivo, escreva um você mesmo e o Constellation mostrará o seu em vez disso.

Para mais detalhes (cada status de ponto, cada chip de regra, passos clique a clique de cenários comuns), consulte os tópicos **Revisão de Origem**, **Classificador** e **Resumos das Notas** no sistema de ajuda.

---

## 10c. Metadados Epistêmicos

Um pequeno conjunto de campos opcionais de frontmatter para registrar informações mais ricas sobre como o conhecimento de uma nota foi adquirido, quem sustenta a posição, a que disciplina pertence e quando você revisou pela última vez sua visão. Adicionado em MIG-022 §A em resposta à análise de lacunas (`docs/epistemic-content-gap-analysis.md`).

Estes campos são **todos opcionais**. Notas sem eles funcionam sem alterações.

### Referência rápida

| Field | Type | Purpose |
|---|---|---|
| `held_by` | text | De quem é esta posição? (padrão `user`; pode ser `"al-Shāfiʿī"`, `"Ḥanafī"`, etc.) |
| `domain` | list | Etiquetas disciplinares para recuperação (`[fiqh, ʿibādāt]`) |
| `function` | text | Para que serve esta nota (`reference` / `seed` / `actionable` / `shipped`) |
| `provenance_civilization` | text | Vocabulário tradicional (`sunni-usuli` / `analytic-western` / `nyaya` / etc.) |
| `updated_at` | date | Quando você revisou deliberadamente sua visão pela última vez (distinto do mtime do sistema de arquivos) |
| `ikhtilāf` | list of objects | Desacordo erudito estruturado (`[{school, position}, ...]`) |
| `warrant` | text | Rótulo de grau (analisado mas inerte até que o Warrant Research workstream seja entregue) |
| `warrant_notes` | text | Texto livre que sustenta o grau de garantia (também inerte) |

### Como aparecem no painel de Propriedades

Cada campo é renderizado com o editor apropriado ao tipo:
- Campos de texto → entrada de texto
- `domain` → lista de etiquetas (Enter para adicionar, × para remover)
- `updated_at` → seletor de data
- **`ikhtilāf` → widget personalizado** com duas entradas lado a lado por linha (school + position) mais um botão de remover por linha, e um botão "Adicionar escola" na parte inferior. O widget lê de e escreve para o YAML estruturado, então as viagens de ida e volta preservam cada campo.

### E quanto a `supersedes`?

`supersedes` é uma *relação entre notas* (esta nota substitui uma anterior), não uma propriedade de uma única nota. O Constellation o trata como um **link tipado**, não como um escalar YAML:

```markdown
Isto substitui minha análise anterior: [[old-note-id|supersedes]]
```

O sufixo `|supersedes` no wikilink o torna um link tipado do tipo `supersedes` — pílula azul-cinza ardósia distinta, aparece nos painéis Backlinks + Outgoing Links, participa da Living Link Architecture.

### O que isto NÃO é

Os novos campos são **esquema** — um vocabulário reconhecido que você pode preencher. O CECE atualmente não os consome para a classificação. MIGs futuros (Warrant Research workstream, MIG-023 eixo temporal) entregarão recursos que leem `warrant`, `updated_at` e companhia.

Para mais detalhes + um exemplo trabalhado, consulte o tópico **Metadados Epistêmicos** no sistema de ajuda.

---

## 11. Modelos

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

## 12. Tabelas

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

## 13. Tarefas

Constellation suporta caixas de selecao de tarefas nas notas:

```markdown
- [ ] Tarefa incompleta
- [x] Tarefa concluida
```

No modo de Pre-visualizacao ao Vivo, as caixas de selecao sao clicaveis. As tarefas podem ser buscadas e filtradas em todas as suas bibliotecas.

---

## 14. Importador

Importe notas de outras ferramentas PKM:

- **Obsidian** — importa vaults com compatibilidade completa de wikilinks
- **Pastas Markdown** — importe qualquer pasta de arquivos `.md`
- **Outros formatos** — HTML, arquivos de texto

Va para **Configuracoes > Importador** para iniciar uma importacao.

---

## 15. Calendario

O **Calendário** é uma visualização mensal em página inteira, aberta a partir da **doca esquerda** (o ícone de calendário). Os dias com notas ou tarefas pendentes são assinalados com **pontos** coloridos. O cabeçalho mostra o mês no calendário que escolheu; se tiver definido um **calendário secundário**, um subtítulo por baixo mostra o intervalo equivalente nesse calendário (por exemplo, um mês gregoriano mostra o seu período em Hijri, "Dhul-Hijjah 1447 – Muharram 1448 AH").

**Clicar num dia.** Cada célula de dia é interativa:

- **Clicar no espaço vazio (ou no número do dia)** → abre (ou cria) a **nota diária** desse dia. Clicar numa data que já tem uma nota diária simplesmente a **abre** — nunca cria um duplicado.
- **Clicar num ponto** → abre esse item específico. Um ponto **dourado** é a nota diária; um ponto **roxo** é outra nota editada nesse dia; um ponto **vermelho** é uma tarefa pendente nesse dia. (As cores são personalizáveis no Style Setter → Calendário.) Se um dia tiver várias notas ou tarefas, clicar no ponto mostra uma pequena **lista** para escolher.
- **Clicar num ponto de tarefa** → abre a nota **com o ecrã posicionado na linha dessa tarefa**, pronta a editar. Na lista de tarefas pode também **marcar a caixa de uma tarefa para a concluir** diretamente a partir do calendário — as tarefas concluídas desaparecem de imediato. Só as tarefas que têm a sua própria data `📅 YYYY-MM-DD` aparecem no calendário (é a data que as coloca num dia).

**Calendários culturais (oito).** Em **Definições → Calendário** pode definir o **sistema de calendário** — **Gregoriano, Hijri (Islâmico), Hijri Solar (Persa), Hebraico, Indiano (Saka), Budista, Chinês ou Coreano** — e toda a grelha mensal muda para esse calendário, mostrando em cada célula tanto a data do calendário escolhido (grande) como a data gregoriana (pequena), além da fase da lua. Cada cabeçalho de mês mostra o **nome do mês, o seu número entre parênteses e o ano** — o número ajuda nos calendários cuja ordem de meses é pouco familiar. Os calendários **Chinês e Coreano** são *lunissolares*: por vezes inserem um **mês intercalar** (闰六月 / 윤6월), que o calendário mostra como uma página própria, para que a navegação nunca o salte nem o duplique. O calendário Hijri usa um motor astronómico preciso; os meses sagrados são realçados e os eventos islâmicos são assinalados. Pode também escolher o **início da semana** (domingo/segunda-feira) e ativar a **coluna do número da semana**.

**Opções do calendário Hijri.** Em **Definições → Calendário → "Hijri calendar (Islamic)"** existem dois controlos adicionais:

- **Método de cálculo** — **Astronómico (Conjunção Lunar)**, que segue a verdadeira lua nova (o mais preciso, e a predefinição), ou **Tabular (al-Tawfīqāt al-Ilhāmiyyah)** (o ciclo aritmético clássico).
- **Correção do mês** — ajuste o início de um mês Hijri em ±1 ou ±2 dias para corresponder a um **avistamento local da lua**. Escolha o ano e o mês Hijri, selecione um deslocamento e clique em **Set**; a correção aplica-se a esse mês e a todos os meses seguintes. As suas correções são listadas (cada uma removível), com um botão **Clear all**.

Ambas as definições (e as suas correções) são guardadas **com o seu universo**, pelo que acompanham os seus dispositivos.

**Opções de exibição Chinesa e Coreana.** A Coreia usa o calendário lunar chinês, por isso os dois partilham datas idênticas — o que os distingue é a escrita e o ano. Quando qualquer um deles é o seu calendário principal ou secundário, **Definições → Calendário** mostra dois controlos adicionais: uma **exibição do ano** (Chinês: o ciclo sexagesimal 丙午年, o ano simples, ou ambos; Coreano: a era **Dangi** 단기 4359, o ano, ou o sexagesimal 병오년) e os **nomes dos meses** — *escrita nativa* (五月 / 5월) ou *fonética*, a pronúncia do mês escrita na sua própria língua (Português "Wǔyuè / Owol"; Árabe "وُو-يوي / أوه-وُل").

**Estilizar o calendário.** Abra o **Style Setter** (doca esquerda, ou **Definições → Style Setter**) e escolha a superfície **Calendário** para reestilizar cada parte — cada elemento tem a sua própria **cor e tamanho de texto** (números dos dias, a data de referência cruzada, a pílula do mês, os cabeçalhos dos dias da semana, os números das semanas, o glifo da lua, o destaque de Hoje, as linhas da grelha e os pontos de nota/tarefa/evento), além da **fonte** do calendário. Uma pré-visualização ao vivo, em tamanho real, atualiza-se à medida que edita; clique em **Keep** para aplicar.

> **Os nomes dos ficheiros das notas diárias permanecem sempre gregorianos** (`YYYY-MM-DD`), independentemente do calendário exibido — para que os seus ficheiros se mantenham portáteis e ordenem corretamente. A data cultural é mostrada no calendário (e pode ser registada no frontmatter da nota).

O Calendário serve plenamente as notas diárias: clique em qualquer dia para a abrir, ou execute o comando **"Daily Note"** (paleta de comandos) para saltar para hoje.

**Registar uma data cultural numa nota.** Duas ferramentas opcionais escrevem a data cultural nas **propriedades** de uma nota (o nome do ficheiro permanece sempre gregoriano `YYYY-MM-DD`):

- **Carimbo Hijri da nota diária** — *Definições → Calendário → "Stamp the Hijri date in daily notes."* Quando ativo (disponível apenas enquanto o calendário Hijri for o seu **principal ou secundário**), cada **nova** nota diária recebe uma linha `hijri:`, por exemplo `hijri: 1448-01-06`. As notas que já tem nunca são alteradas.
- **"+ Hijri" nas Propriedades de uma nota** — abra as **Propriedades** de qualquer nota, passe o rato sobre a data e aparece um pequeno botão **"+ Hijri"** (além de "+ Jalali", "+ Hebrew" e assim por diante — **um botão por cada calendário não-gregoriano que tenha selecionado**). Clique nele e o Constellation lê a data gregoriana da nota e adiciona o equivalente, por exemplo `jalali: 1405-03-30`. O botão coreano escreve o ano **Dangi**; um **mês intercalar** chinês/coreano é marcado com um `L` (por exemplo `chinese: 2025-06L-17`). Se a nota não tiver propriedade de data, usa a data de criação do ficheiro.

---

## 16. Lens e Constellation Base

Uma **Lens** e uma consulta salva que exibe uma lista filtrada e ordenada de notas junto com as propriedades de seu interesse. Constellation oferece dois modos:

### Constellation Base — blocos Lens incorporados

Voce pode incorporar uma Lens diretamente no corpo de qualquer nota Markdown usando um bloco de codigo ` ```base `:

````markdown
```base
schema: 1
view: list
dimensions: [note.name, note.created_at]
sort: [note.created_at, desc]
limit: 20
```
````

Ao visualizar a nota, o bloco de codigo e substituido por uma tabela interativa exibindo as notas correspondentes. Na visualizacao ao vivo, clique no chip **Lens** para editar o bloco.

**Dimensoes disponiveis na v1:** `note.name`, `note.path`, `note.created_at`, `note.headline`.

**Federacao:** por padrao, blocos Lens leem do universo ativo E de cada cUniverso vinculado. Defina `federation: active` no YAML para limitar ao universo ativo.

### Cinco Atos (Five Acts) — Lenses integradas

A secao **Five Acts** da barra lateral (acima de Workspace Bases) lista notas hospedeiras curadas pelo Constellation em `{universe}/Five Acts/*.md`. v1 inclui uma: **Observation — Recent Captures** (lista federada das 20 notas mais recentemente capturadas). Voce pode editar essas notas livremente — Constellation nao sobrescrevera suas alteracoes.

### Painel Lens classico

O painel Lens antigo (filtrar por etiquetas, pastas, propriedades) continua disponivel em **Configuracoes → Paineis → Lens**.

---

## 17. Configuracoes

Acesse as Configuracoes pelo icone de engrenagem na barra lateral ou `Ctrl+,`.

### Geral

- Idioma (15 idiomas)
- Tema (Claro / Escuro)
- Fonte da interface, Fonte de texto, Fonte monoespcada, Tamanho da fonte
- Tema de fonte — combinacoes de fontes predefinidas (Maquina de escrever, Classico, Moderno, etc.) para troca rapida
- **Temas** — escolha entre seis temas integrados, crie temas personalizados (editor de cinco cores), importe temas do registro da comunidade do Obsidian (200+ temas), ou importe um arquivo de tema `.json`. Exclua qualquer tema personalizado com o botao ✕ ao passar o mouse.

### Style Settings

Uma aba dedicada para personalizacao detalhada de cada elemento visivel da interface, aplicada ao vivo ao tema ativo.

- **Cores** — fundo, superficies, texto (normal/atenuado/fraco), acento, bordas, cores de estado
- **Tipografia** — tamanhos de fonte interface/nota/codigo, tamanhos H1–H6, peso de titulos, alturas de linha, espacamento entre paragrafos
- **Layout e Forma** — raios de canto pequeno/medio/grande, larguras de borda, sombras, comprimento de linha legivel do editor, margens laterais
- **Componentes** — dock de faixa, barra de acoes lateral, barra de layout (alternadores de painel), barra superior/faixa de abas, barra de status, barra lateral direita (inspetor), explorador de arquivos (notas do Universo, universos filhos, bibliotecas, pastas, notas), botoes, tags, callouts — cada um com tamanho, raio, cor independentes, e estilo de estado ativo quando aplicavel
- **Editor** — cor/hover/decoracao do link, cor/fundo/raio do codigo em linha, largura/cor da barra de citacao, cor do cursor, fundo de selecao

**Importar / Exportar** — barra de ferramentas no topo da aba:
- Colar da area de transferencia (um clique)
- Importar / Colar (area de texto com Mesclar ou Substituir)
- De arquivo (.json)
- Copiar (valores atuais para area de transferencia)
- Exportar (.json)

O formato corresponde exatamente ao plugin Style Settings do Obsidian, entao voce pode compartilhar ajustes entre Obsidian e Constellation.

As alteracoes sao salvas automaticamente no tema ativo; se voce editar um tema integrado, ele e clonado automaticamente em seus temas personalizados para que as mudancas persistam sem modificar o original.

### O Configurador de Estilo

O **Configurador de Estilo** (Style Setter) e um estudio de design em tela cheia — abra-o em **Configuracoes → Aparencia → "✦ Open Style Setter."** Ele mostra a sua interface real no centro; clique em qualquer parte (barra lateral, titulo da nota, titulo, link, a pagina da nota) e os controles desse elemento aparecem a direita, com a pre-visualizacao se atualizando instantaneamente. Os cartoes de tema (Midnight / Daylight / Chocolate / Nord) semeiam todo um visual — o proprio estudio o veste enquanto voce desenha — e a lista de *Superficies* pre-visualiza o visual em todo o aplicativo, nao apenas no editor. **"Apply to app"** aplica o seu acento, fundos, cor de texto e fontes ao Constellation real; **Esc** ou **✕** fecha apenas o Configurador, nao as Configuracoes. Por enquanto, aplicar e uma pre-visualizacao ao vivo da sessao — salvar um visual como um Estilo permanente e nomeado (com amostras de cor reutilizaveis e renomeaveis, alem de exportacao / importacao) vem a seguir.

### Substituicoes do motor arabe

Um painel por Universo onde voce fixa como o motor arabe analisa certas formas de superficie — as suas proprias cunhagens, nomes locais, emprestimos especificos do seu campo, ou casos em que voce discorda da leitura automatica do motor. Cada substituicao vence o FST generativo, a cascata e o recuo heuristico. Adicionar ou remover uma substituicao dispara uma reindexacao focada apenas nas notas que contem a superficie afetada — sem reconstrucao completa. Veja a secao 19 ("Suporte RTL e Arabe") para o passo a passo.

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

## 18. Atalhos de Teclado

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

## 19. Suporte RTL e Arabe

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

### Substituicoes do motor arabe

O motor arabe do Constellation e um analisador morfologico de cinco camadas que corre sob cada busca, cada link e cada entrada do indice. Ele entende raizes, padroes, nomes proprios, emprestimos e reparos fonologicos — de modo que uma consulta por كاتب encontra كتبنا e كتاب, mas وائل permanece intacto como nome proprio em vez de ser mutilado para ائل.

O painel **Substituicoes do arabe** nas Configuracoes e onde voce ensina a sua propria terminologia ao motor. Cada substituicao e a resposta soberana — ela vence o FST generativo, a cascata e o recuo heuristico.

**Quando usar substituicoes:**
- Nomes de pessoas, toponimos locais ou termos especificos do seu campo que o motor nao conhece
- Cunhagens ou siglas proprias do seu Universo
- Emprestimos em que voce quer preservar uma grafia especifica
- Qualquer caso em que a analise automatica do motor diverge da sua forma de ler a palavra

**Passo a passo:**

1. Abra **Configuracoes** (icone de engrenagem ou `Ctrl + ,` / `Cmd + ,`) e selecione **Substituicoes do arabe** na barra lateral.
2. Clique em **Adicionar substituicao**.
3. Preencha:
   - **Forma de superficie** — a palavra arabe como voce a digita
   - **Lema** — a forma canonica que o motor deve retornar
   - **Raiz** (opcional) — 3 ou 4 consoantes se a palavra tiver raiz classica
   - **Padrao** (opcional) — p. ex. `فاعل`
   - **Categoria** — Nome proprio / Substantivo / Adjetivo / Adverbio / Verbo / Particula / Estrangeiro / Desconhecido
   - **Nota** (opcional) — uma linha de contexto para voce mesmo
4. Clique em **Guardar**. O painel mostra **A reindexar…** enquanto cada nota que contem a superficie e retokenizada e, ao concluir, **N nota(s) reindexada(s)**.
5. Para remover uma substituicao, clique no **x** da sua linha — a mesma varredura de reindexacao corre ao contrario.

As substituicoes sao guardadas por Universo em `<universo>/.constellation/arabic-overrides.json` — texto simples, ordenado alfabeticamente, escrita atomica. Voce pode coloca-lo sob controle de versao ou compartilha-lo entre dispositivos.

---

## 20. Seguranca e Privacidade

- **Todos os dados permanecem locais** — sem sincronizacao na nuvem, sem telemetria, sem rastreamento
- **Arquivos Markdown** — suas notas sao arquivos de texto simples que pertencem totalmente a voce
- **Sem conta necessaria** — Constellation funciona completamente offline
- **Atualizacoes opcionais** — verifique atualizacoes manualmente nas Configuracoes
- **Codigo aberto** — inspecione o codigo em [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 21. Mapa do conhecimento

O Mapa do conhecimento e uma visualizacao radial (sunburst) que mostra a estrutura, densidade e maturidade de todo o seu universo de conhecimento.

### Abrir o Mapa

- **Botao do dock**: Clique no icone do Mapa do conhecimento na barra lateral esquerda
- **Paleta de comandos**: `Ctrl+P` e digite "Constellation Map"

### O que voce ve

- **Centro**: Nome do seu Universo com total de notas e palavras
- **Primeiro anel**: Bibliotecas (cada uma com sua cor). Se seu universo tem universos filhos, eles aparecem aqui tambem.
- **Aneis mais profundos**: Pastas e subpastas dentro de cada biblioteca
- **Segmentos externos**: Notas individuais

### Modos de cor

Alterne entre tres modos pelo menu:
- **Maturidade**: semente (cinza) → muda (verde claro) → perene (verde) → canonico (dourado) → murcho
- **Estrato**: L1 (azul) → L8 (vermelho) — mostra complexidade do conhecimento
- **Biblioteca**: todos os segmentos herdam a cor da sua biblioteca

### Navegacao por aprofundamento

Clique em qualquer segmento de pasta para ampliar. Uma trilha de migalhas mostra seu caminho. Clique em qualquer item da trilha para voltar, ou pressione Escape. Clique em um segmento de nota para abri-la no editor.

### Voltar ao Mapa

Apos abrir uma nota a partir do Mapa, um botao "Voltar ao Mapa" aparece na barra de abas. Clique para retornar exatamente onde estava — mesmo nivel de aprofundamento preservado.

---

## 22. Motor Cognitivo

O Motor Cognitivo e o sistema de inteligencia integrado do Constellation que analisa suas notas e revela padroes ocultos e relacoes entre suas ideias. Sua filosofia fundamental:

> "A quantidade dos seus dados nao importa. Nao se trata de quantas fontes voce armazena, mas de como voce formula seu conhecimento a partir delas e o conecta em uma consciencia unica e significativa."

O Motor Cognitivo e composto por nove ferramentas integradas: Links tipados, Estratos de conhecimento, Ciclo de maturidade, Detector de tensoes, Cadeia de proveniencia, Motor de externalizacao, Pulso de revisao, Trilhas e Visoes multi-lente.

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

### 17.6 Motor de externalizacao

#### O que e?

Um pipeline de formalizacao progressiva que rastreia como suas notas amadurecem de capturas brutas a insights cristalizados. Cada nota pode ser atribuida a um de quatro estagios:

| Estagio | Icone | Significado |
|---------|-------|-------------|
| Fugaz | 🌱 | Captura rapida, pensamento passageiro |
| Literatura | 📖 | Reescrita de uma fonte em suas proprias palavras |
| Permanente | 🔗 | Ideia atomica, um conceito, conectada ao seu grafo |
| Sintese | ✨ | Insight original combinando multiplas notas permanentes |

#### Por que importa?

A maioria dos apps trata todas as notas igualmente. O Motor de externalizacao torna a distincao visivel — voce pode ver de relance quanto da sua biblioteca e captura bruta versus compreensao genuina.

#### Como usar

1. Na barra de navegacao (acima do editor), use o menu suspenso de estagios para selecionar um estagio.
2. Ou expanda as Propriedades e use o menu suspenso de estagios la. Ambos sincronizam instantaneamente com a arvore de arquivos.
3. Para promover uma nota, mude o menu suspenso de um estagio para o proximo. No modo Foco, clique em "Promover para Permanente" na parte inferior.
4. Para remover um estagio, selecione "— Estagio —" no menu suspenso.

#### Onde aparece?

- **Barra de navegacao**: menu suspenso com emoji + nome do estagio
- **Painel de propriedades**: menu suspenso quando a propriedade `stage` existe
- **Arvore de arquivos**: icone emoji ao lado do nome da nota
- **Rodape do modo Foco**: botao "Promover para Permanente"

### 17.7 Pulso de revisao

#### O que e?

O Pulso de revisao e um sistema de ressurgimento espacado que traz notas de volta a sua atencao em intervalos crescentes: 1 dia, depois 3, depois 7, depois 14, depois 30 dias apos a ultima revisao. Tambem monitora notas com as tags `#assumption` ou `#model` como pontos de verificacao de modelos mentais, e mantem uma fila "Nunca revisadas" para notas capturadas mas nunca revisitadas.

#### Por que importa?

O conhecimento se dissipa sem revisitacao. Voce escreve uma nota hoje e em tres semanas esquece que ela existe. A repeticao espacada e a tecnica mais estabelecida na ciencia cognitiva para combater essa degradacao. O Pulso de revisao aplica esse principio as suas notas reais.

#### Como usar

1. Clique na aba **Pulso de revisao** na barra lateral esquerda. Voce vera tres secoes: Pendentes de revisao, Pontos de verificacao de modelos mentais (`#assumption` / `#model`), e Nunca revisadas.
2. Clique em qualquer nota para abri-la e ler.
3. Escolha uma das tres acoes:
   - **Revisada** (marca de verificacao) — agenda a proxima revisao no proximo intervalo (1 → 3 → 7 → 14 → 30 dias).
   - **Adiar 7d** (icone de olho) — adia a nota por 7 dias sem avancar o intervalo.
   - **Descartar** (icone de arquivo) — remove a nota da fila de revisao permanentemente.
4. Abra a Paleta de Comandos e digite "Review due notes" para ir direto as notas pendentes.

#### Onde aparece?

- **Barra lateral esquerda**: Aba Pulso de revisao com contador de notas pendentes
- **Paleta de Comandos**: Comando "Review due notes" para acesso rapido

### 17.8 Trilhas

#### O que e?

Trilhas sao sequencias nomeadas e ordenadas de notas — como capitulos de um livro ou paradas em um tour guiado pelo seu conhecimento. Sao definidas adicionando `trail: true` ao frontmatter de uma nota, e listando wikilinks em ordem no corpo da nota.

#### Por que importa?

O conhecimento nem sempre e uma rede. As vezes e um caminho — uma sequencia de aprendizado, uma progressao de argumentos, uma narrativa. As Trilhas capturam essa ordem explicitamente, adicionando uma dimensao linear a sua biblioteca nao linear.

#### Como usar

1. Crie uma nova nota com `trail: true` no frontmatter.
2. No corpo da nota, liste wikilinks na ordem desejada.
3. Ao abrir qualquer nota que pertenca a uma trilha, a barra de navegacao mostra um indicador com o nome da trilha e a posicao (ex. "Minha Trilha 2/5"). Setas de navegacao permitem ir para a nota anterior e proxima.
4. Abra a Paleta de Comandos e digite "Open Trail" para ver todas as trilhas.

#### Onde aparece?

- **Barra de navegacao**: Indicador da trilha com nome, posicao e setas de navegacao
- **Paleta de Comandos**: Comando "Open Trail" lista todas as trilhas

### 17.9 Visoes multi-lente

#### O que e?

Visoes multi-lente permitem visualizar sua biblioteca atraves de diferentes esquemas de classificacao — sem alterar a estrutura de pastas ou duplicar notas. Uma "lente" e um agrupamento virtual que reorganiza notas com base em uma propriedade ou tag. Lentes integradas: "Por estagio" (Fugaz/Literatura/Permanente/Sintese) e "Por topico" (agrupamento por tags). Lentes personalizadas podem ser criadas nas Configuracoes.

#### Por que importa?

Estruturas de pastas impoem uma unica hierarquia, mas o conhecimento nao cabe em uma unica arvore. Visoes multi-lente permitem alternar entre perspectivas sem mover arquivos. As mesmas notas, vistas atraves de diferentes lentes organizacionais.

#### Como usar

1. Na barra lateral, encontre o **seletor de lentes** no topo da arvore de arquivos (padrao "Pastas").
2. Selecione uma lente: "Por estagio", "Por topico" ou uma lente personalizada. A barra lateral se reorganiza instantaneamente.
3. Selecione "Pastas" para retornar a arvore de arquivos padrao.
4. Para criar uma lente personalizada: abra **Configuracoes > Gestao de conhecimento**, clique em **Criar lente**, nomeie e escolha a propriedade frontmatter para agrupamento.
5. Ou use a Paleta de Comandos: digite "Create Lens".

#### Onde aparece?

- **Seletor na barra lateral**: Seletor de lentes no topo da arvore de arquivos
- **Configuracoes > Gestao de conhecimento**: Criar, editar e excluir lentes personalizadas
- **Paleta de Comandos**: Comando "Create Lens"

### Configuracoes do Motor Cognitivo

Todas as ferramentas do Motor Cognitivo podem ser configuradas em **Configuracoes > Motor Cognitivo**:

- **Classificacao de estratos** — Ativar ou desativar a classificacao automatica
- **Rastreamento de maturidade** — Ativar ou desativar o rastreamento do ciclo de maturidade
- **Links tipados** — Ajustar o limiar de sensibilidade para deteccao de links (0.0 – 1.0)
- **Detector de tensoes** — Ativar ou desativar a deteccao automatica de tensoes
- **Substituicao manual** — Adicione propriedades `stratum` e `maturity` no frontmatter para substituir a classificacao automatica

---

## 18. Conexões sugeridas

O Constellation serve para *formular* conhecimento, e o conhecimento é conexão. As **Conexões sugeridas** encontram as notas que já existem na sua Biblioteca e que mais se relacionam com aquela que está a ver — os parentes a que deveria ligar mas ainda não liga — e transformam qualquer uma delas numa **ligação tipada** com um único clique. É o "mais como esta", mas para o pensamento.

**Toda sugestão é tipada.** Quando aceita uma, o Constellation pergunta *como* se relacionam as duas notas — apoia, contradiz, exemplifica, deriva-de, e assim por diante, ou simplesmente **associativo**. Uma ligação tipada é um raciocínio que pode mais tarde ler, pesquisar e questionar; a funcionalidade nunca adiciona ligações em massa e nunca adiciona silenciosamente uma ligação sem tipo. (Consulte **Formulação de Conhecimento** e **Propriedades**.)

**Como as encontra.** As candidatas vêm **apenas da sua própria Biblioteca**, classificadas em relação ao índice de pesquisa em tempo real do Constellation pelo vocabulário partilhado mais *distintivo* — as palavras raras e reveladoras, não as comuns. Cada sugestão mostra os **termos partilhados** que explicam por que apareceu, para que nunca aceite um palpite de caixa-preta.

**Cinco lugares, uma lista.** A mesma lista de sugestões aparece no **Revisor** (🕐, para notas que assinala como *órfãs* ou *frágeis*), na **Aba de Retroligações** (barra lateral direita), no **Inspetor 360°**, na **Aba de Saúde** e na **Vista do Céu** (🌌 — clique com o botão direito em qualquer estrela → **Sugerir ligações…**).

**Entrada vs saída — e por que não escolhe.** As superfícies de diagnóstico (o **Inspetor 360°** e a **Aba de Saúde**) sugerem conexões de **entrada** — *quais notas deveriam apontar **para aqui***. As superfícies gerais (a **Aba de Retroligações** e a **Vista do Céu**) sugerem conexões de **saída** — *para onde esta nota deveria apontar*. A superfície escolhe a direção que se ajusta à sua função; você escolhe a nota e o tipo. (Uma atualização futura permitir-lhe-á alternar a direção você mesmo.)

**Como usar.** Sob o título **Conexões sugeridas** verá notas relacionadas classificadas das mais próximas primeiro, cada uma com os seus termos partilhados. Clique no botão **Ligar** de uma candidata → no pequeno menu **"Como se relacionam?"** escolha o tipo de relação → a ligação tipada é criada **instantaneamente** e a sugestão sai da lista. Passa então a viver nas **propriedades** da nota e aparece nas suas retroligações/ligações de saída e em todo o grafo. Se nada realmente se ajustar, deixe-as — ou, no Revisor, marque a nota como **autónoma** deliberada. As Conexões sugeridas propõem; quem decide é você.

**Local, privado, não bloqueante.** As sugestões são calculadas a pedido apenas a partir da sua Biblioteca — nada sai do seu dispositivo — e reuni-las nunca bloqueia a sua escrita (verá um breve "A procurar notas relacionadas…" enquanto trabalha). As sugestões, as pistas de termos partilhados e os tipos de relação aparecem todos no idioma que escolheu e refletem-se corretamente para os sistemas de escrita da direita para a esquerda.

---

### Estilização de propriedades (Designer de estilo)

Abra o **Designer de estilo** (Configurações → Aparência → ✦ Abrir Designer de estilo, ou a sua própria aba) e escolha a categoria **Propriedades** para reestilizar as pequenas tags dentro do frontmatter de uma nota. Dois elementos: **Tags de propriedade** (as pílulas comuns no estilo `tags` — Fundo da tag, Texto da tag, Raio da tag 0–20 px, Altura 14–32 px) e **Pílulas de taxonomia** (Fundo, Texto, Raio 0–20 px). Uma pré-visualização ao vivo no centro se atualiza enquanto você edita; cada valor começa exatamente com a aparência de hoje, então nada muda até você tocar em um controle. Clique em **Manter** para salvar neste Universo.

### Cores cognitivas (Designer de estilo)

A categoria **Cores cognitivas** oferece **uma cor compartilhada por estado cognitivo**, de modo que toda superfície que mostra aquele estado concorde. Cinco conjuntos:

- **Maturidade** — Semente, Broto, Perene, Canônica, Murcha.
- **Confiança** — Hipótese, Evidência, Estabelecido, Contestado.
- **Origem** — Recebido, Descoberto, Misto, Nenhum.
- **Estágio** — Faísca, Nascimento, Crescimento, Maturidade, Dormência, Arquivamento.
- **Categoria de correspondência** (por que um resultado de busca correspondeu) — Título, Conteúdo, Tag, Wikilink, Propriedade, Semântica, Estruturado.

O comportamento é **unificar sob demanda**: nada muda até você escolher uma cor. Cada superfície mantém a sua cor atual como reserva, e no momento em que você define a cor de um estado aqui, **todas** as superfícies que mostram aquele estado — árvore de arquivos, abas, o inspetor da nota, o destaque de busca dentro do editor, o selo de correspondência e o destaque do resultado de busca — passam para a sua cor de uma só vez. Deixe um estado intocado e ele fica exatamente como antes. Clique em **Manter** para salvar.

### Menus de clique direito

O Constellation oferece um menu de contexto em três lugares, cada um oferecendo apenas as ações que se ajustam a onde você clicou:

- **Clique direito no corpo da nota** — Adicionar link / Adicionar link externo; **Formatar ▸** (Negrito, Itálico, Sublinhado, Tachado, Realce, Código em linha, Matemática, Alternar comentário, Sobrescrito, Subscrito, Limpar formatação); **Parágrafo ▸** (Lista com marcadores/numerada/de tarefas, H1–H6, Corpo, Citação em bloco); **Inserir ▸** (Nota de rodapé, Tabela, Callout, Régua horizontal, Bloco de código, Bloco de matemática, Imagem); Recortar / Copiar / Colar / Colar como texto simples / Selecionar tudo; e **Estilo…** (abre o Designer de estilo na categoria **Editor**).
- **Clique direito em uma linha de propriedade do frontmatter** — Copiar valor, Copiar nome, Remover propriedade, Adicionar propriedade; depois o mesmo menu de edição do corpo; e **Estilo…** abrindo o Designer de estilo na categoria **Propriedades**.
- **Clique direito em um resultado de busca** — um subconjunto **seguro**: Abrir, Abrir em nova aba, Revelar na árvore, Copiar link, Copiar caminho, Marcar, Mostrar no explorador, Abrir no aplicativo padrão e **Estilo…** (a categoria **Cores cognitivas**). Por design **não há Renomear, Mover ou Excluir** aqui — o painel de busca não mantém uma cópia atualizada ao segundo da árvore de arquivos, então as ações destrutivas permanecem na árvore de arquivos, onde a visão está sempre atual.

Cada entrada **Estilo…** cai na categoria do que você clicou com o botão direito, então você nunca precisa caçar os controles certos. Cada item de menu, nome de categoria e rótulo de estado aparece no idioma de interface escolhido e espelha para layouts da direita para a esquerda.

---

*Manual do Usuario do Constellation — Versao 0.1.0 — Marco 2026*
*uconstellation.world*

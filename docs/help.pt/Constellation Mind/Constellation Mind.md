---
aliases:
  - Mente do Constellation
  - Constellation Mind
  - Mind
  - LLM local
  - Modelo de Linguagem Grande local
  - Fanar
  - Chat de IA
  - IA pessoal
description: Constellation Mind é a camada de Modelo de Linguagem Grande (LLM) local do Constellation — uma IA com a qual você pode conversar sobre suas próprias notas, rodando inteiramente no seu dispositivo. A Fase 0b foi lançada em 2026-05-24 com o modelo Fanar-1-9B com prioridade ao árabe, instalável a partir de Configurações → Mind. A superfície de chat chega na Fase 1.
---

# Constellation Mind (عقل Constellation)

## O que é?

Constellation Mind é a camada de Modelo de Linguagem Grande (LLM) local do Constellation — um assistente de IA que conhece seu Universo e pode conversar com você sobre suas notas, **sem enviar nenhuma delas para a nuvem**.

Três coisas o tornam distinto de qualquer outra ferramenta de "IA para notas":

1. **Local-primeiro.** O modelo roda no seu dispositivo. Suas notas nunca o deixam. Não há ida e volta para a nuvem — o chat é local e capaz de funcionar offline.
2. **Árabe-primeiro.** O modelo padrão incluído é o **Fanar-1-9B**, o modelo árabe-cêntrico, consciente do contexto sunita do Qatar Computing Research Institute. Competência nativa em MSA e dialetos do Golfo; o inglês é a segunda língua, não a única.
3. **Vinculado a citações.** Toda afirmação factual que a IA faz sobre suas notas deve citar a nota de origem. Citações alucinadas são capturadas por um validador pós-geração (Fase 1).

## O que sai hoje (Fase 0b — 2026-05-24)

- **Painel Configurações → Mind** — lista os modelos instaláveis (atualmente apenas Fanar 1.9B Q4_K_M, ~5 GiB), com um botão Instalar que baixa e verifica o modelo.
- **Instalação do modelo** — download em fragmentos a partir de um GitHub Release (sem nuvem de terceiros), verificado por SHA-256 por fragmento e no conjunto montado.
- **Runtime de inferência real** — `llama-cpp-2` (apenas CPU em v1) carrega o GGUF Q4_K_M e faz streaming de tokens.
- **Ainda sem superfície de chat** — essa é a Fase 1 (o próximo marco). Hoje você pode instalar o modelo e verificá-lo; a UI de conversação chega no MIG-048.

## Como instalar o Fanar

1. Abra **Configurações → Mind**.
2. Encontre **Fanar 1.9B (Q4_K_M)** no catálogo. O cartão mostra o tamanho (5,01 GiB), a licença (Apache-2.0 com avisos defensivos da Gemma) e um botão "Definir como ativo" ou "Instalar".
3. Clique em **Instalar**. Uma barra de progresso mostra download + verificação SHA + montagem em três fases.
4. Quando o emblema mudar para **Instalado** + **Ativo**, o modelo está pronto. O Fanar vive em `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf` e é respaldado por mmap (sem cópia para a RAM).

É isso. Até que a Fase 1 entregue a superfície de chat, o modelo instalado está em espera.

## O que vem na Fase 1 (próximo marco)

- **Superfície de chat** — um painel do Constellation onde você fala com o Fanar sobre seu Universo em árabe ou inglês (com consciência RTL por mensagem).
- **Ferramentas de leitura** — o Mind pode chamar `search_notes`, `read_note`, `find_similar`, `list_recent` para fundamentar suas respostas em suas notas reais.
- **Validador de citações** — toda afirmação cita uma nota real; referências `note:UUID` fabricadas são rejeitadas antes de chegarem a você.
- **Pré-aquecimento na inicialização do app** — o Mind carrega em segundo plano para que seu primeiro chat não pague os 10 segundos de carregamento a frio.
- **Histórico de conversas** — salvo por Universo; promovível a uma Nota.

Veja `docs/Constellation-Mind-Concept-Paper-v1.1.md` para a arquitetura completa e `docs/Constellation-Mind-Implementation-Plan-v1.0.md` para o roteiro fase por fase.

## O que vem depois

- **Fase 2 — Ferramentas de escrita** (Mind propõe edições / novas notas / links sob sua aprovação explícita).
- **Fase 2.5 — RoutedProvider + Jais** (um segundo modelo, Jais-2-8B da G42/MBZUAI, junta-se ao Fanar como co-padrão; Mind faz roteamento entre eles com base na solicitação).
- **Fase 3 — Auto-classificação + link inteligente** (Mind propõe facetas e links ao salvar a nota).
- **Fase 4 — Ferramentas de capacidade** (voz → nota, OCR → nota, tradução).
- **Fase 5 — Adesão à nuvem** (sua própria chave Anthropic / OpenAI, com teto de custo por Universo e log de saída por turno).

## Privacidade e fluxo de dados

- **HTTP de saída apenas ao instalar um modelo** — o Constellation baixa arquivos de modelo dos [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases) deste repositório. Sem telemetria. Sem inferência em nuvem (ainda — essa é a Fase 5, e somente com sua adesão explícita).
- **No disco:** o GGUF do modelo + um registro `installed_models.json` que rastreia quais modelos você tem e qual está ativo.
- **Em tempo de execução:** o arquivo do modelo carregado é mapeado em memória; seus prompts e respostas vivem apenas na RAM.

## Licenças

Cada modelo carrega sua própria LICENSE.txt ao lado dele no GitHub Release. Para o Fanar:

- **Apache License 2.0** (a licença declarada pelo QCRI no repositório Fanar-1-9B-Instruct).
- **Termos de Uso da Gemma** — o Fanar é um pré-treinamento continuado do `google/gemma-2-9b`; o Constellation envia os avisos da Gemma defensivamente mesmo que o QCRI rotule novamente o resultado apenas sob Apache-2.0.
- **Citação Fanar** (Fanar Team 2025, arXiv:2501.13944).
- **Aviso de redistribuição do Constellation** — o GGUF no GitHub Release do Constellation é uma quantização dos safetensors upstream do QCRI, produzido por `.github/workflows/model-pipeline.yml` e distribuído sob Apache-2.0 com a LICENSE original acompanhando.

A LICENSE.txt completa vive ao lado de cada modelo em seu release: <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## Solução de problemas

**Emblema "Ainda não pronto" em vez do botão Instalar.** O catálogo incluído tem um SHA-256 marcador de posição para esse modelo. Isso não deve ocorrer em uma instalação normal do Constellation; se você vir, o catálogo não foi atualizado para essa versão do modelo. Abra uma issue.

**A instalação trava em "Baixando parte X/Y".** Problema de rede. Cancele em Configurações → Mind, re-acione Instalar — os fragmentos parciais são limpos automaticamente.

**A instalação tem sucesso, o SHA-256 do arquivo não corresponde.** Um bit-flip no download. A reinstalação obterá uma cópia nova.

**Superfície de chat faltando.** A Fase 1 (MIG-048) ainda não foi lançada. O modelo pode ser instalado e verificado hoje; a UI de conversação chega no próximo lançamento.

---

*Os subtópicos se juntarão a esta pasta conforme a Fase 1 for lançada: visita guiada da UI de chat, comportamento de toque dos chips de citação, seletor de múltiplos modelos, renderização de conversas longas na segunda tela.*

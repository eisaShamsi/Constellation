# Metadados Epistêmicos

> **Nota de tradução:** Este tópico de ajuda é uma tradução gerada
> por IA a partir da versão canônica em inglês em
> `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md`.
> Revisão por falantes nativos pendente. Por favor, envie correções
> através do repositório do projeto.

*(MIG-022 §A — extensões de esquema da análise de lacunas §6.1)*

Este tópico descreve um pequeno conjunto de **campos opcionais de frontmatter** que o Constellation agora reconhece para uma classificação epistêmica mais rica de suas notas. Eles foram adicionados em resposta à análise de lacunas (`docs/epistemic-content-gap-analysis.md`) — o reconhecimento de que o modelo de dois eixos Source × Content Type contra o qual o Constellation Epistemic Content Engine (CECE) classifica não cobre tudo o que você pode querer registrar sobre como veio a saber o que sabe.

Estes campos são **todos opcionais**. Notas existentes sem eles funcionam sem alterações. Você os adiciona à mão (ou, no futuro, através de um editor estruturado) quando uma nota é o tipo de conhecimento que se beneficia do sinal extra.

---

## Os campos

### `held_by` — *de quem é esta posição?*

Uma string curta indicando quem sustenta a posição que a nota descreve. O padrão é `user` (sua própria posição). Outros valores que você pode usar:
- O nome de um estudioso: `held_by: "al-Shāfiʿī"`
- Uma escola: `held_by: "Ḥanafī"`
- Uma figura histórica: `held_by: "Aristotle"`

Quando você escreve uma nota que registra *a posição de outra pessoa* em vez da sua, `held_by` é o campo que o diz. Sem ele, o Constellation tacitamente assume que o estado epistêmico da nota é o seu — o que, para o trabalho acadêmico sério, frequentemente está errado.

### `domain` — *sobre que assunto trata isto?*

Uma lista de etiquetas disciplinares. Distinto do seu campo livre `tags` (folksonomia / humor / projeto), `domain` é o campo estruturado de disciplina/tópico para recuperação e filtragem. Exemplos:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

Uma nota classificada como `content_type: "proposition"` E `source: "inference"` poderia ser um teorema lógico (domain: `[logic, mathematics]`) ou uma opinião jurídica (domain: `[fiqh, ʿibādāt]`) — mesma forma epistêmica, contextos de recuperação muito diferentes. `domain` permite que você diga qual.

### `function` — *para que serve esta nota?*

Uma única string identificando o uso pretendido da nota. Valores reconhecidos:

- `reference` — ler quando necessário (uma definição, uma citação, um fato que você consultará mais tarde)
- `seed` — incubar (uma ideia em estágio inicial que você ainda está desenvolvendo)
- `actionable` — fazer algo com isto (uma tarefa, um acompanhamento, uma decisão a tomar)
- `shipped` — produto finalizado (um ensaio publicado, uma análise entregue, um ciclo fechado)

Distinto do eixo content-type do CECE (que diz que TIPO de conhecimento é) — `function` diz o que você FARÁ com a nota.

### `provenance_civilization` — *que vocabulário tradicional está em jogo?*

Uma string opcional identificando a pegada civilizacional do vocabulário da nota. Útil para a recuperação contra corpora específicos de tradição. Exemplos:

- `provenance_civilization: "sunni-usuli"` — tradição sunita *uṣūl al-fiqh* (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — filosofia analítica pós-Frege
- `provenance_civilization: "nyaya"` — escola indiana Nyāya de epistemologia pramāṇa
- `provenance_civilization: "buddhist-pramana"` — tradição epistemológica budista (Dignāga, Dharmakīrti)

A maioria das notas não precisa disto. Quando você tem, digamos, uma nota que se baseia tanto em *uṣūl* sunita QUANTO em epistemologia analítica anglo-americana, registrar a pegada primária ajuda o você-futuro a recuperar o material comparável adequado.

### `updated_at` — *quando sua posição mudou pela última vez?*

Data ISO da revisão deliberada mais recente do conteúdo epistêmico da nota. Distinto do timestamp `modified` do sistema de arquivos (que captura cada salvamento, mesmo correções de erros de digitação); `updated_at` é o timestamp que VOCÊ define quando realmente repensou a posição.

```yaml
updated_at: 2026-05-09
```

Útil quando o resto do eixo temporal §6.3 chegar (histórico de estados da nota) — até lá, este é um campo de instantâneo único que registra "a última vez que revisei minha visão".

### `ikhtilāf` — *desacordo erudito estruturado*

O mais complexo dos novos campos. Registra o *ikhtilāf* — o desacordo estruturado entre estudiosos ou escolas sobre uma questão — como uma lista de pares `{school, position}`. O Constellation fornece um widget personalizado do painel de Propriedades para editá-lo; você também pode editar o YAML diretamente.

Exemplo:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

Uma nota com `ikhtilāf` não está em nenhum estado epistêmico único — ela registra um *desacordo estruturado* entre múltiplos agentes. Sem este campo, o Constellation trataria tal nota como se ela própria sustentasse uma dessas posições, o que é incorreto.

O painel de Propriedades renderiza cada linha como um cartão editor com duas entradas (school + position) mais um botão de remover, e um botão "Adicionar escola" na parte inferior.

### `warrant` e `warrant_notes` — *analisados mas inertes (por enquanto)*

Dois campos são analisados e armazenados em disco mas **ainda não expostos em nenhuma UI**:

- `warrant: "mutawātir"` — um rótulo de grau para a garantia da afirmação da nota. A hierarquia sunita *uṣūl* usa *mutawātir / mashhūr / āḥād* e dentro do hadith especificamente *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*. Outras tradições têm seus próprios vocabulários de classificação.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — texto livre que sustenta o grau da garantia.

Estes estão prontos para uso quando o **Constellation Warrant Research workstream** entregar seu classificador (projeto de pesquisa de vários meses; veja a análise de lacunas §6.2). Até lá você pode preenchê-los à mão e os dados persistem; nada os exibe. Consultas e badges futuros sensíveis à garantia leem esses valores diretamente.

---

## Onde estes campos aparecem

Quando você preenche qualquer um dos novos campos no frontmatter de uma nota, eles aparecem no **painel de Propriedades** (barra lateral direita) da mesma forma que qualquer outro campo YAML — uma linha por chave, com o editor apropriado ao tipo:

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → entrada de texto
- `domain` → lista de etiquetas (adicionar digitando + Enter, remover com o × em cada etiqueta)
- `updated_at` → seletor de data
- `ikhtilāf` → widget personalizado com linhas `school` / `position` + botões adicionar/remover

---

## E quanto a `supersedes`?

`supersedes` é tecnicamente uma *relação entre notas* em vez de uma propriedade de uma única nota. O Constellation o trata como um **link tipado**, não como um escalar YAML:

```markdown
Esta nota substitui minha análise anterior: [[old-note-id|supersedes]]
```

O sufixo `|supersedes` no wikilink diz ao Constellation que este é um link tipado do tipo `supersedes` — recebe uma cor distinta de pílula (azul-cinza ardósia), aparece nos painéis Backlinks + Outgoing Links junto com outros links tipados, e participa da Living Link Architecture (peso, ciclo de vida, contagens de travessia).

Isto mantém as relações nota-a-nota em um só lugar — o sistema de links tipados — em vez de dividi-las entre links tipados e escalares de frontmatter. O mesmo se aplica a `contradicts:` (já um link tipado no vocabulário pré-MIG-022).

---

## O que isto NÃO é

Estes campos **NÃO** são consumidos pela classificação CECE hoje. O CECE classifica apenas sobre Source × Content Type; os novos campos de metadados são registrados para recuperação conduzida por humanos, futuros classificadores sensíveis à garantia, e o eixo temporal (quando chegar).

Em particular:
- `function: "actionable"` NÃO cria automaticamente uma tarefa no painel de Tarefas
- `held_by: "al-Shāfiʿī"` NÃO muda como o CECE classifica a nota
- `domain: [fiqh]` NÃO filtra seus resultados de busca a menos que você escreva a consulta de busca para incluí-lo

Os campos são **esquema** — um vocabulário reconhecido de campos que você pode adicionar. MIGs futuros entregarão recursos que os consumam (classificador de garantia, consultas temporais, filtragem sensível ao domínio, etc.).

---

## Um exemplo trabalhado

Uma nota registrando as posições das escolas sunitas sobre se a quebra do amanhecer importa para a validade do dia de jejum obrigatório:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

A posição Mālikī clássica (uma niyyah para o mês) é descrita
por [[Ibn-Rushd-bidayah|derives-from]] na passagem sobre niyyah em
bidāyat al-mujtahid. Minha visão atual: [[ramadan-niyyah-personal|supersedes]]
minha nota anterior que confundia a posição Mālikī com a Shāfiʿī.
```

Seis dos sete novos campos preenchidos; `warrant_notes` omitido (sem detalhe de cadeia de transmissão a registrar ainda); `supersedes` e `derives-from` como links tipados no corpo, não como escalares YAML.

---

*MIG-022 §A — as extensões de esquema chegam neste build do Constellation. O Warrant Research workstream (Concept Paper separado, vários meses) entrega o classificador de garantia que consome o campo `warrant`. O eixo temporal (MIG-023, ciclo Architect separado) consome `updated_at` mais o histórico mais amplo de estados da nota.*

---
aliases:
  - Painel do Calendário
  - Calendário de Notas Diárias
  - Calendários Culturais
description: Uma visualização mensal em página inteira em oito calendários, com dias clicáveis, criação de notas diárias, datas de tarefas e registo de datas culturais.
---

# Calendário

O **Calendário** é uma visualização mensal em página inteira, aberta a partir da **doca esquerda** (o ícone de calendário). Os dias que têm notas ou tarefas pendentes são assinalados com **pontos** coloridos. O cabeçalho mostra o mês no calendário que escolheu; se definir um **calendário secundário**, um subtítulo por baixo mostra o intervalo equivalente nesse calendário (por exemplo, um mês gregoriano mostra o seu período em Hijri, "Dhul-Hijjah 1447 – Muharram 1448 AH").

## Clicar num Dia

Cada célula de dia é interativa:

| Ação | Resultado |
|--------|--------|
| Clicar no espaço vazio (ou no número do dia) | Abre — ou cria — a **nota diária** desse dia. Clicar numa data que já tem uma nota diária simplesmente a **abre**; nunca cria um duplicado. |
| Clicar num ponto | Abre esse item específico. Se um dia tiver várias notas ou tarefas, clicar no ponto mostra uma pequena **lista** para escolher. |
| Clicar num ponto de tarefa | Abre a nota **com o ecrã posicionado na linha dessa tarefa**, pronta a editar. |

### Cores dos Pontos

| Cor do Ponto | Significado |
|-----------|---------|
| Dourado | A **nota diária** desse dia |
| Roxo | Outra **nota** editada (ou datada) nesse dia |
| Vermelho | Uma **tarefa** pendente nesse dia |

Todas as cores dos pontos — e todas as outras partes do calendário — são personalizáveis na superfície **Style Setter → Calendário**.

> [!tip]
> Na lista de tarefas pode **marcar a caixa de uma tarefa para a concluir** diretamente a partir do calendário — as tarefas concluídas desaparecem de imediato. Só as tarefas que têm a sua própria data de prazo `📅 YYYY-MM-DD` aparecem no calendário (é a data que as coloca num dia).

## Calendários Culturais (Oito)

Em **Definições → Calendário** pode definir o **sistema de calendário**, e toda a grelha mensal muda para esse calendário:

- **Gregoriano**
- **Hijri (Islâmico)** — um motor astronómico preciso; os meses sagrados são realçados e os eventos islâmicos são assinalados.
- **Hijri Solar (Persa)**
- **Hebraico**
- **Indiano (Saka)**
- **Budista**
- **Chinês** — *lunissolar*
- **Coreano** — *lunissolar*

Cada célula mostra tanto a data do calendário escolhido (grande) como a data gregoriana (pequena), além da fase da lua. Cada cabeçalho de mês mostra o **nome do mês, o seu número entre parênteses e o ano** — o número ajuda nos calendários cuja ordem de meses é pouco familiar.

Os calendários **Chinês e Coreano** são *lunissolares*: por vezes inserem um **mês intercalar** (闰六月 / 윤6월), que o calendário mostra como uma página própria, para que a navegação nunca o salte nem o duplique.

Pode também escolher o **início da semana** (domingo/segunda-feira) e ativar a **coluna do número da semana**.

### Opções do Calendário Hijri

Em **Definições → Calendário → "Hijri calendar (Islamic)"** existem dois controlos adicionais:

- **Método de cálculo** — **Astronómico (Conjunção Lunar)**, que segue a verdadeira lua nova (o mais preciso, e a predefinição), ou **Tabular (al-Tawfīqāt al-Ilhāmiyyah)** (o ciclo aritmético clássico).
- **Correção do mês** — ajuste o início de um mês Hijri em ±1 ou ±2 dias para corresponder a um **avistamento local da lua**. Escolha o ano e o mês Hijri, selecione um deslocamento e clique em **Set**; a correção aplica-se a esse mês e a todos os meses seguintes. As suas correções são listadas (cada uma removível), com um botão **Clear all**.

Ambas as definições (e as suas correções) são guardadas **com o seu universo**, pelo que acompanham os seus dispositivos.

### Opções de Exibição Chinesa e Coreana

A Coreia usa o calendário lunar chinês, por isso os dois partilham datas idênticas — o que os distingue é a **escrita** e o **ano**. Quando qualquer um deles é o seu calendário principal ou secundário, **Definições → Calendário** mostra dois controlos adicionais:

- **Exibição do ano** — Chinês: o ciclo sexagesimal 丙午年, o ano simples, ou ambos; Coreano: a era **Dangi** 단기 4359, o ano, ou o sexagesimal 병오년.
- **Nomes dos meses** — *escrita nativa* (五月 / 5월), ou *fonética* — a pronúncia do mês escrita na sua própria língua (Português "Wǔyuè / Owol"; Árabe "وُو-يوي / أوه-وُل").

## Estilizar o Calendário

Abra o **Style Setter** (doca esquerda, ou **Definições → Style Setter**) e escolha a superfície **Calendário** para reestilizar cada parte — cada elemento tem a sua própria **cor e tamanho de texto** (números dos dias, a data de referência cruzada, a pílula do mês, os cabeçalhos dos dias da semana, os números das semanas, o glifo da lua, o destaque de Hoje, as linhas da grelha e os pontos de nota/tarefa/evento), além da **fonte** do calendário. Uma pré-visualização ao vivo, em tamanho real, atualiza-se à medida que edita; clique em **Keep** para aplicar.

## Notas Diárias

O Calendário serve plenamente as notas diárias: clique em qualquer dia para a abrir, ou execute o comando **"Daily Note"** (paleta de comandos) para saltar para hoje.

> [!tip]
> **Os nomes dos ficheiros das notas diárias permanecem sempre gregorianos** (`YYYY-MM-DD`), independentemente do calendário exibido — para que os seus ficheiros se mantenham portáteis e ordenem corretamente. A data cultural é mostrada no calendário, e pode ser registada no frontmatter da nota (abaixo).

## Registar uma Data Cultural numa Nota

Duas ferramentas opcionais escrevem a data cultural nas **propriedades** de uma nota (o nome do ficheiro permanece sempre gregoriano `YYYY-MM-DD`):

- **Carimbo Hijri da nota diária** — *Definições → Calendário → "Stamp the Hijri date in daily notes."* Quando ativo (disponível apenas enquanto o calendário Hijri for o seu **principal ou secundário**), cada **nova** nota diária recebe uma linha `hijri:`, por exemplo `hijri: 1448-01-06`. As notas que já tem nunca são alteradas.
- **"+ Hijri" nas Propriedades de uma nota** — abra as **Propriedades** de qualquer nota, passe o rato sobre a data e aparece um pequeno botão **"+ Hijri"** (além de "+ Jalali", "+ Hebrew" e assim por diante — **um botão por cada calendário não-gregoriano que tenha selecionado**). Clique nele e o Constellation lê a data gregoriana da nota e adiciona o equivalente, por exemplo `jalali: 1405-03-30`. O botão coreano escreve o ano **Dangi**; um **mês intercalar** chinês/coreano é marcado com um `L` (por exemplo `chinese: 2025-06L-17`). Se a nota não tiver propriedade de data, usa a data de criação do ficheiro.

> [!tip] RTL Support
> A grelha do calendário respeita a direção atual do texto. Nas línguas RTL (Árabe, Hebraico, Persa, Urdu), a disposição do calendário ajusta-se em conformidade.

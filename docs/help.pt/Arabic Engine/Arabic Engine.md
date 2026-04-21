# Motor arabe

O Constellation analisa o texto em arabe com um motor morfologico de cinco camadas, construido do zero para este aplicativo. Nao e a portacao de um stemmer existente — e um instrumento nativo que entende raizes arabes, padroes, nomes proprios, emprestimos e a sua propria terminologia. Voce nunca configura o motor em si; ele corre em silencio sob cada busca, cada link, cada entrada do indice. O que voce *pode* configurar — e o que este topico de ajuda cobre — e o unico ponto em que o motor convida o seu julgamento: o painel **Substituicoes do motor arabe** nas Configuracoes.

---

## Por que o motor existe

O arabe e uma lingua de moldes. Uma unica raiz como ك‑ت‑ب ("escrever") gera dezenas de formas de superficie — كاتب (escritor), مكتوب (escrito), كتاب (livro), يكتب (ele escreve), كتبنا (nos escrevemos) — que, numa busca, deveriam todas colapsar ao mesmo nucleo semantico. Um stemmer ingenuo ou mutila essas formas (descascando em excesso وائل para ائل, por exemplo) ou perde por completo a conexao entre elas. O motor do Constellation evita ambas as falhas passando cada palavra arabe por cinco camadas em ordem estrita de prioridade:

1. **Camada 0 — Substituicoes do usuario** (esta e a que voce controla)
2. **Camada 2 — Lista protegida** (cerca de 1.200 nomes proprios, lugares, emprestimos e palavras funcionais curados a mao, que nunca devem ser tocados)
3. **Camada 3 — FST generativo** (um transdutor de estados finitos compilado que mapeia cerca de 7.000 raizes x 158 padroes para todo o seu vocabulario de superficie)
4. **Camada 3b — Cascata** (reparos fonologicos: assimilacao, raizes fracas, colocacao da hamza)
5. **Camada 5 — Heuristica** (o recuo gentil — um cortador de afixos conservador que so age quando todas as outras camadas recusaram responder)

Uma etapa de classificacao (Camada 4) escolhe a melhor analise quando mais de uma camada produz uma leitura. A classificacao coloca as suas substituicoes acima de tudo o mais.

---

## Recurso: Substituicoes do motor arabe

### O que e

O painel de Substituicoes e uma pequena tabela nas Configuracoes onde voce diz ao motor, com as suas proprias palavras, como analisar superficies arabes especificas. Cada substituicao tem:

- **Forma de superficie** — a palavra arabe exatamente como voce digita (p. ex. وائل).
- **Lema** — a forma canonica que o motor deve retornar (p. ex. وائل).
- **Raiz** — opcional. Tres ou quatro consoantes se a palavra tiver raiz classica.
- **Padrao** — opcional. Um rotulo livre (p. ex. `فاعل`) se voce quiser registrar o molde morfologico.
- **Categoria** — Nome proprio / Substantivo / Adjetivo / Adverbio / Verbo / Particula / Estrangeiro / Desconhecido.
- **Nota** — opcional. Uma linha de contexto para o seu eu futuro.

### Por que importa

Toda rede de conhecimento contem termos que o motor nao pode conhecer a partir de um dicionario: as suas proprias cunhagens, nomes da sua cidade, siglas do seu campo, emprestimos que os seus colegas preferem grafados de um jeito especifico. Sem substituicoes, o motor aplicaria a sua analise generica a essas superficies e os resultados da busca se fragmentariam em torno de pequenas variacoes. Uma substituicao e a resposta soberana — ela vence o FST generativo, a cascata e o recuo heuristico. A classificacao da Camada 4 da as substituicoes a origem mais alta e uma confianca de 1,0, de modo que nunca sao descartadas em favor de outra analise.

As substituicoes residem em um unico arquivo JSON em `<seu Universo>/.constellation/arabic-overrides.json`. O arquivo e texto simples, ordenado alfabeticamente e escrito de forma atomica (via um par `.tmp`+renomear), de modo que uma queda de energia durante uma edicao nao pode corromper o arquivo. Ele e seu — voce pode versiona-lo, diferencia-lo ou compartilha-lo entre dispositivos.

### Como usar

**Passo 1: abrir o painel**

Clique no icone de engrenagem na barra superior direita (ou pressione `Ctrl + ,` / `Cmd + ,`) para abrir as Configuracoes. Na barra lateral esquerda, selecione **Substituicoes do arabe** — ela fica ao lado de **Idioma**. Se nao a vir, role a barra lateral.

**Passo 2: adicionar a sua primeira substituicao**

Clique em **Adicionar substituicao**. Aparece um formulario com seis campos (forma de superficie, lema, raiz, padrao, categoria, nota). Digite a forma de superficie exatamente como voce a escreve nas suas notas — o motor normaliza os diacriticos e as variantes de alif internamente, portanto voce nao precisa reproduzi-los com precisao. Preencha o lema que deseja que o motor retorne. Deixe raiz e padrao em branco se nao os conhecer; o motor continuara a usar a substituicao. Escolha uma categoria no menu suspenso ou deixe em **Desconhecido**. Clique em **Guardar**.

**Passo 3: observar a faixa de reindexacao**

No momento em que voce guarda, o painel mostra **A reindexar…** e o motor varre cada nota do Universo ativo cujo texto contem aquela superficie. Cada nota correspondente e retokenizada sob o novo veredito da substituicao. Quando a varredura termina — normalmente em menos de um segundo num Universo tipico — a faixa passa a **N nota(s) reindexada(s)** e desaparece sozinha apos tres segundos. Voce nao precisa reiniciar o aplicativo nem reconstruir indice algum.

**Passo 4: verificar na busca**

Abra o hub de Busca (`Ctrl + K` / `Cmd + K`) e digite a superficie. As ocorrencias devem refletir agora o lema que voce especificou: consultas pelo lema encontram a superficie, e consultas pela superficie encontram outras flexoes do lema.

**Passo 5: remover uma substituicao**

Clique no botao **x** na linha da substituicao. A entrada e removida do disco imediatamente e a mesma varredura de reindexacao corre ao contrario — as notas que continham a superficie sao retokenizadas sob a analise generica do motor. A faixa informa quantas notas foram afetadas.

### Interacao com a Lista protegida

A Lista protegida (Camada 2) ja contem cerca de 1.200 superficies comuns que nunca devem ser cortadas — nomes como وائل, lugares como فلسطين, emprestimos como إنترنت. Voce nao precisa adiciona-las; o motor ja as traz. Use o painel de Substituicoes para superficies *pessoais* do seu Universo — a sua propria terminologia, nomes locais, emprestimos especificos da sua area, ou casos em que voce discorda da leitura automatica do motor.

### Interacao entre Universos

Cada Universo tem o seu proprio arquivo de substituicoes. Trocar de Universo troca o conjunto ativo na memoria — o motor recarrega o JSON a partir da pasta `.constellation/` do novo Universo. Se o arquivo estiver ausente (Universo novinho em folha), o motor trata o conjunto como vazio. Se o arquivo estiver malformado, o motor registra um aviso e recua para um conjunto vazio, em vez de recusar-se a carregar.

### O que acontece se voce editar o arquivo a mao

Pode faze-lo. O formato do arquivo e:

```json
[
  {
    "surface": "وائل",
    "lemma": "وائل",
    "root": null,
    "pattern": null,
    "pos": "ProperNoun",
    "note": "Nome proprio — nunca cortar"
  }
]
```

Mantenha as entradas em ordem alfabetica pela superficie para obter diffs amigaveis ao git. O motor reordena a cada guardar, portanto reordenacoes manuais nao sobrevivem a uma edicao feita pela interface.

---

## Glossario

- **Forma de superficie** — uma palavra arabe como e escrita, incluindo cliticos anexados (p. ex. الكتاب, بالكتاب, كتبنا).
- **Lema** — a forma de citacao de uma palavra, sem flexao (p. ex. كتاب).
- **Raiz** — o nucleo semantico de 3 ou 4 consoantes compartilhado por uma familia de palavras (p. ex. ك‑ت‑ب).
- **Padrao** — o molde de vogais e afixos que, combinado a uma raiz, produz uma superficie (p. ex. فاعل → كاتب).
- **FST** — um transdutor de estados finitos. O motor usa um para mapear raizes x padroes ao seu vocabulario de superficie de maneira eficiente.
- **Cascata** — a camada de reparos fonologicos que trata assimilacao, consoantes fracas e colocacao da hamza.
- **Substituicao** — o seu proprio veredito sobre como uma superficie especifica deve ser analisada; vence qualquer outra camada.

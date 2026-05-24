---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: es
source: docs/help.uConstellation.World/Note Summaries/Note Summaries.md
aliases:
  - Note Summaries
  - Note Summary
  - Summary
  - NSC
  - Note Summary Creator
  - Build all summaries
  - Resúmenes de Notas
  - Resumen de Nota
  - Resumen
  - Generar todos los resúmenes
description: Los Resúmenes de Notas te dan un breve precis en lenguaje sencillo de una nota para que puedas juzgarla sin abrirla. Constellation siempre honra un resumen que tú mismo escribiste — en el frontmatter o en un callout de resumen — y solo genera uno cuando no lo has hecho. Los resúmenes generados son extractivos (las propias frases más centrales de la nota), de solo lectura (nunca se escriben de vuelta en tu archivo) y se calculan enteramente en tu dispositivo. Los resúmenes aparecen por toda la app dondequiera que aparezca una nota — el **Clasificador**, los **Resultados de búsqueda**, la franja del **Editor**, el panel de **Backlinks**, el panel de **Outgoing Links**, el **Índice** y el tooltip al pasar el cursor en la **Vista del cielo**.
---

# Resúmenes de Notas

> *Si escribiste un resumen, Constellation usa el tuyo. Solo escribe uno cuando no lo has hecho — e incluso entonces, nunca en tu archivo.*

Un **Resumen de Nota** es un breve precis de una nota — unas pocas frases que te dicen de qué trata la nota de un vistazo. Los resúmenes son producidos por el **Note Summary Creator (NSC)**. Los verás **por toda la app, dondequiera que aparezca una nota**: bajo el título de cada tarjeta en la cola del **Clasificador** / **Source Review** (donde decides cómo clasificar una tarjeta sin abrir la nota detrás de ella), como una línea tenue en cursiva bajo cada coincidencia en los **Resultados de búsqueda** (que te dice de qué *trata* una nota, junto al fragmento que muestra por qué coincidió), como una franja fina sobre la nota cuando la abres en el **Editor** (para que la esencia de la nota esté en contexto mientras lees o escribes), bajo cada origen enlazado en el panel de **Backlinks** y bajo cada destino enlazado en el panel de **Outgoing Links** (para que una larga lista de notas relacionadas se escanee como ideas en vez de meros títulos), bajo cada mención de nota cuando expandes un término en el **Índice** (para que las notas de un término sean reconocibles de un vistazo), y dentro del tooltip al pasar el cursor en la **Vista del cielo** al apuntar a una burbuja (para que un grafo cargado siga siendo legible sin tener que hacer clic).

Este tema explica de dónde vienen los resúmenes, el estricto orden de precedencia que siempre prefiere *tus* palabras sobre las de la máquina, cómo se construyen los resúmenes generados, y cómo precalcularlos para toda una Library de una vez.

---

## Por qué existen los resúmenes

Cuando trabajas a través de una cola de revisión de cientos de tarjetas, el título por sí solo a menudo no basta para recordar lo que una nota realmente dice. Abrir cada nota para refrescar tu memoria rompe tu flujo. Un resumen de un vistazo bajo el título arregla eso: lees tres frases, recuerdas la nota, tomas la decisión, sigues adelante.

Pero un resumen es también un pequeño acto de autoría. Si ya has destilado una nota con tus propias palabras — en un campo `summary:` o un callout `> [!summary]` — entonces *ese* es el resumen que debería mostrarse, no la conjetura de una máquina. La primera regla de Constellation para los resúmenes es, por tanto, una regla sobre el respeto a tu escritura: **el tuyo gana.**

---

## De dónde viene un resumen — el orden de precedencia

Para cualquier nota, Constellation elige el resumen recorriendo esta lista y deteniéndose en el primero que existe:

1. **Tu resumen de frontmatter.** Si las propiedades de la nota contienen un campo `summary:`, `description:`, `abstract:` o `excerpt:` (comprobados en ese orden), su texto se usa **exactamente como lo escribiste**.
2. **Tu callout de resumen.** Si el cuerpo de la nota contiene un callout `> [!summary]`, `> [!abstract]` o `> [!tldr]`, su texto se usa **exactamente como lo escribiste** — incluyendo diacríticos y puntuación, preservados textualmente.
3. **Un resumen generado.** Solo si no escribiste ninguno de los anteriores, Constellation genera uno — leyendo la nota y extrayendo sus frases más centrales (ver abajo).
4. **Un texto de apertura como respaldo.** Para una nota que el motor no puede dividir en frases (por ejemplo, texto en un sistema de escritura sin puntuación clara de frases), muestra las líneas de apertura de la nota en lugar de un resumen clasificado.

> **La única regla que más importa:** los pasos 1 y 2 significan que un resumen que escribiste *nunca* se sobrescribe. Si ves un resumen generado en una nota que creías haber resumido, significa que el motor no encontró tu resumen donde lo busca — comprueba que tu campo de frontmatter sea uno de los cuatro nombres de arriba, o que tu callout sea uno de los tres tipos de arriba.

---

## Cómo se construye un resumen generado

Cuando Constellation tiene que generar un resumen (porque no escribiste uno), hace resumen **extractivo** — selecciona frases que ya están en tu nota, en lugar de inventar nueva prosa. El método es uno bien establecido (TextRank, Mihalcea & Tarau 2004):

1. **Dividir en frases.** El cuerpo de la nota se segmenta en frases usando el estándar Unicode para límites de frase, de modo que funciona a través de idiomas y sistemas de escritura.
2. **Leer el significado de cada frase.** Cada frase se convierte en una pequeña "huella de significado" numérica (un embedding) usando un modelo compacto en el dispositivo.
3. **Clasificar por centralidad.** Las frases que son más similares en significado a la mayoría de las *otras* frases puntúan más alto — estas son las frases que mejor representan la nota en su conjunto.
4. **Tomar las tres principales, en orden.** Las tres frases mejor clasificadas se muestran **en el orden en que aparecen en la nota**, de modo que el resumen se lee con naturalidad en lugar de fuera de secuencia.

Las notas muy largas se manejan con suavidad — el motor limita cuánto del cuerpo escanea y cuántas frases clasifica, de modo que resumir una nota enorme nunca ralentiza la app ni arriesga un fallo.

Como es extractivo, un resumen generado siempre está hecho de frases que realmente escribiste. Nunca pondrá palabras en tu boca.

---

## Los resúmenes son de solo lectura — File-Over-App

Constellation **nunca escribe un resumen generado de vuelta en tu nota.** Tus archivos `.md` son la fuente de verdad; el resumen que ves en una tarjeta se calcula al vuelo y se almacena en caché por separado, no se guarda en el texto del archivo ni en su frontmatter.

Esto es deliberado, y sigue el principio *File-Over-App* de Constellation: la app es una ventana hacia tus archivos, no un editor que los cambia silenciosamente. Si quieres que un resumen viva *en* la nota, escribe uno tú mismo (un campo `summary:` o un callout `[!summary]`) — y entonces, por la regla de precedencia de arriba, Constellation mostrará el tuyo y dejará de generar.

Todo se calcula **en tu dispositivo.** Ningún texto de nota se envía jamás a ningún sitio para ser resumido.

---

## Dónde aparecen los resúmenes, y cómo se rellenan

Los resúmenes surgen a lo largo de Constellation dondequiera que aparezca una nota:

- **Cola del Clasificador / Source Review** — bajo el título de cada tarjeta (la superficie original — ver *El Clasificador*).
- **Resultados de búsqueda** — una línea tenue en cursiva bajo cada coincidencia, debajo del fragmento. El fragmento muestra *por qué* un resultado coincidió con tu consulta; la línea de resumen muestra de qué *trata* la nota. Juntos te permiten escanear resultados sin abrir nada.
- **Editor** — una franja fina y atenuada sobre el cuerpo de la nota cuando abres una nota, para que la esencia de la nota esté en contexto mientras lees o escribes. La franja se oculta cuando aún no hay resumen (una nota recién creada, o una cuyo resumen aún se está calculando).
- **Panel de Backlinks** — bajo cada fila de origen que enlaza a la nota que estás leyendo. Una larga lista de menciones entrantes se vuelve escaneable: lees la esencia de la nota enlazante como una sola línea en cursiva bajo su título, sin tener que abrir cada una para recordar qué es.
- **Panel de Outgoing Links** — bajo cada fila de destino al que enlaza la nota que estás leyendo. La misma forma que los Backlinks; ves de un vistazo de qué trata cada conexión saliente.
- **Índice** — cuando expandes un término, cada nota que usa el término muestra el resumen como una línea tenue bajo su título (y bajo el fragmento de contexto coincidente, cuando lo hay). Un término que aparece en docenas de notas se convierte en una lista de ideas en lugar de meros nombres de archivo.
- **Hover en la Vista del cielo** — cuando pasas el cursor sobre una burbuja en el grafo de la **Vista del cielo**, el tooltip flotante muestra el nombre de la nota en la primera línea y el titular de su resumen en una segunda línea en cursiva, para que puedas leer lo que una burbuja *significa* sin salir del grafo.

Por defecto los resúmenes se rellenan **de forma perezosa y suave**: a medida que las tarjetas entran en vista, a medida que aparecen los resultados de búsqueda, a medida que abres una nota, o a medida que expandes un término / pasas el cursor sobre una burbuja, Constellation calcula los resúmenes faltantes de a pocos, deteniéndose siempre que un escaneo de clasificación de Library está en ejecución para que los dos nunca compitan por recursos. Esto mantiene la app responsiva — puedes ver brevemente una tarjeta / resultado / nota abierta / fila / tooltip antes de que aparezca su resumen, y luego el resumen aparece un momento después.

Si prefieres tener cada resumen listo de antemano — para que cada superficie muestre los resúmenes instantáneamente — usa **Generar todos los resúmenes**.

---

## Generar todos los resúmenes — precalcular toda la Library

El botón **Generar todos los resúmenes** (en el encabezado del **Clasificador**) precalcula un resumen para **cada nota que no tenga ya uno actual**, de modo que las tarjetas muestren su resumen instantáneamente en lugar de irse rellenando a medida que te desplazas.

**Para usarlo:**

1. Abre el **Clasificador** (el icono de tarjetas apiladas en el dock izquierdo).
2. Haz clic en **Generar todos los resúmenes** en el encabezado. El botón cambia a *Generando resúmenes de notas…*.
3. El progreso aparece en la **barra de estado** en la parte inferior de la ventana — puedes seguir trabajando mientras se ejecuta.
4. Para detenerlo antes, usa el control **Cancelar** en la franja de progreso de la barra de estado. Una ejecución parcial está bien; retoma donde lo dejó la próxima vez.

Algunas cosas que vale la pena saber:

- Se ejecuta **solo cuando lo pides** — nunca arranca por sí solo, así que nunca puede ralentizar el inicio de la app.
- Se ejecuta **en segundo plano** en un hilo separado; el tecleo y la navegación siguen siendo instantáneos.
- Es **reanudable** — si lo cancelas, o cierras la app a mitad de ejecución, la siguiente ejecución continúa desde donde se detuvo en lugar de empezar de nuevo.
- Solo calcula los resúmenes que están **ausentes o desactualizados** — las notas cuyo resumen ya está actual se omiten, así que una segunda ejecución es rápida.

---

## Asegurarte de que se use tu propio resumen

En una tarjeta, el resumen aparece bajo una sola etiqueta de **Resumen** — la tarjeta no señala si el texto vino de ti o del motor. Lo que decide eso es la precedencia de arriba: si una nota tiene uno de los campos de frontmatter o uno de los callouts de resumen, Constellation muestra *ese* y nunca genera uno.

Así que si una nota muestra un resumen que se lee como si la máquina lo hubiera elegido, esa nota no tiene ni un resumen de frontmatter ni un callout de resumen — y la solución es añadir uno:

- Añade un campo `summary:` (o `description:` / `abstract:` / `excerpt:`) al frontmatter de la nota, **o**
- Añade un callout `> [!summary]` (o `[!abstract]` / `[!tldr]`) al cuerpo.

La próxima vez que se calcule el resumen de esa nota — cuando su tarjeta cargue de nuevo, o después de que ejecutes **Generar todos los resúmenes** — tus palabras toman el control.

---

## Flujos de trabajo comunes

**"Una nota muestra un resumen de máquina, pero yo escribí uno."**
Constellation no encontró tu resumen donde lo busca. Asegúrate de que tu campo de frontmatter se llame `summary`, `description`, `abstract` o `excerpt`, **o** que tu callout sea `[!summary]`, `[!abstract]` o `[!tldr]`. Luego vuelve a abrir el Clasificador (o haz clic en *Generar todos los resúmenes*) para refrescar.

**"Quiero que cada tarjeta muestre su resumen en el instante en que abro el Clasificador."**
Haz clic en **Generar todos los resúmenes** una vez y déjalo terminar. Después de eso, los resúmenes están precalculados y aparecen inmediatamente.

**"Quiero que el resumen sea parte de la nota misma, en disco."**
Escríbelo tú mismo — añade un campo `summary:` de frontmatter o un callout `> [!summary]`. Constellation mostrará entonces tu versión (y dejará de generar uno), y tus palabras viven en el archivo donde cualquier otra app puede leerlas también.

---

## Temas relacionados

- **The Cataloger** — el hogar de página completa donde aparecen los resúmenes bajo cada tarjeta, y donde vive *Generar todos los resúmenes*.
- **Source Review** — las tarjetas de clasificación sobre las que se sitúan los resúmenes.
- **Properties** — los campos de frontmatter `summary:` / `description:` / `abstract:` / `excerpt:`, y cómo añadirlos.
- **Editing and Formatting** — cómo escribir un callout `> [!summary]` en una nota.

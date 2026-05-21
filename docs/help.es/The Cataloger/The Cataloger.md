---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: es
source: docs/help.uConstellation.World/The Cataloger/The Cataloger.md
aliases:
  - The Cataloger
  - Cataloger
  - Classify notes
  - Classification home
  - CECE home
  - Scan library
  - Clasificador
  - Clasificar notas
  - Inicio de clasificación
description: El Clasificador es el hogar a escala de universo para clasificar tus notas. Es la vista de página completa del dock donde ejecutas el Constellation Epistemic Content Engine (CECE) sobre tu Library, clasificas cualquier nota a demanda, generas resúmenes de notas y trabajas a través de la cola de revisión. Si Source Review es la tarjeta sobre la que actúas, el Clasificador es la sala donde lo haces.
---

# Clasificador

> *"Clasifica cada nota por su tipo de conocimiento y su fuente."*

El **Clasificador** es el hogar a escala de universo para la clasificación. Es una vista de página completa, abierta desde el dock izquierdo, que reúne en un solo lugar todo lo que necesitas para leer tus notas a través de la taxonomía de conocimiento de Constellation: un control para escanear toda la Library, una forma de clasificar cualquier nota individual a demanda, un botón para generar resúmenes de notas, y la cola de revisión en vivo donde Aceptas, Editas, Rechazas o desambiguas cada sugerencia.

Si has usado el panel **Source Review** de la barra lateral derecha, ya conoces las tarjetas. El Clasificador es el mismo motor y las mismas tarjetas, promovidos fuera de una estrecha pestaña de barra lateral y dados la ventana completa — además de dos cosas que la pestaña de barra lateral nunca tuvo: un selector de notas y un botón de "Generar todos los resúmenes".

---

## "El Clasificador" vs "los catalogadores" — una nota rápida sobre los nombres

Estos dos nombres se parecen a propósito, pero significan cosas diferentes:

- **El Clasificador** (esta vista) es el *lugar* — la sala de página completa donde ocurre la clasificación.
- **los catalogadores** (en plural) son las *seis lentes* dentro del motor — frontmatter, citas, raíces de palabras, notas vinculadas, notas similares y juicio de IA — cada una de las cuales lee una nota y vota. Cinco de las seis están activas hoy; la sexta (juicio de IA) está construida pero aún no encendida.

Así que: abres **el Clasificador**, y dentro de él **los catalogadores** hacen la lectura. La maquinaria de seis lentes se explica completamente en el tema **Source Review** — este tema trata sobre la sala.

---

## Qué es

El Clasificador responde una pregunta: **«¿Cómo está clasificada cada nota en mi universo — y qué sigue necesitando mi decisión?»**

Está construido alrededor de cuatro cosas apiladas de arriba abajo:

1. **Un encabezado con tres acciones** — *Clasificar una nota…*, *Generar todos los resúmenes* e *Iniciar escaneo*.
2. **Una franja de progreso** — aparece solo mientras se ejecuta un escaneo de Library, mostrando hasta dónde ha llegado.
3. **La cola de revisión** — las mismas tarjetas de Aceptar / Editar / Rechazar / Desambiguar que el panel Source Review, ahora a ancho completo.
4. **Un resumen de nota bajo cada tarjeta** — un breve precis en lenguaje sencillo de la nota para que puedas decidir sin abrirla (ver *Resúmenes de Notas* abajo, y el tema dedicado **Note Summaries**).

Todo se ejecuta **en tu dispositivo**. Ninguna nota sale jamás de Constellation.

---

## Por qué importa

La clasificación es cómo Constellation convierte un montón de archivos `.md` en un cuerpo de conocimiento *moldeado* — cada nota colocada sobre dos ejes (de dónde provino el conocimiento, y qué tipo de conocimiento es). Esa forma es lo que potencia **Constellation Sight**, el panel de **Metadatos Epistémicos** y la búsqueda consciente de la taxonomía.

Pero la clasificación es un trabajo de muchas decisiones. Cuando tienes cientos de notas sin clasificar, hacerlo desde una delgada pestaña de barra lateral — una nota a la vez, sin forma de invocar una nota específica — es lento. El Clasificador existe para hacer el trabajo *sentable*: ábrelo una vez, dale toda la pantalla, y recorre tu Library en una sola sesión enfocada. El selector de notas te permite traer cualquier nota por nombre; los resúmenes te dejan juzgar una tarjeta sin salir de la sala; el control de escaneo siembra la cola en masa.

---

## Cómo abrirlo

1. En el **dock izquierdo** (la franja vertical de iconos en el borde extremo de la ventana), haz clic en el **icono de tarjetas apiladas** — tres tarjetas pequeñas superpuestas una sobre otra. Está entre los demás iconos del espacio de trabajo, como el ojo de Sight y la neurona del Nervous System.
2. El Clasificador se abre como una **vista de página completa**, ocupando el área de contenido.
3. Para cerrarlo: haz clic en la **(×)** en la parte superior derecha del encabezado, o presiona **Esc**. Vuelves a donde estabas.

> **Nota sobre Esc:** si el popover de búsqueda *Clasificar una nota…* está abierto, presionar **Esc** cierra solo el popover y deja el Clasificador abierto. Presiona **Esc** de nuevo (con el popover cerrado) para cerrar el Clasificador en sí.

---

## Lo que ves

### El encabezado — tres acciones

En la parte superior del Clasificador, tres controles se sitúan lado a lado:

| Control | Lo que hace |
|---|---|
| **Clasificar una nota…** | Abre una pequeña caja de búsqueda. Escribe unas pocas letras del título de cualquier nota, elígela de los resultados, y el motor la clasifica en el acto — sin necesidad de abrir la nota primero. La nueva sugerencia aparece en la cola de abajo. |
| **Generar todos los resúmenes** | Precalcula un resumen breve para cada nota que no tenga ya uno. Se ejecuta silenciosamente en segundo plano; el progreso se muestra en la barra de estado en la parte inferior de la ventana; puedes cancelar en cualquier momento. (Detallado en el tema **Note Summaries**.) |
| **Iniciar escaneo** | Ejecuta el motor sobre tu **Library activa completa** de una vez, encolando una sugerencia para cada nota que aún no esté clasificada. Mientras se ejecuta, el botón muestra *En ejecución…* y aparece una franja de progreso bajo el encabezado. |

### La franja de progreso

Directamente bajo el encabezado, aparece una franja delgada **solo mientras se ejecuta un escaneo de Library**. Muestra cuántas notas se han procesado y te deja ver el escaneo completarse. Cuando no se ejecuta ningún escaneo, la franja está oculta y la cola se sitúa justo bajo el encabezado.

### La cola de revisión

El grueso del Clasificador es la **cola de revisión** — las mismas tarjetas que ves en el panel Source Review, solo que a ancho completo. Cada tarjeta muestra una nota, la lectura del motor sobre cómo encaja en tu taxonomía (Source × Content Type), los seis pequeños puntos de los catalogadores, y las acciones que puedes tomar:

- **Accept** — escribe la sugerencia del motor en la nota y limpia la tarjeta.
- **Edit** — eliges los valores tú mismo desde un árbol.
- **Reject** — limpia la tarjeta sin escribir nada.
- **Disambiguate** — en una tarjeta "split", eliges el valor correcto entre los chips candidatos.

La mecánica completa de las tarjetas — los puntos de colores, los regímenes de confianza, la desambiguación de hermanos, los chips de filtro de la cola, "Approve all" y la calibración por Library — está documentada en el tema **Source Review**. El Clasificador usa ese panel exacto; nada sobre las tarjetas cambia entre la barra lateral y la vista de página completa.

### El resumen de nota bajo cada tarjeta

Bajo el título de cada tarjeta se sitúa una breve línea de **Resumen** — unas pocas frases que te dicen de qué trata la nota, para que puedas juzgar la tarjeta sin abrir la nota. Esto es producido por el **Note Summary Creator (NSC)**; ver la siguiente sección y el tema **Note Summaries**.

---

## Clasificar una sola nota — el selector de notas

El botón *Clasificar una nota…* resuelve un problema sencillo: en la pestaña de barra lateral, solo podías clasificar la nota que tenías abierta en ese momento. El Clasificador no tiene "nota abierta", así que te da una forma de invocar cualquier nota por nombre.

**Para clasificar una nota:**

1. Haz clic en **Clasificar una nota…**. Se despliega una caja de búsqueda con el marcador de posición *Buscar notas…*.
2. Empieza a escribir el título de la nota. Tras una breve pausa, las notas coincidentes aparecen en una lista (hasta diez).
3. Haz clic en la nota que quieras. El motor la clasifica, el popover se cierra, y una tarjeta nueva para esa nota aparece en la cola de abajo.
4. Si algo sale mal (un raro error del motor), el mensaje aparece dentro del popover para que sepas que la clasificación no se ejecutó.

No tienes que abrir la nota, y no pierdes tu lugar en la cola. Esta es la forma más rápida de clasificar una nota específica que tengas en mente.

---

## Resúmenes de Notas (NSC) dentro del Clasificador

Cada tarjeta en la cola lleva un breve **Resumen** de su nota, mostrado bajo el título. El resumen es producido por el **Note Summary Creator (NSC)** y sigue una regla firme: **si escribiste un resumen, el motor usa el tuyo; solo genera uno cuando no lo has hecho.**

El orden de precedencia es:

1. **Tu resumen de frontmatter** — un campo `summary:`, `description:`, `abstract:` o `excerpt:` en las propiedades de la nota. Usado exactamente como lo escribiste.
2. **Tu callout de resumen** — un bloque `> [!summary]`, `> [!abstract]` o `> [!tldr]` en el cuerpo de la nota. Usado exactamente como lo escribiste, diacríticos y todo.
3. **Un resumen generado** — solo si no escribiste ninguno de los anteriores. Constellation lee la nota, encuentra sus frases más centrales, y muestra las tres principales en su orden original.

El motor **nunca escribe un resumen generado de vuelta en tu nota** — tus archivos `.md` son la fuente de verdad y el Clasificador solo los *lee*.

El botón **Generar todos los resúmenes** precalcula los resúmenes para toda la Library en segundo plano, de modo que las tarjetas muestren su resumen instantáneamente en lugar de irse rellenando a medida que te desplazas. El detalle completo — incluyendo cómo se producen los resúmenes generados y qué hacer si un resumen se ve mal — está en el tema **Note Summaries**.

---

## Lo que el Clasificador *no* hace

- **No clasifica automáticamente en segundo plano por defecto.** Los escaneos son algo que *inicias*. (Hay un modo de segundo plano opcional en Configuración > Inteligencia > CECE, desactivado por defecto — ver **Source Review**.)
- **No llama a ningún servicio en la nube.** Los cinco catalogadores activos son heurísticos y locales. La sexta lente (juicio de IA, un modelo de lenguaje local) está integrada en el diseño pero aún no encendida, así que permanece silenciosa en cada tarjeta hoy.
- **No cambia la redacción de tus notas.** Aceptar una tarjeta escribe *propiedades* de clasificación (los campos de frontmatter `sources:` y `content_type:`). Nunca edita tu prosa, y nunca escribe un resumen generado en el archivo.

---

## Flujos de trabajo comunes

**"Acabo de abrir el Clasificador por primera vez — ¿por dónde empiezo?"**
Haz clic en **Iniciar escaneo** para encolar una sugerencia para cada nota no clasificada en la Library. Mira la franja de progreso llenarse. Luego baja por la cola, aceptando las que el motor acertó y desambiguando las divididas. Los resúmenes bajo cada tarjeta te dejan decidir rápidamente.

**"Quiero clasificar una nota específica, no toda la Library."**
Haz clic en **Clasificar una nota…**, escribe su título, haz clic en ella. Aparece una tarjeta en la cola. Acéptala o edítala.

**"Mis tarjetas tardan un momento en mostrar sus resúmenes."**
Haz clic en **Generar todos los resúmenes** una vez. Precalcula el resumen de cada nota en segundo plano (progreso en la barra de estado). Después de que termine, los resúmenes aparecen instantáneamente.

**"La cola tiene cientos de tarjetas — ¿cómo me concentro?"**
Usa los chips de filtro sobre la cola (documentados en **Source Review**): empieza con *Catalogers agreed* y *Approve all* para limpiar las fáciles, luego aborda las tarjetas divididas.

---

## Temas relacionados

- **Source Review** — las tarjetas en sí: los seis catalogadores, los puntos de colores, los regímenes de confianza, la desambiguación de hermanos, los filtros de cola, "Approve all" y la calibración por Library. El Clasificador incrusta este panel.
- **Note Summaries** — cómo se produce la línea de Resumen bajo cada tarjeta, la precedencia que prioriza al autor, y el backfill de *Generar todos los resúmenes*.
- **Cognitive Engine** — la filosofía más amplia de formulación de conocimiento en la que encaja la clasificación.
- **Epistemic Metadata** — las propiedades `sources:` y `content_type:` que escribe la clasificación, y cómo leerlas.
- **Constellation Sight** — la vista espacial que potencia la clasificación Source × Content Type.

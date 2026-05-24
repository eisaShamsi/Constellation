---
translation_status: AI-generated 2026-05-24 — native-speaker review recommended
language: es
source: docs/help.uConstellation.World/The Digest/The Digest.md
aliases:
  - The Digest
  - Universe Digest
  - Digest
  - Digest pane
  - Resumen del Universo
  - Resumen
  - Panel del Resumen
description: El Resumen del Universo es un panel en el dock izquierdo que muestra cada nota de tu base de conocimiento al nivel de titular-de-resumen — escalonado Library → Carpeta → Nota — para que puedas hojear todo el Universo sin abrir nada. Haz clic en una fila para expandirla y ver el resumen completo en línea. El filtro reduce toda la lista; el orden alterna entre recencia (por defecto) y alfabético. Lee los mismos resúmenes que ves en todos los demás lugares; sin computación adicional; sin espacio en disco adicional.
---

# Resumen del Universo

> *Piensa en el Resumen como un índice de contenidos para tu mente — no una lista de archivos, una lista de ideas.*

El **Resumen del Universo** es el lugar para hojear toda tu base de conocimiento al nivel del *significado*. En vez de un árbol de archivos (solo nombres) o la Vista del cielo (solo formas), el Resumen te muestra, debajo de cada nota, **la única frase que dice de qué trata la nota**. Toca una fila y el resumen completo de varias frases se expande en línea. Puedes leer la sustancia de cincuenta notas en un minuto, sin abrir ni una.

Vive en tu **dock izquierdo**, junto al árbol de Archivos, el Navegador de Notas y la Vista del cielo — una de las cuatro formas que Constellation te ofrece para navegar.

---

## Por qué existe el Resumen

Un árbol de archivos te dice qué *tienes*. Una búsqueda te dice qué *preguntaste*. El Resumen te dice qué *sabes*.

Cuando tu Universo crece más allá de unos pocos cientos de notas, "abrir cada una para recordar qué dice" se vuelve imposible. Necesitas una manera de leer la **esencia** de cada nota a la velocidad del desplazamiento — y una manera de expandir cualquier esencia en el resumen completo en el momento en que quieras pensar más cuidadosamente sobre ella. Eso es el Resumen.

Es el tercer pilar del Core Plug-In **Note Summary Creator (NSC)**:
- **Pilar 1**: un motor de resúmenes (Fase 1 / MIG-043).
- **Pilar 2**: un servicio que coloca el resumen dondequiera que aparezca una nota (Fase 2 / MIG-044 — Clasificador, Resultados de búsqueda, franja del Editor, Backlinks, Outgoing Links, el Índice, tooltip al pasar el cursor en la Vista del cielo).
- **Pilar 3**: esta vista — el Resumen del Universo (Fase 3 / MIG-045).

---

## Abrir el Resumen

En la **barra lateral izquierda**, haz clic en el **icono del Resumen del Universo** (una pequeña lista con un círculo en la esquina) — es el cuarto icono en la fila, junto al árbol de Archivos / Navegador de Notas / Vista del cielo. La barra lateral cambia al panel del Resumen.

Para volver atrás, haz clic en cualquiera de los otros tres iconos (o presiona **Escape**).

---

## Lo que ves

De arriba hacia abajo:

1. **Barra de herramientas.** Un campo de búsqueda + un pequeño icono de reloj (el conmutador de orden, por defecto "por recencia").
2. **Cabeceras de Library.** Barras moradas en mayúsculas — una por cada library en tu Universo. Cada una muestra el nombre de la library y un conteo de cuántas notas contiene.
3. **Cabeceras de carpeta.** Pequeñas etiquetas atenuadas — una por cada carpeta *que contiene notas*. Las notas que viven en la raíz de la library no obtienen cabecera de carpeta.
4. **Filas de notas.** Cada fila tiene:
   - Un chevron (▶) a la izquierda — haz clic en él para expandir la fila.
   - El **nombre de la nota** en color de acento interactivo — haz clic para **abrir la nota** en el editor.
   - Una línea tenue en cursiva debajo del nombre — el **titular del resumen** (el mismo que aparece en cualquier otra superficie de Fase 1/2).

---

## Expandir una fila para leer el resumen completo

Haz clic en el **chevron** (▶) a la izquierda de una fila — o haz clic en la **línea del titular en cursiva** misma. El chevron rota a ▼ y el **resumen completo de varias frases** aparece en línea debajo del titular, ajustándose naturalmente a tantas líneas como necesite.

Haz clic en el chevron (o titular) de nuevo para colapsar.

La división "haz clic en el chevron para expandir, haz clic en el nombre para abrir" mantiene los dos gestos distintos: puedes expandir para *leer sobre* una nota, luego seguir desplazándote más allá; solo cuando haces clic en el nombre la nota realmente se abre y toma el foco.

---

## Filtrar

Escribe en el **campo de búsqueda** en la parte superior. La lista se reduce mientras escribes — solo las notas cuyo **nombre, titular o resumen completo** contenga tu consulta permanecen visibles. Las cabeceras de library y las cabeceras de carpeta con cero notas coincidentes desaparecen por completo (sin cabeceras vacías).

Borra el campo (botón × o retroceso) para restaurar la lista completa.

El filtro es **instantáneo** — Constellation no toca tu disco ni la base de datos. Lee los resúmenes ya en memoria, así que incluso un Universo de 10.000 notas se filtra a velocidad de tipeo.

---

## Orden: recencia o alfabético

Haz clic en el **icono de reloj** en la barra de herramientas para alternar entre dos modos de orden:

- **Recencia** (por defecto) — dentro de cada carpeta, las notas aparecen en orden de **tiempo de creación, más recientes primero**. Las carpetas dentro de una library se ordenan por la nota más reciente que contienen (para que la carpeta más activa aparezca primero). Este es el orden por defecto porque hace visible *en lo que has estado trabajando últimamente*.
- **Alfabético** — carpetas ordenadas por nombre, notas dentro de cada carpeta ordenadas por nombre. Haz clic de nuevo para volver a recencia.

El conmutador es por sesión; cierra y reabre el Resumen y vuelve a recencia.

---

## Federación: Universos hijos aparecen en línea

Si tu Universo tiene **Universos hijos enlazados** (cUniverses), cada library de un Universo hijo aparece en el Resumen como **su propia cabecera de Library par**, junto a las libraries del Universo padre. El Resumen es una vista unificada de todo lo alcanzable desde este Universo, no solo las libraries que físicamente viven aquí.

(Una futura actualización de Constellation añadirá un conmutador de encendido/apagado para ocultar libraries de Universos hijos del Resumen temporalmente; por ahora siempre aparecen.)

---

## Cómo el Resumen permanece rápido en Universos enormes

El Resumen está **virtualizado**: renderiza solo las filas actualmente visibles en tu puerto de desplazamiento, no todo el árbol. Un Universo de 10.000 notas se desplaza tan suavemente como uno de 50. A medida que las filas se desplazan a la vista, sus resúmenes se obtienen en lotes desde la caché en memoria de Constellation (la misma caché que alimenta todas las demás superficies de Fase 1/2 — sin trabajo separado, sin almacenamiento separado).

El Resumen nunca vuelve a leer tus notas del disco. Nunca vuelve a calcular resúmenes. Es una vista de **lectura** sobre la misma tabla `note_summaries` que el motor llena desde la Fase 1.

---

## Flujos de trabajo comunes

**"Quiero ver en lo que he trabajado esta semana."**
Abre el Resumen con orden = Recencia (por defecto). Las notas más recientemente creadas aparecen en la parte superior de cada library/carpeta. Escanea los titulares.

**"Estoy buscando una nota medio recordada sobre X."**
Abre el Resumen. Escribe X (una palabra que aparecería en el título, titular o resumen completo de la nota). La lista se reduce a candidatos. Haz clic en los chevrons para leer resúmenes completos; haz clic en el nombre para abrir el ganador.

**"Quiero escribir una revisión de arriba hacia abajo de mi Library."**
Abre el Resumen, orden = Alfabético. Recorre los titulares en orden. Haz clic en los chevrons para leer resúmenes más completos cuando algo te atrape. Usa esto como espina dorsal de una nueva nota MOC (Map of Content).

**"Estoy explorando un cUniverse federado por primera vez."**
Abre el Resumen. Desplázate más allá de tus propias libraries hasta las libraries del cUniverse — son filas pares. Lee los titulares para aprender qué contiene el Universo enlazado, sin abrir nada de él.

---

## Lo que NO está en el Resumen

- **Menú contextual de clic derecho** en filas — abrir en una nueva pestaña, archivar, etc. (Para v1, las acciones principales son clic-nombre-para-abrir y clic-chevron-para-expandir. Una futura actualización añadirá un menú contextual.)
- **Agrupaciones personalizadas** — Library → Carpeta es la única estratificación para v1. (Sin "agrupar por etiqueta" o "agrupar por etapa" todavía.)
- **Arrastrar para reordenar** — el Resumen es de solo lectura; el orden proviene de reglas, no de orden manual.
- **Controles de clasificación tipo Clasificador** — el Resumen es una vista de *navegación*; la clasificación vive en el **Clasificador** (panel separado).

---

## Temas relacionados

- **Resúmenes de Notas** — de dónde vienen los resúmenes, la regla de precedencia (el tuyo gana), y la lista completa de superficies que los muestran.
- **El Clasificador** — el hogar de *Generar todos los resúmenes* (precalcular todos los resúmenes de tu Library de una vez para que el Resumen se llene al instante).
- **Vista del cielo** — la vista de *forma* de tu conocimiento (burbujas + enlaces); el Resumen es su vista de *significado* complementaria.
- **Formulación del Conocimiento** — por qué Constellation organiza el conocimiento por *conexión* y *resumen*, no solo por almacenamiento de archivos.

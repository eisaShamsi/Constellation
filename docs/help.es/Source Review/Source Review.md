# Source Review

> **Nota de traducción:** Este tema de ayuda es una traducción
> generada por IA a partir de la versión canónica en inglés en
> `help.uConstellation.World/Source Review/Source Review.md`.
> Pendiente de revisión por hablantes nativos. Por favor, envíe
> correcciones a través del repositorio del proyecto.

*(Constellation Epistemic Content Engine — CECE)*

El panel Source Review es donde Constellation le pide que revise las clasificaciones producidas por el **Constellation Epistemic Content Engine** (CECE). Cada tarjeta en la cola muestra una nota + la lectura del motor sobre cómo encaja esa nota en su taxonomía de conocimiento. Usted Acepta, Edita, Rechaza o elige un chip de Sibling Disambiguation — y con el tiempo el motor aprende la forma de su Library.

Este tema explica cada parte de una tarjeta de Source Review, qué significan los puntos de colores, cuándo confiar en el motor y cómo navegar por cientos de tarjetas sin desplazarse eternamente.

> **Dos lugares, un panel.** Las tarjetas descritas aquí aparecen tanto en la pestaña **Source Review** de la barra lateral derecha como en la vista de página completa del **Clasificador** (el icono de tarjetas apiladas en el dock izquierdo). Son el mismo panel y el mismo motor. El Clasificador da a la cola la ventana entera más un selector de notas y un botón de "Generar todos los resúmenes" — ver el tema **Clasificador**. Todo lo que sigue se aplica a las tarjetas en cualquiera de los dos lugares.

---

## Lo que CECE realmente hace

Cuando usted clasifica una nota (clic derecho → "Sugerir fuentes y tipo de contenido", o mediante Configuración → botón Ejecutar escaneo), CECE ejecuta **seis catalogadores independientes** sobre la nota. Cada catalogador lee la nota a través de su propia lente — frontmatter, citas, raíces de palabras, notas vinculadas, notas similares, juicio de IA — y vota sobre dos preguntas:

- **Source (eje horizontal)**: ¿de dónde *proviene* este conocimiento? Ejemplos: testimonio (alguien me lo dijo), percepción (lo vi), inferencia (lo deduje), revelación (texto sagrado), y ocho más.
- **Content Type (eje vertical)**: ¿qué *tipo* de conocimiento es este? Ejemplos: estado epistémico (duda / certeza / creencia), contenido semántico (concepto / proposición / hecho / teoría), entrada sensorial, entidad simbólica, constructo de orden superior (cosmovisión / doctrina).

Los dos ejes son **independientes**. Una nota sobre "Dudo de la llegada del hombre a la Luna" es testimonio (alguien lo informó) en el eje source + estados-epistémicos/duda (la postura del usuario hacia ello) en el eje content-type.

Después de que los catalogadores votan, una **capa de síntesis** combina sus votos en una única clasificación por eje, con uno de tres regímenes de confianza:

- **Unanimous** — todos los catalogadores que se pronunciaron estuvieron de acuerdo
- **Strong majority** — la mayoría estuvo de acuerdo, uno disintió (la tarjeta muestra el nombre del disidente)
- **Split** — sin mayoría clara; el motor "se negó a asignar" y le pide a *usted* que elija

Todo se ejecuta **en su dispositivo**. Ninguna nota sale jamás de Constellation.

---

## Los dos ejes en lenguaje sencillo

### Source — *¿de dónde proviene este conocimiento?*

Once valores posibles más *no clasificable*:

- **Percepción** — observación sensorial de primera mano
- **Inferencia** — razonamiento a partir de premisas (deducción, inducción, analogía)
- **Testimonio** — informe de otra persona (una cita, una referencia, una fuente referenciada)
- **Transmisión masiva** — informes convergentes de muchos testigos independientes (sunita *al-tawatur*)
- **Comparación** — conocimiento por analogía a un caso conocido (legal *qiyās*, analogías científicas)
- **Postulación** — inferencia a la mejor explicación (*arthapatti*)
- **No-aprehensión** — conocimiento de la ausencia
- **Memoria** — recuerdo de experiencias pasadas
- **Disposición innata** — conocer pre-experiencial (*fitrah*)
- **Inspiración** — aprehensión mística o creativa (*kashf*)
- **Revelación** — transmisión de texto sagrado o profético (*al-wahy*)
- **No clasificable** — optar por no incluir esta clasificación

### Content Type — *¿qué tipo de conocimiento es este?*

Cinco ramas de nivel superior con sub-ramas:

- **Entradas sensoriales** — señales en bruto (visuales, acústicas, químicas, …)
- **Entidades simbólicas** — signos, símbolos, códigos
- **Contenidos semánticos** — conceptos, proposiciones, hechos, ideas, información
- **Estados epistémicos** — duda, creencia, opinión, certeza, conocimiento, ilusión
- **Constructos de orden superior** — teorías, doctrinas, cosmovisiones, paradigmas

Ambos ejes tienen varias capas de refinamiento bajo cada valor de nivel superior (p. ej. *epistemic-states/knowledge/by-content/propositional* es una hoja).

---

## Los seis catalogadores

Cada catalogador es una *lente* a través de la cual CECE lee una nota. La tarjeta de Source Review los muestra como **seis pequeños puntos de colores** en la esquina superior derecha. Pase el cursor sobre cualquier punto para ver su nombre + estado.

| Punto | Catalogador | Lo que lee |
|---|---|---|
| 🔵 azul | **Su frontmatter** (Autoridad de Usuario) | Los campos `sources:` y `content_type:` que ya ha establecido. Si ha clasificado la nota usted mismo, esta lente tiene *autoridad absoluta* — la síntesis adopta su elección y omite las demás. |
| 🌹 rosa | **Citas y estructura** (Estructural) | Citas, blockquotes, bloques de código, marcadores de teorema, frases de definición ("el concepto de X se define como…"), referencias a figuras. Lee la forma estructural de la nota. |
| 🟡 ámbar | **Raíces y léxico** (Lingüístico) | Análisis de raíces árabes (CAE), coincidencia de palabras clave en superficie, equivalencia de términos entre idiomas (Bridge). Captura clasificación consciente del árabe que los embeddings puros pierden. |
| 🟢 verde azulado | **Notas vinculadas** (Grafo) | Living Links tipados (`[[Note\|supports]]`, `[[Note\|contradicts]]`, etc.) a otras notas clasificadas. Hereda la clasificación de los vecinos cuando se agrupan. |
| 🟣 violeta | **Notas similares** (Semántico) | Similitud de embeddings con sus notas ya clasificadas (k-Nearest-Neighbor). Aporta el consenso cuando el vector de contenido de esta nota se agrupa con notas clasificadas. |
| 🟢 verde | **Juicio de IA** (Razonamiento) | Un LLM local (Qwen3-4B Q5_K_M) ejecutando inferencia con restricciones gramaticales. *Aún no activo* — cableado del modelo aplazado para una versión posterior. El punto permanece silencioso en cada tarjeta hoy. |

### Estado del punto

- **Lleno** — se pronunció + concuerda con la síntesis
- **Anillado** — se pronunció + disiente de la síntesis (esta lente eligió algo diferente)
- **Contorno discontinuo** — silencioso (sin señal en esta lente para esta nota)

El cúmulo de puntos es el indicador de un vistazo de la salud del ensamble. Una tarjeta con los seis puntos llenos es la clasificación más fuerte posible del motor (raro). Una tarjeta con uno o dos puntos anillados está mostrando su razonamiento honestamente — las lentes discreparon.

---

## Los tres regímenes de confianza

Después de que los catalogadores votan, CECE etiqueta cada eje con uno de tres regímenes:

- **Unanimous** — todos los catalogadores que se pronunciaron eligieron el mismo valor primario. La tarjeta no tiene una píldora especial.
- **Strong majority (una disensión)** — la mayoría estuvo de acuerdo; un disidente se muestra por nombre. La tarjeta tiene una píldora morada "Strong majority" en el encabezado.
- **Split** — sin mayoría clara. La tarjeta tiene una píldora dorada "Catalogers split — needs your call", **un borde izquierdo dorado** y un formulario de Sibling Disambiguation con chips para que elija.

Cada eje obtiene su propio régimen de forma independiente. Una tarjeta puede ser Unanimous en horizontal + Split en vertical (o viceversa). La píldora del encabezado resume el peor régimen entre ambos ejes.

---

## Sibling Disambiguation

Cuando un eje es Split, CECE se niega a adivinar y en su lugar muestra los valores candidatos como **chips de radio** bajo un mensaje:

> *"Los catalogadores se dividieron entre estos candidatos. Elija cuál encaja mejor con la nota:"*

Usted hace clic en un chip → el motor escribe esa elección en el frontmatter de la nota, elimina la tarjeta de la cola y actualiza los datos de fiabilidad por Library.

Si el OTRO eje estaba resuelto (Unanimous o Strong majority), CECE *también* escribe el valor de ese eje al mismo tiempo — de modo que un solo clic en un chip termina ambos ejes, no solo el que eligió. La misma tarjeta nunca le pregunta dos veces.

Si ambos ejes son Split, elija un chip por eje (dos clics).

---

## El rastro de razonamiento

Cada tarjeta tiene un alternador **"▸ ¿Por qué esta clasificación?"** (o "▾ Ocultar razonamiento" si está abierto). Al expandirlo se muestra una fila por catalogador que se pronunció:

- **Punto del color de la lente** que coincide con el cúmulo de puntos
- **Etiqueta del catalogador** (p. ej. "Raíces y léxico")
- **Confianza autoinformada** entre corchetes: `[high]` `[medium]` `[low]`
- **Razonamiento de una línea** que explica qué se activó (p. ej. *"Linguistic match: vertical → semantic-contents/concept (weight 0.80)"*)
- **Chips de regla amistosos** debajo del razonamiento, como `Surface keyword match`, `Side-channel preference rule`, `Arabic root match (CAE)` — estas son las reglas específicas que activó cada catalogador

Durante sus **primeras 50 revisiones** el rastro se expande automáticamente en cada tarjeta para que pueda construir intuición sobre cuándo confiar en el motor. Después de eso, el rastro se contrae a bajo demanda en tarjetas Unanimous y permanece auto-expandido en tarjetas Strong majority + Split (donde el desacuerdo es informativo).

Puede anular este valor predeterminado en cualquier momento en Configuración → Intelligence → CECE → Visibilidad del rastro de razonamiento:

- **Mostrar siempre** — abierto en cada tarjeta
- **Solo en desacuerdo (predeterminado)** — abierto en tarjetas Split + Strong majority, además de las primeras 50 revisiones
- **Ocultar siempre** — clic manual requerido para expandir

---

## El filtro de composición de la cola

Sobre la barra de conteo hay **cinco chips** que dividen su cola por el tipo de decisión que cada tarjeta necesita de usted:

| Chip | Muestra |
|---|---|
| **All** *(predeterminado)* | la cola completa |
| **Both axes need your call** | tarjetas donde TANTO horizontal COMO vertical son Split |
| **Source needs your call** | tarjetas donde horizontal es Split + vertical está resuelto |
| **Content type needs your call** | tarjetas donde vertical es Split + horizontal está resuelto |
| **Catalogers agreed** | tarjetas donde ningún eje es Split — candidatos rápidos para sello |

Cada chip muestra su recuento de cubo (p. ej. *"Source needs your call (43)"*). Los cubos vacíos están atenuados y deshabilitados. Hacer clic en un chip vuelve a renderizar las tarjetas visibles; la barra de conteo y la matemática de Approve All siempre operan sobre la cola **completa** independientemente del filtro activo, de modo que siempre puede ver los totales reales.

El filtro resuelve el problema de la aguja en el pajar cuando su cola tiene cientos de tarjetas. ¿Quiere borrar primero todos los candidatos para sello? Haga clic en **Catalogers agreed** y luego en **Approve all**. ¿Quiere centrarse en los casos más difíciles? Haga clic en **Both axes need your call**.

---

## El resumen de nota bajo cada tarjeta

Bajo el título de cada tarjeta se sitúa una breve línea de **Resumen** — unas pocas frases que te dicen de qué trata la nota, para que puedas decidir cómo clasificarla sin abrirla. Constellation siempre muestra un resumen que *tú* escribiste (un campo de frontmatter `summary:` / `description:` / `abstract:` / `excerpt:`, o un callout `> [!summary]` / `[!abstract]` / `[!tldr]` en el cuerpo) y solo genera uno cuando no lo has hecho. Los resúmenes generados son extractivos — las propias frases más centrales de la nota — y nunca se escriben de vuelta en tu archivo. El detalle completo está en el tema **Note Summaries**.

---

## Acciones por tarjeta

Cada tarjeta tiene cuatro acciones en la parte inferior (o tres en tarjetas Split donde Disambig reemplaza Accept/Edit):

- **Accept** — escribe el valor primario de la síntesis del motor en ambos ejes en el frontmatter de la nota, elimina la tarjeta de la cola. Actualiza la fiabilidad por catalogador.
- **Edit** — abre un selector de árbol para ambos ejes; usted elige los valores manualmente. Misma actualización de fiabilidad.
- **Reject** — limpia la tarjeta sin escribir nada. El motor volverá a sugerir si reclasifica más tarde. (El rechazo NO actualiza la fiabilidad — el usuario "no quiere ninguna de estas" es ambiguo como señal de retroalimentación.)
- **Chip de Sibling Disambiguation** — en tarjetas Split, haga clic en uno de los chips candidatos. Escribe el valor elegido (y escribe automáticamente el otro eje si estaba resuelto).

---

## El período de calibración de confianza

Sus primeras **50 revisiones** de tarjetas clasificadas por CECE son un *período de calibración de confianza*. Durante este tiempo el rastro de razonamiento se expande automáticamente en cada tarjeta (independientemente del régimen), y un banner discreto en la parte superior del panel le recuerda: *"Showing reasoning trails until you review N more cards — helps you learn when to trust the catalogers."*

Después de 50 revisiones el banner desaparece y los rastros se contraen al comportamiento predeterminado bajo demanda. Puede anularlo a través de Configuración si desea mantenerlos siempre abiertos o siempre cerrados.

El propósito del período de calibración: CECE es un sistema probabilístico que mejora a medida que usted lo corrige (fiabilidad por Library). Ver *por qué* cada catalogador votó como lo hizo durante las primeras 50 revisiones le permite construir su propia intuición sobre cuándo sus conclusiones son confiables sobre el contenido específico de esta Library.

---

## Calibración por Library

Configuración → Intelligence → CECE → **Per-Library calibration** abre una tabla de solo lectura que muestra la precisión por eje de cada catalogador en la Library activa:

```
Cataloger          Horizontal      Vertical
─────────          ──────────      ────────
Your frontmatter   12/12 (100%)    4/4 (100%)
Citations          18/22 (82%)     6/8 (75%)
Wordstems          24/28 (86%)     20/26 (77%)
Linked notes       3/4 (uniform)   2/3 (uniform)
Similar notes      14/19 (74%)     12/19 (63%)
AI judgment        — (not running) — (not running)
```

Los números son recuentos correctos/totales. El porcentaje se muestra después de que un catalogador tiene 20+ correcciones en esa Library × eje (el umbral para datos de precisión estables). Por debajo del umbral, la etiqueta muestra **(uniform)** — el catalogador contribuye con votos ponderados uniformemente hasta que se acumulan suficientes datos.

Diferentes Libraries pueden tener precisiones por catalogador muy diferentes. El catalogador Lingüístico sobresale en Libraries con mucho árabe; el catalogador de Grafo sobresale en Libraries densamente vinculadas. La capa de síntesis utiliza los datos de calibración por Library para ponderar los votos — de modo que un catalogador que se ha equivocado el 70 % del tiempo en *esta* Library tiene sus votos infraponderados en la siguiente ronda de síntesis.

---

## Clasificación en segundo plano

La cola de Source Review puede crecer de dos maneras:

1. **Manual** (predeterminado) — usted hace clic derecho en una nota → "Sugerir fuentes y tipo de contenido", o activa Configuración → Ejecutar escaneo de clasificación.
2. **Segundo plano** — Configuración → Intelligence → CECE → Clasificación en segundo plano. Dos modos:
   - **On note save** — auto-clasifica cada nota ~1,5 segundos después de que deja de escribir (monta sobre el guardado debounced existente; nunca se dispara por pulsación de tecla).
   - **On app start** — escanea notas no clasificadas una vez por lanzamiento.

La clasificación en segundo plano está **desactivada de forma predeterminada**. Ambos modos en segundo plano se ejecutan en un hilo de fondo + emiten eventos de progreso; la escritura permanece instantánea; puede cancelar desde el encabezado del panel Source Review.

---

## Flujos de trabajo comunes

**"Acabo de instalar CECE — ¿por dónde empiezo?"**
Abra el panel Source Review. Haga clic derecho en 5-10 notas de su árbol de archivos → "Sugerir fuentes y tipo de contenido" para sembrar la cola. Vaya pasando por las tarjetas una a la vez. El rastro de razonamiento se expande automáticamente durante sus primeras 50 revisiones — léalo. Después de 5-10 tarjetas comenzará a ver qué catalogadores son fiables en su contenido.

**"Mi cola tiene 1.200 tarjetas — ¿dónde me concentro?"**
Use los chips de filtro. Empiece con **Catalogers agreed** (candidatos para sello) → haga clic en Approve all para limpiarlos. Luego **Source needs your call** + **Content type needs your call** para casos Split que necesitan una decisión cada uno. **Both axes need your call** es el conjunto más difícil; guárdelo para el final.

**"¿Cómo sé cuándo elegir Accept vs. Reject vs. Edit vs. Disambig?"**
- **Accept** cuando el valor primario de la síntesis coincide con su lectura de la nota.
- **Reject** cuando ninguna de las sugerencias encaja (p. ej. el motor se perdió algo que usted sabe sobre la nota).
- **Edit** cuando quiere un valor que no está en ninguna de las sugerencias.
- **Chip de Sibling Disambiguation** cuando la tarjeta es Split y uno de los candidatos es correcto.

**"¿Cómo veo en qué catalogadores confío más?"**
Abra Configuración → Intelligence → CECE → Per-Library calibration. La tabla muestra la precisión por catalogador a través de las correcciones que ha realizado en esta Library.

---

## Temas relacionados

- **Clasificador** (The Cataloger) — el hogar de página completa para estas tarjetas, con un selector de notas ("Clasificar una nota…") y un botón de "Generar todos los resúmenes".
- **Note Summaries** — cómo se produce la línea de Resumen bajo cada tarjeta, y la precedencia que prioriza al autor y siempre prefiere tus propias palabras.
- **Cognitive Engine** — la filosofía más amplia de formulación de conocimiento en la que CECE encaja.
- **Properties** — los campos `sources:` y `content_type:` del frontmatter en los que CECE escribe.
- **Knowledge Hierarchy** — cómo Source × Content Type encaja en la estructura Universe / Library / Folder / Note.


---

## La barra lateral derecha frente al Cataloger — dos lugares distintos

La pestaña **Source Review** de la barra lateral derecha y el **Cataloger** de página completa son ahora **superficies distintas**:

- **Barra lateral derecha → Source Review** muestra la sugerencia pendiente de la **nota que tienes abierta** — su propia tarjeta y sus controles por tarjeta *Accept / Edit / Reject*.
- **El Cataloger** (icono de tarjetas apiladas, dock izquierdo) muestra la cola de revisión de **todo el universo** — cada nota a la espera de una decisión — junto con las herramientas masivas **Approve all / Reject all** y los chips de filtro. Esas herramientas masivas viven *solo* aquí, nunca junto a una sola nota.

Puedes ajustar el tamaño del texto del Cataloger en **Style Setter → Cataloger → Text size**.
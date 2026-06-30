---
aliases:
  - Colores cognitivos
  - Estilo de etiquetas de propiedades
  - Estilo de insignias de taxonomía
  - Colores de madurez
  - Colores de confianza
  - Colores de origen
  - Colores de etapa
  - Colores de categoría de coincidencia
  - Menú de clic derecho
  - Menú contextual
  - Clic derecho en el cuerpo de la nota
  - Clic derecho en propiedad
  - Clic derecho en resultado de búsqueda
  - Unificar bajo demanda
description: Reestiliza las Etiquetas de propiedades y las Insignias de taxonomía del frontmatter, fija un único color compartido para cada estado cognitivo (Madurez, Confianza, Origen, Etapa, Categoría de coincidencia) para que todas las superficies se unifiquen bajo demanda, y usa los menús de clic derecho de toda la aplicación en el cuerpo de la nota, el panel de Propiedades y los resultados de búsqueda.
---

# Colores cognitivos y menús de clic derecho

Este tema cubre dos cosas que llegaron juntas: **dos nuevas categorías del Diseñador de estilo (Style Setter)** — **Propiedades** (reestilizar las pequeñas etiquetas de tu frontmatter) y **Colores cognitivos** (un control de color por estado cognitivo, compartido por toda la aplicación) — y los **menús de clic derecho de toda la aplicación** que ponen la acción adecuada a un clic de distancia sobre el cuerpo de la nota, sobre una propiedad del frontmatter y sobre un resultado de búsqueda.

> El Diseñador de estilo es el estudio de diseño a pantalla completa que abres desde **Configuración → Apariencia → «✦ Abrir Diseñador de estilo»**, o desde su propia pestaña **Diseñador de estilo** en la barra lateral de Configuración. Las dos categorías siguientes se sitúan en la lista de la izquierda de *Superficies*, junto a Interfaz, Editor, Enlaces y las demás. Para el comportamiento general del Diseñador — Inspeccionar, Conservar / Descartar / Restablecer, Estilos guardados — consulta [[Appearance and Themes]].

---

## Diseñador de estilo → Propiedades

La categoría **Propiedades** reestiliza las pequeñas etiquetas que aparecen dentro del **frontmatter** de una nota (su bloque de propiedades YAML) — las fichas que ves para `tags`, `aliases` y similares en el panel de Propiedades y en la parte superior de la nota. Hasta ahora eran fijas; ahora son tuyas para darles forma.

Abre el **Diseñador de estilo** y haz clic en **Propiedades** en la lista de la izquierda. El centro muestra una vista previa en vivo de las píldoras de propiedad; haz clic en un control de la derecha y la vista previa se reestiliza mientras editas. Dos elementos:

### Etiquetas de propiedades

Las fichas de etiqueta ordinarias del frontmatter (por ejemplo, cada valor de una lista `tags`). Cuatro controles:

- **Fondo de etiqueta** — el color de relleno de la ficha.
- **Texto de etiqueta** — el color del texto dentro de la ficha.
- **Radio de etiqueta** — cuán redondeadas están las esquinas de la ficha (0 px = cuadrada, hasta 20 px = totalmente redondeada).
- **Altura** — la altura de la ficha en píxeles (14–32 px).

### Insignias de taxonomía

Las píldoras usadas para valores de tipo taxonomía. Tres controles:

- **Fondo** — el color de relleno de la píldora.
- **Texto** — el color del texto dentro de la píldora.
- **Radio** — redondeo de esquinas (0–20 px).

> **Nada cambia hasta que tocas un control.** Cada valor empieza exactamente con el aspecto que tienes hoy, así que la categoría Propiedades deja tus notas con un aspecto idéntico hasta que eliges deliberadamente un color o arrastras un deslizador. Haz clic en **Conservar (Keep)** para guardar el aspecto en este Universo.

---

## Diseñador de estilo → Colores cognitivos

Constellation pinta tu **vocabulario cognitivo** en color — la *madurez* de una nota, la *confianza* de un enlace, de dónde *vino* una idea, en qué *etapa* de su vida se encuentra y *por qué* coincidió un resultado de búsqueda. El problema era que cada uno de esos colores se decidía por separado en cada superficie: una nota «marchitándose» podía ser un verde en el árbol de archivos y un verde distinto en la Vista del Cielo (Sky View). La categoría **Colores cognitivos** te da **un control de color por estado**, y todo lo que muestra ese estado lo sigue.

Abre el **Diseñador de estilo** y haz clic en **Colores cognitivos** en la lista de la izquierda. El centro muestra una leyenda de color del conjunto que estés editando; elige un control de la derecha y la leyenda se actualiza en vivo. Hay cinco conjuntos.

### Madurez — cuán asentada está una idea

Cinco estados, del más joven al más asentado: **Semilla**, **Retoño**, **Perenne**, **Canónica**, **Marchita**. Cada uno recibe un color, usado por los puntos de nota del árbol de archivos, el marcador de madurez de la pestaña y el inspector de la nota.

### Confianza — cuán seguro es un enlace

Cuatro estados: **Hipótesis**, **Evidencia**, **Establecido**, **Cuestionado**. Un color cada uno.

### Origen — de dónde vino una idea

Cuatro estados: **Recibido** (tomado de una fuente), **Descubierto** (propio tuyo), **Mixto** y **Ninguno**. Un color cada uno.

### Etapa — dónde se sitúa una nota en su vida

Seis estados, en orden: **Chispa**, **Nacimiento**, **Crecimiento**, **Madurez**, **Latencia**, **Archivado**. Un color cada uno.

### Categoría de coincidencia — por qué coincidió un resultado de búsqueda

Siete tipos de coincidencia: **Título**, **Contenido**, **Etiqueta**, **Wikilink**, **Propiedad**, **Semántico** (una coincidencia basada en el significado, no en una palabra exacta) y **Estructurado** (una coincidencia de consulta sobre propiedades). El color que fijas aquí lo comparten el resaltado de búsqueda dentro del editor, la insignia de coincidencia y el resaltado de la fila del resultado en el panel de búsqueda.

### «Unificar bajo demanda» — la regla que lo hace seguro

Los colores cognitivos siguen una regla deliberada: **nada cambia hasta que eliges un color.** Cada superficie conserva el color que tiene hoy como su propio valor de reserva. En el momento en que fijas aquí el color de un estado, **todas** las superficies que muestran ese estado adoptan tu color a la vez — árbol de archivos, pestañas, el inspector, los resaltados de búsqueda, etc. Fija «Perenne» una vez, y cada marcador Perenne de toda la aplicación coincide. Deja un estado sin tocar y se verá exactamente como antes.

Por eso la categoría puede entregarse sin alterar ni un solo aspecto existente: unifica *bajo demanda*, nunca por defecto. Haz clic en **Conservar (Keep)** para guardar tus colores en este Universo.

---

## Menús de clic derecho en toda la aplicación

Constellation ahora te ofrece un menú de clic derecho (menú contextual) completo en los tres lugares donde más a menudo quieres uno: el **cuerpo de la nota**, una **propiedad del frontmatter** y un **resultado de búsqueda**. Cada menú solo ofrece las acciones que tienen sentido donde hiciste clic.

### Clic derecho en el cuerpo de la nota

Haz clic derecho en cualquier parte del texto de una nota para obtener el menú de edición:

- **Añadir enlace** / **Añadir enlace externo** — envuelve la selección (o inserta en el cursor) como un `[[wikilink]]` o un enlace `[texto](url)`.
- **Formato ▸** — un submenú desplegable: Negrita, Cursiva, Subrayar, Tachado, Resaltar, Código en línea, Matemática, Alternar comentario, Superíndice, Subíndice, Limpiar formato.
- **Párrafo ▸** — un desplegable: Lista con viñetas, Lista numerada, Lista de tareas, los niveles de encabezado **H1–H6** y **Cuerpo**, y Cita.
- **Insertar ▸** — un desplegable: Nota al pie, Tabla, Nota destacada, Línea horizontal, Bloque de código, Bloque matemático, Imagen.
- **Portapapeles** — Cortar, Copiar, Pegar, Pegar como texto, Seleccionar todo.
- **Estilo…** — salta directamente al **Diseñador de estilo** centrado en la categoría **Editor**, para que puedas reestilizar justo aquello sobre lo que hiciste clic derecho.

### Clic derecho en una propiedad del frontmatter

Haz clic derecho en una **fila** de propiedad del panel de Propiedades (o en el bloque de propiedades de la parte superior de la nota) y obtienes acciones de propiedad además del menú de edición completo:

- **Copiar valor** — copia el valor de la propiedad al portapapeles.
- **Copiar nombre** — copia la clave de la propiedad.
- **Eliminar propiedad** — borra esa fila de propiedad.
- **Añadir propiedad** — añade una nueva fila de propiedad vacía.
- …seguido de los mismos elementos de **Formato / Párrafo / Insertar / portapapeles** que el cuerpo de la nota, y un elemento **Estilo…** que abre el Diseñador de estilo centrado en la categoría **Propiedades** — de modo que «Estilo…» sobre una etiqueta de propiedad estiliza las etiquetas de propiedad, no el cuerpo de la nota.

### Clic derecho en un resultado de búsqueda

Haz clic derecho en un resultado del panel de búsqueda para obtener un conjunto **seguro** de acciones de nota — las que nunca ponen tus archivos en riesgo:

- **Abrir** — abre la nota.
- **Abrir en una pestaña nueva** — la abre junto a lo que ya tienes.
- **Revelar en árbol de archivos** — resalta la nota en el árbol de archivos para que veas dónde vive.
- **Copiar enlace** / **Copiar ruta** — copia un wikilink a la nota, o su ruta de archivo.
- **Marcador** — añade la nota a tus marcadores.
- **Mostrar en explorador del sistema** — revela el archivo en el gestor de archivos de tu sistema operativo.
- **Abrir en app predeterminada** — abre el archivo en la app que tu sistema use para Markdown.
- **Estilo…** — abre el Diseñador de estilo centrado en la categoría **Colores cognitivos** (donde viven los colores de coincidencia de búsqueda).

> **Por diseño, el menú de resultados de búsqueda no tiene Renombrar, Mover ni Eliminar.** Un panel de búsqueda muestra resultados de todo tu Universo y no mantiene su propia copia al segundo del árbol de archivos, así que una acción destructiva allí podría actuar sobre una vista obsoleta. Constellation mantiene esas operaciones en el árbol de archivos (y el Navegador de notas), donde la vista siempre está actualizada. El menú de búsqueda sirve para *llegar a* una nota de forma segura, no para reestructurar tu biblioteca.

---

## Bueno saberlo

- **Local y privado.** Todo esto se calcula a partir de tus propias notas y ajustes en tu dispositivo. Nada se envía a ningún sitio.
- **Habla tu idioma.** Cada elemento de menú, cada nombre de categoría y cada etiqueta de estado aparece en el idioma de interfaz que hayas elegido y se refleja correctamente en los idiomas de derecha a izquierda. Los colores de estado cognitivo en sí son universales — un color significa el mismo estado en todos los idiomas.
- **«Estilo…» siempre aterriza en la superficie correcta.** Cada entrada «Estilo…» abre el Diseñador de estilo centrado en la categoría del elemento sobre el que hiciste clic derecho: el cuerpo de la nota → **Editor**, una propiedad → **Propiedades**, un resultado de búsqueda → **Colores cognitivos**. Nunca tienes que buscar los controles adecuados.

---

## Relacionado

- [[Appearance and Themes]] — el comportamiento general del Diseñador de estilo, temas, fuentes y Estilos guardados
- [[Properties]] — ver y editar las propiedades del frontmatter cuyas etiquetas reestilizas aquí
- [[Search]] — el panel de búsqueda cuyos resultados llevan el menú de clic derecho
- [[Cognitive Engine]] — qué significan Madurez, Confianza, Origen y Etapa como medidas de conocimiento
- [[Knowledge Formulation]] — los niveles de confianza de los enlaces vivos que representan los colores de Confianza

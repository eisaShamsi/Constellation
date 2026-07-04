---
aliases:
  - Bases
  - Base de Constelación
  - Tablas de notas
  - Vistas estructuradas
  - Archivos base
description: Aprende a usar la Base de Constelación — una tabla viva de tus notas, una fila por nota y una columna por propiedad, que puedes ordenar, editar y reorganizar sin mover nunca un archivo.
---

# Bases

Una **Base** convierte un conjunto de tus notas en una tabla viva: **una fila por nota, una columna por propiedad**. No se copia ni se mueve nada — la tabla lee tus notas en su sitio y las refleja tal como están ahora mismo.

> [!tip] Potente pero sencilla, por defecto
> Una Base se abre con un aspecto familiar y despejado — solo los nombres de tus notas y los campos que te importan. Las columnas más profundas y cognitivas de Constellation están siempre **a un clic de distancia**, pero nunca saturan la primera pantalla. Tú decides cuánta estructura incorporar.

> [!info] No destructiva
> Una Base nunca cambia tus notas por su cuenta. Es un pequeño archivo `.base` que contiene una consulta — "muestra estas notas, con estas columnas, en este orden." Tus archivos Markdown se quedan exactamente donde están.

---

## Dos formas de usar una Base

**1. Como una pestaña completa.** Abre un archivo `.base` y llenará la pestaña como una tabla interactiva.

**2. Dentro de una nota.** Inserta un bloque de código cercado en cualquier nota y se renderiza en línea:

````markdown
```base
view: table
```
````

Ambas funcionan con el mismo motor, así que se comportan de forma idéntica.

---

## Crear una Base

Usa **Nueva Base** desde la barra lateral (la acción "+" / Nueva Base). Constellation escribe por ti un pequeño archivo **YAML** `.base`:

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

| Campo | Significado |
|-------|-------------|
| `schema` | Versión del formato (actualmente `1`). |
| `lens` | El nombre que se muestra en la parte superior de la tabla. |
| `scope.libraries` | `all`, o una lista de bibliotecas específicas a incluir. |
| `scope.federation` | `auto` — incluye también notas de cualquier Universo vinculado (cUniversos). |
| `columns` | Las columnas a mostrar. Una Base nueva empieza solo con el **Nombre** de la nota. |
| `view` | `table` (la tabla es la vista de la Base). |

Rara vez necesitas editar esto a mano — los propios controles de la tabla (más abajo) escriben cada cambio de vuelta al archivo por ti.

---

## La tabla

- **Columna Nombre** — siempre la primera. Haz clic en el nombre de una nota para abrirla.
- **Cada nota coincidente se convierte en una fila.** **No hay límite de filas.** La tabla está *virtualizada* — solo dibuja las filas que están en pantalla — así que una Base con miles de notas se abre al instante y se desplaza con fluidez.
- **Dirección por celda** — cada valor detecta su propia escritura de izquierda a derecha o de derecha a izquierda, de modo que las tablas con idiomas mezclados se leen correctamente.
- El pie de página muestra cuánto tardó la consulta.

---

## Encontrar una nota en la tabla

### Buscar en esta base

La **casilla de búsqueda** del encabezado filtra la tabla a medida que escribes. Coincide con el **nombre** de una nota *y* con el texto de **cada columna visible**, así que puedes encontrar una fila por cualquier valor que veas. La insignia de recuento junto al título muestra **`coincidentes / total`** mientras filtras (por ejemplo `4/7684`), y vuelve al total simple cuando la borras.

La búsqueda funciona en **cualquier escritura** — escribe en árabe para encontrar títulos en árabe, y así con el resto. Como cada fila ya está en memoria, el filtrado es instantáneo incluso en una Base con miles de notas.

### La barra de letras

Cuando una Base tiene **50 filas o más**, aparece una tira estrecha de letras a lo largo del borde de la tabla. Sus letras se construyen a partir de las **primeras letras de los títulos reales de tus notas** — así muestra **A–Z** para títulos en inglés, **أ ب ت …** para árabe, y las letras correctas para cualquier otra escritura que tengas. (En interfaces de derecha a izquierda, la barra se sitúa automáticamente en el lado correcto.)

**Haz clic en una letra para saltar** a la primera nota que empieza por ella. Si la tabla no está ya ordenada por Nombre, al hacer clic en una letra **primero la ordena por Nombre** y luego salta — de modo que las letras siempre significan lo que esperas.

### Clic derecho en una fila

**Haz clic derecho en cualquier fila** para abrir el menú estándar de la nota: **Abrir**, **Abrir en pestaña nueva**, **Marcar**, **Copiar ruta** / **Copiar nombre**, **Mostrar en el árbol de archivos**, **Abrir en la app predeterminada**, **Mostrar en el explorador del sistema** y **Estilo…**. Renombrar, mover y eliminar no se ofrecen aquí de forma deliberada — haz esas acciones desde el árbol de archivos, donde la lista se actualiza de forma segura.

---

## Columnas — añadir, quitar, reordenar

### Añadir una columna

Haz clic en **+ Añadir columna**. El selector está agrupado en dos:

- **Tus campos** — las propiedades de frontmatter que Constellation encontró en tus notas (por ejemplo `status`, `maturity`, `author`). Estos son *tus* datos.
- **Constelación** — campos integrados que la app siempre conoce: **Nombre**, **Ruta**, **Creado** y **Resumen**.

Empieza a escribir para filtrar la lista. Los campos que ya están en la tabla aparecen marcados para que no los añadas dos veces.

### Quitar una columna

Pasa el cursor sobre un encabezado de columna y haz clic en la **×**.

### Reordenar columnas

**Mantén presionado y arrastra un encabezado de columna hacia los lados.** Toda la columna se levanta (se atenúa y el encabezado muestra un contorno de agarre), y una línea vertical marca dónde caerá. Suelta para moverla. La columna Nombre permanece fija como la primera columna.

Cada acción de añadir, quitar y reordenar se guarda automáticamente de vuelta en el archivo `.base`.

---

## Ordenar

**Haz clic en un encabezado de columna para ordenar por él.** Cada clic recorre **ascendente → descendente → desactivado** (una flecha muestra la dirección actual).

Para ordenar por más de una columna, abre el panel **Orden**:

- Añade varias columnas — la primera es el orden principal, las siguientes desempatan.
- Cambia cualquier nivel entre ascendente y descendente.
- Sube o baja los niveles para cambiar la prioridad, o quítalos.

---

## Editar una nota desde la tabla

Haz doble clic en una celda de una de **tus** columnas de frontmatter para editarla:

- **Campos de texto libre** — escribe el nuevo valor; **Enter** guarda, **Escape** cancela.
- **Campos de tipo lista** (como `maturity`) — aparece un **menú desplegable** con los valores válidos **en su orden natural** (para `maturity`: *seed → sapling → evergreen → canonical*). Elige uno, o escribe el tuyo propio.

El cambio se escribe directamente en el frontmatter YAML de esa nota en el disco, y la tabla se actualiza en su sitio.

> [!note] Columnas de solo lectura
> **Nombre** y **Creado** (y las demás columnas integradas de Constelación) se calculan por ti, así que no son editables. Solo tus propios campos de frontmatter pueden cambiarse aquí.

---

## Abrir una Base antigua

Si vienes de Obsidian, o de una versión anterior de Constellation, tus archivos `.base` existentes usan un formato más antiguo.

**Tu archivo nunca se toca.** Cuando Constellation abre uno, muestra un aviso sereno que explica que el formato es antiguo, y ofrece un botón **Convertir a Base de Constelación**. La conversión ocurre **solo cuando haces clic en él** — actualiza el archivo en su sitio al nuevo formato YAML (trasladando lo que puede: el nombre, las columnas y los filtros de texto simples). Hasta que decidas convertirlo, el archivo original se deja exactamente como estaba.

---

## Federación

Una Base es consciente del Universo. Con `federation: auto`, incluye notas de cualquier Universo vinculado (cUniversos) junto a las tuyas. Las notas que viven en un Universo vinculado son de solo lectura — puedes verlas y ordenarlas en la Base, pero la edición se reserva para las notas que te pertenecen.

---

## Local primero y archivo sobre app

Las Bases no contienen datos propios. Cada valor que ves proviene de un archivo `.md` real en tu disco, leído en vivo. Elimina el archivo `.base` y tus notas no se ven afectadas en absoluto — una Base es solo una lente que apuntas a notas que ya tienes.

---

## Teclado y ratón

| Acción | Qué hace |
|--------|----------|
| **Escribir** en la casilla de búsqueda | Filtrar filas por el nombre y cualquier columna visible (cualquier escritura) |
| **Clic** en una letra de la barra | Saltar a la primera nota que empieza por ella (ordena por nombre primero si hace falta) |
| **Clic derecho** en una fila | Menú de la nota: abrir · marcar · copiar · mostrar · estilo |
| **Clic** en un encabezado de columna | Ordenar por él (ascendente → descendente → desactivado) |
| **Arrastrar** un encabezado de columna | Reordenar esa columna |
| **Clic** en la × de un encabezado | Quitar esa columna |
| **Doble clic** en una celda de frontmatter | Editarla (menú desplegable para campos de lista) |
| **Enter** | Guardar la edición |
| **Escape** | Cancelar la edición |
| **Clic** en el nombre de una nota | Abrir la nota |

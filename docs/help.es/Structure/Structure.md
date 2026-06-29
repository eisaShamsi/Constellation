# Estructura

*(La columna vertebral compositiva — dónde se sitúa esta nota dentro de la obra completa)*

Constellation ya te ofrece ocho **enlaces de pensamiento** — *apoya, contradice, causa, ejemplifica, generaliza, deriva-de, parte-de, reemplaza* — el vocabulario que usas para relacionar una idea con otra. Los **enlaces estructurales** son un tipo deliberadamente distinto. No relacionan idea con idea; establecen la **forma ordenada de una obra** que estás construyendo a partir de tus notas: Libro → Parte → Capítulo → Escena, o cualquier esquema de tipo Mapa de Contenido. El panel **Estructura** es donde lees esa forma.

La única pregunta que responde Estructura es: **"¿Dónde se sitúa esta nota dentro de la obra completa?"** — *no* "¿cómo se relaciona esta idea con aquella?". Esa segunda pregunta pertenece a los paneles Retroenlaces (Backlinks) y Enlaces salientes (Outgoing Links), y Estructura no se interpone en su camino.

---

## Por qué los enlaces estructurales se mantienen separados de tu pensamiento

Una colocación estructural es **autoría, no una afirmación sujeta a juicio**. Poner una escena bajo un capítulo, o un capítulo bajo un libro, es una decisión sobre la *forma de tu manuscrito*: no es evidencia, no es un argumento, no es algo que pueda contradecirse o ganar certeza con el tiempo.

Por eso los enlaces estructurales son deliberadamente invisibles para toda medida de pensamiento, madurez y conexión:

- **No** cuentan como conexiones en los retroenlaces ni en los enlaces salientes de una nota.
- **No** elevan la madurez de una nota.
- **No** aparecen en la Vista del Cielo (Sky View) ni en el grafo.

Un índice no debería hacer que una nota parezca más "conectada" de lo que es. Tus enlaces de pensamiento y el esquema de tu manuscrito son dos cosas distintas, y Constellation las mantiene así.

---

## Los dos tipos — solo escribes uno de los lados

Declaras la estructura desde el extremo que te resulte más cómodo, y Constellation deduce el inverso por ti. Nunca tienes que mantener ambos lados.

| Propiedad | Qué significa |
|---|---|
| **`parent`** | El lugar de *esta nota* bajo un único progenitor. (Un capítulo indica a qué parte pertenece.) |
| **`contains`** | La lista ordenada de hijos de *esta nota*. (Un libro enumera sus partes, en orden de lectura.) |

Declarar el `parent` de un hijo y enumerarlo en una lista `contains` son dos maneras de decir lo mismo. Usa la que se ajuste a cómo piensas — de arriba hacia abajo (un libro que *contiene* sus partes) o de abajo hacia arriba (un capítulo que nombra a su *progenitor*).

---

## Crear un enlace estructural — paso a paso

Creas la estructura en las **Propiedades** de una nota — la pestaña Propiedades en la barra lateral derecha, o el bloque de propiedades en la parte superior de la nota.

1. Haz clic en **+ Añadir propiedad**.
2. Como clave, escribe **`parent`** o **`contains`**.
3. En el valor, escribe el **nombre de la nota de destino** — solo el nombre, por ejemplo `Part I - The Cartographer`. **No escribas los corchetes.** Constellation envuelve el nombre en un `[[link]]` por ti automáticamente. (Si pegas un nombre que ya tiene corchetes, se limpia a un único `[[name]]` — nunca un doble `[[[ ]]]`.)
4. Para **`contains`**, añade cada hijo como su propia ficha — escribe un nombre, pulsa Intro, escribe el siguiente. **El orden en que los añades es el orden de lectura** del esquema.

> **Se renombran de forma segura.** Renombra un capítulo y su lugar en la estructura se actualiza automáticamente — el enlace apunta a la nota en sí, no a un fragmento de texto congelado. Nunca tienes que rastrear y arreglar un esquema después de renombrar.

---

## Leer el panel Estructura

Abre la pestaña **Estructura** en la barra lateral derecha — justo después de la pestaña Retroenlaces (Backlinks).

- **El esquema.** Encabezado **OUTLINE** con un recuento, el panel muestra la **obra completa** como un árbol con sangría y viñetas en color verde azulado — cada descendiente de la obra, en orden — no solo los hijos propios de la nota abierta. Así, incluso cuando estás situado en una sola escena, ves todo el libro a su alrededor.
- **"Estás aquí".** La nota que estás viendo en ese momento aparece **resaltada** dentro del esquema, para que siempre sepas dónde te encuentras.
- **La ruta de navegación.** En la parte superior, una ruta de navegación (breadcrumb) en color verde azulado muestra el camino por la columna vertebral — por ejemplo *The Atlas of Lost Places › Part I › Chapter 1*. Haz clic en cualquier migaja (o en cualquier fila del esquema) para saltar directamente a esa nota.
- **Obra completa ⇄ Esta nota.** Un conmutador en la esquina superior derecha alterna entre la obra entera y solo la rama propia de la nota abierta. Aparece únicamente cuando la nota tiene un progenitor (de lo contrario, las dos vistas serían idénticas).

> **Un bucle nunca lo cuelga.** Si la estructura se enlaza accidentalmente sobre sí misma — el progenitor de la nota A es B, y el progenitor de B es A — el esquema dibuja la cadena y luego se detiene limpiamente, marcando el punto de corte con un pequeño **↻**. Pasa el cursor por encima para ver una explicación de una línea.

---

## Cuando dos notas reclaman el mismo hijo — "Contested"

La estructura está pensada para ser un árbol limpio, así que un hijo debería tener exactamente un progenitor. Si dos notas reclaman el mismo hijo — una a través del propio **`parent`** del hijo, la otra a través de su lista **`contains`** — Constellation **no** elige una en silencio y descarta la otra. En cambio, esa fila se marca como **Contested** con una insignia ámbar **⚠** que nombra al otro reclamante, para que puedas ver el conflicto y decidir.

Dos botones de un solo clic lo resuelven:

- **Keep** (Conservar) — conserva el progenitor declarado del propio hijo. (Esta nota renuncia a su reclamación sobre el hijo.)
- **Move here** (Mover aquí) — acepta esta nota como progenitor. (El `parent` del hijo cambia a esta nota.)

Cualquiera de las dos opciones actualiza los archivos de las notas directamente y refresca el esquema. **Nada se cambia jamás sin tu clic** — Constellation marca el conflicto y espera tu decisión.

---

## Bueno saberlo

- **Local y privado.** El esquema se lee de tus propias notas bajo demanda; nada se envía a ningún sitio.
- **Rápido en obras grandes.** Los esquemas largos (más de unas 50 filas) obtienen su propia barra de desplazamiento y solo renderizan las filas que están en pantalla, de modo que un manuscrito grande se abre y se desplaza con fluidez.
- **Habla tu idioma.** Las etiquetas del panel, la ruta de navegación y los botones de resolución aparecen todos en el idioma de interfaz que hayas elegido y se reflejan correctamente en los idiomas de derecha a izquierda. Las *claves* de propiedad `parent` / `contains` permanecen en inglés canónico en el archivo (para que la estructura se lea igual en todos los idiomas), mientras que sus etiquetas de pastilla en pantalla están localizadas.

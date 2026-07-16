# Escribir en árabe y en escrituras mixtas

El editor de Constellation está construido con el idioma como principio de diseño: el árabe, el hebreo, el persa, el urdu y las notas bilingües no son un añadido — el cursor, la selección y la dirección de cada párrafo siguen las mismas reglas que usa Microsoft Word en Windows, así que tu memoria muscular se traslada tal cual. Este tema cubre todo lo relativo a *escribir* en texto de derecha a izquierda y mixto: cómo se mueve el cursor, cómo seleccionar por palabra, oración, línea, párrafo o pantalla, y cómo forzar la dirección de un párrafo cuando la detección automática no es la que quieres.

(Para saber cómo Constellation *entiende* el árabe — raíces, búsqueda y el motor morfológico — consulta el tema **Motor árabe**.)

---

## Cómo se mueve el cursor

- **Las teclas de flecha avanzan un carácter del texto, en orden de lectura** — nunca una posición en la pantalla. En árabe puro o en inglés puro esto se ve exactamente como la flecha que pulsaste. En una costura entre árabe e inglés (por ejemplo, una oración en árabe que contiene una palabra en inglés), el cursor recorre cada carácter en el orden de escritura y «salta» visiblemente a través de la costura — ese salto es correcto; es lo que evita que el cursor parezca atascado en la frontera.
- **Home** va al **inicio** de lectura de la línea — el borde *derecho* de una línea en árabe. **End** va al **final** de lectura — el borde *izquierdo*. Mantén **Shift** con cualquiera de las dos para seleccionar hasta ese borde.
- **Enter** en una línea en árabe coloca el cursor de la línea nueva a la **derecha** — la posición natural de escritura.
- Una **palabra latina al final de una línea en árabe** conserva un cursor claro y estable en lugar de perder su dirección.

Todas las reglas anteriores funcionan de forma idéntica en el editor estándar, en el modo Enfoque y en la vista de fusión de conflictos.

---

## Seleccionar por unidad

Cada unidad de texto tiene su selector rápido, en cualquier idioma y en cualquier mezcla:

| Unidad | Cómo |
|---|---|
| **Palabra** | Doble clic sobre ella |
| **Oración** | **Ctrl+clic** en cualquier punto de ella — o pulsa **Ctrl+Shift+S** con el cursor dentro |
| **Línea** | **Ctrl+L** |
| **Párrafo** | **Ctrl+Shift+L** — o triple clic |
| **Una pantalla** | **Shift+Page Down** / **Shift+Page Up** |
| **Todo** | **Ctrl+A** |

Detalles que conviene conocer:

- **La selección de oraciones entiende la puntuación árabe.** Da por terminada una oración en **؟ ۔ !** y en el punto final — pero el punto y coma árabe **؛** es una pausa *dentro* de la oración, así que la selección lo sobrepasa correctamente. Los números decimales como 3.14 nunca parten una oración.
- Un **párrafo** es un bloque de texto con una línea vacía encima y otra debajo — exactamente como en Word. Las selecciones de línea y de párrafo se ciñen al texto: en una línea en árabe el resaltado se detiene en las palabras en lugar de extenderse por el lado izquierdo vacío.
- Ctrl+clic *sustituye* al antiguo gesto de «añadir otro cursor» en esa tecla — ahora ese clic hace selección de oración.

## Moverse por párrafos

- **Ctrl+↓** salta al inicio del párrafo **siguiente**; **Ctrl+↑** al inicio del **actual** (púlsalo de nuevo para ir al anterior). Añade **Shift** para ir seleccionando párrafo a párrafo mientras saltas. Es la convención de Word, y «siguiente» significa simplemente más abajo en la página — funciona igual en notas en árabe, en inglés y mixtas.

---

## Forzar la dirección de un párrafo

Constellation detecta la dirección de cada línea automáticamente a partir de sus primeras letras. Casi siempre acierta — pero a veces quieres imponer tu criterio: un párrafo en árabe que empieza con una marca comercial en inglés, o un párrafo mayormente en inglés que quieres leer de derecha a izquierda.

**Pulsa y suelta Ctrl+Shift en el lado DERECHO del teclado** → el párrafo donde está el cursor pasa a ser **100 % de derecha a izquierda**.
**Pulsa y suelta Ctrl+Shift en el lado IZQUIERDO** → **100 % de izquierda a derecha**.

Es la convención de Microsoft Word. Cosas que conviene saber:

- **Se dispara al soltar** — pulsa las dos teclas a la vez, suéltalas y no pulses nada más entre medias. Por eso Ctrl+Shift+S, Ctrl+Shift+L y todos los demás atajos siguen funcionando con normalidad: en cuanto se suma una tercera tecla, el cambio de dirección se retira.
- **Es una anulación dura** — vence a la detección automática y se aplica al párrafo entero (o a todos los párrafos que toque una selección).
- **Se guarda dentro del propio texto** como un carácter de dirección invisible, así que sobrevive a cerrar la nota, reiniciar la aplicación y sincronizar — e incluso viaja con el texto si lo pegas en Word o en Obsidian.
- **Un solo Ctrl+Z lo deshace.** Pulsar dos veces el mismo lado no añade nada más.
- **El Markdown queda a salvo.** Las listas siguen siendo listas, los encabezados siguen siendo encabezados, las citas siguen siendo citas. Los bloques de código, las tablas y las líneas horizontales se dejan intactos a propósito. Una línea que *empieza* con un #tag conserva su dirección automática (una marca forzada ahí rompería la etiqueta) — el resto del párrafo sí cambia.

---

## Fuentes e interfaz

- **Fuentes por escritura**: configura las fuentes para árabe, hebreo y CJK de forma independiente en **Configuración → Idioma**.
- **Barras de herramientas por escritura**: botones de símbolos y puntuación específicos de cada idioma.
- **Resaltado del tashkeel**: activa o desactiva el resaltado de los diacríticos árabes desde la barra de herramientas del editor.
- Al seleccionar árabe o hebreo como idioma de la interfaz, toda la aplicación pasa a mostrarse de derecha a izquierda.

---

## Glosario

- **Orden de lectura** — el orden en que los caracteres se escriben y se leen, independientemente de dónde aparezcan en pantalla.
- **Costura** — la frontera entre un tramo de derecha a izquierda y un tramo de izquierda a derecha en la misma línea.
- **Anulación dura** — una dirección explícita que estableces tú y que vence a la detección automática por primera letra.
- **Marca de dirección** — el carácter invisible (RLM/LRM) que guarda tu anulación dentro del propio texto.

---
aliases:
  - Sky View
  - Vista del cielo
  - GraphMind
  - Vista de estrellas
  - Vista de estrellas de enlaces
  - Vista de red
  - Conexiones de notas
  - Grafo 3D
description: Visualiza y explora las conexiones entre tus notas con el Sky View interactivo de Constellation, impulsado por el motor GraphMind.
---

# Sky View

Sky View muestra tus notas como una red interactiva de nodos y enlaces, impulsada por el motor **GraphMind** (Pixi.js WebGL). Cada nodo es una nota, y cada línea representa un `[[wikilink]]` entre notas. Cuantas más conexiones tenga una nota, más grande aparece su nodo.

## Abrir Sky View

| Método | Acción |
|--------|--------|
| **Mission Control** | Pulsa `Ctrl+P` y escribe «star view» |
| **Teclado** | `Ctrl+G` |

Pulsa `Escape` para cerrar Sky View.

> [!note]
> El icono de Sky View se ha quitado de la cinta del panel izquierdo. Ahora se accede a Sky View mediante el atajo de teclado o Mission Control. El modo Sky View (OrgChart) está disponible como pestaña en la barra lateral de Gestión de notas.

---

## Interactuar con el grafo

### Interacciones básicas

| Entrada | Comportamiento |
|-------|----------|
| **Desplazar** | Haz clic y arrastra sobre un espacio vacío |
| **Zoom** | Rueda del ratón (2D) o `Ctrl+Rueda` (3D) |
| **Arrastrar nodos** | Haz clic y arrastra cualquier nodo para reubicarlo |
| **Pasar el cursor** | Muestra el nombre de la nota en la barra de estado y resalta los nodos y aristas conectados |
| **Clic en un nodo** | Abre esa nota en el editor |
| **Doble clic en un nodo** | Acerca y centra el grafo en ese nodo |
| **Clic derecho en un nodo** | Abre el menú contextual |

### Menú contextual

Haz clic derecho en cualquier nodo para acceder a:

| Acción | Descripción |
|--------|-------------|
| **Abrir** | Abre la nota en el editor |
| **Enfocar** | Entra en el modo de enfoque centrado en este nodo |
| **Fijar** | Bloquea el nodo en su posición actual. Haz clic de nuevo para desfijarlo. |
| **Ocultar** | Oculta el nodo del grafo. Usa «Mostrar todo» en la barra de herramientas para revelar los nodos ocultos. |

---

## Navegación en 3D

Sky View admite navegación 3D completa — vuela a través de tus notas como si navegaras entre estrellas.

### Entrar en el modo 3D

**Haz clic con el botón central y arrastra** (o **Alt+clic y arrastra**) para rotar el grafo en el espacio 3D. Una vez rotado, los controles de navegación 3D se activan.

### Controles 3D

| Entrada | Acción |
|-------|--------|
| **Arrastrar con clic central** | Rotar alrededor de los ejes X e Y |
| **Mayús+arrastrar con clic central** | Rotar alrededor del eje Z |
| **W / Flecha arriba** | Volar hacia delante (hacia el interior de la pantalla) |
| **S / Flecha abajo** | Volar hacia atrás |
| **A / Flecha izquierda** | Desplazarse a la izquierda |
| **D / Flecha derecha** | Desplazarse a la derecha |
| **Q** | Bajar |
| **E** | Subir |
| **Ctrl+Rueda** | Zoom (cambiar el campo de visión) |
| **Rueda normal** | Volar hacia delante/atrás en la dirección de la cámara |
| **0** | Restablecer la rotación a la vista 2D plana |
| **Botón de restablecer** (icono ↺) | Igual que pulsar `0` |

### Guía de ejes XYZ

En el modo 3D, una guía de ejes con código de colores aparece en la esquina inferior izquierda:

| Eje | Color | Dirección |
|------|-------|-----------|
| **X** | Rojo | Izquierda–Derecha |
| **Y** | Verde | Arriba–Abajo |
| **Z** | Azul | Adelante–Atrás (profundidad) |

La guía rota con la cámara, así que siempre conoces tu orientación.

### Pasar el cursor y hacer clic en 3D

Puedes pasar el cursor y hacer clic en los nodos mientras navegas en 3D. El nombre de la nota aparece en la barra de estado, y al hacer clic se abre la nota — igual que en el modo 2D.

---

## Modos de disposición

Sky View ofrece tres algoritmos de disposición. Cambia entre ellos pulsando `Ctrl+L` o usando el botón de disposición en la barra de herramientas.

| Modo | Descripción | Ideal para |
|------|-------------|----------|
| **Orgánico** | Disposición dirigida por fuerzas. Los grupos emergen de forma natural según la densidad de enlaces. | Exploración general — el modo predeterminado. |
| **Jerárquico** | Grafo acíclico dirigido (DAG) de arriba abajo. | Bibliotecas estructuradas con relaciones padre–hijo. |
| **Temporal** | Nodos dispuestos a lo largo de un eje de tiempo horizontal según la fecha de creación. | Ver cuándo se crearon las notas y cómo creció la biblioteca. |

Cambiar de modo activa una transición animada y fluida que conserva tu orientación espacial.

> [!tip]
> El modo Jerárquico es especialmente útil para notas que siguen una estructura en árbol (por ejemplo, mapas de contenido que enlazan a subtemas). El modo Temporal revela tu línea de tiempo intelectual — cuándo se crearon los grupos de notas relacionadas.

---

## Modo de enfoque

El modo de enfoque muestra solo una nota concreta y su vecindario. Es un grafo local dinámico e interactivo.

### Entrar en el modo de enfoque

- **Haz clic derecho en un nodo** → **Enfocar**
- **Pulsa Espacio** para activar o desactivar el modo de enfoque en la nota activa

### Controles de enfoque

En el modo de enfoque, aparece una barra de control en la parte superior:

| Control | Descripción |
|---------|-------------|
| **Control deslizante de profundidad** (1–5) | Cuántos saltos de conexiones mostrar. 1 = solo enlaces directos, 5 = cinco niveles de profundidad. |
| **Filtro de dirección** (↔ / ← / →) | Mostrar todos los enlaces, solo entrantes o solo salientes. |
| **Botón de salir** (×) | Volver al Sky View completo |

### Ruta de navegación

A medida que haces clic en los nodos en el modo de enfoque, aparece una ruta de navegación en la parte superior que muestra tu recorrido. Haz clic en cualquier elemento de la ruta para volver al grafo local de esa nota.

> [!tip]
> Combina el modo de enfoque con el control deslizante de profundidad para explorar progresivamente el vecindario de una nota. Empieza en profundidad 1 para ver las conexiones directas, y luego auméntala para descubrir relaciones de segundo y tercer grado.

---

## Buscar y resaltar

Pulsa `Ctrl+F` para abrir la barra de búsqueda. Escribe una consulta para resaltar las notas coincidentes.

A diferencia de un filtro, la función de buscar y resaltar **atenúa** los nodos que no coinciden sin eliminarlos. Conservas toda la estructura del grafo y el contexto espacial mientras los nodos coincidentes quedan resaltados.

> [!tip]
> La búsqueda funciona tanto en el grafo completo como en el modo de enfoque. También puedes buscar mientras estás en el modo 3D.

---

## Panel de ajustes

Haz clic en el icono de engranaje (⚙) en la barra de herramientas para abrir el panel de ajustes. Tiene tres pestañas:

### Apariencia del grafo

| Control | Descripción | Predeterminado |
|---------|-------------|---------|
| **Tamaño del nodo** | Aumentar o reducir el tamaño de todos los nodos | 1.5 |
| **Visibilidad de etiquetas** | Cuándo aparecen las etiquetas: Al pasar el cursor, Siempre o Nunca | Al pasar el cursor |
| **Tamaño de fuente de etiqueta** | Tamaño de las etiquetas con el nombre de la nota | 12 |
| **Grosor del enlace** | Anchura de las líneas de las aristas | 1 |
| **Mostrar notas huérfanas** | Incluir notas sin enlaces | Activado |

> **Color de fondo del lienzo.** El color detrás de las burbujas se establece en **Ajustes → Style Setter → Sky View → Lienzo → Fondo** (no en este panel). Es independiente de tus barras laterales y paneles, así que puedes darle al grafo su propio telón de fondo — un color profundo para que las burbujas resalten, por ejemplo — sin cambiar el resto de la interfaz. Si se deja sin definir, el lienzo coincide con la superficie del panel. Consulta *Apariencia y temas → Lienzo de Sky View*.

### Física

| Control | Descripción | Predeterminado |
|---------|-------------|---------|
| **Repulsión** | Con cuánta fuerza se separan los nodos | 50 |
| **Fuerza del enlace** | Con cuánta fuerza se atraen los nodos enlazados | 0.05 |
| **Distancia del enlace** | Distancia objetivo entre nodos enlazados | 30 |
| **Recalentar simulación** | Reiniciar la disposición por fuerzas desde el estado actual | — |

### IA

Ajustes para los enlaces semánticos de IA (Fase 2 — requiere un modelo de incrustaciones local).

| Control | Descripción |
|---------|-------------|
| **Mostrar enlaces semánticos** | Activar o desactivar las aristas discontinuas detectadas por IA |
| **Umbral de confianza** | Control deslizante para filtrar los enlaces semánticos según su puntuación de similitud |

---

## Leyenda

La leyenda aparece en la esquina inferior derecha y muestra las asignaciones de color de tus bibliotecas.

### Alternar el modo de color

Haz clic en los botones **Biblioteca** o **Carpeta** en la parte superior de la leyenda para cambiar cómo se colorean los nodos:

| Modo | Coloreado |
|------|----------|
| **Biblioteca** | Cada biblioteca recibe un color único |
| **Carpeta** | Cada carpeta de nivel superior recibe un color único |

### Casillas de visibilidad

Cada entrada de la leyenda tiene una casilla. Desmarca una biblioteca o carpeta para ocultar sus nodos del grafo. Esto te permite centrarte en subconjuntos específicos de tu base de conocimiento.

> [!tip]
> En el modo Carpeta, el número de carpetas se muestra entre paréntesis. Las listas largas de carpetas se pueden desplazar.

---

## Barra de estado

La barra de estado de la parte inferior izquierda muestra:

- **Número de nodos** — total de nodos visibles
- **Número de aristas** — total de aristas visibles
- **Número de MOC** — número de mapas de contenido (notas que actúan como concentradores de alta conectividad)
- **Nombre de la nota bajo el cursor** — aparece al pasar el cursor sobre un nodo

---

## Atajos de teclado

| Atajo | Acción |
|----------|--------|
| `Ctrl+G` | Abrir Sky View |
| `Escape` | Cerrar Sky View |
| `Ctrl+F` | Activar o desactivar buscar y resaltar |
| `Ctrl+L` | Cambiar el modo de disposición (Orgánico → Jerárquico → Temporal) |
| `Space` | Activar o desactivar el modo de enfoque en la nota activa |
| `0` | Restablecer la rotación 3D a la vista 2D plana |
| `W/A/S/D` | Volar por el espacio 3D (cuando está rotado) |
| `Q/E` | Bajar/subir en el espacio 3D |

---

## Compatibilidad con RTL

Sky View ofrece compatibilidad de primer nivel con el árabe, el hebreo y otras escrituras de derecha a izquierda (RTL):

- **Las etiquetas de los nodos** detectan automáticamente la dirección de la escritura — los títulos en árabe se muestran de derecha a izquierda
- **Los elementos de la leyenda** invierten el orden del punto y el texto según el idioma del contenido
- **Las descripciones emergentes y los paneles** respetan la disposición RTL
- **Reserva de fuentes árabes** — las etiquetas usan las fuentes árabes del sistema (Noto Naskh Arabic, Segoe UI) cuando la fuente principal carece de cobertura de glifos árabes

---

## Superposición Imagen en Imagen (PiP)

Cuando Sky View está abierto y haces clic en un universo hijo, una biblioteca o una carpeta en la barra lateral de Gestión de notas, aparece una ventana de **Imagen en Imagen (PiP)** como una superposición redimensionable sobre el grafo principal.

### Qué muestra la PiP

La PiP muestra un subgrafo filtrado que contiene solo los nodos que pertenecen al ámbito seleccionado. Por ejemplo, al hacer clic en una biblioteca se muestran solo las notas de esa biblioteca y sus interconexiones.

### Funciones de la PiP

| Función | Descripción |
|---------|-------------|
| **Grafo filtrado** | Solo aparecen los nodos del ámbito seleccionado |
| **Leyenda filtrada** | La PiP tiene su propia leyenda que muestra solo las entradas relevantes |
| **Redimensionable** | Arrastra los bordes o las esquinas para cambiar el tamaño de la ventana PiP |
| **Reubicable** | Arrastra la barra de título para mover la PiP a cualquier parte de la pantalla |

### Sincronización de selección entre modos

Al hacer clic en un universo hijo, una biblioteca, una carpeta o una nota en cualquier modo de la barra lateral (Árbol, Lista u OrgChart), se resaltan los nodos correspondientes en el grafo de Sky View. Esta sincronización bidireccional te ayuda a mantener la conciencia espacial mientras navegas por la barra lateral.

---

## Estratos de conocimiento

Sky View ajusta automáticamente el tamaño de los nodos según su nivel de conocimiento (1-8):

- Puntos pequeños: notas simples (Dato, Información)
- Nodos medianos: notas conectadas (Proposición, Concepto)
- Grandes concentradores luminosos: notas de síntesis (Teoría, Paradigma, Cosmovisión)

Los nodos de nivel superior tienen un halo de brillo de color complementario para dar contraste visual. Esto se activa cuando una biblioteca tiene más de 20 notas.

---

## Madurez de las notas

Los nodos muestran un anillo de color que indica su madurez:

- Sin anillo: Semilla (nota nueva)
- Anillo verde claro: Retoño (en crecimiento)
- Anillo verde intenso: Perenne (bien establecida)
- Anillo dorado: Canónica (referencia autorizada)

La madurez también se muestra en el árbol de archivos (borde izquierdo) y en la barra de pestañas (punto de color).

---

## Brillo de procedencia

Los nodos en Sky View muestran un sutil brillo de color que indica el origen del conocimiento:

- **Brillo azul**: conocimiento recibido — la cadena de origen de la nota se remonta a una referencia externa (una nota con url, autor o doi en sus metadatos)
- **Brillo ámbar**: conocimiento descubierto — la cadena de origen de la nota nace de las propias notas del usuario

---

## Notas técnicas

Sky View está impulsado por el motor **GraphMind**, un renderizador Pixi.js WebGL con una simulación d3-force que se ejecuta en un Web Worker dedicado. Esta arquitectura garantiza:

- **Renderizado a 60 fps** incluso con miles de nodos
- **Disposición sin bloqueos** — la simulación de fuerzas nunca congela la interfaz
- **Pasar el cursor es solo visual** — pasar el cursor nunca desencadena un recálculo de la física
- **La simulación se detiene tras asentarse** — una vez que los nodos encuentran sus posiciones, el motor de física se detiene por completo. Solo arrastrar un nodo o cambiar los ajustes lo reinicia.

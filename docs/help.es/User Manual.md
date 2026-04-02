# Manual de Usuario de Constellation

**Version 0.3.4 | Marzo 2026**

Constellation es una aplicacion de escritorio para la Gestion del Conocimiento Personal (PKM) que permite administrar bibliotecas de notas en Markdown. Desarrollada con Tauri v2, SvelteKit y Rust, funciona de forma nativa en Windows, macOS y Linux con soporte completo para arabe y escritura RTL.

---

## Tabla de Contenidos

1. [Primeros Pasos](#primeros-pasos)
2. [Universo y Bibliotecas](#universo-y-bibliotecas)
3. [Crear y Editar Notas](#crear-y-editar-notas)
4. [Vista Estelar (GraphMind)](#vista-estelar-graphmind)
5. [Segunda Pantalla](#segunda-pantalla)
6. [Propiedades y Frontmatter](#propiedades-y-frontmatter)
7. [Plantillas](#plantillas)
8. [Tablas](#tablas)
9. [Tareas](#tareas)
10. [Importador](#importador)
11. [Calendario](#calendario)
12. [Lens](#lens)
13. [Configuracion](#configuracion)
14. [Atajos de Teclado](#atajos-de-teclado)
15. [Soporte RTL y Arabe](#soporte-rtl-y-arabe)
16. [Seguridad y Privacidad](#seguridad-y-privacidad)
17. [Motor Cognitivo](#motor-cognitivo)

---

## 1. Primeros Pasos

### Instalacion

Descarga el instalador mas reciente desde la [pagina de versiones de Constellation](https://github.com/eisaShamsi/Constellation/releases):

- **Windows**: Instalador `.exe` (NSIS) o `.msi`
- **macOS**: Imagen de disco `.dmg`
- **Linux**: Paquete `.AppImage` o `.deb`

### Primer Inicio

Cuando abras Constellation por primera vez, el **Asistente de Configuracion del Universo** te guiara a traves de:

1. **Elige tu idioma** — 15 idiomas disponibles
2. **Crea o importa una biblioteca** — selecciona una carpeta existente con archivos Markdown, o comienza desde cero
3. **Nombra tu universo** — el universo es el contenedor de todas tus bibliotecas

### Vista General de la Interfaz

| Elemento | Descripcion |
|----------|-------------|
| **Barra lateral (Ribbon)** | Botones de navegacion: Arbol de archivos, Busqueda, Vista Estelar, Calendario, Plantillas, Configuracion |
| **Arbol de Archivos** | Explora notas y carpetas dentro de tus bibliotecas |
| **Editor** | Lee y edita tus notas en Markdown |
| **Barra de Pestanas** | Abre multiples notas en pestanas |
| **Barra de Estado** | Conteo de palabras, caracteres y tiempo de lectura |

---

## 2. Universo y Bibliotecas

### Que es un Universo?

Un **Universo** es el contenedor de nivel superior que alberga todas tus bibliotecas. Piensa en el como tu espacio de trabajo o coleccion de bibliotecas.

### Que es una Biblioteca?

Una **Biblioteca** es una carpeta en tu computadora que contiene archivos Markdown (`.md`). Puedes tener multiples bibliotecas en un solo universo — por ejemplo, una para notas de trabajo y otra para notas personales.

### Administrar Bibliotecas

- **Agregar una biblioteca**: Configuracion > Bibliotecas > Agregar Biblioteca, o arrastra una carpeta a la aplicacion
- **Eliminar una biblioteca**: Configuracion > Bibliotecas > haz clic en el boton de eliminar junto al nombre de la biblioteca
- **Configuracion de biblioteca**: Cada biblioteca puede tener su propia configuracion de apariencia (fuentes, colores)

### Universos Secundarios

Puedes anidar universos dentro de otros universos. Un **Universo Secundario** es otra carpeta de universo referenciada por tu universo principal. Las notas de los universos secundarios aparecen en la Vista Estelar junto a tus propias notas, con enlaces entre bibliotecas mostrados como lineas discontinuas.

### Reapertura automática

Constellation recuerda tu último universo activo y lo reabre automáticamente al iniciar. Si el universo fue movido o su ruta cambió, Constellation lo detecta y corrige la ruta automáticamente.

### Universos portátiles

Los universos de Constellation son completamente portátiles. Puedes mover la carpeta del universo a cualquier ubicacion — una unidad diferente, una memoria USB u otra computadora — y Constellation detectara y corregira automaticamente todas las rutas internas al reabrirlo.

Para mover un universo:
1. Cierra Constellation
2. Mueve o copia la carpeta del universo a la nueva ubicacion
3. Abre Constellation → aparece la pantalla de bienvenida (la ruta anterior ya no es valida)
4. Elige **Abrir Universo Existente** y apunta a la nueva ubicacion
5. Todas las notas y bibliotecas aparecen inmediatamente — las rutas se corrigen automaticamente

La estructura de carpetas del universo sigue el modelo de Obsidian: las notas van directamente en la carpeta raiz, la configuracion reside en `.constellation/`.

---

## 3. Crear y Editar Notas

### Crear una Nota

| Metodo | Accion |
|--------|--------|
| **Teclado** | `Ctrl+N` |
| **Arbol de Archivos** | Clic derecho en una carpeta > Nueva Nota |
| **Mission Control** | `Ctrl+P` > "Nueva nota" |

### Modos del Editor

Constellation ofrece dos modos de editor, seleccionables en **Configuracion > Editor > Tipo de editor**:

#### Editor Markdown (CodeMirror)

El editor predeterminado para usuarios avanzados. Escribe Markdown directamente con:

- **Vista Previa en Vivo** — renderiza el formato en linea mientras escribes
- **Modo Fuente** — muestra la sintaxis Markdown sin procesar
- **Barra de formato** — aparece al seleccionar texto
- **Comandos con barra** — escribe `/` para inserciones rapidas
- **Autocompletado de Wikilinks** — escribe `[[` para enlazar notas
- **Cursores multiples** — `Alt+Click` o `Ctrl+D`

#### Editor de Documentos (TipTap)

Una experiencia WYSIWYG tipo procesador de texto con barra visual:

- Negrita, Cursiva, Subrayado, Tachado, Resaltado
- Encabezados (H1–H3), Alineacion de texto
- Listas con vinetas, Listas numeradas, Listas de tareas
- Citas, Bloques de codigo, Lineas horizontales
- Tablas (insertar, agregar/eliminar filas y columnas)
- Enlaces e Imagenes

Ambos editores guardan como archivos Markdown estandar. Puedes cambiar entre ellos en cualquier momento sin perder datos.

### Callouts (Avisos)

Crea bloques de aviso estilizados para notas, advertencias, consejos y otras indicaciones:

```markdown
> [!note] Informacion importante
> El contenido del callout va aqui.

> [!warning] Ten cuidado
> Esta accion no se puede deshacer.

> [!tip]- Haz clic para expandir
> Contenido de callout plegable.
```

Tipos soportados: `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Cada tipo tiene un color e icono distintivo. Agrega `-` despues del tipo para hacerlo plegable (inicia colapsado), o `+` (inicia expandido).

### Sintaxis de Resaltado

Envuelve el texto con doble signo igual para resaltarlo:

```markdown
Este es ==texto resaltado== en tu nota.
```

En Vista Previa en Vivo, las marcas `==` se ocultan y el texto aparece con fondo amarillo.

### Bloques de Codigo

Los bloques de codigo delimitados se muestran con un color de fondo y etiqueta de lenguaje:

````markdown
```javascript
const greeting = "Hello, world!";
```
````

El nombre del lenguaje aparece como una insignia sobre el bloque de codigo.

### Insercion de Imagenes

Inserta imagenes directamente en tus notas:

```markdown
![Texto alternativo](https://example.com/image.png)   — URL externa
![[photo.jpg]]                                          — archivo local de la biblioteca
```

En Vista Previa en Vivo, las imagenes se renderizan en linea. Las imagenes locales deben estar en la carpeta de tu biblioteca. Las imagenes externas requieren conexion a internet.

### Barra de Herramientas de Tabla

Cuando tu cursor esta dentro de una tabla markdown, aparece una barra de herramientas flotante con:

- **+ Fila / + Columna** — agregar filas o columnas
- **- Fila / - Columna** — eliminar filas o columnas
- **Alineacion** — alineacion izquierda, centro o derecha por columna
- **Ordenar** — ordenar filas ascendente o descendente por la columna actual
- **Tab / Shift+Tab** — navegar entre celdas de la tabla

### Atajos de Formato de Texto

| Atajo | Accion |
|-------|--------|
| `Ctrl+B` | Negrita |
| `Ctrl+I` | Cursiva |
| `Ctrl+Shift+S` | Tachado |
| `Ctrl+Shift+H` | Resaltado |
| `Ctrl+K` | Insertar wikilink |
| `Ctrl+Z` | Deshacer |
| `Ctrl+Shift+Z` | Rehacer |

### Enlazar Notas

Escribe `[[` para abrir el autocompletado de notas. Comienza a escribir el nombre de una nota y selecciona entre las sugerencias. Los enlaces aparecen como wikilinks clicables: `[[Nombre de la Nota]]`.

Tambien puedes enlazar a encabezados especificos: `[[Nombre de la Nota#Encabezado]]`.

---

## 4. Vista Estelar (GraphMind)

La Vista Estelar visualiza tus notas como un grafo 3D interactivo impulsado por el motor **GraphMind** (Pixi.js WebGL).

### Abrir la Vista Estelar

- Haz clic en el icono de grafo en la barra lateral
- Presiona `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Vista Estelar"

### Navegacion

| Entrada | Accion |
|---------|--------|
| **Clic + arrastrar** | Desplazar el grafo |
| **Scroll** | Acercar/alejar |
| **Clic en un nodo** | Abrir la nota |
| **Clic derecho en un nodo** | Menu contextual (Abrir, Enfocar, Fijar, Ocultar) |
| **Clic medio + arrastrar** | Rotar en 3D |
| **W/A/S/D** | Volar por el espacio 3D |
| **0** | Restablecer rotacion a 2D |
| **Ctrl+F** | Buscar y resaltar |
| **Space** | Alternar modo enfoque |

### Modos de Diseno

Presiona `Ctrl+L` para alternar entre:

- **Organico** — diseno dirigido por fuerzas donde los grupos emergen naturalmente
- **Jerarquico** — diseno en arbol de arriba hacia abajo
- **Temporal** — notas organizadas por fecha de creacion en una linea de tiempo

### Modo Enfoque

Clic derecho en un nodo > **Enfocar** para ver solo su vecindario. Ajusta:

- **Profundidad** (1–5 saltos) — cuantos niveles de conexiones mostrar
- **Direccion** (↔/←/→) — todos los enlaces, solo entrantes, o solo salientes

### Navegacion 3D

Haz clic medio y arrastra para rotar. Usa W/A/S/D/Q/E para volar por el campo estelar. Un indicador de ejes XYZ en la esquina muestra tu orientacion. Presiona `0` para restablecer.

### Configuracion

Haz clic en el icono de engranaje para:

- **Apariencia**: Tamano de nodo, visibilidad de etiquetas, tamano de fuente, grosor de enlaces, mostrar huerfanos
- **Fisica**: Fuerza de repulsion, fuerza de enlace, distancia de enlace
- **IA**: Umbral de enlaces semanticos (Fase 2)

### Leyenda

La leyenda en la esquina inferior derecha muestra los colores de biblioteca/carpeta con casillas para alternar la visibilidad.

### Estratos del Conocimiento

La Vista Estelar clasifica automaticamente tus notas en ocho estratos de conocimiento segun el nivel de abstraccion:

| Estrato | Descripcion |
|---------|-------------|
| **Instantanea** | Notas rapidas y efimeras |
| **Registro** | Eventos con fecha y entradas de diario |
| **Tema** | Conceptos atomicos sobre una sola idea |
| **Mapa** | Notas organizativas que conectan otros temas |
| **Marco** | Modelos y marcos de pensamiento |
| **Principio** | Reglas y axiomas verificados |
| **Conviccion** | Valores y creencias fundamentales |
| **Artefacto** | Obras completadas y definitivas |

El estrato se determina automaticamente a partir del frontmatter, la estructura y los enlaces de la nota. Puedes sobrescribir la clasificacion manualmente agregando una propiedad `stratum` en el frontmatter.

### Ciclo de Madurez

Cada nota atraviesa un ciclo de madurez que refleja su grado de desarrollo:

- **Semilla** — Idea inicial o borrador en bruto
- **Plantula** — La nota toma forma y tiene algunos enlaces
- **Perenne** — Nota madura, revisada y bien enlazada
- **Canonica** — Referencia definitiva y autorizada en su tema

El nivel de madurez se actualiza automaticamente segun el numero de enlaces, la fecha de revision y la frecuencia de edicion. Tambien puedes establecerlo manualmente a traves de la propiedad `maturity` en el frontmatter.

---

## 5. Segunda Pantalla

Abre una ventana separada para ver notas lado a lado.

- **Abrir**: Haz clic en el icono de segunda pantalla en la barra lateral, o `Ctrl+Shift+N`
- **Sincronizacion**: Las notas se abren en la segunda pantalla de forma independiente. La configuracion de fuentes y tema se aplica a ambas ventanas.
- **Ancho de nota**: Ajustable mediante el control deslizante en la barra de herramientas

---

## 6. Propiedades y Frontmatter

Las notas pueden tener frontmatter YAML en la parte superior:

```yaml
---
tags: [proyecto, activo]
date: 2026-03-19
status: en-progreso
---
```

Constellation detecta los tipos de propiedades automaticamente:

| Tipo | Ejemplo |
|------|---------|
| **Texto** | `author: Juan` |
| **Numero** | `priority: 5` |
| **Fecha** | `date: 2026-03-19` |
| **Lista** | `tags: [a, b, c]` |
| **Casilla** | `done: true` |
| **Enlace** | `related: [[Otra Nota]]` |

Alterna la visualizacion de propiedades en **Configuracion > Editor > Propiedades en el documento** (Visible / Oculto / Fuente).

---

## 7. Plantillas

Crea plantillas de notas reutilizables:

1. Crea una carpeta para plantillas en tu biblioteca
2. Establece la ruta de la carpeta de plantillas en **Configuracion > Plantillas**
3. Al crear una nueva nota, elige una plantilla desde el selector de plantillas

Las plantillas admiten variables:

| Variable | Se reemplaza con |
|----------|------------------|
| `{{date}}` | Fecha actual |
| `{{time}}` | Hora actual |
| `{{title}}` | Titulo de la nota |
| `{{clipboard}}` | Contenido del portapapeles |

---

## 8. Tablas

### Tablas Markdown

Escribe una tabla Markdown manualmente o usa el comando de barra `/table`:

```markdown
| Encabezado 1 | Encabezado 2 |
|--------------|--------------|
| Celda 1      | Celda 2      |
```

### Barra de Herramientas de Tabla

Cuando tu cursor esta dentro de una tabla, aparece una barra flotante con:

- Agregar/eliminar filas y columnas
- Alinear columnas (izquierda, centro, derecha)
- Navegar entre celdas con `Tab` / `Shift+Tab`

### Tablas en el Editor de Documentos

El editor de Documentos (TipTap) ofrece una experiencia visual de tablas:

- Haz clic en el boton de tabla para insertar
- Usa el menu desplegable para gestionar filas/columnas
- Redimensiona columnas arrastrando los bordes

---

## 9. Tareas

Constellation admite casillas de tareas en las notas:

```markdown
- [ ] Tarea incompleta
- [x] Tarea completada
```

En el modo de Vista Previa en Vivo, las casillas son clicables. Las tareas se pueden buscar y filtrar en todas tus bibliotecas.

---

## 10. Importador

Importa notas desde otras herramientas PKM:

- **Obsidian** — importa vaults con compatibilidad completa de wikilinks
- **Carpetas Markdown** — importa cualquier carpeta de archivos `.md`
- **Otros formatos** — HTML, archivos de texto

Ve a **Configuracion > Importador** para iniciar una importacion.

---

## 11. Calendario

La vista de Calendario muestra las notas organizadas por fecha:

- Las notas con una propiedad `date` aparecen en sus dias correspondientes
- Se pueden crear notas diarias para cualquier fecha
- Navega entre meses con los botones de flecha

Abre el Calendario desde la barra lateral.

---

## 12. Lens

Lens proporciona vistas filtradas de tus notas:

- Filtra por etiquetas, carpetas, propiedades
- Ordena por nombre, fecha o propiedades personalizadas
- Guarda configuraciones de Lens para acceso rapido

---

## 13. Configuracion

Accede a la Configuracion desde el icono de engranaje en la barra lateral o `Ctrl+,`.

### General

- Idioma (15 idiomas)
- Tema (Claro / Oscuro)
- Fuente de interfaz, Fuente de texto, Fuente monoespaciada, Tamano de fuente
- Tema de fuente — combinaciones de fuentes predefinidas (Maquina de escribir, Clasico, Moderno, etc.) para cambio rapido

### Editor

- Tipo de editor (Markdown / Documento)
- Vista predeterminada (Lectura / Edicion)
- Modo de Vista Previa en Vivo
- Numeros de linea, Guias de indentacion, Corrector ortografico
- Auto-cierre de parentesis, Listas inteligentes

### Bibliotecas

- Agregar/eliminar bibliotecas
- Configuracion de apariencia por biblioteca
- Ubicacion de la carpeta de adjuntos

### Actualizaciones

- Buscar actualizaciones
- Token de GitHub para actualizaciones de repositorios privados

---

## 14. Atajos de Teclado

### Globales

| Atajo | Accion |
|-------|--------|
| `Ctrl+N` | Nueva nota |
| `Ctrl+O` | Star Jump (apertura rapida) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Abrir Vista Estelar |
| `Ctrl+,` | Configuracion |
| `Ctrl+Shift+F` | Buscar en la biblioteca |
| `Ctrl+Shift+N` | Segunda pantalla |

### Editor

| Atajo | Accion |
|-------|--------|
| `Ctrl+B` | Negrita |
| `Ctrl+I` | Cursiva |
| `Ctrl+K` | Insertar wikilink |
| `Ctrl+Z` | Deshacer |
| `Ctrl+Shift+Z` | Rehacer |
| `Ctrl+D` | Seleccionar siguiente ocurrencia |
| `Ctrl+/` | Alternar comentario |
| `Tab` | Indentar / siguiente celda de tabla |

### Vista Estelar

| Atajo | Accion |
|-------|--------|
| `Ctrl+F` | Buscar y resaltar |
| `Ctrl+L` | Cambiar modo de diseno |
| `Space` | Alternar modo enfoque |
| `0` | Restablecer rotacion 3D |
| `W/A/S/D/Q/E` | Volar en 3D |
| `Escape` | Cerrar Vista Estelar |

---

## 15. Soporte RTL y Arabe

Constellation ofrece soporte de primera clase para arabe, hebreo, persa, urdu y otros idiomas con escritura RTL:

- **Deteccion automatica**: La direccion de la nota se detecta automaticamente a partir del contenido
- **Interfaz**: Interfaz RTL completa cuando se selecciona el idioma arabe/hebreo
- **Editor**: Edicion de texto RTL con movimiento de cursor y seleccion correctos
- **Vista Estelar**: Las etiquetas en arabe se renderizan de derecha a izquierda con respaldo de fuente adecuado
- **Leyenda**: Los elementos invierten el orden punto/texto segun el idioma del contenido
- **Fuentes de escritura**: Configura fuentes para arabe, hebreo y CJK de forma independiente en Configuracion

### Configuracion para Arabe

1. Ve a **Configuracion > General > Idioma** y selecciona Arabe
2. Opcionalmente, establece una fuente dedicada para arabe en **Configuracion > General > Fuentes de escritura**
3. Las notas con contenido en arabe se renderizaran automaticamente en RTL

---

## 16. Seguridad y Privacidad

- **Todos los datos permanecen locales** — sin sincronizacion en la nube, sin telemetria, sin rastreo
- **Archivos Markdown** — tus notas son archivos de texto plano que te pertenecen completamente
- **Sin cuenta requerida** — Constellation funciona completamente sin conexion
- **Actualizaciones opcionales** — busca actualizaciones manualmente desde Configuracion
- **Codigo abierto** — inspecciona el codigo en [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 17. Motor Cognitivo

El Motor Cognitivo es el sistema de inteligencia integrado de Constellation que analiza tus notas y descubre patrones ocultos y relaciones entre tus ideas.

### Estratos del Conocimiento

El Motor Cognitivo clasifica automaticamente cada nota en uno de ocho estratos de conocimiento (Instantanea, Registro, Tema, Mapa, Marco, Principio, Conviccion, Artefacto) segun la estructura, contenido y enlaces de la nota. Esta clasificacion te ayuda a comprender la distribucion del conocimiento en tu biblioteca e identificar vacios.

### Ciclo de Madurez

El motor rastrea el nivel de madurez de cada nota en cuatro etapas: **Semilla** → **Plantula** → **Perenne** → **Canonica**. El nivel cambia automaticamente a medida que se agregan enlaces y revisiones.

### Enlaces Tipados

El Motor Cognitivo descubre conexiones semanticas entre notas basandose en la similitud del contenido, incluso si no estan vinculadas explicitamente mediante wikilinks. Estas conexiones aparecen como lineas punteadas en la Vista Estelar; su sensibilidad se puede ajustar en la configuracion de la Vista Estelar.

### Cadena de procedencia

La Cadena de procedencia rastrea el linaje de las fuentes a través de enlaces `[[nota|derives-from]]`. Clasifica el conocimiento como "recibido" (de fuentes externas) o "descubierto" (notas propias del usuario). Muestra un árbol de ascendencia en la pestaña de Procedencia de la barra lateral.

### Configuracion del Motor Cognitivo

- **Clasificacion de estratos** — Activar o desactivar la clasificacion automatica
- **Seguimiento de madurez** — Activar o desactivar el seguimiento del ciclo de madurez
- **Enlaces tipados** — Ajustar el umbral de sensibilidad para la deteccion de enlaces (0.0 – 1.0)
- **Sobrescritura manual** — Agrega propiedades `stratum` y `maturity` en el frontmatter para sobrescribir la clasificacion automatica

---

*Manual de Usuario de Constellation — Version 0.3.4 — Marzo 2026*
*uconstellation.world*

# Manual de Usuario de Constellation

**Version 0.1.0 | Marzo 2026**

Constellation es una aplicacion de escritorio para la Gestion del Conocimiento Personal (PKM) que permite administrar bibliotecas de notas en Markdown. Desarrollada con Tauri v2, SvelteKit y Rust, funciona de forma nativa en Windows, macOS y Linux con soporte completo para arabe y escritura RTL.

---

## Tabla de Contenidos

1. [Primeros Pasos](#primeros-pasos)
2. [Universo y Bibliotecas](#universo-y-bibliotecas)
3. [Crear y Editar Notas](#crear-y-editar-notas)
4. [Busqueda](#busqueda)
5. [Vista Estelar (GraphMind)](#vista-estelar-graphmind)
6. [Vista Dividida](#vista-dividida)
7. [Indice](#indice)
8. [Constellation Sight](#constellation-sight)
9. [Segunda Pantalla](#segunda-pantalla)
10. [Propiedades y Frontmatter](#propiedades-y-frontmatter)
10b. [Revisión de Fuentes (CECE)](#10b-revisión-de-fuentes-constellation-epistemic-content-engine--cece)
11. [Plantillas](#plantillas)
12. [Tablas](#tablas)
13. [Tareas](#tareas)
14. [Importador](#importador)
15. [Calendario](#calendario)
16. [Lens](#lens)
17. [Configuracion](#configuracion)
18. [Atajos de Teclado](#atajos-de-teclado)
19. [Soporte RTL y Arabe](#soporte-rtl-y-arabe)
20. [Seguridad y Privacidad](#seguridad-y-privacidad)
21. [Mapa del conocimiento](#mapa-del-conocimiento)
22. [Motor Cognitivo](#motor-cognitivo)

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

## 4. Busqueda

Constellation incluye un motor de busqueda hibrido multilingue basado en SQLite FTS5 con clasificacion BM25, filtros de consulta estructurados y normalizacion optimizada para arabe. La busqueda es accesible desde la barra lateral.

### Como buscar

Haga clic en el icono de busqueda en la barra lateral o use `Ctrl+Shift+F` para activar el modo de busqueda. Escriba su consulta y los resultados aparecen tras un breve retardo (300ms). Presione `Escape` o haga clic en `x` para limpiar la busqueda y volver al arbol de archivos.

### Sintaxis de busqueda

| Sintaxis | Ejemplo | Que encuentra |
|----------|---------|---------------|
| Texto libre | `gestion de proyectos` | Notas que contienen esas palabras en titulo o cuerpo |
| Filtro de etiqueta | `#investigacion` | Notas etiquetadas con `#investigacion` |
| Filtro de propiedad | `status=activo` | Notas con propiedad frontmatter `status` igual a `activo` |
| Filtro de wikilink | `links to [[Clima]]` | Notas que enlazan a `[[Clima]]` |
| Ambito de biblioteca | `in:MiBiblioteca` | Restringe resultados a una biblioteca especifica |
| Combinado | `#investigacion status=activo economia` | Todos los filtros aplicados juntos |

### Insignias de tipo de coincidencia

Cada resultado muestra una insignia de color que indica como se encontro la coincidencia. La insignia muestra una letra localizada para accesibilidad (segura para daltonicos):

| Insignia | Color | Significado |
|----------|-------|-------------|
| **T** | Azul | Coincidencia de titulo — el termino aparece en el nombre de la nota |
| **C** | Verde | Coincidencia de contenido — el termino aparece en el cuerpo de la nota |
| **S** | Purpura | Coincidencia semantica — relacionado conceptualmente (requiere modelo de embeddings) |
| **P** | Ambar | Coincidencia de propiedad — encontrado via filtro de propiedad frontmatter |
| **#** | Rosa | Coincidencia de etiqueta — encontrado via filtro de etiqueta |
| **W** | Azul claro | Coincidencia de wikilink — encontrado via filtro de wikilink |

Las letras de las insignias estan localizadas para los 15 idiomas soportados.

### Resultados fijados (Navegar entre resultados)

Los resultados permanecen visibles despues de hacer clic en uno. La nota abierta se resalta en la lista de resultados para que vea cual esta visualizando. Haga clic en otro resultado para navegar a el sin repetir la busqueda.

Para limpiar la busqueda, presione `Escape` o haga clic en `x`.

### Navegacion por teclado

| Tecla | Accion |
|-------|--------|
| `Flecha abajo` | Seleccionar siguiente resultado |
| `Flecha arriba` | Seleccionar resultado anterior |
| `Enter` | Abrir el resultado seleccionado |
| `Escape` | Limpiar busqueda y volver al arbol de archivos |

### Resaltado del termino de busqueda

Al abrir una nota desde los resultados, todas las apariciones del termino se resaltan en el editor. Funciona con deteccion de diacriticos arabes — buscar "ادارة" resaltara "إدارة" y todas las variantes diacriticas.

### Historial de busqueda

Haga clic en el campo de busqueda cuando este vacio para ver sus busquedas recientes (ultimas 20 consultas). Cada entrada muestra el texto y hace cuanto se realizo. Haga clic en cualquier entrada para repetir esa busqueda. Use el enlace "Borrar historial" en la parte inferior para eliminar todo el historial.

El historial se almacena localmente en su dispositivo y persiste entre reinicios de la aplicacion.

### Search Hub

El Search Hub es una experiencia de busqueda a pantalla completa. Haga clic en el icono de lupa en la barra del dock para abrirlo. Ambas barras laterales se colapsan para dar el maximo espacio. Escriba cualquier termino y Constellation busca en todas partes simultaneamente, agrupando resultados en 5 categorias: Titulos, Contenidos, Etiquetas, Propiedades y Wikilinks. Cada categoria tiene una seccion desplegable con un contador. Haga clic en cualquier resultado para abrirlo en el editor con todas las ocurrencias resaltadas. Aparece un boton "Volver al Search Hub" para que pueda regresar sin volver a buscar.

### Operadores de enlace

Constellation admite 6 operadores de busqueda de topologia de enlaces:

| Sintaxis | Que encuentra |
|----------|---------------|
| `links to [[X]]` | Notas que enlazan a X (backlinks) |
| `links from [[X]]` | Notas a las que X enlaza (enlaces salientes) |
| `mutual [[X]]` | Notas enlazadas a X Y X enlaza de vuelta (bidireccional) |
| `mentions [[X]]` | Notas que contienen el nombre de X sin un [[wikilink]] |
| `orphans` | Notas sin enlaces entrantes ni salientes |
| `links between [[X]] and [[Y]]` | Notas que enlazan tanto a X como a Y |

Al escribir cualquier operador de enlace, el autocompletado `[[` muestra todas las notas del universo. Despues de seleccionar una nota, escriba `#` para completar encabezados o `|type:` para completar el tipo de enlace.

---

## 5. Vista Estelar (GraphMind)

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

## 6. Vista Dividida

La Vista Dividida te permite editar multiples notas lado a lado en la ventana principal.

### Abrir la Vista Dividida

- **Paleta de Comandos**: `Ctrl+P` y luego escribe "Split View"
- **Atajo de teclado**: Usa el atajo asignado para alternar entre modos
- **Ciclo**: Desactivado → Vertical (lado a lado) → Horizontal (arriba y abajo) → Desactivado

### Editar en Vista Dividida

Cada panel es un editor completamente independiente con:
- Barra de herramientas completa (negrita, cursiva, encabezados, alineacion, etc.)
- Navegacion de ruta (biblioteca / nombre de nota)
- Panel de propiedades y menu desplegable de etapa
- Soporte de guardado (`Ctrl+S` guarda el panel enfocado)
- Edicion de titulo y renombrado de archivo

### Redimensionar Paneles

Arrastra el divisor entre paneles para redimensionarlos. Cada divisor es independiente — con 3 o mas notas abiertas, puedes redimensionar cualquier par adyacente sin afectar a los demas. Funciona tanto en modo vertical como horizontal.

### Enfoque

Haz clic en cualquier panel para enfocarlo. El panel enfocado recibe los atajos de teclado y es rastreado por los paneles de la barra lateral derecha (Propiedades, Retroenlaces, etc.).

---

## 7. Indice

El Indice es un glosario completo de terminos de todas tus bibliotecas — cada palabra significativa, ordenada alfabeticamente con conteos de apariciones.

### Abrir el Indice

- **Boton del dock**: Haz clic en el icono del Indice (libro) en el dock izquierdo
- **Paleta de Comandos**: `Ctrl+P` y luego escribe "Index"

### Pipeline NLP Multilingue

El Indice procesa el texto a traves de un pipeline consciente del idioma antes de indexar:

- **Arabe**: Algoritmo Lucene Light10 — elimina tashkeel, unifica hamza, elimina el articulo definido (الـ), elimina sufijos gramaticales
- **Hebreo**: Eliminacion de prefijos (ב/ל/מ/ה/ו/כ/ש)
- **Ingles**: Stemming tipo Porter (plurales, formas verbales, sufijos)
- **Frances/Espanol/Portugues/Aleman**: Eliminacion de sufijos especificos del idioma
- **Ruso/Turco/Hindi/Persa**: Eliminacion de sufijos morfologicos
- **Los 15 idiomas**: Filtrado de palabras vacias (articulos, preposiciones, conjunciones)

### Navegacion

- **Pestanas de idioma**: Alterna entre Todos, Arabe, Hebreo, Ingles o # (caracteres especiales)
- **Barra alfabetica**: Haz clic en una letra para filtrar terminos que comienzan con esa letra — el conteo de terminos se actualiza para mostrar cuantos coinciden
- **Haz clic en la misma letra de nuevo** para borrar el filtro y mostrar todos los terminos
- **Modos de ordenacion**: Alfabetico (predeterminado) o por frecuencia (mas comunes primero)

### Editar desde el Indice

Haz clic en cualquier nota en las referencias de un termino para abrirla en un panel de vista previa dividido junto al Indice. El panel de vista previa es un editor completo — puedes editar, guardar, cambiar propiedades y promover la etapa. El termino de busqueda se resalta en la nota y se desplaza automaticamente.

Presiona `Ctrl+Clic` para abrir la nota como una pestana regular. Aparece un boton "Volver al Indice" en la barra de pestanas — haz clic para volver exactamente donde lo dejaste en el Indice.

### Integracion con la Segunda Pantalla

Cuando la Segunda Pantalla esta abierta:
- **Haz clic en un termino** → La Segunda Pantalla muestra todas las notas que contienen ese termino en una vista dividida (lista de notas + editor)
- **Ctrl+Clic en multiples terminos** → La Segunda Pantalla muestra el modo de comparacion con cada termino en su propia columna

---

## 8. Constellation Sight

Constellation Sight visualiza todo su sistema de conocimiento como un grafico de pozo gravitacional. Responde a la pregunta: **"Como se ve mi conocimiento y que tan saludable esta?"**

### Abrir Sight

Haga clic en el **boton Sight** (icono de ojo) en la cinta izquierda. Aparece el grafico de pozo gravitacional. Haga clic en x para cerrar.

### El Grafico de Pozo Gravitacional

Las notas se organizan en anillos concentricos por importancia (centralidad). Las notas mas conectadas se ubican en el centro; las notas perifericas en los bordes. Dentro de cada anillo, las notas se agrupan por biblioteca (su organizacion). Color del nodo = biblioteca.

| Elemento | Significado |
|----------|-------------|
| **Nodo grande** | Alta centralidad — conecta diferentes areas de conocimiento |
| **Nodo pequeno** | Periferico — dentro de un area |
| **Color del nodo** | Pertenencia a biblioteca |
| **Linea solida** | Enlace entre dos notas |
| **Flechas de direccion** | Pequenas flechas que muestran la direccion del enlace |
| **Grosor de linea** | Nivel de confianza (grueso = establecido, delgado = hipotesis) |

### Interaccion

- **Clic simple** en un nodo: resalta su vecindario (todas las notas conectadas). Todo lo demas se atenua.
- **Doble clic**: abre la nota en el editor.
- **Clic en espacio vacio**: borra el resaltado.
- **Desplazamiento**: zoom. **Arrastrar**: panoramica. **Ajustar a pantalla**: boton de la barra de herramientas.

### Busqueda en Sight

Haga clic en la lupa. Soporta todos los operadores: `links to [[X]]`, `links from [[X]]`, `mutual [[X]]`, `orphans`, `supports [[X]]`, `contradicts [[X]]`, `#tag`, texto libre y busqueda semantica. Los resultados muestran colores direccionales: verde (entrante), rojo (saliente).

### Panel de Analiticas (SightPanel)

Haga clic en el icono de cuadricula para abrir la barra lateral. Muestra: puntuacion de Salud del Universo (0-100), contadores de notas/enlaces/huerfanos, barras de tipo de enlace y confianza, top 10 puentes e Informacion del Conocimiento (evidencia mas fuerte, fundamentos debiles, tensiones, estancados, mas conectados, brechas de conocimiento).

### Configuracion

Icono de engranaje: ajuste el grosor del trazo de enlace, opacidad y tamano de flecha. La configuracion persiste entre sesiones.

### 8a. Campos de tradicion por nota (MIG-029)

El chip de tradicion en la esquina superior izquierda de Sight le permite reencuadrar la cupula a traves de 24 tradiciones academicas en 10 familias epistemicas. Para nueve de esas tradiciones (las de forma sectorial / concentrica / escalonada), cada nota puede **clasificarse explicitamente** mediante un campo en el frontmatter. Las notas sin el campo caen en un cubo predeterminado razonable por tradicion; las notas CON el campo caen en el cubo que usted ha nombrado.

Anada el campo al frontmatter YAML de una nota:

```yaml
---
masadir_source: sunnah
---
```

Cambie al chip de esa tradicion → su nota caera en su sector nombrado en lugar del predeterminado.

**Campos permitidos y valores:**

| Tradicion | Campo frontmatter | Valores permitidos | Predeterminado si ausente |
|---|---|---|---|
| **masādir** (uṣūl al-fiqh sunita) | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` |
| **pramāṇa** (Nyāya indio) | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` |
| **Burhān de Ibn Rushd** | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `shir` (anillo mas externo) |
| **PaRDeS** (hermeneutica judia) | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` |
| **Peirce** (3 categorias faneroscopicas) | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` |
| **Habermas** (3 intereses de conocimiento) | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` |
| **Brotes mencianos** (4 brotes morales) | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` |
| **Sān biǎo mohísta** (3 estandares) | `mohist_zone` | `ben` / `yuan` / `yong` | distribuido por hash en 3 zonas |
| **Sŏngnihak coreano** (debate Cuatro-Siete) | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` |

**Comportamiento:**
- Si escribe un valor que la tradicion no reconoce (error tipografico o inventado), la nota cae en el cubo predeterminado. Sin fallo, sin error de renderizado.
- Los cambios de frontmatter se propagan automaticamente — guarde la nota → el siguiente renderizado de la cupula reflejara el cambio.
- El mismo campo solo es leido por su tradicion nombrada. Establecer `masadir_source: sunnah` en una nota no tiene efecto cuando cambia a PaRDeS o Peirce — cada tradicion lee su propio campo de manera independiente.
- Esta es la forma mas explicita de controlar la gramatica espacial de la cupula. Sin estos campos, la geometria es correcta pero cada nota cae en el mismo cubo predeterminado; con ellos, el chip se vuelve analiticamente significativo.

**Tradiciones sin campos por nota** (actualmente agrupan todas las estrellas por otros medios — carpeta / biblioteca / hash):

- Aristotélica (la predeterminada, sin reasignacion)
- Polanyi (niebla gradiente; sin sectorizacion)
- Husserl, Longino, Maqāṣid de al-Shāṭibī, Profecía maimonideana, 13 middot talmúdicas, Wang Yangming, Pluriversal de Mignolo, Transmodernidad de Dussel, Maldonado-Torres, Akan de Wiredu, ʿUmrān de Ibn Jaldún, Ibuanyidanda

(Las futuras migraciones pueden anadir campos frontmatter por nota para estas a medida que aparezca la demanda de los usuarios.)

---

## 9. Segunda Pantalla

La Segunda Pantalla es una ventana complementaria basada en modos que se adapta al modo actual de tu barra lateral.

- **Abrir**: Haz clic en el icono de segunda pantalla en la barra lateral, o `Ctrl+Shift+2`
- **Cierre automatico**: Cuando cierras la ventana principal, la segunda pantalla se cierra automaticamente

### Complemento basado en modos

La segunda pantalla cambia su contenido segun el modo activo de la barra lateral en la ventana principal:

| Modo de barra lateral | La segunda pantalla muestra |
|---|---|
| **Explorador de archivos** | Panel del universo — estadisticas, desglose de bibliotecas, universos hijos, etiquetas, notas editadas/abiertas recientemente |
| **Navegador** | Vista completa del Navegador para explorar notas |
| **Vista del cielo** | Arbol de Vista del cielo con estructura de directorios |
| **Vista estelar** | Complemento de Vista estelar con retroenlaces, enlaces hacia adelante, etiquetas y grafo local |

### Panel del Universo (Modo Explorador de Archivos)

Cuando la ventana principal esta en modo Explorador de archivos, la segunda pantalla muestra un panel con:

- **Tarjetas de estadisticas** — Nombre del universo, cantidad de universos hijos, total de bibliotecas, carpetas y notas
- **Universos hijos** — Cada universo hijo con sus bibliotecas vinculadas y conteos de carpetas/notas
- **Bibliotecas** — Cada biblioteca con conteos de carpetas/notas en cajas de estadisticas con codigo de colores
- **Editadas recientemente** — Notas que modificaste en la sesion actual (rastreadas al guardar)
- **Abiertas recientemente** — Notas que abriste pero no editaste en la sesion actual
- **Etiquetas** — Todas las etiquetas de todas las bibliotecas ordenadas por cantidad; haz clic en una etiqueta para ver todas las notas que la usan

### Interaccion del panel

Cuando el panel esta activo en la ventana principal, al hacer clic en los elementos se envian a la segunda pantalla:

- **Editadas/Abiertas recientemente**: Haz clic en una nota para abrirla como editor completo en la segunda pantalla
- **Etiquetas**: Haz clic en una etiqueta para mostrar todas las notas que la usan en una vista dividida — lista de notas a la izquierda, editor completo a la derecha

Todas las ediciones en la segunda pantalla se sincronizan automaticamente con la ventana principal.

### Edicion de notas en la Segunda Pantalla

La segunda pantalla soporta edicion completa de notas — escribe, guarda, renombra y cambia propiedades igual que en la ventana principal. Los cambios se sincronizan automaticamente con la ventana principal.

### Sincronizacion de configuracion

Todas las configuraciones visuales se propagan instantaneamente a la segunda pantalla — sin necesidad de reiniciar:

- **Idioma**: Los cambios de idioma de la interfaz se aplican inmediatamente
- **Tema**: El modo claro/oscuro/sistema cambia instantaneamente
- **Fuentes**: Fuente de interfaz, fuente de texto, fuente monoespaciada y fuentes especificas por escritura
- **Tamano de fuente**: Tamanos de fuente de interfaz y editor
- **Editor**: Ancho de linea legible, numeros de linea, barra de herramientas flotante
- **Color de acento**: Cambios de color de acento del tema

---

## 10. Propiedades y Frontmatter

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

## 10b. Revisión de Fuentes (Constellation Epistemic Content Engine — CECE)

> *(Nota de traducción: traducción generada por IA del capítulo V3-§10.F; pendiente de revisión por hablante nativo.)*

Dos de las propiedades de frontmatter más importantes — `sources:` y `content_type:` — describen *cómo llegaste a saber* algo y *qué tipo de conocimiento* es. El **Epistemic Content Engine** (CECE) de Constellation clasifica cada nota a lo largo de estos dos ejes automáticamente usando un conjunto de 6 catalogadores. El panel **Revisión de Fuentes** es donde revisas y corriges esas clasificaciones.

### Lo que hace el motor

Cuando clasificas una nota (clic derecho → «Sugerir fuentes y tipo de contenido», o vía Configuración > Ejecutar escaneo, o automáticamente vía el conmutador de escaneo en segundo plano), CECE ejecuta seis catalogadores independientes contra la nota. Cada uno lee la nota a través de una lente diferente y vota sobre dos preguntas:

- **Fuente** (eje horizontal) — ¿de dónde *vino* este conocimiento? Once valores posibles: percepción, inferencia, testimonio, transmisión-masiva, comparación, postulación, no-aprehensión, memoria, disposición-innata, inspiración, revelación. Más *no clasificable*.
- **Tipo de contenido** (eje vertical) — ¿qué *clase* de conocimiento es este? Cinco ramas principales: entradas sensoriales, entidades simbólicas, contenidos semánticos, estados epistémicos, constructos de orden superior.

Los dos ejes son independientes. Una nota «Dudo del alunizaje» es testimonio (alguien lo reportó) en la fuente + estados-epistémicos/duda (tu postura) en el tipo de contenido.

El motor se ejecuta **en tu dispositivo** — ninguna nota sale jamás de Constellation.

### Los seis catalogadores

Cada catalogador es una lente. La tarjeta de Revisión de Fuentes los muestra como seis pequeños puntos de colores en la esquina superior derecha de cada tarjeta:

- **Tu frontmatter** (azul) — adopta lo que ya has establecido, con autoridad absoluta
- **Citas y estructura** (rosa) — citas, citas en bloque, marcadores de teorema, frases de definición
- **Raíces léxicas y vocabulario** (ámbar) — análisis de raíces árabes + equivalencia de términos entre idiomas
- **Notas enlazadas** (verde azulado) — Living Links tipados a otras notas clasificadas
- **Notas similares** (violeta) — similitud por embeddings con tus notas ya clasificadas
- **Juicio de la IA** (verde) — un LLM local (Qwen3-4B; *aún no activo*, aplazado a una versión futura)

Un punto relleno significa que ese catalogador habló y coincide con la síntesis. Un punto con anillo significa que habló pero discrepó. Un punto con contorno discontinuo significa que permaneció en silencio (sin señal en esa lente).

### Tres regímenes de confianza

Después de que los catalogadores votan, cada eje aterriza en uno de tres regímenes:

- **Unánime** — todos los catalogadores con voz coincidieron
- **Mayoría sólida (una discrepancia)** — la mayoría coincidió; un disidente nombrado
- **Dividido** — sin mayoría clara; el motor se niega a adivinar y te pide que elijas

Cada eje obtiene su propio régimen independientemente — una tarjeta puede ser Unánime en horizontal + Dividida en vertical, etc.

### Sibling Disambiguation

Cuando un eje está Dividido, el motor presenta los valores candidatos como **chips** bajo un mensaje: *«Elige cuál encaja mejor con la nota.»* Haz clic en un chip → el motor escribe esa elección en el frontmatter de la nota y elimina la tarjeta de la cola. Si el OTRO eje estaba resuelto (Unánime o Mayoría sólida), el motor *también* escribe el valor de ese eje al mismo tiempo — un clic finaliza ambos ejes cuando solo uno estaba Dividido.

### El rastro de razonamiento

Cada tarjeta tiene un conmutador *«▸ ¿Por qué esta clasificación?»*. Al expandirlo se muestra una fila por cada catalogador con voz, con el razonamiento, la confianza autorreportada y chips de regla amigables («Coincidencia de palabra clave superficial», «Coincidencia de raíz árabe (CAE)», «Marcador de definición», etc.) — estas son las reglas específicas que cada catalogador activó.

Durante tus **primeras 50 revisiones** el rastro se expande automáticamente en cada tarjeta (un *período de calibración de confianza*) para que puedas desarrollar intuición sobre cuándo confiar en el motor. Después, los rastros se contraen a bajo demanda en las tarjetas Unánimes. Anula en cualquier momento en **Configuración > Inteligencia > CECE > Visibilidad del rastro de razonamiento**.

### El filtro de composición de la cola

Sobre la barra de conteo, cinco chips dividen la cola por el tipo de decisión que cada tarjeta necesita:

- **Todo** — la cola completa
- **Ambos ejes requieren tu decisión** — ambos ejes Divididos
- **La fuente requiere tu decisión** — horizontal Dividido + vertical resuelto
- **El tipo de contenido requiere tu decisión** — vertical Dividido + horizontal resuelto
- **Los catalogadores coincidieron** — ningún eje Dividido (candidatos a sello automático)

Cada chip muestra su recuento de cubo. El filtro es un divisor de capa de renderizado — la matemática de Aceptar Todo siempre opera sobre la cola completa independientemente de qué filtro esté activo.

### Acciones por tarjeta

- **Aceptar** — escribe la síntesis del motor como primaria en ambos ejes; elimina la tarjeta. Actualiza la fiabilidad por catalogador.
- **Editar** — abre un selector de árbol para ambos ejes; elige manualmente. Misma actualización de fiabilidad.
- **Rechazar** — limpia la tarjeta sin escribir.
- **Chip de Sibling Disambiguation** — solo en tarjetas Divididas.

### Calibración por Biblioteca

**Configuración > Inteligencia > CECE > Calibración por Biblioteca** abre una tabla de solo lectura que muestra la exactitud de cada catalogador por eje en la Biblioteca activa. Diferentes Bibliotecas tienen diferentes exactitudes por catalogador — Lingüístico sobresale en Bibliotecas con mucho árabe, Grafo sobresale en las densamente enlazadas. La capa de síntesis usa estos datos de calibración para ponderar votos.

Un catalogador necesita **20 correcciones** antes de que se muestre su ratio de exactitud. Por debajo de ese umbral, la etiqueta dice *«(uniforme)»* — el catalogador contribuye con votos de peso uniforme hasta que se acumulan suficientes datos.

### Clasificación en segundo plano

Por defecto, CECE clasifica notas solo cuando se lo pides (clic derecho o el botón de escaneo en Configuración). Puedes optar por la clasificación automática en **Configuración > Inteligencia > CECE > Clasificación en segundo plano**:

- **Al guardar la nota** — clasifica cada nota ~1,5 segundos después de que dejes de teclear (cabalga sobre el guardado debounced existente; nunca se dispara por pulsación de tecla; la escritura sigue siendo instantánea)
- **Al iniciar la aplicación** — escanea las notas no clasificadas una vez por arranque

### El Clasificador — el hogar de página completa

Las mismas tarjetas también viven en una vista de página completa llamada **el Clasificador**, abierta desde el **icono de tarjetas apiladas en el dock izquierdo**. Es el mismo motor y la misma cola, dados la ventana entera en lugar de una estrecha pestaña de barra lateral — y añade dos controles que la pestaña de barra lateral nunca tuvo:
- **Clasificar una nota…** — una caja de búsqueda que te permite clasificar *cualquier* nota por nombre, sin abrirla primero. Escribe unas pocas letras, elige la nota, y una tarjeta nueva aparece en la cola.
- **Generar todos los resúmenes** — precalcula el resumen de nota (ver abajo) para cada nota que carezca de uno, en segundo plano, con el progreso en la barra de estado.

Un botón de **Iniciar escaneo** (el mismo escaneo a escala de universo que Configuración) y una franja de progreso en vivo completan el encabezado. Cierra el Clasificador con la **(×)** o **Esc**. (Cuando la caja de búsqueda *Clasificar una nota…* está abierta, el primer **Esc** cierra solo esa caja.)

Una nota sobre los nombres: **el Clasificador** es la *sala* (la vista de página completa); **los catalogadores** son las *seis lentes* dentro del motor que votan sobre cada tarjeta. No confundas los dos.

### Resúmenes de notas

Bajo el título de cada tarjeta se sitúa un breve **Resumen** — unas pocas frases que te dicen de qué trata la nota, para que puedas clasificarla sin abrirla. Constellation siempre prefiere un resumen que *tú* escribiste y solo genera uno cuando no lo has hecho:
1. Un **campo de frontmatter** `summary:` / `description:` / `abstract:` / `excerpt:`, usado textualmente.
2. Un **callout** `> [!summary]` / `[!abstract]` / `[!tldr]` en el cuerpo, usado textualmente.
3. De lo contrario, un resumen **generado** — las tres frases más centrales de la nota, extraídas (nunca inventadas) y mostradas en orden original.

Los resúmenes generados son de **solo lectura** — Constellation nunca escribe uno de vuelta en tu nota (File-Over-App), y todo se calcula **en tu dispositivo**. Si quieres que un resumen viva en el archivo, escribe uno tú mismo y Constellation mostrará el tuyo en su lugar.

Para más detalle (cada estado de punto, cada chip de regla, paseos clic a clic), consulta los temas **Revisión de Fuentes**, **Clasificador** y **Note Summaries** en el sistema de ayuda.

---

## 10c. Metadatos Epistémicos

Un pequeño conjunto de campos opcionales de frontmatter para registrar información más rica sobre cómo se adquirió el conocimiento de una nota, quién sostiene la posición, a qué disciplina pertenece y cuándo revisaste por última vez tu visión. Añadido en MIG-022 §A en respuesta al análisis de brechas (`docs/epistemic-content-gap-analysis.md`).

Estos campos son **todos opcionales**. Las notas sin ellos funcionan sin cambios.

### Referencia rápida

| Field | Type | Purpose |
|---|---|---|
| `held_by` | text | ¿De quién es esta postura? (por defecto `user`; puede ser `"al-Shāfiʿī"`, `"Ḥanafī"`, etc.) |
| `domain` | list | Etiquetas disciplinarias para recuperación (`[fiqh, ʿibādāt]`) |
| `function` | text | Para qué sirve esta nota (`reference` / `seed` / `actionable` / `shipped`) |
| `provenance_civilization` | text | Vocabulario tradicional (`sunni-usuli` / `analytic-western` / `nyaya` / etc.) |
| `updated_at` | date | Cuándo revisaste deliberadamente tu visión por última vez (distinto del mtime del sistema de archivos) |
| `ikhtilāf` | list of objects | Desacuerdo erudito estructurado (`[{school, position}, ...]`) |
| `warrant` | text | Etiqueta de grado (parseada pero inerte hasta que el Warrant Research workstream se entregue) |
| `warrant_notes` | text | Texto libre que sustenta el grado de garantía (también inerte) |

### Cómo aparecen en el panel de Propiedades

Cada campo se renderiza con el editor adecuado al tipo:
- Campos de texto → entrada de texto
- `domain` → lista de etiquetas (Enter para añadir, × para eliminar)
- `updated_at` → selector de fecha
- **`ikhtilāf` → widget personalizado** con dos entradas lado a lado por fila (school + position) más un botón eliminar por fila, y un botón "Añadir escuela" en la parte inferior. El widget lee desde y escribe al YAML estructurado, así que los viajes de ida y vuelta preservan cada campo.

### ¿Y `supersedes`?

`supersedes` es una *relación entre notas* (esta nota reemplaza una anterior), no una propiedad de una nota única. Constellation lo maneja como un **enlace tipado**, no como un escalar YAML:

```markdown
Esto reemplaza mi análisis anterior: [[old-note-id|supersedes]]
```

El sufijo `|supersedes` en el wikilink lo convierte en un enlace tipado del tipo `supersedes` — píldora azul-grisácea pizarra distintiva, aparece en los paneles Backlinks + Outgoing Links, participa en la Living Link Architecture.

### Lo que esto NO es

Los nuevos campos son **esquema** — un vocabulario reconocido que puedes rellenar. CECE actualmente no los consume para la clasificación. Futuros MIGs (Warrant Research workstream, MIG-023 eje temporal) entregarán características que lean `warrant`, `updated_at` y compañía.

Para más detalle + un ejemplo trabajado, consulta el tema **Metadatos Epistémicos** en el sistema de ayuda.

---

## 11. Plantillas

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

## 12. Tablas

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

## 13. Tareas

Constellation admite casillas de tareas en las notas:

```markdown
- [ ] Tarea incompleta
- [x] Tarea completada
```

En el modo de Vista Previa en Vivo, las casillas son clicables. Las tareas se pueden buscar y filtrar en todas tus bibliotecas.

---

## 14. Importador

Importa notas desde otras herramientas PKM:

- **Obsidian** — importa vaults con compatibilidad completa de wikilinks
- **Carpetas Markdown** — importa cualquier carpeta de archivos `.md`
- **Otros formatos** — HTML, archivos de texto

Ve a **Configuracion > Importador** para iniciar una importacion.

---

## 15. Calendario

El **Calendario** es una vista mensual a página completa, que se abre desde el **dock izquierdo** (el icono del calendario). Los días que tienen notas o tareas pendientes se marcan con **puntos** de colores. La cabecera muestra el mes en el calendario que hayas elegido; si has definido un **calendario secundario**, un subtítulo debajo muestra el rango equivalente de ese calendario (por ejemplo, un mes gregoriano muestra su intervalo hijrí, "Dhul-Hijjah 1447 – Muharram 1448 AH").

**Hacer clic en un día.** Cada celda de día es interactiva:

- **Haz clic en el espacio vacío (o en el número del día)** → abre (o crea) la **nota diaria** de ese día. Hacer clic en una fecha que ya tiene una nota diaria simplemente la **abre**: nunca crea un duplicado.
- **Haz clic en un punto** → abre ese elemento concreto. Un punto **dorado** es la nota diaria; un punto **morado** es otra nota editada ese día; un punto **rojo** es una tarea que vence ese día. (Los colores son personalizables en el Style Setter → Calendario.) Si un día tiene varias notas o tareas, al hacer clic en el punto se muestra una pequeña **lista** para elegir.
- **Haz clic en un punto de tarea** → abre la nota **desplazada hasta la línea de esa tarea**, lista para editar. En la lista de tareas también puedes **marcar la casilla de una tarea para completarla** directamente desde el calendario: las tareas completadas desaparecen al instante. Solo aparecen en el calendario las tareas que llevan su propia fecha `📅 YYYY-MM-DD` (esa fecha es lo que las sitúa en un día).

**Calendarios culturales (ocho).** En **Configuración → Calendario** puedes definir el **sistema de calendario** — **Gregoriano, Hijrí (Islámico), Hijrí Solar (Persa), Hebreo, Indio (Saka), Budista, Chino o Coreano** — y toda la cuadrícula del mes cambia a ese sistema, mostrando en cada celda tanto la fecha del calendario elegido (grande) como la fecha gregoriana (pequeña), además de la fase lunar. La cabecera de cada mes muestra el **nombre del mes, su número entre paréntesis y el año** — el número ayuda con los calendarios cuyo orden de meses no resulta familiar. Los calendarios **chino y coreano** son *lunisolares*: a veces intercalan un **mes bisiesto** (闰六月 / 윤6월), que el calendario presenta como una página propia para que la navegación nunca lo omita ni lo duplique. El calendario hijrí utiliza un motor astronómico preciso; los meses sagrados se resaltan y los eventos islámicos se señalan. También puedes elegir el **inicio de semana** (domingo/lunes) y activar o desactivar la **columna de número de semana**.

**Opciones del calendario hijrí.** En **Configuración → Calendario → "Calendario hijrí (islámico)"** hay dos controles adicionales:

- **Método de cálculo** — **Astronómico (Conjunción Lunar)**, que sigue la luna nueva real (el más preciso, y el predeterminado), o **Tabular (al-Tawfīqāt al-Ilhāmiyyah)** (la tabla aritmética clásica), el ciclo aritmético clásico.
- **Corrección de mes** — ajusta el comienzo de un mes hijrí en ±1 o ±2 días para que coincida con un **avistamiento local de la luna**. Elige el año y el mes hijrí, escoge un desfase y haz clic en **Establecer**; la corrección se aplica a ese mes y a todos los meses posteriores. Tus correcciones aparecen listadas (cada una se puede eliminar), con un botón **Borrar todo**.

Ambos ajustes (y tus correcciones) se guardan **con tu universo**, de modo que viajan entre tus dispositivos.

**Opciones de visualización chino y coreano.** Corea usa el calendario lunar chino, así que ambos comparten fechas idénticas — lo que los distingue es la escritura y el año. Cuando cualquiera de los dos es tu calendario principal o secundario, **Configuración → Calendario** muestra dos controles adicionales: una **visualización del año** (chino: el ciclo sexagenario 丙午年, el año a secas, o ambos; coreano: la era **Dangi** 단기 4359, el año, o el sexagenario 병오년) y los **nombres de los meses** — *escritura nativa* (五月 / 5월) o *fonético*, la pronunciación del mes escrita en tu propio idioma (Inglés "Wǔyuè / Owol"; Árabe "وُو-يوي / أوه-وُل").

**Dar estilo al calendario.** Abre el **Style Setter** (dock izquierdo, o **Configuración → Style Setter**) y elige la superficie **Calendario** para reestilizar cada parte — cada elemento tiene su propio **color y tamaño de texto** (números de los días, la fecha de referencia cruzada, la pastilla del mes, las cabeceras de los días de la semana, los números de semana, el glifo lunar, el resaltado de Hoy, las líneas de la cuadrícula y los puntos de nota/tarea/evento), además de la **fuente** del calendario. Una vista previa en vivo a tamaño completo se actualiza mientras editas; haz clic en **Conservar** para aplicar.

> **Los nombres de archivo de las notas diarias siempre permanecen en gregoriano** (`YYYY-MM-DD`) independientemente del calendario mostrado — así tus archivos siguen siendo portables y se ordenan correctamente. La fecha cultural se muestra en el calendario (y puede registrarse en el frontmatter de la nota).

El Calendario sirve plenamente a las notas diarias: haz clic en cualquier día para abrirlo, o ejecuta el comando **"Nota Diaria"** (paleta de comandos) para saltar al día de hoy.

**Registrar una fecha cultural en una nota.** Dos herramientas opcionales escriben la fecha cultural en las **propiedades** de una nota (el nombre del archivo siempre permanece en gregoriano `YYYY-MM-DD`):

- **Sello hijrí en las notas diarias** — *Configuración → Calendario → "Sellar la fecha hijrí en las notas diarias."* Cuando está activado (disponible solo mientras el calendario hijrí sea tu **principal o secundario**), cada **nueva** nota diaria recibe una línea `hijri:`, por ejemplo `hijri: 1448-01-06`. Las notas que ya tienes nunca se modifican.
- **"+ Hijrí" en las Propiedades de una nota** — abre las **Propiedades** de cualquier nota, pasa el ratón sobre la fecha y aparecerá un pequeño botón **"+ Hijrí"** (además de "+ Jalali", "+ Hebreo", etc. — **un botón por cada calendario no gregoriano que hayas seleccionado**). Haz clic en él y Constellation lee la fecha gregoriana de la nota y añade la equivalente, por ejemplo `jalali: 1405-03-30`. El botón coreano escribe el año **Dangi**; un **mes bisiesto** chino/coreano se marca con una `L` (por ejemplo `chinese: 2025-06L-17`). Si la nota no tiene una propiedad de fecha, se usa la fecha de creación del archivo.

---

## 16. Lens

Una **Lens** es una consulta guardada que muestra una lista filtrada y ordenada de notas junto con las propiedades que te interesen. Constellation ofrece dos modos:

### Constellation Base — bloques de Lens incrustados

Puedes incrustar una Lens directamente en el cuerpo de cualquier nota Markdown usando un bloque de codigo ` ```base `:

````markdown
```base
schema: 1
view: list
dimensions: [note.name, note.created_at]
sort: [note.created_at, desc]
limit: 20
```
````

Al ver la nota, el bloque de codigo se reemplaza por una tabla interactiva con las notas que coinciden. En la vista previa en vivo, haz clic en la pastilla **Lens** para editar el bloque.

**Dimensiones disponibles en v1:** `note.name`, `note.path`, `note.created_at`, `note.headline`.

**Federacion:** por defecto, los bloques Lens leen del universo activo Y de cada cUniverso enlazado. Establece `federation: active` en el YAML para limitar al universo activo.

### Five Acts — Lenses integradas

La seccion **Five Acts** de la barra lateral (encima de Workspace Bases) lista notas anfitrionas curadas por Constellation en `{universe}/Five Acts/*.md`. v1 incluye una: **Observation — Recent Captures** (lista federada de las 20 notas mas recientes). Puedes editar estas notas libremente — Constellation no sobrescribira tus cambios.

### Panel Lens clasico

El panel Lens anterior (filtrar por etiquetas, carpetas, propiedades) sigue disponible en **Configuracion → Paneles → Lens**.

---

### Estructura (enlaces estructurales)

El panel **Estructura** muestra dónde se sitúa la nota abierta dentro de una *obra* más amplia: un libro, un guion, un curso, un Mapa de Contenido. Responde a una pregunta distinta de la de los paneles Retroenlaces (Backlinks) y Enlaces salientes (Outgoing Links). Aquellos responden a *"¿cómo se relaciona esta idea con otra idea?"* (los enlaces de pensamiento — apoya, contradice, causa…). Estructura responde a *"¿dónde se sitúa esta nota dentro de la obra completa que estoy componiendo?"* — Libro → Parte → Capítulo → Escena.

Esta es la **columna vertebral compositiva** de una obra: el índice, el esquema ordenado. Se mantiene deliberadamente **fuera de** toda medida de pensamiento, madurez y conexión: colocar una nota "bajo un Libro" nunca cambia la madurez de esa nota, sus recuentos de conexiones ni su presencia en la Vista del Cielo (Sky View). Un índice es autoría, no una afirmación sujeta a juicio.

**Los dos tipos de enlace estructural** (solo escribes uno de los lados — Constellation deduce el inverso por ti):

- **`parent`** — el lugar de *esta nota* bajo un único progenitor (p. ej., un capítulo declara la parte a la que pertenece).
- **`contains`** — la lista ordenada de hijos de *esta nota* (p. ej., un libro enumera sus partes en orden de lectura).

**Crear un enlace estructural** — abre las **Propiedades** de la nota (la pestaña Propiedades en la barra lateral derecha, o el bloque de propiedades en la parte superior de la nota):

1. Haz clic en **+ Añadir propiedad** y escribe la clave `parent` o `contains`.
2. En el valor, escribe el **nombre de la nota de destino** — solo el nombre, p. ej. `Part I - The Cartographer`. Constellation lo envuelve en un `[[link]]` por ti; **no** escribas los corchetes. (Si pegas un nombre que ya tiene corchetes, igualmente se guarda de forma limpia como un único `[[name]]`, nunca con doble envoltura.)
3. Para `contains`, añade cada hijo como su propia ficha, en el orden en que quieras que se lean — ese orden se convierte en el orden del esquema.

Los enlaces estructurales **se renombran de forma segura**: renombra un capítulo y su lugar en la estructura se actualiza automáticamente, porque el enlace apunta a la nota, no a un fragmento de texto congelado.

**Leer el panel Estructura** — abre la pestaña **Estructura** en la barra lateral derecha (justo después de Retroenlaces):

- El panel muestra la **obra completa** como un esquema con sangría (viñetas en color verde azulado), encabezado **OUTLINE** con un recuento de los descendientes — no solo los hijos propios de la nota abierta.
- La nota que estás viendo en ese momento aparece **resaltada** ("estás aquí") dentro de ese esquema.
- Una **ruta de navegación** (breadcrumb) en la parte superior muestra el camino por la columna vertebral (p. ej. *The Atlas of Lost Places › Part I › Chapter 1*). Haz clic en cualquier migaja — o en cualquier fila del esquema — para saltar a esa nota.
- Un conmutador **Obra completa ⇄ Esta nota** (en la esquina superior derecha del panel) alterna entre la obra entera y solo el subárbol propio de la nota abierta. Aparece únicamente cuando la nota tiene realmente un progenitor, de modo que las dos vistas difieren.
- Si la estructura se enlaza accidentalmente sobre sí misma (el progenitor de la nota A es B, y el progenitor de B es A), el esquema dibuja la cadena y luego se detiene limpiamente, marcando el punto de corte con un pequeño **↻**. Nunca se cuelga.

**Resolver un conflicto (Contested).** Si dos notas reclaman el mismo hijo — una a través del `parent` propio del hijo, la otra a través de una lista `contains` — el panel marca esa fila como **Contested** (una insignia ámbar ⚠ que nombra al otro reclamante) en lugar de descartarla en silencio. Dos botones de un solo clic lo resuelven:

- **Keep** (Conservar) — conserva el progenitor declarado del propio hijo (esta nota renuncia a su reclamación sobre el hijo).
- **Move here** (Mover aquí) — acepta esta nota como progenitor (el `parent` del hijo cambia a esta nota).

Cualquiera de los botones actualiza los archivos de las notas directamente y refresca el esquema. Nada se cambia jamás sin tu clic.

---

## 17. Configuracion

Accede a la Configuracion desde el icono de engranaje en la barra lateral o `Ctrl+,`.

### General

- Idioma (15 idiomas)
- Tema (Claro / Oscuro)
- Fuente de interfaz, Fuente de texto, Fuente monoespaciada, Tamano de fuente
- Tema de fuente — combinaciones de fuentes predefinidas (Maquina de escribir, Clasico, Moderno, etc.) para cambio rapido
- **Temas** — elige entre seis temas integrados, crea temas personalizados (editor de cinco colores), importa temas del registro de la comunidad de Obsidian (200+ temas), o importa un archivo de tema `.json`. Elimina cualquier tema personalizado con el boton ✕ al pasar el raton.

### Style Settings

Una pestana dedicada para la personalizacion detallada de cada elemento visible de la interfaz, aplicada en vivo al tema activo.

- **Colores** — fondo, superficies, texto (normal/atenuado/debil), acento, bordes, colores de estado
- **Tipografia** — tamanos de fuente de interfaz/nota/codigo, tamanos H1–H6, peso de encabezados, alturas de linea, espacio entre parrafos
- **Diseno y forma** — radios de esquina pequeno/mediano/grande, anchos de borde, sombras, longitud de linea legible del editor, margenes laterales
- **Componentes** — barra de listones, barra de acciones lateral, barra de diseno (alternadores de panel), barra superior/tiras de pestanas, barra de estado, barra lateral derecha (inspector), explorador de archivos (notas del Universo, universos hijo, bibliotecas, carpetas, notas), botones, etiquetas, callouts — cada uno con tamano, radio, color independientes, y estilo de estado activo cuando corresponda
- **Editor** — color/hover/decoracion de enlace, color/fondo/radio de codigo en linea, ancho/color de la barra de cita, color del cursor, fondo de seleccion

**Importar / Exportar** — barra de herramientas arriba de la pestana:
- Pegar desde el portapapeles (un clic)
- Importar / Pegar (area de texto con Fusionar o Reemplazar)
- Desde archivo (.json)
- Copiar (valores actuales al portapapeles)
- Exportar (.json)

El formato coincide exactamente con el plugin Style Settings de Obsidian, por lo que puedes compartir ajustes entre Obsidian y Constellation.

Los cambios se guardan automaticamente en el tema activo; si editas un tema integrado, se auto-clona en tus temas personalizados para que los cambios persistan sin modificar el original.

### El Configurador de Estilo

El **Configurador de Estilo** (Style Setter) es un estudio de diseno a pantalla completa — abrelo desde **Ajustes → Apariencia → "✦ Open Style Setter."** Muestra tu interfaz real en el centro; haz clic en cualquier parte (barra lateral, titulo de la nota, encabezado, enlace, la pagina de la nota) y los controles de ese elemento aparecen a la derecha, mientras la vista previa se actualiza al instante. Las tarjetas de tema (Midnight / Daylight / Chocolate / Nord) siembran todo un aspecto — el propio estudio lo lleva puesto mientras disenas — y la lista de *Superficies* previsualiza el aspecto en toda la aplicacion, no solo en el editor. **"Apply to app"** aplica tu acento, fondos, color de texto y fuentes al Constellation real; **Esc** o **✕** cierra solo el Configurador, no los Ajustes. Por ahora, aplicar es una vista previa en vivo de la sesion — guardar un aspecto como un Estilo permanente y con nombre (con muestras de color reutilizables y renombrables, ademas de exportacion / importacion) llegara a continuacion.

### Anulaciones del motor arabe

Un panel por Universo donde fijas como el motor arabe analiza ciertas formas superficiales — tus propias acunaciones, nombres locales, prestamos especificos de tu campo, o casos en los que no estas de acuerdo con la lectura automatica del motor. Cada anulacion vence al FST generativo, a la cascada y al respaldo heuristico. Anadir o eliminar una anulacion dispara una reindexacion focalizada solo sobre las notas que contienen la superficie afectada — sin reconstruccion completa. Consulta la seccion 19 ("Soporte RTL y Arabe") para el paso a paso.

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

## 18. Atajos de Teclado

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

## 19. Soporte RTL y Arabe

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

### Anulaciones del motor arabe

El motor arabe de Constellation es un analizador morfologico de cinco capas que corre bajo cada busqueda, cada enlace y cada entrada del indice. Entiende raices, patrones, nombres propios, prestamos y reparaciones fonologicas — de modo que una consulta por كاتب encuentra كتبنا y كتاب, pero وائل se mantiene intacto como nombre en lugar de ser mutilado a ائل.

El panel **Anulaciones del arabe** en Configuracion es donde le ensenas al motor tu propia terminologia. Cada anulacion es la respuesta soberana — vence al FST generativo, a la cascada y al respaldo heuristico.

**Cuando usar anulaciones:**
- Nombres de personas, toponimos locales o terminos especificos de tu campo que el motor no conoce
- Acunaciones o siglas propias de tu Universo
- Prestamos donde quieras preservar una grafia concreta
- Cualquier caso en el que el analisis automatico del motor no coincida con tu forma de leer la palabra

**Paso a paso:**

1. Abre **Configuracion** (icono del engranaje o `Ctrl + ,` / `Cmd + ,`) y selecciona **Anulaciones del arabe** en la barra lateral.
2. Haz clic en **Anadir anulacion**.
3. Rellena:
   - **Forma superficial** — la palabra arabe tal como la tecleas
   - **Lema** — la forma canonica que debe devolver el motor
   - **Raiz** (opcional) — 3 o 4 consonantes si la palabra tiene raiz clasica
   - **Patron** (opcional) — p. ej. `فاعل`
   - **Categoria** — Nombre propio / Sustantivo / Adjetivo / Adverbio / Verbo / Particula / Extranjero / Desconocido
   - **Nota** (opcional) — una linea de contexto para ti mismo
4. Haz clic en **Guardar**. El panel muestra **Reindexando…** mientras cada nota que contiene la superficie se vuelve a tokenizar y, al terminar, **Se reindexaron N nota(s)**.
5. Para eliminar una anulacion, haz clic en la **x** de su fila — el mismo barrido de reindexacion corre al reves.

Las anulaciones se guardan por Universo en `<universo>/.constellation/arabic-overrides.json` — texto plano, ordenado alfabeticamente, escritura atomica. Puedes ponerlo bajo control de versiones o compartirlo entre dispositivos.

---

## 20. Seguridad y Privacidad

- **Todos los datos permanecen locales** — sin sincronizacion en la nube, sin telemetria, sin rastreo
- **Archivos Markdown** — tus notas son archivos de texto plano que te pertenecen completamente
- **Sin cuenta requerida** — Constellation funciona completamente sin conexion
- **Actualizaciones opcionales** — busca actualizaciones manualmente desde Configuracion
- **Codigo abierto** — inspecciona el codigo en [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 21. Mapa del conocimiento

El Mapa del conocimiento es una visualizacion radial (sunburst) que muestra la estructura, densidad y madurez de todo tu universo de conocimiento.

### Abrir el Mapa

- **Boton del dock**: Haz clic en el icono del Mapa del conocimiento en la barra lateral izquierda
- **Paleta de comandos**: `Ctrl+P` y escribe "Constellation Map"

### Lo que ves

- **Centro**: El nombre de tu Universo con el total de notas y palabras
- **Primer anillo**: Bibliotecas (cada una con su color). Si tu universo tiene universos hijos, aparecen aqui tambien.
- **Anillos mas profundos**: Carpetas y subcarpetas dentro de cada biblioteca
- **Segmentos exteriores**: Notas individuales

### Modos de color

Cambia entre tres modos con el desplegable:
- **Madurez**: semilla (gris) → plantula (verde claro) → perenne (verde) → canonico (dorado) → marchito
- **Estrato**: L1 (azul) → L8 (rojo) — muestra la complejidad del conocimiento
- **Biblioteca**: todos los segmentos heredan el color de su biblioteca

### Navegacion por profundizacion

Haz clic en cualquier segmento de carpeta para acercarte. Una ruta de migas muestra tu camino. Haz clic en cualquier miga para volver, o presiona Escape. Haz clic en un segmento de nota para abrirla en el editor.

### Volver al Mapa

Despues de abrir una nota desde el Mapa, aparece un boton "Volver al Mapa" en la barra de pestanas. Haz clic para volver exactamente donde estabas — mismo nivel de profundizacion preservado.

---

## 22. Motor Cognitivo

El Motor Cognitivo es el sistema de inteligencia integrado de Constellation que analiza tus notas y descubre patrones ocultos y relaciones entre tus ideas. Su filosofia fundamental:

> «La cantidad de datos no importa. No se trata de cuantas fuentes almacenas, sino de como formulas tu conocimiento a partir de ellas y lo enlazas en una conciencia unica y significativa.»

El Motor Cognitivo esta compuesto por nueve herramientas integradas: Enlaces tipados, Estratos del conocimiento, Ciclo de madurez, Detector de tensiones, Cadena de procedencia, Motor de externalizacion, Pulso de revision, Senderos y Vistas multi-lente.

---

### 17.1 Enlaces tipados

#### Que es?

Los enlaces tipados son wikilinks que llevan un tipo de relacion que describe como se conectan dos notas. En lugar de escribir simplemente `[[nota]]`, escribes `[[nota|tipo-de-relacion]]` para expresar la naturaleza del vinculo: se deriva de ella? La contradice? La extiende?

#### Por que importa?

Un enlace comun dice «hay una conexion», pero no dice cual. Los enlaces tipados transforman tu red de notas de un amontonamiento de referencias en un verdadero mapa de conocimiento que muestra las estructuras de pensamiento, dependencias y razonamientos entre ideas.

#### Como usarlo

1. Abre una nota en el editor
2. Escribe un wikilink con tipo de relacion: `[[Nota destino|derives-from]]`
3. Tipos soportados: `derives-from` (se deriva de), `supports` (apoya), `contradicts` (contradice), `extends` (extiende), `exemplifies` (ejemplifica), `questions` (cuestiona)
4. Tambien puedes agregar tipos desde las propiedades de la nota en la barra lateral derecha

#### Donde se ve?

- **Vista Estelar (GraphMind)**: Como lineas coloreadas y etiquetadas entre los nodos
- **Barra lateral derecha**: En la pestana «Backlinks» indicando el tipo de cada enlace
- **Pestana de Procedencia**: Se usa para construir el arbol genealogico del conocimiento

---

### 17.2 Estratos del conocimiento

#### Que es?

El Motor Cognitivo clasifica automaticamente cada nota en uno de ocho estratos: Instantanea, Registro, Tema, Mapa, Marco, Principio, Conviccion, Artefacto. La clasificacion se basa en la estructura, contenido y cantidad de enlaces de la nota.

#### Por que importa?

Conocer el tipo de cada nota revela el equilibrio del conocimiento en tu biblioteca. Son la mayoria de tus notas meras instantaneas fugaces o han evolucionado hacia principios y marcos? Esta conciencia sobre la naturaleza del contenido es el primer paso para construir conocimiento real en lugar de solo acumular informacion.

#### Como usarlo

1. La clasificacion ocurre automaticamente — no necesitas hacer nada
2. Para sobrescribir la clasificacion automatica, agrega la propiedad `stratum` en el frontmatter:
   ```yaml
   ---
   stratum: framework
   ---
   ```
3. Valores disponibles: `snapshot`, `log`, `topic`, `map`, `framework`, `principle`, `conviction`, `artifact`

#### Donde se ve?

- **Barra lateral derecha**: En la seccion de propiedades bajo «Estrato»
- **Vista Estelar**: Como colores diferentes para los nodos segun el estrato
- **Ajustes > Motor Cognitivo**: Para activar o desactivar la clasificacion automatica

---

### 17.3 Ciclo de madurez

#### Que es?

El motor rastrea el nivel de madurez de cada nota en cuatro etapas: **Semilla** → **Plantula** → **Perenne** → **Canonica**. Cada nota comienza como semilla y crece gradualmente con mas contenido, enlaces y revisiones.

#### Por que importa?

La madurez distingue entre una idea en bruto y conocimiento pulido. La semilla de hoy puede convertirse en referencia manana si le dedicas la atencion necesaria. El seguimiento de madurez te ayuda a identificar las notas que merecen mas desarrollo y atencion.

#### Como usarlo

1. La madurez cambia automaticamente segun: cantidad de palabras, numero de enlaces entrantes y salientes, y fecha de ultima modificacion
2. Para asignar la madurez manualmente, agrega la propiedad `maturity` en el frontmatter:
   ```yaml
   ---
   maturity: evergreen
   ---
   ```
3. Valores disponibles: `seed` (Semilla), `sapling` (Plantula), `evergreen` (Perenne), `canonical` (Canonica)

#### Donde se ve?

- **Barra lateral derecha**: Un icono junto al titulo muestra la etapa de madurez actual
- **Vista Estelar**: Como tamano del nodo — cuanto mas madura la nota, mas grande el nodo
- **Ajustes > Motor Cognitivo**: Para activar o desactivar el seguimiento de madurez

---

### 17.4 Detector de tensiones

#### Que es?

El Detector de tensiones examina notas vinculadas y te alerta cuando hay afirmaciones o conclusiones contradictorias entre dos o mas notas. Se apoya en el analisis de enlaces tipados `contradicts` y la similitud tematica entre notas.

#### Por que importa?

Las tensiones no son necesariamente errores — son invitaciones a pensar mas profundamente. Cuando dos ideas en tu biblioteca se contradicen, significa que tu comprension evoluciono o que hay una complejidad que vale la pena explorar. Detectar tensiones te protege de construir conocimiento sobre bases contradictorias sin darte cuenta.

#### Como usarlo

1. Agrega un enlace tipado `contradicts` entre notas en conflicto: `[[Otra nota|contradicts]]`
2. El motor tambien detecta tensiones implicitas mediante analisis de contenido
3. Revisa la lista de tensiones detectadas desde la barra lateral

#### Donde se ve?

- **Barra lateral derecha**: En la pestana «Tensiones» cuando se detectan contradicciones
- **Vista Estelar**: Como lineas rojas discontinuas entre nodos en conflicto
- **Panel de notificaciones**: Alertas cuando se detecta una nueva tension

---

### 17.5 Cadena de procedencia

#### Que es?

La Cadena de procedencia rastrea el origen de cada idea — de donde proviene y de que se derivo. Utiliza enlaces `[[nota|derives-from]]` para construir un arbol genealogico que muestra el camino de evolucion del conocimiento desde la fuente original hasta la formulacion actual.

#### Por que importa?

Saber de donde vienen tus ideas distingue el conocimiento recibido (de libros, articulos, conferencias) del conocimiento descubierto (tus propias conclusiones y reflexiones). Esta conciencia sobre el origen del conocimiento te ayuda a evaluar la fiabilidad de tus ideas y comprender como se ha formado tu pensamiento con el tiempo.

#### Como usarlo

1. Al crear una nota derivada de una fuente, agrega un enlace: `[[Fuente original|derives-from]]`
2. Se pueden construir cadenas de multiples niveles: nota ← derivada de ← derivada de ← fuente original
3. Clasifica fuentes externas agregando `source-type: received` en el frontmatter

#### Donde se ve?

- **Barra lateral derecha**: La pestana «Procedencia» muestra el arbol genealogico completo
- **Vista Estelar**: Como direccion de las flechas en los enlaces (de fuente a derivado)
- **Propiedades de la nota**: Clasificacion como «recibido» o «descubierto» segun la cadena de procedencia

### 17.6 Motor de externalizacion

#### Que es?

Un pipeline de formalizacion progresiva que rastrea como tus notas maduran desde capturas sin procesar hasta ideas cristalizadas. Cada nota puede asignarse a una de cuatro etapas:

| Etapa | Icono | Significado |
|-------|-------|-------------|
| Fugaz | 🌱 | Captura rapida, pensamiento pasajero |
| Literatura | 📖 | Reescrita desde una fuente en tus propias palabras |
| Permanente | 🔗 | Idea atomica, un concepto, conectada a tu grafo |
| Sintesis | ✨ | Idea original que combina multiples notas permanentes |

#### Por que importa?

La mayoria de las apps tratan todas las notas por igual. El Motor de externalizacion hace visible la distincion — puedes ver de un vistazo cuanto de tu biblioteca es captura sin procesar frente a comprension genuina.

#### Como usarlo

1. En la barra de migas (sobre el editor), usa el desplegable de etapas para seleccionar una etapa.
2. O expande Propiedades y usa el desplegable de etapas alli. Ambos se sincronizan instantaneamente con el arbol de archivos.
3. Para promover una nota, cambia el desplegable de una etapa a la siguiente. En modo Enfoque, haz clic en "Promover a Permanente" en la parte inferior.
4. Para eliminar una etapa, selecciona "— Etapa —" del desplegable.

#### Donde se ve?

- **Barra de migas**: desplegable con emoji + nombre de etapa
- **Panel de propiedades**: desplegable cuando existe la propiedad `stage`
- **Arbol de archivos**: icono emoji junto al nombre de la nota
- **Pie del modo Enfoque**: boton "Promover a Permanente"

### 17.7 Pulso de revision

#### Que es?

El Pulso de revision es un sistema de resurgimiento espaciado que trae notas de vuelta a tu atencion en intervalos crecientes: 1 dia, luego 3, luego 7, luego 14, luego 30 dias despues de la ultima revision. Tambien monitorea notas etiquetadas con `#assumption` o `#model` como puntos de control de modelos mentales, y mantiene una cola de "Nunca revisadas" para notas capturadas pero nunca revisitadas.

#### Por que importa?

El conocimiento se desvanece sin revisitacion. Escribes una nota hoy y en tres semanas olvidas que existe. La repeticion espaciada es la tecnica mas establecida en ciencias cognitivas para combatir este deterioro. El Pulso de revision aplica este principio a tus notas reales.

#### Como usarlo

1. Haz clic en la pestana **Pulso de revision** en la barra lateral izquierda. Veras tres secciones: Pendientes de revision, Puntos de control de modelos mentales (`#assumption` / `#model`), y Nunca revisadas.
2. Haz clic en cualquier nota para abrirla y leerla.
3. Elige una de tres acciones:
   - **Revisada** (marca de verificacion) — programa la siguiente revision en el proximo intervalo (1 → 3 → 7 → 14 → 30 dias).
   - **Posponer 7d** (icono de ojo) — pospone la nota 7 dias sin avanzar el intervalo.
   - **Descartar** (icono de archivo) — elimina la nota de la cola de revision permanentemente.
4. Abre la Paleta de Comandos y escribe "Review due notes" para ir directamente a las notas pendientes.

#### Donde se ve?

- **Barra lateral izquierda**: La pestana Pulso de revision con contador de notas pendientes
- **Paleta de Comandos**: Comando "Review due notes" para acceso rapido

### 17.8 Senderos

#### Que es?

Los Senderos son secuencias ordenadas y nombradas de notas — como capitulos de un libro o paradas en un recorrido guiado por tu conocimiento. Se definen agregando `trail: true` al frontmatter de una nota, y luego listando wikilinks en orden en el cuerpo de la nota.

#### Por que importa?

El conocimiento no siempre es una red. A veces es un camino — una secuencia de aprendizaje, una progresion de argumentos, una narrativa. Los Senderos capturan ese orden explicitamente, anadiendo una dimension lineal a tu biblioteca no lineal.

#### Como usarlo

1. Crea una nueva nota con `trail: true` en el frontmatter.
2. En el cuerpo de la nota, lista wikilinks en el orden deseado.
3. Al abrir cualquier nota que pertenezca a un sendero, la barra de migas muestra un indicador con el nombre del sendero y la posicion (ej. "Mi Sendero 2/5"). Flechas de navegacion permiten ir a la nota anterior y siguiente.
4. Abre la Paleta de Comandos y escribe "Open Trail" para ver todos los senderos.

#### Donde se ve?

- **Barra de migas**: Indicador del sendero con nombre, posicion y flechas de navegacion
- **Paleta de Comandos**: Comando "Open Trail" lista todos los senderos

### 17.9 Vistas multi-lente

#### Que es?

Las Vistas multi-lente permiten ver tu biblioteca a traves de diferentes esquemas de clasificacion — sin cambiar la estructura de carpetas ni duplicar notas. Una "lente" es una agrupacion virtual que reorganiza las notas segun una propiedad o etiqueta. Lentes integradas: "Por etapa" (Fugaz/Literatura/Permanente/Sintesis) y "Por tema" (agrupadas por etiquetas). Puedes crear lentes personalizadas en Ajustes.

#### Por que importa?

Las estructuras de carpetas imponen una sola jerarquia, pero el conocimiento no cabe en un solo arbol. Las Vistas multi-lente permiten cambiar entre perspectivas sin mover archivos. Las mismas notas, vistas a traves de diferentes lentes organizativas.

#### Como usarlo

1. En la barra lateral, encuentra el **selector de lentes** en la parte superior del arbol de archivos (por defecto "Carpetas").
2. Selecciona una lente: "Por etapa", "Por tema" o una lente personalizada. La barra lateral se reorganiza al instante.
3. Selecciona "Carpetas" para volver al arbol de archivos predeterminado.
4. Para crear una lente personalizada: abre **Ajustes > Gestion del conocimiento**, haz clic en **Crear lente**, nombra y elige la propiedad de frontmatter para agrupar.
5. O usa la Paleta de Comandos: escribe "Create Lens".

#### Donde se ve?

- **Selector en barra lateral**: Selector de lentes en la parte superior del arbol de archivos
- **Ajustes > Gestion del conocimiento**: Crear, editar y eliminar lentes personalizadas
- **Paleta de Comandos**: Comando "Create Lens"

### Configuracion del Motor Cognitivo

Todas las herramientas del Motor Cognitivo se configuran desde **Ajustes > Motor Cognitivo**:

- **Clasificacion de estratos** — Activar o desactivar la clasificacion automatica
- **Seguimiento de madurez** — Activar o desactivar el seguimiento del ciclo de madurez
- **Enlaces tipados** — Ajustar el umbral de sensibilidad para la deteccion de enlaces (0.0 – 1.0)
- **Detector de tensiones** — Activar o desactivar la deteccion automatica de tensiones
- **Sobrescritura manual** — Agrega propiedades `stratum` y `maturity` en el frontmatter para sobrescribir la clasificacion automatica

---

*Manual de Usuario de Constellation — Version 0.1.0 — Marzo 2026*
*uconstellation.world*

---

## 23. Conexiones sugeridas

Constellation sirve para *formular* conocimiento, y el conocimiento es conexión. Las **Conexiones sugeridas** encuentran las notas que ya están en tu Biblioteca y que más se relacionan con la que estás mirando — los parientes con los que debería enlazar pero aún no lo hace — y convierten cualquiera de ellas en un **enlace tipado** con un solo clic. Es "más como esto", pero para el pensamiento.

**Cada sugerencia está tipada.** Cuando aceptas una, Constellation pregunta *cómo* se relacionan las dos notas — apoya, contradice, ejemplifica, deriva-de, y así sucesivamente, o simplemente **asociativo**. Un enlace tipado es una pieza de razonamiento que luego podrás leer, buscar y cuestionar; la función nunca añade enlaces en bloque ni añade un enlace sin tipo de forma silenciosa. (Consulta **Formulación del conocimiento** y **Propiedades**.)

**Cómo las encuentra.** Los candidatos provienen **únicamente de tu propia Biblioteca**, clasificados frente al índice de búsqueda en vivo de Constellation por el vocabulario compartido más *distintivo* — las palabras raras y reveladoras, no las comunes. Cada sugerencia muestra los **términos en común** que explican por qué apareció, de modo que nunca aceptas una conjetura de caja negra.

**Cinco lugares, una lista.** La misma lista de sugerencias aparece en el **Revisor** (🕐, para las notas que marca como *huérfanas* o *frágiles*), la **Pestaña de Retroenlaces** (barra lateral derecha), el **Inspector 360°**, la **Pestaña de Salud** y la **Vista del Cielo** (🌌 — haz clic derecho en cualquier estrella → **Sugerir conexiones…**).

**Entrantes frente a salientes — y por qué no eliges tú.** Las superficies de diagnóstico (el **Inspector 360°** y la **Pestaña de Salud**) sugieren conexiones **entrantes** — *qué notas deberían apuntar **aquí***. Las superficies generales (la **Pestaña de Retroenlaces** y la **Vista del Cielo**) sugieren conexiones **salientes** — *hacia dónde debería apuntar esta nota*. La superficie elige la dirección que conviene a su función; tú eliges la nota y el tipo. (Una futura actualización te permitirá cambiar la dirección por ti mismo.)

**Cómo usarlo.** Bajo el encabezado **Conexiones sugeridas** verás notas relacionadas clasificadas de la más cercana a la más lejana, cada una con sus términos en común. Haz clic en el botón **Enlazar** de un candidato → en el pequeño menú **"¿Cómo se relacionan?"** elige el tipo de relación → el enlace tipado se crea **al instante** y la sugerencia desaparece de la lista. Después vive en las **propiedades** de la nota y aparece en sus retroenlaces/enlaces salientes y por todo el grafo. Si nada encaja de verdad, déjalas — o, en el Revisor, marca la nota como **independiente** deliberada. Las Conexiones sugeridas proponen; tú decides.

**Local, privado, sin bloqueos.** Las sugerencias se calculan a demanda y solo a partir de tu Biblioteca — nada sale de tu dispositivo — y reunirlas nunca bloquea tu escritura (verás un breve "Buscando notas relacionadas…" mientras trabaja). Las sugerencias, las pistas de términos en común y los tipos de relación aparecen todos en el idioma que elijas y se reflejan correctamente en las escrituras de derecha a izquierda.

---

## 24. Colores cognitivos y menús de clic derecho

### Estilo de Propiedades (Diseñador de estilo)

Abre el **Diseñador de estilo (Style Setter)** (Configuración → Apariencia → ✦ Abrir Diseñador de estilo, o su propia pestaña) y elige la categoría **Propiedades** para reestilizar las pequeñas etiquetas dentro del frontmatter de una nota. Dos elementos: **Etiquetas de propiedades** (las fichas ordinarias de tipo `tags` — Fondo de etiqueta, Texto de etiqueta, Radio de etiqueta 0–20 px, Altura 14–32 px) e **Insignias de taxonomía** (Fondo, Texto, Radio 0–20 px). Una vista previa en vivo en el centro se actualiza mientras editas; cada valor empieza exactamente con el aspecto de hoy, así que nada cambia hasta que tocas un control. Haz clic en **Conservar (Keep)** para guardar en este Universo.

### Colores cognitivos (Diseñador de estilo)

La categoría **Colores cognitivos** te da **un color compartido por estado cognitivo**, de modo que cada superficie que muestra ese estado coincide. Cinco conjuntos:

- **Madurez** — Semilla, Retoño, Perenne, Canónica, Marchita.
- **Confianza** — Hipótesis, Evidencia, Establecido, Cuestionado.
- **Origen** — Recibido, Descubierto, Mixto, Ninguno.
- **Etapa** — Chispa, Nacimiento, Crecimiento, Madurez, Latencia, Archivado.
- **Categoría de coincidencia** (por qué coincidió un resultado de búsqueda) — Título, Contenido, Etiqueta, Wikilink, Propiedad, Semántico, Estructurado.

El comportamiento es **unificar bajo demanda**: nada cambia hasta que eliges un color. Cada superficie conserva su color actual como valor de reserva, y en el momento en que fijas aquí el color de un estado, **todas** las superficies que muestran ese estado — árbol de archivos, pestañas, el inspector de la nota, el resaltado de búsqueda dentro del editor, la insignia de coincidencia y el resaltado del resultado de búsqueda — adoptan tu color a la vez. Deja un estado sin tocar y se verá exactamente como antes. Haz clic en **Conservar (Keep)** para guardar.

### Menús de clic derecho

Constellation te da un menú contextual en tres lugares, cada uno ofreciendo solo las acciones que encajan donde hiciste clic:

- **Clic derecho en el cuerpo de la nota** — Añadir enlace / Añadir enlace externo; **Formato ▸** (Negrita, Cursiva, Subrayar, Tachado, Resaltar, Código en línea, Matemática, Alternar comentario, Superíndice, Subíndice, Limpiar formato); **Párrafo ▸** (Lista con viñetas/numerada/de tareas, H1–H6, Cuerpo, Cita); **Insertar ▸** (Nota al pie, Tabla, Nota destacada, Línea horizontal, Bloque de código, Bloque matemático, Imagen); Cortar / Copiar / Pegar / Pegar como texto / Seleccionar todo; y **Estilo…** (abre el Diseñador de estilo en la categoría **Editor**).
- **Clic derecho en una fila de propiedad del frontmatter** — Copiar valor, Copiar nombre, Eliminar propiedad, Añadir propiedad; luego el mismo menú de edición que el cuerpo; y **Estilo…** que abre el Diseñador de estilo en la categoría **Propiedades**.
- **Clic derecho en un resultado de búsqueda** — un subconjunto **seguro**: Abrir, Abrir en una pestaña nueva, Revelar en árbol de archivos, Copiar enlace, Copiar ruta, Marcador, Mostrar en explorador del sistema, Abrir en app predeterminada, y **Estilo…** (la categoría **Colores cognitivos**). Por diseño **no hay Renombrar, Mover ni Eliminar** aquí — el panel de búsqueda no mantiene una copia al segundo del árbol de archivos, así que las acciones destructivas se quedan en el árbol de archivos, donde la vista siempre está actualizada.

Cada entrada **Estilo…** aterriza en la categoría del elemento sobre el que hiciste clic derecho, así que nunca tienes que buscar los controles adecuados. Cada elemento de menú, nombre de categoría y etiqueta de estado aparece en el idioma de interfaz que hayas elegido y se refleja en los diseños de derecha a izquierda.

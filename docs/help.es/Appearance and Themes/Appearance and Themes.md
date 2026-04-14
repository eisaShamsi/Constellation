---
aliases:
  - Temas
  - Style Settings
  - Tema personalizado
  - Importar tema de Obsidian
  - Eliminar tema
  - Exportar ajustes de estilo
description: Personaliza cada parte visible de Constellation — temas, colores, tipografía y estilo de componentes mediante Apariencia y la pestaña nativa Style Settings.
---

# Apariencia y Temas

La apariencia de Constellation se controla desde dos lugares en **Configuración**:

1. **Apariencia** — elige o crea un tema, importa temas del registro de la comunidad de Obsidian y ajusta las preferencias globales de fuente y diseño.
2. **Style Settings** — una pestaña dedicada que expone cada pieza visible de la interfaz de Constellation como un control ajustable en vivo (deslizadores, selectores de color, menús desplegables). Los cambios se aplican al instante y se guardan en el tema activo.

Juntos te permiten remodelar la aplicación para que coincida con tu flujo de trabajo, tamaño de pantalla y gusto personal, sin editar una sola línea de CSS.

## Temas

Un **tema** es un paquete con nombre de colores, ajustes y CSS que define cómo se ve Constellation. Constellation incluye seis temas integrados (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark), todos emparejados entre modo claro y oscuro.

### Elegir un tema

1. Abre **Configuración → Apariencia**.
2. Haz clic en cualquier tarjeta de la cuadrícula **Temas**. El tema se aplica de inmediato.
3. La tarjeta activa se resalta con un borde de acento.

### Crear un tema personalizado

1. En la cuadrícula de temas haz clic en la tarjeta discontinua **+ Nuevo tema**.
2. Dale un nombre, elige claro u oscuro y selecciona cinco colores (fondo, superficie, texto, acento, borde).
3. Haz clic en **Guardar**. Tu tema ahora aparece en la cuadrícula.

Todas las demás variables (estados hover, sombras, texto atenuado) se derivan automáticamente de tus cinco colores mediante matemática HSL, así solo controlas lo que importa.

### Editar o eliminar un tema personalizado

Pasa el ratón sobre cualquier tarjeta de tema personalizado:
- **✏️ (lápiz)** — abre el editor para cambiar nombre, tipo o los cinco colores principales.
- **✕ (X roja)** — elimina el tema tras confirmación. Los temas integrados no se pueden eliminar. Si eliminas el tema activo, Constellation vuelve al predeterminado.

### Importar un tema de la comunidad de Obsidian

Haz clic en **🟣 Temas de Obsidian** para explorar más de 200 temas comunitarios:
1. Busca por nombre o autor.
2. Haz clic en **Vista previa** para ver una maqueta del diseño y la paleta de cinco colores.
3. Haz clic en **Importar** — se descarga el CSS del tema, se adapta para Constellation (shim de selectores + extracción de variables + colores de sintaxis de CodeMirror) y se añade a tus temas personalizados.
4. Si el tema admite **Style Settings**, el recuento aparece en su tarjeta; esas opciones aparecen en la pestaña Style Settings tras la importación.

## Style Settings

La pestaña **Style Settings** es el panel de control nativo de Constellation, independiente del tema. Cubre cada pieza visible del marco además del editor, y funciona con cualquier tema (integrado, personalizado o importado).

### Cómo está organizado

Las secciones están plegadas por defecto. Haz clic en el chevron para expandir:

- **Constellation — Colores** — fondo y superficies, texto, acento
- **Constellation — Tipografía** — tamaños de fuente de interfaz/nota/código, tamaños H1–H6, peso de encabezados, altura de línea, espacio entre párrafos
- **Constellation — Diseño y forma** — esquinas (radios pequeño/mediano/grande), anchos de borde, sombras, longitud de línea del editor, márgenes laterales
- **Constellation — Componentes** — barra de listones, barra de acciones lateral, barra de diseño (alternadores de panel), barra superior/tiras de pestañas, barra de estado, explorador de archivos, barra lateral derecha, botones, etiquetas, callouts
- **Constellation — Editor** — enlaces, código y bloques, cita en bloque, cursor y selección

### Cambiar un valor

- **Selectores de color** — haz clic en la muestra, elige un color. El hex aparece al lado.
- **Deslizadores** — arrastra para ajustar. El valor numérico aparece en la unidad (px, %, etc.).
- **Interruptores** — haz clic para alternar clases (sobre todo para temas importados).
- **Menús desplegables** — elige una opción (estilo de decoración de enlace, etc.).
- **Flecha de reinicio (↺)** — aparece al pasar el ratón al final de cada fila. Al hacer clic borra tu anulación y restaura el predeterminado del tema.

### Cómo funciona el guardado

- Los cambios se guardan automáticamente en los **styleSettingsValues** del tema activo.
- Si cambias un Style Setting mientras un tema integrado está activo, Constellation **auto-clona** el integrado en tus temas personalizados (como `{Nombre} (custom)`), luego guarda tus cambios allí. El integrado no se toca.
- La etiqueta **Guardado en:** en la parte inferior de la pestaña muestra qué tema contiene actualmente tus anulaciones.
- Haz clic en **Restablecer todo a predeterminados** para borrar todas las anulaciones en el tema activo.

### Importar / Exportar Style Settings

Barra de herramientas en la parte superior de la pestaña Style Settings:

- **📋 Pegar desde el portapapeles** — un clic: lee el portapapeles y fusiona JSON válido en el tema activo.
- **⬆️ Importar / Pegar** — abre un área de texto; pega JSON manualmente. Elige **Fusionar** (añadir/anular) o **Reemplazar todo** (borrar, usar solo lo pegado).
- **📄 Desde archivo** — abre un archivo `.json` exportado desde el plugin Style Settings de Obsidian u otra instalación de Constellation.
- **📋 Copiar** — copia los valores actuales al portapapeles como JSON formateado.
- **⬇️ Exportar** — guarda los valores como `{theme-name}-style-settings.json`.

El formato JSON coincide exactamente con el plugin Style Settings de Obsidian — un objeto plano que mapea IDs de configuración a valores de cadena:

```json
{
  "h1-size": "36",
  "interactive-accent": "#7c3aed",
  "my-themed-color@@light": "#ffffff",
  "my-themed-color@@dark": "#1e1e2e"
}
```

Esto significa que puedes copiar tus Style Settings de Obsidian y pegarlos directamente en Constellation, o viceversa.

## Qué puedes controlar

Cada ajuste vive bajo uno de los cinco bloques anteriores. Aspectos destacados:

### Tipografía

- **Tamaño de fuente de interfaz** — barra lateral, barras de herramientas, menús
- **Tamaño de fuente de nota** — texto del cuerpo en el editor
- **Tamaño de fuente de código** — código en línea y bloques de código
- **Tamaños H1 – H6** — cada nivel de encabezado individualmente
- **Peso de encabezados** — ligereza o negrita de todos los encabezados
- **Altura de línea** — normal (cuerpo) y ajustada (encabezados y UI densa)
- **Espacio entre párrafos** — separación entre párrafos

### Componentes del marco

- **Barra de listones (iconos izquierdos)** — ancho, tamaño de botón, tamaño de icono, radio, colores
- **Barra de acciones lateral** — iconos nueva nota/tabla/carpeta — tamaño, color, altura, fondo
- **Barra de diseño (alternadores de panel)** — alternadores barra lateral izquierda/división/derecha — tamaño de botón, tamaño de icono, colores, color estado activo
- **Barra superior/Tiras de pestañas** — solo visible cuando hay notas abiertas en pestañas; controla altura de tira, fondo, altura/fuente/radio de pestaña, colores de pestaña activa e inactiva
- **Barra de estado** — altura, tamaño de fuente, fondo, color de texto
- **Barra lateral derecha (inspector)** — fondo, altura de fila de pestañas, tamaño de icono de pestaña, colores
- **Explorador de archivos (barra lateral izquierda)** — fila de notas del Universo, filas de universos hijo (cUniverse), nombres de bibliotecas, carpetas, notas — cada uno con tamaño, peso y color independientes; más espaciado vertical de filas

### Editor

- **Tamaños de encabezados** (H1–H6) y peso
- **Altura de línea** en el cuerpo de la nota
- **Código en línea** fondo, color de texto, radio, tamaño de fuente
- **Color de enlace** (por defecto + hover) y estilo de decoración (ninguno/subrayado/punteado)
- **Ancho de barra de callout** y **radio de callout**
- **Color de cursor** y **fondo de selección**

### Colores (cada color en la aplicación)

- Fondo (primario/alt), superficies, fondo hover, bordes, fondo de entrada
- Texto (normal/atenuado/débil/sobre acento), estados de error/aviso/éxito
- Acento (acento interactivo + hover), texto sobre acento

## Preguntas frecuentes

### ¿Puedo diseñar la barra de título de Windows ("Constellation v0.3.4 — …")?

No — esa barra la dibuja el sistema operativo (Windows/macOS/Linux). Constellation no tiene acceso CSS a ella. Todo lo que está debajo es totalmente estilizable.

### ¿Por qué no funciona el deslizador de ancho de la barra lateral?

El ancho de la barra lateral se controla mediante el manejador de arrastre en el borde de la barra (arrastra para redimensionar). Deliberadamente no duplicamos ese control en Style Settings para evitar fuentes de verdad en conflicto.

### ¿Dónde viven mis Style Settings?

Dentro de `Universe/settings.json` bajo `customThemes[i].styleSettingsValues`, ámbito por tema. Viajan con tu Universo — si sincronizas tu directorio de Universo entre dispositivos, tu estilo viene contigo.

### ¿Puedo compartir un tema con alguien?

Sí:
- **Tema completo** — en el editor de temas, haz clic en **Exportar**. Comparte el archivo `.json`. El destinatario hace clic en **↓ Importar** en la cuadrícula de temas y lo selecciona.
- **Solo valores de Style Settings** — en la pestaña Style Settings, haz clic en **Exportar** para exportar solo los valores de deslizadores/colores (sin la estructura del tema). Útil para aplicar tus ajustes personales sobre el tema de otra persona.

### Un tema de Obsidian importado se ve roto. ¿Qué hago?

Los temas de Obsidian pueden ser complejos. Casos conocidos:
- Temas que usan **colores HSL divididos** (como Minimal) — soportados en Constellation desde esta versión en adelante.
- Temas que dependen de la estructura DOM específica de Obsidian pueden renderizarse parcialmente. Constellation incluye un shim de clases que mapea los selectores más comunes, pero los temas muy estructurales pueden requerir ajustar los cinco colores principales o corregir los valores de Style Settings a mano para compensar.

## Relacionado

- [[Universe]] — donde se almacenan los temas y valores de Style Settings
- [[Libraries]] — acentos de color por biblioteca (definidos en los ajustes de biblioteca, independientes de los temas)
- [[Importer]] — para importar notas, no temas (la importación de temas está en Apariencia)

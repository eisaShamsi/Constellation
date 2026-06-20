---
aliases:
  - Panel de Calendario
  - Calendario de Notas Diarias
  - Calendarios Culturales
description: Una vista mensual a página completa a través de ocho calendarios, con días en los que se puede hacer clic, creación de notas diarias, fechas de vencimiento de tareas y registro de fechas culturales.
---

# Calendario

El **Calendario** es una vista mensual a página completa, que se abre desde el **dock izquierdo** (el icono del calendario). Los días que tienen notas o tareas pendientes se marcan con **puntos** de colores. La cabecera muestra el mes en el calendario que hayas elegido; si defines un **calendario secundario**, un subtítulo debajo muestra el rango equivalente de ese calendario (por ejemplo, un mes gregoriano muestra su intervalo hijrí, "Dhul-Hijjah 1447 – Muharram 1448 AH").

## Hacer clic en un día

Cada celda de día es interactiva:

| Acción | Resultado |
|--------|--------|
| Haz clic en el espacio vacío (o en el número del día) | Abre — o crea — la **nota diaria** de ese día. Hacer clic en una fecha que ya tiene una nota diaria simplemente la **abre**; nunca crea un duplicado. |
| Haz clic en un punto | Abre ese elemento concreto. Si un día tiene varias notas o tareas, al hacer clic en el punto se muestra una pequeña **lista** para elegir. |
| Haz clic en un punto de tarea | Abre la nota **desplazada hasta la línea de esa tarea**, lista para editar. |

### Colores de los puntos

| Color del punto | Significado |
|-----------|---------|
| Dorado | La **nota diaria** de ese día |
| Morado | Otra **nota** editada (o fechada) ese día |
| Rojo | Una **tarea** que vence ese día |

Todos los colores de los puntos — y cualquier otra parte del calendario — son personalizables en la superficie **Style Setter → Calendario**.

> [!tip]
> En la lista de tareas puedes **marcar la casilla de una tarea para completarla** directamente desde el calendario — las tareas completadas desaparecen al instante. Solo aparecen en el calendario las tareas que llevan su propia fecha de vencimiento `📅 YYYY-MM-DD` (esa fecha es lo que las sitúa en un día).

## Calendarios culturales (ocho)

En **Configuración → Calendario** puedes definir el **sistema de calendario**, y toda la cuadrícula del mes cambia a ese sistema:

- **Gregoriano**
- **Hijrí (Islámico)** — un motor astronómico preciso; los meses sagrados se resaltan y los eventos islámicos se señalan.
- **Hijrí Solar (Persa)**
- **Hebreo**
- **Indio (Saka)**
- **Budista**
- **Chino** — *lunisolar*
- **Coreano** — *lunisolar*

Cada celda muestra tanto la fecha del calendario elegido (grande) como la fecha gregoriana (pequeña), además de la fase lunar. La cabecera de cada mes muestra el **nombre del mes, su número entre paréntesis y el año** — el número ayuda con los calendarios cuyo orden de meses no resulta familiar.

Los calendarios **chino y coreano** son *lunisolares*: a veces intercalan un **mes bisiesto** (闰六月 / 윤6월), que el calendario presenta como una página propia para que la navegación nunca lo omita ni lo duplique.

También puedes elegir el **inicio de semana** (domingo/lunes) y activar o desactivar la **columna de número de semana**.

### Opciones del calendario hijrí

En **Configuración → Calendario → "Calendario hijrí (islámico)"** hay dos controles adicionales:

- **Método de cálculo** — **Astronómico (Conjunción Lunar)**, que sigue la luna nueva real (el más preciso, y el predeterminado), o **Tabular (al-Tawfīqāt al-Ilhāmiyyah)** (la tabla aritmética clásica), el ciclo aritmético clásico.
- **Corrección de mes** — ajusta el comienzo de un mes hijrí en ±1 o ±2 días para que coincida con un **avistamiento local de la luna**. Elige el año y el mes hijrí, escoge un desfase y haz clic en **Establecer**; la corrección se aplica a ese mes y a todos los meses posteriores. Tus correcciones aparecen listadas (cada una se puede eliminar), con un botón **Borrar todo**.

Ambos ajustes (y tus correcciones) se guardan **con tu universo**, de modo que viajan entre tus dispositivos.

### Opciones de visualización chino y coreano

Corea usa el calendario lunar chino, así que ambos comparten fechas idénticas — lo que los distingue es la **escritura** y el **año**. Cuando cualquiera de los dos es tu calendario principal o secundario, **Configuración → Calendario** muestra dos controles adicionales:

- **Visualización del año** — chino: el ciclo sexagenario 丙午年, el año a secas, o ambos; coreano: la era **Dangi** 단기 4359, el año, o el sexagenario 병오년.
- **Nombres de los meses** — *escritura nativa* (五月 / 5월), o *fonético* — la pronunciación del mes escrita en tu propio idioma (Inglés "Wǔyuè / Owol"; Árabe "وُو-يوي / أوه-وُل").

## Dar estilo al calendario

Abre el **Style Setter** (dock izquierdo, o **Configuración → Style Setter**) y elige la superficie **Calendario** para reestilizar cada parte — cada elemento tiene su propio **color y tamaño de texto** (números de los días, la fecha de referencia cruzada, la pastilla del mes, las cabeceras de los días de la semana, los números de semana, el glifo lunar, el resaltado de Hoy, las líneas de la cuadrícula y los puntos de nota/tarea/evento), además de la **fuente** del calendario. Una vista previa en vivo a tamaño completo se actualiza mientras editas; haz clic en **Conservar** para aplicar.

## Notas diarias

El Calendario sirve plenamente a las notas diarias: haz clic en cualquier día para abrirlo, o ejecuta el comando **"Nota Diaria"** (paleta de comandos) para saltar al día de hoy.

> [!tip]
> **Los nombres de archivo de las notas diarias siempre permanecen en gregoriano** (`YYYY-MM-DD`) independientemente del calendario mostrado — así tus archivos siguen siendo portables y se ordenan correctamente. La fecha cultural se muestra en el calendario, y puede registrarse en el frontmatter de la nota (más abajo).

## Registrar una fecha cultural en una nota

Dos herramientas opcionales escriben la fecha cultural en las **propiedades** de una nota (el nombre del archivo siempre permanece en gregoriano `YYYY-MM-DD`):

- **Sello hijrí en las notas diarias** — *Configuración → Calendario → "Sellar la fecha hijrí en las notas diarias."* Cuando está activado (disponible solo mientras el calendario hijrí sea tu **principal o secundario**), cada **nueva** nota diaria recibe una línea `hijri:`, por ejemplo `hijri: 1448-01-06`. Las notas que ya tienes nunca se modifican.
- **"+ Hijrí" en las Propiedades de una nota** — abre las **Propiedades** de cualquier nota, pasa el ratón sobre la fecha y aparecerá un pequeño botón **"+ Hijrí"** (además de "+ Jalali", "+ Hebreo", etc. — **un botón por cada calendario no gregoriano que hayas seleccionado**). Haz clic en él y Constellation lee la fecha gregoriana de la nota y añade la equivalente, por ejemplo `jalali: 1405-03-30`. El botón coreano escribe el año **Dangi**; un **mes bisiesto** chino/coreano se marca con una `L` (por ejemplo `chinese: 2025-06L-17`). Si la nota no tiene una propiedad de fecha, se usa la fecha de creación del archivo.

> [!tip] RTL Support
> La cuadrícula del calendario respeta la dirección de texto actual. En idiomas RTL (árabe, hebreo, persa, urdu), la disposición del calendario se ajusta en consecuencia.

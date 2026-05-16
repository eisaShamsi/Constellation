---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: es
source: docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md
aliases:
  - Constellation Sight
  - Vistas Coordinadas
  - Domo de Anclaje
  - Mini-Domos
description: Constellation Sight visualiza todo tu universo de conocimiento como un domo de anclaje estratificado con cuatro mini-domos coordinados que recodifican las mismas notas a través de diferentes canales (Confianza, Etapa, Actos, Procedencia). Pasa el cursor sobre cualquier estrella para verla en todas partes; haz clic en chips de la barra lateral o Shift+clic en estrellas para filtrar; promueve cualquier mini-domo para inspeccionarlo a tamaño completo con zoom.
---

# Constellation Sight

## ¿Qué es?

**Constellation Sight** es el **instrumento diagnóstico** de tu universo de conocimiento. Un **domo de anclaje** central muestra cada nota posicionada por **estrato** (profundidad del pensamiento) y **tiempo** (cuándo se escribió), con cuatro **mini-domos** al lado que recodifican el mismo universo a través de diferentes canales: **Confianza**, **Etapa**, **Actos**, **Procedencia**.

Responde a una pregunta con cinco lentes complementarias: **«¿Cómo está moldeado y organizado mi Contenido Epistémico?»**

Al pasar el cursor sobre cualquier estrella en cualquier domo, la misma nota se ilumina en las cinco superficies — anillo dorado en la estrella, tinte dorado en los chips correspondientes de la barra lateral. Haz clic en un chip de la barra lateral y las cinco vistas se estrechan. Shift+clic en una estrella en el mini Etapa filtra el universo instantáneamente a esa etapa del ciclo de vida. Haz clic en el espacio vacío de cualquier mini-domo y "promueve" al slot principal a tamaño completo, mientras el principal anterior baja al slot mini vacante.

## ¿Por qué importa?

La mayoría de apps de notas te muestran lo que escribiste. Constellation Sight te muestra la **forma** de lo que sabes.

- ¿Dónde se **concentra** tu pensamiento? (el gradiente de densidad en el anclaje)
- ¿Qué está aún **en etapa temprana** vs **fundamento estable**? (gradiente de color del mini Etapa)
- ¿Qué notas son **portantes** vs **aisladas**? (codificación de tamaño del mini Actos)
- ¿De dónde vino cada idea — tu propio pensamiento, lectura, escucha, tradición? (disposición sectorial del mini Procedencia)
- ¿Cuán **confiado** estás en tus conclusiones? (gradiente de opacidad del mini Confianza)

Una nota en el centro del anclaje (alta conectividad → portante) pero en cian en el mini Etapa (`spark` — apenas iniciada) te dice algo diagnóstico: una idea portante que aún no ha madurado.

## Cómo abrirlo

1. Haz clic en el **icono ojo** en el dock al borde izquierdo de Constellation.
2. El domo de anclaje se renderiza en 2–5 segundos para la mayoría de universos.
3. Para cerrar: haz clic en **(×)** arriba a la derecha, o presiona **Esc**.

## Lo que ves

### La Franja de Encabezado

Parte superior, de izquierda a derecha:
- **«Constellation Sight»** — el título.
- **«v6.1 — Coordinated Views (Phase 2)»** — subtítulo de versión.
- **Insignia «X / Y notes»** dorada — visible solo cuando un filtro está activo.
- **Insignia «EXTENDED»** en mayúsculas pequeñas doradas — visible solo cuando la vista Extendida está activada.
- **Botón «Reset View»** — visible solo cuando el diseño se ha cambiado.
- **(×)** botón de cerrar — siempre presente.

### El Domo de Anclaje (Slot Principal)

Gran domo crema-sobre-oscuro en el centro:
- **Anillos de estrato** — 5 círculos concéntricos. El más interno = tus notas más fundamentales; el borde exterior = tus chispas más recientes.
- **Borde de calendario** — 12 etiquetas de mes alrededor del exterior.
- **Etiquetas de estrato** — texto en cursiva arriba.
- **Estrellas** — cada nota dibujada como pequeño punto crema, posicionada por estrato × tiempo.
- **Líneas de conexión** — bordes de enlaces tipados entre notas, baja opacidad debajo de las estrellas.
- **Anillo de hover** — círculo dorado alrededor de cualquier estrella sobre la que pase el cursor.

### Los Cuatro Mini-Domos

Lado derecho, cuadrícula 2×2. Ocultos por defecto; revelados con **Ctrl+D** (solo sesión) o **Ctrl+Shift+D** (persistente — ver vista Extendida).

Cada mini renderiza el mismo universo a través de una codificación:

1. **CONFIDENCE — opacidad.** Notas más confiadas más brillantes; tentativas se desvanecen.
2. **STAGE — color (disco completo).** Color categórico por etapa del ciclo de vida:
   - **Cian** = `spark` (idea recién encendida)
   - **Naranja** = `birth` (tomando forma)
   - **Violeta** = `growth` (en movimiento activo)
   - **Verde** = `maturity` (completamente formada)
   - **Amarillo** = `renewal` (recientemente revisitada)
   - **Gris** = `dormancy` / `archival` (inactiva / cerrada)
3. **ACTS — tamaño (decil superior).** Top 10% por cantidad de enlaces = puntos más grandes; el resto pequeño.
4. **PROVENANCE — 5 sectores.** Estrellas reposicionadas en 5 sectores angulares: **Self / Read / Heard / Reasoned / Tradition**.

### El Anclaje Degradado (Cuando un Mini es Promovido)

Si promueves cualquier mini al slot principal, el anclaje baja al slot mini vacante. Allí se renderiza como **puntos crema neutros** con el título **«UNIVERSE — primary view»**.

### La Barra Lateral de Facetas (Borde Izquierdo)

Panel colapsable con **6 grupos de filtro de facetas**, cada uno mostrando categorías con conteos en vivo:

- **Folder** — jerarquía de carpetas
- **Library** — nombres de bibliotecas
- **Stratum** — Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing
- **Confidence** — Hypothesis / Evidence / Established / Contested
- **Stage** — Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival
- **Provenance** — Self / Read / Heard / Reasoned / Tradition

Clic en la pestaña **▶** del borde para expandir. Clic en cualquier chip para alternar como filtro.

## Interacción

| Gesto | Efecto |
|---|---|
| **Hover sobre estrella** | Anillo dorado en la misma estrella en las 5 superficies + chips correspondientes tinte dorado. |
| **Clic simple en estrella** | Abre la nota en el editor. Botón **«Return to Sight»** aparece. |
| **Shift+clic en estrella** en mini Stage / Confidence / Provenance | Alterna filtro en la categoría de esa estrella. |
| **Shift+clic en estrella** en Acts o anclaje | Sin efecto. |
| **Clic en área vacía de un mini** | Ese mini promueve al slot principal. |
| **Rueda zoom (principal)** | Zoom hacia el cursor. Rango: 0.5× a 24×. |
| **Clic+arrastrar área vacía** | Pan de la vista. |
| **Ctrl+0 / Cmd+0** | Restablece zoom + pan. |
| **Ctrl+D / Cmd+D** | Alterna visibilidad mini-domos — **solo sesión**. |
| **Ctrl+Shift+D / Cmd+Shift+D** | Alterna **vista Extendida** — persistente. |
| **Clic en chip de barra lateral** | Alterna categoría de faceta en el conjunto de filtros. |
| **Botón Reset View** | Regresa al anclaje principal en zoom 1.0. |
| **Esc** | Cierra Sight. |

## Modo Fantasma — Selección Múltiple Desde el Domo

Cuando un filtro está activo, las estrellas no-coincidentes permanecen visibles pero a **baja opacidad (15%)** en lugar de desaparecer. Esto significa:

- Aún puedes VER dónde están las estrellas no-coincidentes.
- Puedes pasar el cursor sobre ellas (anillo dorado aparece).
- Puedes **Shift+clic para AÑADIR su categoría al filtro**.

## Modo Densidad

Cuando el conteo de estrellas visibles (coincidentes) excede el umbral de densidad (por defecto **5,000**), los mini-domos cambian a un **renderizado de densidad perceptual**.

## Vista Extendida

Presionar **Ctrl+Shift+D** (o **Cmd+Shift+D** en Mac) alterna la "vista Extendida" — cuando está activada, los mini-domos son visibles por defecto cada vez que abres Sight. El estado persiste a través de cierres de Sight, reinicios de app y reinicios del sistema.

## Cuándo Sight Es Más Útil

- **Auditar la forma de tu conocimiento** — abrir Sight después de una sesión de escritura.
- **Encontrar puntos ciegos** — sectores del domo con pocas notas pueden ser áreas para explorar.
- **Detectar debilidad portante** — nota posicionada centralmente en color de etapa temprana.
- **Filtrar e inspeccionar** — Shift+clic reduce el universo; promueve un mini para estudiar un canal a tamaño completo.
- **Rastrear procedencia epistémica** — promueve Provenance para ver cómo tu conocimiento se origina.

## Superficies Relacionadas

- **Constellation Nervous System (CNS)** — visualización complementaria (icono neurona junto al ojo Sight).
- **Constellation Map** — visualización de rayos solares.
- **Sky View** — visualización de enlaces basada en grafo.
- **Panel Index** — navegador de términos.

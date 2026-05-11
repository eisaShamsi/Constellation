# Metadatos Epistémicos

> **Nota de traducción:** Este tema de ayuda es una traducción
> generada por IA a partir de la versión canónica en inglés en
> `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md`.
> Pendiente de revisión por hablantes nativos. Por favor, envíe
> correcciones a través del repositorio del proyecto.

*(MIG-022 §A — extensiones de esquema del análisis de brechas §6.1)*

Este tema describe un pequeño conjunto de **campos opcionales de frontmatter** que Constellation ahora reconoce para una clasificación epistémica más rica de tus notas. Se añadieron en respuesta al análisis de brechas (`docs/epistemic-content-gap-analysis.md`) — el reconocimiento de que el modelo de dos ejes Source × Content Type contra el cual clasifica el Constellation Epistemic Content Engine (CECE) no cubre todo lo que podrías querer registrar sobre cómo llegaste a saber lo que sabes.

Estos campos son **todos opcionales**. Las notas existentes sin ellos funcionan sin cambios. Los añades a mano (o, en el futuro, mediante un editor estructurado) cuando una nota es el tipo de conocimiento que se beneficia de la señal adicional.

---

## Los campos

### `held_by` — *¿de quién es esta postura?*

Una cadena corta que indica quién sostiene la posición que la nota describe. Por defecto, `user` (tu propia postura). Otros valores que podrías usar:
- El nombre de un erudito: `held_by: "al-Shāfiʿī"`
- Una escuela: `held_by: "Ḥanafī"`
- Una figura histórica: `held_by: "Aristotle"`

Cuando escribes una nota que registra *la posición de otra persona* en lugar de la tuya propia, `held_by` es el campo que lo dice. Sin él, Constellation asume tácitamente que el estado epistémico de la nota es el tuyo — lo cual, para el trabajo académico serio, suele ser incorrecto.

### `domain` — *¿de qué materia trata esto?*

Una lista de etiquetas disciplinarias. A diferencia de tu campo libre `tags` (folksonomía / estado de ánimo / proyecto), `domain` es el campo estructurado de disciplina/tema para recuperación y filtrado. Ejemplos:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

Una nota clasificada como `content_type: "proposition"` Y `source: "inference"` podría ser un teorema lógico (domain: `[logic, mathematics]`) o una opinión jurídica (domain: `[fiqh, ʿibādāt]`) — la misma forma epistémica, contextos de recuperación muy diferentes. `domain` te permite decir cuál.

### `function` — *¿para qué sirve esta nota?*

Una sola cadena que identifica el uso previsto de la nota. Valores reconocidos:

- `reference` — leer cuando se necesite (una definición, una cita, un dato que consultarás más tarde)
- `seed` — incubar (una idea en etapa temprana que aún estás desarrollando)
- `actionable` — hacer algo con esto (una tarea, un seguimiento, una decisión por tomar)
- `shipped` — producto terminado (un ensayo publicado, un análisis entregado, un ciclo cerrado)

Distinto del eje content-type de CECE (que dice qué TIPO de conocimiento es) — `function` dice qué HARÁS con la nota.

### `provenance_civilization` — *¿qué vocabulario tradicional está en juego?*

Una cadena opcional que identifica la huella civilizacional del vocabulario de la nota. Útil para la recuperación contra corpus específicos de tradición. Ejemplos:

- `provenance_civilization: "sunni-usuli"` — tradición sunita *uṣūl al-fiqh* (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — filosofía analítica posterior a Frege
- `provenance_civilization: "nyaya"` — escuela india Nyāya de epistemología pramāṇa
- `provenance_civilization: "buddhist-pramana"` — tradición epistemológica budista (Dignāga, Dharmakīrti)

La mayoría de las notas no necesitan esto. Cuando tienes, por ejemplo, una nota que se basa tanto en *uṣūl* sunita COMO en epistemología analítica angloamericana, registrar la huella primaria ayuda al tú futuro a recuperar el material comparable adecuado.

### `updated_at` — *¿cuándo cambió por última vez tu postura?*

Fecha ISO de la revisión deliberada más reciente del contenido epistémico de la nota. Distinto del timestamp `modified` del sistema de archivos (que captura cada guardado, incluso correcciones de erratas); `updated_at` es la marca de tiempo que TÚ fijas cuando realmente has reconsiderado la posición.

```yaml
updated_at: 2026-05-09
```

Útil cuando aterrice el resto del eje temporal §6.3 (historial de estados de la nota) — hasta entonces, este es un campo de instantánea única que registra "la última vez que revisé mi postura".

### `ikhtilāf` — *desacuerdo erudito estructurado*

El más complejo de los nuevos campos. Registra el *ikhtilāf* — el desacuerdo estructurado entre eruditos o escuelas sobre una cuestión — como una lista de pares `{school, position}`. Constellation proporciona un widget personalizado del panel de Propiedades para editarlo; también puedes editar el YAML directamente.

Ejemplo:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

Una nota con `ikhtilāf` no está en ningún estado epistémico único — registra un *desacuerdo estructurado* entre múltiples agentes. Sin este campo, Constellation trataría tal nota como si sostuviera ella misma una de estas posiciones, lo cual es incorrecto.

El panel de Propiedades renderiza cada fila como una tarjeta editora con dos entradas (school + position) más un botón de eliminar, y un botón "Añadir escuela" en la parte inferior.

### `warrant` y `warrant_notes` — *parseado pero inerte (por ahora)*

Dos campos se parsean y almacenan en disco pero **aún no se exponen en ninguna UI**:

- `warrant: "mutawātir"` — una etiqueta de grado para la garantía del enunciado de la nota. La jerarquía sunita *uṣūl* usa *mutawātir / mashhūr / āḥād* y dentro del hadiz específicamente *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*. Otras tradiciones tienen sus propios vocabularios de calificación.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — texto libre que sustenta el grado de garantía.

Estos están listos para usar cuando el **Constellation Warrant Research workstream** entregue su clasificador (proyecto de investigación de varios meses; ver el análisis de brechas §6.2). Hasta entonces puedes rellenarlos a mano y los datos persisten; nada los muestra. Las consultas y distintivos futuros conscientes de la garantía leen estos valores directamente.

---

## Dónde aparecen estos campos

Cuando rellenas cualquiera de los nuevos campos en el frontmatter de una nota, aparecen en el **panel de Propiedades** (barra lateral derecha) de la misma forma que cualquier otro campo YAML — una fila por clave, con el editor adecuado al tipo:

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → entrada de texto
- `domain` → lista de etiquetas (añadir escribiendo + Enter, eliminar con la × en cada etiqueta)
- `updated_at` → selector de fecha
- `ikhtilāf` → widget personalizado con filas `school` / `position` + botones añadir/eliminar

---

## ¿Y `supersedes`?

`supersedes` es técnicamente una *relación entre notas* en lugar de una propiedad de una sola nota. Constellation lo maneja como un **enlace tipado**, no como un escalar YAML:

```markdown
Esta nota reemplaza mi análisis anterior: [[old-note-id|supersedes]]
```

El sufijo `|supersedes` en el wikilink le dice a Constellation que se trata de un enlace tipado del tipo `supersedes` — recibe un color de píldora distinto (azul-grisáceo pizarra), aparece en los paneles Backlinks + Outgoing Links junto con otros enlaces tipados, y participa en la Living Link Architecture (peso, ciclo de vida, conteos de recorrido).

Esto mantiene las relaciones nota-a-nota en un solo lugar — el sistema de enlaces tipados — en lugar de dividirlas entre enlaces tipados y escalares de frontmatter. Lo mismo aplica para `contradicts:` (ya un enlace tipado en el vocabulario pre-MIG-022).

---

## Lo que esto NO es

Estos campos **NO** son consumidos por la clasificación CECE hoy. CECE clasifica solo sobre Source × Content Type; los nuevos campos de metadatos se registran para recuperación impulsada por humanos, futuros clasificadores conscientes de la garantía, y el eje temporal (cuando aterrice).

En particular:
- `function: "actionable"` NO crea automáticamente una tarea en el panel de Tareas
- `held_by: "al-Shāfiʿī"` NO cambia cómo CECE clasifica la nota
- `domain: [fiqh]` NO filtra los resultados de tu búsqueda a menos que escribas la consulta de búsqueda para incluirlo

Los campos son **esquema** — un vocabulario reconocido de campos que puedes añadir. MIGs futuros entregarán características que los consuman (clasificador de garantía, consultas temporales, filtrado consciente del dominio, etc.).

---

## Un ejemplo trabajado

Una nota que registra las posiciones de las escuelas sunitas sobre si la ruptura del alba importa para la validez del día de ayuno obligatorio:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

La posición clásica Mālikī (una niyyah para el mes) se describe
por [[Ibn-Rushd-bidayah|derives-from]] en el pasaje sobre niyyah en
bidāyat al-mujtahid. Mi punto de vista actual: [[ramadan-niyyah-personal|supersedes]]
mi nota anterior que confundía la posición Mālikī con la Shāfiʿī.
```

Seis de los siete nuevos campos rellenados; `warrant_notes` omitido (sin detalle de cadena de transmisión que registrar todavía); `supersedes` y `derives-from` como enlaces tipados en el cuerpo, no como escalares YAML.

---

*MIG-022 §A — las extensiones de esquema aterrizan en este build de Constellation. El Warrant Research workstream (Concept Paper separado, varios meses) entrega el clasificador de garantía que consume el campo `warrant`. El eje temporal (MIG-023, ciclo Architect separado) consume `updated_at` más el historial más amplio de estados de la nota.*

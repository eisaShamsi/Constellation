---
id: masadir
name: masādir
family: sunni-islamic-usul
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# masādir

**Familia**: uṣūl sunita islámica · **Forma**: sectorial (4 cuadrantes + 4 chips de extensión)

## Metáfora principal

La cúpula se divide en **cuatro fuentes de prueba autoritativa** en la
*uṣūl al-fiqh* sunita: Corán, sunnah, ijmāʿ (consenso de los doctos) y
qiyās (razonamiento analógico). Cada una es un *tipo* diferente de
prueba — no un grado diferente de una prueba — por lo que el diseño es
sectorial (rebanadas categoriales), no concéntrico (profundidad
graduada). Debajo de la cúpula, cuatro fuentes complementarias se sitúan
como chips: *istiḥsān* (preferencia jurídica), *istiṣḥāb* (presunción de
continuidad), *maṣlaḥa mursalah* (interés público no restringido) y
*ʿurf* (práctica consuetudinaria).

Al igual que pramāṇa, los cuadrantes fueron rotados +π/4 (§θ-fix-1,
2026-05-18) para despejar el eje vertical de las etiquetas de estrato —
así que las posiciones geométricas son ahora E/S/O/N en lugar de los
originalmente documentados NE/SE/SO/NO.

## Alcance

**Cuándo usar esta tradición.** Al trabajar con contenido que es o
podría ser analizado como razonamiento sunita islámico legal-académico.
Útil para ver el balance de tipos-de-prueba a lo largo de una
derivación: ¿está tu argumento fuertemente fundamentado en el Corán?,
¿se apoya en el consenso?, ¿el qiyās hace la mayor parte del trabajo?
Los cuatro chips de extensión son recordatorios visuales de que la
uṣūl clásica reconoce más que las cuatro fuentes principales.

**Cuándo NO usar esta tradición.** Para contenido no islámico, las
etiquetas de los cuadrantes no tienen sentido. El marco también es
específicamente sunita — la uṣūl shīʿī duodecimana reemplaza el qiyās
por ʿaql (razón) y deliberadamente no está incluida por la regla de
linaje religioso (orientación v2.09). Contenido místico, filosófico y
literario encaja pobremente.

## Aplicabilidad

- Derivación de fiqh sunita, cursos de *uṣūl al-fiqh*, análisis de
  fatwa.
- Auditoría de balance entre fuentes en escritura legal-académica.
- Enseñanza de la estructura tipos-de-prueba de la jurisprudencia
  islámica clásica.

## Linaje

Uṣūl al-fiqh sunita clásica — la ciencia de las fuentes y métodos del
razonamiento legal islámico. El canon de cuatro fuentes es convencional
en los cuatro madhāhib sunitas (Hanafí, Malikí, Shafiʿí, Hanbalí), con
variación interna en cómo se pondera cada fuente. La representación
de Constellation sigue la línea del *Mustaṣfā* de al-Ghazālī.

## Crítica

La colocación del ijmāʿ en el cúmulo *ijtihādī* (derivado por
razonamiento) en lugar del cúmulo *naṣṣ* (transmitido textualmente) es
disputada por el kalām ashʿarī/māturīdī, que trata al ijmāʿ como
vinculante-transmitido. Constellation distribuye la lectura alineada al
Mustaṣfā; la lectura alternativa de kalām es un objetivo de pulido
v4.1. El canon de cuatro fuentes también aplana las diferencias
doctrinales entre los cuatro madhāhib — un registro variante específico
de Hanafí o Malikí podría añadirse más tarde.

La exclusión de la uṣūl shīʿī es una elección de diseño de producto
(regla de linaje religioso de orientación v2.09), no un juicio
académico.

## Citación

**Primario.** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*,
ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

**Moderno.** Franz Rosenthal, *Knowledge Triumphant: The Concept of
Knowledge in Medieval Islam* (Leiden: Brill, 1970); Wael B. Hallaq,
*A History of Islamic Legal Theories* (Cambridge: Cambridge University
Press, 1997).

## Frontmatter por nota

`masadir_source: quran | sunnah | ijma | qiyas`. Cuando aterrice la
extensión `LayoutCacheRow` del lado de Rust, este campo sobrescribe la
colocación por defecto (actualmente todas las notas → Corán). La
inclusión voluntaria por nota mediante `istihsan | istishab | maslaha |
urf` para las fuentes de los chips de extensión es un seguimiento.

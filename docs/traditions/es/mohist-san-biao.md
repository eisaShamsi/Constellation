---
id: mohist-san-biao
name: Mohist sān biǎo
family: chinese-pragmatist
shape: horizontal-bands
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# Mohist sān biǎo (三表)

**Familia**: pragmatista china · **Forma**: bandas horizontales (3 zonas)

## Metáfora principal

La cúpula se divide en **tres zonas horizontales apiladas de arriba
abajo**, una por cada estándar mohista para evaluar una doctrina:

- **本 běn (raíz)** — arriba. Precedente histórico de los reyes-sabios:
  ¿tiene la doctrina garantía en la tradición heredada?
- **原 yuán (origen)** — medio. Evidencia observacional directa: ¿la
  gente común ve y oye que es así?
- **用 yòng (uso)** — abajo. Beneficio social práctico: ¿adoptar esta
  doctrina mejora las vidas de la gente?

Una doctrina vale ser sostenida solo si pasa los tres tests — pero la
representación de Sight te permite ver las notas distribuidas a través
de los tres para captar qué tipo-de-garantía hace el mayor trabajo en
tu universo.

El eje horizontal no lleva codificación específica — los tres estándares
mohistas son *categoriales*, no ordinales, por lo que el posicionamiento
dentro de una banda se hace por jitter determinista por nota.

## Alcance

**Cuándo usar esta tradición.** Al trabajar con contenido donde el test
versa sobre *si una doctrina vale ser sostenida*, no sobre qué tipo de
garantía la sustenta. Útil para contenido de política, ética,
aplicado-empírico y de decisión-práctica donde el precedente histórico
/ la observación / el beneficio son los tres ejes de justificación.

**Cuándo NO usar esta tradición.** Cuando el contenido no tiene
dimensión doctrinal ni evaluativa. Contenido puramente descriptivo,
trabajo creativo y notas sobre experiencia subjetiva encajan
pobremente.

## Aplicabilidad

- Propuestas de política y sus justificaciones.
- Análisis de ética comparada (¿pasa esta regla los tres tests?).
- Ingeniería y ciencia aplicada donde el beneficio-para-la-gente es
  explícito.

## Linaje

Epistemología pragmatista china clásica. Mòzǐ 墨子 (~s. V a. C.) fundó
la escuela mohista, que se presentó como una alternativa crítica al
confucianismo. Los sān biǎo aparecen en el capítulo "Anti-Fatalismo"
como el test que los mohistas aplicaron a la doctrina fatalista
heredada — y concluyeron que fallaba los tres tests. La escuela floreció
brevemente y luego fue eclipsada por el ascenso confuciano y legalista;
sobrevive como un texto canónico recuperable estudiado hoy a través de
ediciones como el *Mòzǐ jiāngǔ*.

## Crítica

Los sān biǎo son a veces criticados como una forma temprana de
pragmatismo que confunde la garantía evidencial con la utilidad — el
criterio de "beneficio para la gente" en particular es difícil de
formalizar. Los estudiosos modernos también debaten si sān biǎo es una
teoría epistémica plenamente desarrollada o una herramienta
polémico-retórica desplegada en un argumento anti-fatalista específico.
Incluido en la línea base curada bajo la regla de linaje religioso a
pesar de su contexto teológico del Cielo, porque el núcleo metodológico
es secular.

## Citación

**Primario.** *Mòzǐ* 墨子, Book IX, "Fēi Mìng Shàng" 非命上
("Anti-Fatalism, Part I"). Critical edition: Sūn Yíràng, ed., *Mòzǐ
jiāngǔ* 墨子閒詁, 2 vols. (Beijing: Zhonghua Shuju, 1986). English:
Ian Johnston, trans., *The Mozi: A Complete Translation* (New York:
Columbia University Press, 2010).

**Moderno.** A. C. Graham, *Disputers of the Tao: Philosophical
Argument in Ancient China* (La Salle, IL: Open Court, 1989), ch. 1;
Chris Fraser, "Mohism," *Stanford Encyclopedia of Philosophy* (2020).

## Frontmatter por nota

`mohist_zone: ben | yuan | yong`. Actualmente ausente — las notas se
distribuyen mediante hash determinista por notePath en las tres zonas
de modo que la estructura visual quede poblada. Cuando aterrice la
extensión `LayoutCacheRow` del lado de Rust, este campo sobrescribe la
asignación del hash-bucket.

---
aliases:
  - Mente de Constellation
  - Constellation Mind
  - Mind
  - LLM local
  - Modelo de Lenguaje Grande local
  - Fanar
  - Chat de IA
  - IA personal
description: Constellation Mind es la capa de Modelo de Lenguaje Grande (LLM) local de Constellation — una IA con la que puedes conversar sobre tus propias notas, ejecutándose enteramente en tu dispositivo. La Fase 0b se lanzó el 2026-05-24 con el modelo Fanar-1-9B, prioridad árabe, instalable desde Ajustes → Mind. La superficie de chat aterriza en la Fase 1.
---

# Constellation Mind (عقل Constellation)

## ¿Qué es?

Constellation Mind es la capa de Modelo de Lenguaje Grande (LLM) local de Constellation — un asistente de IA que conoce tu Universo y puede conversar contigo sobre tus notas, **sin enviar ninguna de ellas a la nube**.

Tres cosas lo hacen distinto de cualquier otra herramienta de "IA para notas":

1. **Local primero.** El modelo se ejecuta en tu dispositivo. Tus notas nunca lo abandonan. No hay viaje de ida y vuelta a la nube — el chat es local y funciona sin conexión.
2. **Árabe primero.** El modelo predeterminado incluido es **Fanar-1-9B**, el modelo árabe-céntrico, consciente del contexto sunita del Qatar Computing Research Institute. Competencia nativa en MSA + dialecto del Golfo; el inglés es el segundo idioma, no el único.
3. **Vinculado a citas.** Cada afirmación factual que la IA hace sobre tus notas debe citar la nota fuente. Las citas alucinadas son detectadas por un validador posterior a la generación (Fase 1).

## Lo que se lanza hoy (Fase 0b — 2026-05-24)

- **Panel Ajustes → Mind** — lista los modelos instalables (actualmente solo Fanar 1.9B Q4_K_M, ~5 GiB), con un botón Instalar que descarga y verifica el modelo.
- **Instalación del modelo** — descarga en fragmentos desde un GitHub Release (sin nube de terceros), verificada por SHA-256 por fragmento y sobre el conjunto ensamblado.
- **Tiempo de ejecución de inferencia real** — `llama-cpp-2` (solo CPU en v1) carga el GGUF Q4_K_M y transmite tokens.
- **Aún no hay superficie de chat** — esa es la Fase 1 (el próximo hito). Hoy puedes instalar el modelo y verificarlo; la interfaz de conversación se lanza en MIG-048.

## Cómo instalar Fanar

1. Abre **Ajustes → Mind**.
2. Encuentra **Fanar 1.9B (Q4_K_M)** en el catálogo. La tarjeta muestra el tamaño (5,01 GiB), la licencia (Apache-2.0 con avisos defensivos de Gemma) y un botón "Establecer activo" o "Instalar".
3. Haz clic en **Instalar**. Una barra de progreso muestra la descarga + verificación SHA + ensamblaje en tres fases.
4. Cuando la insignia cambie a **Instalado** + **Activo**, el modelo está listo. Fanar vive en `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf` y está respaldado por mmap (sin copia a la RAM).

Eso es todo. Hasta que la Fase 1 lance la superficie de chat, el modelo instalado está en espera.

## Lo que viene en la Fase 1 (próximo hito)

- **Superficie de chat** — un panel de Constellation donde conversas con Fanar sobre tu Universo en árabe o inglés (con conciencia RTL por mensaje).
- **Herramientas de lectura** — Mind puede llamar a `search_notes`, `read_note`, `find_similar`, `list_recent` para fundamentar sus respuestas en tus notas reales.
- **Validador de citas** — cada afirmación cita una nota real; las referencias `note:UUID` fabricadas son rechazadas antes de llegar a ti.
- **Precalentamiento al iniciar la aplicación** — Mind se carga en segundo plano para que tu primer chat no pague los 10 segundos de carga en frío.
- **Historial de conversaciones** — guardado por Universo; promocionable a una Nota.

Consulta `docs/Constellation-Mind-Concept-Paper-v1.1.md` para la arquitectura completa y `docs/Constellation-Mind-Implementation-Plan-v1.0.md` para la hoja de ruta fase por fase.

## Lo que viene después

- **Fase 2 — Herramientas de escritura** (Mind propone ediciones / notas nuevas / enlaces bajo tu aprobación explícita).
- **Fase 2.5 — RoutedProvider + Jais** (un segundo modelo, Jais-2-8B de G42/MBZUAI, se une a Fanar como co-predeterminado; Mind enruta entre ellos según la solicitud).
- **Fase 3 — Auto-clasificación + enlazado inteligente** (Mind propone facetas y enlaces al guardar la nota).
- **Fase 4 — Herramientas de capacidad** (voz → nota, OCR → nota, traducción).
- **Fase 5 — Adhesión a la nube** (tu propia clave de Anthropic / OpenAI, con tope de costo por Universo y registro de egreso por turno).

## Privacidad y flujo de datos

- **HTTP saliente solo al instalar un modelo** — Constellation descarga archivos de modelo desde los [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases) de este repositorio. Sin telemetría. Sin inferencia en la nube (todavía — esa es la Fase 5, y solo con tu adhesión explícita).
- **En disco:** el GGUF del modelo + un registro `installed_models.json` que rastrea qué modelos tienes y cuál está activo.
- **En tiempo de ejecución:** el archivo del modelo cargado está mapeado en memoria; tus indicaciones y respuestas viven solo en la RAM.

## Licencias

Cada modelo lleva su propia LICENSE.txt junto a él en el GitHub Release. Para Fanar:

- **Apache License 2.0** (la licencia declarada por QCRI en el repositorio Fanar-1-9B-Instruct).
- **Términos de uso de Gemma** — Fanar es un preentrenamiento continuado de `google/gemma-2-9b`; Constellation envía los avisos de Gemma defensivamente aunque QCRI reetiquete el resultado solo como Apache-2.0.
- **Cita de Fanar** (Fanar Team 2025, arXiv:2501.13944).
- **Aviso de redistribución de Constellation** — el GGUF en el GitHub Release de Constellation es una cuantización de los safetensors upstream de QCRI, producidos por `.github/workflows/model-pipeline.yml` y distribuidos bajo Apache-2.0 con la LICENSE original viajando.

La LICENSE.txt completa vive junto a cada modelo en su release: <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## Solución de problemas

**Insignia "Aún no listo" en lugar del botón Instalar.** El catálogo incluido tiene un SHA-256 marcador de posición para ese modelo. Esto no debería ocurrir en una instalación normal de Constellation; si lo ves, el catálogo no se ha actualizado para esa versión del modelo. Abre un issue.

**La instalación se cuelga en "Descargando parte X/Y".** Problema de red. Cancela desde Ajustes → Mind, vuelve a activar Instalar — los fragmentos parciales se limpian automáticamente.

**La instalación tiene éxito, el SHA-256 del archivo no coincide.** Un bit-flip en la descarga. La reinstalación obtendrá una copia fresca.

**Falta la superficie de chat.** La Fase 1 (MIG-048) aún no se ha lanzado. El modelo se puede instalar y verificar hoy; la interfaz de conversación llegará en el próximo lanzamiento.

---

*Los subtemas se unirán a esta carpeta cuando se lance la Fase 1: recorrido de la UI de chat, comportamiento de toque de las fichas de cita, selector multi-modelo, renderizado de chats largos en la segunda pantalla.*

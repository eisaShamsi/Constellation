---
aliases:
  - Constellation Geist
  - Constellation Mind
  - Mind
  - Lokales LLM
  - Lokales Großes Sprachmodell
  - Fanar
  - KI-Chat
  - Persönliche KI
description: Constellation Mind ist die lokale Schicht des Großen Sprachmodells (LLM) in Constellation — eine KI, mit der du über deine eigenen Notizen chatten kannst und die vollständig auf deinem Gerät läuft. Phase 0b wurde am 2026-05-24 ausgeliefert mit dem arabisch-fokussierten Modell Fanar-1-9B, das über Einstellungen → Mind installierbar ist. Die Chat-Oberfläche kommt in Phase 1.
---

# Constellation Mind (عقل Constellation)

## Was ist es?

Constellation Mind ist die lokale Schicht des Großen Sprachmodells (LLM) von Constellation — ein KI-Assistent, der dein Universum kennt und mit dir über deine Notizen sprechen kann, **ohne irgendetwas davon in die Cloud zu senden**.

Drei Dinge unterscheiden ihn von jedem anderen „KI-für-Notizen"-Werkzeug:

1. **Lokal-zuerst.** Das Modell läuft auf deinem Gerät. Deine Notizen verlassen es nie. Es gibt keinen Cloud-Roundtrip — der Chat ist lokal und offline-fähig.
2. **Arabisch-zuerst.** Das mitgelieferte Standardmodell ist **Fanar-1-9B**, das arabisch-zentrische, sunni-bewusste Modell des Qatar Computing Research Institute. Native MSA- und Golfdialekt-Kompetenz; Englisch ist die zweite Sprache, nicht die einzige.
3. **Zitatgebunden.** Jede sachliche Aussage, die die KI über deine Notizen macht, muss die Quellnotiz zitieren. Halluzinierte Zitate werden von einem Validator nach der Generierung abgefangen (Phase 1).

## Was heute ausgeliefert wird (Phase 0b — 2026-05-24)

- **Einstellungen → Mind-Panel** — listet installierbare Modelle auf (derzeit nur Fanar 1.9B Q4_K_M, ~5 GiB), mit einer Installations-Schaltfläche, die das Modell herunterlädt und verifiziert.
- **Modellinstallation** — Chunked-Download von einem GitHub-Release (keine Drittanbieter-Cloud), SHA-256-verifiziert pro Chunk und am zusammengesetzten Ganzen.
- **Echte Inferenz-Laufzeit** — `llama-cpp-2` (nur CPU in v1) lädt das Q4_K_M-GGUF und streamt Tokens.
- **Noch keine Chat-Oberfläche** — das ist Phase 1 (der nächste Meilenstein). Heute kannst du das Modell installieren und verifizieren; die Konversationsoberfläche kommt mit MIG-048.

## So installierst du Fanar

1. Öffne **Einstellungen → Mind**.
2. Finde **Fanar 1.9B (Q4_K_M)** im Katalog. Die Karte zeigt die Größe (5,01 GiB), die Lizenz (Apache-2.0 mit defensiven Gemma-Hinweisen) und eine Schaltfläche „Als aktiv setzen" oder „Installieren".
3. Klicke auf **Installieren**. Ein Fortschrittsbalken zeigt Download + SHA-Verifizierung + Zusammensetzung in drei Phasen.
4. Wenn das Abzeichen auf **Installiert** + **Aktiv** umschaltet, ist das Modell bereit. Fanar lebt unter `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf` und wird per mmap geladen (keine Kopie in den RAM).

Das war's. Bis Phase 1 die Chat-Oberfläche ausliefert, befindet sich das installierte Modell im Standby.

## Was in Phase 1 kommt (nächster Meilenstein)

- **Chat-Oberfläche** — ein Constellation-Panel, in dem du mit Fanar auf Arabisch oder Englisch über dein Universum sprichst (RTL-fähig pro Nachricht).
- **Lese-Tools** — Mind kann `search_notes`, `read_note`, `find_similar`, `list_recent` aufrufen, um seine Antworten in deinen tatsächlichen Notizen zu verankern.
- **Zitat-Validator** — jede Aussage zitiert eine echte Notiz; fabrizierte `note:UUID`-Referenzen werden zurückgewiesen, bevor sie dich erreichen.
- **Vorwärmen beim App-Start** — Mind wird im Hintergrund geladen, damit dein erster Chat nicht die 10-Sekunden-Kaltladezeit zahlt.
- **Konversationsverlauf** — pro Universum gespeichert; kann zu einer Notiz hochgestuft werden.

Siehe `docs/Constellation-Mind-Concept-Paper-v1.1.md` für die vollständige Architektur und `docs/Constellation-Mind-Implementation-Plan-v1.0.md` für die phasenweise Roadmap.

## Was später kommt

- **Phase 2 — Schreib-Tools** (Mind schlägt Bearbeitungen / neue Notizen / Links unter deiner ausdrücklichen Genehmigung vor).
- **Phase 2.5 — RoutedProvider + Jais** (ein zweites Modell, Jais-2-8B von G42/MBZUAI, schließt sich Fanar als Co-Standard an; Mind routet zwischen ihnen basierend auf der Anfrage).
- **Phase 3 — Auto-Klassifizierung + Smart-Linking** (Mind schlägt Facetten und Links beim Speichern einer Notiz vor).
- **Phase 4 — Capability-Tools** (Stimme → Notiz, OCR → Notiz, Übersetzung).
- **Phase 5 — Cloud-Opt-in** (dein eigener Anthropic- / OpenAI-Schlüssel, mit Kostendeckel pro Universum und Egress-Log pro Runde).

## Privatsphäre & Datenfluss

- **Ausgehender HTTP nur bei der Installation eines Modells** — Constellation lädt Modelldateien aus den [`models/*` GitHub-Releases](https://github.com/eisaShamsi/Constellation/releases) dieses Repos herunter. Keine Telemetrie. Keine Cloud-Inferenz (noch — das ist Phase 5, und nur mit deinem ausdrücklichen Opt-in).
- **Auf der Festplatte:** das Modell-GGUF + eine `installed_models.json`-Registry, die verfolgt, welche Modelle du hast und welches aktiv ist.
- **Zur Laufzeit:** die geladene Modelldatei wird per Memory-Mapping eingebunden; deine Prompts und Antworten existieren nur im RAM.

## Lizenzen

Jedes Modell trägt seine eigene LICENSE.txt neben sich im GitHub-Release. Für Fanar:

- **Apache License 2.0** (die von QCRI deklarierte Lizenz im Fanar-1-9B-Instruct-Repo).
- **Gemma Terms of Use** — Fanar ist ein fortgesetztes Pretraining von `google/gemma-2-9b`; Constellation liefert die Gemma-Hinweise defensiv aus, auch wenn QCRI das Ergebnis allein als Apache-2.0 neu deklariert.
- **Fanar-Zitat** (Fanar Team 2025, arXiv:2501.13944).
- **Constellation-Weitergabe-Hinweis** — das GGUF im Constellation-GitHub-Release ist eine Quantisierung der Upstream-Safetensors von QCRI, produziert von `.github/workflows/model-pipeline.yml` und verteilt unter Apache-2.0 mit mitreisender Original-LICENSE.

Die vollständige LICENSE.txt lebt neben jedem Modell in seinem Release: <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## Fehlerbehebung

**„Noch nicht bereit"-Abzeichen anstelle der Installations-Schaltfläche.** Der mitgelieferte Katalog hat einen Platzhalter-SHA-256 für dieses Modell. Dies sollte bei einer normalen Constellation-Installation nicht vorkommen; wenn du es siehst, wurde der Katalog für diese Modellversion nicht aktualisiert. Öffne ein Issue.

**Installation hängt bei „Lade Teil X/Y herunter".** Netzwerkproblem. Brich aus Einstellungen → Mind ab, löse die Installation erneut aus — die partiellen Chunks werden automatisch aufgeräumt.

**Installation erfolgreich, Datei-SHA-256 stimmt nicht überein.** Ein Bit-Flip beim Download. Eine Neuinstallation holt eine frische Datei.

**Chat-Oberfläche fehlt.** Phase 1 (MIG-048) ist noch nicht ausgeliefert. Das Modell kann heute installiert und verifiziert werden; die Konversations-UI kommt im nächsten Release.

---

*Unterthemen werden diesem Ordner beitreten, wenn Phase 1 ausgeliefert wird: Walkthrough der Chat-UI, Tipp-Verhalten der Zitat-Chips, Multi-Modell-Auswahl, Rendering langer Chats auf dem zweiten Bildschirm.*

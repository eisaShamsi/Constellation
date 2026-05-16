---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: de
source: docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md
aliases:
  - Constellation Sight
  - Koordinierte Ansichten
  - Ankerdom
  - Mini-Dome
description: Constellation Sight visualisiert Ihr gesamtes Wissensuniversum als geschichteten Ankerdom mit vier koordinierten Mini-Domen, die dieselben Notizen über verschiedene Kanäle (Konfidenz, Stadium, Akte, Provenienz) neu kodieren. Schweben Sie über einen Stern, um ihn überall zu sehen; klicken Sie Seitenleisten-Chips oder Shift+Klick auf Sterne zum Filtern; befördern Sie einen Mini-Dom zur Vollgrößen-Inspektion mit Zoom.
---

# Constellation Sight

## Was ist es?

**Constellation Sight** ist das **diagnostische Instrument** für Ihr Wissensuniversum. Ein zentraler **Ankerdom** zeigt jede Notiz positioniert nach **Stratum** (Tiefe des Denkens) und **Zeit** (wann geschrieben), mit vier **Mini-Domen** daneben, die das gleiche Universum über verschiedene Kanäle neu kodieren: **Konfidenz**, **Stadium**, **Akte**, **Provenienz**.

Es beantwortet eine Frage mit fünf komplementären Linsen: **„Wie ist mein epistemischer Inhalt geformt und organisiert?"**

Schweben Sie über einen Stern in einem beliebigen Dom, und dieselbe Notiz leuchtet in allen fünf Oberflächen auf — goldener Ring um den Stern, goldener Tönung auf den entsprechenden Chips in der Seitenleiste. Klicken Sie auf einen Chip in der Seitenleiste und alle fünf Ansichten verengen sich. Shift+Klick auf einen Stern im Stadium-Mini filtert das Universum sofort auf dieses Lebenszyklus-Stadium. Klicken Sie auf leeren Raum in einem Mini-Dom und er "befördert" zum primären Slot in voller Größe, während der vorherige Primäre in den freigewordenen Mini-Slot absteigt.

## Warum ist es wichtig?

Die meisten Notiz-Apps zeigen Ihnen, was Sie geschrieben haben. Constellation Sight zeigt Ihnen die **Form** dessen, was Sie wissen.

- Wo ist Ihr Denken **konzentriert**? (Dichtegradient im Anker)
- Was ist noch **im Frühstadium** vs **stabile Grundlage**? (Farbgradient des Stadium-Mini)
- Welche Notizen sind **tragend** vs **isoliert**? (Größenkodierung des Akte-Mini)
- Woher kam jede Idee — eigenes Denken, Lesen, Hören, Tradition? (Sektor-Layout des Provenienz-Mini)
- Wie **zuversichtlich** sind Sie in Ihren Schlussfolgerungen? (Opazitätsgradient des Konfidenz-Mini)

Eine Notiz im Zentrum des Ankers (hohe Konnektivität → tragend) aber in Cyan im Stadium-Mini (`spark` — gerade entzündet) sagt Ihnen etwas Diagnostisches: eine tragende Idee, die noch nicht gereift ist.

## Wie öffnen

1. Klicken Sie auf das **Augensymbol** im Dock am linken Rand von Constellation.
2. Der Ankerdom rendert in 2–5 Sekunden für die meisten Universen.
3. Zum Schließen: klicken Sie auf **(×)** oben rechts, oder drücken Sie **Esc**.

## Was Sie sehen

### Die Kopfleiste

Oben in der Sight-Ansicht, von links nach rechts:
- **„Constellation Sight"** — der Titel.
- **„v6.1 — Coordinated Views (Phase 2)"** — Versions-Untertitel.
- **„X / Y notes"-Abzeichen** in Gold — nur sichtbar wenn ein Filter aktiv ist.
- **„EXTENDED"-Abzeichen** in goldenen Kapitälchen — nur sichtbar wenn Erweiterte Ansicht aktiv ist.
- **„Reset View"-Schaltfläche** — nur sichtbar wenn das Layout geändert wurde.
- **(×)** Schließen-Schaltfläche — immer vorhanden.

### Der Ankerdom (Primärer Slot)

Der große cremefarbene Dom auf dunklem Hintergrund in der Mitte:
- **Stratum-Ringe** — 5 konzentrische Kreise. Innerster = Ihre grundlegendsten Notizen; äußerer Rand = Ihre neuesten Funken.
- **Kalender-Rand** — 12 Monatsetiketten außen.
- **Stratum-Beschriftungen** — kursiver Text oben.
- **Sterne** — jede Notiz als kleiner cremefarbener Punkt, positioniert nach Stratum × Zeit.
- **Verbindungslinien** — typisierte Link-Kanten zwischen Notizen, niedrige Opazität unter den Sternen.
- **Hover-Ring** — goldener Kreis um jeden Stern, über den der Cursor schwebt.

### Die Vier Mini-Dome

Rechte Seite, 2×2-Raster. Standardmäßig versteckt; angezeigt mit **Strg+D** (nur Sitzung) oder **Strg+Umschalt+D** (persistent — siehe Erweiterte Ansicht).

Jeder Mini rendert dasselbe Universum durch eine Kodierung:

1. **CONFIDENCE — Opazität.** Konfidentere Notizen heller; tentative verblassen.
2. **STAGE — Farbe (volle Scheibe).** Kategorische Farbe nach Lebenszyklus-Stadium:
   - **Cyan** = `spark` (gerade entzündete Idee)
   - **Orange** = `birth` (Form annehmend)
   - **Violett** = `growth` (aktiv in Bewegung)
   - **Grün** = `maturity` (vollständig geformt)
   - **Gelb** = `renewal` (kürzlich erneut besucht)
   - **Grau** = `dormancy` / `archival` (inaktiv / geschlossen)
3. **ACTS — Größe (oberes Dezil).** Top 10% nach Link-Anzahl = größere Punkte; Rest klein.
4. **PROVENANCE — 5 Sektoren.** Sterne umpositioniert in 5 Winkelsektoren: **Self / Read / Heard / Reasoned / Tradition**.

### Der Degradierte Anker (Wenn Mini Befördert)

Wenn Sie einen Mini in den primären Slot befördern, steigt der Anker in den freigewordenen Mini-Slot ab. Dort wird er als **neutrale cremefarbene Punkte** mit dem Titel **„UNIVERSE — primary view"** gerendert.

### Die Facetten-Seitenleiste (Linker Rand)

Einklappbares Panel mit **6 Filter-Facettengruppen**, jede mit Kategorien und Live-Zählungen:

- **Folder** — Ordnerhierarchie
- **Library** — Bibliotheksnamen
- **Stratum** — Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing
- **Confidence** — Hypothesis / Evidence / Established / Contested
- **Stage** — Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival
- **Provenance** — Self / Read / Heard / Reasoned / Tradition

Klicken Sie auf **▶** am Rand zum Erweitern. Klicken Sie auf einen Chip zum Umschalten des Filters.

## Interaktion

| Geste | Effekt |
|---|---|
| **Hover über Stern** | Goldener Ring auf demselben Stern in allen 5 Oberflächen + entsprechende Chips golden getönt. |
| **Einfacher Klick auf Stern** | Öffnet die Notiz im Editor. **„Return to Sight"**-Schaltfläche erscheint. |
| **Shift+Klick auf Stern** im Stage/Confidence/Provenance-Mini | Schaltet Filter auf die Kategorie dieses Sterns. |
| **Shift+Klick auf Stern** in Acts oder Anker | Kein Effekt. |
| **Klick auf leeren Bereich eines Mini** | Dieser Mini befördert zum primären Slot. |
| **Mausrad-Zoom (primär)** | Zoom zum Cursor. Bereich: 0,5× bis 24×. |
| **Klick+Ziehen auf leeren Bereich** | Pan der Ansicht. |
| **Strg+0 / Cmd+0** | Setzt Zoom + Pan zurück. |
| **Strg+D / Cmd+D** | Schaltet Mini-Dome-Sichtbarkeit — **nur Sitzung**. |
| **Strg+Umschalt+D / Cmd+Umschalt+D** | Schaltet **Erweiterte Ansicht** — persistent. |
| **Klick auf Seitenleisten-Chip** | Schaltet Facetten-Kategorie im Filter-Set. |
| **Reset View Schaltfläche** | Zurück zum Anker primär bei Zoom 1.0. |
| **Esc** | Schließt Sight. |

## Geistermodus — Mehrfachauswahl aus dem Dom

Wenn ein Filter aktiv ist, bleiben nicht-übereinstimmende Sterne sichtbar aber bei **niedriger Opazität (15%)** statt zu verschwinden. Das bedeutet:

- Sie können noch SEHEN, wo die nicht-übereinstimmenden Sterne sind.
- Sie können über sie schweben (goldener Ring erscheint).
- Sie können **Shift+Klick um ihre Kategorie ZUM Filter hinzuzufügen**.

## Dichtemodus

Wenn die Anzahl sichtbarer (übereinstimmender) Sterne den Dichteschwellenwert überschreitet (Standard **5.000**), wechseln die Mini-Dome zu einem **perzeptuellen Dichte-Rendering**.

## Erweiterte Ansicht

Drücken von **Strg+Umschalt+D** (oder **Cmd+Umschalt+D** auf Mac) schaltet die „Erweiterte Ansicht" — wenn aktiv, sind die Mini-Dome bei jeder Öffnung von Sight standardmäßig sichtbar. Der Zustand bleibt über Sight-Schließungen, App-Neustarts und Reboots erhalten.

## Wann Sight am Nützlichsten Ist

- **Wissensform prüfen** — öffnen Sie Sight nach einer Schreibsitzung.
- **Blinde Flecken finden** — Sektoren des Doms mit wenigen Notizen könnten Bereiche zur Erkundung sein.
- **Tragende Schwäche erkennen** — zentral positionierte Notiz in Früh-Stadium-Farbe sagt Ihnen, dass Sie sich auf etwas Unreifes verlassen.
- **Filtern und inspizieren** — Shift+Klick verengt das Universum; befördern Sie einen Mini zur Vollgrößen-Studie.
- **Epistemische Provenienz verfolgen** — befördern Sie Provenance um zu sehen, wie Ihr Wissen entstanden ist.

## Verwandte Oberflächen

- **Constellation Nervous System (CNS)** — komplementäre Visualisierung (Neuron-Symbol neben dem Sight-Augensymbol).
- **Constellation Map** — Sonnenstrahl-Visualisierung.
- **Sky View** — graphbasierte Link-Visualisierung.
- **Index-Panel** — Begriffsbrowser.

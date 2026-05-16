---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: de
source: docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md
aliases:
  - Constellation Nervous System
  - CNS
  - Constellation Nervensystem
description: Constellation Nervous System (CNS) ist die Verbindungs-Traversierungs-Ansicht Ihres Universums. Es analysiert den Link-Graph zwischen Ihren Notizen und bringt Universumsgesundheits-Metriken, Communities, Top-Brücken zwischen Clustern und „Blinde Flecken"-strukturelle Lücken zum Vorschein. CNS ist die komplementäre Ansicht zu Constellation Sight — wenn Sight die sensorische Form Ihres Universums ist, ist CNS seine neuralen Verbindungen.
---

# Constellation Nervous System (CNS)

## Was ist es?

**Constellation Nervous System** ist die **Verbindungs-Traversierungs**-Ansicht Ihres Universums. Während Constellation Sight die *Form* Ihrer Notizen zeigt (Stratum × Zeit × Kanal-Kodierung), zeigt CNS die *Verkabelung* — den typisierten Link-Graph, der sie verbindet und die strukturellen Muster, die in diesem Graph verborgen sind.

Es beantwortet: **„Wie sind die Ideen in meinem Universum verbunden, und wo sind die Lücken?"**

Die Ansicht ist um vier analytische Oberflächen aufgebaut:
- **Universe Health** — Gesamt- und Pro-Metrik-Scores für wie verbunden, ausgewogen und modular Ihr Wissen ist.
- **Communities** — Gruppen dicht vernetzter Notizen („ideologische Cluster").
- **Top Bridges** — die wenigen Notizen, die sonst separate Communities verbinden („tragende Verbinder").
- **Blind Spots** — strukturelle Lücken, wo Sie Verbindungen erwarten würden, aber noch keine haben.

Der Name „Nervous System" ist anatomisch: Nerven sind Verbindungspfade, die Signale zwischen entfernten Teilen eines Organismus tragen. CNS behandelt Ihren typisierten Link-Graph auf dieselbe Weise.

## Warum ist es wichtig?

Die meisten Notiz-Apps behandeln Links als Rohrleitungen (von hier nach dort springen). Constellation behandelt sie als **Wissensarchitektur**:

- Eine Notiz mit vielen eingehenden Links ist **tragend** — viele Ideen hängen davon ab.
- Eine Notiz, die zwei Communities überbrückt, ist ein **Synthesepunkt**.
- Eine Community mit schwacher interner Verlinkung ist **fragil**.
- Ein „Blind Spot" ist ein Ort, wo die Struktur eine Verbindung haben SOLLTE, aber nicht hat — eine zu erforschende Hypothese.

## Wie öffnen

1. Klicken Sie auf das **Neuron-Symbol** (kleine verzweigte Nervenzellenform — Zellkörper in der Mitte mit drei Dendriten und synaptischen Terminalen) im Dock am linken Rand.
2. CNS öffnet sich in einem Vollfenster-Overlay im Schwerkraftbrunnen-Stil — kraftgerichteter Graph, wo jede Notiz ein Knoten und jeder typisierte Link eine Kante ist.
3. Zum Schließen: klicken Sie auf **(×)** oben, oder drücken Sie **Esc**.

## Was Sie sehen

### Die Universe Health Karte

Zusammenfassungs-Panel, das die Gesamt-Konnektivitäts-Gesundheit Ihres Universums zeigt, mit goldener Rondelle einer zusammengesetzten Punktzahl (z.B. **91 / 100**) und vier Metriken:

- **Modularity** — wie sauber Ihre Notizen in eindeutige Communities clustern.
- **Dominance** — ob eine Community das Universum dominiert.
- **Entropy** — Vielfalt der Community-Größen.
- **Connectivity** — durchschnittliche Links pro Notiz.

Jede Metrik hat eine farbige Status-Pille: **HEALTHY** (grün) / **CAUTION** (gelb) / **IMBALANCED** (rot).

### Der Schwerkraftbrunnen

Die Hauptvisualisierung: Notizen schweben als Knoten, Links ziehen sie zusammen, Abstoßung drückt sie auseinander. Communities organisieren sich selbst in Cluster.

- **Knotengröße** = Link-Anzahl.
- **Knotenfarbe** = Community-Zugehörigkeit.
- **Kante** = typisierter Link zwischen zwei Notizen.

### Top Bridges

Liste der Notizen, die die meisten unterschiedlichen Communities verbinden — das sind Ihre Synthesepunkte.

### Communities

Liste der erkannten Notizencluster.

### Blind Spots (Strukturelle Lücken)

Vorgeschlagene fehlende Verbindungen — Paare von Notizen, die der Graph-Algorithmus für verlinkt halten würde.

## Interaktion

CNS verwendet ein **Einzelklick-Vorschau / Doppelklick-Öffnen**-Muster (anders als Sights Einzelklick-öffnet):

| Geste | Effekt |
|---|---|
| **Einzelklick auf Knoten** | Wählt ihn aus. Rechtes Seitenpanel gleitet ein mit Titel, Community, Zentralitätsrang, eingehenden/ausgehenden Links. Notiz wird NICHT geöffnet. |
| **Doppelklick auf Knoten** | Öffnet die Notiz im Editor. **„Return to CNS"**-Schaltfläche erscheint. |
| **Hover über Knoten** | Tooltip mit Titel. |
| **Klick auf leeren Bereich** | Räumt die Auswahl. |
| **Mausrad** | Zoom in / out. |
| **Klick + Ziehen** | Pan. |
| **Klick auf Community in der Liste** | Hebt Notizen der Community im Schwerkraftbrunnen hervor. |
| **Klick auf Top-Bridge-Eintrag** | Fokussiert auf die Brückennotiz. |
| **Esc** | Schließt CNS. |

Die Einzelklick-Vorschau ist absichtlich: Sie können viele Notiz-Details (und ihre Verbindungen) scannen, ohne sich zu verpflichten, jede einzelne im Editor zu öffnen.

## Wann CNS am Nützlichsten Ist

- **Verbindungsdichte prüfen** — Universe Health gibt eine Auf-einen-Blick-Lesung.
- **Synthesepunkte finden** — Top Bridges zeigt die Notizen, die die meiste Architekturarbeit leisten.
- **Communities entdecken, von denen Sie nicht wussten** — Cluster, die aus dem Graph emergieren.
- **Blind Spots flicken** — wenn der Graph vorschlägt, zwei Notizen SOLLTEN verlinkt sein, aber nicht sind.
- **Reorganisation planen** — Communities entsprechen natürlich der Ordnerstruktur.

## CNS vs Sight — Wann was zu verwenden

- **Sight** = „Wie ist mein Universum GEFORMT?" Räumliche / kategorische Analyse.
- **CNS** = „Wie ist mein Universum VERBUNDEN?" Netzwerk / topologische Analyse.

Sie sind komplementär: Sight liest die Oberfläche; CNS liest die Verkabelung darunter.

## Verwandte Oberflächen

- **Constellation Sight** — die Schwester-Visualisierung (Augensymbol im Dock).
- **Sky View** — auch Graph-Ansicht, aber anders gebaut.
- **Backlinks / Outgoing Links Panels** — Pro-Notiz-Verbindungslisten.

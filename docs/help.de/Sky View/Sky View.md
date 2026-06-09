---
aliases:
  - Sky View
  - Himmelsansicht
  - GraphMind
  - Sternenansicht
  - Link-Sternansicht
  - Netzwerkansicht
  - Notizverbindungen
  - 3D-Graph
description: Visualisiere und erkunde die Verbindungen zwischen deinen Notizen mit der interaktiven Sky View von Constellation, angetrieben von der GraphMind-Engine.
---

# Sky View

Die Sky View stellt deine Notizen als interaktives Netzwerk aus Knoten und Verknüpfungen dar, angetrieben von der **GraphMind**-Engine (Pixi.js WebGL). Jeder Knoten ist eine Notiz, und jede Linie steht für eine `[[Wikilink]]`-Verbindung zwischen Notizen. Je mehr Verbindungen eine Notiz hat, desto größer erscheint ihr Knoten.

## Sky View öffnen

| Methode | Aktion |
|--------|--------|
| **Mission Control** | `Ctrl+P` drücken, „Sternenansicht“ eingeben |
| **Tastatur** | `Ctrl+G` |

Drücke `Escape`, um die Sky View zu schließen.

> [!note]
> Das Sky-View-Symbol in der linken Leiste wurde entfernt. Die Sky View ist jetzt über das Tastaturkürzel oder Mission Control erreichbar. Der Sky-View-Modus (OrgChart) ist als Tab in der Seitenleiste „Notizverwaltung“ verfügbar.

---

## Mit dem Graphen interagieren

### Grundlegende Interaktionen

| Eingabe | Verhalten |
|-------|----------|
| **Verschieben** | Auf eine leere Fläche klicken und ziehen |
| **Zoomen** | Scrollrad (2D) oder `Ctrl+Scroll` (3D) |
| **Knoten ziehen** | Einen beliebigen Knoten anklicken und ziehen, um ihn neu zu positionieren |
| **Überfahren** | Zeigt den Notiznamen in der Statusleiste an und hebt verbundene Knoten und Kanten hervor |
| **Auf einen Knoten klicken** | Öffnet diese Notiz im Editor |
| **Doppelklick auf einen Knoten** | Zoomt heran und zentriert auf diesen Knoten |
| **Rechtsklick auf einen Knoten** | Öffnet das Kontextmenü |

### Kontextmenü

Klicke mit der rechten Maustaste auf einen beliebigen Knoten, um darauf zuzugreifen:

| Aktion | Beschreibung |
|--------|-------------|
| **Öffnen** | Öffnet die Notiz im Editor |
| **Fokussieren** | Wechselt in den Fokusmodus, zentriert auf diesen Knoten |
| **Anheften** | Fixiert den Knoten an seiner aktuellen Position. Erneut klicken, um ihn loszulösen. |
| **Ausblenden** | Blendet den Knoten aus dem Graphen aus. Verwende „Alle anzeigen“ in der Werkzeugleiste, um ausgeblendete Knoten wieder einzublenden. |

---

## 3D-Navigation

Die Sky View unterstützt vollständige 3D-Navigation — fliege durch deine Notizen, als würdest du durch Sterne navigieren.

### In den 3D-Modus wechseln

**Mit der mittleren Maustaste klicken und ziehen** (oder **Alt+Klick und ziehen**), um den Graphen im 3D-Raum zu drehen. Sobald gedreht wurde, werden die 3D-Navigationssteuerungen aktiv.

### 3D-Steuerung

| Eingabe | Aktion |
|-------|--------|
| **Mittlere Maustaste ziehen** | Um die X- und Y-Achse drehen |
| **Shift+Mittlere Maustaste ziehen** | Um die Z-Achse drehen |
| **W / Pfeil nach oben** | Vorwärts fliegen (in den Bildschirm hinein) |
| **S / Pfeil nach unten** | Rückwärts fliegen |
| **A / Pfeil nach links** | Nach links seitwärts ziehen |
| **D / Pfeil nach rechts** | Nach rechts seitwärts ziehen |
| **Q** | Nach unten bewegen |
| **E** | Nach oben bewegen |
| **Ctrl+Scroll** | Zoomen (Sichtfeld ändern) |
| **Normales Scrollen** | Vorwärts/rückwärts entlang der Kamerarichtung fliegen |
| **0** | Drehung auf die flache 2D-Ansicht zurücksetzen |
| **Zurücksetzen-Schaltfläche** (↺-Symbol) | Wie das Drücken von `0` |

### XYZ-Achsen-Gizmo

Im 3D-Modus erscheint eine farbcodierte Achsenhilfe in der unteren linken Ecke:

| Achse | Farbe | Richtung |
|------|-------|-----------|
| **X** | Rot | Links–Rechts |
| **Y** | Grün | Oben–Unten |
| **Z** | Blau | Vorne–Hinten (Tiefe) |

Das Gizmo dreht sich mit der Kamera, sodass du deine Ausrichtung immer kennst.

### Überfahren und Klicken in 3D

Du kannst Knoten überfahren und anklicken, während du in 3D navigierst. Der Notizname erscheint in der Statusleiste, und ein Klick öffnet die Notiz — genau wie im 2D-Modus.

---

## Layout-Modi

Die Sky View bietet drei Layout-Algorithmen. Wechsle zwischen ihnen mit `Ctrl+L` oder über die Layout-Schaltfläche in der Werkzeugleiste.

| Modus | Beschreibung | Am besten für |
|------|-------------|----------|
| **Organisch** | Kräftebasiertes Layout. Cluster entstehen auf natürliche Weise aus der Verbindungsdichte. | Allgemeines Erkunden — der Standardmodus. |
| **Hierarchisch** | Gerichteter azyklischer Graph (DAG) von oben nach unten. | Strukturierte Bibliotheken mit Eltern-Kind-Beziehungen. |
| **Zeitlich** | Knoten entlang einer waagerechten Zeitachse nach Erstellungsdatum angeordnet. | Erkennen, wann Notizen erstellt wurden und wie die Bibliothek gewachsen ist. |

Ein Moduswechsel löst einen sanften animierten Übergang aus, der deine räumliche Ausrichtung bewahrt.

> [!tip]
> Der hierarchische Modus ist besonders nützlich für Notizen, die einer baumartigen Struktur folgen (z. B. Inhaltskarten, die auf Unterthemen verweisen). Der zeitliche Modus enthüllt deine intellektuelle Zeitleiste — wann Cluster verwandter Notizen entstanden sind.

---

## Fokusmodus

Der Fokusmodus zeigt nur eine bestimmte Notiz und ihre Nachbarschaft. Es ist ein dynamischer, interaktiver lokaler Graph.

### In den Fokusmodus wechseln

- **Rechtsklick auf einen Knoten** → **Fokussieren**
- **Leertaste drücken**, um den Fokusmodus für die aktuell aktive Notiz umzuschalten

### Fokus-Steuerung

Im Fokusmodus erscheint oben eine Steuerleiste:

| Steuerelement | Beschreibung |
|---------|-------------|
| **Tiefen-Schieberegler** (1–5) | Wie viele Verbindungssprünge angezeigt werden. 1 = nur direkte Verknüpfungen, 5 = fünf Ebenen tief. |
| **Richtungsfilter** (↔ / ← / →) | Alle Verknüpfungen, nur eingehende oder nur ausgehende anzeigen. |
| **Beenden-Schaltfläche** (×) | Zurück zur vollständigen Sky View |

### Navigations-Brotkrumenpfad

Während du im Fokusmodus durch Knoten klickst, erscheint oben ein Brotkrumenpfad, der deinen Navigationsweg anzeigt. Klicke auf eine beliebige Brotkrume, um zum lokalen Graphen jener Notiz zurückzuspringen.

> [!tip]
> Kombiniere den Fokusmodus mit dem Tiefen-Schieberegler, um die Nachbarschaft einer Notiz schrittweise zu erkunden. Beginne bei Tiefe 1, um direkte Verbindungen zu sehen, und erhöhe sie dann, um Beziehungen zweiten und dritten Grades zu entdecken.

---

## Suchen-und-Hervorheben

Drücke `Ctrl+F`, um die Suchleiste zu öffnen. Gib eine Suchanfrage ein, um passende Notizen hervorzuheben.

Anders als ein Filter **dimmt** Suchen-und-Hervorheben nicht passende Knoten ab, ohne sie zu entfernen. Du behältst die gesamte Graphstruktur und den räumlichen Kontext, während die passenden Knoten hervorgehoben werden.

> [!tip]
> Die Suche funktioniert sowohl im vollständigen Graphen als auch im Fokusmodus. Du kannst auch im 3D-Modus suchen.

---

## Einstellungsbereich

Klicke auf das Zahnradsymbol (⚙) in der Werkzeugleiste, um den Einstellungsbereich zu öffnen. Er hat drei Tabs:

### Graph-Darstellung

| Steuerelement | Beschreibung | Standard |
|---------|-------------|---------|
| **Knotengröße** | Alle Knoten größer oder kleiner skalieren | 1.5 |
| **Beschriftungssichtbarkeit** | Wann Beschriftungen erscheinen: Beim Überfahren, Immer oder Nie | Beim Überfahren |
| **Schriftgröße der Beschriftung** | Größe der Notiznamen-Beschriftungen | 12 |
| **Verbindungsstärke** | Breite der Kantenlinien | 1 |
| **Verwaiste Notizen anzeigen** | Notizen ohne Verknüpfungen einbeziehen | Ein |

> **Hintergrundfarbe der Leinwand.** Die Farbe hinter den Blasen wird unter **Einstellungen → Stil-Gestalter → Sky View → Leinwand → Hintergrund** festgelegt (nicht in diesem Bereich). Sie ist unabhängig von deinen Seitenleisten/Bereichen, sodass du dem Graphen seinen eigenen Hintergrund geben kannst — eine tiefe Farbe, damit die Blasen hervortreten, zum Beispiel — ohne den Rest der Oberfläche zu verändern. Bleibt sie ungesetzt, passt sich die Leinwand der Bereichsoberfläche an. Siehe *Erscheinungsbild & Themes → Sky-View-Leinwand*.

### Physik

| Steuerelement | Beschreibung | Standard |
|---------|-------------|---------|
| **Abstoßung** | Wie stark sich Knoten gegenseitig abstoßen | 50 |
| **Verbindungskraft** | Wie stark sich verbundene Knoten anziehen | 0.05 |
| **Verbindungsabstand** | Zielabstand zwischen verbundenen Knoten | 30 |
| **Simulation neu aufheizen** | Das kräftebasierte Layout vom aktuellen Zustand aus neu starten | — |

### KI

Einstellungen für semantische KI-Verknüpfungen (Phase 2 — erfordert ein lokales Embedding-Modell).

| Steuerelement | Beschreibung |
|---------|-------------|
| **Semantische Verknüpfungen anzeigen** | KI-erkannte gestrichelte Kanten ein-/ausschalten |
| **Konfidenzschwelle** | Schieberegler zum Filtern semantischer Verknüpfungen nach Ähnlichkeitswert |

---

## Legende

Die Legende erscheint in der unteren rechten Ecke und zeigt die Farbzuordnungen für deine Bibliotheken.

### Umschalter für den Farbmodus

Klicke oben in der Legende auf die Schaltflächen **Bibliothek** oder **Ordner**, um umzuschalten, wie Knoten eingefärbt werden:

| Modus | Einfärbung |
|------|----------|
| **Bibliothek** | Jede Bibliothek erhält eine eindeutige Farbe |
| **Ordner** | Jeder Ordner der obersten Ebene erhält eine eindeutige Farbe |

### Sichtbarkeits-Kontrollkästchen

Jeder Legendeneintrag hat ein Kontrollkästchen. Entferne das Häkchen bei einer Bibliothek oder einem Ordner, um deren Knoten aus dem Graphen auszublenden. So kannst du dich auf bestimmte Teilmengen deiner Wissensbasis konzentrieren.

> [!tip]
> Im Ordner-Modus wird die Ordneranzahl in Klammern angezeigt. Lange Ordnerlisten sind scrollbar.

---

## Statusleiste

Die Statusleiste unten links zeigt:

- **Knotenzahl** — alle sichtbaren Knoten
- **Kantenzahl** — alle sichtbaren Kanten
- **MOC-Zahl** — Anzahl der Inhaltskarten (Maps of Content — Hub-Notizen mit hoher Vernetzung)
- **Name der überfahrenen Notiz** — erscheint, wenn du einen Knoten überfährst

---

## Tastaturkürzel

| Kürzel | Aktion |
|----------|--------|
| `Ctrl+G` | Sky View öffnen |
| `Escape` | Sky View schließen |
| `Ctrl+F` | Suchen-und-Hervorheben umschalten |
| `Ctrl+L` | Layout-Modus durchschalten (Organisch → Hierarchisch → Zeitlich) |
| `Space` | Fokusmodus für aktive Notiz umschalten |
| `0` | 3D-Drehung auf flaches 2D zurücksetzen |
| `W/A/S/D` | Durch den 3D-Raum fliegen (wenn gedreht) |
| `Q/E` | Im 3D-Raum nach unten/oben bewegen |

---

## RTL-Unterstützung

Die Sky View bietet erstklassige Unterstützung für Arabisch, Hebräisch und andere RTL-Schriften:

- **Knotenbeschriftungen** erkennen die Schriftrichtung automatisch — arabische Titel werden von rechts nach links dargestellt
- **Legendeneinträge** kehren die Reihenfolge von Punkt und Text je nach Inhaltssprache um
- **Tooltips und Bereiche** respektieren das RTL-Layout
- **Arabische Schrift-Ersatzdarstellung** — Beschriftungen verwenden arabische Systemschriften (Noto Naskh Arabic, Segoe UI), wenn der primären Schrift die Abdeckung arabischer Glyphen fehlt

---

## Bild-im-Bild-Überlagerung (PiP)

Wenn die Sky View geöffnet ist und du in der Seitenleiste „Notizverwaltung“ auf ein Kinder-Universum, eine Bibliothek oder einen Ordner klickst, erscheint ein **Bild-im-Bild-Fenster (PiP)** als größenveränderbare Überlagerung über dem Hauptgraphen.

### Was das PiP zeigt

Das PiP zeigt einen gefilterten Teilgraphen, der nur die Knoten enthält, die zum ausgewählten Bereich gehören. Beispielsweise zeigt ein Klick auf eine Bibliothek nur die Notizen dieser Bibliothek und ihre Verflechtungen.

### PiP-Funktionen

| Funktion | Beschreibung |
|---------|-------------|
| **Gefilterter Graph** | Es erscheinen nur Knoten aus dem ausgewählten Bereich |
| **Gefilterte Legende** | Das PiP hat seine eigene Legende, die nur die relevanten Einträge zeigt |
| **Größenveränderbar** | Ziehe an den Kanten oder Ecken, um die Größe des PiP-Fensters zu ändern |
| **Verschiebbar** | Ziehe an der Titelleiste, um das PiP an eine beliebige Stelle auf dem Bildschirm zu verschieben |

### Modusübergreifende Auswahlsynchronisierung

Ein Klick auf ein Kinder-Universum, eine Bibliothek, einen Ordner oder eine Notiz in einem beliebigen Seitenleisten-Modus (Baum, Liste oder OrgChart) hebt die entsprechenden Knoten im Sky-View-Graphen hervor. Diese bidirektionale Synchronisierung hilft dir, die räumliche Orientierung zu behalten, während du in der Seitenleiste stöberst.

---

## Wissensschichten

Die Sky View bemisst die Größe der Knoten automatisch anhand ihrer Wissensebene (1–8):

- Kleine Punkte: einfache Notizen (Datum, Information)
- Mittlere Knoten: verbundene Notizen (Aussage, Konzept)
- Große leuchtende Hubs: Synthese-Notizen (Theorie, Paradigma, Weltanschauung)

Knoten höherer Ebenen haben einen Leuchthof in einer Komplementärfarbe für visuellen Kontrast. Dies wird aktiviert, wenn eine Bibliothek mehr als 20 Notizen hat.

---

## Notizreife

Knoten zeigen einen farbigen Ring, der die Reife angibt:

- Kein Ring: Keimling (neue Notiz)
- Hellgrüner Ring: Setzling (wachsend)
- Sattgrüner Ring: Immergrün (gut etabliert)
- Goldener Ring: Kanonisch (maßgebliche Referenz)

Die Reife wird auch im Dateibaum (linker Rand) und in der Tableiste (farbiger Punkt) angezeigt.

---

## Herkunfts-Leuchten

Knoten in der Sky View zeigen ein dezentes Farbleuchten, das den Ursprung des Wissens angibt:

- **Blaues Leuchten**: Empfangenes Wissen — die Quellkette der Notiz führt zu einer externen Referenz (eine Notiz mit url, author oder doi in ihrem Frontmatter)
- **Bernsteinfarbenes Leuchten**: Entdecktes Wissen — die Quellkette der Notiz entspringt den eigenen Notizen des Nutzers

---

## Technische Hinweise

Die Sky View wird von der **GraphMind**-Engine angetrieben, einem Pixi.js-WebGL-Renderer mit einer d3-force-Simulation, die in einem eigenen Web Worker läuft. Diese Architektur gewährleistet:

- **Darstellung mit 60 fps** selbst bei Tausenden von Knoten
- **Nicht blockierendes Layout** — die Kräftesimulation friert die Benutzeroberfläche niemals ein
- **Überfahren ist rein visuell** — das Überfahren löst niemals eine Neuberechnung der Physik aus
- **Die Simulation stoppt nach dem Einpendeln** — sobald die Knoten ihre Positionen gefunden haben, hält die Physik-Engine vollständig an. Nur das Ziehen eines Knotens oder das Ändern von Einstellungen startet sie erneut.

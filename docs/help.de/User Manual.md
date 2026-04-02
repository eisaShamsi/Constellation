# Constellation Benutzerhandbuch

**Version 0.3.4 | Maerz 2026**

Constellation ist eine Desktop-Anwendung fuer persoenliches Wissensmanagement (PKM) zur Verwaltung von Markdown-Notizbibliotheken. Entwickelt mit Tauri v2, SvelteKit und Rust, laeuft sie nativ auf Windows, macOS und Linux mit vollstaendiger Unterstuetzung fuer Arabisch und RTL.

---

## Inhaltsverzeichnis

1. [Erste Schritte](#erste-schritte)
2. [Universum und Bibliotheken](#universum-und-bibliotheken)
3. [Notizen erstellen und bearbeiten](#notizen-erstellen-und-bearbeiten)
4. [Sternenansicht (GraphMind)](#sternenansicht-graphmind)
5. [Zweiter Bildschirm](#zweiter-bildschirm)
6. [Eigenschaften und Frontmatter](#eigenschaften-und-frontmatter)
7. [Vorlagen](#vorlagen)
8. [Tabellen](#tabellen)
9. [Aufgaben](#aufgaben)
10. [Importer](#importer)
11. [Kalender](#kalender)
12. [Lens](#lens)
13. [Einstellungen](#einstellungen)
14. [Tastenkuerzel](#tastenkuerzel)
15. [RTL- und Arabisch-Unterstuetzung](#rtl--und-arabisch-unterstuetzung)
16. [Sicherheit und Datenschutz](#sicherheit-und-datenschutz)
17. [Kognitive Engine](#kognitive-engine)

---

## 1. Erste Schritte

### Installation

Laden Sie das neueste Installationsprogramm von der [Constellation-Release-Seite](https://github.com/eisaShamsi/Constellation/releases) herunter:

- **Windows**: `.exe` (NSIS) oder `.msi` Installationsprogramm
- **macOS**: `.dmg` Disk-Image
- **Linux**: `.AppImage` oder `.deb` Paket

### Erster Start

Beim ersten Oeffnen von Constellation fuehrt Sie der **Universum-Einrichtungsassistent** durch folgende Schritte:

1. **Sprache waehlen** — 15 Sprachen werden unterstuetzt
2. **Bibliothek erstellen oder importieren** — verweisen Sie auf einen vorhandenen Ordner mit Markdown-Dateien oder beginnen Sie neu
3. **Universum benennen** — das Universum ist der Behaelter fuer alle Ihre Bibliotheken

### Oberflaeche im Ueberblick

| Element | Beschreibung |
|---------|--------------|
| **Seitenleiste (Ribbon)** | Navigationsschaltflaechen: Dateibaum, Suche, Sternenansicht, Kalender, Vorlagen, Einstellungen |
| **Dateibaum** | Notizen und Ordner innerhalb Ihrer Bibliotheken durchsuchen |
| **Editor** | Ihre Markdown-Notizen lesen und bearbeiten |
| **Tab-Leiste** | Mehrere Notizen in Tabs oeffnen |
| **Statusleiste** | Wortanzahl, Zeichenanzahl, Lesezeit |

---

## 2. Universum und Bibliotheken

### Was ist ein Universum?

Ein **Universum** ist der uebergeordnete Behaelter, der alle Ihre Bibliotheken enthaelt. Stellen Sie es sich als Ihren Arbeitsbereich oder Ihre Bibliothekssammlung vor.

### Was ist eine Bibliothek?

Eine **Bibliothek** ist ein Ordner auf Ihrem Computer, der Markdown-Dateien (`.md`) enthaelt. Sie koennen mehrere Bibliotheken in einem Universum haben — zum Beispiel eine fuer Arbeitsnotizen und eine fuer persoenliche Notizen.

### Bibliotheken verwalten

- **Bibliothek hinzufuegen**: Einstellungen > Bibliotheken > Bibliothek hinzufuegen, oder ziehen Sie einen Ordner in die App
- **Bibliothek entfernen**: Einstellungen > Bibliotheken > klicken Sie auf die Entfernen-Schaltflaeche neben dem Bibliotheksnamen
- **Bibliothekseinstellungen**: Jede Bibliothek kann eigene Darstellungseinstellungen haben (Schriftarten, Farben)

### Kind-Universen

Sie koennen Universen in Universen verschachteln. Ein **Kind-Universum** ist ein weiterer Universumsordner, auf den Ihr uebergeordnetes Universum verweist. Notizen aus Kind-Universen erscheinen in der Sternenansicht neben Ihren eigenen Notizen, wobei bibliotheksuebergreifende Links als gestrichelte Linien dargestellt werden.

### Automatisches Wiedereröffnen

Constellation merkt sich das zuletzt aktive Universum und öffnet es beim Start automatisch wieder. Wenn das Universum verschoben wurde oder sich sein Pfad geändert hat, erkennt Constellation dies und korrigiert den Pfad automatisch.

### Portable Universen

Constellation-Universen sind vollstaendig portabel. Sie koennen einen Universumsordner an einen beliebigen Ort verschieben — ein anderes Laufwerk, einen USB-Stick oder einen anderen Computer — und Constellation erkennt und repariert automatisch alle internen Pfade beim erneuten Oeffnen.

So verschieben Sie ein Universum:
1. Schliessen Sie Constellation
2. Verschieben oder kopieren Sie den Universumsordner an den neuen Speicherort
3. Oeffnen Sie Constellation → der Willkommensbildschirm erscheint (alter Pfad nicht mehr gueltig)
4. Waehlen Sie **Vorhandenes Universum oeffnen** und navigieren Sie zum neuen Speicherort
5. Alle Notizen und Bibliotheken erscheinen sofort — Pfade werden automatisch korrigiert

Die Ordnerstruktur des Universums folgt dem Obsidian-Modell: Notizen befinden sich direkt im Stammordner, die Konfiguration befindet sich in `.constellation/`.

---

## 3. Notizen erstellen und bearbeiten

### Eine Notiz erstellen

| Methode | Aktion |
|---------|--------|
| **Tastatur** | `Ctrl+N` |
| **Dateibaum** | Rechtsklick auf einen Ordner > Neue Notiz |
| **Mission Control** | `Ctrl+P` > "Neue Notiz" |

### Editor-Modi

Constellation bietet zwei Editor-Modi, auswaehlbar unter **Einstellungen > Editor > Editortyp**:

#### Markdown-Editor (CodeMirror)

Der Standard-Editor fuer Power-User. Schreiben Sie Markdown direkt mit:

- **Live-Vorschau** — rendert Formatierung inline waehrend der Eingabe
- **Quellmodus** — zeigt die rohe Markdown-Syntax
- **Formatierungssymbolleiste** — erscheint bei Textauswahl
- **Slash-Befehle** — tippen Sie `/` fuer schnelle Einfuegungen
- **Wikilink-Autovervollstaendigung** — tippen Sie `[[` um Notizen zu verknuepfen
- **Mehrfach-Cursor** — `Alt+Click` oder `Ctrl+D`

#### Dokument-Editor (TipTap)

Ein WYSIWYG-Textverarbeitungserlebnis mit visueller Symbolleiste:

- Fett, Kursiv, Unterstrichen, Durchgestrichen, Hervorhebung
- Ueberschriften (H1–H3), Textausrichtung
- Aufzaehlungslisten, Nummerierte Listen, Aufgabenlisten
- Zitate, Codebloecke, Horizontale Linien
- Tabellen (einfuegen, Zeilen und Spalten hinzufuegen/entfernen)
- Links und Bilder

Beide Editoren speichern als Standard-Markdown-Dateien. Sie koennen jederzeit zwischen ihnen wechseln, ohne Datenverlust.

### Callouts (Hinweisbloecke)

Erstellen Sie gestaltete Hinweisbloecke fuer Notizen, Warnungen, Tipps und andere Hinweise:

```markdown
> [!note] Wichtige Information
> Der Inhalt des Callouts steht hier.

> [!warning] Vorsicht
> Diese Aktion kann nicht rueckgaengig gemacht werden.

> [!tip]- Klicken zum Aufklappen
> Einklappbarer Callout-Inhalt.
```

Unterstuetzte Typen: `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Jeder Typ hat eine eigene Farbe und ein eigenes Symbol. Fuegen Sie `-` nach dem Typ hinzu, um ihn einklappbar zu machen (startet eingeklappt), oder `+` (startet ausgeklappt).

### Hervorhebungssyntax

Umschliessen Sie Text mit doppelten Gleichheitszeichen, um ihn hervorzuheben:

```markdown
Dies ist ==hervorgehobener Text== in Ihrer Notiz.
```

In der Live-Vorschau werden die `==`-Zeichen ausgeblendet und der Text erscheint mit gelbem Hintergrund.

### Codebloecke

Umzaeunte Codebloecke werden mit einer Hintergrundfarbe und Sprachbezeichnung angezeigt:

````markdown
```javascript
const greeting = "Hello, world!";
```
````

Der Sprachname erscheint als Badge ueber dem Codeblock.

### Bildeinbettungen

Betten Sie Bilder direkt in Ihre Notizen ein:

```markdown
![Alt-Text](https://example.com/image.png)   — externe URL
![[photo.jpg]]                                 — lokale Datei aus der Bibliothek
```

In der Live-Vorschau werden Bilder inline gerendert. Lokale Bilder muessen sich in Ihrem Bibliotheksordner befinden. Externe Bilder erfordern eine Internetverbindung.

### Tabellen-Symbolleiste

Wenn sich Ihr Cursor innerhalb einer Markdown-Tabelle befindet, erscheint eine schwebende Symbolleiste mit:

- **+ Zeile / + Spalte** — Zeilen oder Spalten hinzufuegen
- **- Zeile / - Spalte** — Zeilen oder Spalten entfernen
- **Ausrichtung** — Links-, Mitte-, Rechtsausrichtung pro Spalte
- **Sortierung** — Zeilen aufsteigend oder absteigend nach der aktuellen Spalte sortieren
- **Tab / Shift+Tab** — Zwischen Tabellenzellen navigieren

### Textformatierungs-Kuerzel

| Kuerzel | Aktion |
|---------|--------|
| `Ctrl+B` | Fett |
| `Ctrl+I` | Kursiv |
| `Ctrl+Shift+S` | Durchgestrichen |
| `Ctrl+Shift+H` | Hervorhebung |
| `Ctrl+K` | Wikilink einfuegen |
| `Ctrl+Z` | Rueckgaengig |
| `Ctrl+Shift+Z` | Wiederholen |

### Notizen verknuepfen

Tippen Sie `[[`, um die Notizvervollstaendigung zu oeffnen. Beginnen Sie mit der Eingabe eines Notiznamens und waehlen Sie aus den Vorschlaegen. Links erscheinen als anklickbare Wikilinks: `[[Notizname]]`.

Sie koennen auch auf bestimmte Ueberschriften verlinken: `[[Notizname#Ueberschrift]]`.

---

## 4. Sternenansicht (GraphMind)

Die Sternenansicht visualisiert Ihre Notizen als interaktiven 3D-Graphen, angetrieben von der **GraphMind**-Engine (Pixi.js WebGL).

### Sternenansicht oeffnen

- Klicken Sie auf das Graph-Symbol in der Seitenleiste
- Druecken Sie `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Sternenansicht"

### Navigation

| Eingabe | Aktion |
|---------|--------|
| **Klicken + ziehen** | Graph verschieben |
| **Scrollen** | Hinein-/Herauszoomen |
| **Knoten anklicken** | Notiz oeffnen |
| **Rechtsklick auf Knoten** | Kontextmenue (Oeffnen, Fokussieren, Anheften, Ausblenden) |
| **Mittelklick + ziehen** | In 3D drehen |
| **W/A/S/D** | Durch den 3D-Raum fliegen |
| **0** | Rotation auf 2D zuruecksetzen |
| **Ctrl+F** | Suchen und hervorheben |
| **Leertaste** | Fokusmodus umschalten |

### Layout-Modi

Druecken Sie `Ctrl+L`, um zwischen folgenden zu wechseln:

- **Organisch** — kraeftebasiertes Layout, bei dem Cluster natuerlich entstehen
- **Hierarchisch** — Baumansicht von oben nach unten
- **Zeitlich** — Notizen chronologisch auf einer Zeitleiste angeordnet

### Fokusmodus

Rechtsklick auf einen Knoten > **Fokussieren**, um nur seine Nachbarschaft zu sehen. Einstellbar:

- **Tiefe** (1–5 Hops) — wie viele Verbindungsebenen angezeigt werden
- **Richtung** (Alle/Eingehend/Ausgehend) — alle Links, nur eingehende oder nur ausgehende

### 3D-Navigation

Mittelklick und ziehen zum Drehen. Verwenden Sie W/A/S/D/Q/E, um durch das Sternenfeld zu fliegen. Ein XYZ-Achsen-Gizmo in der Ecke zeigt Ihre Orientierung an. Druecken Sie `0` zum Zuruecksetzen.

### Einstellungen

Klicken Sie auf das Zahnrad-Symbol fuer:

- **Darstellung**: Knotengroesse, Beschriftungssichtbarkeit, Schriftgroesse, Linkstaerke, Verwaiste anzeigen
- **Physik**: Abstossungskraft, Linkkraft, Linkdistanz
- **KI**: Semantischer Link-Schwellenwert (Phase 2)

### Legende

Die Legende unten rechts zeigt Bibliotheks-/Ordnerfarben mit Kontrollkaestchen zum Umschalten der Sichtbarkeit.

### Wissensschichten

Die Sternenansicht klassifiziert Ihre Notizen automatisch in acht Wissensschichten basierend auf dem Abstraktionsgrad:

| Schicht | Beschreibung |
|---------|--------------|
| **Schnappschuss** | Schnelle, fluechtige Notizen |
| **Protokoll** | Datierte Ereignisse und Tagebucheintraege |
| **Thema** | Atomare Konzepte zu einer einzelnen Idee |
| **Karte** | Organisationsnotizen, die andere Themen verbinden |
| **Rahmen** | Modelle und Denkrahmen |
| **Prinzip** | Verifizierte Regeln und Axiome |
| **Ueberzeugung** | Grundlegende Werte und Ueberzeugungen |
| **Artefakt** | Abgeschlossene, endgueltige Werke |

Die Schicht wird automatisch aus Frontmatter, Struktur und Verlinkungen der Notiz bestimmt. Sie koennen die Klassifizierung manuell ueberschreiben, indem Sie eine `stratum`-Eigenschaft in Frontmatter hinzufuegen.

### Reifelebenszyklus

Jede Notiz durchlaeuft einen Reifelebenszyklus, der ihren Entwicklungsstand widerspiegelt:

- **Samen** — Erste Idee oder Rohentwurf
- **Setzling** — Notiz nimmt Form an und hat einige Verlinkungen
- **Immergruen** — Ausgereifte, ueberpruefte und gut verlinkte Notiz
- **Kanonisch** — Endgueltiges, autoritatives Nachschlagewerk

Der Reifegrad wird automatisch basierend auf Anzahl der Verlinkungen, Ueberarbeitungsdatum und Bearbeitungshaeufigkeit aktualisiert. Sie koennen ihn auch manuell ueber die `maturity`-Eigenschaft in Frontmatter festlegen.

---

## 5. Zweiter Bildschirm

Oeffnen Sie ein separates Fenster fuer die Nebeneinander-Ansicht von Notizen.

- **Oeffnen**: Klicken Sie auf das Zweitbildschirm-Symbol in der Seitenleiste oder druecken Sie `Ctrl+Shift+N`
- **Synchronisierung**: Notizen werden im zweiten Bildschirm unabhaengig geoeffnet. Schrift- und Themeneinstellungen gelten fuer beide Fenster.
- **Notizbreite**: Einstellbar ueber den Breitenregler in der Symbolleiste

---

## 6. Eigenschaften und Frontmatter

Notizen koennen YAML-Frontmatter am Anfang enthalten:

```yaml
---
tags: [project, active]
date: 2026-03-19
status: in-progress
---
```

Constellation erkennt Eigenschaftstypen automatisch:

| Typ | Beispiel |
|-----|----------|
| **Text** | `author: John` |
| **Zahl** | `priority: 5` |
| **Datum** | `date: 2026-03-19` |
| **Liste** | `tags: [a, b, c]` |
| **Kontrollkaestchen** | `done: true` |
| **Link** | `related: [[Andere Notiz]]` |

Eigenschaftsanzeige umschalten unter **Einstellungen > Editor > Eigenschaften im Dokument** (Sichtbar / Ausgeblendet / Quelltext).

---

## 7. Vorlagen

Erstellen Sie wiederverwendbare Notizvorlagen:

1. Erstellen Sie einen Ordner fuer Vorlagen in Ihrer Bibliothek
2. Legen Sie den Vorlagenordner-Pfad unter **Einstellungen > Vorlagen** fest
3. Beim Erstellen einer neuen Notiz waehlen Sie eine Vorlage aus der Vorlagenauswahl

Vorlagen unterstuetzen Variablen:

| Variable | Wird ersetzt durch |
|----------|--------------------|
| `{{date}}` | Aktuelles Datum |
| `{{time}}` | Aktuelle Uhrzeit |
| `{{title}}` | Notiztitel |
| `{{clipboard}}` | Zwischenablage-Inhalt |

---

## 8. Tabellen

### Markdown-Tabellen

Geben Sie eine Markdown-Tabelle manuell ein oder verwenden Sie den Slash-Befehl `/table`:

```markdown
| Spalte 1 | Spalte 2 |
|----------|----------|
| Zelle 1  | Zelle 2  |
```

### Tabellen-Symbolleiste

Wenn sich Ihr Cursor in einer Tabelle befindet, erscheint eine schwebende Symbolleiste mit:

- Zeilen und Spalten hinzufuegen/entfernen
- Spalten ausrichten (links, zentriert, rechts)
- Zwischen Zellen navigieren mit `Tab` / `Shift+Tab`

### Tabellen im Dokument-Editor

Der Dokument-Editor (TipTap) bietet eine visuelle Tabellenerfahrung:

- Klicken Sie auf die Tabellen-Schaltflaeche zum Einfuegen
- Verwenden Sie das Dropdown-Menue fuer Zeilen-/Spaltenverwaltung
- Spaltenbreite aendern durch Ziehen der Raender

---

## 9. Aufgaben

Constellation unterstuetzt Aufgaben-Kontrollkaestchen in Notizen:

```markdown
- [ ] Unerledigte Aufgabe
- [x] Erledigte Aufgabe
```

Im Live-Vorschau-Modus sind Kontrollkaestchen anklickbar. Aufgaben koennen bibliotheksuebergreifend gesucht und gefiltert werden.

---

## 10. Importer

Importieren Sie Notizen aus anderen PKM-Tools:

- **Obsidian** — importiert Vaults mit vollstaendiger Wikilink-Kompatibilitaet
- **Markdown-Ordner** — importieren Sie jeden Ordner mit `.md`-Dateien
- **Andere Formate** — HTML, Textdateien

Gehen Sie zu **Einstellungen > Importer**, um einen Import zu starten.

---

## 11. Kalender

Die Kalenderansicht zeigt Notizen nach Datum geordnet:

- Notizen mit einer `date`-Eigenschaft erscheinen an ihrem jeweiligen Tag
- Tagesnotizen koennen fuer jedes Datum erstellt werden
- Navigieren Sie mit den Pfeilschaltflaechen durch die Monate

Oeffnen Sie den Kalender ueber die Seitenleiste.

---

## 12. Lens

Lens bietet gefilterte Ansichten Ihrer Notizen:

- Filtern nach Tags, Ordnern, Eigenschaften
- Sortieren nach Name, Datum oder benutzerdefinierten Eigenschaften
- Lens-Konfigurationen fuer Schnellzugriff speichern

---

## 13. Einstellungen

Zugriff auf die Einstellungen ueber das Zahnrad-Symbol in der Seitenleiste oder `Ctrl+,`.

### Allgemein

- Sprache (15 Sprachen)
- Thema (Hell / Dunkel)
- Oberflaechen-Schriftart, Textschriftart, Monospace-Schriftart, Schriftgroesse
- Schriftthema — vorgefertigte Schriftkombinationen (Schreibmaschine, Klassisch, Modern usw.) fuer schnellen Wechsel

### Editor

- Editortyp (Markdown / Dokument)
- Standardansicht (Lesen / Bearbeiten)
- Live-Vorschau-Modus
- Zeilennummern, Einrueckungshilfen, Rechtschreibpruefung
- Automatische Klammerpaare, Intelligente Listen

### Bibliotheken

- Bibliotheken hinzufuegen/entfernen
- Darstellungseinstellungen pro Bibliothek
- Speicherort fuer Anhaenge

### Updates

- Nach Updates suchen
- GitHub-Token fuer Updates aus privaten Repositories

---

## 14. Tastenkuerzel

### Global

| Kuerzel | Aktion |
|---------|--------|
| `Ctrl+N` | Neue Notiz |
| `Ctrl+O` | Sternensprung (Schnelloeffnen) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Sternenansicht oeffnen |
| `Ctrl+,` | Einstellungen |
| `Ctrl+Shift+F` | Bibliothek durchsuchen |
| `Ctrl+Shift+N` | Zweiter Bildschirm |

### Editor

| Kuerzel | Aktion |
|---------|--------|
| `Ctrl+B` | Fett |
| `Ctrl+I` | Kursiv |
| `Ctrl+K` | Wikilink einfuegen |
| `Ctrl+Z` | Rueckgaengig |
| `Ctrl+Shift+Z` | Wiederholen |
| `Ctrl+D` | Naechstes Vorkommen auswaehlen |
| `Ctrl+/` | Kommentar umschalten |
| `Tab` | Einruecken / naechste Tabellenzelle |

### Sternenansicht

| Kuerzel | Aktion |
|---------|--------|
| `Ctrl+F` | Suchen und hervorheben |
| `Ctrl+L` | Layout-Modus wechseln |
| `Leertaste` | Fokusmodus umschalten |
| `0` | 3D-Rotation zuruecksetzen |
| `W/A/S/D/Q/E` | Durch 3D fliegen |
| `Escape` | Sternenansicht schliessen |

---

## 15. RTL- und Arabisch-Unterstuetzung

Constellation bietet erstklassige Unterstuetzung fuer Arabisch, Hebraeisch, Persisch, Urdu und andere RTL-Schriften:

- **Automatische Erkennung**: Die Notizrichtung wird automatisch anhand des Inhalts erkannt
- **Oberflaeche**: Vollstaendige RTL-Oberflaeche bei Auswahl von Arabisch/Hebraeisch
- **Editor**: RTL-Textbearbeitung mit korrekter Cursorbewegung und Auswahl
- **Sternenansicht**: Arabische Beschriftungen werden von rechts nach links mit korrektem Schrift-Fallback gerendert
- **Legende**: Elemente wechseln Punkt-/Textreihenfolge basierend auf der Inhaltssprache
- **Schrift-Skripte**: Arabische, hebraeische und CJK-Schriftarten unabhaengig in den Einstellungen konfigurierbar

### Einrichtung fuer Arabisch

1. Gehen Sie zu **Einstellungen > Allgemein > Sprache** und waehlen Sie Arabisch
2. Optional: Legen Sie eine eigene arabische Schriftart unter **Einstellungen > Allgemein > Skript-Schriftarten** fest
3. Notizen mit arabischem Inhalt werden automatisch in RTL dargestellt

---

## 16. Sicherheit und Datenschutz

- **Alle Daten bleiben lokal** — keine Cloud-Synchronisierung, keine Telemetrie, kein Tracking
- **Markdown-Dateien** — Ihre Notizen sind einfache Textdateien, die Ihnen vollstaendig gehoeren
- **Kein Konto erforderlich** — Constellation funktioniert vollstaendig offline
- **Optionale Updates** — suchen Sie manuell nach Updates ueber die Einstellungen
- **Open Source** — pruefen Sie den Code unter [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 17. Kognitive Engine

Die Kognitive Engine ist das eingebaute Intelligenzsystem von Constellation, das Ihre Notizen analysiert und verborgene Muster und Zusammenhaenge zwischen Ihren Ideen aufdeckt. Ihre Kernphilosophie:

> „Es kommt nicht auf die Menge Ihrer Daten an. Entscheidend ist nicht, wie viele Quellen Sie speichern, sondern wie Sie daraus Wissen formen und zu einem sinnvollen Bewusstsein verknuepfen."

Die Kognitive Engine besteht aus sechs integrierten Werkzeugen: Typisierte Links, Wissensschichten, Reifelebenszyklus, Spannungsdetektor, Herkunftskette und Externalisierungsmaschine.

---

### 17.1 Typisierte Links

#### Was ist das?

Typisierte Links sind Wikilinks, die eine Beziehungsart tragen und beschreiben, wie zwei Notizen zueinander stehen. Statt einfach `[[Notiz]]` schreiben Sie `[[Notiz|Beziehungstyp]]`, um die Art der Verbindung auszudruecken — baut sie darauf auf? Widerspricht sie? Erweitert sie?

#### Warum ist das wichtig?

Ein normaler Link sagt nur „es gibt eine Verbindung", aber nicht welche. Typisierte Links verwandeln Ihr Notiznetzwerk von einer losen Sammlung in eine echte Wissenslandkarte, die Denkstrukturen, Abhaengigkeiten und Schlussfolgerungen sichtbar macht.

#### So verwenden Sie es

1. Oeffnen Sie eine Notiz im Editor
2. Schreiben Sie einen Wikilink mit Beziehungstyp: `[[Zielnotiz|derives-from]]`
3. Unterstuetzte Typen: `derives-from` (abgeleitet von), `supports` (unterstuetzt), `contradicts` (widerspricht), `extends` (erweitert), `exemplifies` (veranschaulicht), `questions` (hinterfragt)
4. Alternativ koennen Sie Typen ueber die Notizeigenschaften in der rechten Seitenleiste hinzufuegen

#### Wo sehen Sie es?

- **Sternenansicht (GraphMind)**: Als farbige, beschriftete Linien zwischen den Knoten
- **Rechte Seitenleiste**: Im Tab „Backlinks" mit Angabe des jeweiligen Linktyps
- **Herkunfts-Tab**: Wird zum Aufbau des Wissens-Stammbaums verwendet

---

### 17.2 Wissensschichten

#### Was ist das?

Die Kognitive Engine klassifiziert jede Notiz automatisch in eine von acht Wissensschichten: Schnappschuss, Protokoll, Thema, Karte, Rahmen, Prinzip, Ueberzeugung, Artefakt. Die Einordnung basiert auf Struktur, Inhalt und Verlinkungsgrad der Notiz.

#### Warum ist das wichtig?

Die Schicht-Zuordnung zeigt Ihnen die Wissensbalance Ihrer Bibliothek. Bestehen Ihre Notizen hauptsaechlich aus fluechtige Schnappschuessen oder haben sie sich zu Prinzipien und Rahmenwerken entwickelt? Dieses Bewusstsein fuer die Art des Inhalts ist der erste Schritt, um echtes Wissen aufzubauen statt nur Informationen zu sammeln.

#### So verwenden Sie es

1. Die Klassifizierung erfolgt automatisch — Sie muessen nichts tun
2. Um die automatische Einstufung zu ueberschreiben, fuegen Sie die Eigenschaft `stratum` im Frontmatter hinzu:
   ```yaml
   ---
   stratum: framework
   ---
   ```
3. Verfuegbare Werte: `snapshot`, `log`, `topic`, `map`, `framework`, `principle`, `conviction`, `artifact`

#### Wo sehen Sie es?

- **Rechte Seitenleiste**: Im Bereich Notizeigenschaften unter „Schicht"
- **Sternenansicht**: Als unterschiedliche Farben der Knoten je nach Schicht
- **Einstellungen > Kognitive Engine**: Automatische Klassifizierung aktivieren/deaktivieren

---

### 17.3 Reifelebenszyklus

#### Was ist das?

Die Engine verfolgt den Reifegrad jeder Notiz in vier Stufen: **Samen** → **Setzling** → **Immergruen** → **Kanonisch**. Jede Notiz beginnt als Samen und waechst schrittweise mit zunehmendem Inhalt, mehr Verlinkungen und Ueberarbeitungen.

#### Warum ist das wichtig?

Der Reifegrad unterscheidet zwischen einem rohen Gedanken und ausgereiftem Wissen. Der Samen von heute kann morgen zum Referenzwerk werden, wenn Sie ihn pflegen. Die Reifeverfolgung hilft Ihnen, Notizen zu identifizieren, die weitere Aufmerksamkeit und Ausarbeitung verdienen.

#### So verwenden Sie es

1. Der Reifegrad aendert sich automatisch basierend auf: Wortanzahl, Anzahl ein- und ausgehender Links und Datum der letzten Bearbeitung
2. Um den Reifegrad manuell festzulegen, fuegen Sie die Eigenschaft `maturity` im Frontmatter hinzu:
   ```yaml
   ---
   maturity: evergreen
   ---
   ```
3. Verfuegbare Werte: `seed` (Samen), `sapling` (Setzling), `evergreen` (Immergruen), `canonical` (Kanonisch)

#### Wo sehen Sie es?

- **Rechte Seitenleiste**: Ein Symbol neben dem Notiztitel zeigt die aktuelle Reifestufe
- **Sternenansicht**: Als Knotengroesse — je reifer die Notiz, desto groesser der Knoten
- **Einstellungen > Kognitive Engine**: Reife-Tracking aktivieren/deaktivieren

---

### 17.4 Spannungsdetektor

#### Was ist das?

Der Spannungsdetektor prueft verknuepfte Notizen und warnt Sie, wenn Behauptungen oder Schlussfolgerungen zwischen zwei oder mehr Notizen im Widerspruch stehen. Er stuetzt sich auf die Analyse von `contradicts`-Links und thematische Aehnlichkeit.

#### Warum ist das wichtig?

Spannungen sind nicht unbedingt Fehler — sie sind Anstoesse zum tieferen Nachdenken. Wenn sich zwei Ideen in Ihrer Bibliothek widersprechen, bedeutet das, dass sich Ihr Verstaendnis weiterentwickelt hat oder eine Komplexitaet vorliegt, die es zu erforschen lohnt. Der Spannungsdetektor schuetzt Sie davor, unbewusst Wissen auf widersprüchlichen Grundlagen aufzubauen.

#### So verwenden Sie es

1. Fuegen Sie einen typisierten Link `contradicts` zwischen widersprüchlichen Notizen ein: `[[Andere Notiz|contradicts]]`
2. Die Engine erkennt auch implizite Spannungen durch Inhaltsanalyse
3. Pruefen Sie die Liste der erkannten Spannungen in der Seitenleiste

#### Wo sehen Sie es?

- **Rechte Seitenleiste**: Im Tab „Spannungen", wenn Widersprueche erkannt wurden
- **Sternenansicht**: Als rote gestrichelte Linien zwischen widersprüchlichen Knoten
- **Benachrichtigungen**: Hinweise bei neu erkannten Spannungen

---

### 17.5 Herkunftskette

#### Was ist das?

Die Herkunftskette verfolgt den Ursprung jeder Idee — woher sie stammt und woraus sie abgeleitet wurde. Sie nutzt `[[Notiz|derives-from]]`-Links, um einen Stammbaum aufzubauen, der den Entwicklungsweg des Wissens von der Originalquelle bis zur aktuellen Formulierung zeigt.

#### Warum ist das wichtig?

Zu wissen, woher Ihre Ideen kommen, unterscheidet empfangenes Wissen (aus Buechern, Artikeln, Vortraegen) von entdecktem Wissen (Ihre eigenen Schlussfolgerungen und Ueberlegungen). Dieses Quellenbewusstsein hilft Ihnen, die Zuverlaessigkeit Ihrer Ideen einzuschaetzen und nachzuvollziehen, wie sich Ihr Denken ueber die Zeit geformt hat.

#### So verwenden Sie es

1. Wenn Sie eine Notiz aus einer Quelle ableiten, fuegen Sie einen Link hinzu: `[[Originalquelle|derives-from]]`
2. Mehrstufige Ketten sind moeglich: Notiz ← abgeleitet von ← abgeleitet von ← Originalquelle
3. Klassifizieren Sie externe Quellen mit `source-type: received` im Frontmatter

#### Wo sehen Sie es?

- **Rechte Seitenleiste**: Der Tab „Herkunft" zeigt den vollstaendigen Stammbaum
- **Sternenansicht**: Als Pfeilrichtungen auf den Links (von Quelle zum Abgeleiteten)
- **Notizeigenschaften**: Klassifizierung als „empfangen" oder „entdeckt" basierend auf der Herkunftskette

### 17.6 Externalisierungsmaschine

#### Was ist das?

Eine progressive Formalisierungspipeline, die verfolgt, wie Ihre Notizen von rohen Erfassungen zu kristallisierten Erkenntnissen reifen. Jede Notiz kann einer von vier Stufen zugewiesen werden:

| Stufe | Symbol | Bedeutung |
|-------|--------|-----------|
| Fluechtig | 🌱 | Schnelle Erfassung, fluechtige Idee |
| Literatur | 📖 | In eigenen Worten aus einer Quelle umgeschrieben |
| Permanent | 🔗 | Atomare Idee, ein Konzept, mit Ihrem Graphen verbunden |
| Synthese | ✨ | Originale Erkenntnis aus mehreren permanenten Notizen |

#### Warum ist das wichtig?

Die meisten Apps behandeln alle Notizen gleich. Die Externalisierungsmaschine macht den Unterschied sichtbar — Sie koennen auf einen Blick sehen, wie viel Ihrer Bibliothek aus rohen Erfassungen besteht und wie viel echtes Verstaendnis ist.

#### So verwenden Sie es

1. Verwenden Sie in der Breadcrumb-Leiste (ueber dem Editor) das Stufen-Dropdown, um eine Stufe auszuwaehlen.
2. Oder erweitern Sie die Eigenschaften und verwenden Sie das Stufen-Dropdown dort. Beides synchronisiert sich sofort mit dem Dateibaum.
3. Um eine Notiz hochzustufen, aendern Sie das Dropdown von einer Stufe zur naechsten. Im Fokus-Modus klicken Sie unten auf „Hochstufen zu Permanent".
4. Um eine Stufe zu entfernen, waehlen Sie „— Stufe —" aus dem Dropdown.

#### Wo sehen Sie es?

- **Breadcrumb-Leiste**: Dropdown mit Symbol + Stufenname
- **Eigenschaftenpanel**: Dropdown, wenn die `stage`-Eigenschaft vorhanden ist
- **Dateibaum**: Emoji-Symbol neben dem Notiznamen
- **Fokus-Modus-Fusszeile**: Schaltflaeche „Hochstufen zu Permanent"

### Einstellungen der Kognitiven Engine

Alle Werkzeuge der Kognitiven Engine koennen unter **Einstellungen > Kognitive Engine** konfiguriert werden:

- **Schicht-Klassifizierung** — Automatische Klassifizierung aktivieren oder deaktivieren
- **Reife-Tracking** — Reifelebenszyklus-Tracking aktivieren oder deaktivieren
- **Typisierte Links** — Empfindlichkeitsschwelle fuer Link-Erkennung anpassen (0.0 – 1.0)
- **Spannungsdetektor** — Automatische Spannungserkennung aktivieren oder deaktivieren
- **Manuelle Ueberschreibung** — Fuegen Sie `stratum`- und `maturity`-Eigenschaften in Frontmatter hinzu, um die automatische Klassifizierung zu ueberschreiben

---

*Constellation Benutzerhandbuch — Version 0.3.4 — Maerz 2026*
*uconstellation.world*

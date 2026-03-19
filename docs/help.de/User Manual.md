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

*Constellation Benutzerhandbuch — Version 0.3.4 — Maerz 2026*
*uconstellation.world*

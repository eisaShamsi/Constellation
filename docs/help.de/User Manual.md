# Constellation Benutzerhandbuch

**Version 0.3.4 | Maerz 2026**

Constellation ist eine Desktop-Anwendung fuer persoenliches Wissensmanagement (PKM) zur Verwaltung von Markdown-Notizbibliotheken. Entwickelt mit Tauri v2, SvelteKit und Rust, laeuft sie nativ auf Windows, macOS und Linux mit vollstaendiger Unterstuetzung fuer Arabisch und RTL.

---

## Inhaltsverzeichnis

1. [Erste Schritte](#erste-schritte)
2. [Universum und Bibliotheken](#universum-und-bibliotheken)
3. [Notizen erstellen und bearbeiten](#notizen-erstellen-und-bearbeiten)
4. [Suche](#suche)
5. [Sternenansicht (GraphMind)](#sternenansicht-graphmind)
6. [Geteilte Ansicht](#geteilte-ansicht)
7. [Index](#index)
8. [Constellation Sight](#constellation-sight)
9. [Zweiter Bildschirm](#zweiter-bildschirm)
10. [Eigenschaften und Frontmatter](#eigenschaften-und-frontmatter)
11. [Vorlagen](#vorlagen)
12. [Tabellen](#tabellen)
13. [Aufgaben](#aufgaben)
14. [Importer](#importer)
15. [Kalender](#kalender)
16. [Lens](#lens)
17. [Einstellungen](#einstellungen)
18. [Tastenkuerzel](#tastenkuerzel)
19. [RTL- und Arabisch-Unterstuetzung](#rtl--und-arabisch-unterstuetzung)
20. [Sicherheit und Datenschutz](#sicherheit-und-datenschutz)
21. [Wissenskarte](#wissenskarte)
22. [Kognitive Engine](#kognitive-engine)

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

## 4. Suche

Constellation verfuegt ueber eine hybride mehrsprachige Suchmaschine auf Basis von SQLite FTS5 mit BM25-Ranking, strukturierten Abfragefiltern und arabisch-optimierter Normalisierung. Die Suche ist ueber die Seitenleiste erreichbar.

### So suchen Sie

Klicken Sie auf das Suchsymbol in der Seitenleiste oder verwenden Sie `Ctrl+Shift+F`, um den Suchmodus zu aktivieren. Geben Sie Ihre Abfrage ein und Ergebnisse erscheinen nach einer kurzen Verzoegerung (300ms). Druecken Sie `Escape` oder klicken Sie auf die Schaltflaeche `x`, um die Suche zu loeschen und zum Dateibaum zurueckzukehren.

### Suchsyntax

| Syntax | Beispiel | Was gefunden wird |
|--------|----------|-------------------|
| Freitext | `Projektmanagement` | Notizen mit diesen Woertern in Titel oder Text |
| Tag-Filter | `#forschung` | Notizen mit dem Tag `#forschung` |
| Eigenschaftsfilter | `status=aktiv` | Notizen mit Frontmatter-Eigenschaft `status` gleich `aktiv` |
| Wikilink-Filter | `links to [[Klima]]` | Notizen mit Verweis auf `[[Klima]]` |
| Bibliotheksbereich | `in:MeineBibliothek` | Ergebnisse auf eine bestimmte Bibliothek beschraenken |
| Kombiniert | `#forschung status=aktiv Wirtschaft` | Alle Filter gemeinsam angewendet |

### Treffer-Badges

Jedes Suchergebnis zeigt ein farbiges Badge, das angibt, wie der Treffer gefunden wurde. Das Badge zeigt einen lokalisierten Buchstaben fuer Barrierefreiheit (farbenblind-sicher):

| Badge | Farbe | Bedeutung |
|-------|-------|-----------|
| **T** | Blau | Titeltreffer — der Suchbegriff erscheint im Notiznamen |
| **I** | Gruen | Inhaltstreffer — der Suchbegriff erscheint im Notiztext |
| **S** | Lila | Semantischer Treffer — konzeptuell verwandt (erfordert Embedding-Modell) |
| **E** | Bernstein | Eigenschaftstreffer — ueber Frontmatter-Eigenschaftsfilter gefunden |
| **#** | Pink | Tag-Treffer — ueber Tag-Filter gefunden |
| **W** | Hellblau | Wikilink-Treffer — ueber Wikilink-Filter gefunden |

Badge-Buchstaben sind fuer alle 15 unterstuetzten Sprachen lokalisiert.

### Angeheftete Ergebnisse (Durch Ergebnisse navigieren)

Suchergebnisse bleiben nach dem Anklicken sichtbar. Die geoeffnete Notiz wird in der Ergebnisliste hervorgehoben, sodass Sie sehen koennen, welches Ergebnis Sie gerade betrachten. Klicken Sie auf ein anderes Ergebnis, um dorthin zu navigieren, ohne erneut zu suchen.

Um die Suche zu loeschen, druecken Sie `Escape` oder klicken Sie auf `x`.

### Tastaturnavigation

| Taste | Aktion |
|-------|--------|
| `Pfeil nach unten` | Naechstes Ergebnis auswaehlen |
| `Pfeil nach oben` | Vorheriges Ergebnis auswaehlen |
| `Enter` | Ausgewaehltes Ergebnis oeffnen |
| `Escape` | Suche loeschen und zum Dateibaum zurueckkehren |

### Suchbegriff-Hervorhebung

Wenn Sie eine Notiz aus den Suchergebnissen oeffnen, werden alle Vorkommen des Suchbegriffs im Editor hervorgehoben. Dies funktioniert mit arabischer diakritischer Erkennung — die Suche nach "ادارة" hebt "إدارة" und alle diakritischen Varianten hervor.

### Suchverlauf

Klicken Sie auf das Suchfeld, wenn es leer ist, um Ihre letzten Suchen (letzte 20 Abfragen) anzuzeigen. Jeder Eintrag zeigt den Abfragetext und den Zeitpunkt der Ausfuehrung. Klicken Sie auf einen Eintrag, um die Suche sofort erneut auszufuehren. Nutzen Sie den Link "Verlauf loeschen" am Ende, um den gesamten Verlauf zu entfernen.

Der Suchverlauf wird lokal auf Ihrem Geraet gespeichert und bleibt ueber Neustarts erhalten.

### Search Hub

Der Search Hub ist eine Vollbild-Sucherfahrung. Klicken Sie auf das Lupensymbol in der Dock-Leiste, um ihn zu oeffnen. Beide Seitenleisten klappen ein, um maximalen Platz zu bieten. Geben Sie einen beliebigen Begriff ein und Constellation durchsucht gleichzeitig alles und gruppiert Ergebnisse in 5 Kategorien: Titel, Inhalte, Tags, Eigenschaften und Wikilinks. Jede Kategorie hat einen aufklappbaren Abschnitt mit einem Zaehler-Badge. Klicken Sie auf ein Ergebnis, um es im Editor zu oeffnen, wobei alle Vorkommen hervorgehoben werden. Eine Schaltflaeche "Zurueck zum Search Hub" erscheint, damit Sie zurueckkehren koennen, ohne erneut zu suchen.

### Link-Operatoren

Constellation unterstuetzt 6 Link-Topologie-Suchoperatoren:

| Syntax | Was es findet |
|--------|---------------|
| `links to [[X]]` | Notizen, die auf X verlinken (Backlinks) |
| `links from [[X]]` | Notizen, auf die X verlinkt (ausgehende Links) |
| `mutual [[X]]` | Notizen, die mit X verlinkt sind UND X verlinkt zurueck (bidirektional) |
| `mentions [[X]]` | Notizen, die den Namen von X ohne [[Wikilink]] enthalten |
| `orphans` | Notizen ohne eingehende oder ausgehende Links |
| `links between [[X]] and [[Y]]` | Notizen, die sowohl auf X als auch auf Y verlinken |

Bei der Eingabe eines Link-Operators zeigt die `[[`-Autovervollstaendigung alle Notizen im Universum. Nach Auswahl einer Notiz geben Sie `#` fuer Ueberschriften-Vervollstaendigung oder `|type:` fuer Linktyp-Vervollstaendigung ein.

---

## 5. Sternenansicht (GraphMind)

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

## 6. Geteilte Ansicht

Die geteilte Ansicht ermoeglicht es Ihnen, mehrere Notizen nebeneinander im Hauptfenster zu bearbeiten.

### Geteilte Ansicht oeffnen

- **Befehlspalette**: `Ctrl+P` und dann "Split View" eingeben
- **Tastenkuerzel**: Verwenden Sie das zugewiesene Kuerzel, um zwischen den Modi zu wechseln
- **Zyklus**: Aus → Vertikal (nebeneinander) → Horizontal (uebereinander) → Aus

### Bearbeiten in der geteilten Ansicht

Jedes Feld ist ein vollstaendig unabhaengiger Editor mit:
- Vollstaendiger Symbolleiste (Fett, Kursiv, Ueberschriften, Ausrichtung usw.)
- Breadcrumb-Navigation (Bibliothek / Notizname)
- Eigenschaftenpanel und Stufen-Dropdown
- Speicherunterstuetzung (`Ctrl+S` speichert das fokussierte Feld)
- Titelbearbeitung und Dateiumbenennung

### Felder in der Groesse aendern

Ziehen Sie den Teiler zwischen den Feldern, um ihre Groesse zu aendern. Jeder Teiler ist unabhaengig — bei 3 oder mehr geoeffneten Notizen koennen Sie jedes benachbarte Paar anpassen, ohne die anderen zu beeinflussen. Funktioniert sowohl im vertikalen als auch im horizontalen Modus.

### Fokus

Klicken Sie auf ein beliebiges Feld, um es zu fokussieren. Das fokussierte Feld empfaengt Tastenkuerzel und wird von den Panels der rechten Seitenleiste (Eigenschaften, Rueckverweise usw.) verfolgt.

---

## 7. Index

Der Index ist ein umfassendes Begriffsglossar ueber alle Ihre Bibliotheken — jedes bedeutungsvolle Wort, alphabetisch sortiert mit Vorkommenshaeufigkeiten.

### Index oeffnen

- **Dock-Schaltflaeche**: Klicken Sie auf das Index-Symbol (Buch) im linken Dock
- **Befehlspalette**: `Ctrl+P` und dann "Index" eingeben

### Mehrsprachige NLP-Pipeline

Der Index verarbeitet Text durch eine sprachbewusste Pipeline vor der Indexierung:

- **Arabisch**: Lucene Light10-Algorithmus — entfernt Tashkeel, vereinheitlicht Hamza, entfernt bestimmten Artikel (الـ), entfernt grammatische Suffixe
- **Hebraeisch**: Praefixentfernung (ב/ל/מ/ה/ו/כ/ש)
- **Englisch**: Porter-aehnliches Stemming (Pluralformen, Verbformen, Suffixe)
- **Franzoesisch/Spanisch/Portugiesisch/Deutsch**: Sprachspezifische Suffixentfernung
- **Russisch/Tuerkisch/Hindi/Persisch**: Morphologische Suffixentfernung
- **Alle 15 Sprachen**: Stoppwortfilterung (Artikel, Praepositionen, Konjunktionen)

### Durchsuchen

- **Sprach-Tabs**: Wechseln Sie zwischen Alle, Arabisch, Hebraeisch, Englisch oder # (Sonderzeichen)
- **Alphabetleiste**: Klicken Sie auf einen Buchstaben, um Begriffe zu filtern, die mit diesem Buchstaben beginnen — die Begriffsanzahl aktualisiert sich entsprechend
- **Klicken Sie erneut auf denselben Buchstaben**, um den Filter zu loeschen und alle Begriffe anzuzeigen
- **Sortiermodi**: Alphabetisch (Standard) oder nach Haeufigkeit (haeufigste zuerst)

### Bearbeiten aus dem Index

Klicken Sie auf eine Notiz in den Referenzen eines Begriffs, um sie in einem geteilten Vorschaufeld neben dem Index zu oeffnen. Das Vorschaufeld ist ein vollstaendiger Editor — Sie koennen bearbeiten, speichern, Eigenschaften aendern und die Stufe hochsetzen. Der Suchbegriff wird in der Notiz hervorgehoben und automatisch dorthin gescrollt.

Druecken Sie `Ctrl+Klick`, um die Notiz als normalen Tab zu oeffnen. Eine Schaltflaeche "Zurueck zum Index" erscheint in der Tab-Leiste — klicken Sie darauf, um genau an die Stelle im Index zurueckzukehren, an der Sie aufgehoert haben.

### Integration mit dem Zweiten Bildschirm

Wenn der Zweite Bildschirm geoeffnet ist:
- **Klicken Sie auf einen Begriff** → Der Zweite Bildschirm zeigt alle Notizen mit diesem Begriff in einer geteilten Ansicht (Notizliste + Editor)
- **Ctrl+Klick auf mehrere Begriffe** → Der Zweite Bildschirm zeigt den Vergleichsmodus mit jedem Begriff in einer eigenen Spalte

---

## 8. Constellation Sight

Constellation Sight visualisiert Ihr gesamtes Wissenssystem als Gravitationsschacht-Graph. Es beantwortet die Frage: **"Wie sieht mein Wissen aus und wie gesund ist es?"**

### Sight oeffnen

Klicken Sie auf die **Sight-Schaltflaeche** (Augensymbol) in der linken Leiste. Der Gravitationsschacht-Graph erscheint. Klicken Sie auf x zum Schliessen.

### Der Gravitationsschacht-Graph

Notizen sind in konzentrischen Ringen nach Wichtigkeit (Zentralitaet) angeordnet. Die am staerksten vernetzten Notizen befinden sich im Zentrum; periphere Notizen am Rand. Innerhalb jedes Rings sind Notizen nach Bibliothek (Ihrer Organisation) gruppiert. Knotenfarbe = Bibliothek.

| Element | Bedeutung |
|---------|-----------|
| **Grosser Knoten** | Hohe Zentralitaet — verbindet verschiedene Wissensbereiche |
| **Kleiner Knoten** | Peripher — innerhalb eines Bereichs |
| **Knotenfarbe** | Bibliothekszugehoerigkeit |
| **Durchgezogene Linie** | Verbindung zwischen zwei Notizen |
| **Richtungspfeile** | Kleine Pfeile, die die Verbindungsrichtung anzeigen |
| **Linienstaerke** | Konfidenzniveau (dick = etabliert, duenn = Hypothese) |

### Interaktion

- **Einfachklick** auf einen Knoten: Hebt seine Nachbarschaft hervor (alle verbundenen Notizen). Alles andere wird abgeblendet.
- **Doppelklick**: Oeffnet die Notiz im Editor.
- **Klick auf leeren Bereich**: Loescht die Hervorhebung.
- **Scrollen**: Zoom. **Ziehen**: Schwenken. **An Bildschirm anpassen**: Symbolleisten-Schaltflaeche.

### Suche in Sight

Klicken Sie auf die Lupe. Unterstuetzt alle Operatoren: `links to [[X]]`, `links from [[X]]`, `mutual [[X]]`, `orphans`, `supports [[X]]`, `contradicts [[X]]`, `#tag`, Freitext und semantische Suche. Ergebnisse zeigen Richtungsfarben: Gruen (eingehend), Rot (ausgehend).

### Analyse-Panel (SightPanel)

Klicken Sie auf das Rastersymbol, um die Seitenleiste zu oeffnen. Zeigt: Universum-Gesundheitswert (0-100), Notiz-/Link-/Orphan-Zaehler, Linktyp- und Konfidenzbalken, Top 10 Bruecken und Wissenseinblicke (staerkste Evidenz, schwache Grundlagen, Spannungen, stagnierende, am staerksten vernetzte, Wissensluecken).

### Einstellungen

Zahnradsymbol: Passen Sie Linkstrichstaerke, Deckkraft und Pfeilgroesse an. Einstellungen bleiben sitzungsuebergreifend erhalten.

---

## 9. Zweiter Bildschirm

Der Zweite Bildschirm ist ein modusbasiertes Begleitfenster, das sich an Ihren aktuellen Seitenleistenmodus anpasst.

- **Oeffnen**: Klicken Sie auf das Zweitbildschirm-Symbol in der Seitenleiste, oder `Ctrl+Shift+2`
- **Automatisches Schliessen**: Wenn Sie das Hauptfenster schliessen, wird der zweite Bildschirm automatisch geschlossen

### Modusbasierter Begleiter

Der zweite Bildschirm aendert seinen Inhalt basierend auf dem aktiven Seitenleistenmodus im Hauptfenster:

| Hauptseitenleistenmodus | Zweiter Bildschirm zeigt |
|---|---|
| **Datei-Explorer** | Universum-Dashboard — Statistiken, Bibliotheksaufschluesselung, Kind-Universen, Tags, zuletzt bearbeitete/geoeffnete Notizen |
| **Navigator** | Vollstaendige Navigator-Ansicht zum Durchsuchen von Notizen |
| **Himmelsansicht** | Himmelsansicht-Baum mit Verzeichnisstruktur |
| **Sternenansicht** | Sternenansicht-Begleiter mit Rueckverweisen, Vorwaertsverweisen, Tags und lokalem Graph |

### Universum-Dashboard (Datei-Explorer-Modus)

Wenn das Hauptfenster im Datei-Explorer-Modus ist, zeigt der zweite Bildschirm ein Dashboard mit:

- **Statistikkarten** — Universumsname, Anzahl der Kind-Universen, Gesamtbibliotheken, Ordner und Notizen
- **Kind-Universen** — Jedes Kind-Universum mit seinen verknuepften Bibliotheken und Ordner-/Notizanzahlen
- **Bibliotheken** — Jede Bibliothek mit Ordner-/Notizanzahlen in farbcodierten Statistikboxen
- **Zuletzt bearbeitet** — Notizen, die Sie in der aktuellen Sitzung geaendert haben (verfolgt beim Speichern)
- **Zuletzt geoeffnet** — Notizen, die Sie geoeffnet, aber nicht bearbeitet haben
- **Tags** — Alle Tags ueber Bibliotheken hinweg nach Haeufigkeit sortiert; klicken Sie auf einen Tag, um alle zugehoerigen Notizen zu sehen

### Dashboard-Interaktion

Wenn das Dashboard im Hauptfenster aktiv ist, werden durch Klicken auf Elemente diese an den zweiten Bildschirm gesendet:

- **Zuletzt bearbeitet/geoeffnet**: Klicken Sie auf eine Notiz, um sie als vollstaendigen Editor auf dem zweiten Bildschirm zu oeffnen
- **Tags**: Klicken Sie auf einen Tag, um alle Notizen mit diesem Tag in einer geteilten Ansicht anzuzeigen — Notizliste links, vollstaendiger Editor rechts

Alle Bearbeitungen auf dem zweiten Bildschirm werden automatisch mit dem Hauptfenster synchronisiert.

### Notizen im zweiten Bildschirm bearbeiten

Der zweite Bildschirm unterstuetzt die vollstaendige Notizbearbeitung — tippen, speichern, umbenennen und Eigenschaften aendern, genau wie im Hauptfenster. Aenderungen werden automatisch mit dem Hauptfenster synchronisiert.

### Einstellungssynchronisierung

Alle visuellen Einstellungen werden sofort auf den zweiten Bildschirm uebertragen — kein Neustart erforderlich:

- **Sprache**: Aenderungen der Oberflaechensprache werden sofort angewendet
- **Thema**: Hell/Dunkel/System-Modus wechselt sofort
- **Schriften**: Oberflaechenschrift, Textschrift, Monospace-Schrift und schriftartspezifische Schriften
- **Schriftgroesse**: Sowohl Oberflaechen- als auch Editor-Schriftgroessen
- **Editor**: Lesbare Zeilenlaenge, Zeilennummern, schwebende Symbolleiste
- **Akzentfarbe**: Aenderungen der Themen-Akzentfarbe

---

## 10. Eigenschaften und Frontmatter

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

## 11. Vorlagen

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

## 12. Tabellen

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

## 13. Aufgaben

Constellation unterstuetzt Aufgaben-Kontrollkaestchen in Notizen:

```markdown
- [ ] Unerledigte Aufgabe
- [x] Erledigte Aufgabe
```

Im Live-Vorschau-Modus sind Kontrollkaestchen anklickbar. Aufgaben koennen bibliotheksuebergreifend gesucht und gefiltert werden.

---

## 14. Importer

Importieren Sie Notizen aus anderen PKM-Tools:

- **Obsidian** — importiert Vaults mit vollstaendiger Wikilink-Kompatibilitaet
- **Markdown-Ordner** — importieren Sie jeden Ordner mit `.md`-Dateien
- **Andere Formate** — HTML, Textdateien

Gehen Sie zu **Einstellungen > Importer**, um einen Import zu starten.

---

## 15. Kalender

Die Kalenderansicht zeigt Notizen nach Datum geordnet:

- Notizen mit einer `date`-Eigenschaft erscheinen an ihrem jeweiligen Tag
- Tagesnotizen koennen fuer jedes Datum erstellt werden
- Navigieren Sie mit den Pfeilschaltflaechen durch die Monate

Oeffnen Sie den Kalender ueber die Seitenleiste.

---

## 16. Lens

Lens bietet gefilterte Ansichten Ihrer Notizen:

- Filtern nach Tags, Ordnern, Eigenschaften
- Sortieren nach Name, Datum oder benutzerdefinierten Eigenschaften
- Lens-Konfigurationen fuer Schnellzugriff speichern

---

## 17. Einstellungen

Zugriff auf die Einstellungen ueber das Zahnrad-Symbol in der Seitenleiste oder `Ctrl+,`.

### Allgemein

- Sprache (15 Sprachen)
- Thema (Hell / Dunkel)
- Oberflaechen-Schriftart, Textschriftart, Monospace-Schriftart, Schriftgroesse
- Schriftthema — vorgefertigte Schriftkombinationen (Schreibmaschine, Klassisch, Modern usw.) fuer schnellen Wechsel
- **Themen** — waehlen Sie aus sechs eingebauten Themen, erstellen Sie benutzerdefinierte Themen (Fuenf-Farben-Editor), importieren Sie Themen aus der Obsidian-Community-Registry (200+ Themen) oder importieren Sie eine `.json`-Themendatei. Loeschen Sie jedes benutzerdefinierte Thema mit dem ✕-Button beim Hover.

### Style Settings

Ein eigener Tab fuer die feinkoernige Anpassung jedes sichtbaren Interface-Elements, live auf das aktive Thema angewendet.

- **Farben** — Hintergrund, Oberflaechen, Text (normal/gedaempft/schwach), Akzent, Rahmen, Zustandsfarben
- **Typografie** — Schriftgroessen fuer Interface/Notizen/Code, H1–H6-Groessen, Ueberschriftgewicht, Zeilenhoehen, Absatzabstand
- **Layout & Form** — klein/mittel/gross Eckenradien, Rahmenbreiten, Schatten, lesbare Editor-Zeilenlaenge, Seitenraender
- **Komponenten** — Ribbon-Dock, Aktionsleiste der Seitenleiste, Layout-Leiste (Panel-Toggles), Top-Leiste/Tab-Leiste, Statusleiste, rechte Seitenleiste (Inspector), Datei-Explorer (Universum-Notizen, Kind-Universen, Bibliotheken, Ordner, Notizen), Buttons, Tags, Callouts — jeweils mit unabhaengiger Groesse, Radius, Farbe und gegebenenfalls Aktiv-Zustand-Styling
- **Editor** — Linkfarbe/Hover/Dekoration, Inline-Code-Farbe/Hintergrund/Radius, Blockzitat-Balkenbreite/-Farbe, Cursorfarbe, Auswahlhintergrund

**Import / Export** — Symbolleiste oben im Tab:
- Aus Zwischenablage einfuegen (ein Klick)
- Import / Einfuegen (Textfeld mit Merge oder Ersetzen)
- Aus Datei (.json)
- Kopieren (aktuelle Werte in Zwischenablage)
- Exportieren (.json)

Das Format entspricht exakt dem Style-Settings-Plugin von Obsidian, sodass Sie Einstellungen zwischen Obsidian und Constellation teilen koennen.

Aenderungen werden automatisch im aktiven Thema gespeichert; wenn Sie ein eingebautes Thema bearbeiten, wird es automatisch in Ihre benutzerdefinierten Themen geklont, sodass Aenderungen bestehen bleiben, ohne das Original zu veraendern.

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

## 18. Tastenkuerzel

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

## 19. RTL- und Arabisch-Unterstuetzung

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

## 20. Sicherheit und Datenschutz

- **Alle Daten bleiben lokal** — keine Cloud-Synchronisierung, keine Telemetrie, kein Tracking
- **Markdown-Dateien** — Ihre Notizen sind einfache Textdateien, die Ihnen vollstaendig gehoeren
- **Kein Konto erforderlich** — Constellation funktioniert vollstaendig offline
- **Optionale Updates** — suchen Sie manuell nach Updates ueber die Einstellungen
- **Open Source** — pruefen Sie den Code unter [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 21. Wissenskarte

Die Wissenskarte ist eine radiale Sunburst-Visualisierung, die Struktur, Dichte und Reifegrad Ihres gesamten Wissensuniversums zeigt.

### Karte oeffnen

- **Dock-Schaltflaeche**: Klicken Sie auf das Wissenskarte-Symbol in der linken Leiste
- **Befehlspalette**: `Ctrl+P` dann "Constellation Map" eingeben

### Was Sie sehen

- **Mitte**: Ihr Universums-Name mit Gesamtzahl der Notizen und Woerter
- **Erster Ring**: Bibliotheken (jeweils in ihrer Bibliotheksfarbe). Wenn Ihr Universum Unteruniversen hat, erscheinen sie hier.
- **Tiefere Ringe**: Ordner und Unterordner innerhalb jeder Bibliothek
- **Aeusserste Segmente**: Einzelne Notizen

### Farbmodi

Wechseln Sie zwischen drei Modi ueber das Dropdown:
- **Reife**: Samen (grau) → Saemling (hellgruen) → Immergruen (gruen) → Kanonisch (gold) → Welkend
- **Schicht**: L1 (blau) → L8 (rot) — zeigt Wissenskomplexitaet
- **Bibliothek**: Alle Segmente erben die Farbe ihrer Bibliothek

### Drill-Down-Navigation

Klicken Sie auf ein Ordner-Segment zum Hineinzoomen. Ein Breadcrumb-Pfad zeigt Ihren Weg. Klicken Sie auf ein Breadcrumb-Element zum Zuruecknavigieren, oder druecken Sie Escape. Klicken Sie auf ein Notiz-Segment, um es im Editor zu oeffnen.

### Zurueck zur Karte

Nach dem Oeffnen einer Notiz aus der Karte erscheint eine "Zurueck zur Karte"-Schaltflaeche in der Tab-Leiste. Klicken Sie darauf, um genau dorthin zurueckzukehren — gleiche Drill-Down-Ebene beibehalten.

---

## 22. Kognitive Engine

Die Kognitive Engine ist das eingebaute Intelligenzsystem von Constellation, das Ihre Notizen analysiert und verborgene Muster und Zusammenhaenge zwischen Ihren Ideen aufdeckt. Ihre Kernphilosophie:

> „Es kommt nicht auf die Menge Ihrer Daten an. Entscheidend ist nicht, wie viele Quellen Sie speichern, sondern wie Sie daraus Wissen formen und zu einem sinnvollen Bewusstsein verknuepfen."

Die Kognitive Engine besteht aus neun integrierten Werkzeugen: Typisierte Links, Wissensschichten, Reifelebenszyklus, Spannungsdetektor, Herkunftskette, Externalisierungsmaschine, Überprüfungspuls, Pfade und Multi-Linsen-Ansichten.

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

### 17.7 Überprüfungspuls

#### Was ist das?

Der Überprüfungspuls ist ein System zur zeitversetzten Wiedervorlage, das Notizen in wachsenden Abstaenden wieder in Ihre Aufmerksamkeit bringt: 1 Tag, dann 3, dann 7, dann 14, dann 30 Tage nach der letzten Ueberpruefung. Es ueberwacht auch Notizen mit den Tags `#assumption` oder `#model` als Denkmodell-Checkpoints und fuehrt eine "Nie ueberprueft"-Warteschlange fuer erfasste aber nie wiederbesuchte Notizen.

#### Warum ist das wichtig?

Wissen verfaellt ohne Wiederholung. Sie schreiben heute eine Notiz und haben in drei Wochen vergessen, dass sie existiert. Verteilte Wiederholung ist die am besten belegte Technik der Kognitionswissenschaft gegen diesen Verfall. Der Überprüfungspuls wendet dieses Prinzip auf Ihre tatsaechlichen Notizen an.

#### So verwenden Sie es

1. Klicken Sie auf den **Überprüfungspuls**-Tab in der linken Seitenleiste. Sie sehen drei Bereiche: Faellig zur Ueberpruefung, Denkmodell-Checkpoints (`#assumption` / `#model`), und Nie ueberprueft.
2. Klicken Sie auf eine Notiz, um sie zu oeffnen und durchzulesen.
3. Waehlen Sie eine der drei Aktionen:
   - **Ueberprueft** (Haekchen) — plant die naechste Ueberpruefung im naechsten Intervall (1 → 3 → 7 → 14 → 30 Tage).
   - **7 Tage zurueckstellen** (Augensymbol) — verschiebt die Notiz um 7 Tage ohne das Intervall voranzutreiben.
   - **Verwerfen** (Archivsymbol) — entfernt die Notiz dauerhaft aus der Ueberpruefungsliste.
4. Oeffnen Sie die Befehlspalette und geben Sie "Review due notes" ein.

#### Wo sehen Sie es?

- **Linke Seitenleiste**: Der Überprüfungspuls-Tab mit Badge-Zaehler fuer faellige Notizen
- **Befehlspalette**: Befehl "Review due notes" fuer schnellen Zugriff

### 17.8 Pfade

#### Was ist das?

Pfade sind benannte, geordnete Sequenzen von Notizen — wie Kapitel in einem Buch oder Stationen einer gefuehrten Tour durch Ihr Wissen. Ein Pfad wird definiert durch `trail: true` im Frontmatter einer Notiz, gefolgt von einer geordneten Liste von Wikilinks im Notiztext.

#### Warum ist das wichtig?

Wissen ist nicht immer ein Netz. Manchmal ist es ein Weg — eine Lernsequenz, eine Argumentationskette, eine Erzaehlung. Pfade erfassen diese Reihenfolge explizit und fuegen Ihrer nicht-linearen Bibliothek eine lineare Dimension hinzu.

#### So verwenden Sie es

1. Erstellen Sie eine neue Notiz mit `trail: true` im Frontmatter.
2. Listen Sie im Notiztext Wikilinks in der gewuenschten Reihenfolge auf.
3. Wenn Sie eine Notiz oeffnen, die zu einem Pfad gehoert, zeigt die Breadcrumb-Leiste einen Pfad-Indikator mit Name und Position (z.B. "Mein Pfad 2/5"). Pfeiltasten navigieren zur vorherigen und naechsten Notiz.
4. Oeffnen Sie die Befehlspalette und geben Sie "Open Trail" ein, um alle Pfade anzuzeigen.

#### Wo sehen Sie es?

- **Breadcrumb-Leiste**: Pfad-Indikator mit Name, Position und Navigationspfeilen
- **Befehlspalette**: Befehl "Open Trail" listet alle Pfade auf

### 17.9 Multi-Linsen-Ansichten

#### Was ist das?

Multi-Linsen-Ansichten ermoeglichen es, Ihre Bibliothek durch verschiedene Klassifikationsschemata zu betrachten — ohne die Ordnerstruktur zu aendern oder Notizen zu duplizieren. Eine "Linse" ist eine virtuelle Gruppierung, die Notizen basierend auf einer Eigenschaft oder einem Tag neu organisiert. Integrierte Linsen: "Nach Stadium" (Fluechtig/Literatur/Permanent/Synthese) und "Nach Thema" (Gruppierung nach Tags). Benutzerdefinierte Linsen koennen in den Einstellungen erstellt werden.

#### Warum ist das wichtig?

Ordnerstrukturen erzwingen eine einzelne Hierarchie, aber Wissen passt nicht in einen einzigen Baum. Multi-Linsen-Ansichten ermoeglichen den Wechsel zwischen verschiedenen Perspektiven, ohne Dateien zu verschieben. Dieselben Notizen, durch verschiedene organisatorische Linsen betrachtet.

#### So verwenden Sie es

1. Finden Sie im Seitenleiste das **Linsen-Dropdown** oben im Dateibaum (Standard: "Ordner").
2. Waehlen Sie eine Linse: "Nach Stadium", "Nach Thema" oder eine benutzerdefinierte Linse. Die Seitenleiste organisiert sich sofort neu.
3. Waehlen Sie "Ordner", um zum Standard-Dateibaum zurueckzukehren.
4. Um eine benutzerdefinierte Linse zu erstellen: Oeffnen Sie **Einstellungen > Wissensmanagement**, klicken Sie auf **Linse erstellen**, benennen Sie sie und waehlen Sie die Frontmatter-Eigenschaft fuer die Gruppierung.
5. Oder verwenden Sie die Befehlspalette: Geben Sie "Create Lens" ein.

#### Wo sehen Sie es?

- **Seitenleisten-Dropdown**: Linsen-Auswahl oben im Dateibaum
- **Einstellungen > Wissensmanagement**: Benutzerdefinierte Linsen erstellen, bearbeiten und loeschen
- **Befehlspalette**: Befehl "Create Lens"

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

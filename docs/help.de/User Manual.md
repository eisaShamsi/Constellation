# Constellation Benutzerhandbuch

**Version 0.1.0 | Maerz 2026**

Constellation ist eine Desktop-Anwendung fuer persoenliches Wissensmanagement (PKM) zur Verwaltung von Markdown-Notizbibliotheken. Entwickelt mit Tauri v2, SvelteKit und Rust, laeuft sie nativ auf Windows, macOS und Linux mit vollstaendiger Unterstuetzung fuer Arabisch und RTL.

---

## Inhaltsverzeichnis

1. [Erste Schritte](#erste-schritte)
2. [Universum und Bibliotheken](#universum-und-bibliotheken)
2b. [Der Datei-Explorer](#2b-der-datei-explorer)
3. [Notizen erstellen und bearbeiten](#notizen-erstellen-und-bearbeiten)
4. [Suche](#suche)
5. [Sternenansicht (GraphMind)](#sternenansicht-graphmind)
6. [Geteilte Ansicht](#geteilte-ansicht)
7. [Index](#index)
8. [Constellation Sight](#constellation-sight)
9. [Zweiter Bildschirm](#zweiter-bildschirm)
10. [Eigenschaften und Frontmatter](#eigenschaften-und-frontmatter)
10b. [Quellen-Prüfung (CECE)](#10b-quellen-prüfung-constellation-epistemic-content-engine--cece)
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

### Synchronisierung und externe Änderungen

Constellation folgt dem Prinzip **File Over App** — Ihre Notizen sind schlichte `.md`-Dateien auf der Festplatte, und die App überwacht sie auf Änderungen. Wenn eine Notiz von *außerhalb* von Constellation hinzukommt oder sich ändert, während die App geöffnet ist — eine Obsidian-Synchronisierung von einem anderen Gerät, ein `git pull`, ein Cloud-Synchronisierungsdienst (iCloud / Syncthing / OneDrive) oder eine Datei, die Sie in einen Bibliotheksordner ablegen —, erkennt Constellation dies **automatisch**, innerhalb von etwa einer Sekunde und **ohne Neustart**:

- Die Notiz erscheint im **Dateibaum**.
- Sie wird auffindbar über den **Sternensprung** (`Ctrl+O`), die **Suche**, den **Index**, **Rückverweise** und die **Notizanzahl** der Bibliothek — alles aktualisiert sich von selbst.
- Wenn Sie einen Ordner außerhalb der App umbenennen, bleiben seine Notizen am neuen Ort auffindbar und die alten Einträge werden bereinigt.
- Ein großer Schwung (ein `git pull` vieler Notizen oder eine erste Synchronisierung) wird im Hintergrund indiziert — das Tippen bleibt verzögerungsfrei, während die Suche nachzieht.

Sie müssen nichts tun: Constellation hält seinen Suchindex im Gleichschritt mit Ihren Dateien, während sich diese auf der Festplatte ändern. *(Ein Detail: Das Umbenennen eines Ordners von **außerhalb** der App setzt den Wiedervorlage-Zeitplan und den Link-Gewichtungsverlauf dieser Notizen zurück — der Notiztext selbst bleibt unberührt. Das Umbenennen von Ordnern **innerhalb** von Constellation bewahrt hingegen alles.)*

**Ist die geänderte Notiz gerade in einem Tab GEÖFFNET**, bringt Constellation sie sicher auf den neuesten Stand — Ihre Arbeit wird niemals unbemerkt überschrieben:

- Wenn Sie in dieser Notiz **keine ungespeicherten Änderungen** haben, aktualisiert sich die geöffnete Notiz unauffällig und zeigt die externe Bearbeitung, sodass Ihr nächster Tastenanschlag auf der neuen Version aufbaut. *(Zuvor zeigte eine geöffnete Notiz weiterhin den alten Text, und Ihr nächster Tastenanschlag konnte die externe Bearbeitung unbemerkt überschreiben — das kann nun nicht mehr geschehen.)*
- Wenn Sie in dieser Notiz **doch ungespeicherte Änderungen** haben, während gleichzeitig eine externe Änderung eintrifft — ein echter Konflikt —, rührt Constellation Ihre ungespeicherte Arbeit niemals an. Es behält **Ihre** Version im Editor, schreibt die eingehende externe Version in eine **Nebenkopie** neben der Notiz (mit dem Namen `<note>.conflict-<timestamp>.md.txt`, sodass nie etwas verloren geht) und zeigt ein Banner: *"Eine externe Bearbeitung von {note} wurde als separate Kopie gesichert — deine Version ist unverändert."* Klicken Sie auf **Kopie anzeigen**, um den Ordner mit dieser Nebenkopie zu öffnen und sie bei Bedarf von Hand zusammenzuführen. Die Nebenkopie ist eine inaktive `.txt`-Datei — sie erscheint niemals in Ihrer Seitenleiste oder Suche und löst niemals eine weitere Synchronisierung aus.

**Die beiden Versionen zusammenführen.** Das Konflikt-Banner besitzt außerdem eine Schaltfläche **Zusammenführen…**. Sie öffnet eine zweispaltige Vollbildansicht — links **Deine Version** (bearbeitbar) neben der **Externen Kopie** rechts (schreibgeschützt) —, wobei die Unterschiede hervorgehoben und die identischen Teile eingeklappt werden. Neben jedem Unterschied befindet sich eine Schaltfläche **In deine übernehmen**, die diese externe Änderung in Ihre Version zieht; Sie können die linke Spalte auch frei bearbeiten, um beide von Hand zu kombinieren. Wenn Sie fertig sind, schreibt **Zusammengeführt speichern** Ihre abgeglichene Notiz und verschiebt die Nebenkopie in den Papierkorb der Bibliothek (wiederherstellbar, nie gelöscht); **Abbrechen** ändert nichts — beide Versionen bleiben genau so, wie sie waren. Constellation führt niemals automatisch zusammen — der Abgleich ist immer Ihre Entscheidung.

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

## 2b. Der Datei-Explorer

Der **Datei-Explorer** ist der Dateibaum in der linken Seitenleiste — Ihr Zuhause für die Dateiverwaltung. Er zeigt Ihre Bibliotheken, Ordner und Notizen genau so, wie sie auf der Festplatte liegen, und ist der eine Ort, an dem Sie Notizen anlegen, umbenennen, verschieben und löschen. Er kümmert sich ausschließlich um das Dateisystem: Er durchsucht *Namen*, niemals den Inhalt einer Notiz — die Volltextsuche im Notiztext gehört zum **Suchzentrum** (`Ctrl+Shift+F`).

Über das reine Durchstöbern hinaus bietet der Datei-Explorer vier Werkzeuge, die aus ihm ein vollwertiges Verwaltungswerkzeug machen: einen Filter, eine Sortierung, Mehrfachauswahl und eine Stapelleiste.

### Filtern nach Namen

Über dem Baum steht ein **Filterfeld**. Tippen Sie einige Zeichen ein, und der Baum zeigt nur noch die Notizen und Ordner, deren *Name* auf das Getippte passt; die übergeordneten Ordner bleiben sichtbar, damit Sie erkennen, wo ein Treffer liegt.

- Der Filter durchsucht **alle Bibliotheken** — auch die eingeklappten. Eine eingeklappte Bibliothek wird geladen und aufgeklappt, um ihre Treffer zu zeigen; sobald Sie den Filter löschen, wird der vorherige Aufklappzustand exakt wiederhergestellt.
- Der Filter passt **nur auf Namen**, nie auf den Notizinhalt. Für die Suche im Text einer Notiz verwenden Sie das Suchzentrum.
- Das Filtern funktioniert in jeder Schrift — tippen Sie Arabisch, um arabische Namen zu finden.

### Sortieren

Ein **Sortier-Umschalter** über dem Baum durchläuft acht Ordnungen. Ordner stehen dabei immer oben, die Notizen darunter werden in der gewählten Reihenfolge angezeigt:

- **Name** — A→Z oder Z→A
- **Geändert** — neueste oder älteste zuerst
- **Erstellt** — neueste oder älteste zuerst
- **Größe** — größte oder kleinste zuerst

### Mehrfachauswahl

Sie können mehrere Notizen und Ordner auf einmal auswählen:

- **Strg-Klick** (⌘-Klick auf dem Mac) schaltet eine einzelne Notiz oder einen Ordner an oder ab.
- **Umschalt-Klick** wählt einen zusammenhängenden Bereich zwischen der zuletzt angeklickten und der jetzt angeklickten Zeile aus.
- Ein **einfacher Klick** öffnet die Notiz weiterhin wie gewohnt (bzw. klappt einen Ordner auf oder zu).
- **Escape** hebt die Auswahl wieder auf.

Ausgewählte Zeilen werden hervorgehoben, sodass Sie jederzeit sehen, was Teil Ihrer Auswahl ist.

### Die Stapelleiste

Sobald mindestens ein Element ausgewählt ist, erscheint eine **Stapelleiste**. Sie zeigt die Anzahl der ausgewählten Elemente und bietet drei Aktionen, die auf die gesamte Auswahl auf einmal wirken:

- **Tag hinzufügen** — vergibt einen Tag an alle ausgewählten Notizen.
- **Verschieben** — verschiebt die Auswahl in einen anderen Ordner.
- **Löschen** — verschiebt die Auswahl in den Papierkorb (nichts wird endgültig gelöscht; jede Löschung ist umkehrbar).

Jede Stapelaktion läuft über genau dieselben sicheren, abgesicherten Abläufe, die auch eine einzelne Notiz verwendet — dieselben Schutzmechanismen, nur auf viele Notizen zugleich angewandt. Notizen, die schreibgeschützt aus einem verlinkten Kind-Universum stammen, werden dabei übersprungen.

---

## 3. Notizen erstellen und bearbeiten

### Eine Notiz erstellen

| Methode | Aktion |
|---------|--------|
| **Tastatur** | `Ctrl+N` |
| **Dateibaum** | Rechtsklick auf einen Ordner > Neue Notiz |
| **Mission Control** | `Ctrl+P` > "Neue Notiz" |

### Ihre Tabs kehren beim Neustart zurück

Bisher vergaß Constellation beim Schließen, welche Notizen geöffnet waren — jeder Start begann leer. Jetzt merkt sich die App Ihre offenen Tabs, welcher davon aktiv war und ob das Fenster geteilt war, und stellt all das beim nächsten Start automatisch wieder her. Ihr Schreibtisch sieht so aus, wie Sie ihn verlassen haben.

- Die Erinnerung gilt **pro Universum** und aktualisiert sich unauffällig etwa eine Sekunde, nachdem Sie Tabs öffnen, schließen oder umordnen. Bei einem Absturz oder erzwungenen Beenden geht höchstens die letzte Sekunde der *Anordnung* verloren — niemals Notizinhalt (die Inhaltssicherung ist ein eigener, älterer Mechanismus).
- Eine Notiz, die verschoben oder gelöscht wurde, während die App geschlossen war, wird einfach übersprungen; die übrigen Tabs kehren trotzdem zurück.
- Zum Ausschalten: **Einstellungen → Editor → Tabs beim Neustart wiederherstellen**. Beim Ausschalten wird die gespeicherte Sitzung ebenfalls gelöscht — aus bedeutet *nicht mehr merken*.
- Benannte **Arbeitsbereiche** bleiben unberührt: Sie sind weiterhin Ihre bewussten, von Hand gespeicherten Schnappschüsse. Diese Funktion ist lediglich der fortlaufende „letzte Stand".
- Bekannte Einschränkung: Bei einer geteilten Ansicht kehrt die Teilung selbst zurück, aber welche Tabs in welcher Hälfte lagen, wird noch nicht gemerkt.

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

#### Callouts anpassen — Farben, Symbole und eigene Typen

Die Farben und Symbole der Callouts gehoeren Ihnen, und Sie koennen eigene Callout-Typen erfinden. Oeffnen Sie den Style Setter (die Schaltflaeche 🎨 im Dock), waehlen Sie die Kategorie Editor und klicken Sie dann auf Callouts. In der Mitte oeffnet sich eine einzige Callouts-Verwaltung, in der jeder Callout eine Zeile ist, die seine Farbe, sein Symbol und seinen Namen zeigt.

- Einen integrierten Callout umfaerben. Klicken Sie auf das Farbfeld in seiner Zeile. Eine kleine Palette oeffnet sich mit Ihren gespeicherten Farben (eine anklicken, um sie anzuwenden) sowie einer "Benutzerdefiniert…"-Auswahl fuer jede neue Farbe — eine von Ihnen gewaehlte Farbe wird zudem fuer das naechste Mal zu Ihrer Palette hinzugefuegt. Farbaenderungen an den integrierten Typen werden gespeichert, wenn Sie im Style Setter auf Behalten/Anwenden druecken.
- Das Symbol eines integrierten Callouts aendern. Klicken Sie auf das Symbol in seiner Zeile. Die Emoji- & Symbolbibliothek oeffnet sich — waehlen Sie ein beliebiges Emoji oder Vektorsymbol. Es aendert sich sofort ueberall, in der Farbe dieses Callouts. Ein kleines ↺ erscheint, sodass Sie nur dieses Symbol zuruecksetzen koennen.
- Die integrierten Typen zuruecksetzen. Die Schaltflaeche "↺ Dieses Element zuruecksetzen" oben in der Verwaltung setzt alle Farben und Symbole der integrierten Callouts auf ihre Standardwerte zurueck. (Ihre benutzerdefinierten Callouts bleiben unberuehrt — entfernen Sie diese einzeln.)
- Einen eigenen Callout-Typ erstellen. Unter dem Trennstrich befindet sich die Hinzufuegen-Zeile. Geben Sie einen Namen ein (z. B. Decision oder فكرة), ein Ausloeser-Wort (das [!word], das Sie tippen werden — jede Sprache funktioniert, auch Arabisch), waehlen Sie eine Farbe und ein Symbol und klicken Sie auf Hinzufuegen. Wenn Sie nun in einer beliebigen Notiz > [!decision] (oder > [!فكرة]) tippen, wird Ihr Callout gerendert. Wenn Sie nach dem Ausloeser keinen Titel eingeben, zeigt die Callout-Kopfzeile den Namen Ihres Callouts in Fettschrift.
- Einen benutzerdefinierten Callout bearbeiten oder entfernen. Verwenden Sie ✎ (Name/Ausloeser bearbeiten) und ✕ (entfernen) in seiner Zeile. Beim Entfernen eines Typs bleibt der [!…]-Text in Ihren Notizen unberuehrt — er kehrt einfach zum schlichten Notiz-Aussehen zurueck, bis Sie den Typ neu erstellen.

Ihre benutzerdefinierten Callouts, Farben und Symbole werden mit diesem Universum gespeichert, sodass sie mit Ihrer Library mitreisen.

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

### Speichern und Wiederherstellung

Constellation **speichert automatisch** waehrend der Eingabe — es gibt keine Speichern-Schaltflaeche. Ihre Aenderungen werden einen Moment nach dem Innehalten in die `.md`-Datei geschrieben (und immer dann, wenn Sie die Notiz wechseln oder einen Tab schliessen). Eine Notiz gilt erst dann als "gespeichert", wenn sie tatsaechlich auf die Festplatte geschrieben wurde.

Sollte ein Speichern einmal **fehlschlagen** — zum Beispiel weil ein Synchronisierungsdienst (iCloud / OneDrive / Syncthing) oder ein Virenscanner die Datei kurz sperrt —, geht Ihre Arbeit **nicht** verloren:

- Oben erscheint ein Banner: *"{note} konnte nicht gespeichert werden — deine Änderung ist sicher und wird erneut versucht."* Ihre Eingabe bleibt auf dem Bildschirm und wird sicher im Speicher gehalten (sowie in einem Wiederherstellungspuffer, der einen Neustart uebersteht).
- Constellation **versucht es automatisch alle paar Sekunden erneut**, sodass Ihre Aenderung von selbst geschrieben wird, sobald die Datei wieder frei ist — selbst wenn Sie sich zwischenzeitlich entfernt haben.
- Sie koennen auch auf **Erneut versuchen** im Banner klicken, um sofort zu speichern. Das Banner verschwindet, sobald die Notiz gespeichert ist.

Sie muessen sich nie Sorgen machen, dass eine gesperrte oder kurzzeitig nicht verfuegbare Datei Sie eine Aenderung kostet.

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

### 8a. Tradition-Felder pro Notiz (MIG-029)

Der Tradition-Chip oben links in Sight ermoeglicht es Ihnen, die Kuppel durch 24 wissenschaftliche Traditionen in 10 epistemischen Familien neu zu rahmen. Fuer neun dieser Traditionen (die mit sektor-, konzentrischen oder leiterfoermigen Formen) kann jede Notiz **explizit klassifiziert** werden ueber ein Feld im Frontmatter. Notizen ohne dieses Feld landen in einem sinnvollen, traditions-spezifischen Standard-Bucket; Notizen MIT dem Feld landen im Bucket, den Sie benannt haben.

Fuegen Sie das Feld zum YAML-Frontmatter einer Notiz hinzu:

```yaml
---
masadir_source: sunnah
---
```

Wechseln Sie zum Chip dieser Tradition → Ihre Notiz landet in deren benanntem Sektor statt im Standard.

**Erlaubte Felder und Werte:**

| Tradition | Frontmatter-Feld | Erlaubte Werte | Standard bei Abwesenheit |
|---|---|---|---|
| **masādir** (sunnitisches uṣūl al-fiqh) | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` |
| **pramāṇa** (indisches Nyāya) | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` |
| **Ibn Rushd burhān** | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `shir` (aeusserster Ring) |
| **PaRDeS** (juedische Hermeneutik) | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` |
| **Peirce** (3 phaneroskopische Kategorien) | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` |
| **Habermas** (3 Erkenntnisinteressen) | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` |
| **Menzianische Keime** (4 moralische Keime) | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` |
| **Mohistische sān biǎo** (3 Standards) | `mohist_zone` | `ben` / `yuan` / `yong` | per Hash auf 3 Zonen verteilt |
| **Koreanisches Sŏngnihak** (Vier-Sieben-Debatte) | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` |

**Verhalten:**
- Wenn Sie einen Wert schreiben, den die Tradition nicht erkennt (Tippfehler oder erfunden), landet die Notiz im Standard-Bucket. Kein Absturz, kein Rendering-Fehler.
- Frontmatter-Aenderungen propagieren automatisch — speichern Sie die Notiz → das naechste Rendern der Kuppel spiegelt die Aenderung wider.
- Dasselbe Feld wird nur von seiner benannten Tradition gelesen. `masadir_source: sunnah` auf einer Notiz hat keinen Effekt, wenn Sie zu PaRDeS oder Peirce wechseln — jede Tradition liest ihr eigenes Feld unabhaengig.
- Dies ist der explizitste Weg, die raeumliche Grammatik der Kuppel zu kontrollieren. Ohne diese Felder ist die Geometrie korrekt, aber jede Notiz landet standardmaessig im selben Bucket; mit ihnen wird der Chip analytisch aussagekraeftig.

**Traditionen ohne Felder pro Notiz** (bucketisieren derzeit alle Sterne mit anderen Mitteln — Ordner / Bibliothek / Hash):

- Aristotelisch (Standard, keine Neuzuordnung)
- Polanyi (Gradienten-Nebel; keine Sektorisierung)
- Husserl, Longino, Shāṭibī maqāṣid, Maimonidische Prophetie, Talmudische 13 middot, Wang Yangming, Mignolo pluriversal, Dussel Transmoderne, Maldonado-Torres, Akan Wiredu, Ibn Khaldūn ʿumrān, Ibuanyidanda

(Zukuenftige Migrationen koennen Frontmatter-Felder pro Notiz fuer diese hinzufuegen, sobald Nutzerbedarf aufkommt.)

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

## 10b. Quellen-Prüfung (Constellation Epistemic Content Engine — CECE)

> *(Übersetzungshinweis: KI-generierte Übersetzung des Kapitels V3-§10.F; muttersprachliche Überprüfung steht aus.)*

Zwei der wichtigsten Frontmatter-Eigenschaften — `sources:` und `content_type:` — beschreiben, *wie du etwas erfahren hast* und *welche Art von Wissen* es ist. Constellations **Epistemic Content Engine** (CECE) klassifiziert jede Notiz entlang dieser beiden Achsen automatisch mithilfe eines Ensembles aus 6 Katalogisierern. Das Panel **Quellen-Prüfung** ist der Ort, an dem du diese Klassifizierungen prüfst und korrigierst.

### Was die Engine tut

Wenn du eine Notiz klassifizierst (Rechtsklick → „Quellen & Inhaltstyp vorschlagen", oder über Einstellungen > Scan ausführen, oder automatisch über den Hintergrund-Scan-Schalter), führt CECE sechs unabhängige Katalogisierer gegen die Notiz aus. Jeder liest die Notiz durch eine andere Linse und stimmt über zwei Fragen ab:

- **Quelle** (horizontale Achse) — woher *kam* dieses Wissen? Elf mögliche Werte: Wahrnehmung, Schlussfolgerung, Zeugenaussage, Massenüberlieferung, Vergleich, Postulation, Nicht-Erfassung, Erinnerung, angeborene Veranlagung, Inspiration, Offenbarung. Plus *unklassifizierbar*.
- **Inhaltstyp** (vertikale Achse) — welche *Art* von Wissen ist dies? Fünf Hauptzweige: sensorische Eingaben, symbolische Entitäten, semantische Inhalte, epistemische Zustände, Konstrukte höherer Ordnung.

Die beiden Achsen sind unabhängig. Eine Notiz „Ich zweifle an der Mondlandung" ist Zeugenaussage (jemand hat darüber berichtet) auf der Quellenachse + epistemische Zustände/Zweifel (deine Haltung) auf der Inhaltstyp-Achse.

Die Engine läuft **auf deinem Gerät** — keine Notiz verlässt jemals Constellation.

### Die sechs Katalogisierer

Jeder Katalogisierer ist eine Linse. Die Quellen-Prüfungs-Karte zeigt sie als sechs kleine farbige Punkte oben rechts an jeder Karte:

- **Dein Frontmatter** (blau) — übernimmt das, was du bereits gesetzt hast, mit absoluter Autorität
- **Zitate & Struktur** (rosa) — Zitate, Blockzitate, Theorem-Marker, Definitionsformulierungen
- **Wortstämme & Lexikon** (bernstein) — arabische Wurzelanalyse + sprachübergreifende Begriffsäquivalenz
- **Verknüpfte Notizen** (türkis) — typisierte Living Links zu anderen klassifizierten Notizen
- **Ähnliche Notizen** (violett) — Embedding-Ähnlichkeit zu deinen bereits klassifizierten Notizen
- **KI-Urteil** (grün) — ein lokales LLM (Qwen3-4B; *noch nicht aktiv*, auf eine zukünftige Version verschoben)

Ein gefüllter Punkt bedeutet, dass dieser Katalogisierer sich geäußert und der Synthese zustimmt. Ein umrandeter Punkt bedeutet, dass er sich geäußert hat, aber widersprochen hat. Ein gestrichelter Punkt bedeutet, dass er stumm geblieben ist (kein Signal in dieser Linse).

### Drei Vertrauens-Regime

Nachdem die Katalogisierer abgestimmt haben, landet jede Achse in einem von drei Regimen:

- **Einstimmig** — jeder sich äußernde Katalogisierer war einverstanden
- **Starke Mehrheit (eine Gegenstimme)** — die meisten waren einverstanden; ein Andersdenkender benannt
- **Uneinig** — keine klare Mehrheit; die Engine weigert sich zu raten und bittet dich zu wählen

Jede Achse erhält ihr eigenes Regime unabhängig — eine Karte kann horizontal Einstimmig + vertikal Uneinig sein, etc.

### Sibling Disambiguation

Wenn eine Achse Uneinig ist, präsentiert die Engine die Kandidatenwerte als **Chips** unter einer Eingabeaufforderung: *„Wähle, welcher am besten zur Notiz passt."* Klicke auf einen Chip → die Engine schreibt diese Wahl in das Frontmatter der Notiz und entfernt die Karte aus der Warteschlange. Wenn die ANDERE Achse bereits geklärt war (Einstimmig oder Starke Mehrheit), schreibt die Engine *gleichzeitig* auch den Wert dieser Achse — ein Klick beendet beide Achsen, wenn nur eine Uneinig war.

### Die Begründungs-Spur

Jede Karte hat einen Schalter *„▸ Warum diese Klassifizierung?"*. Beim Aufklappen wird eine Zeile pro sich äußerndem Katalogisierer angezeigt — mit Begründung, selbst gemeldetem Vertrauen und freundlichen Regel-Chips („Oberflächliche Stichwortübereinstimmung", „Arabische Wurzelübereinstimmung (CAE)", „Definitionsmarker" usw.) — das sind die spezifischen Regeln, die jeder Katalogisierer ausgelöst hat.

Während deiner **ersten 50 Prüfungen** wird die Spur auf jeder Karte automatisch ausgeklappt (eine *Vertrauenskalibrierungs-Phase*), damit du ein Gefühl dafür bekommst, wann du der Engine vertrauen kannst. Danach werden die Spuren auf Einstimmigen Karten zu Auf-Anfrage-Anzeige zusammengeklappt. Jederzeit überschreibbar in **Einstellungen > Intelligenz > CECE > Sichtbarkeit der Begründungs-Spur**.

### Der Warteschlangen-Zusammensetzungsfilter

Über der Zählleiste schneiden fünf Chips die Warteschlange nach der Art der Entscheidung, die jede Karte benötigt:

- **Alle** — die volle Warteschlange
- **Beide Achsen brauchen deine Entscheidung** — beide Achsen Uneinig
- **Quelle braucht deine Entscheidung** — horizontal Uneinig + vertikal geklärt
- **Inhaltstyp braucht deine Entscheidung** — vertikal Uneinig + horizontal geklärt
- **Katalogisierer waren sich einig** — keine Achse Uneinig (Stempel-Kandidaten)

Jeder Chip zeigt seine Eimer-Anzahl. Der Filter ist ein Render-Schicht-Slicer — die Mathematik von Alle Akzeptieren operiert immer auf der vollen Warteschlange, unabhängig davon, welcher Filter aktiv ist.

### Aktionen pro Karte

- **Akzeptieren** — schreibe die Synthese der Engine als primären Wert auf beiden Achsen; entferne die Karte. Aktualisiert die Zuverlässigkeit pro Katalogisierer.
- **Bearbeiten** — öffnet einen Baum-Picker für beide Achsen; manuell auswählen. Gleiche Zuverlässigkeitsaktualisierung.
- **Ablehnen** — entfernt die Karte ohne zu schreiben.
- **Sibling-Disambiguation-Chip** — nur auf Uneinigen Karten.

### Bibliotheksbezogene Kalibrierung

**Einstellungen > Intelligenz > CECE > Bibliotheksbezogene Kalibrierung** öffnet eine schreibgeschützte Tabelle, die die Genauigkeit jedes Katalogisierers pro Achse in der aktiven Bibliothek anzeigt. Verschiedene Bibliotheken haben unterschiedliche Genauigkeiten pro Katalogisierer — Linguistik glänzt bei arabisch-lastigen Bibliotheken, Graph glänzt bei dicht verlinkten. Die Synthese-Schicht verwendet diese Kalibrierungsdaten, um Stimmen zu gewichten.

Ein Katalogisierer braucht **20 Korrekturen**, bevor sein Genauigkeitsverhältnis angezeigt wird. Unterhalb dieser Schwelle steht das Label *„(gleichmäßig)"* — der Katalogisierer trägt gleichmäßig gewichtete Stimmen bei, bis genug Daten vorliegen.

### Hintergrund-Klassifizierung

Standardmäßig klassifiziert CECE Notizen nur, wenn du es darum bittest (Rechtsklick oder Einstellungen-Scan-Schaltfläche). Du kannst die automatische Klassifizierung in **Einstellungen > Intelligenz > CECE > Hintergrund-Klassifizierung** aktivieren:

- **Beim Speichern der Notiz** — klassifiziere jede Notiz ~1,5 Sekunden nachdem du aufhörst zu tippen (reitet auf dem bestehenden debounced Speichern; wird nie pro Tastendruck ausgelöst; Tippen bleibt sofort)
- **Beim Programmstart** — scanne unklassifizierte Notizen einmal pro Start

### Der Klassifikator — die Vollfenster-Startseite

Dieselben Karten leben auch in einer Vollfenster-Ansicht namens **Klassifikator**, geöffnet über das **Symbol mit den gestapelten Karten im Dock am linken Rand**. Es ist dieselbe Engine und dieselbe Warteschlange, ihr aber das ganze Fenster statt eines schmalen Seitenleisten-Tabs gegeben — und es fügt zwei Steuerelemente hinzu, die der Seitenleisten-Tab nie hatte:

- **Notiz klassifizieren…** — ein Suchfeld, mit dem du *jede* Notiz nach Namen klassifizieren kannst, ohne sie zuerst zu öffnen. Tippe ein paar Buchstaben, wähle die Notiz, und eine frische Karte erscheint in der Warteschlange.
- **Alle Zusammenfassungen erstellen** — berechnet die Notizzusammenfassung (siehe unten) für jede Notiz, die keine hat, im Hintergrund vor, mit Fortschritt in der Statusleiste.

Eine Schaltfläche **Scan starten** (derselbe universumsweite Scan wie in den Einstellungen) und ein Live-Fortschrittsstreifen runden den Header ab. Schließe den Klassifikator mit **(×)** oder **Esc**. (Wenn das Suchfeld *Notiz klassifizieren…* geöffnet ist, schließt das erste **Esc** nur dieses Feld.)

Ein Hinweis zur Benennung: **der Klassifikator** ist der *Raum* (die Vollfenster-Ansicht); **die Katalogisierer** sind die *sechs Linsen* innerhalb der Engine, die über jede Karte abstimmen. Verwechsle die beiden nicht.

### Notizzusammenfassungen

Unter dem Titel jeder Karte sitzt eine kurze **Zusammenfassung** — ein paar Sätze, die dir sagen, worum es in der Notiz geht, sodass du sie klassifizieren kannst, ohne sie zu öffnen. Constellation bevorzugt immer eine Zusammenfassung, die *du* geschrieben hast, und generiert nur dann eine, wenn du es nicht getan hast:

1. Ein **Frontmatter-Feld** `summary:` / `description:` / `abstract:` / `excerpt:`, wortgetreu verwendet.
2. Ein **Callout** `> [!summary]` / `[!abstract]` / `[!tldr]` im Text, wortgetreu verwendet.
3. Andernfalls eine **generierte** Zusammenfassung — die drei zentralsten Sätze der Notiz, extrahiert (nie erfunden) und in der ursprünglichen Reihenfolge angezeigt.

Generierte Zusammenfassungen sind **schreibgeschützt** — Constellation schreibt niemals eine in deine Notiz zurück (File-Over-App), und alles wird **auf deinem Gerät** berechnet. Wenn du möchtest, dass eine Zusammenfassung in der Datei lebt, schreibe selbst eine, und Constellation zeigt stattdessen deine an.

Für tiefere Details (jeder Punkt-Status, jeder Regel-Chip, Klick-für-Klick-Anleitungen) siehe die Themen **Quellen-Prüfung**, **The Cataloger** und **Note Summaries** im Hilfesystem.

---

## 10c. Epistemische Metadaten

Eine kleine Reihe optionaler Frontmatter-Felder zum Festhalten reichhaltigerer Informationen darüber, wie das Wissen einer Notiz erworben wurde, wer die Position vertritt, welcher Disziplin sie angehört und wann du deine Sicht zuletzt überarbeitet hast. In MIG-022 §A als Reaktion auf die Lückenanalyse (`docs/epistemic-content-gap-analysis.md`) hinzugefügt.

Diese Felder sind **alle optional**. Notizen ohne sie funktionieren unverändert.

### Schnellreferenz

| Feld | Typ | Zweck |
|---|---|---|
| `held_by` | text | Wessen Position ist das? (Standard: `user`; kann `"al-Shāfiʿī"`, `"Ḥanafī"` usw. sein) |
| `domain` | list | Disziplinäre Tags für Abruf (`[fiqh, ʿibādāt]`) |
| `function` | text | Wozu diese Notiz dient (`reference` / `seed` / `actionable` / `shipped`) |
| `provenance_civilization` | text | Traditionsvokabular (`sunni-usuli` / `analytic-western` / `nyaya` / usw.) |
| `updated_at` | date | Wann du deine Sicht zuletzt bewusst überarbeitet hast (im Unterschied zur Dateisystem-mtime) |
| `ikhtilāf` | list of objects | Strukturierte gelehrte Meinungsverschiedenheit (`[{school, position}, ...]`) |
| `warrant` | text | Stufenbezeichnung (geparst, aber inert, bis der Warrant Research Workstream ausgeliefert wird) |
| `warrant_notes` | text | Freitext zur Stützung der Berechtigungsstufe (ebenfalls inert) |

### Wie sie im Properties-Panel erscheinen

Jedes Feld wird mit dem typgerechten Editor gerendert:
- Textfelder → Texteingabe
- `domain` → Tag-Liste (Enter zum Hinzufügen, × zum Entfernen)
- `updated_at` → Datumsauswahl
- **`ikhtilāf` → eigenes Widget** mit zwei nebeneinanderliegenden Eingaben pro Zeile (school + position) plus einer Entfernen-Schaltfläche pro Zeile und einer „Schule hinzufügen"-Schaltfläche unten. Das Widget liest aus dem strukturierten YAML und schreibt in dieses zurück, sodass Roundtrips jedes Feld bewahren.

### Was ist mit `supersedes`?

`supersedes` ist eine *Beziehung zwischen Notizen* (diese Notiz ersetzt eine frühere), keine Eigenschaft einer einzelnen Notiz. Constellation behandelt es als **typisierten Link**, nicht als YAML-Skalar:

```markdown
Dies ersetzt meine frühere Analyse: [[old-note-id|supersedes]]
```

Das `|supersedes`-Suffix am Wikilink macht ihn zu einem typisierten Link der Art `supersedes` — eigene schiefer-blaugraue Pille, erscheint in den Backlinks- + Outgoing-Links-Panels, nimmt an der Living Link Architecture teil.

### Was das NICHT ist

Die neuen Felder sind **Schema** — ein anerkanntes Vokabular, das du ausfüllen kannst. CECE konsumiert sie derzeit nicht für die Klassifizierung. Zukünftige MIGs (Warrant Research Workstream, MIG-023 zeitliche Achse) liefern Funktionen aus, die `warrant`, `updated_at` und Verwandte lesen.

Für tiefere Details + ein durchgearbeitetes Beispiel siehe das Thema **Epistemische Metadaten** im Hilfesystem.

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

Der **Kalender** ist eine ganzseitige Monatsansicht, die über das **linke Dock** (das Kalendersymbol) geöffnet wird. Tage mit Notizen oder fälligen Aufgaben sind mit farbigen **Punkten** markiert. Die Kopfzeile zeigt den Monat in Ihrem gewählten Kalender; wenn Sie einen **Zweitkalender** eingestellt haben, zeigt ein Untertitel darunter den entsprechenden Zeitraum dieses Kalenders (z. B. zeigt ein gregorianischer Monat seine Hidschri-Spanne, „Dhul-Hidscha 1447 – Muharram 1448 AH").

**Einen Tag anklicken.** Jede Tageszelle ist interaktiv:

- **Klick auf die leere Fläche (oder die Tageszahl)** → öffnet (oder erstellt) die **Tagesnotiz** dieses Tages. Ein Klick auf ein Datum, das bereits eine Tagesnotiz hat, **öffnet** sie einfach — es entsteht nie ein Duplikat.
- **Klick auf einen Punkt** → öffnet genau dieses Element. Ein **goldener** Punkt ist die Tagesnotiz; ein **violetter** Punkt ist eine andere an diesem Tag bearbeitete Notiz; ein **roter** Punkt ist eine an diesem Tag fällige Aufgabe. (Die Farben sind im Style Setter → Kalender anpassbar.) Hat ein Tag mehrere Notizen oder Aufgaben, zeigt ein Klick auf den Punkt eine kleine **Liste** zur Auswahl.
- **Klick auf einen Aufgabenpunkt** → öffnet die Notiz **bis zur Zeile dieser Aufgabe gescrollt**, bereit zum Bearbeiten. In der Aufgabenliste können Sie außerdem **das Kontrollkästchen einer Aufgabe abhaken, um sie abzuschließen** — direkt aus dem Kalender heraus; abgeschlossene Aufgaben verschwinden sofort. Nur Aufgaben, die ihr eigenes `📅 YYYY-MM-DD` tragen, erscheinen im Kalender (das Datum platziert sie auf einem bestimmten Tag).

**Kulturkalender (acht).** In **Einstellungen → Kalender** können Sie das **Kalendersystem** festlegen — **Gregorianisch, Hidschri (Islamisch), Solar-Hidschri (Persisch), Hebräisch, Indisch (Saka), Buddhistisch, Chinesisch oder Koreanisch** — und das gesamte Monatsraster wechselt darauf um, wobei in jeder Zelle sowohl das Datum des gewählten Kalenders (groß) als auch das gregorianische Datum (klein) angezeigt wird, dazu die Mondphase. Jede Monatskopfzeile zeigt den **Namen des Monats, seine Nummer in Klammern und das Jahr** — die Nummer hilft bei Kalendern, deren Monatsreihenfolge ungewohnt ist. Der **chinesische und der koreanische** Kalender sind *lunisolar*: Sie fügen mitunter einen **Schaltmonat** (闰六月 / 윤6월) ein, den der Kalender als eigene Seite darstellt, sodass die Navigation ihn nie überspringt oder doppelt zeigt. Der Hidschri-Kalender nutzt eine präzise astronomische Engine; heilige Monate werden hervorgehoben und islamische Ereignisse markiert. Sie können außerdem den **Wochenbeginn** (Sonntag/Montag) wählen und die **Wochennummern-Spalte** ein- und ausschalten.

**Optionen des Hidschri-Kalenders.** Unter **Einstellungen → Kalender → „Hidschri-Kalender (Islamisch)"** gibt es zwei zusätzliche Steuerungen:

- **Berechnungsmethode** — **Astronomisch (Mondkonjunktion)**, die dem echten Neumond folgt (am genauesten, die Voreinstellung), oder **Tabellarisch (al-Tawfīqāt al-Ilhāmiyyah)**, der klassische arithmetische Zyklus.
- **Monatskorrektur** — verschieben Sie den Beginn eines Hidschri-Monats um ±1 oder ±2 Tage, um ihn an eine **lokale Mondsichtung** anzupassen. Wählen Sie Hidschri-Jahr und -Monat, wählen Sie einen Versatz und klicken Sie auf **Setzen**; die Korrektur gilt für diesen Monat und jeden folgenden Monat. Ihre Korrekturen werden aufgelistet (jede entfernbar), mit einer Schaltfläche **Alle löschen**.

Beide Einstellungen (und Ihre Korrekturen) werden **mit Ihrem Universum** gespeichert und reisen so über Ihre Geräte hinweg mit.

**Anzeigeoptionen für Chinesisch & Koreanisch.** Korea verwendet den chinesischen Mondkalender, daher teilen beide identische Daten — was sie unterscheidet, sind die Schrift und das Jahr. Wenn einer von beiden Ihr Haupt- oder Zweitkalender ist, zeigt **Einstellungen → Kalender** zwei zusätzliche Steuerungen: eine **Jahresanzeige** (Chinesisch: der sexagesimale Zyklus 丙午年, das schlichte Jahr oder beides; Koreanisch: die **Dangi**-Ära 단기 4359, das Jahr oder der sexagesimale 병오년) und **Monatsnamen** — *native Schrift* (五月 / 5월) oder *phonetisch*, die Aussprache des Monats in Ihrer eigenen Sprache geschrieben (Deutsch „Wǔyuè / Owol"; Arabisch „وُو-يوي / أوه-وُل").

**Den Kalender gestalten.** Öffnen Sie den **Style Setter** (linkes Dock, oder **Einstellungen → Style Setter**) und wählen Sie die Oberfläche **Kalender**, um jeden Teil neu zu gestalten — jedes Element hat seine eigene **Farbe und Textgröße** (Tageszahlen, das Querverweis-Datum, die Monatspille, Wochentagsköpfe, Wochennummern, das Mondsymbol, die Heute-Hervorhebung, Gitterlinien und die Notiz-/Aufgaben-/Ereignispunkte), dazu die **Schriftart** des Kalenders. Eine lebendige Vorschau in voller Größe aktualisiert sich, während Sie bearbeiten; klicken Sie auf **Behalten**, um die Änderungen anzuwenden.

> **Dateinamen von Tagesnotizen bleiben stets gregorianisch** (`YYYY-MM-DD`), unabhängig vom angezeigten Kalender — so bleiben Ihre Dateien portabel und sortieren sich korrekt. Das kulturelle Datum wird im Kalender angezeigt (und kann im Frontmatter der Notiz festgehalten werden).

Der Kalender bedient Tagesnotizen vollständig: Klicken Sie auf einen beliebigen Tag, um ihn zu öffnen, oder führen Sie den Befehl **„Daily Note"** (Befehlspalette) aus, um zum heutigen Tag zu springen.

**Ein kulturelles Datum in einer Notiz festhalten.** Zwei optionale Werkzeuge schreiben das kulturelle Datum in die **Eigenschaften** einer Notiz (der Dateiname bleibt stets gregorianisch `YYYY-MM-DD`):

- **Hidschri-Stempel für Tagesnotizen** — *Einstellungen → Kalender → „Hidschri-Datum in Tagesnotizen stempeln."* Wenn aktiviert (nur verfügbar, solange der Hidschri-Kalender Ihr **Haupt- oder Zweitkalender** ist), erhält jede **neue** Tagesnotiz eine `hijri:`-Zeile, z. B. `hijri: 1448-01-06`. Bereits vorhandene Notizen werden nie angetastet.
- **„+ Hijri" in den Eigenschaften einer Notiz** — öffnen Sie die **Eigenschaften** einer beliebigen Notiz, fahren Sie mit der Maus über das Datum, und es erscheint eine kleine Schaltfläche **„+ Hijri"** (dazu „+ Jalali", „+ Hebrew" usw. — **eine Schaltfläche pro nicht-gregorianischem Kalender, den Sie ausgewählt haben**). Klicken Sie darauf, und Constellation liest das gregorianische Datum der Notiz und fügt das Äquivalent hinzu, z. B. `jalali: 1405-03-30`. Die koreanische Schaltfläche schreibt das **Dangi**-Jahr; ein chinesischer/koreanischer **Schaltmonat** wird mit einem `L` markiert (z. B. `chinese: 2025-06L-17`). Hat die Notiz keine Datums-Eigenschaft, wird das Erstellungsdatum der Datei verwendet.

---

## 16. Lens

Eine **Lens** ist eine gespeicherte Abfrage, die eine gefilterte und sortierte Liste von Notizen mit den gewuenschten Eigenschaften anzeigt. Constellation bietet zwei Wege:

### Constellation Base — eingebettete Lens-Bloecke

Sie koennen eine Lens direkt in den Inhalt jeder Markdown-Notiz einbetten, indem Sie einen ` ```base ` Codeblock verwenden:

````markdown
```base
schema: 1
view: list
dimensions: [note.name, note.created_at]
sort: [note.created_at, desc]
limit: 20
```
````

Beim Anzeigen der Notiz wird der Codeblock durch eine interaktive Tabelle mit passenden Notizen ersetzt. In der Live-Vorschau klicken Sie auf den **Lens**-Chip, um den Block zu bearbeiten.

**Verfuegbare Dimensionen in v1:** `note.name`, `note.path`, `note.created_at`, `note.headline`.

**Foederation:** Standardmaessig lesen Lens-Bloecke ueber das aktive Universum UND jedes verlinkte cUniverse. Setzen Sie `federation: active` im YAML, um nur das aktive Universum zu durchsuchen.

### Constellation Base — vollständiger Tab

Öffnen Sie eine `.base`-Datei, und sie füllt den Tab als interaktive Tabelle: eine Zeile pro Notiz, eine Spalte pro Eigenschaft. Über die eingebettete Form hinaus bietet der vollständige Tab drei Wege, eine Notiz in der Tabelle zu finden:

- **Diese Basis durchsuchen** — das Suchfeld in der Kopfzeile filtert die Zeilen, während Sie tippen, und passt sowohl auf den Namen einer Notiz *als auch* auf den Text jeder sichtbaren Spalte. Der Zähler neben dem Titel zeigt beim Filtern `Treffer / Gesamt` an (z. B. `4/7684`). Die Suche funktioniert in jeder Schrift — tippen Sie Arabisch, um arabische Titel zu finden. Das Filtern ist selbst bei Tausenden von Zeilen sofort da.
- **Buchstabenleiste** — bei einer Basis mit 50 oder mehr Zeilen erscheint am Rand der Tabelle ein schmaler Streifen mit Buchstaben, gebildet aus den Anfangsbuchstaben Ihrer tatsächlichen Notiztitel (also A–Z für englische Titel, أ ب ت … für arabische und die richtigen Buchstaben für jede andere Schrift). Klicken Sie auf einen Buchstaben, um direkt zur ersten Notiz zu springen, die mit ihm beginnt — ist die Tabelle noch nicht nach Namen sortiert, sortiert sie zuerst nach Namen und springt dann.
- **Rechtsklick auf eine Zeile** — öffnet das Standard-Notizmenü: Öffnen, In neuem Tab öffnen, Lesezeichen, Pfad kopieren / Namen kopieren, Im Dateibaum anzeigen, In Standard-App öffnen, Im Datei-Explorer anzeigen und Stil… (Umbenennen, Verschieben und Löschen werden hier bewusst nicht angeboten — erledigen Sie diese im Dateibaum, wo sich die Liste sicher aktualisiert.)

### Fuenf Akte (Five Acts) — eingebaute Lenses

Der Seitenleisten-Abschnitt **Five Acts** (oberhalb von Workspace Bases) listet von Constellation kuratierte Host-Notizen unter `{universe}/Five Acts/*.md`. v1 enthaelt eine: **Observation — Recent Captures** (foederierte Liste der 20 zuletzt erfassten Notizen). Sie koennen diese Notizen frei bearbeiten — Constellation ueberschreibt Ihre Aenderungen nicht.

### Klassisches Lens-Panel

Das aeltere Lens-Panel (Filterung nach Tags, Ordnern, Eigenschaften) ist weiterhin unter **Einstellungen → Panels → Lens** verfuegbar.

### Struktur (strukturelle Links)

Das **Struktur**-Panel zeigt, wo die geöffnete Notiz innerhalb eines größeren *Werks* sitzt — eines Buches, eines Drehbuchs, eines Kurses, einer Map of Content. Es beantwortet eine andere Frage als die Panels Backlinks und Outgoing Links. Diese beantworten *„wie verhält sich diese Idee zu einer anderen Idee?"* (die Denk-Links — supports, contradicts, causes …). Die Struktur beantwortet *„wo sitzt diese Notiz in dem ganzen Werk, das ich gerade verfasse?"* — Buch → Teil → Kapitel → Szene.

Dies ist das **kompositorische Rückgrat** eines Werks: das Inhaltsverzeichnis, die geordnete Gliederung. Es wird bewusst **aus** jedem Denk-, Reife- und Verbindungsmaß **herausgehalten** — eine Notiz „unter ein Buch" zu setzen ändert niemals die Reife dieser Notiz, ihre Verbindungszahlen oder ihre Präsenz in der Sternenansicht. Ein Inhaltsverzeichnis ist Autorschaft, keine zu bewertende Behauptung.

**Die zwei Arten von strukturellem Link** (Sie tippen immer nur eine Seite — Constellation ermittelt die Gegenseite für Sie):

- **`parent`** — der Platz *dieser Notiz* unter einem Elternteil (z. B. ein Kapitel gibt an, zu welchem Teil es gehört).
- **`contains`** — die geordnete Liste der Kinder *dieser Notiz* (z. B. ein Buch listet seine Teile in der Lesereihenfolge auf).

**Einen strukturellen Link anlegen** — öffnen Sie die **Eigenschaften** der Notiz (den Eigenschaften-Tab in der rechten Seitenleiste oder den Eigenschaftenblock am Anfang der Notiz):

1. Klicken Sie auf **+ Eigenschaft hinzufügen** und tippen Sie den Schlüssel `parent` oder `contains`.
2. Tippen Sie als Wert den **Namen der Zielnotiz** — nur den Namen, z. B. `Part I - The Cartographer`. Constellation verpackt ihn für Sie in einen `[[link]]`; Sie tippen die eckigen Klammern **nicht**. (Wenn Sie einen Namen einfügen, der bereits Klammern enthält, wird er dennoch sauber als einzelner `[[name]]` gespeichert — niemals doppelt verpackt.)
3. Für `contains` fügen Sie jedes Kind als eigenen Chip hinzu, in der Reihenfolge, in der sie gelesen werden sollen — diese Reihenfolge wird zur Gliederungsreihenfolge.

Strukturelle Links **überstehen Umbenennungen sicher**: Benennen Sie ein Kapitel um, und sein Platz in der Struktur folgt automatisch, weil der Link auf die Notiz selbst zeigt, nicht auf ein eingefrorenes Stück Text.

**Das Struktur-Panel lesen** — öffnen Sie den **Struktur**-Tab in der rechten Seitenleiste (direkt nach Backlinks):

- Das Panel zeigt das **ganze Werk** als eingerückte Gliederung (blaugrüne Aufzählungspunkte), überschrieben mit **OUTLINE** und einer Zählung der Nachkommen — nicht nur die eigenen Kinder der geöffneten Notiz.
- Die Notiz, die Sie gerade ansehen, ist innerhalb dieser Gliederung **hervorgehoben** („Sie sind hier").
- Ein **Breadcrumb** am oberen Rand zeigt den Pfad das Rückgrat hinauf (z. B. *The Atlas of Lost Places › Part I › Chapter 1*). Klicken Sie auf einen beliebigen Breadcrumb — oder auf eine beliebige Gliederungszeile —, um zu dieser Notiz zu springen.
- Ein Umschalter **Ganzes Werk ⇄ Diese Notiz** (oben rechts im Panel) wechselt zwischen dem gesamten Werk und nur dem eigenen Teilbaum der geöffneten Notiz. Er erscheint nur, wenn die Notiz tatsächlich einen Elternteil hat, sodass sich die beiden Ansichten unterscheiden.
- Wenn die Struktur versehentlich auf sich selbst zurückläuft (Elternteil von Notiz A ist B, und Elternteil von B ist A), zeichnet die Gliederung die Kette und stoppt dann sauber, wobei sie die Schnittstelle mit einem kleinen **↻** markiert. Sie hängt sich niemals auf.

**Einen Konflikt lösen (Umstritten).** Wenn zwei Notizen dasselbe Kind beanspruchen — eine über den eigenen `parent` des Kindes, die andere über eine `contains`-Liste —, kennzeichnet das Panel diese Zeile als **Umstritten** (ein bernsteinfarbenes ⚠-Badge, das den anderen Beanspruchenden benennt), anstatt sie stillschweigend zu verwerfen. Zwei Schaltflächen mit einem Klick lösen den Konflikt:

- **Behalten** — den eigenen deklarierten Elternteil des Kindes behalten (diese Notiz gibt ihren Anspruch auf das Kind auf).
- **Hierher verschieben** — diese Notiz als Elternteil akzeptieren (der `parent` des Kindes wechselt zu dieser Notiz).

Beide Schaltflächen aktualisieren die Notizdateien direkt und aktualisieren die Gliederung. Nichts wird jemals ohne Ihren Klick geändert.

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

### Der Style Setter

Der **Style Setter** ist ein bildschirmfüllendes Design-Studio — öffnen Sie ihn über **Einstellungen → Darstellung → "✦ Open Style Setter."** Er zeigt Ihre echte Oberfläche in der Mitte; klicken Sie auf einen beliebigen Teil (Seitenleiste, Notiztitel, Überschrift, Link, die Notizseite), und die Steuerelemente dieses Elements erscheinen rechts, während sich die Vorschau sofort aktualisiert. Themenkarten (Midnight / Daylight / Chocolate / Nord) legen einen kompletten Look an — das Studio selbst trägt ihn, während Sie gestalten — und die Liste der *Oberflächen* schaut den Look über die gesamte App hinweg vor, nicht nur im Editor. **"Apply to app"** überträgt Ihren Akzent, Ihre Hintergründe, Textfarbe und Schriftarten auf das echte Constellation; **Esc** oder **✕** schließt nur den Setter, nicht die Einstellungen. Das Anwenden ist vorerst eine Live-Vorschau für die Sitzung — das Speichern eines Looks als dauerhafter, benannter Style (mit wiederverwendbaren, umbenennbaren Farbflächen sowie Export / Import) kommt als Nächstes.

### Ueberschreibungen der Arabisch-Engine

Ein Panel pro Universum, in dem Sie festlegen, wie die Arabisch-Engine bestimmte Oberflaechenformen analysiert — Ihre eigenen Wortschoepfungen, lokale Namen, fachspezifische Lehnwoerter oder Faelle, in denen Sie der automatischen Lesart der Engine widersprechen. Jede Ueberschreibung schlaegt den generativen FST, die Kaskade und den heuristischen Fallback. Das Hinzufuegen oder Entfernen einer Ueberschreibung loest eine gezielte Neuindizierung nur fuer die Notizen aus, die die betroffene Oberflaechenform enthalten — keine Komplettneuaufbau. Siehe Abschnitt 19 ("RTL- und Arabisch-Unterstuetzung") fuer die Schritt-fuer-Schritt-Anleitung.

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

### Ueberschreibungen der Arabisch-Engine

Die Arabisch-Engine von Constellation ist ein fuenfschichtiger morphologischer Analysator, der unter jeder Suche, jedem Link und jedem Indexeintrag laeuft. Sie versteht Wurzeln, Muster, Eigennamen, Lehnwoerter und phonologische Reparaturen — so findet eine Anfrage nach كاتب auch كتبنا und كتاب, waehrend وائل als Name unversehrt bleibt und nicht zu ائل verstuemmelt wird.

Das Panel **Arabische Ueberschreibungen** in den Einstellungen ist der Ort, an dem Sie der Engine Ihre eigene Terminologie beibringen. Jede Ueberschreibung ist die souveraene Antwort — sie schlaegt den generativen FST, die Kaskade und den heuristischen Fallback.

**Wann Sie Ueberschreibungen verwenden sollten:**
- Personennamen, lokale Ortsnamen oder fachspezifische Begriffe, die die Engine nicht kennt
- Wortschoepfungen oder Abkuerzungen, die nur in Ihrem Universum vorkommen
- Lehnwoerter, bei denen Sie eine bestimmte Schreibung bewahren wollen
- Jeder Fall, in dem die automatische Analyse der Engine Ihrer Lesart widerspricht

**Schritt fuer Schritt:**

1. Oeffnen Sie **Einstellungen** (Zahnradsymbol oder `Ctrl + ,` / `Cmd + ,`) und waehlen Sie **Arabische Ueberschreibungen** in der Seitenleiste.
2. Klicken Sie auf **Ueberschreibung hinzufuegen**.
3. Tragen Sie ein:
   - **Oberflaechenform** — das arabische Wort so, wie Sie es tippen
   - **Lemma** — die kanonische Form, die die Engine zurueckgeben soll
   - **Wurzel** (optional) — 3 oder 4 Konsonanten, wenn das Wort eine klassische Wurzel hat
   - **Muster** (optional) — z.B. `فاعل`
   - **Wortart** — Eigenname / Substantiv / Adjektiv / Adverb / Verb / Partikel / Fremd / Unbekannt
   - **Notiz** (optional) — eine Zeile Kontext fuer Sie selbst
4. Klicken Sie auf **Speichern**. Das Panel zeigt **Neuindiziert…**, waehrend jede Notiz mit der Oberflaechenform neu tokenisiert wird, und dann **N Notiz(en) neu indiziert**, wenn der Vorgang abgeschlossen ist.
5. Um eine Ueberschreibung zu entfernen, klicken Sie auf das **x** in ihrer Zeile — derselbe Neuindizierungs-Durchlauf laeuft rueckwaerts.

Ueberschreibungen werden pro Universum unter `<universe>/.constellation/arabic-overrides.json` gespeichert — reiner Text, alphabetisch sortiert, atomar geschrieben. Sie koennen die Datei in die Versionskontrolle aufnehmen oder geraeteuebergreifend teilen.

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

*Constellation Benutzerhandbuch — Version 0.1.0 — Maerz 2026*
*uconstellation.world*

---

## 23. Vorgeschlagene Verknüpfungen

Constellation dient dem *Formulieren* von Wissen, und Wissen ist Verbindung. **Vorgeschlagene Verknüpfungen** findet die Notizen, die bereits in Ihrer Bibliothek liegen und mit der gerade betrachteten am engsten verwandt sind — die Verwandten, mit denen sie verknüpft sein sollte, es aber noch nicht ist — und verwandelt jede davon mit einem einzigen Klick in eine **typisierte Verknüpfung**. Es ist „mehr in dieser Art", aber fürs Denken.

**Jeder Vorschlag ist typisiert.** Wenn Sie einen annehmen, fragt Constellation, *wie* die beiden Notizen zusammenhängen — unterstützt, widerspricht, veranschaulicht, abgeleitet-von und so weiter, oder einfach **assoziativ**. Eine typisierte Verknüpfung ist ein Stück Schlussfolgerung, das Sie später lesen, durchsuchen und hinterfragen können; die Funktion fügt niemals Verknüpfungen im Stapel hinzu und legt niemals stillschweigend eine untypisierte Verknüpfung an. (Siehe **Wissensformulierung** und **Eigenschaften**.)

**Wie es sie findet.** Die Kandidaten stammen **ausschließlich aus Ihrer eigenen Bibliothek** und werden gegen den laufenden Suchindex von Constellation nach dem meisten *unterscheidungskräftigen* gemeinsamen Wortschatz gewichtet — den seltenen, aussagekräftigen Wörtern, nicht den alltäglichen. Jeder Vorschlag zeigt die **gemeinsamen Begriffe**, die erklären, warum er aufgetaucht ist, sodass Sie niemals eine Black-Box-Vermutung annehmen.

**Fünf Stellen, eine Liste.** Dieselbe Vorschlagsliste erscheint in der **Wiedervorlage** (🕐, für Notizen, die sie als *verwaist* oder *fragil* kennzeichnet), im **Backlinks-Tab** (rechte Seitenleiste), im **360°-Inspektor**, im **Zustands-Tab** und in der **Sternenansicht** (🌌 — Rechtsklick auf einen beliebigen Stern → **Verknüpfungen vorschlagen…**).

**Eingehend vs. ausgehend — und warum Sie nicht wählen.** Diagnostische Oberflächen (der **360°-Inspektor** und der **Zustands-Tab**) schlagen **eingehende** Verknüpfungen vor — *welche Notizen **hierher** zeigen sollten*. Allgemeine Oberflächen (der **Backlinks-Tab** und die **Sternenansicht**) schlagen **ausgehende** Verknüpfungen vor — *worauf diese Notiz zeigen sollte*. Die Oberfläche wählt die Richtung, die zu ihrer Aufgabe passt; Sie wählen die Notiz und den Typ. (Ein künftiges Update wird Ihnen erlauben, die Richtung selbst umzuschalten.)

**So nutzen Sie es.** Unter der Überschrift **Vorgeschlagene Verknüpfungen** sehen Sie verwandte Notizen, gewichtet mit der nächstgelegenen zuerst, jeweils mit ihren gemeinsamen Begriffen. Klicken Sie bei einem Kandidaten auf die Schaltfläche **Verknüpfen** → wählen Sie im kleinen Menü **„Wie hängen sie zusammen?"** die Beziehungsart → die typisierte Verknüpfung wird **sofort** erstellt und der Vorschlag fällt aus der Liste. Sie lebt dann in den **Eigenschaften** der Notiz und erscheint in ihren Backlinks/ausgehenden Verknüpfungen sowie im gesamten Graphen. Wenn nichts wirklich passt, lassen Sie sie stehen — oder markieren Sie die Notiz in der Wiedervorlage als bewusst **eigenständig**. Vorgeschlagene Verknüpfungen schlägt vor; Sie entscheiden.

**Lokal, privat, nicht blockierend.** Vorschläge werden auf Anfrage allein aus Ihrer Bibliothek berechnet — nichts verlässt Ihr Gerät — und ihr Zusammenstellen blockiert niemals Ihr Tippen (Sie sehen kurz „Verwandte Notizen werden gesucht…", während es arbeitet). Die Vorschläge, die Hinweise auf gemeinsame Begriffe und die Beziehungsarten erscheinen alle in Ihrer gewählten Sprache und werden für rechtsläufige Schriften korrekt gespiegelt.

---

## 24. Kognitive Farben und Rechtsklick-Menüs

### Eigenschaften-Gestaltung (Stil-Gestalter)

Öffnen Sie den **Stil-Gestalter** (Einstellungen → Darstellung → ✦ Stil-Gestalter öffnen, oder seinen eigenen Tab) und wählen Sie die Kategorie **Eigenschaften**, um die kleinen Tags im Frontmatter einer Notiz neu zu gestalten. Zwei Elemente: **Eigenschaft-Tags** (die gewöhnlichen `tags`-artigen Chips — Tag-Hintergrund, Tag-Text, Tag-Radius 0–20 px, Höhe 14–32 px) und **Taxonomie-Badges** (Hintergrund, Text, Radius 0–20 px). Eine Live-Vorschau in der Mitte aktualisiert sich, während Sie editieren; jeder Wert startet exakt mit dem heutigen Aussehen, sodass sich nichts ändert, bis Sie ein Steuerelement berühren. Klicken Sie auf **Behalten**, um für dieses Universum zu speichern.

### Kognitive Farben (Stil-Gestalter)

Die Kategorie **Kognitive Farben** gibt Ihnen **eine gemeinsame Farbe pro kognitivem Zustand**, sodass jede Oberfläche, die diesen Zustand anzeigt, übereinstimmt. Fünf Sätze:

- **Reife** — Samen, Setzling, Immergrün, Kanonisch, Welkend.
- **Konfidenz** — Hypothese, Beleg, Etabliert, Umstritten.
- **Herkunft** — Empfangen, Entdeckt, Gemischt, Keine.
- **Phase** — Funke, Geburt, Wachstum, Reife, Ruhephase, Archivierung.
- **Treffer-Kategorie** (warum ein Suchergebnis ein Treffer war) — Titel, Inhalt, Tag, Wikilink, Eigenschaft, Semantisch, Strukturiert.

Das Verhalten lautet **bei Bedarf vereinheitlichen**: nichts ändert sich, bis Sie eine Farbe wählen. Jede Oberfläche behält ihre aktuelle Farbe als Rückfallwert, und in dem Moment, in dem Sie hier die Farbe eines Zustands festlegen, schnappt **jede** Oberfläche, die diesen Zustand anzeigt — Dateibaum, Tabs, der Notiz-Inspektor, die Suchhervorhebung im Editor, das Treffer-Badge und die Hervorhebung des Suchergebnisses — auf einmal auf Ihre Farbe um. Lassen Sie einen Zustand unberührt, sieht er genau wie zuvor aus. Klicken Sie auf **Behalten**, um zu speichern.

### Rechtsklick-Menüs

Constellation gibt Ihnen an drei Stellen ein Kontextmenü, das jeweils nur die Aktionen bietet, die dort passen, wo Sie geklickt haben:

- **Rechtsklick im Notiztext** — Link einfügen / Externer Link; **Format ▸** (Fett, Kursiv, Unterstreichen, Durchgestrichen, Hervorheben, Inline-Code, Inline-Mathe, Kommentar umschalten, Hochgestellt, Tiefgestellt, Formatierung löschen); **Absatz ▸** (Aufzählung/Nummerierte/Aufgabenliste, H1–H6, Absatz, Zitat); **Einfügen ▸** (Fußnote, Tabelle, Hinweis, Horizontale Linie, Codeblock, Matheblock, Bild); Ausschneiden / Kopieren / Einfügen / Als Text einfügen / Alles auswählen; und **Stil…** (öffnet den Stil-Gestalter auf der Kategorie **Editor**).
- **Rechtsklick auf eine Frontmatter-Eigenschaftszeile** — Wert kopieren, Name kopieren, Eigenschaft entfernen, Eigenschaft hinzufügen; danach dasselbe Bearbeitungsmenü wie im Notiztext; und **Stil…** öffnet den Stil-Gestalter auf der Kategorie **Eigenschaften**.
- **Rechtsklick auf ein Suchergebnis** — eine **sichere** Teilmenge: Öffnen, In neuem Tab öffnen, Im Dateibaum anzeigen, Link kopieren, Pfad kopieren, Lesezeichen, Im Datei-Explorer anzeigen, In Standard-App öffnen und **Stil…** (die Kategorie **Kognitive Farben**). Bewusst gibt es hier **kein Umbenennen, Verschieben oder Löschen** — das Suchpanel hält keine sekundengenaue Kopie des Dateibaums vor, sodass zerstörerische Aktionen im Dateibaum bleiben, wo die Ansicht stets aktuell ist.

Jeder **Stil…**-Eintrag landet auf der Kategorie für das, worauf Sie rechtsgeklickt haben, sodass Sie nie nach den richtigen Steuerelementen suchen müssen. Jeder Menüeintrag, Kategoriename und jede Zustandsbeschriftung erscheint in Ihrer gewählten Oberflächensprache und spiegelt sich für Rechts-nach-links-Layouts.

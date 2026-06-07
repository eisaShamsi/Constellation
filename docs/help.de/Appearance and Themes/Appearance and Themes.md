---
aliases:
  - Themen
  - Stil-Gestalter
  - Style Settings
  - Gespeicherte Stile
  - Benutzerdefiniertes Thema
  - Obsidian-Thema importieren
  - Thema loeschen
  - Stil exportieren
description: Passen Sie jeden sichtbaren Teil von Constellation an — das gesamte Styling lebt jetzt im Stil-Gestalter (Farben, Typografie, Komponenten, getypte Links, gespeicherte Stile); Themen werden unter Darstellung gewaehlt und erstellt.
---

# Darstellung und Themen

Die Darstellung von Constellation wird an zwei Stellen in den **Einstellungen** gesteuert:

1. **Darstellung** — Thema waehlen oder erstellen, Themen aus Obsidians Community-Registry importieren sowie ein paar wenige Voreinstellungen (Titelausrichtung, Lebenszyklus-Abklingen der Links).
2. **Stil-Gestalter** — der eine zentrale Ort fuer das gesamte Styling. Ein eigener Tab in der Einstellungen-Seitenleiste, der jedes sichtbare Element der Constellation-Oberflaeche als live einstellbares Steuerelement (Farbwaehler, Schriftart-Dropdowns, Schieberegler) bereitstellt. Der frueher eigenstaendige **Style-Settings**-Tab wurde abgeloest und vollstaendig in den Stil-Gestalter ueberfuehrt.

Gemeinsam ermoeglichen sie es, die App an Ihren Arbeitsablauf, Ihre Bildschirmgroesse und Ihren persoenlichen Geschmack anzupassen — ohne eine einzige Zeile CSS zu schreiben.

## Themen

Ein **Thema** ist ein benanntes Buendel aus Farben, Einstellungen und CSS, das das Aussehen von Constellation definiert. Constellation bringt sechs eingebaute Themen mit (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark), alle paarweise fuer helle und dunkle Systemmodi.

### Ein Thema waehlen

1. Oeffnen Sie **Einstellungen → Darstellung**.
2. Klicken Sie auf eine Karte im **Themen**-Raster. Das Thema wird sofort angewendet.
3. Die aktive Karte ist mit einem Akzentrahmen hervorgehoben.

### Ein benutzerdefiniertes Thema erstellen

1. Klicken Sie im Themen-Raster auf die gestrichelte Karte **+ Neues Thema**.
2. Geben Sie einen Namen ein, waehlen Sie Hell oder Dunkel und fuenf Farben (Hintergrund, Oberflaeche, Text, Akzent, Rahmen).
3. Klicken Sie auf **Speichern**. Ihr Thema erscheint nun im Raster.

Alle anderen Variablen (Hover-Zustaende, Schatten, gedaempfter Text) werden automatisch aus Ihren fuenf Farben mittels HSL-Mathematik abgeleitet, sodass Sie nur das Wesentliche steuern.

### Benutzerdefiniertes Thema bearbeiten oder loeschen

Fahren Sie mit der Maus ueber eine Karte:
- **✏️ (Bleistift)** — oeffnet den Editor, um Name, Typ oder die fuenf Kernfarben zu aendern.
- **✕ (rotes X)** — loescht das Thema nach Bestaetigung. Eingebaute Themen koennen nicht geloescht werden. Wird das aktive Thema geloescht, faellt Constellation auf das Standardthema zurueck.

### Ein Obsidian-Community-Thema importieren

Klicken Sie auf **🟣 Obsidian-Themen**, um ueber 200 Community-Themen zu durchstoebern:
1. Suchen Sie nach Name oder Autor.
2. Klicken Sie auf **Vorschau** fuer ein Mockup des Layouts und der Fuenf-Farben-Palette.
3. Klicken Sie auf **Import** — das CSS des Themas wird heruntergeladen, fuer Constellation angepasst (Selektor-Shim + Variablenextraktion + CodeMirror-Syntaxfarben) und zu Ihren benutzerdefinierten Themen hinzugefuegt.
4. Unterstuetzt das Thema **Style Settings**, wird die Anzahl auf seiner Karte angezeigt; diese Optionen erscheinen nach dem Import im Style-Settings-Tab.

## Style Settings → jetzt im Stil-Gestalter

> **Hinweis:** Der eigenstaendige **Style-Settings**-Tab wurde **abgeloest**. Jedes Steuerelement, das er hatte, lebt jetzt im **Stil-Gestalter** (einem eigenen Tab in der Einstellungen-Seitenleiste) — der all diese Elemente abdeckt und mehr. Die folgende Liste beschreibt, was der Stil-Gestalter nun abdeckt; sie wird ueber den Stil-Gestalter erreicht.

Diese Styling-Oberflaeche ist Constellations native, themenunabhaengige Steuerzentrale. Sie umfasst jedes sichtbare Element des Rahmens sowie den Editor und funktioniert mit jedem Thema (eingebaut, benutzerdefiniert oder importiert).

### Aufbau

Abschnitte sind standardmaessig eingeklappt. Klicken Sie auf das Chevron zum Ausklappen:

- **Constellation — Farben** — Hintergrund & Oberflaechen, Text, Akzent
- **Constellation — Typografie** — Schriftgroessen fuer Oberflaeche/Notizen/Code, H1–H6-Groessen, Ueberschriftgewicht, Zeilenhoehen, Absatzabstand
- **Constellation — Layout & Form** — Ecken (klein/mittel/gross Radien), Rahmenbreiten, Schatten, Editor-Zeilenlaenge, Seitenraender
- **Constellation — Komponenten** — Ribbon-Dock, Aktionsleiste der Seitenleiste, Layout-Leiste (Panel-Toggles), Top-Leiste/Tab-Leiste, Statusleiste, Datei-Explorer, rechte Seitenleiste, Buttons, Tags, Callouts
- **Constellation — Editor** — Links, Code & Bloecke, Blockzitate, Cursor & Auswahl

### Einen Wert aendern

- **Farbwaehler** — auf das Farbfeld klicken, Farbe waehlen. Der Hex-Wert wird daneben angezeigt.
- **Schieberegler** — ziehen zum Anpassen. Der Zahlenwert erscheint in der Einheit (px, %, etc.).
- **Schalter** — klicken, um Klassen ein-/auszuschalten (meist fuer importierte Themen).
- **Dropdowns** — Option waehlen (Link-Dekorationsstil etc.).
- **Reset-Pfeil (↺)** — erscheint beim Hover am Ende jeder Zeile. Ein Klick darauf entfernt Ihren Override und stellt die Standardeinstellung des Themas wieder her.

### Wie das Speichern funktioniert

- Aenderungen werden automatisch in den **styleSettingsValues** des aktiven Themas gespeichert.
- Aendern Sie eine Style-Einstellung, waehrend ein eingebautes Thema aktiv ist, **klont** Constellation das eingebaute Thema automatisch in Ihre benutzerdefinierten Themen (als `{Name} (custom)`) und speichert Ihre Aenderungen dort. Das eingebaute Thema bleibt unveraendert.
- Das **Gespeichert in:**-Label am unteren Rand des Tabs zeigt, welches Thema Ihre Overrides enthaelt.
- Klicken Sie auf **Alle auf Standard zuruecksetzen**, um alle Overrides im aktiven Thema zu loeschen.

### Style Settings importieren / exportieren

Symbolleiste oben im Style-Settings-Tab:

- **📋 Aus Zwischenablage einfuegen** — ein Klick: liest die Zwischenablage und fuegt gueltiges JSON in das aktive Thema ein.
- **⬆️ Import / Einfuegen** — oeffnet ein Textfeld; JSON manuell einfuegen. **Merge** (hinzufuegen/ueberschreiben) oder **Alles ersetzen** (loeschen, nur Eingefuegtes verwenden).
- **📄 Aus Datei** — eine `.json`-Datei oeffnen, die aus Obsidians Style-Settings-Plugin oder einer anderen Constellation-Installation exportiert wurde.
- **📋 Kopieren** — kopiert die aktuellen Werte als formatiertes JSON in die Zwischenablage.
- **⬇️ Exportieren** — speichert die Werte als `{theme-name}-style-settings.json`.

Das JSON-Format entspricht exakt dem von Obsidians Style-Settings-Plugin — ein flaches Objekt, das Einstellungs-IDs auf String-Werte abbildet:

```json
{
  "h1-size": "36",
  "interactive-accent": "#7c3aed",
  "my-themed-color@@light": "#ffffff",
  "my-themed-color@@dark": "#1e1e2e"
}
```

Das heisst, Sie koennen Ihre Style Settings aus Obsidian kopieren und direkt in Constellation einfuegen — oder umgekehrt.

## Was Sie steuern koennen

Jede Einstellung gehoert zu einem der fuenf obigen Bloecke. Highlights:

### Typografie

- **Oberflaechen-Schriftgroesse** — Seitenleiste, Symbolleisten, Menues
- **Notiz-Schriftgroesse** — Fliesstext im Editor
- **Code-Schriftgroesse** — Inline-Code und Codebloecke
- **H1 – H6 Groessen** — jede Ueberschriftebene einzeln
- **Ueberschriftgewicht** — Leichtigkeit oder Fettigkeit aller Ueberschriften
- **Zeilenhoehen** — normal (Text) und eng (Ueberschriften und dichte UI)
- **Absatzabstand** — Luecke zwischen Absaetzen

### Shell-Komponenten

- **Ribbon-Dock (linke Icons)** — Breite, Buttongroesse, Icongroesse, Radius, Farben
- **Aktionsleiste der Seitenleiste** — Icons fuer Neue Notiz/Tabelle/Ordner — Groesse, Farbe, Hoehe, Hintergrund
- **Layout-Leiste (Panel-Toggles)** — Links/Split/Rechts-Seitenleisten-Toggles — Buttongroesse, Icongroesse, Farben, Aktiv-Status-Farbe
- **Top-Leiste/Tab-Leiste** — nur sichtbar, wenn Notizen in Tabs geoeffnet sind; Leistenhoehe, Hintergrund, Tab-Hoehe/Schrift/Radius, Aktiv- und Inaktiv-Tab-Farben
- **Statusleiste** — Hoehe, Schriftgroesse, Hintergrund, Textfarbe
- **Rechte Seitenleiste (Inspector)** — Hintergrund, Tab-Reihenhoehe, Tab-Icongroesse, Tab-Iconfarben
- **Datei-Explorer (linke Seitenleiste)** — Universum-Notizen-Zeile, Kind-Universen (cUniverse), Bibliotheksnamen, Ordner, Notizen — jeweils mit unabhaengiger Groesse, Gewicht und Farbe; plus vertikaler Zeilenabstand

### Editor

- **Ueberschriftgroessen** (H1–H6) und Gewicht
- **Zeilenhoehe** im Notiztext
- **Inline-Code** Hintergrund, Textfarbe, Radius, Schriftgroesse
- **Linkfarbe** (Standard + Hover) und Dekorationsstil (keine/Unterstrich/gepunktet)
- **Callout-Balkenbreite** und **Callout-Radius**
- **Cursorfarbe** und **Auswahlhintergrund**

### Farben (jede Farbe in der App)

- Hintergrund (primaer/alt), Oberflaechen, Hover-Hintergrund, Rahmen, Eingabehintergrund
- Text (normal/gedaempft/schwach/auf Akzent), Fehler-/Warn-/Erfolg-Zustaende
- Akzent (interaktiver Akzent + Hover), Akzenttext

## Der Stil-Gestalter

Der **Stil-Gestalter** ist ein bildschirmfuellendes Design-Studio fuer Ihre gesamte Oberflaeche — der eine zentrale Ort fuer das gesamte Styling. Anstatt Einstellungen einzeln anzupassen und sich das Ergebnis vorzustellen, aendern Sie ein Steuerelement und beobachten, wie sich Ihre **echte App** dabei umgestaltet.

**Oeffnen:** Der Stil-Gestalter hat einen **eigenen Tab in der Einstellungen-Seitenleiste** (frueher war es eine Schaltflaeche innerhalb von Darstellung). Klicken Sie diesen Tab an, um das Studio zu oeffnen.

**Inspektions-Fadenkreuz.** Ueber dem Einstellungs-Zahnrad des Docks finden Sie ein **Fadenkreuz-Symbol**. Klicken Sie darauf, fahren Sie dann mit der Maus ueber einen beliebigen Teil der App und klicken Sie ihn an — der Stil-Gestalter springt direkt zu den Steuerelementen genau dieses Elements. So muessen Sie nicht erst suchen, welche Kategorie einen bestimmten Knopf, Tab oder Text steuert.

**Was Sie gestalten — die linke Liste.** Auf der linken Seite stehen die *Oberflaechen*, die Sie gestalten koennen:

- **Oberflaeche** — Datei-Explorer, Statusleiste und Universum-Leiste.
- **Komponenten** — Ribbon-Dock, Symbolleisten, Top-Leiste & Tabs, Buttons, Tags & Callouts.
- **Editor** — die Notiz selbst: die **Brotkrumen**-Pfadzeile, Ueberschriften, Fett, Kursiv, Links, Inline-Code, Blockzitate und die Notiz-Zusammenfassung.
- **Global** — Hintergrund- und Textabstufungen, Akzentabstufungen, Schrift & Abstaende, Ecken & Rahmen sowie schriftspezifische Schriftarten.
- **Links** — die Farben getypter Links und ihre Anzeige (siehe unten).
- **Sternenansicht / OrgChart / Index / Cataloger / Shell** — die Plug-in-Oberflaechen.

Darunter stehen Ihre **gespeicherten Stile** — ein Klick wendet den ganzen Look auf einen Schlag an (siehe *Einen Look als benannten Stil speichern* weiter unten). *(Eingebaute Themen werden unter Einstellungen → Darstellung gewaehlt, nicht hier.)*

**Zwei Arten, wie Sie Ihre Aenderungen sehen:**

- **Die Editor-Kategorie** zeigt eine **Notiz-Vorschau in der Mitte.** Klicken Sie auf eine Ueberschrift, auf Fett, einen Link oder die Seite, und die zugehoerigen Steuerelemente erscheinen rechts; die Vorschau aktualisiert sich sofort.
- **Jede andere Kategorie** dockt das Panel an eine Seite und wird durchscheinend, und Ihre Aenderungen erscheinen **live auf der echten App.** Aendern Sie die Statusleisten-Farbe oder die Dock-Breite, und die tatsaechliche Seitenleiste, das Dock, die Tabs und die Statusleiste gestalten sich **waehrend des Ziehens** um. Ein gruenes **● live**-Etikett in der oberen Leiste erinnert Sie daran, dass Sie am echten Ding arbeiten.

**Die Links-Kategorie** haelt die Farben getypter Links und deren Form an einem Ort. Jeder der acht Typen (supports, contradicts, …) wird als seine echte farbige **Pille** angezeigt — **klicken Sie eine Pille an, um sie umzufaerben,** und die Aenderung wird live ueberall uebernommen (die Editor-Links sowie die Pillen in Backlinks / Ausgehende Links). Ueber der Liste liegen Schalter — **Getypte Links einfaerben** und **Typ-Beschriftungen anzeigen** — sowie die Steuerung der **Pillen-Form** (Eckenradius, Hoehe, Beschriftungsgewicht).

**Behalten, Verwerfen, Zuruecksetzen.** Wenn Ihnen gefaellt, was Sie sehen, klicken Sie auf **Behalten** (oben rechts), um den Look **fuer dieses Universum** zu speichern — er uebersteht einen Neustart. **Verwerfen** (oder einfach das Schliessen mit **✕** oder **Esc**) wirft Ihre ungespeicherten Aenderungen weg, und die echte App springt auf den gespeicherten Look zurueck. **Zuruecksetzen** setzt alles auf das schlichte Thema zurueck. Nichts wird auf die Festplatte geschrieben, bis Sie auf Behalten klicken.

**Einen Look als benannten Stil speichern.** Um einen Look wiederzuverwenden, speichern Sie ihn unter einem Namen: Geben Sie oben im Feld **"Entwurf:"** einen Namen ein und klicken Sie auf **"+ Aktuellen als Stil speichern"** (unten links). Er reiht sich in Ihre Liste **Gespeicherte Stile** ein — app-global (ueber jedes Universum hinweg wiederverwendbar) und erfasst den im Stil-Gestalter entworfenen Look, nicht nur ein Thema. **Klicken Sie einen gespeicherten Stil an, um ihn anzuwenden.** Fahren Sie mit der Maus ueber eine Stilzeile fuer ihre Aktionen: **↻ Aktualisieren** (diesen Stil mit Ihrem *aktuellen* Look ueberschreiben — behaelt den Namen), **⤓ Exportieren** (als `.constellation-style.json` teilen), **✎ Umbenennen** und **✕ Loeschen**.

## Haeufige Fragen

### Kann ich die Windows-Titelleiste stylen ("Constellation v0.3.4 — …")?

Nein — diese Leiste wird vom Betriebssystem (Windows/macOS/Linux) gezeichnet. Constellation hat keinen CSS-Zugriff darauf. Alles darunter ist voll stylbar.

### Wie aendere ich die Seitenleistenbreite?

Ziehen Sie am Rand der Seitenleiste (am Ziehgriff zwischen Seitenleiste und Notiz). Den frueheren Schieberegler dafuer gibt es nicht mehr — die Breite wird allein ueber das Ziehen gesteuert, um konkurrierende Quellen der Wahrheit zu vermeiden.

### Wo leben meine Stil-Einstellungen?

Der Look, den Sie im Stil-Gestalter mit **Behalten** speichern, wird **pro Universum** abgelegt und uebersteht einen Neustart. Er reist mit Ihrem Universum — wenn Sie Ihr Universum-Verzeichnis zwischen Geraeten synchronisieren, kommt Ihr Styling mit. **Gespeicherte Stile** hingegen sind app-global und ueber jedes Universum hinweg wiederverwendbar.

### Kann ich ein Thema mit jemandem teilen?

Ja:
- **Komplettes Thema** — im Themen-Editor auf **Exportieren** klicken. Die `.json`-Datei teilen. Der Empfaenger klickt **↓ Import** im Themen-Raster und waehlt sie aus.
- **Ein kompletter Look (gespeicherter Stil)** — im Stil-Gestalter bei einem gespeicherten Stil auf **⤓ Exportieren** klicken. Sie erhalten eine `.constellation-style.json`-Datei, die Sie an jeden senden koennen; der Empfaenger importiert sie als neuen Stil. Erfasst den ganzen Look (Thema, Schriftarten, Link-Farben, Form), nicht nur einzelne Slider-Werte.

### Ein importiertes Obsidian-Thema sieht kaputt aus. Was nun?

Obsidian-Themen koennen komplex sein. Bekannte Faelle:
- Themen mit **HSL-geteilten Farben** (wie Minimal) — ab dieser Version in Constellation unterstuetzt.
- Themen, die auf Obsidians spezifische DOM-Struktur angewiesen sind, werden moeglicherweise teilweise dargestellt. Constellation enthaelt einen Klassen-Shim, der die haeufigsten Selektoren abbildet, aber sehr strukturabhaengige Themen erfordern eventuell Anpassung der fuenf Kernfarben oder manuelle Korrektur der Style-Settings-Werte.

## Verwandt

- [[Universe]] — wo Themen und Style-Settings-Werte gespeichert sind
- [[Libraries]] — Farbakzente pro Bibliothek (in Bibliothekseinstellungen gesetzt, unabhaengig von Themen)
- [[Importer]] — fuer Notiz-Import, nicht Themen (Themen-Import ist unter Darstellung)

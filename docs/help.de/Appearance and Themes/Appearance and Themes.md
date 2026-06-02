---
aliases:
  - Themen
  - Style Settings
  - Benutzerdefiniertes Thema
  - Obsidian-Thema importieren
  - Thema loeschen
  - Style Settings exportieren
description: Passen Sie jeden sichtbaren Teil von Constellation an — Themen, Farben, Typografie und Komponenten-Styling ueber Darstellung und den nativen Style-Settings-Tab.
---

# Darstellung und Themen

Die Darstellung von Constellation wird an zwei Stellen in den **Einstellungen** gesteuert:

1. **Darstellung** — Thema waehlen oder erstellen, Themen aus Obsidians Community-Registry importieren und globale Schrift- und Layout-Einstellungen anpassen.
2. **Style Settings** — ein eigener Tab, der jedes sichtbare Element der Constellation-Oberflaeche als live einstellbares Steuerelement (Schieberegler, Farbwaehler, Dropdowns) bereitstellt. Aenderungen werden sofort angewendet und im aktiven Thema gespeichert.

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

## Style Settings

Der **Style-Settings-Tab** ist Constellations native, themenunabhaengige Steuerzentrale. Er umfasst jedes sichtbare Element des Rahmens sowie den Editor und funktioniert mit jedem Thema (eingebaut, benutzerdefiniert oder importiert).

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

## Der Style Setter

Der **Style Setter** ist ein bildschirmfüllendes Design-Studio für Ihre gesamte Oberfläche. Anstatt Einstellungen einzeln anzupassen und sich das Ergebnis vorzustellen, sehen Sie Ihre tatsächliche Oberfläche in der Bildschirmmitte, klicken auf den Teil, den Sie ändern möchten, und beobachten, wie er sich sofort aktualisiert.

**Öffnen:** Gehen Sie zu **Einstellungen → Darstellung** und klicken Sie auf **"✦ Open Style Setter."** Das Studio füllt den Bildschirm in drei Zonen:

- **Links** — Ihre *Oberflächen* (Editor, Sternenansicht, OrgChart, Index, Cataloger, Shell) und Ihre *Themenkarten* (Midnight, Daylight, Chocolate, Nord).
- **Mitte** — eine Live-Vorschau Ihrer Oberfläche.
- **Rechts** — die Steuerelemente für den jeweils ausgewählten Teil.

**Ein Element gestalten.** Fahren Sie mit der Maus über einen beliebigen Teil der Vorschau in der Mitte — die Seitenleiste, den Notiztitel, eine Überschrift, einen Link, die Notizseite — und ein Rahmen zeigt an, dass er anklickbar ist. Klicken Sie darauf, und seine Steuerelemente erscheinen rechts (eine Farbfläche, ein Schriftart-Dropdown und so weiter). Ändern Sie einen Wert, und die Vorschau aktualisiert sich in dem Moment, in dem Sie es tun — kein Raten.

**Von einem Thema ausgehen.** Klicken Sie links auf eine Themenkarte, um auf einen Schlag einen kompletten Look anzulegen. Das Studio selbst nimmt diesen Look an, während Sie arbeiten, sodass Sie *innerhalb* des Themas gestalten und anschließend einzelne Elemente darauf feinjustieren können.

**Oberflächen wechseln.** Klicken Sie links auf eine Oberfläche, um den Look über verschiedene Teile der App hinweg vorzuschauen, nicht nur im Editor.

**Anwenden.** Wenn Ihnen gefällt, was Sie sehen, klicken Sie auf **"Apply to app"** (oben rechts). Ihr Akzent, Ihre Hintergründe, Textfarbe und Schriftarten werden auf das echte Constellation übertragen. Drücken Sie **Esc** oder **✕**, um zu schließen (dies schließt nur den Setter, nicht die Einstellungen).

> Heute wendet der Style Setter Ihren Look für die aktuelle Sitzung an (eine Live-Vorschau auf der echten App). Das Speichern eines Looks als dauerhafter, benannter **Style** — mit wiederverwendbaren, umbenennbaren Farbflächen sowie Export / Import — kommt als Nächstes.

## Haeufige Fragen

### Kann ich die Windows-Titelleiste stylen ("Constellation v0.3.4 — …")?

Nein — diese Leiste wird vom Betriebssystem (Windows/macOS/Linux) gezeichnet. Constellation hat keinen CSS-Zugriff darauf. Alles darunter ist voll stylbar.

### Warum funktioniert der Seitenleistenbreiten-Schieberegler nicht?

Die Seitenleistenbreite wird ueber den Ziehgriff am Rand gesteuert (zum Aendern ziehen). Wir duplizieren diese Steuerung bewusst nicht in Style Settings, um konkurrierende Quellen der Wahrheit zu vermeiden.

### Wo leben meine Style Settings?

In `Universe/settings.json` unter `customThemes[i].styleSettingsValues`, je Thema. Sie reisen mit Ihrem Universum — wenn Sie Ihr Universum-Verzeichnis zwischen Geraeten synchronisieren, kommt Ihr Styling mit.

### Kann ich ein Thema mit jemandem teilen?

Ja:
- **Komplettes Thema** — im Themen-Editor auf **Exportieren** klicken. Die `.json`-Datei teilen. Der Empfaenger klickt **↓ Import** im Themen-Raster und waehlt sie aus.
- **Nur Style-Settings-Werte** — im Style-Settings-Tab auf **Exportieren** klicken, um nur die Slider-/Farbwerte zu exportieren (nicht die Themenstruktur). Nuetzlich, um persoenliche Anpassungen ueber das Thema eines anderen zu legen.

### Ein importiertes Obsidian-Thema sieht kaputt aus. Was nun?

Obsidian-Themen koennen komplex sein. Bekannte Faelle:
- Themen mit **HSL-geteilten Farben** (wie Minimal) — ab dieser Version in Constellation unterstuetzt.
- Themen, die auf Obsidians spezifische DOM-Struktur angewiesen sind, werden moeglicherweise teilweise dargestellt. Constellation enthaelt einen Klassen-Shim, der die haeufigsten Selektoren abbildet, aber sehr strukturabhaengige Themen erfordern eventuell Anpassung der fuenf Kernfarben oder manuelle Korrektur der Style-Settings-Werte.

## Verwandt

- [[Universe]] — wo Themen und Style-Settings-Werte gespeichert sind
- [[Libraries]] — Farbakzente pro Bibliothek (in Bibliothekseinstellungen gesetzt, unabhaengig von Themen)
- [[Importer]] — fuer Notiz-Import, nicht Themen (Themen-Import ist unter Darstellung)

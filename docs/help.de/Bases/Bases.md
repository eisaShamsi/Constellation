---
aliases:
  - Basen
  - Sternbild-Basis
  - Notiztabellen
  - Strukturierte Ansichten
  - Base-Dateien
description: Lerne, wie du die Sternbild-Basis verwendest — eine lebendige Tabelle deiner Notizen, eine Zeile pro Notiz und eine Spalte pro Eigenschaft, die du sortieren, bearbeiten und umformen kannst, ohne jemals eine Datei zu verschieben.
---

# Basen

Eine **Basis** verwandelt eine Menge deiner Notizen in eine lebendige Tabelle: **eine Zeile pro Notiz, eine Spalte pro Eigenschaft**. Nichts wird kopiert oder verschoben — die Tabelle liest deine Notizen an Ort und Stelle und gibt sie so wieder, wie sie genau jetzt sind.

> [!tip] Stark und doch einfach, von Haus aus
> Eine Basis öffnet sich vertraut und aufgeräumt — nur die Namen deiner Notizen und die Felder, die dir wichtig sind. Sternbilds tiefere, kognitive Spalten sind immer **einen Klick entfernt**, doch sie drängen sich nie auf den ersten Blick. Du entscheidest, wie viel Struktur du hineinholst.

> [!info] Nicht-destruktiv
> Eine Basis ändert deine Notizen niemals von selbst. Sie ist eine kleine `.base`-Datei, die eine Abfrage enthält — „zeige diese Notizen, mit diesen Spalten, in dieser Reihenfolge“. Deine Markdown-Dateien bleiben genau dort, wo sie sind.

---

## Zwei Arten, eine Basis zu nutzen

**1. Als vollständiger Tab.** Öffne eine `.base`-Datei, und sie füllt den Tab als interaktive Tabelle.

**2. Innerhalb einer Notiz.** Füge einen umzäunten Codeblock in eine beliebige Notiz ein, und er wird inline gerendert:

````markdown
```base
view: table
```
````

Beide werden von derselben Engine angetrieben, daher verhalten sie sich identisch.

---

## Eine Basis erstellen

Verwende **Neue Basis** aus der Seitenleiste (die Aktion „+“ / Neue Basis). Sternbild schreibt eine kleine **YAML**-`.base`-Datei für dich:

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

| Feld | Bedeutung |
|-------|---------|
| `schema` | Formatversion (derzeit `1`). |
| `lens` | Der Name, der oben in der Tabelle angezeigt wird. |
| `scope.libraries` | `all`, oder eine Liste bestimmter Bibliotheken, die einbezogen werden. |
| `scope.federation` | `auto` — bezieht auch Notizen aus verknüpften Universen (cUniversen) ein. |
| `columns` | Die anzuzeigenden Spalten. Eine neue Basis startet nur mit dem **Namen** der Notiz. |
| `view` | `table` (die Tabelle ist die Basis-Ansicht). |

Du musst dies selten von Hand bearbeiten — die eigenen Bedienelemente der Tabelle (unten) schreiben jede Änderung für dich in die Datei zurück.

---

## Die Tabelle

- **Namensspalte** — immer zuerst. Klicke auf den Namen einer Notiz, um sie zu öffnen.
- **Jede passende Notiz wird zu einer Zeile.** Es gibt **kein Zeilenlimit**. Die Tabelle ist *virtualisiert* — sie zeichnet nur die Zeilen, die gerade auf dem Bildschirm sind — sodass sich eine Basis über Tausende von Notizen sofort öffnet und flüssig scrollt.
- **Richtung pro Zelle** — jeder Wert erkennt seine eigene Schrift von links nach rechts oder von rechts nach links, sodass Tabellen mit gemischten Sprachen korrekt gelesen werden.
- Die Fußzeile zeigt, wie lange die Abfrage gedauert hat.

---

## Spalten — hinzufügen, entfernen, neu anordnen

### Eine Spalte hinzufügen

Klicke auf **+ Spalte hinzufügen**. Die Auswahl ist in zwei Gruppen unterteilt:

- **Deine Felder** — die Frontmatter-Eigenschaften, die Sternbild in deinen Notizen gefunden hat (zum Beispiel `status`, `maturity`, `author`). Das sind *deine* Daten.
- **Sternbild** — eingebaute Felder, die die App immer kennt: **Name**, **Pfad**, **Erstellt** und **Zusammenfassung**.

Beginne zu tippen, um die Liste zu filtern. Felder, die bereits in der Tabelle sind, werden markiert, damit du sie nicht doppelt hinzufügst.

### Eine Spalte entfernen

Fahre über eine Spaltenüberschrift und klicke auf das **×**.

### Spalten neu anordnen

**Halte eine Spaltenüberschrift gedrückt und ziehe sie zur Seite.** Die gesamte Spalte hebt sich an (sie wird abgedunkelt, und die Überschrift zeigt einen Griff-Umriss), und eine vertikale Linie markiert, wo sie abgelegt wird. Lasse los, um sie zu verschieben. Die Namensspalte bleibt als erste Spalte fixiert.

Jedes Hinzufügen, Entfernen und Neuanordnen wird automatisch in die `.base`-Datei zurückgeschrieben.

---

## Sortieren

**Klicke auf eine Spaltenüberschrift, um danach zu sortieren.** Jeder Klick durchläuft **aufsteigend → absteigend → aus** (ein Pfeil zeigt die aktuelle Richtung).

Um nach mehr als einer Spalte zu sortieren, öffne das **Sortieren**-Panel:

- Füge mehrere Spalten hinzu — die erste ist die primäre Sortierung, die nächsten lösen Gleichstände auf.
- Schalte jede Ebene zwischen aufsteigend und absteigend um.
- Verschiebe Ebenen nach oben oder unten, um die Priorität zu ändern, oder entferne sie.

---

## Eine Notiz aus der Tabelle bearbeiten

Doppelklicke auf eine Zelle in einer deiner **eigenen** Frontmatter-Spalten, um sie zu bearbeiten:

- **Freitextfelder** — tippe den neuen Wert; **Enter** speichert, **Escape** bricht ab.
- **Listenfelder** (wie `maturity`) — ein **Auswahlmenü** erscheint mit den gültigen Werten **in ihrer natürlichen Reihenfolge** (für `maturity`: *seed → sapling → evergreen → canonical*). Wähle einen aus oder tippe deinen eigenen.

Die Änderung wird direkt in das YAML-Frontmatter dieser Notiz auf der Festplatte geschrieben, und die Tabelle aktualisiert sich an Ort und Stelle.

> [!note] Schreibgeschützte Spalten
> **Name** und **Erstellt** (und die anderen eingebauten Sternbild-Spalten) werden für dich berechnet, daher sind sie nicht bearbeitbar. Nur deine eigenen Frontmatter-Felder können hier geändert werden.

---

## Eine ältere Basis öffnen

Wenn du von Obsidian oder von einer früheren Version von Sternbild wechselst, verwenden deine vorhandenen `.base`-Dateien ein älteres Format.

**Deine Datei wird niemals angetastet.** Wenn Sternbild eine öffnet, zeigt es einen ruhigen Hinweis, der erklärt, dass das Format älter ist, und bietet eine Schaltfläche **In Sternbild-Basis umwandeln** an. Die Umwandlung geschieht **nur, wenn du darauf klickst** — sie aktualisiert die Datei an Ort und Stelle auf das neue YAML-Format (und überträgt, was sie kann: den Namen, die Spalten und einfache Textfilter). Bis du dich für die Umwandlung entscheidest, bleibt die Originaldatei genau so, wie sie war.

---

## Föderation

Eine Basis ist universumsbewusst. Mit `federation: auto` bezieht sie Notizen aus verknüpften Universen (cUniversen) neben deinen eigenen ein. Notizen, die in einem verknüpften Universum liegen, sind schreibgeschützt — du kannst sie in der Basis ansehen und sortieren, aber das Bearbeiten ist Notizen vorbehalten, die dir gehören.

---

## Local-First & Datei-über-App

Basen enthalten keine eigenen Daten. Jeder Wert, den du siehst, stammt aus einer echten `.md`-Datei auf deiner Festplatte und wird live gelesen. Lösche die `.base`-Datei, und deine Notizen bleiben völlig unberührt — eine Basis ist nur eine Linse, die du auf Notizen richtest, die du bereits hast.

---

## Tastatur & Maus

| Aktion | Was sie bewirkt |
|--------|--------------|
| **Klick** auf eine Spaltenüberschrift | Danach sortieren (aufsteigend → absteigend → aus) |
| **Ziehen** einer Spaltenüberschrift | Diese Spalte neu anordnen |
| **Klick** auf das × in einer Überschrift | Diese Spalte entfernen |
| **Doppelklick** auf eine Frontmatter-Zelle | Sie bearbeiten (Auswahlmenü bei Listenfeldern) |
| **Enter** | Die Bearbeitung speichern |
| **Escape** | Die Bearbeitung abbrechen |
| **Klick** auf den Namen einer Notiz | Die Notiz öffnen |

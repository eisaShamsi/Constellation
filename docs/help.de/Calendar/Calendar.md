---
aliases:
  - Kalenderfenster
  - Tagesnotizen-Kalender
  - Kulturkalender
description: Eine ganzseitige Monatsansicht über acht Kalender hinweg, mit anklickbaren Tagen, Erstellung von Tagesnotizen, Fälligkeitsdaten von Aufgaben und der Erfassung kultureller Daten.
---

# Kalender

Der **Kalender** ist eine ganzseitige Monatsansicht, die über das **linke Dock** (das Kalendersymbol) geöffnet wird. Tage mit Notizen oder fälligen Aufgaben sind mit farbigen **Punkten** markiert. Die Kopfzeile zeigt den Monat in Ihrem gewählten Kalender; wenn Sie einen **Zweitkalender** einstellen, zeigt ein Untertitel darunter den entsprechenden Zeitraum dieses Kalenders (zum Beispiel zeigt ein gregorianischer Monat seine Hidschri-Spanne, „Dhul-Hidscha 1447 – Muharram 1448 AH").

## Einen Tag anklicken

Jede Tageszelle ist interaktiv:

| Aktion | Ergebnis |
|--------|----------|
| Klick auf die leere Fläche (oder die Tageszahl) | Öffnet — oder erstellt — die **Tagesnotiz** dieses Tages. Ein Klick auf ein Datum, das bereits eine Tagesnotiz hat, **öffnet** sie einfach; es entsteht nie ein Duplikat. |
| Klick auf einen Punkt | Öffnet genau dieses Element. Hat ein Tag mehrere Notizen oder Aufgaben, zeigt ein Klick auf den Punkt eine kleine **Liste** zur Auswahl. |
| Klick auf einen Aufgabenpunkt | Öffnet die Notiz **bis zur Zeile dieser Aufgabe gescrollt**, bereit zum Bearbeiten. |

### Punktfarben

| Punktfarbe | Bedeutung |
|-----------|-----------|
| Gold | Die **Tagesnotiz** dieses Tages |
| Violett | Eine andere an diesem Tag bearbeitete (oder datierte) **Notiz** |
| Rot | Eine an diesem Tag fällige **Aufgabe** |

Alle Punktfarben — und jeder andere Teil des Kalenders — sind über die Oberfläche **Style Setter → Kalender** anpassbar.

> [!tip]
> In der Aufgabenliste können Sie **das Kontrollkästchen einer Aufgabe abhaken, um sie abzuschließen** — direkt aus dem Kalender heraus; abgeschlossene Aufgaben verschwinden sofort. Nur Aufgaben, die ihr eigenes Fälligkeitsdatum `📅 YYYY-MM-DD` tragen, erscheinen im Kalender (das Datum platziert sie auf einem bestimmten Tag).

## Kulturkalender (acht)

In **Einstellungen → Kalender** können Sie das **Kalendersystem** festlegen, und das gesamte Monatsraster wechselt darauf um:

- **Gregorianisch**
- **Hidschri (Islamisch)** — eine präzise astronomische Engine; heilige Monate werden hervorgehoben und islamische Ereignisse markiert.
- **Solar-Hidschri (Persisch)**
- **Hebräisch**
- **Indisch (Saka)**
- **Buddhistisch**
- **Chinesisch** — *lunisolar*
- **Koreanisch** — *lunisolar*

Jede Zelle zeigt sowohl das Datum des gewählten Kalenders (groß) als auch das gregorianische Datum (klein), dazu die Mondphase. Jede Monatskopfzeile zeigt den **Namen des Monats, seine Nummer in Klammern und das Jahr** — die Nummer hilft bei Kalendern, deren Monatsreihenfolge ungewohnt ist.

Der **chinesische und der koreanische** Kalender sind *lunisolar*: Sie fügen mitunter einen **Schaltmonat** (闰六月 / 윤6월) ein, den der Kalender als eigene Seite darstellt, sodass die Navigation ihn nie überspringt oder doppelt zeigt.

Sie können außerdem den **Wochenbeginn** (Sonntag/Montag) wählen und die **Wochennummern-Spalte** ein- und ausschalten.

### Optionen des Hidschri-Kalenders

Unter **Einstellungen → Kalender → „Hidschri-Kalender (Islamisch)"** gibt es zwei zusätzliche Steuerungen:

- **Berechnungsmethode** — **Astronomisch (Mondkonjunktion)**, die dem echten Neumond folgt (am genauesten, die Voreinstellung), oder **Tabellarisch (al-Tawfīqāt al-Ilhāmiyyah)**, der klassische arithmetische Zyklus.
- **Monatskorrektur** — verschieben Sie den Beginn eines Hidschri-Monats um ±1 oder ±2 Tage, um ihn an eine **lokale Mondsichtung** anzupassen. Wählen Sie Hidschri-Jahr und -Monat, wählen Sie einen Versatz und klicken Sie auf **Setzen**; die Korrektur gilt für diesen Monat und jeden folgenden Monat. Ihre Korrekturen werden aufgelistet (jede entfernbar), mit einer Schaltfläche **Alle löschen**.

Beide Einstellungen (und Ihre Korrekturen) werden **mit Ihrem Universum** gespeichert und reisen so über Ihre Geräte hinweg mit.

### Anzeigeoptionen für Chinesisch & Koreanisch

Korea verwendet den chinesischen Mondkalender, daher teilen beide identische Daten — was sie unterscheidet, sind die **Schrift** und das **Jahr**. Wenn einer von beiden Ihr Haupt- oder Zweitkalender ist, zeigt **Einstellungen → Kalender** zwei zusätzliche Steuerungen:

- **Jahresanzeige** — Chinesisch: der sexagesimale Zyklus 丙午年, das schlichte Jahr oder beides; Koreanisch: die **Dangi**-Ära 단기 4359, das Jahr oder der sexagesimale 병오년.
- **Monatsnamen** — *native Schrift* (五月 / 5월), oder *phonetisch* — die Aussprache des Monats in Ihrer eigenen Sprache geschrieben (Deutsch „Wǔyuè / Owol"; Arabisch „وُو-يوي / أوه-وُل").

## Den Kalender gestalten

Öffnen Sie den **Style Setter** (linkes Dock, oder **Einstellungen → Style Setter**) und wählen Sie die Oberfläche **Kalender**, um jeden Teil neu zu gestalten — jedes Element hat seine eigene **Farbe und Textgröße** (Tageszahlen, das Querverweis-Datum, die Monatspille, Wochentagsköpfe, Wochennummern, das Mondsymbol, die Heute-Hervorhebung, Gitterlinien und die Notiz-/Aufgaben-/Ereignispunkte), dazu die **Schriftart** des Kalenders. Eine lebendige Vorschau in voller Größe aktualisiert sich, während Sie bearbeiten; klicken Sie auf **Behalten**, um die Änderungen anzuwenden.

## Tagesnotizen

Der Kalender bedient Tagesnotizen vollständig: Klicken Sie auf einen beliebigen Tag, um ihn zu öffnen, oder führen Sie den Befehl **„Daily Note"** (Befehlspalette) aus, um zum heutigen Tag zu springen.

> [!tip]
> **Dateinamen von Tagesnotizen bleiben stets gregorianisch** (`YYYY-MM-DD`), unabhängig vom angezeigten Kalender — so bleiben Ihre Dateien portabel und sortieren sich korrekt. Das kulturelle Datum wird im Kalender angezeigt und kann im Frontmatter der Notiz festgehalten werden (siehe unten).

## Ein kulturelles Datum in einer Notiz festhalten

Zwei optionale Werkzeuge schreiben das kulturelle Datum in die **Eigenschaften** einer Notiz (der Dateiname bleibt stets gregorianisch `YYYY-MM-DD`):

- **Hidschri-Stempel für Tagesnotizen** — *Einstellungen → Kalender → „Hidschri-Datum in Tagesnotizen stempeln."* Wenn aktiviert (nur verfügbar, solange der Hidschri-Kalender Ihr **Haupt- oder Zweitkalender** ist), erhält jede **neue** Tagesnotiz eine `hijri:`-Zeile, zum Beispiel `hijri: 1448-01-06`. Bereits vorhandene Notizen werden nie angetastet.
- **„+ Hijri" in den Eigenschaften einer Notiz** — öffnen Sie die **Eigenschaften** einer beliebigen Notiz, fahren Sie mit der Maus über das Datum, und es erscheint eine kleine Schaltfläche **„+ Hijri"** (dazu „+ Jalali", „+ Hebrew" usw. — **eine Schaltfläche pro nicht-gregorianischem Kalender, den Sie ausgewählt haben**). Klicken Sie darauf, und Constellation liest das gregorianische Datum der Notiz und fügt das Äquivalent hinzu, zum Beispiel `jalali: 1405-03-30`. Die koreanische Schaltfläche schreibt das **Dangi**-Jahr; ein chinesischer/koreanischer **Schaltmonat** wird mit einem `L` markiert (zum Beispiel `chinese: 2025-06L-17`). Hat die Notiz keine Datums-Eigenschaft, wird das Erstellungsdatum der Datei verwendet.

> [!tip] RTL Support
> Das Kalenderraster respektiert die aktuelle Textrichtung. In RTL-Sprachen (Arabisch, Hebräisch, Persisch, Urdu) passt sich das Kalenderlayout entsprechend an.

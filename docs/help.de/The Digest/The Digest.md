---
translation_status: AI-generated 2026-05-24 — native-speaker review recommended
language: de
source: docs/help.uConstellation.World/The Digest/The Digest.md
aliases:
  - The Digest
  - Universe Digest
  - Digest
  - Digest pane
  - Universum-Digest
  - Digest
  - Digest-Panel
description: Das Universum-Digest ist ein Panel in der linken Andockleiste, das jede Notiz Ihrer Wissensbasis auf Zusammenfassungs-Überschriftenebene zeigt — gestaffelt nach Library → Ordner → Notiz — sodass Sie das gesamte Universum überfliegen können, ohne irgendetwas zu öffnen. Klicken Sie eine Zeile an, um sie aufzuklappen und die volle Zusammenfassung inline zu sehen. Der Filter grenzt die gesamte Liste ein; die Sortierung schaltet zwischen Aktualität (Standard) und alphabetisch um. Liest dieselben Zusammenfassungen, die Sie überall sonst auch sehen; keine zusätzliche Berechnung; kein zusätzlicher Speicherplatz.
---

# Universum-Digest

> *Stellen Sie sich das Digest als Inhaltsverzeichnis Ihres Verstandes vor — keine Liste von Dateien, eine Liste von Ideen.*

Das **Universum-Digest** ist der Ort, an dem Sie Ihre gesamte Wissensbasis auf der Ebene der *Bedeutung* überfliegen. Statt eines Dateibaums (nur Namen) oder der Himmelsansicht (nur Formen) zeigt Ihnen das Digest unter jeder Notiz **den einen Satz, der sagt, worum es in der Notiz geht**. Klicken Sie eine Zeile an und die volle mehrsätzige Zusammenfassung klappt inline aus. Sie können den Inhalt von fünfzig Notizen in einer Minute lesen, ohne auch nur eine zu öffnen.

Es lebt in Ihrer **linken Andockleiste**, neben dem Dateibaum, dem Notizen-Navigator und der Himmelsansicht — eine der vier Möglichkeiten, mit denen Constellation Ihnen die Navigation erlaubt.

---

## Warum das Digest existiert

Ein Dateibaum sagt Ihnen, was Sie *haben*. Eine Suche sagt Ihnen, wonach Sie *gefragt haben*. Das Digest sagt Ihnen, was Sie *wissen*.

Wenn Ihr Universum auf einige hundert Notizen anwächst, wird „jede einzelne öffnen, um sich zu erinnern, was sie sagt" unmöglich. Sie brauchen eine Möglichkeit, den **Kern** jeder Notiz mit Scroll-Geschwindigkeit zu lesen — und eine Möglichkeit, jeden Kern in die volle Zusammenfassung auszudehnen, in dem Moment, in dem Sie sorgfältiger darüber nachdenken möchten. Das ist das Digest.

Es ist die dritte Säule des **Note Summary Creator (NSC)** Core Plug-Ins:
- **Säule 1**: eine Zusammenfassungs-Engine (Phase 1 / MIG-043).
- **Säule 2**: ein Dienst, der die Zusammenfassung dort platziert, wo immer eine Notiz auftaucht (Phase 2 / MIG-044 — Klassifikator, Suchergebnisse, Editor-Band, Backlinks, Outgoing Links, Index, Hover der Himmelsansicht).
- **Säule 3**: diese Ansicht — das Universum-Digest (Phase 3 / MIG-045).

---

## Das Digest öffnen

Klicken Sie in der **linken Seitenleiste** auf das **Symbol des Universum-Digest** (eine kleine Liste mit einem Kreis in der Ecke) — es ist das vierte Symbol in der Reihe, neben Dateibaum / Notizen-Navigator / Himmelsansicht. Die Seitenleiste schaltet auf das Digest-Panel um.

Um zurückzuschalten, klicken Sie auf eines der drei anderen Symbole (oder drücken Sie **Escape**).

---

## Was Sie sehen

Von oben nach unten:

1. **Werkzeugleiste.** Ein Sucheingabefeld + ein kleines Uhrsymbol (der Sortier-Umschalter, Standard „nach Aktualität").
2. **Library-Kopfzeilen.** Großgeschriebene violette Balken — eine pro Library in Ihrem Universum. Jede zeigt den Namen der Library und die Anzahl der enthaltenen Notizen.
3. **Ordner-Kopfzeilen.** Kleine gedämpfte Labels — eine pro Ordner *der Notizen enthält*. Notizen, die in der Library-Wurzel liegen, bekommen keine Ordner-Kopfzeile.
4. **Notizzeilen.** Jede Zeile hat:
   - Einen Chevron (▶) auf der linken Seite — klicken Sie ihn an, um die Zeile aufzuklappen.
   - Den **Notiznamen** in der interaktiven Akzentfarbe — klicken Sie ihn an, um die **Notiz zu öffnen** im Editor.
   - Eine feine kursive Zeile unter dem Namen — die **Zusammenfassungs-Überschrift** (dieselbe, die in jeder anderen Phase-1/2-Oberfläche erscheint).

---

## Eine Zeile aufklappen, um die volle Zusammenfassung zu lesen

Klicken Sie auf den **Chevron** (▶) links von einer Zeile — oder klicken Sie auf die **kursive Überschriftenzeile** selbst. Der Chevron dreht sich auf ▼ und die **volle mehrsätzige Zusammenfassung** erscheint inline unter der Überschrift und umbricht natürlich über so viele Zeilen wie nötig.

Klicken Sie den Chevron (oder die Überschrift) erneut an, um wieder einzuklappen.

Die Aufteilung „klicken Sie den Chevron, um aufzuklappen, klicken Sie den Namen, um zu öffnen" hält die beiden Gesten unterscheidbar: Sie können aufklappen, um *über* eine Notiz zu lesen, und dann an ihr vorbei weiterscrollen; nur wenn Sie den Namen anklicken, öffnet sich die Notiz tatsächlich und übernimmt den Fokus.

---

## Filtern

Tippen Sie in das **Sucheingabefeld** oben. Die Liste grenzt sich ein, während Sie tippen — nur Notizen, deren **Name, Überschrift oder volle Zusammenfassung** Ihre Anfrage enthält, bleiben sichtbar. Library-Kopfzeilen und Ordner-Kopfzeilen mit null passenden Notizen verschwinden vollständig (keine leeren Kopfzeilen).

Leeren Sie das Eingabefeld (× Knopf oder Rücktaste), um die volle Liste wiederherzustellen.

Der Filter ist **sofort** — Constellation greift nicht auf Ihre Festplatte oder die Datenbank zu. Es liest die bereits im Speicher befindlichen Zusammenfassungen, sodass selbst ein Universum mit 10.000 Notizen mit Tippgeschwindigkeit gefiltert wird.

---

## Sortierung: Aktualität oder alphabetisch

Klicken Sie auf das **Uhrsymbol** in der Werkzeugleiste, um zwischen zwei Sortiermodi umzuschalten:

- **Aktualität** (Standard) — innerhalb jedes Ordners erscheinen Notizen in der Reihenfolge der **Erstellungszeit, neueste zuerst**. Ordner innerhalb einer Library werden nach der jüngsten enthaltenen Notiz sortiert (sodass der aktivste Ordner zuerst erscheint). Dies ist der Standard, weil er sichtbar macht, *woran Sie kürzlich gearbeitet haben*.
- **Alphabetisch** — Ordner werden nach Namen sortiert, Notizen innerhalb jedes Ordners nach Namen. Klicken Sie erneut, um zur Aktualität zurückzukehren.

Der Umschalter gilt pro Sitzung; schließen Sie das Digest und öffnen Sie es erneut, und es geht zurück zur Aktualität.

---

## Föderation: untergeordnete Universen erscheinen inline

Wenn Ihr Universum **verlinkte untergeordnete Universen** (cUniverses) hat, erscheint jede Library aus einem untergeordneten Universum im Digest als **eigene gleichrangige Library-Kopfzeile**, neben den Libraries des übergeordneten Universums. Das Digest ist eine vereinheitlichte Ansicht von allem, was von diesem Universum aus erreichbar ist, nicht nur der Libraries, die physisch hier leben.

(Ein zukünftiges Constellation-Update wird einen An/Aus-Umschalter hinzufügen, um Libraries untergeordneter Universen vorübergehend aus dem Digest auszublenden; vorerst erscheinen sie immer.)

---

## Wie das Digest auf riesigen Universen schnell bleibt

Das Digest ist **virtualisiert**: Es rendert nur die Zeilen, die aktuell in Ihrem Scrollport sichtbar sind, nicht den gesamten Baum. Ein Universum mit 10.000 Notizen scrollt genauso reibungslos wie eines mit 50. Wenn Zeilen in die Ansicht scrollen, werden ihre Zusammenfassungen in Stapeln aus Constellations In-Memory-Cache abgerufen (derselbe Cache, der jede andere Phase-1/2-Oberfläche antreibt — keine separate Arbeit, kein separater Speicher).

Das Digest liest Ihre Notizen niemals neu von der Festplatte. Es berechnet Zusammenfassungen niemals neu. Es ist eine **Lese**-Ansicht auf dieselbe `note_summaries`-Tabelle, die die Engine aus Phase 1 befüllt.

---

## Häufige Arbeitsabläufe

**„Ich möchte sehen, woran ich diese Woche gearbeitet habe."**
Öffnen Sie das Digest mit Sortierung = Aktualität (Standard). Die zuletzt erstellten Notizen erscheinen oben in jeder Library/jedem Ordner. Überfliegen Sie die Überschriften.

**„Ich suche eine halb in Erinnerung gebliebene Notiz über X."**
Öffnen Sie das Digest. Tippen Sie X (ein Wort, das im Titel, in der Überschrift oder in der vollen Zusammenfassung der Notiz erscheinen würde). Die Liste grenzt sich auf Kandidaten ein. Klicken Sie auf Chevrons, um volle Zusammenfassungen zu lesen; klicken Sie den Namen, um den Gewinner zu öffnen.

**„Ich möchte eine Top-down-Review meiner Library schreiben."**
Öffnen Sie das Digest, Sortierung = Alphabetisch. Gehen Sie die Überschriften der Reihe nach durch. Klicken Sie auf Chevrons, um vollere Zusammenfassungen zu lesen, wenn Sie etwas einfängt. Verwenden Sie dies als Rückgrat einer neuen MOC-Notiz (Map of Content).

**„Ich erkunde ein föderiertes cUniverse zum ersten Mal."**
Öffnen Sie das Digest. Scrollen Sie an Ihren eigenen Libraries vorbei zu den Libraries des cUniverses — sie sind gleichrangige Zeilen. Lesen Sie die Überschriften, um zu erfahren, was das verlinkte Universum enthält, ohne irgendetwas davon zu öffnen.

---

## Was NICHT im Digest enthalten ist

- **Rechtsklick-Kontextmenü** auf Zeilen — Öffnen in einem neuen Tab, Archivieren etc. (Für v1 sind die primären Aktionen Klick-Name-zum-Öffnen und Klick-Chevron-zum-Aufklappen. Ein zukünftiges Update wird ein Kontextmenü hinzufügen.)
- **Eigene Gruppierungen** — Library → Ordner ist die einzige Staffelung für v1. (Noch kein „Gruppieren nach Tag" oder „Gruppieren nach Stufe".)
- **Drag-to-reorder** — das Digest ist schreibgeschützt; die Sortierung kommt aus Regeln, nicht aus manueller Anordnung.
- **Klassifikator-ähnliche Klassifizierungs-Steuerungen** — das Digest ist eine *Browse*-Ansicht; Klassifizierung lebt im **Klassifikator** (separates Panel).

---

## Verwandte Themen

- **Notizzusammenfassungen** — woher Zusammenfassungen kommen, die Vorrang-Regel (Ihre gewinnt) und die vollständige Liste der Oberflächen, die sie zeigen.
- **Der Klassifikator** — die Heimat von *Alle Zusammenfassungen erstellen* (jede Zusammenfassung in Ihrer Library auf einmal vorberechnen, sodass das Digest sofort gefüllt ist).
- **Himmelsansicht** — die *Form*-Ansicht Ihres Wissens (Blasen + Verbindungen); das Digest ist seine komplementäre *Bedeutungs*-Ansicht.
- **Wissensformulierung** — warum Constellation Wissen nach *Verbindung* und *Zusammenfassung* organisiert, nicht nur nach Dateispeicherung.

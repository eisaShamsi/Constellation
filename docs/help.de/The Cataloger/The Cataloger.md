---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: de
source: docs/help.uConstellation.World/The Cataloger/The Cataloger.md
aliases:
  - The Cataloger
  - Cataloger
  - Classify notes
  - Classification home
  - CECE home
  - Scan library
  - Klassifikator
  - Notizen klassifizieren
  - Klassifizierungs-Startseite
description: Der Klassifikator ist die universumsweite Startseite zum Klassifizieren Ihrer Notizen. Es ist die Vollfenster-Dock-Ansicht, in der Sie die Constellation Epistemic Content Engine (CECE) über Ihre Library laufen lassen, jede Notiz auf Abruf klassifizieren, Notizzusammenfassungen erstellen und die Prüf-Warteschlange abarbeiten. Wenn Source Review die Karte ist, auf der Sie handeln, ist der Klassifikator der Raum, in dem Sie es tun.
---

# Klassifikator

> *„Klassifiziere jede Notiz nach ihrer Wissensart und ihrer Quelle."*

Der **Klassifikator** ist die universumsweite Startseite für die Klassifizierung. Es ist eine Vollfenster-Ansicht, geöffnet aus dem Dock am linken Rand, die alles an einem Ort versammelt, was Sie brauchen, um Ihre Notizen durch Constellations Wissens-Taxonomie zu lesen: ein Steuerelement zum Scannen der gesamten Library, eine Möglichkeit, jede einzelne Notiz auf Abruf zu klassifizieren, eine Schaltfläche zum Erstellen von Notizzusammenfassungen und die Live-Prüf-Warteschlange, in der Sie jeden Vorschlag akzeptieren, bearbeiten, ablehnen oder disambiguieren.

Wenn Sie das **Source-Review**-Panel in der rechten Seitenleiste verwendet haben, kennen Sie die Karten bereits. Der Klassifikator ist dieselbe Engine und dieselben Karten, aus einem schmalen Seitenleisten-Tab herausgehoben und mit dem ganzen Fenster ausgestattet — plus zwei Dingen, die der Seitenleisten-Tab nie hatte: einem Notiz-Picker und einer Schaltfläche „Alle Zusammenfassungen erstellen".

---

## „Der Klassifikator" vs. „die Katalogisierer" — ein kurzes Wort zu den Namen

Diese beiden Namen ähneln sich absichtlich, aber sie bedeuten Unterschiedliches:

- **Der Klassifikator** (diese Ansicht) ist der *Ort* — der Vollfenster-Raum, in dem die Klassifizierung stattfindet.
- **die Katalogisierer** (Plural) sind die *sechs Linsen* innerhalb der Engine — Frontmatter, Zitate, Wortstämme, verlinkte Notizen, ähnliche Notizen und KI-Urteil — von denen jede eine Notiz liest und abstimmt. Fünf der sechs sind heute aktiv; die sechste (KI-Urteil) ist gebaut, aber noch nicht eingeschaltet.

Also: Sie öffnen **den Klassifikator**, und darin nehmen **die Katalogisierer** das Lesen vor. Die Sechs-Linsen-Maschinerie wird vollständig im Thema **Source Review** erklärt — dieses Thema handelt vom Raum.

---

## Was es ist

Der Klassifikator beantwortet eine Frage: **„Wie ist jede Notiz in meinem Universum klassifiziert — und was braucht noch meine Entscheidung?"**

Es ist um vier Dinge herum aufgebaut, von oben nach unten gestapelt:

1. **Ein Header mit drei Aktionen** — *Notiz klassifizieren…*, *Alle Zusammenfassungen erstellen* und *Scan starten*.
2. **Ein Fortschrittsstreifen** — erscheint nur, während ein Library-Scan läuft, und zeigt, wie weit er gekommen ist.
3. **Die Prüf-Warteschlange** — dieselben Karten zum Akzeptieren / Bearbeiten / Ablehnen / Disambiguieren wie im Source-Review-Panel, jetzt über die volle Breite.
4. **Eine Notizzusammenfassung unter jeder Karte** — ein kurzer Abriss der Notiz in einfacher Sprache, sodass Sie entscheiden können, ohne sie zu öffnen (siehe *Notizzusammenfassungen* unten und das eigene Thema **Note Summaries**).

Alles läuft **auf Ihrem Gerät**. Keine Notiz verlässt jemals Constellation.

---

## Warum es wichtig ist

Klassifizierung ist die Art, wie Constellation einen Stapel `.md`-Dateien in einen *geformten* Wissenskörper verwandelt — jede Notiz auf zwei Achsen platziert (woher das Wissen kam und welche Art von Wissen es ist). Diese Form ist das, was **Constellation Sight**, das **Epistemic-Metadata**-Panel und die taxonomiebewusste Suche antreibt.

Aber Klassifizierung ist eine entscheidungsintensive Aufgabe. Wenn Sie Hunderte unklassifizierter Notizen haben, ist es langsam, dies aus einem schmalen Seitenleisten-Tab zu tun — eine Notiz nach der anderen, ohne eine Möglichkeit, eine bestimmte Notiz herbeizurufen. Der Klassifikator existiert, um die Aufgabe *sitzbar* zu machen: Öffnen Sie es einmal, geben Sie ihm den ganzen Bildschirm und arbeiten Sie Ihre Library in einer einzigen fokussierten Sitzung durch. Der Notiz-Picker lässt Sie jede Notiz nach Namen heranziehen; die Zusammenfassungen lassen Sie eine Karte beurteilen, ohne den Raum zu verlassen; das Scan-Steuerelement füllt die Warteschlange in großen Mengen.

---

## Wie man es öffnet

1. Klicken Sie im **Dock am linken Rand** (der vertikale Streifen mit Symbolen am äußersten Fensterrand) auf das **Symbol mit den gestapelten Karten** — drei kleine, übereinandergelegte Karten. Es sitzt zwischen den anderen Arbeitsbereich-Symbolen wie dem Sight-Auge und dem Nervous-System-Neuron.
2. Der Klassifikator öffnet sich als **Vollfenster-Ansicht** und übernimmt den Inhaltsbereich.
3. Zum Schließen: klicken Sie auf **(×)** oben rechts im Header oder drücken Sie **Esc**. Sie kehren dorthin zurück, wo Sie waren.

> **Hinweis zu Esc:** Wenn das Suchfeld *Notiz klassifizieren…* geöffnet ist, schließt das Drücken von **Esc** nur dieses Feld und lässt den Klassifikator geöffnet. Drücken Sie **Esc** erneut (bei geschlossenem Feld), um den Klassifikator selbst zu schließen.

---

## Was Sie sehen

### Der Header — drei Aktionen

Oben im Klassifikator sitzen drei Steuerelemente nebeneinander:

| Steuerelement | Was es tut |
|---|---|
| **Notiz klassifizieren…** | Öffnet ein kleines Suchfeld. Tippen Sie ein paar Buchstaben des Titels einer beliebigen Notiz, wählen Sie sie aus den Ergebnissen aus, und die Engine klassifiziert sie auf der Stelle — ohne dass Sie die Notiz erst öffnen müssen. Der neue Vorschlag erscheint in der Warteschlange darunter. |
| **Alle Zusammenfassungen erstellen** | Berechnet im Voraus eine kurze Zusammenfassung für jede Notiz, die noch keine hat. Läuft leise im Hintergrund; der Fortschritt erscheint in der Statusleiste am unteren Fensterrand; Sie können jederzeit abbrechen. (Im Detail beschrieben im Thema **Note Summaries**.) |
| **Scan starten** | Lässt die Engine auf einmal über Ihre **gesamte aktive Library** laufen und stellt einen Vorschlag für jede noch nicht klassifizierte Notiz in die Warteschlange. Während er läuft, zeigt die Schaltfläche *Läuft…* und ein Fortschrittsstreifen erscheint unter dem Header. |

### Der Fortschrittsstreifen

Direkt unter dem Header erscheint ein schmaler Streifen **nur, während ein Library-Scan läuft**. Er zeigt, wie viele Notizen verarbeitet wurden, und lässt Sie den Scan bis zum Ende verfolgen. Wenn kein Scan läuft, ist der Streifen ausgeblendet und die Warteschlange sitzt direkt unter dem Header.

### Die Prüf-Warteschlange

Der Großteil des Klassifikators ist die **Prüf-Warteschlange** — dieselben Karten, die Sie im Source-Review-Panel sehen, nur über die volle Breite. Jede Karte zeigt eine Notiz, die Einschätzung der Engine, wie sie in Ihre Taxonomie passt (Source × Content Type), die sechs kleinen Katalogisierer-Punkte und die Aktionen, die Sie ergreifen können:

- **Accept** — schreibt den Vorschlag der Engine in die Notiz und räumt die Karte.
- **Edit** — wählen Sie die Werte selbst aus einem Baum.
- **Reject** — räumt die Karte, ohne etwas zu schreiben.
- **Disambiguate** — bei einer „Split"-Karte wählen Sie den richtigen Wert aus den Kandidaten-Chips.

Die vollständige Mechanik der Karten — die farbigen Punkte, die Konfidenzregime, die Sibling Disambiguation, die Filter-Chips der Warteschlange, „Approve all" und die Kalibrierung pro Library — ist im Thema **Source Review** dokumentiert. Der Klassifikator verwendet genau dieses Panel; an den Karten ändert sich nichts zwischen der Seitenleiste und der Vollfenster-Ansicht.

### Die Notizzusammenfassung unter jeder Karte

Unter dem Titel jeder Karte sitzt eine kurze **Zusammenfassung**-Zeile — ein paar Sätze, die Ihnen sagen, worum es in der Notiz geht, sodass Sie die Karte beurteilen können, ohne die Notiz zu öffnen. Diese wird vom **Note Summary Creator (NSC)** erzeugt; siehe den nächsten Abschnitt und das Thema **Note Summaries**.

---

## Eine einzelne Notiz klassifizieren — der Notiz-Picker

Die Schaltfläche *Notiz klassifizieren…* löst ein einfaches Problem: Im Seitenleisten-Tab konnten Sie nur die Notiz klassifizieren, die Sie gerade geöffnet hatten. Der Klassifikator hat keine „geöffnete Notiz", daher gibt er Ihnen eine Möglichkeit, jede Notiz nach Namen herbeizurufen.

**Um eine Notiz zu klassifizieren:**

1. Klicken Sie auf **Notiz klassifizieren…**. Ein Suchfeld klappt herunter mit dem Platzhalter *Notizen suchen…*.
2. Beginnen Sie, den Titel der Notiz zu tippen. Nach einer kurzen Pause erscheinen passende Notizen in einer Liste (bis zu zehn).
3. Klicken Sie auf die gewünschte Notiz. Die Engine klassifiziert sie, das Feld schließt sich, und eine frische Karte für diese Notiz erscheint in der Warteschlange darunter.
4. Wenn etwas schiefgeht (ein seltener Engine-Fehler), erscheint die Meldung innerhalb des Feldes, sodass Sie wissen, dass die Klassifizierung nicht gelaufen ist.

Sie müssen die Notiz nicht öffnen und Sie verlieren Ihren Platz in der Warteschlange nicht. Dies ist der schnellste Weg, eine bestimmte Notiz zu klassifizieren, die Sie im Sinn haben.

---

## Notizzusammenfassungen (NSC) im Klassifikator

Jede Karte in der Warteschlange trägt eine kurze **Zusammenfassung** ihrer Notiz, angezeigt unter dem Titel. Die Zusammenfassung wird vom **Note Summary Creator (NSC)** erzeugt und folgt einer festen Regel: **Wenn Sie eine Zusammenfassung geschrieben haben, verwendet die Engine Ihre; sie generiert nur eine, wenn Sie es nicht getan haben.**

Die Rangfolge ist:

1. **Ihre Frontmatter-Zusammenfassung** — ein Feld `summary:`, `description:`, `abstract:` oder `excerpt:` in den Eigenschaften der Notiz. Genau so verwendet, wie Sie es geschrieben haben.
2. **Ihr Zusammenfassungs-Callout** — ein Block `> [!summary]`, `> [!abstract]` oder `> [!tldr]` im Notiztext. Genau so verwendet, wie Sie ihn geschrieben haben, samt diakritischer Zeichen.
3. **Eine generierte Zusammenfassung** — nur wenn Sie weder das eine noch das andere geschrieben haben. Constellation liest die Notiz, findet ihre zentralsten Sätze und zeigt die obersten drei in ihrer ursprünglichen Reihenfolge.

Die Engine **schreibt eine generierte Zusammenfassung niemals in Ihre Notiz zurück** — Ihre `.md`-Dateien sind die Quelle der Wahrheit, und der Klassifikator *liest* sie immer nur.

Die Schaltfläche **Alle Zusammenfassungen erstellen** berechnet die Zusammenfassungen für die gesamte Library im Hintergrund vor, sodass Karten ihre Zusammenfassung sofort anzeigen, statt sie beim Scrollen nachzuladen. Vollständige Details — einschließlich der Frage, wie die generierten Zusammenfassungen erzeugt werden und was zu tun ist, wenn eine Zusammenfassung falsch aussieht — finden sich im Thema **Note Summaries**.

---

## Was der Klassifikator *nicht* tut

- **Er klassifiziert standardmäßig nicht automatisch im Hintergrund.** Scans sind etwas, das Sie *starten*. (Es gibt einen optionalen Hintergrundmodus in Einstellungen → Intelligenz → CECE, standardmäßig deaktiviert — siehe **Source Review**.)
- **Er ruft keinen Cloud-Dienst auf.** Die fünf aktiven Katalogisierer sind heuristisch und lokal. Die sechste Linse (KI-Urteil, ein lokales Sprachmodell) ist im Design eingebaut, aber noch nicht eingeschaltet, daher bleibt sie heute auf jeder Karte stumm.
- **Er ändert nicht den Wortlaut Ihrer Notizen.** Das Akzeptieren einer Karte schreibt Klassifizierungs-*Eigenschaften* (die Frontmatter-Felder `sources:` und `content_type:`). Es bearbeitet niemals Ihre Prosa und es schreibt niemals eine generierte Zusammenfassung in die Datei.

---

## Häufige Workflows

**„Ich habe den Klassifikator gerade zum ersten Mal geöffnet — wo fange ich an?"**
Klicken Sie auf **Scan starten**, um einen Vorschlag für jede unklassifizierte Notiz in der Library in die Warteschlange zu stellen. Beobachten Sie, wie sich der Fortschrittsstreifen füllt. Arbeiten Sie dann die Warteschlange ab, akzeptieren Sie die, die die Engine richtig hatte, und disambiguieren Sie die gespaltenen. Die Zusammenfassungen unter jeder Karte lassen Sie schnell entscheiden.

**„Ich möchte eine bestimmte Notiz klassifizieren, nicht die ganze Library."**
Klicken Sie auf **Notiz klassifizieren…**, tippen Sie ihren Titel, klicken Sie sie an. Eine Karte erscheint in der Warteschlange. Akzeptieren oder bearbeiten Sie sie.

**„Meine Karten brauchen einen Moment, um ihre Zusammenfassungen anzuzeigen."**
Klicken Sie einmal auf **Alle Zusammenfassungen erstellen**. Es berechnet die Zusammenfassung jeder Notiz im Hintergrund vor (Fortschritt in der Statusleiste). Nachdem es fertig ist, erscheinen die Zusammenfassungen sofort.

**„Die Warteschlange hat Hunderte von Karten — wie fokussiere ich mich?"**
Verwenden Sie die Filter-Chips über der Warteschlange (dokumentiert in **Source Review**): Beginnen Sie mit *Catalogers agreed* und *Approve all*, um die einfachen zu löschen, und nehmen Sie sich dann die gespaltenen Karten vor.

---

## Verwandte Themen

- **Source Review** — die Karten selbst: die sechs Katalogisierer, die farbigen Punkte, Konfidenzregime, Sibling Disambiguation, Warteschlangen-Filter, „Approve all" und die Kalibrierung pro Library. Der Klassifikator bettet dieses Panel ein.
- **Note Summaries** — wie die Zusammenfassungs-Zeile unter jeder Karte erzeugt wird, die Autoren-zuerst-Rangfolge und der *Alle Zusammenfassungen erstellen*-Backfill.
- **Cognitive Engine** — die umfassendere Wissensformulierungsphilosophie, in die die Klassifizierung passt.
- **Epistemic Metadata** — die Eigenschaften `sources:` und `content_type:`, die die Klassifizierung schreibt, und wie man sie liest.
- **Constellation Sight** — die räumliche Ansicht, die die Source × Content Type-Klassifizierung antreibt.

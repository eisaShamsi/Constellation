---
aliases:
  - Kognitive Farben
  - Eigenschaft-Tags gestalten
  - Taxonomie-Badges gestalten
  - Reife-Farben
  - Konfidenz-Farben
  - Herkunft-Farben
  - Phasen-Farben
  - Treffer-Kategorie-Farben
  - Rechtsklick-Menue
  - Kontextmenue
  - Rechtsklick im Notiztext
  - Rechtsklick auf Eigenschaft
  - Rechtsklick auf Suchergebnis
  - Bei Bedarf vereinheitlichen
description: Gestalten Sie die Eigenschaft-Tags im Frontmatter und die Taxonomie-Badges neu, legen Sie eine gemeinsame Farbe fuer jeden kognitiven Zustand fest (Reife, Konfidenz, Herkunft, Phase, Treffer-Kategorie), sodass sich alle Oberflaechen bei Bedarf vereinheitlichen, und nutzen Sie die app-weiten Rechtsklick-Menues im Notiztext, im Eigenschaften-Panel und bei Suchergebnissen.
---

# Kognitive Farben und Rechtsklick-Menues

Dieses Thema behandelt zwei Dinge, die gemeinsam eingetroffen sind: **zwei neue Stil-Gestalter-Kategorien** — **Eigenschaften** (zum Umgestalten der kleinen Tags in Ihrem Frontmatter) und **Kognitive Farben** (ein Farbregler pro kognitivem Zustand, geteilt ueber die gesamte App) — sowie die **app-weiten Rechtsklick-Menues**, die die richtigen Aktionen mit einem Klick erreichbar machen: im Notiztext, auf einer Frontmatter-Eigenschaft und auf einem Suchergebnis.

> Der Stil-Gestalter ist das bildschirmfuellende Design-Studio, das Sie ueber **Einstellungen → Darstellung → „✦ Stil-Gestalter oeffnen"** oder ueber seinen eigenen **Stil-Gestalter**-Tab in der Einstellungen-Seitenleiste oeffnen. Die beiden folgenden Kategorien stehen in der linken Liste der *Oberflaechen* neben Oberflaeche, Editor, Links und den uebrigen. Zum allgemeinen Verhalten des Gestalters — Inspizieren, Behalten / Verwerfen / Zuruecksetzen, gespeicherte Stile — siehe [[Appearance and Themes]].

---

## Stil-Gestalter → Eigenschaften

Die Kategorie **Eigenschaften** gestaltet die kleinen Tags neu, die im **Frontmatter** einer Notiz erscheinen (ihrem YAML-Eigenschaftenblock) — die Chips, die Sie fuer `tags`, `aliases` und Aehnliches im Eigenschaften-Panel und am Anfang der Notiz sehen. Bislang waren sie fest vorgegeben; nun koennen Sie sie selbst gestalten.

Oeffnen Sie den **Stil-Gestalter** und klicken Sie in der linken Liste auf **Eigenschaften**. Die Mitte zeigt eine Live-Vorschau der Eigenschafts-Pillen; klicken Sie rechts auf ein Steuerelement, und die Vorschau gestaltet sich um, waehrend Sie editieren. Zwei Elemente:

### Eigenschaft-Tags

Die gewoehnlichen Frontmatter-Tag-Chips (zum Beispiel jeder Wert in einer `tags`-Liste). Vier Steuerelemente:

- **Tag-Hintergrund** — die Fuellfarbe des Chips.
- **Tag-Text** — die Farbe des Texts im Chip.
- **Tag-Radius** — wie stark die Ecken des Chips abgerundet sind (0 px = eckig, bis 20 px = voll abgerundet).
- **Hoehe** — die Hoehe des Chips in Pixeln (14–32 px).

### Taxonomie-Badges

Die Pillen, die fuer taxonomieartige Werte verwendet werden. Drei Steuerelemente:

- **Hintergrund** — die Fuellfarbe der Pille.
- **Text** — die Textfarbe in der Pille.
- **Radius** — Eckenrundung (0–20 px).

> **Nichts aendert sich, bis Sie ein Steuerelement beruehren.** Jeder Wert startet exakt mit dem Aussehen, das Sie heute haben, sodass die Kategorie Eigenschaften Ihre Notizen unveraendert laesst, bis Sie bewusst eine Farbe waehlen oder einen Schieberegler ziehen. Klicken Sie auf **Behalten**, um den Look fuer dieses Universum zu speichern.

---

## Stil-Gestalter → Kognitive Farben

Constellation faerbt Ihr **kognitives Vokabular** in Farbe — die *Reife* einer Notiz, die *Konfidenz* eines Links, woher eine Idee *stammt*, in welcher *Phase* ihres Lebens sie ist und *warum* ein Suchergebnis ein Treffer war. Das Problem war, dass jede dieser Farben auf jeder Oberflaeche einzeln entschieden wurde: eine „welkende" Notiz konnte im Dateibaum ein Gruenton sein und in der Sternenansicht ein anderer. Die Kategorie **Kognitive Farben** gibt Ihnen **einen Farbregler pro Zustand**, und alles, was diesen Zustand anzeigt, folgt ihm.

Oeffnen Sie den **Stil-Gestalter** und klicken Sie in der linken Liste auf **Kognitive Farben**. Die Mitte zeigt eine Farblegende fuer den Satz, den Sie gerade bearbeiten; waehlen Sie rechts ein Steuerelement, und die Legende aktualisiert sich live. Es gibt fuenf Saetze.

### Reife — wie gefestigt eine Idee ist

Fuenf Zustaende, vom juengsten zum gefestigtsten: **Samen**, **Setzling**, **Immergruen**, **Kanonisch**, **Welkend**. Jeder erhaelt eine Farbe, verwendet von den Notizpunkten im Dateibaum, der Reife-Markierung im Tab und dem Notiz-Inspektor.

### Konfidenz — wie gewiss ein Link ist

Vier Zustaende: **Hypothese**, **Beleg**, **Etabliert**, **Umstritten**. Je eine Farbe.

### Herkunft — woher eine Idee stammt

Vier Zustaende: **Empfangen** (aus einer Quelle uebernommen), **Entdeckt** (Ihr eigenes Denken), **Gemischt** und **Keine**. Je eine Farbe.

### Phase — wo eine Notiz in ihrem Leben steht

Sechs Zustaende, in dieser Reihenfolge: **Funke**, **Geburt**, **Wachstum**, **Reife**, **Ruhephase**, **Archivierung**. Je eine Farbe.

### Treffer-Kategorie — warum ein Suchergebnis ein Treffer war

Sieben Trefferarten: **Titel**, **Inhalt**, **Tag**, **Wikilink**, **Eigenschaft**, **Semantisch** (ein bedeutungsbasierter Treffer, kein exaktes Wort) und **Strukturiert** (ein Treffer per Eigenschaftsabfrage). Die hier gesetzte Farbe wird von der Suchhervorhebung im Editor, dem Treffer-Badge und der Hervorhebung der Ergebniszeile im Suchpanel gemeinsam genutzt.

### „Bei Bedarf vereinheitlichen" — die Regel, die das sicher macht

Kognitive Farben folgen einer bewussten Regel: **nichts aendert sich, bis Sie eine Farbe waehlen.** Jede Oberflaeche behaelt die Farbe, die sie heute hat, als eigenen Rueckfallwert. In dem Moment, in dem Sie hier die Farbe eines Zustands festlegen, schnappt **jede** Oberflaeche, die diesen Zustand anzeigt, auf einmal auf Ihre Farbe um — Dateibaum, Tabs, der Inspektor, Suchhervorhebungen und so weiter. Setzen Sie „Immergruen" einmal, und jede Immergruen-Markierung in der gesamten App stimmt ueberein. Lassen Sie einen Zustand unberuehrt, sieht er genau wie zuvor aus.

Deshalb kann die Kategorie ausgeliefert werden, ohne ein einziges bestehendes Aussehen zu veraendern: sie vereinheitlicht *bei Bedarf*, niemals standardmaessig. Klicken Sie auf **Behalten**, um Ihre Farben fuer dieses Universum zu speichern.

---

## Rechtsklick-Menues in der gesamten App

Constellation gibt Ihnen nun ein vollstaendiges Rechtsklick-Menue (Kontextmenue) an den drei Stellen, an denen Sie es am haeufigsten brauchen: im **Notiztext**, auf einer **Frontmatter-Eigenschaft** und auf einem **Suchergebnis**. Jedes Menue bietet nur Aktionen, die dort sinnvoll sind, wo Sie geklickt haben.

### Rechtsklick im Notiztext

Klicken Sie mit der rechten Maustaste irgendwo in den Text einer Notiz, um das Bearbeitungsmenue zu erhalten:

- **Link einfuegen** / **Externer Link** — umschliesst die Auswahl (oder fuegt an der Cursorposition ein) als `[[Wikilink]]` oder als `[Text](url)`-Link.
- **Format ▸** — ein Ausklappmenue: Fett, Kursiv, Unterstreichen, Durchgestrichen, Hervorheben, Inline-Code, Inline-Mathe, Kommentar umschalten, Hochgestellt, Tiefgestellt, Formatierung loeschen.
- **Absatz ▸** — ein Ausklappmenue: Aufzaehlung, Nummerierte Liste, Aufgabenliste, die Ueberschriftebenen **H1–H6** und **Absatz** sowie Zitat.
- **Einfuegen ▸** — ein Ausklappmenue: Fussnote, Tabelle, Hinweis, Horizontale Linie, Codeblock, Matheblock, Bild.
- **Zwischenablage** — Ausschneiden, Kopieren, Einfuegen, Als Text einfuegen, Alles auswaehlen.
- **Stil…** — springt direkt in den **Stil-Gestalter**, fokussiert auf die Kategorie **Editor**, sodass Sie genau das umgestalten koennen, worauf Sie gerade rechtsgeklickt haben.

### Rechtsklick auf eine Frontmatter-Eigenschaft

Klicken Sie mit der rechten Maustaste auf eine Eigenschafts**zeile** im Eigenschaften-Panel (oder im Eigenschaftenblock am Anfang der Notiz), und Sie erhalten Eigenschafts-Aktionen zusaetzlich zum vollstaendigen Bearbeitungsmenue:

- **Wert kopieren** — kopiert den Wert der Eigenschaft in die Zwischenablage.
- **Name kopieren** — kopiert den Schluessel der Eigenschaft.
- **Eigenschaft entfernen** — loescht diese Eigenschaftszeile.
- **Eigenschaft hinzufuegen** — fuegt eine neue, leere Eigenschaftszeile hinzu.
- …gefolgt von denselben Eintraegen **Format / Absatz / Einfuegen / Zwischenablage** wie im Notiztext, und einem Eintrag **Stil…**, der den Stil-Gestalter fokussiert auf die Kategorie **Eigenschaften** oeffnet — sodass „Stil…" auf einem Eigenschaft-Tag die Eigenschaft-Tags gestaltet, nicht den Notiztext.

### Rechtsklick auf ein Suchergebnis

Klicken Sie mit der rechten Maustaste auf ein Ergebnis im Suchpanel fuer einen **sicheren** Satz an Notiz-Aktionen — die, die Ihre Dateien nie gefaehrden:

- **Oeffnen** — die Notiz oeffnen.
- **In neuem Tab oeffnen** — sie neben dem oeffnen, was Sie schon offen haben.
- **Im Dateibaum anzeigen** — die Notiz im Dateibaum hervorheben, damit Sie sehen, wo sie liegt.
- **Link kopieren** / **Pfad kopieren** — einen Wikilink zur Notiz oder ihren Dateipfad kopieren.
- **Lesezeichen** — die Notiz zu Ihren Lesezeichen hinzufuegen.
- **Im Datei-Explorer anzeigen** — die Datei im Dateimanager Ihres Betriebssystems zeigen.
- **In Standard-App oeffnen** — die Datei in der App oeffnen, die Ihr System fuer Markdown verwendet.
- **Stil…** — den Stil-Gestalter fokussiert auf die Kategorie **Kognitive Farben** oeffnen (wo die Treffer-Farben der Suche leben).

> **Bewusst hat das Suchergebnis-Menue kein Umbenennen, Verschieben oder Loeschen.** Ein Suchpanel zeigt Ergebnisse aus Ihrem gesamten Universum und haelt keine sekundengenaue eigene Kopie des Dateibaums vor, sodass eine zerstoererische Aktion dort auf einer veralteten Ansicht arbeiten koennte. Constellation belaesst diese Operationen im Dateibaum, wo die Ansicht stets aktuell ist. Das Suchmenue dient dazu, *sicher zu einer Notiz zu gelangen*, nicht dazu, Ihre Bibliothek umzustrukturieren.

---

## Gut zu wissen

- **Lokal und privat.** All dies wird aus Ihren eigenen Notizen und Einstellungen auf Ihrem Geraet berechnet. Nichts wird irgendwohin gesendet.
- **Es spricht Ihre Sprache.** Jeder Menueeintrag, jeder Kategoriename, jede Zustandsbeschriftung erscheint in Ihrer gewaehlten Oberflaechensprache und spiegelt sich korrekt fuer Rechts-nach-links-Sprachen. Die Farben der kognitiven Zustaende selbst sind universell — eine Farbe bedeutet in jeder Sprache denselben Zustand.
- **„Stil…" landet immer auf der richtigen Oberflaeche.** Jeder „Stil…"-Eintrag oeffnet den Stil-Gestalter fokussiert auf die Kategorie fuer das, worauf Sie rechtsgeklickt haben: der Notiztext → **Editor**, eine Eigenschaft → **Eigenschaften**, ein Suchergebnis → **Kognitive Farben**. Sie muessen nie nach den richtigen Steuerelementen suchen.

---

## Verwandt

- [[Appearance and Themes]] — das allgemeine Verhalten des Stil-Gestalters, Themen, Schriftarten und gespeicherte Stile
- [[Properties]] — Anzeigen und Bearbeiten der Frontmatter-Eigenschaften, deren Tags Sie hier umgestalten
- [[Search]] — das Suchpanel, dessen Ergebnisse das Rechtsklick-Menue tragen
- [[Cognitive Engine]] — was Reife, Konfidenz, Herkunft und Phase als Wissensmasse bedeuten
- [[Knowledge Formulation]] — die Konfidenzstufen der lebenden Links, die die Konfidenz-Farben darstellen

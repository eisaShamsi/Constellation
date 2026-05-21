---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: de
source: docs/help.uConstellation.World/Note Summaries/Note Summaries.md
aliases:
  - Note Summaries
  - Note Summary
  - Summary
  - NSC
  - Note Summary Creator
  - Build all summaries
  - Notizzusammenfassungen
  - Notizzusammenfassung
  - Zusammenfassung
  - Alle Zusammenfassungen erstellen
description: Notizzusammenfassungen geben Ihnen einen kurzen Abriss einer Notiz in einfacher Sprache, sodass Sie sie beurteilen können, ohne sie zu öffnen. Constellation respektiert immer eine Zusammenfassung, die Sie selbst geschrieben haben — im Frontmatter oder in einem Zusammenfassungs-Callout — und generiert nur dann eine, wenn Sie es nicht getan haben. Generierte Zusammenfassungen sind extraktiv (die zentralsten Sätze der Notiz selbst), schreibgeschützt (werden nie in Ihre Datei zurückgeschrieben) und vollständig auf Ihrem Gerät berechnet.
---

# Notizzusammenfassungen

> *Wenn Sie eine Zusammenfassung geschrieben haben, verwendet Constellation Ihre. Es schreibt nur dann eine, wenn Sie es nicht getan haben — und selbst dann niemals in Ihre Datei.*

Eine **Notizzusammenfassung** ist ein kurzer Abriss einer Notiz — ein paar Sätze, die Ihnen auf einen Blick sagen, worum es in der Notiz geht. Zusammenfassungen werden vom **Note Summary Creator (NSC)** erzeugt. Heute erscheinen sie unter dem Titel jeder Notiz in der **Klassifikator**- / **Source-Review**-Warteschlange, sodass Sie entscheiden können, wie eine Karte zu klassifizieren ist, ohne die Notiz dahinter zu öffnen.

Dieses Thema erklärt, woher Zusammenfassungen kommen, die strikte Rangfolge, die immer *Ihre* Worte denen der Maschine vorzieht, wie die generierten Zusammenfassungen aufgebaut werden und wie man sie für eine ganze Library auf einmal vorberechnet.

---

## Warum Zusammenfassungen existieren

Wenn Sie eine Prüf-Warteschlange mit Hunderten von Karten abarbeiten, reicht der Titel allein oft nicht aus, um sich zu erinnern, was eine Notiz tatsächlich sagt. Jede Notiz zu öffnen, um Ihr Gedächtnis aufzufrischen, unterbricht Ihren Fluss. Eine Auf-einen-Blick-Zusammenfassung unter dem Titel behebt das: Sie lesen drei Sätze, Sie erinnern sich an die Notiz, Sie treffen die Entscheidung, Sie machen weiter.

Aber eine Zusammenfassung ist auch ein kleiner Akt der Autorschaft. Wenn Sie eine Notiz bereits in Ihren eigenen Worten destilliert haben — in einem `summary:`-Feld oder einem `> [!summary]`-Callout —, dann ist *das* die Zusammenfassung, die angezeigt werden sollte, nicht die Vermutung einer Maschine. Constellations erste Regel für Zusammenfassungen ist daher eine Regel über den Respekt vor Ihrem Schreiben: **Ihre gewinnt.**

---

## Woher eine Zusammenfassung kommt — die Rangfolge

Für jede Notiz wählt Constellation die Zusammenfassung aus, indem es diese Liste durchgeht und beim ersten anhält, der existiert:

1. **Ihre Frontmatter-Zusammenfassung.** Wenn die Eigenschaften der Notiz ein Feld `summary:`, `description:`, `abstract:` oder `excerpt:` enthalten (in dieser Reihenfolge geprüft), wird dessen Text **genau so verwendet, wie Sie ihn geschrieben haben**.
2. **Ihr Zusammenfassungs-Callout.** Wenn der Notiztext ein Callout `> [!summary]`, `> [!abstract]` oder `> [!tldr]` enthält, wird dessen Text **genau so verwendet, wie Sie ihn geschrieben haben** — einschließlich diakritischer Zeichen und Interpunktion, wortgetreu erhalten.
3. **Eine generierte Zusammenfassung.** Nur wenn Sie keines der beiden oben Genannten geschrieben haben, generiert Constellation eine — indem es die Notiz liest und ihre zentralsten Sätze extrahiert (siehe unten).
4. **Ein Rückgriff auf den Anfangstext.** Bei einer Notiz, die die Engine nicht in Sätze zerlegen kann (zum Beispiel Text in einer Schrift ohne klare Satzinterpunktion), zeigt sie statt einer gerankten Zusammenfassung die Anfangszeilen der Notiz.

> **Die eine Regel, die am wichtigsten ist:** Schritte 1 und 2 bedeuten, dass eine von Ihnen geschriebene Zusammenfassung *niemals* überschrieben wird. Wenn Sie eine generierte Zusammenfassung bei einer Notiz sehen, die Sie für zusammengefasst hielten, bedeutet das, dass die Engine Ihre Zusammenfassung nicht dort gefunden hat, wo sie sucht — prüfen Sie, ob Ihr Frontmatter-Feld einen der vier obigen Namen trägt oder ob Ihr Callout einer der drei obigen Typen ist.

---

## Wie eine generierte Zusammenfassung aufgebaut wird

Wenn Constellation eine Zusammenfassung generieren muss (weil Sie keine geschrieben haben), macht es eine **extraktive** Zusammenfassung — es wählt Sätze aus, die bereits in Ihrer Notiz stehen, statt neue Prosa zu erfinden. Die Methode ist eine gut etablierte (TextRank, Mihalcea & Tarau 2004):

1. **In Sätze zerlegen.** Der Notiztext wird mithilfe des Unicode-Standards für Satzgrenzen in Sätze segmentiert, sodass es sprachen- und schriftübergreifend funktioniert.
2. **Die Bedeutung jedes Satzes lesen.** Jeder Satz wird mithilfe eines kompakten Modells auf dem Gerät in einen kleinen numerischen „Bedeutungs-Fingerabdruck" (ein Embedding) umgewandelt.
3. **Nach Zentralität ranken.** Sätze, die in ihrer Bedeutung den meisten *anderen* Sätzen am ähnlichsten sind, erhalten die höchste Punktzahl — das sind die Sätze, die die Notiz als Ganzes am besten repräsentieren.
4. **Die obersten drei nehmen, in Reihenfolge.** Die drei am höchsten gerankten Sätze werden **in der Reihenfolge angezeigt, in der sie in der Notiz vorkommen**, sodass sich die Zusammenfassung natürlich liest und nicht durcheinander.

Sehr lange Notizen werden behutsam behandelt — die Engine begrenzt, wie viel vom Text sie scannt und wie viele Sätze sie rankt, sodass das Zusammenfassen einer riesigen Notiz die App niemals verlangsamt oder einen Absturz riskiert.

Weil sie extraktiv ist, besteht eine generierte Zusammenfassung immer aus Sätzen, die Sie tatsächlich geschrieben haben. Sie wird Ihnen niemals Worte in den Mund legen.

---

## Zusammenfassungen sind schreibgeschützt — File-Over-App

Constellation **schreibt eine generierte Zusammenfassung niemals in Ihre Notiz zurück.** Ihre `.md`-Dateien sind die Quelle der Wahrheit; die Zusammenfassung, die Sie auf einer Karte sehen, wird im laufenden Betrieb berechnet und separat zwischengespeichert, nicht in den Text oder das Frontmatter der Datei gespeichert.

Das ist beabsichtigt und folgt Constellations *File-Over-App*-Prinzip: Die App ist ein Fenster auf Ihre Dateien, kein Editor, der sie stillschweigend ändert. Wenn Sie möchten, dass eine Zusammenfassung *in* der Notiz lebt, schreiben Sie selbst eine (ein `summary:`-Feld oder ein `[!summary]`-Callout) — und dann zeigt Constellation gemäß der obigen Rangfolge Ihre an und hört auf zu generieren.

Alles wird **auf Ihrem Gerät** berechnet. Kein Notiztext wird jemals irgendwohin gesendet, um zusammengefasst zu werden.

---

## Wann Zusammenfassungen erscheinen und wie sie sich füllen

Zusammenfassungen tauchen unter dem Notiztitel auf jeder Karte in der **Klassifikator**- / **Source-Review**-Warteschlange auf.

Standardmäßig füllen sie sich **träge und behutsam**: Während Karten ins Sichtfeld scrollen, berechnet Constellation ihre Zusammenfassungen in kleinen Mengen und pausiert immer dann, wenn ein Library-Klassifizierungsscan läuft, sodass die beiden niemals um Ressourcen konkurrieren. Das hält die App reaktionsfähig — Sie sehen eine Karte möglicherweise kurz, bevor ihre Zusammenfassung erscheint, dann taucht die Zusammenfassung einen Moment später auf.

Wenn Sie lieber jede Zusammenfassung im Voraus bereit haben möchten, verwenden Sie **Alle Zusammenfassungen erstellen**.

---

## Alle Zusammenfassungen erstellen — die ganze Library vorberechnen

Die Schaltfläche **Alle Zusammenfassungen erstellen** (im **Klassifikator**-Header) berechnet eine Zusammenfassung für **jede Notiz vor, die noch keine aktuelle hat**, sodass Karten ihre Zusammenfassung sofort anzeigen, statt sie beim Scrollen nachzuladen.

**Um sie zu verwenden:**

1. Öffnen Sie den **Klassifikator** (das Symbol mit den gestapelten Karten im Dock am linken Rand).
2. Klicken Sie im Header auf **Alle Zusammenfassungen erstellen**. Die Schaltfläche wechselt zu *Notizzusammenfassungen werden erstellt…*.
3. Der Fortschritt erscheint in der **Statusleiste** am unteren Fensterrand — Sie können weiterarbeiten, während es läuft.
4. Um vorzeitig zu stoppen, verwenden Sie das **Abbrechen**-Steuerelement im Fortschrittsstreifen der Statusleiste. Ein Teildurchlauf ist in Ordnung; er macht beim nächsten Mal dort weiter, wo er aufgehört hat.

Ein paar Dinge, die zu wissen sich lohnen:

- Es läuft **nur, wenn Sie es verlangen** — es startet niemals von selbst, sodass es den App-Start niemals verlangsamen kann.
- Es läuft **im Hintergrund** auf einem separaten Thread; Tippen und Navigation bleiben sofort.
- Es ist **fortsetzbar** — wenn Sie es abbrechen oder die App mitten im Durchlauf schließen, fährt der nächste Durchlauf dort fort, wo er gestoppt hat, statt von vorne zu beginnen.
- Es berechnet nur Zusammenfassungen, die **fehlen oder veraltet** sind — Notizen, deren Zusammenfassung bereits aktuell ist, werden übersprungen, sodass ein zweiter Durchlauf schnell ist.

---

## Sicherstellen, dass Ihre eigene Zusammenfassung verwendet wird

Auf einer Karte erscheint die Zusammenfassung unter einem einzigen **Zusammenfassung**-Label — die Karte kennzeichnet nicht, ob der Text von Ihnen oder von der Engine stammt. Was das entscheidet, ist die obige Rangfolge: Wenn eine Notiz eines der Frontmatter-Felder oder eines der Zusammenfassungs-Callouts hat, zeigt Constellation *dieses* an und generiert niemals eines.

Wenn eine Notiz also eine Zusammenfassung anzeigt, die klingt, als hätte die Maschine sie gewählt, hat diese Notiz weder eine Frontmatter-Zusammenfassung noch ein Zusammenfassungs-Callout — und die Lösung ist, eines hinzuzufügen:

- Fügen Sie dem Frontmatter der Notiz ein `summary:`-Feld (oder `description:` / `abstract:` / `excerpt:`) hinzu, **oder**
- Fügen Sie dem Text ein `> [!summary]`-Callout (oder `[!abstract]` / `[!tldr]`) hinzu.

Beim nächsten Mal, wenn die Zusammenfassung dieser Notiz berechnet wird — wenn ihre Karte als Nächstes lädt oder nachdem Sie **Alle Zusammenfassungen erstellen** ausgeführt haben —, übernehmen Ihre Worte.

---

## Häufige Workflows

**„Eine Notiz zeigt eine Maschinen-Zusammenfassung, aber ich habe eine geschrieben."**
Constellation hat Ihre Zusammenfassung nicht dort gefunden, wo es sucht. Stellen Sie sicher, dass Ihr Frontmatter-Feld `summary`, `description`, `abstract` oder `excerpt` heißt, **oder** dass Ihr Callout `[!summary]`, `[!abstract]` oder `[!tldr]` ist. Öffnen Sie dann den Klassifikator erneut (oder klicken Sie auf *Alle Zusammenfassungen erstellen*), um zu aktualisieren.

**„Ich möchte, dass jede Karte ihre Zusammenfassung in dem Moment anzeigt, in dem ich den Klassifikator öffne."**
Klicken Sie einmal auf **Alle Zusammenfassungen erstellen** und lassen Sie es zu Ende laufen. Danach sind die Zusammenfassungen vorberechnet und erscheinen sofort.

**„Ich möchte, dass die Zusammenfassung Teil der Notiz selbst ist, auf der Festplatte."**
Schreiben Sie sie selbst — fügen Sie ein `summary:`-Frontmatter-Feld oder ein `> [!summary]`-Callout hinzu. Constellation zeigt dann Ihre Version an (und hört auf, eine zu generieren), und Ihre Worte leben in der Datei, wo jede andere App sie ebenfalls lesen kann.

---

## Verwandte Themen

- **The Cataloger** — die Vollfenster-Startseite, auf der Zusammenfassungen unter jeder Karte erscheinen und wo *Alle Zusammenfassungen erstellen* lebt.
- **Source Review** — die Klassifizierungskarten, auf denen die Zusammenfassungen sitzen.
- **Properties** — die Frontmatter-Felder `summary:` / `description:` / `abstract:` / `excerpt:` und wie man sie hinzufügt.
- **Editing and Formatting** — wie man ein `> [!summary]`-Callout in einer Notiz schreibt.

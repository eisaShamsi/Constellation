# Source Review

> **Übersetzungshinweis:** Dieses Hilfethema ist eine KI-generierte
> Übersetzung der kanonischen englischen Version unter
> `help.uConstellation.World/Source Review/Source Review.md`. Eine
> Überprüfung durch Muttersprachler steht noch aus. Bitte reichen Sie
> Korrekturen über das Projekt-Repository ein.

*(Constellation Epistemic Content Engine — CECE)*

Das Source-Review-Panel ist der Ort, an dem Constellation Sie bittet, die von der **Constellation Epistemic Content Engine** (CECE) erzeugten Klassifizierungen zu überprüfen. Jede Karte in der Warteschlange zeigt eine Notiz + die Einschätzung der Engine, wie diese Notiz in Ihre Wissens-Taxonomie passt. Sie wählen Akzeptieren, Bearbeiten, Ablehnen oder einen Sibling-Disambiguation-Chip — und mit der Zeit lernt die Engine die Form Ihrer Library.

Dieses Thema erklärt jeden Teil einer Source-Review-Karte, was die farbigen Punkte bedeuten, wann Sie der Engine vertrauen können und wie Sie durch Hunderte von Karten navigieren, ohne endlos zu scrollen.

---

## Was CECE tatsächlich tut

Wenn Sie eine Notiz klassifizieren (Rechtsklick → "Quellen & Inhaltstyp vorschlagen", oder über Einstellungen → Schaltfläche Scan ausführen), führt CECE **sechs unabhängige Katalogisierer** auf der Notiz aus. Jeder Katalogisierer liest die Notiz durch seine eigene Linse — Frontmatter, Zitate, Wortstämme, verlinkte Notizen, ähnliche Notizen, KI-Urteil — und stimmt zu zwei Fragen ab:

- **Source (horizontale Achse)**: Woher *kommt* dieses Wissen? Beispiele: Zeugnis (jemand hat es mir gesagt), Wahrnehmung (ich habe es gesehen), Inferenz (ich habe es abgeleitet), Offenbarung (heiliger Text) und acht weitere.
- **Content Type (vertikale Achse)**: Was *für eine Art* von Wissen ist das? Beispiele: epistemischer Zustand (Zweifel / Gewissheit / Glaube), semantischer Inhalt (Konzept / Aussage / Tatsache / Theorie), sensorische Eingabe, symbolische Entität, Konstrukt höherer Ordnung (Weltanschauung / Doktrin).

Die beiden Achsen sind **unabhängig**. Eine Notiz über "Ich bezweifle die Mondlandung" ist Zeugnis (jemand hat es berichtet) auf der Source-Achse + epistemische-Zustände/Zweifel (die Haltung des Benutzers dazu) auf der Content-Type-Achse.

Nachdem die Katalogisierer abgestimmt haben, kombiniert eine **Synthese-Schicht** ihre Stimmen zu einer einzigen Klassifizierung pro Achse, mit einem von drei Konfidenzregimen:

- **Unanimous** — jeder sich äußernde Katalogisierer war sich einig
- **Strong majority** — die meisten waren sich einig, einer war anderer Meinung (Karte zeigt den Namen des Andersmeinenden)
- **Split** — keine klare Mehrheit; die Engine "hat sich geweigert zu entscheiden" und bittet *Sie*, zu wählen

Alles läuft **auf Ihrem Gerät**. Keine Notizen verlassen jemals Constellation.

---

## Die zwei Achsen in einfacher Sprache

### Source — *woher kommt dieses Wissen?*

Elf mögliche Werte plus *unklassifizierbar*:

- **Wahrnehmung** — sensorische Beobachtung aus erster Hand
- **Inferenz** — Schlussfolgern aus Prämissen (Deduktion, Induktion, Analogie)
- **Zeugnis** — der Bericht eines anderen (ein Zitat, eine Quellenangabe, eine referenzierte Quelle)
- **Massenübertragung** — konvergente Berichte vieler unabhängiger Zeugen (sunnitisch *al-tawatur*)
- **Vergleich** — Wissen durch Analogie zu einem bekannten Fall (juristisch *qiyās*, wissenschaftliche Analogien)
- **Postulation** — Inferenz zur besten Erklärung (*arthapatti*)
- **Nicht-Wahrnehmung** — Wissen über Abwesenheit
- **Erinnerung** — Erinnerung an vergangene Erfahrungen
- **Angeborene Disposition** — vorerfahrungsmäßiges Wissen (*fitrah*)
- **Inspiration** — mystische oder kreative Erfassung (*kashf*)
- **Offenbarung** — Übertragung durch heiligen Text oder Propheten (*al-wahy*)
- **Unklassifizierbar** — diese Klassifizierung überspringen

### Content Type — *was für eine Art von Wissen ist das?*

Fünf Hauptzweige mit Unterzweigen:

- **Sensorische Eingaben** — Rohsignale (visuell, akustisch, chemisch, …)
- **Symbolische Entitäten** — Zeichen, Symbole, Codes
- **Semantische Inhalte** — Konzepte, Aussagen, Fakten, Ideen, Informationen
- **Epistemische Zustände** — Zweifel, Glaube, Meinung, Gewissheit, Wissen, Illusion
- **Konstrukte höherer Ordnung** — Theorien, Doktrinen, Weltanschauungen, Paradigmen

Beide Achsen haben mehrere Verfeinerungsebenen unter jedem Hauptwert (z. B. ist *epistemic-states/knowledge/by-content/propositional* ein Blatt).

---

## Die sechs Katalogisierer

Jeder Katalogisierer ist eine *Linse*, durch die CECE eine Notiz liest. Die Source-Review-Karte zeigt sie als **sechs kleine farbige Punkte** in der oberen rechten Ecke. Bewegen Sie den Mauszeiger über einen Punkt, um seinen Namen + Status zu sehen.

| Punkt | Katalogisierer | Was er liest |
|---|---|---|
| 🔵 blau | **Ihr Frontmatter** (Benutzerautorität) | Die Felder `sources:` und `content_type:`, die Sie bereits gesetzt haben. Wenn Sie die Notiz selbst klassifiziert haben, hat diese Linse *absolute Autorität* — die Synthese übernimmt Ihre Wahl und überspringt die anderen. |
| 🌹 rosé | **Zitate & Struktur** (Strukturell) | Zitate, Blockquotes, Codeblöcke, Theorem-Marker, Definitionsphrasen ("das Konzept von X ist definiert als…"), Abbildungsverweise. Liest die strukturelle Form der Notiz. |
| 🟡 bernsteinfarben | **Wortstämme & Lexikon** (Linguistisch) | Arabische Wurzelanalyse (CAE), Oberflächen-Schlüsselwortabgleich, sprachübergreifende Begriffsäquivalenz (Bridge). Erkennt arabisch-bewusste Klassifizierung, die reine Embeddings übersehen. |
| 🟢 türkis | **Verlinkte Notizen** (Graph) | Typisierte Living Links (`[[Note\|supports]]`, `[[Note\|contradicts]]` usw.) zu anderen klassifizierten Notizen. Erbt die Klassifizierung von Nachbarn, wenn diese sich gruppieren. |
| 🟣 violett | **Ähnliche Notizen** (Semantisch) | Embedding-Ähnlichkeit zu Ihren bereits klassifizierten Notizen (k-Nearest-Neighbor). Zieht den Konsens heran, wenn der Inhaltsvektor dieser Notiz mit klassifizierten Notizen gruppiert. |
| 🟢 grün | **KI-Urteil** (Reasoning) | Ein lokales LLM (Qwen3-4B Q5_K_M), das Grammatik-eingeschränkte Inferenz ausführt. *Noch nicht aktiv* — Modell-Verdrahtung auf eine spätere Version verschoben. Der Punkt bleibt heute auf jeder Karte stumm. |

### Punkt-Status

- **Gefüllt** — geäußert + stimmt mit der Synthese überein
- **Ringförmig** — geäußert + abweichend von der Synthese (diese Linse hat etwas anderes gewählt)
- **Gestrichelter Umriss** — stumm (kein Signal in dieser Linse für diese Notiz)

Der Punktcluster ist der auf-einen-Blick-Indikator für die Ensemble-Gesundheit. Eine Karte mit allen sechs gefüllten Punkten ist die stärkstmögliche Klassifizierung der Engine (selten). Eine Karte mit einem oder zwei ringförmigen Punkten zeigt ihre Argumentation ehrlich — die Linsen waren sich uneinig.

---

## Die drei Konfidenzregime

Nachdem die Katalogisierer abgestimmt haben, kennzeichnet CECE jede Achse mit einem von drei Regimen:

- **Unanimous** — jeder sich äußernde Katalogisierer wählte denselben Primärwert. Die Karte hat keine spezielle Pille.
- **Strong majority (eine Abweichung)** — die meisten waren sich einig; ein Andersmeinender wird namentlich angezeigt. Die Karte hat eine violette "Strong majority"-Pille im Header.
- **Split** — keine klare Mehrheit. Die Karte hat eine goldene "Catalogers split — needs your call"-Pille, **einen goldenen linken Rand** und ein Sibling-Disambiguation-Formular mit Chips zur Auswahl.

Jede Achse erhält ihr eigenes Regime unabhängig. Eine Karte kann horizontal Unanimous + vertikal Split sein (oder umgekehrt). Die Header-Pille fasst das schlechteste Regime über beide Achsen hinweg zusammen.

---

## Sibling Disambiguation

Wenn eine Achse Split ist, weigert sich CECE zu raten und bringt stattdessen die Kandidatenwerte als **Radio-Chips** unter einer Aufforderung zum Vorschein:

> *"Die Katalogisierer haben sich zwischen diesen Kandidaten gespalten. Wählen Sie, welcher am besten zur Notiz passt:"*

Sie klicken auf einen Chip → die Engine schreibt diese Wahl in das Frontmatter der Notiz, entfernt die Karte aus der Warteschlange und aktualisiert die Zuverlässigkeitsdaten pro Library.

Wenn die ANDERE Achse geklärt war (Unanimous oder Strong majority), schreibt CECE *auch* den Wert dieser Achse zur gleichen Zeit — sodass ein einziger Chip-Klick beide Achsen abschließt, nicht nur die, die Sie ausgewählt haben. Dieselbe Karte fragt Sie nie zweimal.

Wenn beide Achsen Split sind, wählen Sie einen Chip pro Achse (zwei Klicks).

---

## Die Argumentationsspur

Jede Karte hat einen Umschalter **"▸ Warum diese Klassifizierung?"** (oder "▾ Argumentation ausblenden", wenn geöffnet). Beim Aufklappen wird eine Zeile pro sich äußerndem Katalogisierer angezeigt:

- **Linsenfarbiger Punkt**, der mit dem Punktcluster übereinstimmt
- **Katalogisierer-Etikett** (z. B. "Wortstämme & Lexikon")
- **Selbstgemeldete Konfidenz** in Klammern: `[high]` `[medium]` `[low]`
- **Einzeilige Argumentation**, die erklärt, was ausgelöst wurde (z. B. *"Linguistic match: vertical → semantic-contents/concept (weight 0.80)"*)
- **Freundliche Regel-Chips** unter der Argumentation, wie `Surface keyword match`, `Side-channel preference rule`, `Arabic root match (CAE)` — dies sind die spezifischen Regeln, die jeder Katalogisierer ausgelöst hat

Während Ihrer **ersten 50 Überprüfungen** klappt die Spur auf jeder Karte automatisch auf, damit Sie ein Gefühl dafür entwickeln können, wann der Engine zu vertrauen ist. Danach klappt die Spur auf Unanimous-Karten auf Anfrage zusammen und bleibt auf Strong-majority- + Split-Karten automatisch aufgeklappt (wo die Meinungsverschiedenheit informativ ist).

Sie können diese Standardeinstellung jederzeit unter Einstellungen → Intelligenz → CECE → Sichtbarkeit der Argumentationsspur überschreiben:

- **Immer anzeigen** — auf jeder Karte geöffnet
- **Nur bei Meinungsverschiedenheit (Standard)** — geöffnet auf Split- + Strong-majority-Karten plus den ersten 50 Überprüfungen
- **Immer ausblenden** — manuelles Klicken zum Aufklappen erforderlich

---

## Der Filter für die Warteschlangenzusammensetzung

Über dem Zähler-Streifen befinden sich **fünf Chips**, die Ihre Warteschlange nach der Art der Entscheidung aufteilen, die jede Karte von Ihnen benötigt:

| Chip | Zeigt |
|---|---|
| **All** *(Standard)* | die vollständige Warteschlange |
| **Both axes need your call** | Karten, bei denen SOWOHL horizontal ALS AUCH vertikal Split sind |
| **Source needs your call** | Karten, bei denen horizontal Split ist + vertikal geklärt ist |
| **Content type needs your call** | Karten, bei denen vertikal Split ist + horizontal geklärt ist |
| **Catalogers agreed** | Karten, bei denen keine Achse Split ist — schnelle Stempelkandidaten |

Jeder Chip zeigt seinen Eimerzähler an (z. B. *"Source needs your call (43)"*). Leere Eimer werden gedimmt und deaktiviert. Beim Klicken auf einen Chip werden die sichtbaren Karten neu gerendert; der Zählerstreifen und die Approve-All-Mathematik arbeiten immer auf der **vollständigen** Warteschlange unabhängig vom aktiven Filter, sodass Sie stets die wahren Gesamtzahlen sehen können.

Der Filter löst das Nadel-im-Heuhaufen-Problem, wenn Ihre Warteschlange Hunderte von Karten enthält. Möchten Sie zuerst alle Stempelkandidaten löschen? Klicken Sie auf **Catalogers agreed** und dann auf **Approve all**. Möchten Sie sich auf die schwierigsten Fälle konzentrieren? Klicken Sie auf **Both axes need your call**.

---

## Aktionen pro Karte

Jede Karte hat unten vier Aktionen (oder drei auf Split-Karten, bei denen Disambig Accept/Edit ersetzt):

- **Accept** — schreibt den primären Wert der Synthese der Engine auf beiden Achsen in das Frontmatter der Notiz, entfernt die Karte aus der Warteschlange. Aktualisiert die Zuverlässigkeit pro Katalogisierer.
- **Edit** — öffnet einen Tree-Picker für beide Achsen; Sie wählen die Werte manuell. Gleiche Zuverlässigkeitsaktualisierung.
- **Reject** — räumt die Karte ohne etwas zu schreiben. Die Engine wird erneut vorschlagen, wenn Sie später erneut klassifizieren. (Ablehnung aktualisiert die Zuverlässigkeit NICHT — der Benutzer "möchte keinen davon" ist ein mehrdeutiges Feedbacksignal.)
- **Sibling-Disambiguation-Chip** — auf Split-Karten klicken Sie auf einen der Kandidaten-Chips. Schreibt den ausgewählten Wert (und schreibt automatisch die andere Achse, wenn sie geklärt war).

---

## Die Vertrauenskalibrierungsphase

Ihre ersten **50 Überprüfungen** von CECE-klassifizierten Karten sind eine *Vertrauenskalibrierungsphase*. Während dieser Zeit klappt die Argumentationsspur auf jeder Karte automatisch auf (unabhängig vom Regime), und ein dezentes Banner oben im Panel erinnert Sie: *"Showing reasoning trails until you review N more cards — helps you learn when to trust the catalogers."*

Nach 50 Überprüfungen verschwindet das Banner und die Spuren klappen auf das standardmäßige On-Demand-Verhalten zusammen. Sie können dies über die Einstellungen überschreiben, wenn Sie sie immer geöffnet oder immer geschlossen halten möchten.

Der Sinn der Kalibrierungsphase: CECE ist ein probabilistisches System, das besser wird, wenn Sie es korrigieren (Zuverlässigkeit pro Library). Zu sehen, *warum* jeder Katalogisierer so abgestimmt hat, wie er es während der ersten 50 Überprüfungen tat, ermöglicht es Ihnen, Ihre eigene Intuition dafür zu entwickeln, wann seine Schlussfolgerungen für den spezifischen Inhalt dieser Library vertrauenswürdig sind.

---

## Kalibrierung pro Library

Einstellungen → Intelligenz → CECE → **Per-Library calibration** öffnet eine schreibgeschützte Tabelle, die die Genauigkeit jedes Katalogisierers pro Achse in der aktiven Library anzeigt:

```
Cataloger          Horizontal      Vertical
─────────          ──────────      ────────
Your frontmatter   12/12 (100%)    4/4 (100%)
Citations          18/22 (82%)     6/8 (75%)
Wordstems          24/28 (86%)     20/26 (77%)
Linked notes       3/4 (uniform)   2/3 (uniform)
Similar notes      14/19 (74%)     12/19 (63%)
AI judgment        — (not running) — (not running)
```

Die Zahlen sind richtig/gesamt-Zählungen. Der Prozentsatz wird angezeigt, nachdem ein Katalogisierer 20+ Korrekturen für diese Library × Achse hat (die Schwelle für stabile Genauigkeitsdaten). Unterhalb der Schwelle zeigt das Etikett **(uniform)** — der Katalogisierer trägt gleichmäßig gewichtete Stimmen bei, bis sich genügend Daten angesammelt haben.

Verschiedene Libraries können sehr unterschiedliche Genauigkeiten pro Katalogisierer haben. Der linguistische Katalogisierer glänzt bei arabisch-lastigen Libraries; der Graph-Katalogisierer glänzt bei dicht verlinkten Libraries. Die Synthese-Schicht verwendet die Kalibrierungsdaten pro Library, um Stimmen zu gewichten — sodass ein Katalogisierer, der in *dieser* Library zu 70 % falsch lag, in der nächsten Synthese-Runde mit weniger Gewicht versehen wird.

---

## Hintergrundklassifizierung

Die Source-Review-Warteschlange kann auf zwei Arten wachsen:

1. **Manuell** (Standard) — Sie klicken mit der rechten Maustaste auf eine Notiz → "Quellen & Inhaltstyp vorschlagen" oder Sie lösen Einstellungen → Klassifizierungsscan ausführen aus.
2. **Hintergrund** — Einstellungen → Intelligenz → CECE → Hintergrundklassifizierung. Zwei Modi:
   - **On note save** — automatische Klassifizierung jeder Notiz ~1,5 Sekunden nachdem Sie aufhören zu tippen (reitet auf dem bestehenden Debounced-Save; löst nie pro Tastenanschlag aus).
   - **On app start** — scannt unklassifizierte Notizen einmal pro Start.

Die Hintergrundklassifizierung ist standardmäßig **deaktiviert**. Beide Hintergrundmodi laufen auf einem Hintergrund-Thread + senden Fortschrittsereignisse aus; die Eingabe bleibt sofort; Sie können vom Header des Source-Review-Panels abbrechen.

---

## Häufige Workflows

**"Ich habe gerade CECE installiert — wo fange ich an?"**
Öffnen Sie das Source-Review-Panel. Klicken Sie mit der rechten Maustaste auf 5-10 Notizen aus Ihrem Dateibaum → "Quellen & Inhaltstyp vorschlagen", um die Warteschlange zu seeden. Klicken Sie sich Karte für Karte durch. Die Argumentationsspur klappt während Ihrer ersten 50 Überprüfungen automatisch auf — lesen Sie sie. Nach 5-10 Karten werden Sie beginnen zu sehen, welche Katalogisierer für Ihren Inhalt zuverlässig sind.

**"Meine Warteschlange hat 1.200 Karten — wo soll ich mich konzentrieren?"**
Verwenden Sie die Filter-Chips. Beginnen Sie mit **Catalogers agreed** (Stempelkandidaten) → klicken Sie auf Approve all, um sie zu löschen. Dann **Source needs your call** + **Content type needs your call** für Split-Fälle, die je eine Entscheidung benötigen. **Both axes need your call** ist die schwerste Gruppe; heben Sie sie für zuletzt auf.

**"Wie weiß ich, wann ich Accept vs. Reject vs. Edit vs. Disambig wählen soll?"**
- **Accept**, wenn der primäre Wert der Synthese mit Ihrer Lesart der Notiz übereinstimmt.
- **Reject**, wenn keiner der Vorschläge passt (z. B. die Engine hat etwas übersehen, das Sie über die Notiz wissen).
- **Edit**, wenn Sie einen Wert wünschen, der in keinem der Vorschläge enthalten ist.
- **Sibling-Disambiguation-Chip**, wenn die Karte Split ist und einer der Kandidaten korrekt ist.

**"Wie sehe ich, welchen Katalogisierern ich am meisten vertraue?"**
Öffnen Sie Einstellungen → Intelligenz → CECE → Per-Library calibration. Die Tabelle zeigt die Genauigkeit pro Katalogisierer über die Korrekturen, die Sie in dieser Library vorgenommen haben.

---

## Verwandte Themen

- **Cognitive Engine** — die umfassendere Wissensformulierungsphilosophie, in die CECE passt.
- **Properties** — die Frontmatter-Felder `sources:` und `content_type:`, in die CECE schreibt.
- **Knowledge Hierarchy** — wie Source × Content Type in die Universe / Library / Folder / Note-Struktur passt.

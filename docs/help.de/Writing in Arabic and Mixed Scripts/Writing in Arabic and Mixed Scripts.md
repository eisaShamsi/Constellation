# Schreiben auf Arabisch und in gemischten Schriften

Constellations Editor ist von Grund auf sprachorientiert gebaut: Arabisch, Hebräisch, Persisch, Urdu und zweisprachige Notizen sind kein nachträglicher Zusatz — die Einfügemarke, die Auswahl und die Richtung jedes Absatzes folgen denselben Regeln, die Microsoft Word unter Windows verwendet, sodass Ihr Muskelgedächtnis direkt übertragbar ist. Dieses Thema behandelt alles rund um das *Schreiben* in Rechts-nach-links-Text und gemischtem Text: wie sich die Einfügemarke bewegt, wie Sie nach Wort, Satz, Zeile, Absatz oder Bildschirmseite auswählen und wie Sie die Richtung eines Absatzes erzwingen, wenn die automatische Erkennung nicht das ist, was Sie wollen.

(Wie Constellation Arabisch *versteht* — Wurzeln, Suche und die morphologische Engine — beschreibt das Hilfethema **Arabische Engine**.)

---

## Wie sich die Einfügemarke bewegt

- **Die Pfeiltasten bewegen die Einfügemarke um jeweils ein Zeichen des Textes, in Lesereihenfolge** — nie um eine Position auf dem Bildschirm. In rein arabischem oder rein englischem Text sieht das genauso aus wie der gedrückte Pfeil. An einer Nahtstelle zwischen Arabisch und Englisch (etwa in einem arabischen Satz, der ein englisches Wort enthält) schreitet die Einfügemarke Zeichen für Zeichen in Schreibreihenfolge voran und „springt" sichtbar über die Nahtstelle — dieser Sprung ist korrekt; er verhindert, dass sich die Einfügemarke an der Grenze wie festgefahren anfühlt.
- **Home** springt an den **Leseanfang** der Zeile — bei einer arabischen Zeile ist das der *rechte* Rand. **End** springt an das **Leseende** — den *linken* Rand. Halten Sie zusätzlich **Shift** gedrückt, um bis zum jeweiligen Rand auszuwählen.
- **Enter** in einer arabischen Zeile setzt die Einfügemarke der neuen Zeile nach **rechts** — die natürliche Schreibposition.
- Ein **lateinisches Wort am Ende einer arabischen Zeile** behält eine klare, stabile Einfügemarke, statt seine Richtung zu verlieren.

Jede dieser Regeln gilt identisch im Standard-Editor, im Fokusmodus und in der Zusammenführungsansicht bei Konflikten.

---

## Auswählen nach Einheit

Jede Texteinheit hat ihren schnellen Auswahlweg — in jeder Sprache und jeder Mischung:

| Einheit | So geht's |
|---|---|
| **Wort** | Doppelklick darauf |
| **Satz** | **Ctrl+Klick** an beliebiger Stelle im Satz — oder **Ctrl+Shift+S**, während die Einfügemarke darin steht |
| **Zeile** | **Ctrl+L** |
| **Absatz** | **Ctrl+Shift+L** — oder Dreifachklick |
| **Bildschirmseite** | **Shift+Page Down** / **Shift+Page Up** |
| **Alles** | **Ctrl+A** |

Wissenswerte Details:

- **Die Satzauswahl versteht arabische Interpunktion.** Ein Satz endet an **؟ ۔ !** und am Punkt — das arabische Semikolon **؛** ist dagegen eine Pause *innerhalb* eines Satzes, die Auswahl läuft also korrekt darüber hinweg. Dezimalzahlen wie 3.14 zerteilen niemals einen Satz.
- Ein **Absatz** ist ein Textblock mit einer Leerzeile darüber und darunter — genau wie in Word. Zeilen- und Absatzauswahlen schmiegen sich an den Text: In einer arabischen Zeile endet die Markierung bei den Wörtern, statt sich über die leere linke Seite zu erstrecken.
- Ctrl+Klick *ersetzt* die frühere Geste „weiteren Cursor hinzufügen" auf dieser Taste — der Klick löst jetzt die Satzauswahl aus.

## Absatzweise bewegen

- **Ctrl+↓** springt an den Anfang des **nächsten** Absatzes; **Ctrl+↑** an den Anfang des **aktuellen** (erneut drücken für den vorherigen). Mit zusätzlichem **Shift** wählen Sie beim Springen Absatz für Absatz aus. Das ist die Word-Konvention, und „nächster" bedeutet schlicht: weiter unten auf der Seite — es funktioniert in arabischen, englischen und gemischten Notizen identisch.

---

## Die Richtung eines Absatzes erzwingen

Constellation erkennt die Richtung jeder Zeile automatisch anhand ihrer ersten Buchstaben. Meistens ist das genau richtig — aber manchmal möchten Sie es übersteuern: ein arabischer Absatz, der mit einem englischen Markennamen beginnt, oder ein überwiegend englischer Absatz, der von rechts nach links gelesen werden soll.

**Drücken Sie Ctrl+Shift auf der RECHTEN Seite Ihrer Tastatur und lassen Sie wieder los** → der Absatz, in dem der Cursor steht, wird zu **100 % rechts-nach-links**.
**Drücken und Loslassen von Ctrl+Shift auf der LINKEN Seite** → **100 % links-nach-rechts**.

Das ist die Microsoft-Word-Konvention. Wissenswert:

- **Der Wechsel greift beim Loslassen** — beide Tasten zusammen drücken, loslassen und dazwischen nichts anderes drücken. Genau deshalb funktionieren Ctrl+Shift+S, Ctrl+Shift+L und alle anderen Kürzel unverändert weiter: Sobald eine dritte Taste dazukommt, tritt der Richtungswechsel zurück.
- **Es ist eine harte Überschreibung** — sie schlägt die automatische Erkennung und gilt für den ganzen Absatz (beziehungsweise für jeden Absatz, den eine Auswahl berührt).
- **Sie wird im Text selbst gespeichert**, als unsichtbares Richtungszeichen — sie übersteht also das Schließen der Notiz, den Neustart der App und die Synchronisierung, und sie wandert sogar mit, wenn Sie den Text in Word oder Obsidian einfügen.
- **Ein einziges Ctrl+Z macht sie rückgängig.** Zweimal dieselbe Seite zu drücken bewirkt nichts Zusätzliches.
- **Markdown bleibt unversehrt.** Listen bleiben Listen, Überschriften bleiben Überschriften, Zitate bleiben Zitate. Codeblöcke, Tabellen und horizontale Trennlinien werden bewusst nicht angetastet. Eine Zeile, die mit einem #Tag *beginnt*, behält ihre automatische Richtung (ein erzwungenes Zeichen an dieser Stelle würde den Tag zerstören) — der Rest des Absatzes wechselt trotzdem.

---

## Schriften und die Oberfläche

- **Skript-Schriftarten**: Konfigurieren Sie arabische, hebräische und CJK-Schriften unabhängig voneinander unter **Einstellungen → Sprache**.
- **Skript-Werkzeugleisten**: sprachspezifische Schaltflächen für Symbole und Interpunktion.
- **Taschkīl-Hervorhebung**: Die Hervorhebung arabischer Diakritika schalten Sie in der Editor-Werkzeugleiste um.
- Wählen Sie Arabisch oder Hebräisch als Oberflächensprache, wechselt die gesamte App auf RTL.

---

## Glossar

- **Lesereihenfolge** — die Reihenfolge, in der die Zeichen geschrieben und gelesen werden, unabhängig davon, wo sie auf dem Bildschirm stehen.
- **Nahtstelle** — die Grenze zwischen einem Rechts-nach-links-Lauf und einem Links-nach-rechts-Lauf in derselben Zeile.
- **Harte Überschreibung** — eine ausdrücklich von Ihnen gesetzte Richtung, die die automatische Erkennung anhand der ersten Buchstaben schlägt.
- **Richtungszeichen** — das unsichtbare Zeichen (RLM/LRM), das Ihre Überschreibung im Text selbst speichert.

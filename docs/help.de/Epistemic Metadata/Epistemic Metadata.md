# Epistemische Metadaten

> **Übersetzungshinweis:** Dieses Hilfethema ist eine KI-generierte
> Übersetzung der kanonischen englischen Version unter
> `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md`.
> Eine Überprüfung durch Muttersprachler steht noch aus. Bitte reichen
> Sie Korrekturen über das Projekt-Repository ein.

*(MIG-022 §A — Schemaerweiterungen aus der Lückenanalyse §6.1)*

Dieses Thema beschreibt eine kleine Reihe von **optionalen Frontmatter-Feldern**, die Constellation jetzt für eine reichhaltigere epistemische Klassifizierung deiner Notizen erkennt. Sie wurden als Reaktion auf die Lückenanalyse (`docs/epistemic-content-gap-analysis.md`) hinzugefügt — die Erkenntnis, dass das zweiachsige Source-×-Content-Type-Modell, gegen das die Constellation Epistemic Content Engine (CECE) klassifiziert, nicht alles abdeckt, was du darüber festhalten möchtest, wie du zu deinem Wissen gelangt bist.

Diese Felder sind **alle optional**. Bestehende Notizen ohne sie funktionieren unverändert. Du fügst sie von Hand hinzu (oder in Zukunft über einen strukturierten Editor), wenn eine Notiz die Art von Wissen ist, die vom zusätzlichen Signal profitiert.

---

## Die Felder

### `held_by` — *wessen Position ist das?*

Eine kurze Zeichenfolge, die angibt, wer die in der Notiz beschriebene Position vertritt. Standardmäßig `user` (deine eigene Position). Andere Werte, die du verwenden könntest:
- Den Namen eines Gelehrten: `held_by: "al-Shāfiʿī"`
- Eine Schule: `held_by: "Ḥanafī"`
- Eine historische Figur: `held_by: "Aristotle"`

Wenn du eine Notiz schreibst, die *die Position einer anderen Person* statt deiner eigenen festhält, ist `held_by` das Feld, das das aussagt. Ohne es nimmt Constellation stillschweigend an, dass der epistemische Zustand der Notiz dein eigener ist — was für ernsthafte wissenschaftliche Arbeit oft falsch ist.

### `domain` — *worum geht es inhaltlich?*

Eine Liste disziplinärer Tags. Im Unterschied zu deinem freien `tags`-Feld (Folksonomie / Stimmung / Projekt) ist `domain` das strukturierte Disziplin-/Themenfeld für Abruf und Filterung. Beispiele:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

Eine als `content_type: "proposition"` UND `source: "inference"` klassifizierte Notiz könnte ein Logiksatz (domain: `[logic, mathematics]`) oder eine Rechtsmeinung (domain: `[fiqh, ʿibādāt]`) sein — gleiche epistemische Form, sehr unterschiedliche Abrufkontexte. `domain` lässt dich sagen, welcher davon.

### `function` — *wofür ist diese Notiz?*

Eine einzelne Zeichenfolge, die den vorgesehenen Zweck der Notiz benennt. Erkannte Werte:

- `reference` — bei Bedarf nachlesen (eine Definition, ein Zitat, ein Fakt zum späteren Nachschlagen)
- `seed` — inkubieren (eine frühe Idee, an der du noch arbeitest)
- `actionable` — etwas damit tun (eine Aufgabe, eine Folgemaßnahme, eine zu treffende Entscheidung)
- `shipped` — fertiges Produkt (ein veröffentlichter Aufsatz, eine gelieferte Analyse, ein abgeschlossener Vorgang)

Im Unterschied zur Content-Type-Achse von CECE (die sagt, welche ART von Wissen es ist) — `function` sagt, was du mit der Notiz TUN wirst.

### `provenance_civilization` — *welches Traditionsvokabular ist im Spiel?*

Eine optionale Zeichenfolge, die den zivilisatorischen Fußabdruck des Vokabulars der Notiz benennt. Nützlich für den Abruf gegen traditionsspezifische Korpora. Beispiele:

- `provenance_civilization: "sunni-usuli"` — Sunnitische *uṣūl al-fiqh*-Tradition (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — analytische Philosophie nach Frege
- `provenance_civilization: "nyaya"` — indische Nyāya-Schule der pramāṇa-Epistemologie
- `provenance_civilization: "buddhist-pramana"` — buddhistische epistemologische Tradition (Dignāga, Dharmakīrti)

Die meisten Notizen brauchen das nicht. Wenn du etwa eine Notiz hast, die sich sowohl auf sunnitische *uṣūl* ALS AUCH auf die analytische angloamerikanische Epistemologie stützt, hilft das Festhalten des primären Fußabdrucks dem zukünftigen Du, das passende Vergleichsmaterial zu finden.

### `updated_at` — *wann hat sich deine Position zuletzt geändert?*

ISO-Datum der jüngsten bewussten Überarbeitung des epistemischen Inhalts der Notiz. Im Unterschied zum Dateisystem-Zeitstempel `modified` (der jedes Speichern erfasst, sogar Tippfehlerkorrekturen); `updated_at` ist der Zeitstempel, den DU setzt, wenn du die Position tatsächlich neu durchdacht hast.

```yaml
updated_at: 2026-05-09
```

Nützlich, wenn der Rest der zeitlichen Achse aus §6.3 landet (Notiz-Zustandshistorie) — bis dahin ist dies ein Einzel-Snapshot-Feld, das festhält: „der letzte Zeitpunkt, zu dem ich meine Sicht überarbeitet habe."

### `ikhtilāf` — *strukturierte gelehrte Meinungsverschiedenheit*

Das komplexeste der neuen Felder. Hält *ikhtilāf* fest — die strukturierte Meinungsverschiedenheit zwischen Gelehrten oder Schulen zu einer Frage — als Liste von `{school, position}`-Paaren. Constellation stellt ein eigenes Properties-Panel-Widget zum Bearbeiten bereit; du kannst auch das YAML direkt bearbeiten.

Beispiel:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

Eine Notiz mit `ikhtilāf` befindet sich in keinem einzelnen epistemischen Zustand — sie hält eine *strukturierte Meinungsverschiedenheit* zwischen mehreren Akteuren fest. Ohne dieses Feld würde Constellation eine solche Notiz so behandeln, als hielte sie selbst eine dieser Positionen, was falsch ist.

Das Properties-Panel rendert jede Zeile als Editor-Karte mit zwei Eingabefeldern (school + position) plus einer Entfernen-Schaltfläche und einer „Schule hinzufügen"-Schaltfläche unten.

### `warrant` und `warrant_notes` — *geparst, aber inert (vorerst)*

Zwei Felder werden geparst und auf der Festplatte gespeichert, aber **noch in keiner UI angezeigt**:

- `warrant: "mutawātir"` — eine Stufenbezeichnung für die Berechtigung des Anspruchs der Notiz. Die sunnitische *uṣūl*-Hierarchie verwendet *mutawātir / mashhūr / āḥād* und speziell beim Hadith *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*. Andere Traditionen haben ihre eigenen Bewertungsvokabulare.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — Freitext zur Stützung der Berechtigungsstufe.

Diese sind einsatzbereit, sobald der **Constellation Warrant Research Workstream** seinen Klassifikator ausliefert (mehrmonatiges Forschungsprojekt; siehe Lückenanalyse §6.2). Bis dahin kannst du sie von Hand füllen, und die Daten bleiben erhalten; nichts zeigt sie an. Zukünftige berechtigungsfähige Abfragen und Badges lesen diese Werte direkt.

---

## Wo diese Felder erscheinen

Wenn du eines der neuen Felder im Frontmatter einer Notiz ausfüllst, erscheinen sie im **Properties-Panel** (rechte Seitenleiste) genauso wie jedes andere YAML-Feld — eine Zeile pro Schlüssel, mit dem typgerechten Editor:

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → Texteingabe
- `domain` → Tag-Liste (mit Tippen + Enter hinzufügen, mit dem × auf jedem Tag entfernen)
- `updated_at` → Datumsauswahl
- `ikhtilāf` → eigenes Widget mit `school`- / `position`-Zeilen + Hinzufügen/Entfernen-Schaltflächen

---

## Was ist mit `supersedes`?

`supersedes` ist technisch gesehen eine *Beziehung zwischen Notizen* und nicht eine Eigenschaft einer einzelnen Notiz. Constellation behandelt es als **typisierten Link**, nicht als YAML-Skalar:

```markdown
Diese Notiz ersetzt meine frühere Analyse: [[old-note-id|supersedes]]
```

Das `|supersedes`-Suffix am Wikilink sagt Constellation, dass dies ein typisierter Link der Art `supersedes` ist — er bekommt eine eigene Pillenfarbe (schiefer-blaugrau), erscheint in den Backlinks- + Outgoing-Links-Panels neben anderen typisierten Links und nimmt an der Living Link Architecture (Gewicht, Lebenszyklus, Traversierungszählung) teil.

Dies hält Beziehungen zwischen Notizen an einem Ort — im typisierten Linksystem — statt sie zwischen typisierten Links und Frontmatter-Skalaren aufzuteilen. Dasselbe gilt für `contradicts:` (bereits ein typisierter Link im Vor-MIG-022-Vokabular).

---

## Was das NICHT ist

Diese Felder werden heute **NICHT** von der CECE-Klassifizierung konsumiert. CECE klassifiziert nur über Source × Content Type; die neuen Metadaten-Felder werden für menschengesteuerten Abruf, zukünftige berechtigungsfähige Klassifikatoren und die zeitliche Achse (sobald sie ausgeliefert wird) erfasst.

Insbesondere:
- `function: "actionable"` erstellt NICHT automatisch eine Aufgabe im Tasks-Panel
- `held_by: "al-Shāfiʿī"` ändert NICHT, wie CECE die Notiz klassifiziert
- `domain: [fiqh]` filtert deine Suchergebnisse NICHT, es sei denn, du schreibst die Suchanfrage so, dass sie es einschließt

Die Felder sind **Schema** — ein anerkanntes Vokabular für Felder, die du hinzufügen kannst. Zukünftige MIGs liefern Funktionen aus, die sie konsumieren (Berechtigungsklassifikator, zeitliche Abfragen, domänensensitive Filterung usw.).

---

## Ein durchgearbeitetes Beispiel

Eine Notiz, die die Positionen der sunnitischen Schulen dazu festhält, ob das Anbrechen der Morgendämmerung der Fastenpflicht für die Gültigkeit des Tages relevant ist:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

Die klassische Mālikī-Position (eine niyyah für den Monat) wird beschrieben
von [[Ibn-Rushd-bidayah|derives-from]] in der Passage zu niyyah in
bidāyat al-mujtahid. Meine aktuelle Sicht: [[ramadan-niyyah-personal|supersedes]]
meine frühere Notiz, die die Mālikī-Position mit der Shāfiʿī-Position vermengte.
```

Sechs der sieben neuen Felder ausgefüllt; `warrant_notes` weggelassen (noch keine Überlieferungsdetails festzuhalten); `supersedes` und `derives-from` als typisierte Links im Text, nicht als YAML-Skalare.

---

*MIG-022 §A — Schemaerweiterungen landen in diesem Constellation-Build. Der Warrant Research Workstream (separates Concept Paper, mehrmonatig) liefert den Berechtigungsklassifikator aus, der das `warrant`-Feld konsumiert. Die zeitliche Achse (MIG-023, separater Architect-Zyklus) konsumiert `updated_at` plus die breitere Notiz-Zustandshistorie.*

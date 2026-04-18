# Arabische Engine

Constellation analysiert arabische Texte mit einer fuenfschichtigen morphologischen Engine, die von Grund auf fuer diese Anwendung entwickelt wurde. Es handelt sich nicht um die Portierung eines vorhandenen Stemmers — es ist ein natives Instrument, das arabische Wurzeln, Muster, Eigennamen, Lehnwoerter und Ihre eigene Terminologie versteht. Die Engine selbst konfigurieren Sie nie; sie laeuft im Hintergrund unter jeder Suche, jedem Link, jedem Indexeintrag. Was Sie *konfigurieren koennen* — und was dieses Hilfethema behandelt — ist der eine Ort, an dem die Engine Ihr Urteil einlaedt: das Panel **Ueberschreibungen der Arabisch-Engine** in den Einstellungen.

---

## Warum die Engine existiert

Arabisch ist eine Schablonensprache. Eine einzige Wurzel wie ك‑ت‑ب ("schreiben") erzeugt Dutzende von Oberflaechenformen — كاتب (Schreiber), مكتوب (geschrieben), كتاب (Buch), يكتب (er schreibt), كتبنا (wir schrieben) — die bei einer Suche alle auf denselben semantischen Kern kollabieren sollten. Ein naiver Stemmer verstuemmelt diese Formen entweder (er reduziert وائل beispielsweise uebereifrig zu ائل) oder verfehlt die Verbindung zwischen ihnen ganz. Die Engine von Constellation vermeidet beide Fehler, indem sie jedes arabische Wort in strikter Prioritaetsreihenfolge durch fuenf Schichten schickt:

1. **Schicht 0 — Benutzer-Ueberschreibungen** (diese steuern Sie)
2. **Schicht 2 — Geschuetzte Liste** (ca. 1.200 handkuratierte Eigennamen, Ortsnamen, Lehnwoerter und Funktionswoerter, die niemals beruehrt werden duerfen)
3. **Schicht 3 — Generativer FST** (ein kompilierter endlicher Transduktor, der ca. 7.000 Wurzeln x 158 Muster auf ihr vollstaendiges Oberflaechenvokabular abbildet)
4. **Schicht 3b — Kaskade** (phonologische Reparaturen: Assimilation, schwache Wurzeln, Hamza-Platzierung)
5. **Schicht 5 — Heuristik** (der schonende Fallback — ein konservativer Affix-Stripper, der nur dann greift, wenn alle anderen Schichten abgelehnt haben)

Ein Ranking-Schritt (Schicht 4) waehlt die einzelne beste Analyse, wenn mehrere Schichten eine Lesart liefern. Das Ranking stellt Ihre Ueberschreibungen ueber alles andere.

---

## Funktion: Ueberschreibungen der Arabisch-Engine

### Was es ist

Das Ueberschreibungs-Panel ist eine kleine Tabelle in den Einstellungen, in der Sie der Engine in Ihren eigenen Worten mitteilen, wie bestimmte arabische Oberflaechenformen zu analysieren sind. Jede Ueberschreibung hat:

- **Oberflaechenform** — das arabische Wort genau so, wie Sie es schreiben (z.B. وائل).
- **Lemma** — die kanonische Form, die die Engine zurueckgeben soll (z.B. وائل).
- **Wurzel** — optional. Drei oder vier Konsonanten, wenn das Wort eine klassische Wurzel hat.
- **Muster** — optional. Eine frei formulierte Bezeichnung (z.B. `فاعل`), wenn Sie die morphologische Schablone festhalten moechten.
- **Wortart** — Eigenname / Substantiv / Adjektiv / Adverb / Verb / Partikel / Fremd / Unbekannt.
- **Notiz** — optional. Eine Zeile Kontext fuer Ihr zukuenftiges Ich.

### Warum es wichtig ist

Jedes Wissensnetz enthaelt Begriffe, die die Engine nicht aus einem Woerterbuch kennen kann: Ihre eigenen Wortschoepfungen, Namen aus Ihrer Heimatstadt, Abkuerzungen aus Ihrem Fachgebiet, Lehnwoerter, die Ihre Kollegen in einer bestimmten Schreibung bevorzugen. Ohne Ueberschreibungen wuerde die Engine ihre generische Analyse auf diese Oberflaechenformen anwenden, und Ihre Suchergebnisse zerfielen an leichten Variationen. Eine Ueberschreibung ist die souveraene Antwort — sie schlaegt den generativen FST, die Kaskade und den heuristischen Fallback. Das Ranking von Schicht 4 gibt Ueberschreibungen den hoechsten Ursprungswert und eine Konfidenz von 1,0, sodass sie niemals zugunsten einer anderen Analyse verworfen werden.

Ueberschreibungen liegen in einer einzigen JSON-Datei unter `<Ihr Universum>/.constellation/arabic-overrides.json`. Die Datei ist reiner Text, alphabetisch sortiert und wird atomar geschrieben (ueber ein `.tmp`+Umbenennen-Paar), sodass ein Stromausfall waehrend einer Aenderung sie nicht beschaedigen kann. Sie gehoert Ihnen — Sie koennen sie in eine Versionskontrolle einchecken, diffen oder geraeteuebergreifend teilen.

### So verwenden Sie es

**Schritt 1: Das Panel oeffnen**

Klicken Sie auf das Zahnradsymbol oben rechts in der Symbolleiste (oder druecken Sie `Ctrl + ,` / `Cmd + ,`), um die Einstellungen zu oeffnen. Waehlen Sie in der linken Seitenleiste **Arabische Ueberschreibungen** — sie steht neben **Sprache**. Wenn Sie sie nicht sehen, scrollen Sie in der Seitenleiste.

**Schritt 2: Die erste Ueberschreibung hinzufuegen**

Klicken Sie auf **Ueberschreibung hinzufuegen**. Ein Formular mit sechs Feldern erscheint (Oberflaechenform, Lemma, Wurzel, Muster, Wortart, Notiz). Geben Sie die Oberflaechenform genau so ein, wie Sie sie in Ihren Notizen schreiben — die Engine normalisiert Diakritika und Alef-Varianten intern, Sie muessen sie also nicht exakt treffen. Tragen Sie das Lemma ein, das zurueckgegeben werden soll. Lassen Sie Wurzel und Muster leer, wenn Sie sie nicht kennen; die Engine verwendet die Ueberschreibung trotzdem. Waehlen Sie eine Wortart aus dem Dropdown aus oder belassen Sie es bei **Unbekannt**. Klicken Sie auf **Speichern**.

**Schritt 3: Den Neuindizierungs-Banner beobachten**

In dem Moment, in dem Sie speichern, zeigt das Panel **Neuindiziert…** und die Engine durchsucht jede Notiz im aktiven Universum, deren Text diese Oberflaechenform enthaelt. Jede uebereinstimmende Notiz wird unter dem neuen Ueberschreibungs-Urteil neu tokenisiert. Wenn der Durchlauf abgeschlossen ist — meist innerhalb einer Sekunde in einem typischen Universum — wechselt der Banner zu **N Notiz(en) neu indiziert** und wird nach drei Sekunden automatisch ausgeblendet. Sie muessen die Anwendung nicht neu starten und auch keinen Index neu aufbauen.

**Schritt 4: In der Suche ueberpruefen**

Oeffnen Sie den Such-Hub (`Ctrl + K` / `Cmd + K`) und tippen Sie die Oberflaechenform ein. Die Treffer sollten jetzt das Lemma widerspiegeln, das Sie angegeben haben: Anfragen nach dem Lemma finden die Oberflaechenform, und Anfragen nach der Oberflaechenform finden andere Flexionen des Lemmas.

**Schritt 5: Eine Ueberschreibung entfernen**

Klicken Sie auf die Schaltflaeche **x** in der Zeile der Ueberschreibung. Der Eintrag wird sofort von der Festplatte entfernt, und derselbe Neuindizierungs-Durchlauf laeuft rueckwaerts — die Notizen, die die Oberflaechenform enthielten, werden unter der generischen Analyse der Engine neu tokenisiert. Der Banner meldet, wie viele Notizen betroffen waren.

### Zusammenspiel mit der geschuetzten Liste

Die geschuetzte Liste (Schicht 2) enthaelt bereits ca. 1.200 haeufige Oberflaechenformen, die niemals gestripped werden duerfen — Namen wie وائل, Orte wie فلسطين, Lehnwoerter wie إنترنت. Sie muessen diese nicht selbst hinzufuegen; die Engine liefert sie mit. Verwenden Sie das Ueberschreibungs-Panel fuer Oberflaechenformen, die *persoenlich* zu Ihrem Universum gehoeren — Ihre eigene Terminologie, lokale Namen, fachspezifische Lehnwoerter oder Faelle, in denen Sie der automatischen Lesart der Engine widersprechen.

### Zusammenspiel zwischen Universen

Jedes Universum hat seine eigene Ueberschreibungsdatei. Das Wechseln des Universums tauscht die aktive Ueberschreibungsmenge im Speicher aus — die Engine laedt das JSON aus dem `.constellation/`-Ordner des neuen Universums neu. Fehlt die Datei (ein frisches Universum), behandelt die Engine die Ueberschreibungsmenge als leer. Ist die Datei fehlerhaft, protokolliert die Engine eine Warnung und faellt auf eine leere Menge zurueck, anstatt das Laden zu verweigern.

### Was passiert, wenn Sie die Datei von Hand bearbeiten

Sie koennen das tun. Das Dateiformat lautet:

```json
[
  {
    "surface": "وائل",
    "lemma": "وائل",
    "root": null,
    "pattern": null,
    "pos": "ProperNoun",
    "note": "Personenname — niemals strippen"
  }
]
```

Halten Sie die Eintraege alphabetisch nach Oberflaechenform sortiert, damit git-freundliche Diffs entstehen. Die Engine sortiert bei jedem Speichern neu, sodass manuelle Umsortierungen einer UI-Aenderung nicht ueberleben.

---

## Glossar

- **Oberflaechenform** — ein arabisches Wort so, wie es geschrieben wird, einschliesslich angehaengter Klitika (z.B. الكتاب, بالكتاب, كتبنا).
- **Lemma** — die Zitierform eines Wortes, von Flexion befreit (z.B. كتاب).
- **Wurzel** — der 3- oder 4-konsonantische semantische Kern, den eine Wortfamilie teilt (z.B. ك‑ت‑ب).
- **Muster** — die Vokal- und Affixschablone, die mit einer Wurzel kombiniert eine Oberflaechenform ergibt (z.B. فاعل → كاتب).
- **FST** — ein endlicher Transduktor. Die Engine verwendet einen, um Wurzeln x Muster effizient auf ihr vollstaendiges Oberflaechenvokabular abzubilden.
- **Kaskade** — die phonologische Reparaturschicht, die Assimilation, schwache Konsonanten und Hamza-Platzierung behandelt.
- **Ueberschreibung** — Ihr eigenes Urteil darueber, wie eine bestimmte Oberflaechenform analysiert werden soll; schlaegt jede andere Schicht.

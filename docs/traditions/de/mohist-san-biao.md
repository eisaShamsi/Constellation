---
id: mohist-san-biao
name: Mohist sān biǎo
family: chinese-pragmatist
shape: horizontal-bands
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# Mohist sān biǎo (三表)

**Familie**: chinesisch-pragmatistisch · **Form**: horizontale Bänder (3 Zonen)

## Leitmetapher

Die Kuppel teilt sich in **drei horizontale Zonen, von oben nach unten
gestapelt**, je eine pro mohistischem Standard zur Bewertung einer Lehre:

- **本 běn (Wurzel)** — oben. Historischer Präzedenzfall der
  Weisen-Könige: Hat die Lehre Gewähr in der ererbten Tradition?
- **原 yuán (Ursprung)** — Mitte. Direkte beobachtende Evidenz: Sehen
  und hören gewöhnliche Menschen, dass es so ist?
- **用 yòng (Anwendung)** — unten. Praktischer sozialer Nutzen:
  Verbessert die Übernahme dieser Lehre das Leben der Menschen?

Eine Lehre ist nur dann zu halten, wenn sie alle drei Tests besteht —
aber die Sight-Darstellung lässt Sie Notizen über die drei verteilt sehen,
um ein Gefühl dafür zu bekommen, welche Begründungsart in Ihrem Universum
die meiste Arbeit leistet.

Die horizontale Achse trägt keine spezifische Kodierung — die drei
Standards des Mohismus sind *kategorial*, nicht ordinal, sodass die
Positionierung innerhalb eines Bandes durch deterministisches
Per-Notiz-Jitter erfolgt.

## Geltungsbereich

**Wann diese Tradition verwendet werden soll.** Bei der Arbeit mit
Inhalten, bei denen der Test darin besteht, *ob eine Lehre wert ist,
gehalten zu werden*, nicht welche Art von Gewähr ihr zugrunde liegt.
Nützlich für Politik, Ethik, angewandt-empirische und
praktisch-entscheidungsbezogene Inhalte, bei denen historischer
Präzedenzfall / Beobachtung / Nutzen die drei Achsen der Rechtfertigung
sind.

**Wann diese Tradition NICHT verwendet werden soll.** Wenn Inhalte keine
doktrinäre oder bewertende Dimension haben. Reine deskriptive Inhalte,
kreative Arbeit und Notizen über subjektive Erfahrung passen schlecht.

## Anwendbarkeit

- Politikvorschläge und ihre Rechtfertigungen.
- Vergleichend-ethische Analyse (besteht diese Regel die drei Tests?).
- Ingenieurwesen und angewandte Wissenschaft, in denen
  Nutzen-für-die-Menschen explizit ist.

## Lineage

Klassische chinesisch-pragmatistische Erkenntnistheorie. Mòzǐ 墨子
(~5. Jh. v. Chr.) gründete die mohistische Schule, die sich als kritische
Alternative zum Konfuzianismus präsentierte. Die sān biǎo erscheinen im
Kapitel „Anti-Fatalismus" als Test, den die Mohisten auf die ererbte
fatalistische Lehre anwandten — und schlossen, dass sie alle drei Tests
nicht bestand. Die Schule blühte kurz auf und wurde dann vom konfuzianischen
und legalistischen Aufstieg überschattet; sie überlebt als
wiederherstellbarer kanonischer Text, der heute durch Ausgaben wie das
*Mòzǐ jiāngǔ* studiert wird.

## Kritik

Die sān biǎo werden manchmal als frühe Form des Pragmatismus kritisiert,
die evidenzbasierte Gewähr mit Nützlichkeit verschmilzt — insbesondere
das Kriterium „Nutzen für die Menschen" ist schwer zu formalisieren.
Moderne Gelehrte debattieren auch, ob sān biǎo eine voll entwickelte
epistemische Theorie ist oder ein polemisch-rhetorisches Werkzeug, das in
einem spezifischen anti-fatalistischen Argument eingesetzt wurde. Wurde
in die kuratierte Grundlinie trotz seines Himmel-theologischen Kontextes
unter der Religions-Lineage-Regel übernommen, weil der methodologische
Kern säkular ist.

## Zitation

**Primär.** *Mòzǐ* 墨子, Book IX, "Fēi Mìng Shàng" 非命上
("Anti-Fatalism, Part I"). Critical edition: Sūn Yíràng, ed., *Mòzǐ
jiāngǔ* 墨子閒詁, 2 vols. (Beijing: Zhonghua Shuju, 1986). English:
Ian Johnston, trans., *The Mozi: A Complete Translation* (New York:
Columbia University Press, 2010).

**Modern.** A. C. Graham, *Disputers of the Tao: Philosophical
Argument in Ancient China* (La Salle, IL: Open Court, 1989), ch. 1;
Chris Fraser, "Mohism," *Stanford Encyclopedia of Philosophy* (2020).

## Per-Notiz-Frontmatter

`mohist_zone: ben | yuan | yong`. Derzeit nicht vorhanden — Notizen
werden durch Hash-Bucketing deterministisch nach notePath in die drei
Zonen platziert, sodass die visuelle Struktur befüllt ist. Wenn die
Rust-seitige `LayoutCacheRow`-Erweiterung landet, überschreibt dieses
Feld die Hash-Bucket-Zuordnung.

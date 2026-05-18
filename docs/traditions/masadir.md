---
id: masadir
name: masādir
family: sunni-islamic-usul
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
---

# masādir

**Family**: Sunni Islamic uṣūl · **Shape**: sectoral (4 quadrants + 4 extension chips)

## Hero metaphor

The dome divides into **four sources of authoritative proof** in Sunni
*uṣūl al-fiqh*: Qur'an, sunnah, ijmāʿ (scholarly consensus), and qiyās
(analogical reasoning). Each is a different *kind* of proof — not a
different degree of one proof — and so the layout is sectoral (categorical
slices), not concentric (graded depth). Below the dome, four supplementary
sources sit as chips: *istiḥsān* (juristic preference), *istiṣḥāb*
(presumption of continuity), *maṣlaḥa mursalah* (unrestricted public
interest), and *ʿurf* (customary practice).

Like pramāṇa, the quadrants were rotated +π/4 (§θ-fix-1, 2026-05-18) to
clear the vertical axis from stratum labels — so the geometric positions
are now E/S/W/N rather than the originally documented NE/SE/SW/NW.

## Scope

**When to use this tradition.** When working with content that is or
could be analyzed as Sunni Islamic legal-scholarly reasoning. Useful for
seeing the balance of proof-types across a derivation: is your argument
heavily Qur'an-grounded? Does it lean on consensus? Does qiyās do most
of the work? The four extension chips are visual reminders that
classical uṣūl recognizes more than the four headline sources.

**When NOT to use this tradition.** For non-Islamic content the
quadrant labels make no sense. The framework is also specifically
Sunni — Twelver Shīʿī uṣūl replaces qiyās with ʿaql (reason) and is
deliberately not included per the religious-lineage rule (orientation
v2.09). Mystical, philosophical, and literary content fits poorly.

## Applicability

- Sunni fiqh derivation, *uṣūl al-fiqh* coursework, fatwa analysis.
- Cross-source balance audit in legal-scholarly writing.
- Teaching the kinds-of-proof structure of classical Islamic
  jurisprudence.

## Lineage

Classical Sunni uṣūl al-fiqh — the science of the sources and methods
of Islamic legal reasoning. The four-source canon is conventional
across the four Sunni madhāhib (Hanafi, Maliki, Shafiʿi, Hanbali),
with internal variation in how each source is weighted. The
Constellation rendering follows the al-Ghazālī *Mustaṣfā* line.

## Critique

The placement of ijmāʿ in the *ijtihādī* (reasoning-derived) cluster
rather than the *naṣṣ* (textually-transmitted) cluster is contested by
Ashʿarī/Māturīdī kalām, which treats ijmāʿ as binding-transmitted.
Constellation ships the Mustaṣfā-aligned reading; the alternative kalām
reading is a v4.1 polish target. The four-source canon also flattens
the doctrinal differences across the four madhāhib — a Hanafi-specific
or Maliki-specific variant register could be added later.

The exclusion of Shīʿī uṣūl is a product-design choice (orientation
v2.09's religious-lineage rule), not a scholarly judgment.

## Citation

**Primary.** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*,
ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

**Modern.** Franz Rosenthal, *Knowledge Triumphant: The Concept of
Knowledge in Medieval Islam* (Leiden: Brill, 1970); Wael B. Hallaq,
*A History of Islamic Legal Theories* (Cambridge: Cambridge University
Press, 1997).

## Per-note frontmatter

`masadir_source: quran | sunnah | ijma | qiyas`. When the Rust-side
`LayoutCacheRow` extension lands, this field overrides the default
placement (currently all notes → Qur'an). Per-note opt-in via
`istihsan | istishab | maslaha | urf` for the extension-chip sources is
a follow-up.

---
id: pramana
name: pramāṇa
family: indian-nyaya
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
---

# pramāṇa

**Family**: Indian Nyāya · **Shape**: sectoral (4 quadrants)

## Hero metaphor

The dome divides into **four quadrants of valid knowing**, each housing
notes whose epistemic ground is of one kind. Knowledge is sorted not by
*how mature* it is (Aristotelian) but by *how it came to be known*:
through direct perception, through inference from evidence, through
analogy from a known case, or through trusted testimony. The pramāṇas
are **kinds, not levels** — moving a note from one quadrant to another
is a change of warrant, not a change of confidence.

Within each quadrant, the radial stratum encoding from Aristotelian is
preserved, so a note's depth-of-understanding stays legible inside its
warrant-kind. After §δ.2-fix-1 (2026-05-17) the quadrants sit at
E/S/W/N rather than the original NE/SE/SW/NW, to clear the vertical
axis from stratum-label collision.

## Scope

**When to use this tradition.** When you want to see at a glance how
your knowledge is *grounded* — what proportion of your work rests on
direct observation vs. inferred conclusion vs. comparison vs. authority.
Useful for epistemic self-audit: are you over-reliant on testimony? Are
your inferences carrying weight they don't deserve?

**When NOT to use this tradition.** When the warrant doesn't vary
across your notes — e.g., a universe entirely about lived experience
(all pratyakṣa) or entirely citation-driven (all śabda) won't surface
useful structure under this lens. Also a poor fit for content that
doesn't admit a clean source-of-knowing classification (creative work,
speculation, fiction).

## Applicability

- Self-audit of epistemic balance across a research project.
- Distinguishing primary from secondary sources at a glance.
- Teaching the cognitive-act analysis of knowledge.

## Lineage

Classical Indian Nyāya — the school of formal Indian epistemology that
analyzed cognition by enumerating the valid means by which it arises.
The four-pramāṇa Nyāya canon is the version Constellation ships
(other Indian schools count differently — Sāṃkhya recognizes three,
Mīmāṃsā six). Sūtra-era India through the medieval commentaries; today
a live tradition through the work of B. K. Matilal, J. N. Mohanty, and
others.

## Critique

Choosing the four-pramāṇa Nyāya variant is itself a scholarly stake —
the Mīmāṃsā six-pramāṇa view (adding *arthāpatti* postulation and
*anupalabdhi* non-apprehension) was explicitly excluded by the
religious-lineage rule (orientation v2.09) because it is Vedic-authority
based; the Buddhist Pramāṇavāda traditions (Dignāga, Dharmakīrti) were
likewise excluded. Users from other Indian-philosophical lineages may
find the Constellation rendering reductive.

## Citation

**Primary.** *Nyāya-Sūtra* 1.1.3 (the enumeration of the four
pramāṇas). Available in Gautama, *The Nyāya Sūtras of Gautama*, trans.
Satisa Chandra Vidyābhūṣana, rev. ed. Nandalal Sinha (Delhi: Motilal
Banarsidass, 1990).

**Modern.** J. N. Mohanty, *Classical Indian Philosophy* (Lanham:
Rowman & Littlefield, 2000), 17–34; Bimal Krishna Matilal,
*Perception: An Essay on Classical Indian Theories of Knowledge*
(Oxford: Clarendon Press, 1986), ch. 1.

## Per-note frontmatter

`pramana_kind: pratyaksha | anumana | upamana | shabda`. When the
Rust-side `LayoutCacheRow` extension lands, this field overrides the
default placement (currently all notes → `pratyaksha`). The
philosophical default is defensible: all knowledge begins as perception
until reflectively reclassified.

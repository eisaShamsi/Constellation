---
id: mohist-san-biao
name: Mohist sān biǎo
family: chinese-pragmatist
shape: horizontal-bands
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
---

# Mohist sān biǎo (三表)

**Family**: Chinese pragmatist · **Shape**: horizontal-bands (3 zones)

## Hero metaphor

The dome divides into **three horizontal zones stacked top to bottom**,
one per Mohist standard for evaluating a doctrine:

- **本 běn (root)** — top. Historical precedent of the sage-kings:
  does the doctrine have warrant in the inherited tradition?
- **原 yuán (origin)** — middle. Direct observational evidence: do
  ordinary people see and hear that it is so?
- **用 yòng (use)** — bottom. Practical social benefit: does adopting
  this doctrine improve the lives of the people?

A doctrine is worth holding only if it passes all three tests — but
the Sight rendering lets you see notes distributed across the three to
get a feel for which warrant-type is doing the most work in your
universe.

The horizontal axis carries no specific encoding — Mohist's three
standards are *categorical*, not ordinal, so positioning within a band
is by deterministic per-note jitter.

## Scope

**When to use this tradition.** When working with content where the
test is *whether a doctrine is worth holding*, not what kind of warrant
underlies it. Useful for policy, ethics, applied-empirical, and
practical-decision content where historical precedent / observation /
benefit are the three axes of justification.

**When NOT to use this tradition.** When content has no doctrinal or
evaluative dimension. Pure descriptive content, creative work, and
notes about subjective experience fit poorly.

## Applicability

- Policy proposals and their justifications.
- Comparative-ethics analysis (does this rule pass the three tests?).
- Engineering and applied-science where benefit-to-the-people is
  explicit.

## Lineage

Classical Chinese pragmatist epistemology. Mòzǐ 墨子 (~5th c. BCE)
founded the Mohist school, which presented itself as a critical
alternative to Confucianism. The sān biǎo appear in the "Anti-Fatalism"
chapter as the test the Mohists applied to the inherited fatalist
doctrine — and concluded it failed all three tests. The school
flourished briefly then was overshadowed by Confucian and Legalist
ascendancy; it survives as a recoverable canonical text studied today
through editions like the *Mòzǐ jiāngǔ*.

## Critique

The sān biǎo are sometimes critiqued as an early form of pragmatism
that conflates evidential warrant with utility — the "benefit to the
people" criterion in particular is hard to formalize. Modern scholars
also debate whether sān biǎo is a fully-developed epistemic theory or
a polemical-rhetorical tool deployed in a specific anti-fatalist
argument. Grandfathered into the curated baseline under the
religious-lineage rule despite its Heaven-theology context, because the
methodological core is secular.

## Citation

**Primary.** *Mòzǐ* 墨子, Book IX, "Fēi Mìng Shàng" 非命上
("Anti-Fatalism, Part I"). Critical edition: Sūn Yíràng, ed., *Mòzǐ
jiāngǔ* 墨子閒詁, 2 vols. (Beijing: Zhonghua Shuju, 1986). English:
Ian Johnston, trans., *The Mozi: A Complete Translation* (New York:
Columbia University Press, 2010).

**Modern.** A. C. Graham, *Disputers of the Tao: Philosophical
Argument in Ancient China* (La Salle, IL: Open Court, 1989), ch. 1;
Chris Fraser, "Mohism," *Stanford Encyclopedia of Philosophy* (2020).

## Per-note frontmatter

`mohist_zone: ben | yuan | yong`. Currently absent — notes are
hash-bucketed into the three zones deterministically by notePath so the
visual structure is populated. When the Rust-side `LayoutCacheRow`
extension lands, this field overrides the hash-bucket assignment.

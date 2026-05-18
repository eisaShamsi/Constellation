---
id: peirce
name: Peirce
family: modern-western
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
---

# Peirce

**Family**: Modern Western · **Shape**: sectoral (3 wedges)

## Hero metaphor

The dome divides into **three phenomenological categories** that Peirce
argued underlie all experience and all reasoning:

- **Firstness** — quality, feeling, possibility. The "what it is to be
  red" *before* anything red exists.
- **Secondness** — action, reaction, brute fact. The actual collision,
  the resistance of the world.
- **Thirdness** — mediation, law, habit, sign. The pattern that connects
  Firstness and Secondness; the regularity that explains why this
  happens after that.

The three sectors sit at 120° each, rotated +π/6 from the cardinal axes
(§δ.1-fix-1) so no divider runs through the stratum labels at the top
of the dome.

## Scope

**When to use this tradition.** When the question is *what category of
experience* this content describes. Excellent for semiotic work, for
distinguishing felt quality from acted fact from explanatory law, and
for analyzing argument types (abductive Firstness, deductive Secondness,
inductive Thirdness in one Peircean reading).

**When NOT to use this tradition.** When the content has no
phenomenological cut — when it is all data, or all action, or all
law, the category vocabulary provides no useful sorting. Also a
demanding tradition: the categories take effort to apply correctly,
and naïve readings tend to collapse Thirdness into Secondness.

## Applicability

- Semiotics, sign theory, communication studies.
- Phenomenology of experience.
- Argument-type analysis (abduction / deduction / induction).

## Lineage

American pragmatism. Charles Sanders Peirce (1839–1914), founder of
pragmaticism and modern formal logic, articulated the three categories
across his entire career; they appear early ("On a New List of
Categories," 1867) and become more refined in his semiotic and
phenomenological work. The categories are *universal*: Peirce argued
they constitute the irreducible vocabulary of all phenomena, not just
one domain.

## Critique

The categories are notoriously hard to apply without training, and
Peirce himself revised his presentations many times. Critics from the
analytic side question whether the three-way partition is exhaustive;
critics from the phenomenological side argue Peirce's categories are
too formal to capture lived experience. The current Constellation
rendering defaults all notes to Firstness because per-note frontmatter
extraction has not yet shipped — once it does, users can opt notes
into the category they intend.

## Citation

**Primary.** Charles S. Peirce, "On a New List of Categories" (1867),
in *Writings of Charles S. Peirce*, vol. 2, ed. Edward C. Moore et al.
(Bloomington: Indiana University Press, 1984).

**Modern.** T. L. Short, *Peirce's Theory of Signs* (Cambridge:
Cambridge University Press, 2007); Robert Lane, *Peirce on Realism and
Idealism* (Cambridge: Cambridge University Press, 2018).

## Per-note frontmatter

`peirce_category: firstness | secondness | thirdness`. Currently absent
on the Rust side — all notes default to Firstness as a visually
populated baseline.

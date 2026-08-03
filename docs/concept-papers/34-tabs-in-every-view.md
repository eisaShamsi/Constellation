# MIG-110 — Tabs in Every View

**Status:** ALLOCATED, not scheduled · **Concept paper** · Boss-directed 2026-08-02
**Tracks:** PJ-208 (the concept + Architect), PJ-209 (the always-visible ×, SHIPPED), PJ-210 (split-view collapse rule, SHIPPED)

---

## 1. The horse — the concept, before any function

> **A note that is open must always be reachable, closable, and visible as one of the things
> you currently have open — no matter which view it is being displayed in.**

That is the whole concept. It is not "add a tab bar to split view." It is that **the set of
open notes is a property of the session, not of a layout**, and every view is obliged to
express it.

Today the opposite is true: the list of open notes is a property of *one* layout. Turn split
view on and the tab bar disappears (`+layout.svelte`, `{#if !$splitActive}`), taking with it
the only place a note could be switched to, closed, middle-clicked, or even *counted*. The
notes are still open. The app simply stops saying so.

**Why this is a formulation concern and not a cosmetic one.** Constellation's claim is that
knowledge is made by holding things next to each other — Observation → Connection → Tension →
Synthesis. The set of notes a user has deliberately kept open *is* their current working set:
the raw material of the connection they are trying to make. A view that hides that set hides
the user's own thinking from them at the moment they are doing it.

## 2. How the gap was found

Boss test, 2026-08-02. Testing an unrelated fix, the instruction was "close every open tab in
split view." It could not be done — **there was no way to close a note in split view at all**:
no tab bar (hence no `×`, no right-click Close/Close-others), nothing in the note's ⋯ menu, and
nothing in the file-tree right-click. A pane with no exit.

The Boss's ruling, verbatim:

> *"Every note should have the close icon (x) visible in any situation or condition, beside it
> existing in the 'More options' and the RC from the file tree."*
>
> *"When closing notes in split view, and there is only one note remaining, the logic is to have
> it go back to the normal view, where its tab is showing."*
>
> *"I want to explore the possibilities of having the note tabs available in all views and
> conditions. It could be at the top, the usual way, or on the side; it all depends on the way
> it is displayed/viewed."*

## 3. Already shipped under this concept (2026-08-02)

These closed the dead end. They are **not** the migration — they are the floor it builds on.

| | What | Where |
|---|---|---|
| **PJ-209** | An always-visible `×` on the note itself | `NotePane.svelte` — `.e-bc-close`, beside the ⋮ in the breadcrumb header, present in every view including split |
| — | **Close** in the note's ⋯ menu | `NotePane.svelte` → `NoteEditor.handleMoreAction`, in the *always* group so it works on the second screen too |
| — | **Close** in the file-tree right-click | `contextMenuBuilder.ts` (shared, so tree + OrgChart + second screen inherit it); wired only when the note is actually open |
| **PJ-210** | Split view collapses to normal view when fewer than two notes remain | `store.ts` — `collapseSplitIfBelowTwo`, shared by `closeTab` **and** the Delete/Overwrite vacate path |

## 4. What MIG-110 has to decide — the open questions

The shipped work makes every note closable. It does **not** make the open set *visible* in
every view. That is the migration, and it is a design question before it is a code question.

**Q1 — Where does the tab strip live when the layout is not a single column?**
The Boss named two candidates explicitly: *top, the usual way*, or *on the side*, "depending on
the way it is displayed/viewed." Per-pane strips (Obsidian's answer) is a third. Each implies a
different answer to Q2.

**Q2 — Is the open set global, or per pane?**
Today it is global (`openTabs`) with two pointers into it (`activeTabId`, `focusedTabId`).
Per-pane strips would require the open set to become per-pane, which is a **data-model change**,
not a layout change — and it touches session restore (`session.json`), the second screen, and
every consumer of `openTabs`. This is the question that decides whether MIG-110 is small or
large. **Answer it first.**

**Q3 — What does the second screen show?**
It is display-only by contract (PJ-130/PJ-108). Does it get a tab strip, and if so is it the
main window's set or its own?

**Q4 — Does the collapse rule survive tabs-in-split?**
PJ-210 collapses to normal view below two notes *because* split without a tab bar strands the
survivor. If split view gains its own tab strip, that reason evaporates and the rule may need
revisiting — it would then be a preference, not a repair.

**Q5 — RTL and the side placement.** A side strip has a reading-edge; Language-First by Design
means it flips. Settle this with the placement, not after.

## 5. Constraints this must not break

- **Form-Aligns-To-Purpose** — a tab strip in a view whose purpose does not include switching
  is noise. If a view genuinely has no switching need, the answer may be "no strip here", and
  that must remain a permissible outcome of the exploration.
- **Constraint as Design** — "tabs everywhere" must not become chrome everywhere.
- **Rule 1 / Rule 5** — a strip that re-renders on every keystroke or forces layout across the
  tree is disqualified regardless of how it looks. Virtualize beyond 50 (Rule 3).
- **The Boss's Art Director ruling (2026-07-10)** — the visual design and its coding go through
  the Art Director & Team multi-agent workflow, not solo hand-iteration.

## 6. Method

`/migration` — this crosses layout, the tab data model, session restore, and the second screen.
**Phase 1 Architect must answer Q2 before anything else**, because global-vs-per-pane decides
the entire shape. Concept-Before-Function: no code until the placement rule can be stated in one
sentence that survives Q1 through Q5.

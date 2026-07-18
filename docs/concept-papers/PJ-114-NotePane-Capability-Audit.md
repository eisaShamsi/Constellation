# NotePane Capability Audit — Current State & Enrichment Assessment

*PJ-114 (Boss-directed step) · 2026-07-18 · read-only audit `wf_baf417ca-22a` · the FM+ cross-check is a LATER step, not decided here*

**Purpose (Boss):** "Audit what we have in NotePane today to see if we need to enrich it. Then we'll cross-check it with FM+ to choose what's suitable." NotePane is the *full* editor, so it should own the *full* living-link treatment. This maps what it owns today and where it falls short.

---

## 1. What NotePane can do today

**Editor / text (a competent Markdown formatter):** bold/italic/underline/strike/highlight/code/super/subscript/clear · H1–H6, lists, task lists, quote, HR, callout, code/math block, image, table (grid picker + full TableToolbar) · undo/redo, find, font + text-color pickers · language-aware script-symbol panels (Arabic/Quran, Hebrew, CJK, Cyrillic, Devanagari) · clipboard incl. paste-as-plain-text.

**Link actions (Markdown-token level only):** author `[[ ]]` (Ctrl+K) · follow/open (Ctrl-click) · "Edit link" = just text-selects the syntax · "Remove link" = strips syntax · **assign a cognitive TYPE only while first typing** (the `[[…` autocomplete offers the 8 kinds → `[[type::target]]`).

**Note-level:** breadcrumb history + **stage badge (Promote/Demote)** + trail prev/next · inline title/rename · **⋯ menu** (source/live-preview toggle, Focus, Add property, Rename, Reveal in tree, Show in explorer, Open default app, Copy path/name, Delete) · **PropertyEditor** (properties, stage combobox, tags/aliases, structural links, taxonomy pickers).

**Panels (the real living-link surface):** Backlinks/Outgoing (type pill, `×N` traversal chip, read-only annotation, "Link it", right-click → Confidence / Archive) · Structure (ancestor/descendant outline) · Suggested connections (one-click typed link via LinkTypePicker) · Review, Provenance, 360° Inspector, Health/Tension, Local Sky. **Caveat:** in ordinary editing these are one-at-a-time sidebar tabs — a note's body, incoming, and outgoing links are never co-visible.

---

## 2. The living-link scorecard — can you VIEW / SET each of the 8 properties in NotePane today?

| Property | View | Set | Note |
|---|:--:|:--:|---|
| **Type** (8 kinds) | ✅ | ⚠️ | Seen as a pill in panels; settable **only via autocomplete while first typing**. No way to re-type an existing link except editing raw `[[…]]` — even though a real LinkTypePicker exists elsewhere. |
| **Annotation** (the reasoning) | ⚠️ | ⚠️ | Read-only/truncated in panels; settable **only by hand-typing** `[[type::Target\|reasoning]]`. **No `setLinkAnnotation` IPC exists at all.** |
| **Confidence** | ⚠️ | ✅ | The one real setter (right-click a panel row). But level shows only inside the popover — **no badge on the row**. |
| **Weight** (earned) | ⚠️ | ❌ auto | Never shown as a number; implied by sort + tier color. |
| **Direction** | ⚠️ | ❌ | Implicit (which panel); no flip/declare control. |
| **Created** | ❌ | ❌ auto | Not surfaced anywhere. |
| **Last Traversed** | ⚠️ | ❌ auto | Tooltip only, hidden when count 0. |
| **Traversal Count** | ✅ | ❌ auto | The `×N` chip (when > 0). |

**Verdict:** of 8 properties, only **1 (Confidence)** has a real setter on the note surface; **2 (Type, Annotation)** are authorable only by raw syntax; **5** are auto. **NotePane is a Markdown-link editor, not yet a living-link editor.**

---

## 3. Enrichment gaps (ranked; almost all are WIRING, not new engines)

1. **Can't re-classify an existing link's TYPE from the editor.** Re-settable in Reviewer/Inspector360/GraphMind — but not in the editor, its menu, or the panel rows. **Engine exists:** `LinkTypePicker.svelte`.
2. **Living-link controls aren't on the link where you write.** Right-click a `[[…]]` → open/copy/edit/remove, none of the 8 props. **Engine exists:** ConfidencePicker + LinkTypePicker are already popovers.
3. **Annotation is write-once-by-syntax.** No edit path, **no `setLinkAnnotation` IPC**. For "links are living vessels carrying annotation," a structural hole. **[new engine — small]:** 1 IPC + a field.
4. **Confidence has no at-a-glance cue.** Settable but invisible on the row. **Engine exists:** display wiring only.
5. **No single per-link inspector** shows all 8 properties in one place. **Engine exists:** `getLinkStage`/`linkLifecycle`/`effectiveLinkWeight` already compute the values.
6. **The temporal life of a link is nearly invisible** (Created absent; Last-Traversed hover-only; Weight/stage computed but never shown). **Engine exists:** `getLinkStage`/`linkLifecycle`.
7. **Frontmatter typed links aren't shown as living links** (a `supports: [[X]]` row is a plain value — no pill/confidence/annotation). **Engine exists:** the same components.
8. **The ⋯ menu carries no knowledge verbs** — entirely view-mode + file-management.

*Housekeeping (separate): 5 dead menu items (Copy target, External link, Footnote, Math, Select all) + 3 dead toolbar align buttons — wire or remove; they currently mislead.*

---

## 4. Recommendation

**Yes — NotePane needs enrichment before the FM+ conversation is worth having.** It's an excellent *Markdown* editor with a rich *diagnostic* panel set, but as the supposed home of the living link it exposes only **1 of 8** properties as directly settable and leaves the note's core cognitive act — typing and reasoning about a link — reachable only through raw syntax or non-co-visible side panels. The good news: almost every gap is **wiring, not a new engine** (the type picker, confidence picker, lifecycle/weight calculators, archive path all already exist).

**Highest-value enrichments, in order:** (1) **wire `LinkTypePicker` into the editor menu + the Backlinks/Outgoing rows** so an existing link's kind is re-settable without editing text; (2) **add link-annotation editing** (1 IPC + a field) so the promised "reasoning" can be authored/revised; (3) **surface confidence as a visible badge** (+ cheaply weight-tier/last-traversed) so a link's state is legible; (4) a **single per-link inspector** gathering all 8 properties. A NotePane that truly owns the living link is the prerequisite for deciding, in the cross-check, what FM+ should complement rather than duplicate.

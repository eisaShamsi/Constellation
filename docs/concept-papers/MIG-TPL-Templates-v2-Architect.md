# Templates v2 — /migration Phase 1 (Architect)

*2026-07-19 · Boss directive: "My Templates engine is not working. We need to fix it now. Check
what we have today, then do the necessary research to create a state-of-the-art template engine."*
*Sources: audit + research workflow `wf_b414642f-909` (full structured output in the session task
log); main-session verification of the load-bearing facts.*

---

## The concept (the horse) — developed first, per the Boss's correction of 2026-07-19

**The question a template answers:** *"What shape does this kind of thinking take?"* — asked
once, answered permanently.

In a knowledge **formulation** system, thought has recurring shapes. An observation log, a
contradiction workup, a synthesis note, a daily review, a book note — each is a *kind* of
cognitive move, and each kind has a form: the frontmatter it needs to be born into the hierarchy,
the questions it must answer before it counts as thought, and the exact spot where thinking
begins. **A template is the captured shape of a recurring cognitive move.** Its purpose is to
spend the user's effort on formulation and none of it on scaffolding: pick the move, answer its
questions in your own language, land with the cursor where the first sentence of real thinking
goes.

**The load-bearing distinction: a template is a MOLD; a note is a CAST.** A note is a thought; a
template is the shape of a *class* of thoughts. Every design decision in this migration is this
one distinction applied:

- **The mold is owned like a file, but is not knowledge.** It is a visible, ordinary `.md` the
  user edits like any note (File-Over-App applies to the mold exactly as to the cast) — yet it
  does not appear among the user's *thoughts*: excluded by default from search, Index, Sky, and
  link surfaces. A mold on the workbench, not a sculpture in the gallery.
- **The mold has NO identity — the Boss's 2026-07-19 ruling, which is the concept speaking.**
  `cid_cn` is a note's identity; `created:` is the timestamp of a thought coming into being. A
  mold that carried them would stamp every future cast with the identity and birthday of the
  mold. Therefore: a template NEVER contains `cid_cn` or creation date/time — not when scaffolded
  new, not when saved from an existing note, not when opened for editing. Identity and birth
  belong to the cast, minted fresh at the moment of casting.
- **The mold is inert.** It shapes; it never acts. No code, no scripts, no side effects — which
  is also the federation-safety line (a cUniverse's templates are someone else's molds).
- **The mold may ask questions.** Prompts and choices are part of a shape ("a contradiction
  workup needs to know: contradicting WHAT?") — declarative form fields, in the user's language,
  where declining to answer means "never mind," not "yes but blank."
- **The cast is born formed.** Expansion happens at the moment of creation, atomically, so a
  note created from a template has never existed in a half-formed state — and is born already
  inside the knowledge hierarchy (frontmatter, and one day living-link properties, pre-seeded)
  instead of being filed into it afterward.

The Five-Acts reading: templates are the launch rail of **Observation** and the standard form of
every repeated act after it.

## Why it is "not working" — the verified diagnosis

The engine is real; the **plumbing is severed**:

1. **The Settings "Template folder" field is a placebo.** `appSettings.templateFolder` (default
   `'Templates'`) is written by Settings and **read by no other line of code in either language**
   (main-session grep, conclusive). Rust `list_templates`/`get_templates_dir`
   (`universe.rs:1778–1821`) unconditionally use the **hidden** `<universe>/.constellation/templates/`.
2. **The Boss's universe has no such hidden directory** → the picker is empty, always — and its
   empty-state message points at the *placebo* folder.
3. **No create-template flow exists.** No "New template", no "Save as template", no
   "Open templates folder" — nothing in `src/` writes a template. The only route is hand-placing
   files in a hidden directory the app never reveals.

## Audit — the full surface map (16 surfaces; status at HEAD)

| Surface | Status | Key finding |
|---|---|---|
| Engine `processTemplateAsync` | partial | The only engine actually called (3 sites). Defects: `formatDate` first-occurrence-only `.replace` corrupts repeated tokens (`YYYY…YYYY` → `2026…26YY`); `$`-substitution hazard on title/folder/library/clipboard; case-inconsistency; unknown `{{var}}` lands verbatim; **prompt answers are re-scanned** (an answer containing `{{prompt:…}}` loops). |
| Engine `processTemplate` (sync) | disconnected | **Dead code** — zero callers; silently lacks half the vocabulary (a trap for a future caller). |
| `extractTemplateBody` | partial | FM close via `indexOf('---', 3)` — not line-anchored; a `---` inside a frontmatter value truncates FM and leaks the rest into the body. |
| Insert-at-cursor | **working** | Fixed this session (PJ-125/PJ-105): path-guarded dispatch into the active editor; guard test-locked. |
| TemplatePicker | working | Sound component; starved by `list_templates`; empty-state names the wrong folder. |
| Prompt/Suggester dialogs | working | i18n'd, RTL-safe. But **cancel → empty string**: the template still applies with blanks; no abort path. |
| Palette entry + Ctrl+T + `/template` | partial | **Not gated** by the `enabledFeatures.templates` toggle (create/daily ARE) — inconsistent. Explicit-Ctrl default = macOS keymap item. |
| Per-template hotkeys | unreachable | `templateHotkeys` read at `+layout:4203` but **no UI writes it anywhere**. |
| New-note-from-template | partial | Folder-template matching is **substring** (`'Work'` matches `'Homework'`); template FM **discarded** while `help/Properties.md:177` promises a merge; whole block in silent `catch`; raw two-write shape. |
| Daily-note flow | partial | Works only from the hidden dir; gated on a fragile `content.length < 50` "freshly created" heuristic; silent catch; cursorOffset ignored. |
| Settings (Universe section) | disconnected | The placebo folder field + dailyNoteTemplate + variable card live in the **Universe** section; the dedicated **Templates** section holds only the toggle. |
| Rust `get_templates_dir` / `list_templates` | working/disconnected | Correct for the hidden dir; never consulted the setting. |
| `folderTemplates` map | partial | Honored by code; **no editing UI** (orphaned i18n keys prove one was intended); help docs describe a system that does not exist. |
| Create-template flow | **absent** | Nowhere in the app. |

## Research digest (4 tracks, sources in the workflow output)

- **Obsidian core Templates**: the right *storage* model (one user-visible vault folder, templates
  are plain notes) with a too-thin vocabulary (3 variables) — which is *why* Templater exists.
  Five-year-old unshipped asks: cursor tabstops, new-note-from-template, subfolder discovery.
- **Templater** (the de-facto state of the art): its **declarative tier-1** (title, dates+offsets,
  frontmatter, clipboard, prompt, suggester, cursor) covers ~90% of real usage — *our vocabulary
  already matches it*. The last 10% is arbitrary JS execution: its own README warns it is an RCE
  surface; a consent gate was refused upstream; auto-apply races produce silently-empty daily
  notes. **The lesson: take tier-1, refuse the cliff.**
- **Logseq/Notion**: two philosophies (in-document blocks vs database-scoped prototypes). The
  transferable idea is Notion's *context-scoped auto-apply* (≈ folder templates done properly).
- **Engine architectures**: the industry answer for "templates must not execute code" is a
  **logic-less/declarative grammar** (Mustache/Liquid family) with a **single-pass tokenizer**;
  VS Code's snippet DSL is the reference for **ordered tabstops with placeholders**. Moment-style
  date tokens remain the user-facing convention; formatting belongs to `Intl` (locale + calendar
  per call), not hand-rolled English arrays.

## Design options

**A — Reconnect & Patch** *(fast, low risk)*: honor the folder setting, add create affordances,
fix the engine defects in place. **Not the destination** — leaves English-only dates, race-shaped
daily flow, no-abort prompts, patched-not-tokenized engine. It IS the correct first phase.

**B — Declarative Template System v2** *(RECOMMENDED)* — four pillars:
1. **Discovery**: templates are visible `.md` files in ONE user-chosen folder (default
   `Templates/` at the universe root), honored by Rust as the single source of truth; subfolders
   supported; folder excluded from search/Sky/link surfaces (flag at write time, Rule 8);
   validated on save AND read with a visible, localized error — never an unexplained empty picker.
   One-time **visible, lossless copy** of anything in the hidden dir.
2. **Engine**: ONE single-pass tokenizer (dead sync twin deleted); existing `{{var}}` grammar
   preserved (no migration imposed on user files) and extended — Moment-compatible date tokens
   formatted via `Intl.DateTimeFormat` with per-call locale **and per-call calendar: Hijri via
   Eisa's own vendored engine as a first-class `{{date}}` calendar — a capability no competitor
   structurally offers**; declarative prompt/suggester fields with **cancel-aborts-insert**;
   answers are inert text (never re-scanned); unknown-variable warnings, localized ×15.
3. **Application**: New-note-from-template · Save-as-template · folder auto-apply with
   **path-boundary deepest-match** and **template-FM merge** (makes the help docs true) — all
   expanded **atomically inside the Rust create write** (Write-Time Derivation; the Templater
   race class becomes structurally impossible); insert keeps the just-fixed path-guarded
   dispatch; **CM6 ordered tabstops with placeholders** (leapfrogs both Obsidian systems).
4. **Surface**: ONE consolidated Settings → Templates section; a real folder-templates +
   hotkeys editor; consistent toggle gating across ALL entry points; every error localized ×15;
   help docs rewritten to describe reality.
   **Explicitly OUT**: scripting/JS of any kind (templates are inert data — the federation-safety
   line for cUniverses), read-time dynamic commands, startup templates, web-fetch variables,
   external template-language libraries.

**C — Templater-class scripting, sandboxed** — **rejected on principle**: buys the last 10% of
Templater's value at the cost of its entire footgun inventory, catastrophically worse under
cUniverse federation (templates from other people's Universes are untrusted input by definition).
If a genuine need materializes it returns as its own concept paper with a consent gate designed
in — never as scope creep here.

## The identity-clean invariant (Boss ruling 2026-07-19) — and its three engineering consequences

**A template never contains `cid_cn` or creation date/time.** Verified consequences in today's
code, each needing explicit handling in the plan:

1. **Editing a template must not stamp it.** Opening any `.md` runs `ensure_cid_cn_cmd`
   (`store.ts:2168/2468` → `canonical.rs:1224`), which INJECTS a `cid_cn` on first open. The
   templates folder must be exempt from this injection path — otherwise the first edit of a mold
   gives it an identity.
2. **"Save as template" strips.** Saving an existing note as a template removes `cid_cn`,
   `created` (and any modified/temporal canonical fields) from the copy; the source note is
   untouched.
3. **Casting mints fresh.** Applying a template gives the new note its OWN `cid_cn` and its OWN
   `created:` at creation time; the FM-merge (decision 5) merges the template's *other*
   properties but identity/temporal fields are never merge candidates — they are generated, not
   copied.

## Invariants (must not break; the Phase-2 plan verifies each)

The path-guarded insert dispatch (tests stay green every phase) · single content ownership — no
raw write against any OPEN note; atomic expansion only at creation, before an editor owns the
file · existing `{{var}}` files keep working unchanged · daily-note continuity through the folder
move (copy, never silent re-point) · templates are inert data forever · lossless visible
migration of the hidden dir · Rule-8 performance floor (no boot scans; nothing on the keystroke
path) · the feature toggle survives and gates ALL entry points · Language-First (every new string
×15; grammar tokens stay universal) · Cross-Platform (no explicit-Ctrl in new bindings; flag the
existing Ctrl+T for the macOS pass) · index-flag exclusion applied at write time with a resumable
back-fill · per-build safety inspection + Boss test before every commit.

## The quick unblock — "Templates You Can See" (Phase §1 of the migration, not a detour)

Five small honest changes, landable this week: (1) Rust honors the real folder (setting passed as
a parameter; default a **visible** `Templates/` at the universe root, created on first use);
(2) one-time visible copy from the hidden dir; (3) "Open templates folder" + "New template"
(scaffolds a starter `.md` with the variable card as commented examples) ; (4) the picker's
empty state names the ACTUAL folder and offers New-template; (5) three surgical engine de-fusings
(ordered-token/replaceAll fix · function-replacers for the `$` hazard · line-anchored FM close).
Nothing is throwaway: (1)–(4) ARE pillar 1 landed early; (5) shrinks what the tokenizer must
cover. Even this touches Rust ↔ Svelte ↔ settings — hence inside the migration, not ahead of it.

## Decisions the Boss must make

1. **Approve Option B** as a `/migration`, with "Templates You Can See" as its first phase? *(rec: yes)*
2. **Folder scope** — one folder per Universe (rec — matches the "It is ONE universe" ruling;
   per-Library can be added later additively) or per-Library now?
3. **Prompt cancel** — Escape aborts the whole insertion cleanly (rec) or applies with blanks?
4. **Hijri in `{{date}}`** — in scope for v2 (rec — flagship Language-First proof, engine already
   vendored) or deferred?
5. **Template frontmatter merge** on create — yes (rec — makes the docs true; doorway to seeding
   living-link properties) or keep discarding?
6. **Exclude the Templates folder** from search/Index/Sky/links by default, with a toggle (rec) or include?
7. **Confirm the constitutional line**: scripting permanently out; templates are inert data. *(rec: confirm)*

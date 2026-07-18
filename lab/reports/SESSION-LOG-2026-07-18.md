# Session Log — 2026-07-18 (session began 2026-07-17 evening)

**Headline:** PJ-114 started as "add a right-click menu to Focus mode" and became something much bigger — **NotePane was elevated to a top-principal as the GATE to Knowledge Cognition**, audited, and given a vision + phased roadmap with an approved Phase-1 migration now underway. Along the way: 3 user-facing fixes, 2,550 lines of dead code removed, and one hard process lesson re-learned.

---

## Commits (11, all Boss-validated before commit)

| Commit | What |
|---|---|
| `9f1b57cb` | PJ-114 FM+ design brief + migration Plan (Boss-approved direction) |
| `455a7930` | **§0.1 / PJ-116** — a title typed in Focus now renames the note (was silently discarded); also landed the parked sweep-#3 cascade-freeze protection |
| `2772e2be` | **§0.2** — shared parser-free wikilink finder `findWikilinkAtLineOffset` + 11 unit tests |
| `22a9de41` | **§1.1** — `FM_PLUS_ENABLED` build flag + persisted `focusModePlus` setting (default off) |
| `11ace3fe` | **§1.2** — FM+ footer toggle: localized mark ×15, tooltip-above, RTL, fade-on-type |
| `8ee48a86` | ⋯ note-menu **RTL truncation** fix (fixed-position + measure/clamp) |
| `d410f1ea` | **NotePane Cognitive Gate** — capability audit, vision + roadmap, Phase-1 Architect + Plan |
| `e88cafb8` | **§1** — deleted **2,550 lines** of unmounted editor components |
| `ca6ce954` | **Caret continuity** across the NotePane ↔ Focus switch |
| `cc35524d` | ⋯ menu — anchor by the **note's** direction (opens into the note, never over the sidebar) |
| `6c810836` | **§3a** — read-widening: a link's `created` now reaches the UI (Rust → TS → row maps) |

---

## The arc

**1. FM+ (Focus Mode Plus) — shipped the foundation, then deliberately paused the menu.**
Boss directed a right-click menu for Focus mode. Concept locked: FM+ = an opt-in switch (footer token, single master, default off) that unlocks extra Focus affordances *without* Focus becoming "another NotePane" (Boss's guardrail). Design pass (Art Director & Team, `wf_12bd4169-78c`) → Architect → Plan → build. §0.1/§0.2/§1.1/§1.2 shipped and validated.

Then **v1 of the menu was Boss-REJECTED**: Copy-link-text/Copy-path + Cut/Copy/Paste is strictly *poorer* than the native webview menu. Boss: *"If we only have these limited choices while there is a richer native one, then we don't need it. Do some digging and give me more smart choices."* A research dig (`wf_b5c67f60-646`) reframed it: **the native menu owns the clipboard; FM+ owns knowledge formulation.** Then the decisive Boss ruling: *"it is the NotePane that should have the full living-link, so we are not going to replicate it in the FM"* → **audit NotePane first, cross-check FM+ after.** FM+ menu code was reverted; the shipped toggle/flag/finder stay.

**2. NotePane elevated to a top-principal.**
> *"NotePane is the GATE to the knowledge cognition, it is the key to one's knowledge. If Constellation is going to be a unique, powerful, and smart PKM/PKF system, it will be through the NotePane capabilities. It is the starting point of one's cognitive journey, so we have to make sure it is well-equipped and well-instrumented."* — Eisa, 2026-07-18

**The audit (`wf_baf417ca-22a`) verdict:** NotePane is a competent *Markdown* editor with rich *diagnostic* panels, but **not a living-link editor**. Of the 8 living-link properties, **only 1 (Confidence) is truly settable** on the note surface; Type + Annotation are authorable only via raw `[[type::Target|reason]]` syntax; right-clicking a real `[[link]]` offers only open/copy/edit/remove. **~90% of the gap is wiring engines that already exist.**

**The vision (`wf_cfffaf11-0b3`)** organizes NotePane's instrumentation by the Five Acts and phases the build-out (Phase 0 housekeeping → Phase 1 own the living link → Phase 2 tension first-class → Phase 3 co-visibility → Phase 4 discovery → Phase 5 distillation/synthesis).

**Boss rulings:** Phase 1 = "stretch it" (the four picks + housekeeping + `supersedes` + confidence badge); controls at **three depths**; tension **early** (Phase 2); at-a-glance density **minimal by default, richer by choice via Settings** *(a better answer than either option offered)*; delete the dead components; **one continuous migration**.

**3. Phase-1 Architect + Plan — which overturned assumptions and found live bugs.**
- **The documented dual-layer link storage does not exist.** No LINK file is ever written. For Type + Annotation the **body text is the source of truth**; `note_links` is a derived index rebuilt on every save. A DB-only `setLinkAnnotation` would be silently reverted → item 1.2 was the wrong shape. Boss ruled: **body-text is the write** (File-Over-App).
- **Link identity = `(source_path, target_name, link_type)`** (UNIQUE). No occurrence index — repeated identical tokens collapse; **the 2nd token's annotation is already silently dropped today**.
- **LIVE data-loss bug:** the indexer's preserve gate ignores `confidence`, and the re-INSERT hardcodes `'hypothesis'`, `created = now`, `status = 'active'` → editing an annotation **wipes a user-set confidence**, **resets `created`**, and **resurrects archived links** (so `archiveLink` doesn't survive a save — breaking `supersedes` at its root). Blocking prerequisite, scheduled as §5.
- **"Remove link" strips EVERY link on the line** (5 first-match-on-the-line regexes); right-clicking the 2nd link acts on the 1st. Fixed as part of §7.
- **Backlinks rows cannot be a write depth** — their `sourcePath` is another, usually closed note; rewriting there is the forbidden disk read-modify-write. Write depths = editor right-click · Outgoing rows · inspector. *(Adjusts the Boss's "three depths" ruling.)*

**4. Build progress:** §1 (delete 4 unmounted components, 2,550 lines) ✅ · §2 (align-buttons reproduce) — buttons *work*, but a different defect surfaced (filed) · §3a (read-widening) ✅.

---

## Fixes shipped for Boss-reported issues

- **⋯ menu RTL truncation** — clipped by `.e-desk`'s `overflow-x: hidden`; made fixed-position with measure-and-clamp. Then a **second** report (Latin UI + Arabic note) disproved the viewport-overflow rule: with the sidebar open the menu spilled over the file tree. Correct rule = **anchor by the NOTE's direction, always open into the note**. Verified across all four UI×note-direction combinations.
- **Caret continuity** NotePane ↔ Focus (see the process lesson below).

---

## PROCESS LESSON — Reproduce-First, re-learned the hard way

I shipped **two** caret fixes built on *plausible mechanisms* (first "onDestroy runs in time," then "it's an ordering race") without ever observing the failure. Both failed live; the Boss caught both and finally said **"Enough guessing."**

The rule I broke is a project top-principal: *a plausible mechanism is not a diagnosis; if the bug cannot be reproduced, the ONLY shippable work is the instrumentation that makes it reproducible.* Because release builds disable devtools, the instrumentation had to be **on-screen** — a temporary trace panel. **One** run produced the answer: the hand-off worked perfectly (`NP APPLIED @38`) and was then **overwritten** by the app's own per-tab cursor memory (`tab.cursorPos` → `initialCursorPos`, `NotePane.svelte:848-851`). I had built a **second, competing cursor memory**. The fix deleted mine and fed Focus's cursor into the existing one. Instrumentation removed after confirmation.

**Cost:** two wasted Boss test cycles. **Takeaway:** go to instrumentation *first*, not after the Boss stops you.

Also logged: **§0.2's "proven against the main editor" claim was false** — it refactored `CodeMirrorEditor.svelte`, which nothing mounts. The helper is sound (11 tests, used by FocusPane), but the validation claim wasn't. Caught by §1's dead-code discovery.

---

## New jobs filed (for the PJ ledger)

1. **Markdown tables render as raw pipes** — live preview has *no* table renderer (only the Bases/Lens one). Boss wants real tables. Own concept paper + migration.
2. **Text-align inserts raw HTML** — `<div style="text-align: center">…</div>` visible on the active line, renders only after the cursor leaves, line hard to re-click. Same family as (1).
3. **`BacklinksPanel` "Link it" clobber risk** — raw `readNote` → `write_note` on a possibly-open-dirty note, wrapped in `catch {}`. Same class as the `b6310479`/`baae4533` sweeps. Out of scope for this migration; fix separately.
4. **FM+ menu (PJ-114 §1.3+)** — paused pending the NotePane cross-check.

---

## State at close

- **On `main`, pushed:** everything through `cc35524d`. `6c810836` (§3a) committed — push at session end.
- **Working tree:** clean apart from the runtime `.claude/scheduled_tasks.lock` deletion (leave it).
- **Next:** Phase-1 **§3b** — extract the shared *localized* traversal helper (`fmtTraversed` is byte-identical in both panels **and** hardcoded English; the chip tooltip is a hardcoded English template). ~8 keys ×15 locales. Then §4 (confidence badge + density setting, folding in the `LinkStateChips` extraction), §5 (the blocking indexer fix), §6–§10.
- **Help/manual:** deliberately NOT updated this session — the only new user-facing surface (the FM+ toggle) is default-off and its feature set is paused pending the cross-check. Documenting it now would document something incomplete. **Due when FM+ resumes.**

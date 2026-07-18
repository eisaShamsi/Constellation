# Handover — 2026-07-18 (NotePane = the Gate to Knowledge Cognition; Phase-1 in build)

**Read `docs/Constellation Orientation & Onboarding v3.57.md` first** (highest version), then this file, then `git pull origin main` + `git log --oneline -12`.

## HEAD
`main` — all 11 session commits pushed, ending at `6c810836` (§3a). Working tree clean apart from the runtime `.claude/scheduled_tasks.lock` deletion (**leave it**). No parked/uncommitted code this time.

## THE BIG SHIFT — read this before anything else
PJ-114 started as "a right-click menu for Focus mode" and became the elevation of **NotePane to a TOP-PRINCIPAL**:

> *"NotePane is the GATE to the knowledge cognition, it is the key to one's knowledge. If Constellation is going to be a unique, powerful, and smart PKM/PKF system, it will be through the NotePane capabilities. It is the starting point of one's cognitive journey, so we have to make sure it is well-equipped and well-instrumented."* — Eisa, 2026-07-18

**Audit verdict:** NotePane is a competent *Markdown* editor with rich *diagnostic* panels but **NOT a living-link editor** — of 8 living-link properties only **1 (Confidence)** is truly settable on the note surface; Type + Annotation are authorable only via raw `[[type::Target|reason]]` syntax. ~90% of the gap is **wiring engines that already exist**.

**⚠️ ARCHITECTURAL CORRECTION (supersedes CLAUDE.md's "dual-layer storage"):** **no LINK file is ever written.** For **Type + Annotation the note BODY TEXT is the source of truth**; `note_links` is a derived index rebuilt by `index_note` on every save. A DB-only annotation setter would be **silently reverted**. Link identity = `(source_path, target_name, link_type)` UNIQUE — no occurrence index.

Docs: `PJ-114-NotePane-Capability-Audit.md` · `PJ-114-NotePane-Cognitive-Gate-Vision.md` · `PJ-114-NotePane-LivingLink-P1-Architect.md` · `PJ-114-NotePane-LivingLink-P1-Plan.md` (all in `docs/concept-papers/`).

## ► NEXT ACTION — Phase-1 **§3b**
Extract the shared **localized** traversal helper. `fmtTraversed` is **byte-identical** in `BacklinksPanel.svelte:34` and `OutgoingLinksPanel.svelte:18` **and hardcoded English** (`'today'`, `'3d ago'`); the chip tooltip is a hardcoded English template (`Traversed 3 times · emerging · Last: …`, `BacklinksPanel.svelte:220`). Create `src/lib/links/linkDisplay.ts`, wire both panels, add ~8 keys ×15 locales (`linkState.relative.*`, `linkState.traversed`, `linkState.lastTraversed`). `$t(key, params)` and `$tn(key, count, params)` both support interpolation; plurals are CLDR-shaped (Arabic has 5 forms) — the `×N` phrasing avoids needing a new plural key.

**Then:** §4 (confidence badge + density setting — **fold in the `LinkStateChips` component extraction here**, since the badge touches that exact markup: one pass, one pixel-identical verification) → **§5 BLOCKING** (the indexer fix) → §6 (inspector popover) → §7 (body-token rewrite engine + link identity) → §8 re-type → §9 annotate → §10 supersedes. Full step list with verification clauses + Boss-test hooks: the Plan doc.

## Boss rulings in force (do not re-litigate)
Body-text is the write (File-Over-App) · annotation slot reused (`[[type::Target|reason]]`) · identity `(source_path, folded target, link_type)` · inspector = **popover** · density = **enum setting, minimal by default / richer by choice** · "stretch it" scope (+`supersedes` +confidence badge) · **ONE continuous migration** · **three depths adjusted**: Backlinks rows are **read-only for writes** (their `sourcePath` is another, usually closed note = the forbidden disk read-modify-write).

## ⚠️ Live bugs already found — scheduled INSIDE the migration
- **§5 BLOCKING** — the indexer wipes living-link state (`search.rs:6058/6107/6145`): editing an annotation **wipes a user-set confidence**, **resets `created`**, and **resurrects archived links** (so `archiveLink` does not survive a save → breaks `supersedes` at its root). Needs `safety-inspection` + Reproduce-First.
- **§7** — **"Remove link" strips EVERY link on the line**; right-clicking the 2nd link acts on the 1st (5 first-match-on-the-line regexes, `NotePane.svelte:1141/1280`).
- Duplicate identical `[[type::target]]` tokens already collapse to one row — the 2nd token's annotation is silently dropped **today**.

## FM+ — foundation shipped, MENU paused
Shipped + Boss-validated: PJ-116 title→rename (`455a7930`, also landed the parked sweep-#3 freeze) · shared wikilink finder (`2772e2be`) · flag + `focusModePlus` setting (`22a9de41`) · footer toggle with localized mark ×15 (`11ace3fe`). The **menu was Boss-REJECTED** (poorer than the native webview menu), reframed as a *knowledge* menu, then **deferred**: NotePane owns the full living-link; **FM+ complements, never duplicates**. Resume = the FM+ cross-check **after** Phase 1 lands.

## Newly filed (ledger v1.36)
**PJ-121** render markdown tables as real tables (live preview has NO table renderer) · **PJ-122** text-align inserts raw HTML (visible, hard to re-click — design with PJ-121) · **PJ-123** `BacklinksPanel` "Link it" raw read-modify-write on a possibly-dirty note (clobber class, HIGH, fix separately).

## ⚠️ PROCESS — Reproduce-First, violated and re-learned
Two caret fixes shipped on *plausible mechanisms* without observing the failure; both failed live and cost two Boss test cycles before *"Enough guessing."* **Release builds disable devtools**, so instrumentation must be **on-screen**; one instrumented run found the true cause immediately (a second, competing cursor memory overwriting the first). Also **§0.2's "proven against the main editor" claim was false** — it refactored `CodeMirrorEditor.svelte`, which nothing mounts. **Go to instrumentation FIRST.**

## Standing rules — do NOT regress
**Boss tests EVERY build before commit** (mandatory) · **Reproduce-First on the running app** · **Cross-Platform by Design** (macOS in every decision) · **Full localization ×15 — including brand-ish marks** (the Boss localized `FM+` itself to `وضع التركيز+`) · SO#6/8/9 · Art Director & Team own UX/UI · `npm run build` BEFORE `cargo build --release` · **close Constellation before building** (a running .exe blocks the cargo link with `Access is denied (os error 5)`).

## Environment
Release binary `src-tauri\target\release\constellation.exe` built 2026-07-18 **16:10** (contains everything through `cc35524d`; §3a is source-only — rebuild before the next Boss test). Boss's active universe root = `E:\Cognitive Knowledge`. One location: `E:\مشاريع كلاود\Constellation`, branch `main`.

## Help / User Manual
**Deliberately NOT updated this session** — the only new user-facing surface (the FM+ toggle) is default-off and its feature set is paused pending the cross-check; documenting it now would document something incomplete. **Due when FM+ resumes.**

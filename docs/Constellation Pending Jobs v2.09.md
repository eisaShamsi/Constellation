# Constellation Pending Jobs

**Version 2.09 | 2026-08-31 — PJ-433 CLOSED: the Boot Chooser, Boss-passed on ALL SEVEN stages (1, 1b, 2, 3, 4, 5A, 5B) · A′ folded in (removal never guesses a successor) · docs ×15 · six follow-ups filed (PJ-440…PJ-445) · **the per-cycle whole-app sweep re-run and panelled: 8 new PJs (PJ-446…PJ-453), and the ledger diagnosed as a working NET but a failing QUEUE**. **► BOSS-RULED: PJ-433 + PJ-446 both PASSED and committed (`5e56c00a`). NEXT = the DRAIN CYCLE, and its FIRST item is PJ-454 (102 stamped molds measured on his disk; repair approved, file-list-first).**

> **What changed in v2.09** (**the PJ-433 `/migration`, run end to end in one session: Architect →
> panel → Boss rulings → plan → build → /simplify → inspection → Boss test → close**):
>
> **► NEXT ACTION: THE DRAIN CYCLE, opening with PJ-454.** Boss-ruled 2026-09-01: the Two-Signal
> Choke Point is built there, and the **102 stamped molds are repaired after he approves an exact
> file list** (82 in Eisa Universe, 19 in موسوعة عيسى, 1 in his daily universe — measured, and no
> cast inherited a stamp). The brief for producing that list safely is in PJ-454's entry.
>
> **The rest of the DRAIN CYCLE — Boss-ruled 2026-08-31.** The whole-app sweep at this close
> proved the ledger is a working NET and a failing QUEUE: 8 of its 17 distinct findings were
> already filed, some since 2026-08-11, and ~158 confirmed findings sit invisible inside two
> umbrella entries (**PJ-264 ≈100, PJ-378 = 58**). The panel: *"This sweep spent most of its
> budget re-proving known bugs. The cure is not a better sweep."* **The Boss ruled a DRAIN
> cycle: fix the backlog, run NO new hunt.** First act of that cycle is to UNPACK PJ-264 and
> PJ-378 into individually numbered, visible entries and rank them — a defect nobody can see is
> not filed, it is buried. PJ-434 (the unreachable Linked Universe) and PJ-438 wait behind the
> drain; PJ-437 (identity-relative addressing) remains the direction that outlives them all.
>
> ### ✅ PJ-433 — CLOSED (2026-08-31): the Boot Chooser. Boss-passed on all seven stages
> **The app no longer substitutes a universe, and no longer records its own substitution as the
> user's choice.** When the recorded universe is unreachable at boot, NOTHING activates and
> NOTHING persists: a chooser names the universe, its path and the reason, lists every registered
> universe with live reachability, and waits. Boss rulings that shaped it (2026-08-31): the Boot
> Chooser shape (panel-unanimous), **no Remove on the boot screen**, and **wait for the click**
> when the drive returns.
>
> **The seven live stages, on his screen, on the 15:15:37 release binary:** the honest screen with
> nine universes listed Reachable (incl. موسوعة عيسى at its own root — the may-live-anywhere
> ruling, visible); **1b — close at the chooser, relaunch, same screen returns** (the panel's own
> addition; the ONLY observation that proves *nothing is remembered*, and without it a build that
> silently persisted a substitute would have passed every other stage); the mount-watch lighting
> **"It's back — Open"** by itself and waiting for his click; a deliberate pick honored AND
> remembered across a restart; **"Constellation will then open: Eisa Cognitive Knowledge"** naming
> the successor that then opened, with files intact on disk; the wizard's context-only **Back**;
> and **"Open from folder…"** at the moved location — which handed straight over to PJ-435's
> repair banner, the two features interlocking live on one screen.
>
> ### 🧱 What shipped inside it
> `get_registry_status` (boot reads the recorded `active_id` — `list_universes` hid it behind its
> sort, which is where the silent fallback was born) · `check_universe_reachability`
> (spawn_blocking; machine keys the UI translates) · **A′**: `remove_universe_from_registry` sets
> `active_id = None` — never an `entries.first()` guess — and the confirm dialog NAMES the
> successor it will open (one computation, captured, so promise and act cannot drift) ·
> `enterUniverse` (one enter kernel) + `finishBoot` (one post-activation tail — which also cured a
> PRE-EXISTING first-run defect: a universe created through the wizard ran with no file-watcher
> and no federation listener until restart) · `BootChooser.svelte` · second-screen title from the
> universe actually open · i18n ×15 · docs ×15.
>
> ### 🔍 What the gates caught (all fixed before the Boss saw it)
> **/simplify (4 agents, 14 fixes):** the standout — my unconditional boot notify would have made
> the hidden second screen re-walk ~8,000 note files on EVERY normal boot; guarded. Also: a mirror
> DTO cut, the successor single-sourced, completion-keyed listener flags, `spawn_blocking`.
> **Diff-scoped inspection (2 LOW, both fixed):** an unguarded mid-tail `await` whose failure
> landed in an unmounted chooser (console-only, devtools off in release) and left listeners
> untracked → the await is guarded, every unlisten registers immediately, and a post-activation
> failure now RE-OPENS the chooser carrying its error. **Test pipeline:** auditor → inspector
> (3 rounds, 37 claims; two REJECTIONS: "the red Remove button" is red only on hover, and the
> reachability chip can lag a moment) → panel (FIX-FIRST, six edits incl. Stage 1b) → Boss.
>
> ### 📋 Filed from this job (SO#9.2) — PJ-440 … PJ-445
> Three are `/simplify` findings deliberately NOT taken mid-migration (a shared universe-row
> component; a dispatcher-level `appReady` gate; **the un-generalized LEAVE half — the
> remove-last→create door runs none of `handleUniverseSwitch`'s ~50-line residue sweep**), two are
> pre-existing Rust gaps surfaced while mapping (`migrate_legacy_data`'s half-activation; the
> corrupt-registry lenient load), and one is the Phase-4 audit's own catch (**PJ-445** — a FAILED
> "Open from folder…" still moves the recorded choice, because `open_existing_universe` writes
> `active_id` before activation; bounded and self-announcing, but a narrow contradiction of this
> feature's own promise). Full entries in the body.
>
> ### 🔬 Phase-4 audit (the `/migration` close) — PASS
> **4A invariants: all nine STILL HOLD, zero regressions** (empty-registry→wizard; the MIG-079
> guard; `switch_lock`; PJ-435's heal strictly after the reachability check; MIG-100 write
> authority; all-unreachable still reaches a door; second-screen title semantics — *strengthened*,
> both `universes[0]` readers gone; MIG-061 §J.2 listener ordering; i18n parity 15/15).
> **4B drift: no new bypass** — every `setActiveUniverse` caller accounted for, and **no
> `listUniverses()[0]`-as-active reader remains anywhere in the repo**. **4C migration path: PASS**
> across all seven upgrade/downgrade scenarios — no data loss, no wedge; downgrade tolerates
> `active_id: None` (it simply resumes the old silent fallback). Its four notes: M1 = PJ-444,
> M2 = PJ-445, M3 = the deliberate documented listener tradeoff, M4 = an imprecise code comment,
> **corrected in this commit**.
>
> ### 🌍 A docs find worth more than the feature's own paragraph
> The ×14 manual pass provoked a **truth sweep** (one agent flagged, outside its task, that the
> Hindi manual still carried the PJ-435-era **false auto-repair promise**). Result, 14/14 checked:
> the false promise survived in **Hindi only** (fixed) — but the **Full-re-read warning was
> missing from ALL FOURTEEN translations.** *Do not reach for a Full re-read to "fix" a move — it
> rebuilds from scratch and resets every link's birth date to today* existed in English and in no
> other language. **The PJ-435 pass removed the lie everywhere and never checked that the truth
> had been carried** — every non-English reader had the move procedure without the one warning
> that protects the link graph's age. Now in all fifteen, each anchored to its own locale's
> `movedRepairNow` + `fullReread` strings (verified programmatically). **Method note: a correction
> sweep that only deletes the false sentence leaves the docs honest and incomplete.**
>
> ### ⚠️ Honest coverage note carried forward
> Four states ship code-verified but NEVER SEEN LIVE: all-unreachable, no-recorded-choice, the
> inline pick-failure path, and the Unreachable chip on a list row. Desktop control was declined
> and sandbox registry manipulation is unreliable (the MSIX ghost — re-confirmed this session:
> the file the sandbox reads shows ONE universe and a three-week-stale mtime, while the running
> app showed NINE). They are covered by Rust pin tests, four review agents, the inspection and 37
> verified UI claims — not by a live run. **The per-cycle whole-app safety sweep for this cycle
> did not complete** (all 14 hunters died on a model rate limit; an empty findings list from a
> run where nothing ran is NOT a pass) — it is re-run at this close; if it cannot complete, it
> carries to the next cycle boundary, stated rather than assumed.


> **What changed in v2.08** (**session-close PCS on the Boss's order: "PCS + Orientation, and
> handover file and prompt for a new session, including updating all manuals and help files"**):
>
> **► NEXT ACTION: PJ-433** — the silent boot fallback persisted as the user's choice. Then
> PJ-438 / PJ-434; PJ-437 (identity-relative addressing) remains the direction. Handover for the
> next session: `lab/reports/HANDOVER-2026-08-31.md` (with the ready-to-paste prompt).
>
> ### ✅ The Linked Universe naming ruling — DOC HALF DONE (2026-08-31)
> A 26-file workflow (English User Manual + 11 help topics + 14 translated manuals, each with an
> independent verifier) renamed every prose use of "cUniverse"/"Child Universe" — and each
> language's NATIVE equivalent — to **Linked Universe**. 25/26 verified clean first pass; Hebrew's
> two hyphen-variant residuals (יקומי-בת ASCII-hyphen, יקום־בן maqaf) were fixed and the file
> re-swept to zero. The ONLY permitted survivals are verbatim quotes of on-screen labels the app
> still shows (each byte-checked against its locale file, each carrying a predates-the-name
> parenthetical) — because **PJ-331's UI half is still open**: the running app's own labels
> (`universe.setup.addChildUniverse`, `universe.manager.children`/`addChild`,
> `secondScreen.dashboard.childUniverses`, `federation.cuniverseLabel`/`warningBadge`,
> `constellationMap.childUniverse`, `styleSetter.labels.cuniverse` — ×15 locales) still carry the
> old names. **PJ-331 is now precisely scoped: rename those visible strings (build + Boss test),
> then strip the docs' honesty parentheticals.** It is the most visible naming drift left.
>
> ### 📌 Also carried out at close
> Session log §29 (state of standing); MoCh 2130 §4; orientation v4.25; this ledger. The PJ-435
> close itself was committed at `e48ff80e` (v2.07's story).


> **What changed in v2.07** (**PJ-435 live-passed and verified; the inspection then earned its
> standing order; one cosmetic PJ filed**):
>
> **► NEXT ACTION: PJ-433** (the silent boot fallback persisted as the user's choice) tops the
> Group-1 queue, then PJ-438 / PJ-434; PJ-437 (identity-relative addressing) remains the
> direction that outlives them all. PJ-435 is CLOSED — smoke re-test passed 8/8 on the 21:23
> build (session log §28), everything since `4aee6ea2` committed.
>
> ### 🛡 AFTER the pass: the diff-scoped inspection returned 8 confirmed findings — all fixed
> All in the new PJ-435 code, all on edge paths the live test never walked (session log §27):
> a COPIED moved-but-unrepaired universe inherits a record aiming the rewrite at the SOURCE's
> living root (HIGH — activation now deletes foreign/unreadable records, and the command refuses
> them); an unreadable record blacked out every notice incl. the repair button itself (HIGH —
> `moved` now parses, never bare-exists); the window's stale in-memory stores could silently
> clobber the repaired JSON on the next save (MED — post-repair reload through the boot path, the
> proven mig108 pattern); the record write was the file's one non-atomic manifest write (MED —
> `atomic_write`); a swallowed disarm left false text armed forever while re-clicks rotated the
> only genuine backup toward deletion (LOW×2 — disarm-with-retry + the already-repaired fast path
> that never re-runs the engine); a failed post-repair init_db was eprintln-only (LOW — retry +
> diagnostics receipt; the receipt now also carries the remapped-row count). RED→GREEN throughout;
> suite **1,612/0** (+4 = the tests added). The re-inspection then confirmed all six dead
> with ONE new LOW (unconditional "removed" logging at the removal sites — fixed via the
> reporting helper; its read-only premise REFUTED by probe: Rust 1.94 deletes read-only files).
> Final: suite **1,613/0**, binary 21:23:37.
>
> ### ✅ PJ-435 — CLOSED (2026-08-31): Boss-passed twice — the full three stages, then the smoke re-test
> Stage 1 (the honest banner, 8 steps), Stage 2A (the small universe: cold-start persistence — also
> proven through a genuine PC crash — and the predicted 3→2 duplicate fold, without a relaunch), and
> Stage 2B (501 notes, the scale where the self-heal caps refuse and the one-click repair is the ONLY
> path) all passed on his screen. Offline verification through findings-verifier, **7/7 CONFIRMED**:
> **1,000 of 1,000 link `created` dates byte-identical** to the repair's own pre-write backup;
> **501 of 501 review-schedule rows re-addressed**; record deleted, backup + journal present,
> receipts logged in both universes. His reported "~95 seconds" forensically attributed: **91.6 s**
> was the pre-existing MIG-003 identity-injection pass reaching 500 generator-made stamp-less notes
> for the first time — the PJ-435 rewrite itself fits in ~3 s, and his daily universe would surface
> at most 13 candidates (27 empty cid_cn − 14 exempt templates), seconds at an ordinary boot.
> CLAUDE.md's storage section amended in the same commit, as its own closing instruction ordered:
> the MOVE exposure of `created` + `review_schedule` is closed; the REBUILD exposure remains, owned
> by the PJ-437 direction. Session log §26.
>
> ### 🆕 PJ-439 *(LOW — Group 3 — cosmetic)* — the relocation backup's db file is named `search.db.pre-mig108`
> `take_snapshot` (mig108.rs:564) hardcodes the filename regardless of the PJ-435
> `backup_dir_name` parameter, so `relocation-backup\` holds a file whose name claims a different
> migration. Misleading to anyone hand-restoring. Renaming touches restore guidance → its own small
> job, filed rather than reopening a Boss-passed build.
>
> ### 📌 PJ-331 scope measured wider: the User Manual violates the naming ruling 24×
> While sweeping the manuals for the false auto-repair claim (2026-08-31), a grep found the
> ENGLISH User Manual uses "cUniverse" / "Child Universe" **24 times** — including a whole
> "### Child Universes" section — against the Boss's naming ruling ("it is a Linked Universe,
> never cUniverse/Child Universe, in any user-facing text, help file, User Manual, or new
> document"). The 14 translations presumably mirror it. This is PJ-331's rename job (visible
> strings), now explicitly including **User Manual ×15**; filed rather than silently parked —
> a 15-file systematic rename does not belong inside the PJ-435 close.
>
> ### 📌 Carried datum: PJ-438's which-route-wins question got no new evidence
> The Stage-2B tutorial asked which first-indexing route the Boss observed (progress strip or not);
> his report did not say. The race stays characterized by code-reading only.


> **What changed in v2.06** (**two Boss passes committed at `4aee6ea2`; the portability arc ran from
> a ruling to a shipped repair in one day; five new PJs filed and one closed in-pass**):
>
> **► NEXT ACTION: the Boss runs the PJ-435 test** — a real move of a disposable universe. Commit on
> his pass.
>
> ### ✅ PASSED AND COMMITTED (`4aee6ea2`): PJ-407 + PJ-409, PJ-428, PJ-431
> PJ-407 Boss-verified on his screen (7494 → 7496 notes; 27 dead links resolving). PJ-428's refusal
> fired verbatim on his own attempt (`zarquan` into his open universe — refused, nothing created,
> list unchanged). PJ-431 (the on-open identity write never told the index) fixed with a RED-proven
> test.
>
> ### 🧭 THE PORTABILITY RULING → CLAUDE.md, and the audit it triggered
> Boss-ruled: a universe may live ANYWHERE; "identify" = explicit registration must work anywhere,
> NOT auto-discovery. Audit verdict: **capability already honoured by construction** — no location
> constraint exists in production code. What was wrong was the TELLING: the docs promised an
> auto-repair that did not exist, and the first-run button described the ruling's own operation as
> its opposite. Both fixed; **PJ-433…PJ-436 filed** (silent boot fallback persisted as the user's
> choice; unreachable Linked Universe shown as present-and-empty; the move gap = PJ-435; the
> unwired Open Folder action) and **PJ-437** (the root direction: the index addresses notes less
> portably than the durable layer beneath it).
>
> ### 🚨 PJ-435 — BUILT (2026-08-30), the full four phases, awaiting his test
> The panel's philosophy review re-scoped it first: **CLAUDE.md's storage section was STALE** —
> `earned.jsonl` (MIG-104) already carries walks/trust/retire/priority on disk, identity-keyed;
> what a move (or the recommended Full re-read!) actually destroys is **every link's `created`**
> (234,917 of 234,917 carry one) **and the path-keyed review schedule** (8,033 rows). The section
> was corrected; the harmful re-read advice pulled from the manual the same hour it was written.
> Then, on his order ("Handle the earned.jsonl first, then proceed"):
> - **Phase 0** — the earned layer proven through the SHIPPING restore across a physical move, with
>   a control and both casualties pinned as executable assertions; the CLEAN fixture reproduces.
> - **Phase 1** — ONE detector (`heal_paths_after_move`): pair persisted to `relocation.json`
>   BEFORE healing, read-back verified; a COPY never arms; second moves CHAIN the original root;
>   moving back home disarms; the duplicate registry entry on reopen is dead (identity-match +
>   mandatory repoint). Five behavioural tests; **two caught real bugs in the first version**.
> - **Phase 2** — `DriftReport.moved`; the honest sentence ×15 REPLACES the drift+phantom rows
>   (both suppressed while armed — each offers a destructive button on a moved universe).
> - **Phase 3** — `relocate.rs::repair_moved_universe`: verified backup into its OWN directory
>   (`take_snapshot` parameterised, 11 call sites), its OWN journal file (never mig108's — the
>   boot resume would present a crashed relocation as a half-finished unification), no move phase,
>   post-run trigger recreation, one-click, idempotent-by-re-click. **The engine's destination
>   purge is now CONDITIONAL** (spares any dest row with no old counterpart — also fixes a latent
>   mig108 defect that deleted a note genuinely created in a crash window). RED-proven.
> Suite **1,608 / 0** (+8 = exactly the eight tests added). Binary 2026-08-30 06:19:53, chain
> verified, strings in the bundle. Docs updated to describe the SHIPPED repair.
> **Honest gap:** the DIRTY end-to-end (self-healed rows through the full command) is unit-proven
> only; the Boss's test exercises the CLEAN end-to-end.
>
> ### 📌 The self-matching trap, third instance — caught BEFORE first run this time
> A negative source-text assertion whose own literal contained the forbidden token. Truncate at the
> test module + build the token with `concat!`. The first two instances cost revert-and-rerun to
> find; this one died unborn.

---


> **What changed in v2.05** (**PJ-407 built and its blast radius corrected DOWNWARD, PJ-409 closed
> in-pass, three more dot-doors closed than the first pass claimed, eight new PJs filed. The review
> panel found a foreign agent's code inside my source file, and a test that would have PASSED while
> leaving the job half done.**):
>
> **► NEXT ACTION: the Boss runs the PJ-428 test.** PJ-407 is **PASSED** (2026-08-29); commit
> everything on his PJ-428 pass.
>
> ### ✅ PJ-407 — BUILT (awaiting his test). The filing was wrong about its own defect.
> Filed as *"importers never index what they import — 400 imported notes exist on disk with no
> `note_meta` row."* **Measured: no live victim** — reconcile re-adopts an import under its
> `max(200, 10%)` cap, and his universes carry no such orphan population. The REAL defect was next
> door and had been invisible for as long as the notes existed: **a note whose file name begins with
> a dot is invisible twice over** — never indexed (`search.rs:9041`), and never reportable as missing
> either, because the orphan walk skips it on the same rule. Two of his real notes, 23 KB and 35 KB,
> in `Computer Science\Algorithms & Data Structures`. Built: the counter + a third notice row (no
> button — a repair would skip them on the very rule that hid them), and the doors below.
>
> ### ✅ PJ-409 — CLOSED in the same pass
> `sanitize_filename` called `String::truncate(200)`, which **panics** off a UTF-8 char boundary. An
> import is exactly where non-ASCII titles arrive, and the panic aborts the command *after* some
> notes are already written. Fixed with a 300-Arabic-character regression test. Closed in-pass rather
> than filed because it is inside the function this change edits.
>
> ### 🔴 A REVIEW AGENT'S CODE WAS INSIDE MY SOURCE FILE, AND THE SUITE WAS GREEN
> `importers.rs` was 1,159 lines; the authored diff ended at 1,111. Lines 1113–1159 held
> `mod tests_pj407_refute_l3_05` — a refuter agent's `println!` probes, left behind from an earlier
> round; earlier variants carried `assert!(false)` and a syntax error. It compiled, so `cargo test`
> counted it as **passing tests**. Second time in two days a test count moved the wrong way without
> failing (26 Aug: my insertion stole an existing test's `#[test]`). **A green suite is not evidence
> about what is in the file.** Removed.
>
> ### 🔴 "This one line is the whole defect" — FALSE, written into a durable comment
> Three lenses independently extracted `copy_full_tree`, compiled it and ran it: **a folder holding
> `.NET.md` lands `.NET.md`.** That routes `markdown | folder | bear | obsidian`, and **`obsidian` is
> the default selection in the first-run Universe Setup wizard** — the likeliest import a new user
> ever performs. A fourth door too: `sanitize_template_stem` trimmed **trailing** dots only, so "Save
> as template" under `.Draft` wrote `.Draft.md`, the picker skipped it, and the command returned
> `Ok` — a silent false success on a write path. Both fixed (`unhide_md_leaf`, narrow and tested;
> the template stem); all four false comments corrected. One day after the law written for that shape.
>
> ### 🔴 The test would have PASSED while leaving two wrongly-named files
> `HideFileExt = 1` on his machine, so Explorer renders `.NET.md` as `.NET`. The instruction
> *"rename to `NET.md`"* produces **`NET.md.md`** — and because a note's name comes from frontmatter
> `title:`, that still indexes as `.NET`, still repairs the links, and still clears the bar. **All
> three stated post-state signals pass.** Rewritten to "press Home, delete only the leading dot."
> Also: the draft told him 25 links were dead and never said not to click one — `handleLinkClick`
> creates the missing note **in the source note's folder**, taking the exact name the rename needs,
> and **22 of the 25 sources are in that folder.**
>
> ### ⚖️ TWO PANEL CLAIMS REFUTED BEFORE THEY REACHED HIM
> (a) *"Constellation writes an identity line into each file by itself at a following launch, whether
> or not you open them."* **FALSE** — `mig003_backfill_cid_cn` is gated on
> `stored_note_meta_version < NOTE_META_SCHEMA_VERSION`; his live value is **1** and the target is
> **1**, so it short-circuits, and every other `ensure_cid_cn` caller is the note-OPEN path. A claim
> about writing into his notes is not one to relay unchecked. (b) *"It will open كون عيسى — the only
> universe in the list."* **Not asserted, then settled by the inspector.** A file-ID comparison
> **cannot disagree with itself** when both paths go through one redirect; `fsutil hardlink list`
> can, and showed one NTFS record — so the registry really does hold one entry, and **`Eisa Cognitive
> Knowledge` is not registered at all.** The test's route works either way and stands.
> Also corrected: the panel's "37 notes under two `.trash` folders" — a direct `find` says **94**
> (73 + 21). See PJ-425.
>
> ### 🔢 THREE link counts, all correct, measuring three different things
> **25** rows in `note_links` (its `UNIQUE(source_path, target_name, link_type)` collapses a repeat
> within one note) · **28** occurrences in the text of those 25 notes (the inspector's figure — it
> counted the FILES, I had counted the INDEX) · **38** universe-wide across **27** files, because the
> two hidden notes carry **10** links of their own that are in NO index, an unindexed note's outgoing
> links never having been parsed. 38 − 10 = 28. The test states 25 notes / 28 links, scoped to notes
> pointing AT the pair. Recorded because the third figure surfaced only from a whole-universe grep
> left running after the question had already been "answered" from the database.
>
> ### 🔴 MY OWN false sentence, caught by my own check
> I wrote that his daily universe is *"the only universe with hidden notes."* `MIG108 Rehearsal`
> holds the same two files; the inspector's broader search found a backup universe as well. Cut.
>
> ### 📄 DOC DRIFT FIXED: a refuted cause still live in the User Manual
> `docs/User Manual.md:321` still carried the cause corrected in the app ×15 the day before — that a
> note leaves no delete record because Constellation *"never gave it an identity of its own."* False
> for all 8; a duplicate had claimed it first. Corrected, then **verified against the code** rather
> than against the string it now matches. The 14 translated manuals do not contain that chapter at
> all (~1,635 lines vs English's 2,687) — PJ-394's gap, not this pass's.
>
> ### 🆕 FILED: PJ-420 … PJ-427
> **PJ-420** (HIGH) note rename admits a leading dot · **PJ-421** (HIGH) folder rename, same, whole
> subtree · **PJ-422** (MED) New Folder · **PJ-423** (LOW) New Library · **PJ-424** (MED) daily-note
> and quick-capture folder fields · **PJ-425** (LOW) the counter does not descend into
> dot-directories · **PJ-426** (MED) the delete archive cannot record a row with an empty `cid_cn` ·
> **PJ-427** (MED) the de-adoption count reaches no user surface (this is the former "D1", filed
> rather than put to him — SO#10 forbids a ruling request against an unreconciled record, and nothing
> here needs his decision today). **PJ-416** (canvas with a leading dot) belongs to the PJ-420…425
> family and should be resolved with it.
>
> ### 🛡️ THE PER-BUILD SAFETY INSPECTION CONFIRMED ONE FINDING — NOT IN THIS DIFF
> **PJ-428 (HIGH, LATENT)** — MIG-112's declared-library exemption (`reconcile.rs:539`, committed
> yesterday in `890aae25`) KEEPS a registered library's rows when it sits behind a universe manifest,
> while every walker and the watcher fence it out. The rows go stale and **the drift report cannot see
> the subtree, so the boot notice reports a clean launch.** Worse: `reindex_changed_paths` Pass 2 has
> no fence at all, so an external rename purges the row and its earned link data — which `search.db`
> alone holds. **Measured latent: 52 library entries across 8 universes, ZERO meet the precondition.**
> Not folded in — it is not in this diff, and fixing it would alter the fence the Boss validated
> yesterday. Flagged to him rather than parked (WA#6).

---

## 🕳️ Filed 2026-08-27 — the dot-name doors PJ-407 did NOT close

*One concern: **a name beginning with a dot makes a note, a folder or a library invisible to the
entire app, silently.** PJ-407 closed three doors (both importers + the template namer). These are
the ones left, each verified in the working tree, each needing a decision I am not taking on the
Boss's behalf: **strip the dot silently, or refuse and say why?** Obsidian's own stated answer is
"most likely preventing users from creating .dot files" — i.e. refuse. Either way it needs a string
in fifteen languages, which is why these are filed rather than folded in.*

**Related and already filed: PJ-416** (a canvas created with a leading dot is returned `Ok`, then
permanently invisible) belongs to this same family and should be resolved with it, not separately.

### 🚨 PJ-420 *(HIGH — Group 1 — silent invisibility + link breakage)* — renaming a note to a dot-leading name hides it, with no error and no way back in the app

`rename_item`'s `.md` branch (`libraries.rs:2081–2095`) validates containment and collision and
**never sanitises the leaf**, unlike `create_note`, which routes through `note_display_filename`.
Rename a note to `.Foo` and: it disappears from the sidebar, every incoming wikilink stops
resolving, the `note_meta` row survives so PJ-407's new notice stays **silent** about it, and there
is no in-app route to rename it back — the only repair is File Explorer. Entry points:
`NoteEditor.svelte:510`, `+layout.svelte:7414`.

### 🚨 PJ-421 *(HIGH — Group 1 — silent invisibility of a whole subtree)* — renaming a folder to a dot-leading name hides everything under it

`rename_item`'s folder branch (`libraries.rs:2031–2049`), same omission. The subtree vanishes from
the sidebar **and** from the Move picker. Strictly worse than PJ-420 in blast radius, equal in
mechanism — fix both in one pass (Whole-Ecosystem Fix Law).

### 🆕 PJ-422 *(MED — Group 2 — silent false success)* — New Folder admits a leading dot

`sanitize_name` (`libraries.rs:1073–1082`, called at `:1805`) does not trim leading dots. The folder
is created on disk, returns `Ok`, renders nowhere — and a retry then reports *"already exists"*
about a folder the user cannot see. That second message is what makes this worse than a no-op.

### 🆕 PJ-423 *(LOW — Group 3)* — New Library admits a leading dot

`create_new_library_at` (`libraries.rs:4601`), same sanitiser. **Not** invisible in the same way —
libraries render from `libraries.json`, not from a walk — so the harm is confined to what reconcile
can reach. Filed for consistency of the rule, not for damage.

### 🆕 PJ-424 *(MED — Group 2 — free-text field, whole-feature blackout)* — the daily-note and quick-capture folder fields admit a leading dot

`libraries.rs:7458`, `:7466–7492`, `:7515`; UI at `SettingsModal.svelte:3042` / `:3051`. A dot-named
daily-note folder removes every daily note from the index, search, the tree, backlinks, the task
scan and the calendar dots at once — and the field is free text with only a partial guard.

### 🆕 PJ-425 *(LOW — Group 3 — coverage gap, not a false statement)* — the hidden-note counter does not descend into dot-directories

`reconcile.rs:1050–1100` counts dot-named `.md` FILES; it does not walk INTO a dot-named directory.
Deliberate for now. **Whoever fixes this must first exclude the bookkeeping set.** Measured
2026-08-27 on `Eisa Cognitive Knowledge`: **94** `.md` files sit under dot-directories, and every
one of them is in a `.trash` — 73 under the universe's own `.trash`, 21 under
`التصوير/.trash`. (The review panel reported 37; the direct `find` says 94. The gap is the
counter's `!known` gate, which excludes any that still carry an index row — so the number a naive
recursion would SURFACE is not the number of files on disk. Whoever takes this must derive it from
the shipping predicate, not from a walk.) Either way the set is entirely trash, and the remedy the
notice names — "remove the leading dot" — must never be applied to `.trash`.

**Explicitly NOT a gap — do not file, do not "fix":** a file named exactly `.md`. Every note-detection
site in the codebase, the indexer included, uses `Path::extension()`, which is `None` for such a
name. Making the counter disagree would report 1-byte junk as a "note" and attach a remedy that does
not work. Measured with a `rustc` probe 2026-08-27. `Eisa Universe` holds exactly two such files,
both 1 byte: `الكون المعرفي/Atlas/Dots/People/.md` and `الكون المعرفي/Atlas/Sources/Books/.md`.

---

## 🛡️ Filed 2026-08-27 — the per-build Safety Inspection's one confirmed finding

*Run diff-scoped over `importers.rs`, `reconcile.rs`, `universe.rs`, `driftReport.ts`,
`+layout.svelte` per the standing order. **One CONFIRMED finding, HIGH — and it is NOT in this
diff.** My reconcile.rs hunks are at lines 128 / 197 / 242 / 264 / 592 / 1051 / 1359; the finding is
at **539**, which is MIG-112 code committed yesterday in `890aae25`. `search.rs`, the other half, is
not touched by this pass at all. Filed, not folded in, per the 2026-08-25 precedent — and because
fixing it would alter the very fence the Boss validated yesterday, un-testing it.*

### ✅ PJ-428 *(HIGH — Group 1 — index-divergence + irrecoverable loss of earned link data)* — FIXED 2026-08-29, Boss-ordered ahead of the PJ-407 test — a declared library behind a universe manifest was KEPT in the index but fenced out of every walker

**Boss: "Fix the High App-Killer first."** Fixed in three parts, each pinned by a test proven RED
against the pre-fix code and GREEN after:

1. **The irrecoverable half — `search.rs`, Pass 2 of `reindex_changed_paths`.** Pass 2 now carries
   the SAME foreign-universe fence as Pass 1. The invariant, stated once in the code: **what Pass 1
   will not index, Pass 2 must not purge.** Pass 2's old note claimed a delete "is correct in every
   scope … it can only ever remove rows, never create them" — true of the row, false of the user's
   data, and the asymmetry with Pass 1 was the whole defect. Self-correcting for the case that
   SHOULD purge: if the nested universe folder itself is deleted its manifest goes with it, the
   check reads false, and the prefix-purge proceeds exactly as before.
   Test: `both_passes_of_the_watcher_agree_on_what_is_ours` (RED: "found 1").
2. **The false-success half — `reconcile.rs`, `run`.** A registered library whose path resolves
   inside a foreign universe is now counted and logged, and clears `walk_complete`. That reuses the
   field's own existing contract — *"a sweep that could not look must never report 'nothing
   changed'"* — so it needs **no new user-facing string in fifteen languages**, and it disables
   dead-row removal for the pass, which is the conservative side to fail on. Cost is per REGISTERED
   LIBRARY (~19–25), not per note; it cannot scale with the 8,031 notes the walk visits.
   Deliberately NOT placed in `collect_md`: an ordinary linked-universe root being skipped is not a
   failure to look at OUR content, and an existing MIG-112 test pins exactly that.
   Test: `a_declared_library_the_walk_could_not_reach_makes_the_sweep_incomplete`.
3. **The reachable door — `universe.rs`, `link_library_as_universe`.** Refuses to MAKE a universe
   out of a folder inside the active universe, or one standing above a registered library, with a
   plain-language reason. Placed AFTER the "already a universe" delegation so OPENING one still
   works — it is the CREATION that manufactures the contradiction. Invents no new rule: MIG-108
   ("One Universe, One Location") and MIG-112 ("a universe is never content of another universe")
   both already point this way.
   Test: `a_folder_inside_the_open_universe_cannot_be_made_into_one`.

**Suite 1,598 / 0** (was 1,595; +3 is exactly these three tests, which is also the check that
nothing was stolen or lost). The MIG-112 regression pin
`collect_md_skips_a_linked_universes_root_without_marking_the_walk_incomplete` still passes.

**Two bugs in my own TESTS, both caught only by reverting the fix and re-running — neither would
have failed:**
- `code_of` ran to EOF for a function with no `#[tauri::command]` after it, so the extracted body
  swallowed the test module and an assertion **matched its own literal**. Green forever.
- The first attempt to fix that truncated the FILE at the first `#[cfg(test)]`, which broke the
  search outright: `search.rs` carries inline test modules from line 495 while the function under
  test sits at 13573. The body is now bounded at whichever marker comes first.
- And the RED *proof* itself was wrong once: a blanket string-replace rewrote the test's own
  assertion to match the reverted code, so the test "passed" against code that lacked the fix.
  A revert must touch production lines only.

**REVIEWED AFTER THE FACT, and it found seven required changes (2026-08-29).** The agent limit reset,
so the panel I had recorded as impossible was convened: 22 agents, three lenses, every finding
refuted independently. Verdict SHIP-WITH-CHANGES; all seven applied.

- 🔴 **My change 2's comment was FALSE.** It claimed clearing `walk_complete` ended the clean-launch
  defect. `has_findings()` deliberately excludes `walk_complete`, and the frontend returns early on
  `!hasFindings(r)` before reaching the `!walkComplete` branch — so the flag went false, dead-row
  removal was correctly disabled, **and the user still saw a clean launch.** Verified myself, not
  taken on the panel's word. PJ-428 would have been closed on that basis. Now a real
  `DriftReport.fenced_libraries` field + `has_fenced()`, mirrored in TS, wired as a **fourth notice
  row** through six sites, `indexDrift.fencedLibraries` ×15 (parity **3,694**). NOT in
  `has_findings`: that band offers "Repair now" and the repair carries this same fence.
- 🔴 **The fourth self-satisfying check, in my own test.** `assert!(body.contains("!l.is_universe_notes"))`
  was satisfied by **pre-existing MIG-112 code at line 461**, inside an extraction window
  over-reaching ~270 lines — green with my filter deleted outright. Anchored to `body[calc..used]`
  and **proven RED**. A behavioural test with a CONTROL now backs the three source-text greps.
- 🔴 **A panel number I checked and it was UNDERSTATED.** It said 3 of 13 registries carry a drifted
  `universe_notes.path`; my first scan said **zero** (it walked only top-level directories — a search
  that could not find its target). Recursively: **7 of 17**, including a backup of the daily universe
  with 18 libraries whose recorded root names the LIVE directory. `own_root` now comes from
  `active_universe_dir` at both sites.
- `mig108::norm_under` replaces a hand-rolled comparison that dropped **NFC** — live on Arabic names.
- Both refusal strings rewritten: mine had 18–22 literal spaces mid-sentence and named two controls
  that do not exist ("New Library", "Unregister"; the app says "Bring In a Library", "Remove").
- A **liveness check** before the refusal — `active_path` is never cleared and can outlive its
  universe. Fails open.
- **`create_universe` now carries the same refusal** (Whole-Ecosystem): it is reachable from the
  Universe Manager **while a universe is open**, while the door I had guarded is only on the
  first-run screen. I had guarded the less likely one.

**Suite 1,599 / 0.** Binary 2026-08-29 07:17, chain verified, string confirmed in the built chunks.
`ui-inspector` reviewing the three new user-facing strings.

**What is NOT fixed, and is filed below as PJ-429:** whether the declared-library exemption should
be honoured at the OTHER ~22 fence sites. Content under such a library still goes stale silently
between repairs. That is a design ruling on MIG-112's contract, not a bug fix.

**Original finding, kept for the record:**

**The contradiction.** `reconcile.rs:539-548` (MIG-112 step 3) exempts a REGISTERED library from
de-adoption when it sits inside a nested universe — *"an explicit declaration beats a filesystem
inference."* Nothing else has that exemption. `collect_md` stops at the manifest-bearing ancestor via
`is_walk_boundary`, and `run` de-overlaps roots so the library is never walked separately; the
watcher's add pass (`search.rs:13638`) drops every path in a foreign universe with a bare `continue`
and no declared-library check. So the rows are kept and never refreshed — **and because the subtree is
never walked, `walk.drifted` and `orphans` stay 0 for it, `has_findings()` is false, and the boot
notice reports a clean launch.** The one signal designed to surface staleness structurally cannot see
it.

**Reachable in one deliberate action, no legacy state required.** `universe::link_library_as_universe`
(`universe.rs:1458`) writes `.constellation/universe.json` into ANY directory, with no guard against
that directory being an ancestor of a registered library. `add_library`'s new MIG-112 guards refuse
only at REGISTRATION time and cannot prevent that ordering.

**The half that cannot be undone.** Pass 2 (deletes) of `reindex_changed_paths` has **no**
foreign-universe fence at all. An external rename or move inside such a library PURGES the row —
taking `note_links.weight`, `traversal_count`, `last_traversed`, `confidence`, `status`,
`note_meta.review_priority` and `review_schedule` with it — while Pass 1 refuses to index the new
path. CLAUDE.md records `search.db` as the ONLY system of record for that earned data. **No walk can
regenerate it.**

**LATENT — measured, not assumed (2026-08-27).** Across all 8 universes carrying a `libraries.json`,
**52 library entries, zero** meet the precondition: no registered library has an ancestor between it
and its universe root carrying a manifest in either form. Checked the one library that sits OUTSIDE
its root as well (`Constellation Test → Ideaverse Pro 2.5` at `E:/Ideaverse Pro 2.5`, a pre-MIG-108
external) — no manifest anywhere in its chain either. **Nothing is degrading while this waits.**

**One claim of the finding was REFUTED during its own verification, and the correction is kept here so
nobody re-forms it:** the candidate said the subtree is frozen *indefinitely*. It is not.
`reconcile_filesystem` starts `index_library_recursive` AT `lib.path`, and `is_walk_boundary` is
applied only to child entries during descent — the helper's own doc says never to call it on the
walk's own start root. So a **user-triggered Repair / Full re-read DOES reach and heal the content
staleness.** That does not dissolve the finding: the repair is never offered, because the drift report
cannot see the subtree; and the purged earned data is beyond any walk's reach.

**Why not fixed in this pass** (WA#6 — flagged to the Boss, not silently parked): it is not in this
diff; the fix changes the MIG-112 fence he validated yesterday; and it spans `reconcile.rs`,
`search.rs` Pass 1 AND Pass 2, and the watcher — cross-subsystem, so `/migration`-shaped. Fix it with
**PJ-419** (bare `reindex_single_note` across a universe switch), which lives in the same fence
contract.

---

## 📋 Filed 2026-08-27 — the 16 / 8 / 0 gap (was "D1", the decision held back after MIG-112)

*Not sent to the Boss as a ruling request. SO#10 forbids asking against an unreconciled record, and
there is nothing here he needs to decide: no `.md` file was touched, the notes live in their own
universes, and all 16 removals are in the app's own log. He is told what happened; the fix is filed.*

**The verified three-way state.** `Eisa Universe/.constellation/diagnostics.log:2072` records
`MIG-112: de-adopted 16 row(s)`. `note-history.jsonl` holds exactly **8** records with
`"reason":"foreign_universe"` — what Settings → Deleted notes can show. The same log carries **8**
`delete archive SKIPPED … no cid_cn` lines at the two timestamps immediately preceding it
(`1787754513`/`1787754514`, log lines 2064–2071): four under `كون عيسى 2` and four under
`كون عيسى 3`, the same four note names in each — which is the duplication that made the identity
blank in the first place. 8 + 8 = 16, exactly.

**A grep for that phrase returns 9, not 8.** The ninth (line 1997, `Archive Probe.md`, timestamp
`1787649416`) is an earlier unrelated event. Counting it would misattribute one skip to MIG-112 and
break the 8 + 8 = 16 arithmetic — check the timestamps, not the total.

`DriftReport` has no `de_adopted` field at all.

### 🆕 PJ-426 *(MED — Group 2 — incomplete record)* — the delete archive cannot record a row whose `cid_cn` is empty

`search.rs:~13140` returns `Vec::new()` when `cid_cn` is blank, so the removal happens with no
entry. The decision to key the archive on the universe-relative path instead was already taken and
written at `search.rs:13149–13153` — *"changes shared delete semantics and is filed as its own job
rather than smuggled in here."* This is that job. Blocks nothing; unblocks the honesty of every
count that surface shows.

### 🆕 PJ-427 *(MED — Group 2 — a clean-up that does not announce itself)* — the de-adoption count reaches no user surface

MIG-112's own comment names *"no drift notice and no repair receipt"* as the defect it exists to
end, and then ends it only in the log. One field on `DriftReport` plus one sentence ×15. Do it with
PJ-426 so the surface and the record land together.


> **What changed in v2.04** (**MIG-112 BUILT and GATE-CLEARED — awaiting the Boss's Stage-1 test.
> CODE DELIBERATELY UNCOMMITTED. Ten defects of mine caught; the last two were false statements to
> him, not code.**):
>
> **► NEXT ACTION: the Boss runs the Stage-1 test.** Commit only on his pass.
>
> ### 🔴 I told him the binary was current when it was an hour stale
> Reported "binary rebuilt 16:53 after every source change" — true when written, **repeated after
> adding two guards** at 17:04/17:05. Caught by the final panel. Rebuilt **18:17**, mtimes
> re-verified. This is what `feedback_verify_binary_before_testing.md` exists to prevent.
>
> ### 🔴 A FALSE CAUSE in a shipped string — one day after the law written for that shape
> `settings.deleted.intro` claimed a note leaves no entry because Constellation *"never gave it an
> identity of its own."* **False for all 8 of the 16** — their files carry a real `cid_cn`; the index
> row is blank because a **duplicate claimed it first**. Corrected ×15, cause-neutral, naming both
> causes. Parity 3,692 ×15; confirmed in the built bundle.
>
> ### 🔴 The test was UNOBSERVABLE — a design flaw, not wording
> The folders leave the tree via a per-read filesystem check, so under the new binary they are absent
> from the **first render in every ordering**. There is no sequence in which he can watch them go; the
> test had him verifying an unanchored negative. Fixed with **Step 0 — capture the before-state on his
> CURRENT build, before installing.** `ui-inspector` APPROVED after verifying HEAD has no manifest arm
> and no caching layer exists.
>
> ### ❌ I misread the instrument I had just corrected the panel for misreading
> `boot-perf.history.jsonl` writes **one record per PHASE**: 1,002 records, **502 boots**. My "last ten
> boots" was **five**. Deduped: daily universe **3,391 ms median, 3 of 10 FAIL** (I reported 6,275 /
> 6 of 10). `Eisa Universe` 19.9 s, **10 of 10 FAIL — pre-existing, so boot time measured there proves
> nothing.** Also: the register claimed `svelte-check` 0 without anyone re-running it after the
> frontend edits. Run: **0 errors.**
>
> ### 🆕 PJ-419 — bare `reindex_single_note` across a universe switch (MED, PRE-EXISTING, FILED)
> `reconcile.rs:744`/`:840` pass generation `None` where MIG-111 B1 converted equivalent sites. A
> switch in a microsecond window during boot reconcile writes ONE note from the departed universe into
> the arriving one — silent, durable, non-self-healing. **I called it "four sites"; the panel counted
> ELEVEN** across 8 files (`bases.rs`, `index_repair.rs`, `libraries.rs`×2, `reconcile.rs`×2,
> `search.rs`×3, `shape.rs`, `tasks.rs`, `universe.rs`). Filed as ONE entry over the whole concern per
> the 2026-08-25 precedent — landing 2 of 11 would be the Whole-Ecosystem Fix Law's canonical
> violation. **Reopen: the next build that touches `reconcile.rs` for any reason.**
> Same entry: **`add_child_universe` (`universe.rs:1590`) has no nesting guard**, and `add_library`'s
> new refusal message points straight at it. Not destructive (verified); the second door of the room
> MIG-112 just fenced.
>
> ### 📌 Recorded, not filed
> **R4** — `Eisa Cognitive Knowledge/.constellation/mig108-backup/universe.json` returns true for
> `MustLookLikeOne`; inert (0 `.md` beneath it), but the honest sentence is "no nested manifest **in
> content space**." **LOW** — `add_library`'s `PresenceIsEnough` refuses a folder holding any stray
> `universe.json` as "a universe of its own" while the walk fence calls it ordinary; soften or switch
> to `MustLookLikeOne`. **Second live de-adopt site**: `CE Test Universe/CE Test`.
> **REFUTED, so nobody re-forms it**: the 9 rows pointing into his daily universe are NOT purged —
> they `continue` at `reconcile.rs:463` before the MIG-112 arm. **16 is the right number.**
>
> ### ⚖️ ONE DECISION owed to him AFTER the test (D1)
> `de_adopted` reaches only a `diag()` line. The true state is three-way: **the log has all 16, the
> delete record has 8, the drift notice has none** — and the pass's own comment names "no drift notice
> and no repair receipt" as the defect it exists to end. One line to surface it, one word back from
> him. A once-per-universe event may not earn a permanent notice; that is his taste, not ours.

---

> **What changed in v2.03** (**MIG-112 BUILT — Boss-approved, steps 1–8, awaiting his test. NOT
> COMMITTED. Seven defects of mine caught by the gates; twelve pre-existing inspection findings
> filed as PJ-407…PJ-418.**):
>
> **► NEXT ACTION: the Boss tests the MIG-112 build.** Test material goes `tutorial-auditor` →
> `ui-inspector` → panel first. Commit only on his pass.
>
> ### ✅ PJ-403 (MIG-112) BUILT — steps 1–8
> One shared rule — *a directory carrying a universe manifest is a boundary* — reached three ways:
> the predicate 13 walkers already call (`is_nested_library` → **`is_walk_boundary`**, extended), a
> direct check for the ~12 walkers that had **no** boundary at all, and **`path_is_in_foreign_universe`**
> for row-driven writers no folder fence can stop. **Three hand-inlined COPIES** of the exclude test
> (indexer, sidebar tree, Move picker) were routed through the shared predicate — the exact drift the
> Whole-Ecosystem Fix Law names. Step 8 de-adopts the 16 rows via a new
> **`DeleteReason::ForeignUniverse`**; **no `.md` is deleted or touched**.
>
> ### 🔴 SEVEN defects of mine, all caught by a gate — none by re-reading my own work
> 1. **APP-KILLER**: the fence sat in the `.md` arm, so Windows' single DIRECTORY event bypassed it and
>    re-adopted the whole nested subtree — *my fix silently undone by the next external touch, in the
>    build that added it.* 2. **Silent-hiding**: a stray file named `universe.json` in any note folder
>    would have made it invisible to ~25 surfaces with **nothing logged**. 3. **Opposite postures**:
>    MIG-108 RELOCATES files, so its doubt must mean "universe"; a walk fence needs the reverse — now a
>    **required** argument. 4. Borrow error. 5. **BOOT REGRESSION** (Rule 8) — the §8 check walked every
>    note's ancestors to the volume root; now bounded at our own root and memoised per directory.
>    6. **DEAD CODE — step 8 never ran**: it sat below the clean-drift early return, and the state it
>    fixes is exactly the state that takes that return. *It would have shipped looking correct, passing
>    every test, and doing nothing.* 7. **A test that could see itself** — matched its own source line
>    and reported working code broken.
>
> ### 📌 One correction that changed a shipped test rather than the code
> MIG-112 made a PJ-207 §8 assertion obsolete (roots-narrowing alone now IS enough for a universe
> ROOT). The assertion was relaxed 1 → 0 **and `the_exclusion_set_is_still_load_bearing_for_a_nested_library`
> was added in the same change** — flipping the number alone would have let someone delete the
> exclusion set entirely and still see green. A library is not a universe, so the manifest check
> cannot cover it.
>
> ### 🧪 Verification
> `cargo test --lib` **1,584 / 0 failed** (+12) · `svelte-check` 0 · `i18n-parity` 15/15 · binary
> rebuilt after the dead-code fix. Placement guard proven **RED→GREEN**. **Live probe** of the
> shipping predicate against his real folders: fires on the three `كون عيسى` copies, does **not** fire
> on `Constellation PKM` / `موسوعة عيسى` / `3mooR` / `Eisa Test`.
> **NOT verified, and owed to his round rather than asserted:** boot time before/after, and that the
> 16 rows actually leave with nothing else leaving.
>
> ### 🆕 FILED: PJ-407 … PJ-418 — the twelve PRE-EXISTING safety-inspection findings
> Filed, not folded in, per the 2026-08-25 precedent ("all are new in this diff, so none was filable").
> **PJ-407** (HIGH) importers never index what they import — 400 imported notes exist on disk with no
> `note_meta` row, invisible to search/backlinks, and an import larger than reconcile's re-adopt cap is
> never recovered. **PJ-408** `reconcile.rs:601` `let _ =` swallows a reachable Err, leaving a row at
> the right path with the OLD name and body. **PJ-409** `sanitize_filename` byte-truncates UTF-8 and
> **panics** mid-import. **PJ-410** `SenseMakingCanvas` clears its dirty flag AFTER the await, dropping
> edits made in flight from the sole-copy `.canvas`. **PJ-411** provenance matches only the legacy
> wikilink form, so chains written by the app's own autocomplete are invisible. **PJ-412** `mig108`
> returns early after the DB commit, leaving sky triggers dropped and the libraries cache stale.
> **PJ-413** `constellation_map_universe` returns an empty **Ok** tree when the DB is closed.
> **PJ-414** universe activation silently repoints a missing library to a same-named folder and
> rewrites `libraries.json` with no backup. **PJ-415** strata reads the ACTIVE universe's alias map for
> a Linked Universe's library. **PJ-416** a canvas named with a leading dot is created, returned Ok,
> then permanently invisible. **PJ-417** `openCanvas`'s bare `catch {}`. **PJ-418** — the second
> inspection pass's own finding, closed in-pass (defect 6 above), recorded so the pass is not silent.

---

> **What changed in v2.02** (**"Fix the duplication across other universes" — MIG-112 investigated;
> the GENERATOR found in the app's own journal and already closed; PJ-403 … PJ-406 filed; one panel
> finding DISSOLVED as the shadow trap. Nothing built, nothing written to the Boss's data.**):
>
> **► NEXT ACTION: the Boss approves or amends the MIG-112 eight-step plan** (Migration Rule Phase 2).
> Then his one-word calls on the four data questions. **SO#10 satisfied:** this version, session log
> §12, MoCh-2026-08-25-2000 and orientation v4.19 land BEFORE the approval request.
>
> ### 🔴 THE GENERATOR — Constellation copied a WHOLE UNIVERSE into another universe's root
> `Eisa Universe`'s own `mig108-journal.json` records it verbatim: `library_name "كون عيسى"`,
> `old_path E:\Constellation Universes\كون عيسى`, `new_path …\Eisa Universe\كون عيسى 3`,
> `"action": "copy"`. The MIG-108 "One Universe, One Location" unification treated another universe's
> root as a library — because a universe's `universe_notes` library has `path == the universe root`.
> **ALREADY CLOSED**: `mig108.rs:192` now consults `universe_manifest_at_or_above` **ahead of the Copy
> arm** (`227f5b3a`, 2026-08-21; strengthened `a088226b`, 2026-08-22). The copies date from
> **2026-08-07/08** — before the fix. **Generator fixed · debris remains · downstream blindness remains.**
>
> ### 📊 The duplication, measured whole-ecosystem
> 20,945 `.md` under `E:\Constellation Universes` (**including `.trash`**; excluding hidden folders
> gives 20,259 — always state the criterion). **8,006 identities claimed by more than one file**
> across 16,073 files. But **almost all of it is inert**: 7,778 are `MIG108 Rehearsal` (a 4.6 GB
> standalone copy of ECK, federated to nothing) and 182 are byte-identical photography notes in two
> universes that never collide. **Duplication a single index can see: 6 in `Eisa Universe`, 11 in
> `Eisa Cognitive Knowledge` (`3mooR\`), 1 in the rehearsal.**
>
> ### ✅ Severity corrected DOWN from my own brief
> Three of the four "renames your files" commands (`canonicalize_execute`, `de_canonicalize_library`,
> `inject_cid_library`) have **no caller anywhere in `src/`**. The boot repair IS live and does rename
> — but fires only on a canonical-named `.md`, and **none exists inside any nested universe** (all
> 20,945 scanned). **The rename cascade is the only genuinely live path**, realised reach today:
> **three lines in three files.** **Nothing has been destroyed** — all four copies compared byte by
> byte; 13 of the 16 adopted notes are byte-identical to the linked universe's, the other 3 differ
> only in their identity line.
>
> ### 🆕 FILED: PJ-403 … PJ-406
> **PJ-403** MIG-112 itself — the nested-universe boundary, ~30 surfaces closing through two shared
> switches. **PJ-404** the first MIG-108 run **dropped `موسوعة عيسى` from the registry** (backups:
> 6 entries at 16:53, 5 at 16:54 on 2026-08-07); **607 rows carry an unregistered library name**.
> **PJ-405** index-driven writers (identity back-fill / healing) iterate rows and write to whatever
> file a row names — **a folder boundary cannot stop them**. **PJ-406** the write journal holds **no
> record of any write into the three copies, ever**, though three files there were rewritten on
> 2026-08-08 18:05:50 with different identities — writer NOT established, NOT attributed.
>
> ### 🚫 DISSOLVED — not filed: the master-registry "anomaly" is PJ-321's shadow trap, 3rd occurrence
> The panel reported the universe registry as stale and unexplained, and leaned a claim on it.
> `fsutil hardlink list` on `%APPDATA%\world.uconstellation.app\universes.json` resolves to
> `…\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\…` — the same frozen 277 bytes that cost this
> project a Group-1 entry. **Checked before filing; the dependent claim is withdrawn.**
>
> ### ❌ CORRECTION — PJ-387's archive fix is NOT a prerequisite for MIG-112
> I briefed it as one. Wrong twice: a durable per-note line IS written to `diagnostics.log` for every
> skipped note, and **the 8 blank rows are not identity-less** — their files carry real identities
> (they are blank in the index because a duplicate claimed the identity first). The de-index step
> reads the identity off the file and archives properly. PJ-387's §0 remains worth doing **on its own
> merits** (234 blank rows, 4 carrying change history) and `build_delete_archive`'s own comment —
> that changing it "changes shared delete semantics and is filed as its own job" — is obeyed.

---

> **What changed in v2.01** (**PJ-387 INVESTIGATED — the 13 are explained; the ledger's own remedy
> for 10 of them was wrong; seven new entries filed. Nothing built, nothing written to the Boss's
> data.**):
>
> **► NEXT ACTION: the Boss rules on PJ-387** — the order of work, the two copied universe folders,
> and options A/B/C for the 221. **SO#10 satisfied: this version, the session log, the MoCh and
> orientation v4.18 all land BEFORE the ruling request.**
>
> ### ✅ "Why it happened" is no longer unknown — and the app had been logging it all along
> PJ-387 recorded the 13-row divergence as *"NOT KNOWN and must be investigated before anything is
> changed."* It is now established, with **two distinct causes**:
>
> **TEN of the 13 are a deliberate refusal.** `search.rs:8497-8536` — on a `cid_cn` UNIQUE violation
> whose owner file is still on disk, the indexer refuses to steal the identity and writes the `''`
> sentinel, logging every occurrence. `diagnostics.log` holds **845 such lines naming exactly 10
> distinct paths and 6 distinct cids**, latest **2026-08-25 12:36:27Z**. Verified per path by a
> second, log-independent method: 10/10, every owner live. Source of the duplicates: `كون عيسى 2\`
> and `كون عيسى 3\` are folder copies of a **whole universe** (each with its own `.constellation\`,
> `search.db` and `universe.json`) sitting inside `Eisa Universe`'s root.
>
> **THREE of the 13 carry two stacked frontmatter blocks** — a first block with the legacy `cid:`,
> then a second with `cid_cn`. `split_frontmatter` reads only to the first closing fence, so the real
> identity is body text. `ensure_cid_cn` then early-returns **forever**, because its guard
> (`canonical.rs:1451`) scans the whole file and finds `cid_cn` in the body. Their author is **NOT
> established and is NOT attributed** — no code path at HEAD or at `b19908c1` can produce the shape,
> and no commit touched `canonical.rs` between 2026-04-14 and 2026-06-11.
>
> ### ❌ CORRECTION TO THIS LEDGER'S OWN ENTRY — "reindex the 13" is wrong for 10 of them
> PJ-387 listed *"reindex the 13 (an index bug — fix regardless)"* as a remedy. **It cannot work for
> the ten.** Re-indexing re-enters the same collision arm; each path has already been re-attempted
> **81-85 times across 46-54 distinct log timestamps** with the same outcome. The entry is rewritten
> below rather than amended.
>
> ### ❌ CORRECTION — the "give them identities anyway" reasoning is void
> A load-bearing fact was **the opposite of what was believed**. `ensure_cid_cn` writes **0 of the
> 234** on open — proven by running the shipping function against copies of all 234 (**234/234: no
> disk write, no content change**). 232 trip its whole-file `\ncid_cn:` guard because the orientation
> documents contain a worked YAML example; the 2 `CANONICAL` files take the legacy branch, where
> `migrate_cid_to_cid_cn` returns early for want of a leading fence. **A note is permanently denied an
> identity because it explains how identities work.** The population is therefore **stable** — "leave
> them" is durable, not a slow drift.
>
> ### 🆕 FILED THIS CLOSE: PJ-396 … PJ-402
> PJ-396 (**a LIVE frontend path at HEAD that prepends a second frontmatter block — reproduced**) ·
> PJ-397 (34 files already stacked; 12 with the block filed as body; the invisible `tags: idea`) ·
> PJ-398 (1,204 notes lose block-list properties that have no dedicated column) · PJ-399
> (`write_gate`'s attestation is blind to exactly this corruption) · PJ-400 (cid minting: 8 sites, no
> identity-uniqueness check) · PJ-401 (the on-open identity write is undoable-proof and undocumented
> in all 15 languages) · PJ-402 (`last_traversed == created` on every link row).
>
> ### 📌 Ranking
> **PJ-387 stays MED / Group 2.** Measured across both live universes, the affected rows carry **no
> earned link data** (`weight <> 1.0` → 0; `traversal_count > 0` → 0; all `confidence = 'hypothesis'`),
> **no review priority**, and their only `note_state_history` is the app's own identity churn. A real
> hole in a promise, over a set that currently holds nothing irreplaceable. **Its escalation trigger
> is replaced**: "when PJ-386 is approved" is blocked on another job and may never fire; use instead
> *any affected note acquiring earned link data, review history or state history on a live
> non-template path*, or *the archive beginning to carry the note's real text*.
> **PJ-396 enters at Group 1** — it is a live generator of the very corruption PJ-387 documents.

---

> **What changed in v2.00** (**PJ-385 and PJ-369 CLOSED — Boss-tested, Boss-authorised, executed
> and verified against a pre-run snapshot; commit `35a9921d`**):
>
> **► NEXT ACTION: PJ-387 — the 234 notes with no identity**, the Boss-ordered job, and the one
> whose 13-row index divergence is unexplained. It escalates to Group 1 the moment PJ-386 is
> approved, and PJ-386 is blocked on PJ-388. Then PJ-378 triage (58 sweep findings).
>
> ### ✅ PJ-385 CLOSED — the delete record is readable, and the Boss read it
> Stage-1 test **PASSED**. He confirmed the universe, created an `Archive Probe` note, deleted it,
> found it at the top of the record, and opened it. Two questions, both correct behaviour:
> *"why is the amber strip still here?"* (it reports a count; nothing had been removed yet;
> `indexPhantomDismissed` is in-session state) and — **the finding, observed by him unprompted** —
> *"where are the square brackets on the link line?"*
>
> **That question is PJ-388 reproduced on his own note by the person who designed the app.** His
> three-line probe archived as `  My headline` / `this is  important` / `test1-fiction-brief`: the
> `#` gone leaving its space, the `**` gone leaving a double space, the `[[ ]]` gone with the target
> surviving as bare words. A prediction that could have failed did not — he was told in advance the
> record held exactly five entries, named; his screen read **"6 deletions recorded"**, those five
> plus his probe.
>
> **Fixed in-pass (WA#6):** he read the caption and *still had to ask*. "Strips markup" did not read
> as "the brackets around your links". The caption now NAMES `#`, `*`, `[[ ]]` and Arabic
> diacritics, and says what a link pointed at survives while the brackets do not. ×15 locales.
>
> ### ✅ PJ-369 CLOSED — the 603 removed, and verified against the snapshot rather than the receipt
> He pressed **Remove**. Receipt: *"Removed 603 entries."* The control vanished (it renders only
> while `phantomCount > 0`) and the amber notice with it.
>
> | table | before | after | delta |
> |---|---:|---:|---:|
> | `note_meta` | 2,731 | 2,128 | **−603** |
> | `note_links` | 31,368 | 11,896 | **−19,472** |
> | `sky_links` | 31,361 | 11,889 | −19,472 |
> | `sky_nodes` · `note_body` · `review_schedule` | 2,731 | 2,128 | −603 |
> | `note_summaries` | 753 | 254 | −499 |
> | `note_aliases` | 157 | 30 | −127 |
>
> **Every figure was forecast before the run.** Full path-set comparison: **603 removed, 0 added,
> 0 removed that were not on the list.** Archive: 5 del-envelopes → 609; **603 new
> `phantom_prune` envelopes matching the list exactly**, 20,484,230 characters, **2 with no text**
> — the two frontmatter-only notes predicted. Zero unparseable lines. The 604th new envelope is his
> own `Archive Probe`. **Zero orphans left** in `note_embeddings`, `note_body`, `note_links`,
> `note_summaries`, `review_schedule` or `note_aliases`. `Eisa Cognitive Knowledge` untouched
> (8,031). `integrity_check` = ok. FTS in step (2,128 = 2,128). **No `.md` file touched.**
>
> **Backup retained:** `E:\Backups\Constellation\EisaUniverse-preprune-20260825` — 313 MB,
> integrity-verified before the run. It is the only rollback; the record is not one.
>
> **PJ-392 does not apply to this path** — `note_body` was purged with the rest, 0 orphans. Its ECK
> observation stands unre-derived.
>
> ### 🆕 PJ-395 filed — 4,062 orphaned embeddings, PROVEN pre-existing
> Identical count in the snapshot and the live database — the check that could have disagreed and
> did not. Not caused by the prune, not cleaned by it. **Do not delete them until the producer is
> found.**
>
> ### 📌 Where the other new entries stand
> PJ-387 (Boss-ordered, next) · PJ-388 (blocks PJ-386; first deliverable is the envelope fixture
> that cannot currently fail) · PJ-389 · PJ-390 (**partly overtaken**: the consent string now
> discloses the connections; what remains is the RECEIPT, which still reports index rows only —
> the Boss saw "Removed 603 entries" and not "and 19,472 connections") · PJ-391 · PJ-392 · PJ-393 ·
> PJ-394 · PJ-395.
>
> ### ✅ Closed-by-evidence this version
> **PJ-321** — an observation artifact, not an app defect (see v1.99). **PJ-384** — the mechanism
> stands; its population is now PJ-387.

---

> **What changed in v1.99** (**the archive is not the note — six findings filed, five inspection
> findings fixed, and a Group-1 entry closed as an observation artifact**):
>
> **► NEXT ACTION: the Boss tests PJ-385 (Deleted notes), then rules on the 603 removal.** The
> ruling request carries PJ-390's collateral figures and PJ-388's finding — not a bare yes/no.
> Then PJ-387, then PJ-378 triage.
>
> **⚠️ SO#9 SELF-CORRECTION, second commission of the same defect.** The 2026-08-25 entries —
> including PJ-385 and PJ-386 — were being written into `v1.98.md` **in place**, under a preamble
> belonging to v1.97, which is exactly the defect v1.98's own preamble confesses about `ad08c542`.
> Caught by the panel at this close. `v1.98.md` has been reverted to its committed state and those
> entries now live here, under their own preamble. Stated in the open rather than quietly fixed,
> because the ledger's value is that its trail is honest.
>
> ### 🚨 PJ-321 CLOSED — the app was never at fault; I was reading a shadow copy
> The Group-1 entry *"the universe registry is not tracking reality"* accumulated **five
> corroborations**, including a controlled experiment the Boss performed himself (creating two
> universes through the Universe Manager and watching the registry file stay byte-identical).
> **All five are one artifact.** `%APPDATA%\world.uconstellation.app\universes.json`, read from a
> Claude session, resolves to a frozen copy inside the Claude Desktop MSIX container:
>
> ```
> fsutil hardlink list "C:\Users\ealsh\AppData\Roaming\world.uconstellation.app\universes.json"
>   \Users\ealsh\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\...
> ```
>
> The three sibling files in that directory pass through to the real location; only this one is
> shadowed. The app is proven correct by its own boot trace: `Eisa Universe`'s
> `boot-perf.latest.json` logs one `set_active_universe` 16 ms after process start, and that
> command hard-fails when the id is absent from the registry (`universe.rs:1026`) then writes
> `active_id` (`:1225-1226`). **The registry held the entry and was written that day.**
>
> **The contamination reached the repo.** `lab/reports/pj321-evidence-snapshot-2026-08-22/universes.json`
> — force-added as PJ-321's durable evidence — is a copy of the shadow; the committed blob, the
> working tree and the live container file all hash to `c20f9694…`. The entry at `:926` records
> that invariance **as the finding**. A `READ-THIS-FIRST.md` now sits beside it. The bundle's two
> other files came from `E:\` and are genuine.
>
> **The shape:** *a cross-check that could not disagree* — the fourth failure mode named in the
> 2026-08-25 law, in its purest form. Five reads of the same frozen 277 bytes through the same
> redirected path, their agreement recorded as mounting evidence. The discriminating command took
> one line and was never run. **Standing rule:** any Constellation file read under `%APPDATA%`
> from a Claude session must be `fsutil hardlink list`-checked before its contents are treated as
> fact. Added to the `findings-verifier` brief. Full account: `SESSION-LOG-2026-08-24.md` §21.
>
> **Two genuine ordering defects surfaced while settling it** (neither is the premise):
> `set_active_universe` flips `active_path` and takes the OS owner lock **before** it saves the
> durable intent, with no rollback — a failed `save_registry` returns `Err` after the process has
> already switched, leaving the window rendering universe A while every Rust command targets
> universe B, and the boot loop then calls the command again for the next entry. Filed below as
> part of PJ-393. `remove_universe_from_registry` never clears `active_path` — **already filed
> under PJ-322**; this is an independent second observation, not a new entry.
>
> ### 📏 THE ARCHIVE KEEPS A SEARCH RENDERING, NOT THE NOTE — measured
> `note_meta.body_text` is `parse_frontmatter` (drops the whole YAML block) → `strip_markdown`
> (`search.rs:8234`) → `normalize_arabic_for_search` (`:8274`). That column is the ONLY text a
> delete archives. Proven on real shipping output, not a re-implementation: the live envelope for
> `Vindar.md`, whose recovered file reads `[[supports::Quazzle Renamed|it explains why]]`,
> archived `supports::Quazzle Renamed` — **the annotation, one of the eight Living-Link
> properties, is gone.** Three sibling files of 148–166 bytes archived as `''`. Worst case
> measured: 8,037 characters → 713. Headings, emphasis, code-block contents, link URLs and Arabic
> diacritics are lost universally; markdown tables survive; 7–11% of ordinary short notes do
> round-trip byte-exactly. → **PJ-388**, which **blocks PJ-386**.
>
> ### ✅ FIVE SAFETY-INSPECTION FINDINGS, ALL FIXED BEFORE COMMIT (WA#6)
> Diff-scoped over the 11 changed files. **HIGH ×2, one root:** `read_lines` (`link_life.rs:445`)
> mapped EVERY `read_to_string` error to an empty Vec, so an archive that exists and cannot be
> decoded — a tear inside a multi-byte UTF-8 sequence, an antivirus sharing violation, an
> unhydrated cloud placeholder — reached the user as *"The record exists and is empty"*, asserted
> as fact about the last surviving record of destroyed notes. `unreadableLines` could not rescue
> it: that banner renders only when `total > 0`. Fixed by distinguishing `NotFound` from every
> other error in the one shared reader; both archive commands now refuse. **Mutation-tested** —
> removing the fix turns the new test red. **MED:** `lastPruneReceipt` was the one per-universe
> surface `handleUniverseSwitch` did not clear. **LOW ×2:** three buttons bypassed the
> `closeSettings()` re-entrancy gate; the Deleted-notes list had no refresh after the prune that
> sits on the same page.
>
> ### 📌 FILED THIS CLOSE: PJ-387 … PJ-395
> PJ-387 (the Boss-ordered 234) · PJ-388 (archive fidelity, blocks PJ-386) · PJ-389 (three
> hand-mirrored earned-data predicates) · PJ-390 (the prune's receipt understates what it removes)
> · PJ-391 (the Cataloger's dead typed-neighbour signal) · PJ-392 (orphan `note_body` rows) ·
> PJ-393 (`set_active_universe` saves the durable intent last) · PJ-394 (the 14 translated User
> Manuals are far behind English) · PJ-395 (4,062 orphaned embeddings, pre-existing).

---

## 🆕 Filed 2026-08-25

### 🚨 PJ-403 *(HIGH — Group 1 — MIG-112)* — a universe is never content of another universe: ~30 walkers cannot tell a nested universe root from a folder
**Boss-ordered 2026-08-25:** *"I noticed the duplication across other universes. Fix it."*

**Concept (the horse):** *a universe is a peer, never content — so no walk of one may enter another.*

**The generator, from the app's own journal (`mig108-journal.json`, `Eisa Universe`):**
`library_name "كون عيسى"` · `old_path E:\Constellation Universes\كون عيسى` ·
`new_path …\Eisa Universe\كون عيسى 3` · `"action": "copy"`. The MIG-108 unification treated another
universe's ROOT as a library, because `universe_notes.path == the universe root`. **CLOSED**
`227f5b3a` (2026-08-21) / `a088226b` (2026-08-22): `mig108.rs:192` consults
`universe_manifest_at_or_above` ahead of the Copy arm. Copies date from 2026-08-07/08.

**The blindness that remains.** `resolve_libraries_recursive` (`universe.rs:601`) builds library
identity from the **registry only** and never lists directories, so an undeclared nested universe
contributes nothing to `nested_library_paths` / `foreign_roots_of` / `walk_exclusions`.
`require_own_library_in` confesses it at `libraries.rs:322-323`. Verified working for **declared**
linked universes: `Eisa Universe` holds **0** rows for the federated top-level `كون عيسى`.
**Live effect: 16 notes from three nested roots indexed as `Eisa Universe`'s own; 8 blank-identity.**

**The primitive already exists, fail-closed, and is unreachable:** `carries_universe_manifest`
(`mig108.rs:287` — checks `.constellation/universe.json` AND the legacy root form, and treats an
unreadable manifest as PRESENT) + `universe_manifest_at_or_above` (`mig108.rs:301`, **private**).

**Scope: ~40 participating surfaces, ~30 closing through TWO shared switches.** Coverage today:
`both` at 5 sites · `nested` only at ~15 · **none at all** at ~15, including wikilink resolution
(`libraries.rs:3923`/`:3960`), strata, provenance, trails, canvas, inspector360.

**Severity, corrected DOWN.** `canonicalize_execute` / `de_canonicalize_library` /
`inject_cid_library` have **no caller in `src/`**. `repair_external_libraries_on_startup` IS live and
renames, but fires only on a canonical-named `.md` and **none exists inside any nested universe**
(20,945 scanned). **The rename cascade is the only live path** — realised reach **three lines in
three files**. **Nothing destroyed**: byte-comparison across all four copies is clean.

**Plan: eight steps** (see session log §12.9). Ordering constraints that survive: the boundary lands
as ONE commit across index + sidebar; the watcher path is fenced BEFORE the de-index; and **PJ-405
must be fenced too, because a folder boundary cannot stop a row-driven writer**.
**Ruling: an undeclared nested universe gets the wikilink fence; a DECLARED linked universe does not**
— "It is ONE universe" (2026-07-05) and Universe-of-Universes exist to make links span universes the
Boss CHOSE to federate. He never chose these three.

**NOT a fix: declaring them as Linked Universes.** Each copy's own `libraries.json` holds a **single
entry pointing at the ORIGINAL** `E:\Constellation Universes\كون عيسى`, not at itself — and that
original is already federated. Verified in all three. The app would report success and change nothing.
*(That the app accepts a "linked universe" lying INSIDE the active root is itself unguarded.)*

### 🚨 PJ-404 *(HIGH — Group 1 — registry↔index divergence — a second MIG-108 casualty)* — the unification DROPPED a registered library; 607 rows carry a library name that no longer exists
Traced from `Eisa Universe`'s own run backups:

| file | mtime | entries |
|---|---|---|
| `mig108-backup.prev/libraries.json` | 2026-08-07 16:53 | **6** — incl. `موسوعة عيسى` |
| `mig108-backup/libraries.json` | 2026-08-07 16:54 | **5** |
| `libraries.json` (live) | 2026-08-08 14:09 | **5** |

**The first MIG-108 unification run removed `موسوعة عيسى` from the registry.** Its folder is still on
disk and **589 `note_meta` rows are still labelled with it**. Two smaller cases share the shape:
**9** rows labelled `Eisa Cognitive Knowledge`, **9** labelled `PJ-065-test-book` — **607 rows total**
carrying an unregistered library name. Found by the panel; traced here. Consequences to establish
before any fix: what `library_name_for_path` now returns for those 589 paths, what the sidebar shows,
and whether `$libraryStats` counts them. **Do not "repair" the registry until the Boss rules** —
re-adding a library is a change to his universe's shape.

### 🆕 PJ-405 *(MED — Group 2 — a boundary that cannot hold)* — row-driven writers bypass every folder boundary
Every fix in PJ-403 fences **walking**. The identity back-fill and identity-healing family does not
walk — it iterates `note_meta` rows and writes to whatever file a row names. **A folder boundary
cannot stop it**, so a contaminated row is enough to write into another universe. Needs its own
per-note ownership check at the write, not at the walk. Same class as MIG-111 §0.4 R1
(`ensure_cid_cn_cmd` writing a linked universe's note on open).

**The shape that fits the one thing that demonstrably happened**, stated as a shape and attributed to
nothing: on **2026-08-08 18:05:50** three files — one inside each copy — were rewritten within 0.4 s
of each other, each receiving a **different** identity, while every other file in those folders still
carries its June timestamp.

### 🆕 PJ-406 *(MED — Group 2 — the journal has a hole)* — the write journal holds no record of writes that demonstrably happened
Constellation's write journal contains **no entry for any write into the three nested copies, ever** —
yet three files there were rewritten on 2026-08-08 (PJ-405). A journal that misses a write cannot be
used to clear a suspect, which is exactly what it was reached for here. Establish which write paths
are unjournalled before the journal is cited as evidence again. Related: **PJ-399** (`write_gate`'s
self-attestation blind to a stacking rewrite).

### 🔍 PJ-387 *(MED — Group 2 — silent-data-loss + index-divergence)* — the 234 notes with no identity — **INVESTIGATED 2026-08-25, awaiting the Boss's ruling**
**Boss-ordered 2026-08-25:** *"Regarding the 234 notes, file them as their own job."* Then:
*"Start with the 13, not the 221… Investigate it — don't theorise it."*

**Concept (the horse):** *a note the app cannot name is a note the app cannot remember destroying.*

**Measured, `Eisa Universe`, read-only against a scratch copy:** `note_meta` **2,128** rows · **234**
with `cid_cn = ''` (0 NULL) · all 234 files on disk · **13** carry a `cid_cn` key in their file,
**221** have no frontmatter fence · the 234 hold **32,625,753** characters, **65% of the universe's
body text**, and source **2,461** `note_links` (0 target them).

#### The 13 — two causes, both established

**A. TEN are a deliberate refusal (not a bug in the ordinary sense).** `search.rs:8497-8536`: on a
`note_meta.cid_cn` UNIQUE violation whose owner file still exists, the indexer will not steal the
identity — it writes `''` and logs it. **845 lines in `diagnostics.log`, exactly 10 distinct paths,
6 distinct cids**, latest 2026-08-25 12:36:27Z. Confirmed per path by a log-independent method
(10/10 owners live). Provenance: `كون عيسى 2\` and `كون عيسى 3\` are folder copies of a **whole
universe**, each with its own `.constellation\`/`search.db`/`universe.json`, sitting inside
`Eisa Universe`'s root — 8 of the 10. The other 2: a BUG-015 lab artifact, and
`Working Docs\README.md` vs `MIG-090-Plan-Notes-Navigator.md` (two *differently-named* files sharing
one cid).
**Re-index does NOT heal them** — 81-85 re-attempts each, across 46-54 log timestamps.

**B. THREE have two stacked frontmatter blocks** (`الكيماويات السامة` · `الحروف العربية` ·
`الواجهة`, all under `موسوعة عيسى\`). Block 1 carries the legacy `cid:`; block 2 carries `cid_cn`.
`split_frontmatter` (`search.rs:3835-3845`) reads only to the first closing fence → `""`. Proven three
ways (their `properties_json` holds block 1's keys; their `body_text` **begins with block 2**; they
appear **zero** times in `diagnostics.log`). **`ensure_cid_cn` can never repair them** — its guard
(`canonical.rs:1451`) scans the whole file. `note_state_history` 445/446/447 timestamp the state
change at **2026-08-08T12:46:45Z**. **Origin NOT established; NOT attributed.**

#### What breaks
`build_delete_archive` returns an empty `Vec` for an empty cid; Phase 2's whole archive-or-refuse
contract sits inside `if !archive.is_empty()` (`search.rs:12881`) and is skipped **silently**; Phase 3
purges and returns `Ok`. **`PhantomPrune` refuses** (`phantom_prune.rs:836-843`); Trash/SystemTrash
leave the `.md`. **`Permanent`, `Vanished`, `ReconcileGone` are unbounded**, and the last two are
**automatic** — all three re-`stat` and fire only when the file is already gone, which is why a naive
"refuse to purge" fix is wrong (it would leave a row for a file that does not exist).
The pinning test `a_note_without_a_cid_is_purged_but_not_archived` **never asserts the purge**.

#### Remedies — REWRITTEN (the previous three were wrong in one place and void in another)
1. **Make the archive survive an empty cid** — key it on the universe-relative path. Writes no note
   file, needs no ruling. Same commit: fix the half-blind test, and the five stale `2,731` comments.
2. **Land PJ-396 and PJ-400 before anything regenerates identities**, or the class regrows.
3. **~~Reindex the 13~~ — REFUTED for 10 of 13.** For those ten the question is the duplicates
   themselves (the two copied universe folders), which is the Boss's call, not a mechanical fix.
4. **~~Back-fill identities into the 221~~ — the "the app would do it anyway" premise is VOID**
   (0 of 234 are written on open). Options A/B/C put to him separately.

**Stays Group 2.** No earned link data, no review priority, and the only state history is the app's
own churn. **Escalation trigger replaced** (see the v2.01 preamble). **Not a duplicate of PJ-384:
that is the mechanism, this is the population.**

### 🚨 PJ-396 *(HIGH — Group 1 — silent content-integrity — LIVE GENERATOR, reproduced 2026-08-25)* — the frontend can prepend a SECOND frontmatter block, pushing a note's real properties into its body
**Two parsers disagree about where frontmatter starts.** Rust `split_frontmatter` (`search.rs:3835`)
uses `trim_start()` — its own doc says it *"tolerates leading whitespace / BOM before the opening
fence."* The frontend's **both** parsers require line 0 to be exactly `---`: `yamlDoc.ts:182-186`
(`splitFrontmatter`) and `store.ts:2573-2575` (`parseFrontmatter`). On a note whose bytes begin with a
newline before the fence, the frontend concludes `hadFence = false`, treats the **entire file** as
body, and `composeFrontmatter`'s no-fence branch (`yamlDoc.ts:449-459`) prepends a second block.

**Reproduced through the shipping entry points**, not a re-implementation: `composeUpdatedContent`
(`store.ts:2957`, the closed-note property/tag/typed-link write) and the `noteModel` compose path
(`noteModel.ts:183-189 → 549`, the open-editor save) both emitted
`---\nstage: seed\n---\n\n---\ntitle: Real Title\ntags: idea\n---\n\nThe body.` from a one-property
write. **Control:** the identical write on an ordinary note merged correctly.

**This is the frontend half of a class whose Rust half was already fixed.** `sources/mod.rs:326-347`
(`fence_offset`, PJ-207 §15) records it verbatim: *"the writers concluded the note had NO frontmatter
and PREPENDED a second one, pushing the note's real YAML down into its body… a scan of the live
universes found 28 notes with exactly this shape."* **The frontend was never swept** — a
Whole-Ecosystem Fix Law miss.

**Exposure today: 0 of 10,159 indexed notes are in the vulnerable state**; 9 files have the
leading-whitespace shape and all 9 are under `.trash`. That is why it is HIGH-not-APP-KILLER — but it
is a **live generator** of PJ-387's Cause B and of PJ-397, and one externally-created or
sync-delivered note is enough. Fix = one shared fence rule across the Rust and TypeScript parsers,
per the Whole-Ecosystem Fix Law, with the reproduction kept as the regression test.

### 🆕 PJ-397 *(MED — Group 2 — silent-degradation)* — 34 notes already carry two stacked YAML blocks; 12 have the second read as body, and one loses a tag entirely
**Measured across both live universes** (10,088 `.md` outside `.constellation`/`.trash`): **34** files
with two stacked YAML blocks — 26 `Eisa Universe`, 8 `Eisa Cognitive Knowledge` — separated from the
46 files with a benign `---` rule after frontmatter. Asking the **index itself** (*does `body_text`
begin with the second block?*), **12** have the block filed as body.

**Three lose their identity** (PJ-387 Cause B). **Nine keep it** — and were therefore **invisible to
PJ-387's own `cid_cn = ''` lens.** Their block-2 keys absent from the index include `المصدر`,
`أنشئ في`, `حُدّث في`, `المصادر`, `الاسم`, `اللغة`, `الوصف`, `تاريخ الإنشاء`. **Honest caveat:** on
re-reading the bytes, several are not losses — one is a comment block deliberately written out of
view, one an unexpanded template placeholder, one a key already indexed under a different spelling.
**Three genuine cases remain**, of which the sharpest is:

`Eisa Cognitive Knowledge\Eisa Test\تجربة الكتابة باللغة العربية.md` carries `tags: idea` in its
second block. Index `tags_json` = `[]`; **no `idea` row among `tag_counts`' 20,462**; a filesystem
sweep of all 8,070 ECK `.md` finds **0** notes with `idea` as a first-block tag. **A tag the Boss
wrote exists nowhere in the app.** (The note is a typing test — whether it matters is his call.)

**Detect and REPORT; never auto-merge.** Merging two blocks is a content decision. Blocked-by nothing,
but pointless before **PJ-396** stops new ones being made.

### 🆕 PJ-398 *(MED — Group 2 — missing capability, not damage)* — a property written as a list is stored as an empty value
`search::parse_frontmatter` (`search.rs:6842`) returns `HashMap<String,String>` — scalars only. A block
list (`key:` then `- item` lines) inserts `("key","")` and the items are skipped.

**`tags`, `sources`, `content_type` and `aliases` DO survive** in dedicated columns / `note_aliases`,
so the honest residual — list properties with **no** dedicated storage — is **1,204 notes**
(394 `Eisa Universe` + 810 `Eisa Cognitive Knowledge`), **not** the 3,244 an unfiltered count gives.
Top keys: `المجموعة` 197 · `institutions` 191 · `main_interests` 170 · `school` 143 · `field` 133 ·
`notable_ideas` 130 · `notable_works` 128 · `author` 100 · `collections` 99 · `up` 84 · `predecessor`
84 · `founder` 80 · `awards` 77 · `الوسم` 52 · `المؤلف` 49.

Not damage — a capability the index never had. Larger than PJ-387 in reach. **Measure once, properly,
as committed code before any figure is quoted again.**

### 🆕 PJ-399 *(MED — Group 2 — a guard blind to the thing it guards)* — `write_gate`'s self-attestation cannot see a frontmatter-stacking rewrite
With `expect: None`, `gate_write` reads `extract_frontmatter_cid_cn(content)` to decide whether the
content belongs to the file. A PJ-396 rewrite's **first** block has no `cid_cn`, so the verdict is
`OkUnchecked`, the write proceeds, and **no anomaly is journalled**. The one guard designed to catch
"this content does not belong to this file" is blind to a rewrite that removes the file's identity
from exactly where the guard looks.

### 🆕 PJ-400 *(MED — Group 2 — latent — identity)* — content-id minting has 8 sites and no identity-uniqueness check
`generate_canonical` (`canonical.rs:49-93`) loops 10 times, but the only test inside the loop is
whether a **file of that canonical filename** exists in one directory — dead since MIG-003 made
filenames human, and **entirely dead at the 5 of 8 sites that pass `target_dir: None`**, where it
returns on the first iteration. The suffix is 16 bits (`rand … {:04X}`, `canonical.rs:33`).
Sites: `canonical.rs:513, 587, 822, 946, 1289, 1467` · `libraries.rs:1536` · `importers.rs:871`.

The only real enforcement is the partial index `idx_note_meta_cid_cn … WHERE cid_cn != ''`, **whose
remedy is to silently blank the losing note's identity** — i.e. PJ-387 Cause A, 845 occurrences.
**Orientation §6.4 must be corrected**: "collision avoidance tries 10 hex suffixes" describes real
code but implies a protection that does not exist for `cid_cn`.

### 🆕 PJ-401 *(MED — Group 2 — undisclosed irreversible write)* — the app writes an identity into the user's `.md` on first open, cannot undo it, and has never said so
`ensure_cid_cn_cmd` is invoked from `openNoteTab` (`store.ts:3369-3371`, and again at `:3714`) and
writes through `write_gate::gate_write` (`canonical.rs:1460`, `:1470`). Exempt: templates (Boss ruling
2026-07-19), the second screen (`displayOnlyWindow`), and linked universes (`require_own_library`).

**Not undoable:** `atomic_write` → `ReplaceFileW` with `lpBackupFileName = std::ptr::null()` — the
API's own backup facility explicitly declined; the journal stores an FNV-1a hash, not content; neither
universe root is a git repo.
**Undocumented:** 313 files searched (`docs/User Manual.md` + all 15 `docs/help.*` trees, including
each language's native identity vocabulary) — **zero** passages describe the write. Two nearby
passages describe only its consequences.
Fix = one paragraph in the Properties topic and the User Manual ×15, and one line in the existing
write-integrity readout. **No dialog** — it would fire on every first open.

### 🆕 PJ-402 *(LOW — Group 3 — a cross-check that cannot disagree)* — `last_traversed` is stamped at creation on every link row
In both live universes, **every** `note_links` row has `last_traversed == created` (2,461/2,461 and
41/41 on the PJ-387 population). Anyone reading that column as evidence that a link was ever traversed
will be wrong, and the reading cannot fail. Either stop writing it at creation, or rename it to what it
is. Related: **PJ-389** (three hand-mirrored earned-data predicates).

### 🆕 PJ-388 *(MED — Group 2 — false-promise)* — the archive keeps the search index's rendering, not the note. **BLOCKS PJ-386.**
See the preamble for the measurement and the `Vindar.md` proof. Why it is a job and not a comment
fix: **it decides PJ-386.** "Should Constellation restore a note from the archive?" cannot be
answered *yes* against this source — a restored note returns without its properties, its code, its
link addresses and its link annotations. Either the archive starts carrying **raw file bytes** (a
write-path change with a real cost — **28.0 KB mean per note, measured**, not the "~35 KB" the
comment claimed), or restore is off the table and must be said so plainly.

**First deliverable: the `search.rs` envelope test fixture.** Its body is
`"the body that must survive"` — no frontmatter, no heading, no code fence, no wikilink, no Arabic
— so every transform `body_text` applies is a no-op on it and the test would stay green if the
archive lost all of them. **A cross-check that could not disagree**, guarding the exact property
it cannot see. Re-seed it with a real note and assert what actually survives.

### 🆕 PJ-389 *(LOW — Group 2 — latent — Whole-Ecosystem)* — three hand-mirrored "earned link data" predicates, none covering all eight properties
`link_row_is_preserved` (`search.rs:487`, `weight != 1.0`, excludes structural) ·
`EARNED_PREDICATE` (`link_life_backfill.rs:56`, no weight clause, excludes structural) ·
`has_earned_data` (`phantom_prune.rs:518`, **`weight > 1.0`**, does **not** exclude structural).

**Four defects, all currently masked — measured divergence: 0 rows.** `> 1.0` misses a *decayed*
earned link (CLAUDE.md's 5%-monthly decay puts one below 1.0) · the predicate is **not NULL-safe**
(`confidence != 'hypothesis'` yields NULL, read as *not earned*, in a gate whose stated law is
FAIL CLOSED) · **`annotation` is not checked at all** (12,673 of the 19,472 rows in this prune
carry one) · the prune's test schema has **no `target_path` column**, so no test can express the
scope question.

**The outgoing-only scope is CORRECT and must NOT be widened.** All eight earned properties live
on one row keyed by `source_path`; the scope protects exactly the row whose deletion would destroy
earned data, and widening it would keep dead, unopenable index rows alive to protect data that is
not on them. Land the test that pins it: *an earned incoming edge survives a prune.*

The job is ONE shared predicate — NULL-safe, structural-aware, covering all eight properties.
`search.rs:481` already records that this codebase paid for hand-mirroring once; there are three
copies now. **Ranking context: 63 earned rows across both live universes out of 266,267 —
0.024%**; the 7 in `Eisa Universe` are all machine-written PJ-065 structural stamps in a lab test
book, written inside 0.13 s by one backfill. **`Eisa Universe` contains zero user-earned links.**

### 🆕 PJ-390 *(MED — Group 2 — honesty)* — the prune tells the user what it removes from the list, not what it removes
`PruneReceipt` (`phantom_prune.rs:583`) reports index rows only. Going with the 603:
**19,472 `note_links`** (62.08% of that universe's rows, re-derived four ways including a
maintained aggregate that could have disagreed), **19,472 `sky_links`**, **603 `sky_nodes`**, plus
**3,879 `sky_links` targeting them by name left orphaned**. The archive records **none** —
`build_delete_archive` emits `del` + `nh` + `sh` and contains no `note_links` query; for these 603
the other two arms are empty, so **each phantom's archive is exactly one line**.

This close's consent-string fix puts the fact in front of the user before he acts; **this job
decides what the RECEIPT should say**, and goes through `tutorial-auditor` → `ui-inspector` →
panel like any other user-facing string.

**Unmeasured cost owed before the live run, stated as a cost and not a verdict:**
`schema_versions.incoming_links = 1`, so the delete tail is live; the 19,472 edges name 3,756
distinct targets, giving order ~19k resolves and up to ~39k correlated UPDATEs across 603 separate
write transactions, with **no safety cap** (deliberate, `phantom_prune.rs:631`). **Measure on a
copy first.**

### 🆕 PJ-391 *(MED — Group 2 — silent-degradation — a Whole-Ecosystem miss of the PJ-065 audit)* — the Cataloger's typed-neighbour signal is dead: it resolves by a column that is empty on every row
`note_links.target_path` is empty **by design** — stated at `sight_v6.rs:873`/`:1150`, where this
exact bug was already found and fixed with a regression test. Measured: **0 of 31,368 rows
non-empty.** `cece/wiring.rs:239-254` was never brought along: its `WHERE l.target_path = ?1` arm
— the "this note as target" direction — **can never match**, while the comment three lines above
says "we look up both directions"; and its `LEFT JOIN note_meta m ON m.path = l.target_path` never
matches, so `m.sources` and `m.content_type` are NULL for **every** outgoing neighbour, not "some"
as its comment claims.

Fix as a family: the correct rule is `m.path = nl.target_path OR m.name_lower = nl.target_name_lower`
(the form `sight_v6.rs:879` already ships); audit every remaining `target_path` reader in one pass
behind a shared helper.

### 🆕 PJ-392 *(LOW — Group 3 — index-divergence)* — orphan `note_body` rows survive a delete
`Eisa Cognitive Knowledge` holds **8,037 `note_body` rows against 8,031 `note_meta` rows**.
`note_body` is byte-identical to `note_meta.body_text` (verified two ways), so nothing is lost —
but the delete funnel is leaving rows in a table PJ-369 lists among those it must prune together.
**Provenance stated honestly: measured by one verifier on the ECK database; not independently
re-derived. Re-measure before acting.**

### 🆕 PJ-393 *(MED — Group 2 — half-state)* — `set_active_universe` saves the durable intent LAST, with no rollback
`universe.rs` flips `active_path` at `:1166` and takes the OS owner lock at `:1170`, but
`save_registry` is the **final** statement at `:1226`. If that write fails (an antivirus or sync
tool holding the file, a sharing violation, disk-full), the command returns `Err` **after** the
process has already switched.

**Consequences, traced:** `UniverseManager.handleSwitch` catches into `error` and does **not** call
`onSwitch()` — so the window keeps rendering universe A's tree and tabs while every Rust command
(`active_universe_dir`, index writes, `write_boot_perf_report`) targets universe B. That is exactly
the "half a switch" state the PJ-310 comment at `:1369-1387` was written to eliminate — closed at
the function's *entry*, still open at its *tail*. At boot, `+layout.svelte:3596` swallows the throw
and `continue`s to the next entry, calling the command again — moving the pointer and the lock a
second time in a loop that believes the first attempt did nothing. And the registry keeps naming
the old universe, so the next boot silently returns there.

**Fix shape:** save the registry *before* flipping `active_path` and taking the lock, or restore
the previous pointer + lock on save failure. Found while settling PJ-321; it is a real defect and
PJ-321's premise was not.

### 🆕 PJ-394 *(MED — Group 3 — documentation drift — PRE-EXISTING, measured 2026-08-25)* — the 14 translated User Manuals are far behind the English one
**Measured while doing SO#2 for this build.** English `docs/User Manual.md` is **2,687 lines**;
`docs/help.ar/User Manual.md` is **1,982**. The index-repair section that this build's changes
belong in exists in **3 of 14** translations (de, fr, ar) and is absent from the other eleven —
so the corrected description of what the delete record keeps could only be written in English.

**Distinct from PJ-336**, which measured the *help topics* (43 English, 21 translated). This is the
**User Manual** itself, and it is the file CLAUDE.md's SO#2 names first.

**Why it matters more than it looks:** SO#2 requires every user-facing change to reach all 15
languages. When the target section does not exist in a translation, the standing order cannot be
satisfied by an edit — it needs the section written — and the honest outcome each time is an
English-only update that quietly widens the gap. Left alone, SO#2 becomes a rule that reports
success while drifting.

**Owed:** measure the true per-locale delta (sections present/absent, not line counts), then decide
whether to back-fill wholesale or to gate future SO#2 work on the section existing. **Do not
back-fill by machine translation of a section whose English wording is still moving** — this
build's own text changed materially twice in one day.

### 🆕 PJ-395 *(MED — Group 2 — index-divergence — PRE-EXISTING, measured 2026-08-25)* — 4,062 orphaned embeddings in `Eisa Universe`
**Measured on both the pre-prune snapshot and the live database: 4,062 in each — identical.** The
PJ-369 removal neither created them nor cleaned them; they predate it. Of 5,161
`note_embeddings` rows, only **1,099** have a `note_meta` row. The other 4,062 point at paths the
index no longer knows.

**Why this is not cosmetic**, in the codebase's own words: the delete funnel purges
`note_embeddings` precisely because *"the orphan embedding keeps surfacing in semantic search"*
(`search.rs`, PJ-140 #17). 4,062 of them are doing that now.

**What produced them is NOT known** and must be established before anything is deleted — candidates
include deletes that predate the PJ-140 purge, and rename/move paths that rewrite `note_meta` but
not `note_embeddings`. **Do not "clean up" by deleting orphans until the producer is found**, or
the same population regrows silently.

**Evidence the prune is not implicated** (a check that could have disagreed): 0 embedding rows
remained at any of the 603 paths afterwards, and the orphan count is byte-identical before and
after.

---

## 📖 Filed 2026-08-25 — PJ-385 and PJ-386

### 📖 PJ-385 *(Group 1 · built 2026-08-25, awaiting Boss test)* — a way to read the delete archive back
**His ruling:** *"First, build a way to read that archive back."* Asked whether to proceed with the
603-row removal or build the reader first, he chose the reader.

**Concept:** when Constellation destroys something permanently, the person must be able to see what
it destroyed. Every delete already wrote an envelope before purging and refused to purge if that
write failed — true and useless in the same breath, because the only reader needed a content id the
caller already had to know and returned only change-events.

**Built:** `Settings → Universe & Libraries → Deleted notes` — every removal (trash, recycle bin,
permanent, vanished, startup cleanup, index prune), newest first, with what/when/where/why,
characters of indexed text kept, and changes recorded. Click for the archived text. Reads a file,
opens no database, writes nothing. **Read-only — a record, not an undo.**

**What it revealed on his real data:** 5 deletions in `Eisa Universe`, 8 in
`Eisa Cognitive Knowledge`, zero unreadable lines.

**⚠️ WITHDRAWN — the alarm that prompted this was mine and it was false.** *"Several entries kept 0
characters — the archive is hiding something"* was wrong, and so was the explanation shipped in
fifteen languages (*"its file was already gone before the text could be read"*). The mechanism runs
the other way: the body comes from `note_meta.body_text`, and **601 of the 603 phantoms carry text
— 20,484,230 characters, median 18,984** (18,944 is the median of all 603; an earlier version of
this line attached the wrong population's figure). Zero-character entries are frontmatter-only
notes. Corrected in code, in the TS mirror, and in all 15 locales at this close.

**Nine inspection findings fixed at build time**, the two serious ones both about telling the user
something false on the last surviving copy: a stale-result race that painted one deleted note's
text under another's heading, and envelope addressing by cid alone so a note deleted twice showed
the newer text under both rows. Now `(cid, at)` end to end, with change-events attributed by file
order. **Five more found by the diff-scoped inspection at this close — all fixed** (see preamble).

**Owed, and deliberately NOT built:** a restore → **PJ-386**, now blocked on **PJ-388**.

### 📌 PJ-386 *(Group 2 · decision, not a task — **BLOCKED on PJ-388**)* — should Constellation be able to RESTORE from the archive?
PJ-385 makes the record readable. Whether the app should be able to put a note BACK is a different
question: it needs a write path (where does it go if its original path is occupied or its library
deregistered?), collision rules, and re-indexing — /migration-sized.

**Blocked, and the block is the point.** PJ-388 measured what the archive actually holds: a
stripped search rendering with the frontmatter, code contents, link URLs and link annotations
already gone. **A restore from this source cannot return the note.** The real question this
decision must answer first is whether the archive should start keeping **raw file bytes** at delete
time — 28.0 KB mean per note, measured.

---

## ⏱️ Corrected 2026-08-25

### ⏱️ PJ-380 CORRECTED — it is not (only) flakiness; the budget is marginal
The best-of-five estimator fixed the noise, and the sibling file `tradition-perf.test.ts` was fixed
the next day after being left behind (a half-sweep of my own). But it went red again under load,
and the honest diagnosis is different from "flaky": ranking all 24 traditions on an unloaded
machine, `maimonidean-prophecy` is the **slowest**, and the slowest few sit close enough to the
16 ms budget that any real contention tips them over. In isolation all 27 pass.

So the remaining redness is a **signal about Sight v6's cost on the slowest traditions**, not a
defective test. Treating it as noise — which is what I did twice — would have buried that. Owed:
either measure relative cost (ratio to a baseline op in the same run, so machine speed cancels) or
justify and restate the budget. **NOT another round of "make the timing kinder".**

### 🚨 PJ-384 — the empty-cid archive gap (population now filed separately as PJ-387)
Measured: a delete of a row with an empty `cid_cn` writes **no archive line at all**. **Zero of the
603 are affected** — all 603 carry a content id, so all 603 will archive. Not a blocker for the
prune; it IS a gap in the promise the new reader makes, and this close's `settings.deleted.intro`
now states it plainly to the user. PJ-384 remains the **mechanism**; **PJ-387** is the population
and the Boss-ordered job.

---

> **What changed in v1.98** (**PJ-369 Steps 3 and 4 — the executor shipped, the door built and
> panelled; and what the 603 actually ARE, measured**):
>
> **► NEXT ACTION (2026-08-24 evening): the Boss decides whether to proceed with the removal.**
> Not a test request — a disclosure and one ruling, per the panel. Step 4 is built and its six
> blocking findings are fixed, but the panel's verdict is **"Safe: yes. Right, now: no"** until the
> record is current, which this version makes it.
>
> **⚠️ SO#9 SELF-CORRECTION.** Commit `ad08c542` (PJ-369 Step 3) edited `v1.97` **in place** rather
> than writing a new version. The ledger's own rule is that each close writes a NEW file so the
> trail is durable. Recorded here rather than quietly fixed: v1.97 on disk now contains entries
> filed during the Step-3 close, which its own preamble does not announce.
>
> ### 📏 WHAT THE 603 ACTUALLY ARE — measured, and it changes the stakes
> Every phantom in `Eisa Universe` was resolved against the daily universe's index and disk:
>
> | | count |
> |---|---|
> | alive in `Eisa Cognitive Knowledge`, matched by **content id** | **597** (590 byte-identical) |
> | alive, matched by filename | 1 |
> | **genuinely gone** | **5** |
> | `note_state_history` rows at stake | **0** |
> | earned links at stake | **0** |
>
> The five, itemised: four inside an already-emptied `Eisa Test\.trash` (7, 7, 18 and 39 characters)
> and `Town Eisa v5.md` (63 characters) — **134 characters in total**. These are not lost notes.
> They are forwarding addresses left behind when MIG-108 unified his libraries under the universe
> root; his daily universe is a Linked Universe of the one being cleaned, which is *why* they exist.
>
> ### ✅ PJ-369 Step 3 — the prune executor (committed `ad08c542`)
> Classify-all-first, then remove through the single delete funnel, archive-first. Universe check
> before EVERY delete; re-stat immediately before each; no safety cap (the human confirm is the
> ceiling); a row whose cid is empty is REFUSED because its history could not be archived.
> Proven against a COPY of the live database: 603 removed / 0 failed, all 12 path-bearing tables
> clean including `sky_nodes` and `sky_links`, 603 archive lines, 2,731 → 2,128 rows, second run
> finds nothing, mid-run switch stops after 3 with an honest receipt.
>
> ### 🚪 PJ-369 Step 4 — the Settings control (built; NOT committed; six panel findings fixed)
> "Remove stale index entries" in Settings → Index, rendered only above zero; a `danger` confirm
> quoting the count; a receipt with a separate honest line per outcome. Six blocking fixes from the
> panel, every one areal defect:
> - **C1 (BLOCKER)** — the consent sentence promised "the note archive", a place that does not
>   exist as a destination: `read_history_for` has **no callers outside its own tests** (its module
>   header says so) and the two commands that surface note history query a table the removal
>   deletes. Reworded in 15 locales + manual + help: a record is written to a file in the universe
>   folder, **Constellation cannot read it back**, this cannot be undone.
> - **C2** — the dialog quoted the boot pass's STORED count. Deregistering a library raises the real
>   count immediately, so the user would consent to one number while a larger set was removed. New
>   `phantom_prune_count` command classifies at click time, sharing `derive_candidates` with the
>   remover so the quoted number comes from the code that decides.
> - **C3** — `refreshPhantomCount` turned a null report into `0`, i.e. "no answer yet" into "all
>   clear", hiding the control rather than admitting it did not know. The loader's own doc warns
>   against exactly this.
> - **C4** — `prune_stale_phantoms` had **never executed**: both live runs entered one level below
>   it. `derive_candidates` extracted and pinned by a live test — 603 candidates, 0 undecided,
>   agreeing with the harness's independent derivation.
> - **C5** — the modal could be closed mid-run and the receipt was component-local, so a removal
>   could complete with its outcome shown nowhere. Overlay guarded while busy; receipt moved to a
>   session store.
> - **C6** — the notice still said "Nothing has been changed" in a state reachable only AFTER a
>   partial run had changed things. Clause dropped in 15 locales; the sentence is now true before
>   and after.
>
> Plus, from the `ui-inspector`: **Enter instantly confirmed the dialog.** The first fix keyed it to
> `danger`, which would also have taken Enter from the everyday note-delete confirm — an unasked-for
> change to a daily flow, and a note delete is recoverable. Narrowed to an explicit `enterConfirms`
> opt-out used only where there is no way back.
>
> ### 🧾 Still owed to the Boss as decisions, not tasks
> **PJ-381** (Overwrite-on-collision can still discard unsaved work — two product options),
> **PJ-378** (58 sweep findings needing ranking), **PJ-384** (an empty-cid delete purges with no
> archive; mitigated in the phantom path, root fix outstanding — the panel recommends proceeding
> rather than blocking on it, and says the appetite is his).

---

> **What changed in v1.97** (**PJ-369 Step 2 BUILT and awaiting the Boss's test; a whole-app
> sweep's APP-KILLER fixed and reproduced; a provenance error of mine corrected in three files**):
>
> **► NEXT ACTION (2026-08-24): the Boss tests PJ-369 Step 2.** Nothing here is committed — the
> standing order is that he tests and passes every build BEFORE the commit. After his pass:
> PJ-369 Steps 3–5, then **PJ-378 triage** (the 58 remaining sweep findings), then PJ-375's
> probe repair, then PJ-367 → PJ-366 → PJ-360, then MIG-111 B2/B7.
>
> ### 🛑 PJ-377 — Delete could destroy the only copy of work a failed save left behind (APP-KILLER · **FIXED after panel rework, pending Boss test** · one part OPEN → Boss ruling)
> - **PJ-377** *(APP-KILLER · Group 1 · silent-data-loss · FIXED in the working tree)* —
>   `preserveWorkBeforeVacating` (`src/lib/libraries/store.ts:5291`) decided whether it was safe
>   to destroy a path's recovery state by asking **`isNoteDirty` alone**. A model can be *clean*
>   and still hold the only copy of the user's work: after a failed save (file locked by a sync
>   agent or AV) the edit lives only in the write-ahead net, and the next boot's
>   `restoreSessionTabs` seeds a model from that net and truth-sets its baseline, leaving
>   `netUnsaved = true` on a model that is clean by construction. Delete, an **ancestor-folder**
>   delete, or an Overwrite in a collision dialog then reported "already durable", and the caller
>   wiped the net, its localStorage backup, the save-failure banner and the model. The trashed
>   file was the PRE-EDIT version; the paragraph existed nowhere; nothing was surfaced; two of the
>   three triggers never touch the note directly.
>   **The obvious fix was rejected after checking:** adding `|| hasUnsavedRecovery(t.id)` to the
>   flush list would still report durability having written nothing, because `flushIfDirty`
>   returns `ok` WITHOUT writing for a clean model (`noteSession.ts:392`) — the bug surviving
>   behind a correct-looking guard, the third "half a sweep" in a function whose own comments
>   already record two. The shipped fix keeps the net and returns `false`, which both callers
>   already honour (they complete the delete, skip the aux-state wipe).
>   **Reproduced first:** `tests/pj-377/vacateKeepsRecoveryNet.test.ts` — all three triggers go
>   red pre-fix (`expected undefined to be 'the paragraph that never reached disk'`), control green.
>   Found by the 2026-08-24 whole-app sweep; register in `lab/reports/safety-sweep-2026-08-24-whole-app.md`.
>
>   **AMENDED after the adversarial panel — the first fix was INCOMPLETE.** The panel reproduced
>   two doors it left open, and the record above understated the exposure. What changed:
>   - **It asked about the TAB; the net is keyed to the PATH and outlives the tab.** `closeTab`
>     deliberately keeps both the net and the banner ("preserve a failed write and restore it on
>     reopen"), so the commonest sequence — save fails, tab closed, note deleted later — walked
>     past every guard. The predicate now reads the net itself (`netPathsToPreserve`).
>   - **Protecting one note leaked every sibling.** A bare `false` skipped the aux cleanup for the
>     whole deleted folder. `AuxStateAction` gained a `keep` variant; cleanup is now per-key.
>   - **`netUnsaved` is set at TWO sites**, not one — `noteModel.ts:535` and `noteSession.ts:333`,
>     the second in a LIVE session on the superseded-write branch, no restart involved.
>   - **A test named a trigger it did not run** ("Overwrite-on-collision" calling `moveToTrash`
>     alone). Retitled; the gap is stated in the test body.
>   - **Overwrite-on-collision is NOT fixed and cannot be fixed by a predicate** — the vacated path
>     is re-occupied in the same click, so path-keyed preservation has nowhere to live. The
>     remedies (a `(recovered copy).md` sibling, or an honest warning in the dialog) are product
>     decisions. **→ Boss ruling owed.**
>   Six cases now pass; five go red against the pre-fix code, control green.
>
> ### 🛡️ PJ-382 — the Attack-1 federation guard refused only a TOTALLY unresolved federation (**FIXED same day**)
> - **PJ-382** *(MED · Group 2 · index-divergence · found AND FIXED 2026-08-24)* — found by the
>   first correctly-scoped diff inspection, **in code written and called fixed two hours earlier**.
>   `ClassifierCtx::build` tested `linked_roots.is_empty()`, while the field's own doc promises
>   *"If ANY linked universe fails to resolve … `refused` is set."* The gap is reachable because
>   the guard's two inputs disagree about a missing child: the strict resolver **keeps** a
>   `NotFound` child (universe.rs:750), `resolve_libraries_recursive` **skips** it (:641-644).
>   Two Linked Universes, one renamed in Explorer between sessions → non-empty set → guard silent
>   → every parent-index row under the renamed child's old path classifies as a phantom, although
>   those notes exist.
>   **Not hypothetical here:** `Eisa Universe` declares TWO children (`كون عيسى` and
>   `Eisa Cognitive Knowledge`). Both resolve today, so the 603 stands — but rename either folder
>   and the count would start counting real notes as phantoms. At Step 3/4 the same verdict feeds
>   `reindex_delete_note`: deleting a Linked Universe's index rows, a write-sovereignty violation.
>   **FIXED in-pass** (WA#6) as a pure `federation_is_complete(declared, linked)` — every declared
>   child must contribute at least one resolved library root — extracted free of `AppHandle` on
>   the `foreign_roots_of` precedent so the test exercises the real function. Four new tests; two
>   go red against the old semantics, including a prefix-lookalike case (`child b2` must not
>   satisfy `child b`).
>   **Why nothing caught it:** `attack1b` passed throughout — it proves a *refused* context yields
>   `Unknown`, i.e. the **consumption** of a refusal, never the **decision** to refuse. A guard
>   needs a test on the predicate that arms it, not only on what it triggers.
>
> ### 🧨 PJ-383 — the PJ-382 fix BROKE the feature; caught only by the Boss's test (**FIXED**)
> - **PJ-383** *(HIGH · Group 1 · regression-from-a-fix · found by BOSS TEST 2026-08-24, FIXED)* —
>   PJ-382's Attack-1 guard compares declared child roots against resolved linked-library roots.
>   `resolve_child_universe_roots_recursive_strict` returns **canonicalised** paths; on Windows
>   `fs::canonicalize` yields the VERBATIM form. Probed on this machine:
>   `canonicalize` on the universe folder returns the verbatim form — the path prefixed with
>   `\\?\` — e.g. `\\?\E:\Constellation Universes\Eisa Universe`,
>   which normalises to `//?/e:/…` while every library path from `libraries.json` normalises to
>   `e:/…`. They can never be equal → **every** declared child looked unresolved → the guard
>   refused every run → the feature went permanently silent.
>   **The Boss's Step 2 showed no sentence at all.** Diagnosed in one read from his
>   `diagnostics.log`: *"phantom classification INCOMPLETE — 612 row(s) undecided; run refused:
>   the federation resolved only partially"* — the very diagnostic added earlier the same day
>   because a refusal and a clean universe were otherwise indistinguishable.
>   **FIXED** by stripping the verbatim prefix inside `norm` (a no-op for plain paths, so it
>   cannot disturb them, and it inoculates every future comparison rather than one call site).
>   **Verified against his real data** before re-asking: replaying both sides from `universe.json`
>   and each child's `libraries.json` yields `federation_is_complete -> True`, 2/2 children
>   contributing, 20 linked roots.
>   **Why no test caught it:** `attack1c`–`attack1f` fed the function **pre-normalised literals** —
>   exercising the LOGIC, never the INPUT FORM the caller supplies. The same blind spot as
>   `attack1b` (which tested the *consumption* of a refusal, not the *decision*), twice in one
>   module in one day. New rule, now in the orientation: **a pure function extracted for
>   testability inherits none of its caller's input forms — test it with a value produced the way
>   the caller produces it, or the test only proves the arithmetic.**
>   `attack1g` now canonicalises a real temp directory; it goes red without the strip.
>
> ### 🗄️ PJ-384 — a delete with no content id purges without an archive, silently
> - **PJ-384** *(LOW · Group 3 · silent-data-loss · found 2026-08-24 by the Step-3 diff inspection ·
>   PARTLY mitigated, root fix owed)* — `build_delete_archive` returns an EMPTY archive when
>   `note_meta.cid_cn` is empty (deliberately: a record keyed on no identity is one no reader could
>   find). But Phase 2 of `reindex_delete_note` is gated on `!archive.is_empty()`, so an empty
>   archive **skips the archive-first contract entirely** and Phase 3 purges anyway, returning Ok.
>   The note's `body_text` and its `note_state_history` (destroyed by the ON DELETE CASCADE at the
>   `note_meta` delete) go with it. CLAUDE.md states that history lives in exactly one place.
>   **Measured, and the two agents disagreed because they read different universes** — the same
>   which-universe error made on paper earlier the same day: `Eisa Universe` has **234 of 2,731**
>   rows with an empty cid (8.6%); `Eisa Cognitive Knowledge` has **25 of 8,031** (0.31%), of which
>   15 carry change history. Of the 603 current phantom candidates, **0** have an empty cid — so the
>   phantom path is latent, not live.
>   **Mitigated in-pass:** PJ-369's executor now REFUSES such a row (`skipped`, with a reason a
>   receipt can carry) rather than purging it unarchived — for a phantom the file is already gone,
>   so the index holds the last copy. The skip notice also moved from `eprintln!` (invisible in a
>   release Windows GUI build) to `diagnostics.log`.
>   **Root fix still owed, and deliberately not smuggled in:** the archive should survive an empty
>   cid by keying on the universe-relative path the `link_life` header already promises. That
>   changes SHARED delete semantics — it affects trash, permanent delete and boot reconcile — so it
>   is its own job, not a rider on Step 3. Two live funnels reach the unguarded line today:
>   `delete_path` mode="permanent" (`libraries.rs:9717`) and reconcile's `ReconcileGone`
>   (`reconcile.rs:663`).
>
> ### 🧾 PJ-378 — the 58 remaining confirmed findings from the 2026-08-24 whole-app sweep
> - **PJ-378** *(Group 1 triage · 8 HIGH + ~26 MED + ~24 LOW)* — a full cycle's remediation,
>   filed as ONE entry for ranking rather than 58 rows that would bury the ledger. They are **not**
>   "logged and shipped": no ruling has been requested yet because SO#10 requires the PCS and
>   orientation current first. Several must be fixed as **families**, not one-by-one
>   (Whole-Ecosystem Fix Law):
>   - **missing `ensure_search_db_ready`** — `bases.rs:437`, `tasks.rs:540`, `shape.rs:214`,
>     `universe.rs:2643`, `libraries.rs:1598` (HIGH): a gated frontmatter write lands on disk
>     while its reindex fails to a diagnostics line only.
>   - **YAML quote/escape** — `store.ts:2846`, `libraries.rs:3131` (HIGH), `:2849`, `:3002`,
>     `ExpressionForge.svelte:139`: emitters that quote without escaping, and a Rust rename path
>     that never received the escape-DECODE fix its TS twin did.
>   - **swallowed write errors** — `propertyTypeRegistry.ts:111/126`, `file_kinds.rs:187`,
>     `shape.rs:239`, `libraries.rs:2144`, `+layout.svelte:4900`.
>   - **PropertyEditor re-seed without `{#key}`** — `PropertyEditor.svelte:457` (HIGH, twice) and
>     `:628`: a tab switch can discard a pending property edit silently.
>   Process note, owned: the per-build inspection fell back to a whole-app sweep because I passed
>   `args.files` as a string — **the same mistake as 2026-08-23**, already documented in that day's
>   own register.
>
> ### 🧭 PJ-381 — Overwrite-on-collision can still discard unsaved work (BOSS RULING OWED)
> - **PJ-381** *(HIGH · Group 1 · silent-data-loss · NOT fixable by a predicate · found 2026-08-24)*
>   — the collision dialog's **Overwrite** runs `moveToTrash` then `renameItem` on ONE click, so
>   the vacated path is **immediately re-occupied** by the incoming note. PJ-377's protection is
>   path-keyed, and a path that is re-occupied has nowhere to keep a net for the note that just
>   left it. So a note holding unsaved, save-failed work that is displaced by Overwrite can still
>   lose that work.
>   Two real remedies, both **product decisions, not refactors**:
>   1. **Write a `(recovered copy).md` sibling before vacating** — the work survives as a real file
>      the user can see, at the cost of an extra file appearing without being asked for.
>   2. **Warn honestly in the collision dialog** — "this note has unsaved work that could not be
>      saved; overwriting will discard it" — cheaper, but relies on the user reading it.
>   **→ The Boss chooses.** Filed rather than guessed, because the trade-off is his taste and risk
>   appetite, not an engineering fact. Surfaced by the adversarial panel on the PJ-377 rework.
>
> ### 🌍 PJ-379 — the index-drift band is documented in English only (12 of 14 manuals lack the whole section)
> - **PJ-379** *(MED · Group 2 · localization debt · PRE-EXISTING, found 2026-08-24)* — the User
>   Manual section "If your notes changed while Constellation was closed" (the amber band, Repair
>   now, Settings → Index → Index repair) exists in **English, ja and zh only**. Checked across all
>   14 translated manuals: ar, de, es, fa, fr, he, hi, ko, pt, ru, tr, ur carry **no** coverage of
>   it at all. This predates PJ-369 — the band shipped with PJ-207 §9 and was never back-translated.
>   It matters because [[feedback-full-localization-everything]] is a TOP-PRINCIPAL standing order:
>   when the user switches language, EVERYTHING adapts. A user reading Arabic gets an amber band in
>   Arabic and a manual that never explains it.
>   PJ-369 Step 2 added one paragraph to the English manual + English help topic. Deliberately NOT
>   inserted into the 12 manuals lacking the surrounding section — a lone paragraph about phantom
>   entries, dropped into a manual that never introduced the band, would be incoherent. The right
>   fix is to translate the SECTION, then the paragraph with it. Owed at PJ-369 Step 4, when the
>   removal control makes the text final and worth translating once rather than twice.
>
> ### ⏱️ PJ-380 — a perf-budget test can go red because another process is busy (**FIXED same day**)
> - **PJ-380** *(LOW · Group 3 · test determinism · found AND FIXED 2026-08-24)* — `tests/sight-v6/perf.test.ts`
>   asserts "Hearst facet-count rebalancing on 7,636 notes completes in **≤32 ms**". It failed once
>   during a concurrent `cargo build --release`, then passed in isolation (35 ms total test time) and
>   again on a clean full run. Nothing in that day's diff touches Sight v6. The assertion measures
>   **wall clock**, so it reports the machine's load as a code regression.
>   Why it is worth fixing rather than tolerating: a suite that can go red for reasons unrelated to
>   the code teaches everyone to re-run red tests instead of reading them — which is precisely how a
>   real regression gets waved through. Fix shape: measure work (operations/allocations) rather than
>   elapsed time, or move wall-clock budgets into a quarantined suite that never gates a commit.
>   **FIXED**: it failed **3 of 5** full-suite runs, so it was fixed rather than tolerated. All four
>   budgets in that file now take the **best of five** runs (`fastestMs`) — the standard estimator,
>   since scheduler noise and GC can only ADD time while a genuine regression slows even the
>   fastest run. The budgets keep their meaning; only the noise is removed. Three consecutive
>   clean full-suite runs afterwards (1008 passing each).
>
> ### 🧭 PJ-369 — Step 1 committed; **Step 2 built, awaiting the Boss's test**
> Step 2 counts phantoms at boot and says the number; it deletes nothing and is read-only by
> construction (`open_read_only_search_conn`: `SQLITE_OPEN_READ_ONLY` + `query_only=ON`). The
> sentence carries **no "Repair now"** button — a repair walks libraries and re-reads files, and a
> phantom has neither, so the control would be a door that does nothing.
> **Two defects in my own draft, caught before shipping:** (1) it named a Settings route that does
> not exist (the nav is flat; there is no "Universe & Libraries" parent); (2) it pointed at the
> removal control, which does not land until **Step 4** — a false door, one paragraph after the
> design forbids exactly that. Step 2 ships the honest half; Step 4 appends the route.
> **Deliberate, recorded deviation from the approved plan's sentence.**
>
> ### 📍 A provenance error of mine, corrected — the 603 are NOT in his daily universe
> Step 1's audit was recorded in source as *"the Boss's own **daily** universe."* Measured today,
> read-only, against both live databases: **`Eisa Cognitive Knowledge` (his daily universe, 8,031
> rows, 19 libraries) has ZERO phantoms and zero missing files** — structurally, because its own
> root is itself a registered library, so no row beneath it can be "outside a library."
> The 603 phantoms and 19,472 edges are in **`Eisa Universe`** (2,731 rows), all pointing at the
> dead `E:\Cognitive Knowledge\…` path. Corrected in `phantom_prune.rs`, `reconcile.rs`,
> `driftReport.ts`. Uncorrected, it would have sent his test to a universe where the feature
> correctly reports nothing. Caught by *measuring boot cost*, not by re-reading the comment.
>
> ### ⚡ Boot cost of Step 2, measured (not estimated)
> On the live 312 MB `Eisa Universe` db, 621 candidates: per-row indexed lookups (what ships)
> **11.6 s cold / 0.042 s warm**; the "obvious" three-batched-scans optimisation **38.8 s** — ~3×
> WORSE, because the cost is cold random page reads, not query shape. **0.000 s** on the daily
> universe (nothing to classify), and the pass runs on a background thread, so it delays no paint.
> The table is now a comment in `has_earned_data` so the next reader does not "optimise" it into
> the slower form.
>
> ### 🔬 PJ-321 — a FIFTH corroboration, observed in passing
> Read independently today while verifying the test's universe-switching step:
> `%APPDATA%\world.uconstellation.app\universes.json` (identifier confirmed at
> `tauri.conf.json:5`) still holds **one entry (`كون عيسى`), mtime 2026-08-07** — while all three
> universes' databases were written today and today's write journal names all three. **The STOP
> order was honoured: observed, not diagnosed.** Consequence for the test: the app demonstrably
> lists and switches to universes absent from that file, so the Universe-Manager step is expected
> to work — the `ui-inspector` was asked to rule on it rather than my assuming either way.

---

> **What changed in v1.96** (**MIG-111 Stage B1 SHIPPED — the backfills' recomputes take their
> callers' pinned vocabulary; R4 proven; PLUS the largest single-build inspection harvest of the
> project: 46 confirmed findings across SEVENTEEN inspection passes — 40 fixed, 6 filed — plus a
> whole-app sweep register**):
>
> **► NEXT ACTION (2026-08-24): PJ-375 — INVESTIGATED, one unknown left to close.** The probe
> repair is designable now; what re-fragments the index within minutes is not yet evidenced and
> must be established before anything is changed. See the investigation under PJ-375 below.
>
> **► (superseded) NEXT ACTION (post-B1, 2026-08-23): PJ-375** — the disk churn he CONFIRMED hearing. It is
> the only item in the queue backed by a measurement on his own machine of a cost he is already
> paying (54s and 78s FTS5 optimizes today; 130 runs, 510s total, against an in-code claim that
> it is cheap after the first), and it fires on exactly the federated path his ruling below
> selects. Reproduce-First governs it: investigate WHY the optimize keeps doing full work before
> changing anything, and do not "fix" it by deleting the prewarm, which exists to make federated
> search usable. Then the federation-honesty cluster (PJ-367 → PJ-366 → PJ-360), then B2.
>
> **► BOSS RULING 2026-08-23: "Do you expect to start working across Linked Universes soon?"
> — YES.** The close panel's branch is therefore selected: **federated search is fixed next,
> ahead of everything.** And his answer promotes a CLUSTER, not one item, because all three of
> the federation gaps found today bite exactly when he starts — and the two cheap ones bite
> FIRST, before search is even reached:
>   1. **PJ-367** — he links a universe from the Universe Manager and it does not appear.
>   2. **PJ-366** — it is not searchable at all until he restarts, with nothing saying so.
>   3. **PJ-360** — and after restarting, searching still does not reach it.
> That is the order he will physically encounter them in. 367 and 366 are small; 360 is
> `/migration`-sized (a read-path architecture change across Rust and Svelte) and goes through
> the front door: concept first, Architect, panel, then his approval — not a patch.
> **PJ-344's REPRODUCTION still comes first** (hours, and Reproduce-First makes it the only
> shippable work on that ticket until its trigger fires on demand), then this cluster, then B2.
>
> **► SUPERSEDED — PJ-344** (HIGH · the normPath cascade-reload class in `store.ts` — an open
> referrer with a form-drifted tab path silently keeps its pre-cascade body and the first
> keystroke's save durably reverts the cascade's rewrite on disk; the fix is the 2026-08-01
> normPath treatment applied to `reloadTabsFromDisk`'s three raw comparisons + the sibling raw
> sites; sequencing ruling owed — recommended BEFORE **MIG-111 B2** because it is cascade
> integrity, this stage's own concern), then **B2** (DDL takes the vocabulary explicitly), then
> **B7** (pin the watcher fence). PJ-326..331 + PJ-341..343 rulings queued.
>
> ### ✅ MIG-111 Stage B1 — SHIPPED, BOSS-VALIDATED, COMMITTED `3f0f06a7` (2026-08-23)
> Stage 1 (rename in the active universe · rename inside a Linked Universe · reopen from disk ·
> delete) and Stage 2 (the new folder-holds-a-Linked-Universe refusal, banner verbatim) both
> passed on the 18:31 release binary. Committed as ONE commit: the pre-existing repairs are
> interleaved with B1's own hunks in the same files, so a split would have manufactured an
> intermediate state nobody ever built or tested — the panel's own condition for collapsing it.
> The commit message carries a "NOT B1" section naming every pre-existing repair for a future
> bisect.
> All six backfill `recompute_*` functions and the five converge entry points take
> `&LinkTypeRegistry` from their callers' pinned scope. The B1 investigation found
> `links_backfill::run` still re-locked the swappable `SearchState.db` per batch with no pin
> (the PJ-332 wrong-stamp class; the sky doc's "sibling pinned" claim was inaccurate) — full
> pinned rework (own connection, generation stop, pinned-fingerprint finalize); incoming + sky
> pin the same way, with the universe root DERIVED from the pinned db path (one ambient read —
> the two-read window self-caught in diff review and closed at all four backfills incl.
> name_fold). Strict converge-caller reads (repair → all-Failed report keeps markers armed;
> heal → Err; mig108 → abort). New R4 test: a routed `WriteScope` write changes NO
> `schema_versions` row (the two stamp writers are module-private, callers = their own
> schedulers). Census: sky_backfill + links_backfill ABSENT, incoming 4 (reader-gate roles).
>
> ### 🛡 The B1 inspection cycle — 17 passes, 46 confirmed, 40 fixed, 6 filed
> *(14 over a changing tree, then 3 FROZEN. The frozen passes were the panel's closing gate and
> earned it: the second found an APP-KILLER that was itself a fix from the first. Detail below
> and in the session log.)*
> Pass 1 (4): the B6 Err-fallback's universe_notes trap at BOTH rename tails (HIGH — a routed
> refusal rerouted into the ACTIVE tail = wrong-DB adoption); links cursor resumed across a
> vocab edit stamps the new fingerprint over an old band (cursor now records `vocab_fp`);
> boot-heal vs vocab-change backfill interleave (heal now gives way; stamps cleared + backfills
> rescheduled after waiting for in-flight runs — the wait itself a self-caught fix); bring-in
> COPY arm now refuses junction'd sources BEFORE copying (Move-arm parity). Pass 2 (2 + 4
> limit-orphaned candidates verified by hand): geometry-first fallback (`legacy_external_own_path`
> — outside-the-active-root, not the degradable lenient foreign set); the gate/run fingerprint
> SPLIT my own B1 pin created — with the new re-arm it was a session-long zero-delay full-table
> recompute loop in BOTH modules (gates now single-source from the disk registry); sky Phase C
> per-column guards. Pass 3 (4): the ROUTED folder tail got the strict third-universe fence
> (and the ACTIVE tail the best-effort one); over-refusal of plain active notes on a corrupt
> unrelated manifest (new first-level admission); gen0 captured at COMMAND start on the IPC
> thread and threaded into all three detached tails; sky cursor `vocab_fp`. Pass 4 = the dry
> check. Suite 1541×2 green after every round (LL-050 relinks applied).
>
> ### Passes 4-6 addendum (written before the Boss test; counts final at commit)
> Pass 4 confirmed the sharpest catch of the cycle (HIGH): the pass-3 ACTIVE-arm fence
> silently degraded to an EMPTY set in exactly the states the pass-3 admission opens, and a
> folder rename MOVES a nested linked universe while its declared path goes stale. Fixed:
> the door guard gained UNIVERSE-ROOT granularity (declared roots from universe.json checked
> strictly; refusal names the linked universe); the ACTIVE fence is strict-or-loud-skip
> (reindex loop skipped, migration still runs); both arms translate moved roots to their
> post-rename location. Pass 5 confirmed 3 (MED: the cursor-ALTER `let _` swallowed
> busy/locked and sky's whole-table wipe ran through the broken column — both ALTERs now
> tolerate only duplicate-column; LOW: the translation used raw case-sensitive strip_prefix
> — now the shared normalized `translate_moved_root`; LOW: the guard's door check
> disappeared silently on a strict failure — now loudly logged, and the unrepaired
> universe.json↔disk divergence is filed):
> - **PJ-359** *(LOW · Group 3 · ruling owed — product affordance)* — a blind rename (both
>   door belts degraded) can leave `universe.json` declaring a dead child path: the linked
>   universe silently drops out of the federation. The tails protect the INDEX loudly, but
>   nothing surfaces or repairs the severed declaration (recovery today: rename back, or
>   unlink/re-link). Wants a federation-health surface — detect dead declared paths, offer
>   re-link — which is a Boss product decision (the PJ-341/342 family).
>
> ### 📋 The 2026-08-23 whole-app sweep — 29 confirmed, register filed (SO#9)
> Launched as the per-build B1 inspection with a malformed args string; fell back to the
> whole-app cycle sweep (49 agents, 14 scopes). Recorded honestly: full register in
> `lab/reports/safety-sweep-2026-08-23-whole-app.md` + the Charter. 4 findings in B1 files
> were FIXED IN THE B1 BUILD (PJ-332b slots for links/incoming schedulers; the
> `reindex_delete_note` park-window guard; the federation attach publish inside-lock check).
> The rest are FILED here; remediation sequencing = Boss ruling owed at the stage close:
> - **PJ-344** *(HIGH · content-loss · Group 1 · ①)* — the normPath cascade-reload class.
>   **Widened 2026-08-23 by the close panel, which caught me committing the very error I had
>   just written up as my pattern: I filed ONE member of the class.** Verified at source:
>   `reloadTabsFromDisk` compares raw in THREE places (store.ts:1130 `t.path === fp`, :1163 and
>   :1184 `byPath.get(t.path)`) while both siblings arbitrating the same concern were
>   normPath-fixed on 2026-08-01 — and the same function calls a NORMALIZING comparator on
>   itself a hundred lines later (`focusPathAmong(byPath.keys())`, :1203, whose `norm()` at
>   :1100 does `normPath().toLowerCase()`). One function, two comparison policies.
>   **And the half I missed is the more dangerous one:** :1198 does
>   `for (const fp of byPath.keys()) clearWriteAhead(fp)` — a BLIND clear of the write-ahead
>   recovery net — while its sibling at :1403 does `clearWriteAheadIf(p, byPath.get(normPath(p)))`
>   under a PJ-207 §15 comment stating the rule verbatim: *"COMPARE-and-clear, never a blind
>   clear… a net whose content is not what we just adopted… exists nowhere else."* Same
>   function family, same lineage, opposite policy.
>   Also in the class: `toggleTaskReconciled` :1433, `restoreSessionTabs` dedup :3829/:3781,
>   `focusPathAmong`'s missing NFC :1100.
>   **NOT a one-liner** (the panel's correction): normalizing only the filter changes which
>   downstream branches execute on tabs nothing adopted. And **the TRIGGER is not yet
>   demonstrated** — the mechanism is confirmed, the form-drift precondition is not. Per
>   Reproduce-First, the only shippable work on this ticket until it fires on demand is the
>   reproduction: mint a case-drifted `note_meta.path`, run the cascade, observe the skip.
>   I had written the trigger as fact; that is the laundering shape SO#10 exists to prevent,
>   in the same document that cites SO#10.
>
>   **✅ REPRODUCED 2026-08-23 — `tests/pj344/reloadTabsFromDisk-path-drift.test.ts` (5/5).** The
>   trigger now fires on demand, so the fix is designable. What the reproduction establishes:
>   a tab whose stored path differs by SEPARATOR, by CASE, or by NFC/NFD form is not merely
>   un-adopted — **the file is never even READ**; the cascade does not know the tab exists. The
>   discriminator is deliberately "was `read_note` called for this path", not "was the tab
>   adopted", because adoption needs the note-model layer the harness does not register, and an
>   assertion failing for the harness's reasons would make the rest unreadable.
>   Two things the reproduction changed about the fix: (1) `normPath` folds SEPARATORS ONLY —
>   it does not lowercase and does not normalize Unicode — so "just use normPath" would close
>   one of the three drift forms and leave two, and the test carries the three-fold spec the fix
>   must satisfy; (2) writing it caught my own bad example — my first NFC case used an Arabic
>   name whose diacritics are already separate combining marks, so NFC and NFD were identical
>   and its guard failed rather than the code. The guard stayed in.
> - **PJ-345** *(MED · Group 1 · MIG-111)* — `delete_path` has no owner-routing: deleting a
>   Linked-Universe note purges only the ACTIVE DB (a no-op) — the delete tail is the
>   rename/move tails' missing sibling; fold into MIG-111 Phase 1.3.
> - **PJ-346** *(MED · Group 2)* — the missing `screen:note-saved` class: addTagToNote closed
>   branch (+layout:7164), addLinkToNote closed branch (store:1586), linkMentionInNote
>   (store:1667), Global-Tasks toggles — second-screen views stay silently stale. One emit
>   helper on every closed-note/body write.
> - **PJ-347** *(MED · Group 2)* — closed-note frontmatter writes: false success on
>   immutable-block keys (addTag/addLink report success while composeFrontmatter refuses the
>   write) + the JS-side read→modify→write races (store:1578 class).
> - **PJ-348** *(MED · Group 2)* — YAML scalar escaping: ExpressionForge raw-interpolated
>   titles (:139) + `quoteIfNeeded`/Rust alias quoting escaping the quote but not the
>   backslash — one shared quoting helper.
> - **PJ-349** *(MED · Group 2)* — one-shot migration journaling: `migrate_to_constellation`
>   has no commit marker (universe.rs:367); `migrate_legacy_data` saves a registry containing
>   only the migrated entry (universe.rs:2234).
> - **PJ-350** *(MED · Group 2)* — ensure-or-refuse completion: `create_note` (libraries:1431)
>   and `apply_shape` (shape.rs:239 area) are the 2026-08-01 guard's last unguarded siblings.
> - **PJ-351** *(MED · Group 2)* — `shape.rs::record_change` four silent returns + two
>   swallowed writes — the shape trail silently loses entries.
> - **PJ-352** *(MED · Group 2)* — `handleCalendarToggleTask` console-only failure while the
>   sibling popover checkbox surfaces it (+layout:2204).
> - **PJ-353** *(MED · Group 2)* — the second-screen universe-switch handler leaves
>   dashboardMode/indexMode/split-companion residue (SecondScreenPage:747).
> - **PJ-354** *(MED · Group 2)* — `markRecoveredFromNet` has no arbiter for an external disk
>   edit that landed while the app was closed (noteModel:493).
> - **PJ-355** *(LOW · Group 3)* — `saveRecoveredCopy` probes existence via read-failure then
>   overwrites; use exclusive create (store:622).
> - **PJ-356** *(LOW · Group 3)* — CECE detached cataloger threads pile up behind the global
>   embedding mutex during a scan (orchestrator:175; compounds PJ-282).
> - **PJ-357** *(LOW · Group 3 · Rule-1)* — FocusPane's updateListener runs
>   `doc.toString()` + a full-text regex word-split on EVERY keystroke (FocusPane:267) — the
>   exact hot-path class Rule 1 forbids on the plain-capture surface.
> - **PJ-358** — **WITHDRAWN, REFUTED BY ITS OWN AUTHOR (2026-08-23, same day it was filed).**
>   I filed it as "a structural-membership vocabulary change never re-materializes sky's
>   stratum/maturity." Pass 6 forced me to read `LinkTypeRegistry::merge`, and the premise is
>   false: `merge` forces `structural = false` for every seed AND every custom type, and
>   `true` only for the two hardcoded `STRUCTURAL_SEED_IDS` (`parent`, `contains`). **No user
>   action can change the structural set**, so `structural_not_in_clause` — sky's only path to
>   the registry — emits an identical clause for every registry, and the staleness I filed is
>   not reachable. Kept in the ledger as a withdrawal rather than deleted: the reasoning that
>   produced it (assuming a flag in a struct is user-settable because it is a field) is the
>   error worth remembering, and the ledger is the record. The guard code it motivated
>   (sky's cursor `vocab_fp`) stays, annotated honestly at the site as a guard rather than a
>   live fix, and becomes load-bearing only if the structural lane is ever opened up.
>
> ### 🔭 PJ-360 — filed 2026-08-23, found by the TEST GATE, not by the code inspection
> - **PJ-360** *(severity + sequencing = PANEL, then Boss · Group 1 · Universe-of-Universes)* —
>   **Search Hub's ordinary search does not reach Linked Universes.** Typing plain text into
>   Search Hub runs `universalSearch` → `constellation_search_universal`, which reads
>   `state.db` — the ACTIVE universe's own connection. The Linked Universes' databases are
>   only ever ATTACHed to `state.federated_conn`, and the single reader of that connection
>   (`federated_lexical_search_or_fallback`) is called from exactly two places, both inside
>   `execute_search` — the **advanced-syntax** command. `SearchHub.svelte` picks between them
>   on `hasAdvancedSyntax(q)`, so a plain title query never reaches the federated path.
>   Verified independently at the source, not taken on report.
>
>   **WIDENED 2026-08-23 by the `ui-inspector`, which refuted the mitigation I had attached to
>   this very filing.** I wrote — here, in the Federation help file, and in the Boss's test —
>   that "searches written in the advanced syntax DO span Linked Universes." That is false, and
>   it made the gap look half as large as it is. Verified mode by mode at source:
>   `parseSearchQuery` (store.ts) sets `mode = hasQuery && hasFilters ? 'hybrid' : hasQuery ?
>   'lexical' : 'structured'`, and in `execute_search`:
>   * `structured` — **active-universe only** (`structured_search(conn, …)`). This is EVERY
>     filter-only form: `#tag`, `in:Library`, `key=value`, `links to/from/between/all [[X]]`,
>     `mutual`, `mentions`, `orphans`, and every typed-link operator (`supports [[X]]` …).
>   * `semantic` — **active-universe only** (`semantic_search(conn, …)`).
>   * `hybrid` (text AND a filter) — only the LEXICAL half federates; its `structured_search`
>     and `semantic_search` halves take the local connection.
>   * `lexical` — federates, but requires free text with NO filter extracted, which by
>     definition is not an advanced-syntax query. From Search Hub, plain text goes to
>     `constellation_search_universal` (active-only) instead.
>
>   **The practical consequence, which is the real finding:** from Search Hub, essentially
>   nothing the user types reaches a Linked Universe's notes — not plain text, not tags, not
>   `in:`, not properties, not typed links — except the free-text half of a mixed query. The
>   federated machinery exists and is reachable only by a caller that sends free text through
>   the advanced command, which Search Hub never does. Every one of those returns EMPTY rather
>   than saying it did not look.
>
>   **Why it matters more than its size:** the answer is not "unavailable", it is **empty** —
>   the user searches for a note that exists, is told nothing matched, and concludes it is
>   gone. That is the silent-wrong-answer shape, on the app's most-used surface, and it sits
>   directly against the top-principal **Universe of Universes** ruling of 2026-08-22 ("a
>   feature that stops at the universe boundary is incomplete, not scoped"; an honest refusal
>   is a stopgap, and this is not even honest). It also makes the *advanced* search quietly
>   more capable than the simple one, which no user would predict.
>
>   **How it surfaced, which is the part worth keeping:** the `ui-inspector` rejected a step
>   in the B1 Boss test that asked him to confirm a renamed Linked-Universe note via Search
>   Hub. The step would have "failed" in his hands and I would have hunted a rename bug that
>   does not exist — the rename indexes the note correctly into the owner universe's own
>   database (separately confirmed). **Ten adversarial code-inspection passes over this diff
>   did not find this; the gate that checks whether the Boss can actually follow a test did.**
>
>   Not fixed in B1: it is a search-architecture change (route the universal path through the
>   federated connection, or merge per-universe results the way the lexical path does), it is
>   outside this stage's diff, and the Boss's own ruling makes the destination a design
>   question rather than a patch. Panel first, then his call on when.
>
> ### 🪞 PJ-361 — filed by the close panel, against me, on the diff's own premise
> - **PJ-361** *(MED · structural · Group 2 · MIG-111)* — **B1 added a NEW process-global
>   mutable static to the file whose entire purpose was removing ambient global reads.**
>   `link_types::ACTIVE_VOCAB_DEGRADED` (new in this diff) and `recover_active_vocabulary`,
>   which performs a full `set_active(...)` swap of the process-global registry — called from
>   THREE back-fill scheduler threads. It resolves `link_types_path(app)`, i.e. the ACTIVE
>   universe, from inside the very workers B1 exists to make scope-routed, and it sits eighty
>   lines from B1's own comment that "an eleventh ambient reader cannot appear without a
>   compile error."
>   It is not shown to produce a wrong write, and it repairs a real defect (a session stranded
>   on the seed fallback). But it was added at passes 9–10 to fix a pass-7 problem, and **I
>   listed five self-injected defects in my own register and omitted the one that is
>   structural** — the panel found it, not me. The clean shape is to carry the recovered
>   vocabulary in the scope rather than swapping a global; that is B2 territory.
>
> ### ❄ The FROZEN pass (2026-08-23) — 3 fixed, 3 FILED under the panel's closing rule
> The panel's stopping criterion required one inspection over a tree with **zero further edits**
> — every earlier pass ran over a changing tree, which measures what the last fix broke rather
> than what is left. It confirmed six. Three were fixed because each is a law rather than a
> severity call (a content-loss with no recovery path; the loop class's forbidden THIRD
> instance; a read-blanking regression this diff introduced). The other three are filed here
> rather than fixed, because fixing them restarts the freeze and the panel ruled the dominant
> remaining risk is now the author's own fixes under fatigue:
> - **PJ-362** *(MED · index-divergence · Group 2 · MIG-111)* — **the degraded-vocabulary
>   self-heal repairs the aggregates but not the EDGES.** A session on the seed fallback indexes
>   `[[refutes::B]]` as an untyped link with `target_name = 'refutes::b'` in `note_links`.
>   Clearing the two aggregate certificates makes the next boot recompute *from those corrupt
>   edges* and stamp them current; `note_links` is only rebuilt by re-indexing the note, and the
>   boot reconcile is mtime-gated so it skips a note that WAS indexed during the degraded
>   session. The backlink is permanently gone while the `.md` on disk is perfectly correct —
>   **and my own log line promises "will be rebuilt on the next start that reads the file",
>   which is false for the edges.** Fix shape: mark notes indexed under a degraded vocabulary
>   for re-index, or refuse to index at all while degraded. The false promise should be
>   corrected even if the repair waits.
> - **PJ-363** *(MED · cross-window-clobber · Group 2 · MIG-111)* — **`move_item_db_tail` is the
>   fourth sibling.** B6 routed both rename tails by owner and this cycle gave them the geometry
>   belt; the MOVE tail got neither. With the lenient foreign-root set degraded, a move inside a
>   nested Linked Universe indexes that universe's note into the ACTIVE database and strands the
>   owner's rows at dead paths. Same concern, fourth surface — the Whole-Ecosystem Fix Law,
>   recorded against me for the third time this cycle.
> - **PJ-364** *(MED · index-divergence · Group 3)* — **rank 0 baked into the Reviewer during a
>   Sky walk.** The run-start wipe NULLs `stratum`/`maturity` for the band and leaves it NULL
>   for the minutes the walk takes, while the two writers that persist stratum into
>   `review_schedule` read it with `.unwrap_or(0)` and are not gated on sky readiness — so any
>   note touched inside that window is permanently ranked 0. Pre-existing; the fix is a
>   readiness gate on those two writers, not another wipe change.
>
> - **PJ-365** *(MED · index-divergence · Group 2 · MIG-111)* — **the cascade-reindex worker's
>   library fallback is now universe-wide.** B6 broadened the cascade walk from the caller's
>   library to the whole active universe root, but the detached worker still falls back to the
>   RENAMED note's library name for any referrer whose own attribution misses — so a transiently
>   unreadable `libraries.json` stamps one library's name onto rewritten referrers across every
>   library in the universe. Filed under the frozen-pass rule rather than fixed. Found by frozen
>   pass 2.
>
> - **PJ-366** *(MED · Group 2 · Universe-of-Universes · found at the final test gate)* — **a
>   Linked Universe added mid-session is invisible to search until Constellation restarts.**
>   `add_child_universe` calls only `invalidate_libraries_cache()`; the federation attach runs
>   once per universe-activation inside `ensure_search_db_ready`, and its only re-trigger is
>   `invalidate_search_state`, whose sole caller is the universe-SWITCH path. Verified at
>   source. So the newly linked universe's notes appear in the sidebar immediately (the
>   libraries cache was invalidated) while search cannot see them at all — in ANY mode — and
>   says nothing about why. Distinct from PJ-360: that is about which search modes federate;
>   this is about a federation that was never attached in the first place. Fix shape: re-attach
>   (or invalidate) after `add_child_universe`, or surface an honest "restart to search this
>   universe" state. Found because the test gate asked what the Boss would actually observe in
>   Stage 2 — not by any code pass.
>
> - **PJ-367** *(MED · Group 2 · two surfaces, one action · found at the final test gate)* —
>   **linking a universe from the Universe Manager does not refresh the sidebar; from the
>   library switcher it does.** `UniverseManager.svelte`'s `handleAddChild` calls
>   `addChildUniverse` then only its own local `refresh()` (which re-fetches the modal's list);
>   it has no `onChildUniverseChanged` prop at all. `LibrarySwitcher.svelte` does the same
>   action and then calls `getChildUniverses()` + `onChildUniverseChanged?.()`, which is what
>   repopulates the sidebar's Child Universes section. So the SAME user action produces two
>   different outcomes depending on which surface you used, and via the Universe Manager the
>   linked universe simply does not appear until restart or a universe switch — with no
>   explanation. Verified at source. Fix shape: one shared post-link refresh both surfaces call.
>   *(Not a blocker for the B1 test: the rename refusal it exercises is computed in Rust from a
>   fresh disk read of `universe.json`, which `add_child_universe` atomic-writes before
>   returning — so the guard fires even while the sidebar is stale. Verified.)*
>
> ### ⚠ PJ-368 — the WRITE-side twin of PJ-360, and it accumulates with use
> - **PJ-368** *(HIGH · index-divergence + write-sovereignty · Group 1 · MIG-111)* — **saving a
>   note that lives in a Linked Universe writes its row into the PARENT's index.** The save
>   command does `ensure_search_db_ready(&app)` then
>   `reindex_single_note(&state, &note_path, &library_name)` — the ACTIVE universe's
>   `SearchState`, with a library name supplied by the FRONTEND. There is no owner routing on
>   this path at all. Verified at source 2026-08-23.
>
>   **This is the same class MIG-111 B6 fixed for RENAME and missed on SAVE** — the fourth
>   instance this session of fixing one member of a concern and leaving a sibling. B6's rename
>   tail routes through `WriteScope` into the owner's own database precisely because the Boss
>   ruled (MIG-111 ruling 2) that an operation on a Linked Universe's note does its bookkeeping
>   in THAT universe's database. The save path never got it.
>
>   **Why it outranks PJ-360 in sequence even though PJ-360 is the ruled priority:** PJ-360 is a
>   read-side gap that is constant. This is a write-side contamination that GROWS every time he
>   edits a federated note — the parent accumulates rows for files it does not own (which the
>   boot reconcile skips rather than heals), while the OWNER universe's own index goes stale for
>   the note he just edited. And he has just said he is about to start working across Linked
>   Universes, which is exactly the activity that accumulates it. Every completeness claim
>   PJ-360 would make is made over an index that is being contaminated underneath it.
>
>   Filed as the Architect document's **Phase 0**: apply B6's owner-routing pattern to the
>   save/reindex tail, and purge the foreign rows already present. Not a new design and not a
>   new ruling — an existing ruling applied to the surface that missed it.
>
>   **MEASURED on his own data 2026-08-23** (read-only, against today's backup, not theorised):
>   **9 rows** in `Eisa Universe`'s index belong to notes living in `Eisa Cognitive Knowledge`,
>   and 0 to `كون عيسى`. So the mechanism is not hypothetical — it has fired, nine times. It is
>   small TODAY only because he has barely worked federated yet, which is precisely the activity
>   he has just said he is about to start. The fix is worth doing before that, not after.
>
> ### 🔎 PJ-369 — 601 search results in his everyday universe that open nothing (MEASURED)
> - **PJ-369** *(HIGH · index-divergence · user-facing · Group 1)* — **`Eisa Universe`'s index
>   holds 612 rows for notes at `E:\Cognitive Knowledge\…`, a location no registered library of
>   that universe covers — and NONE of the 400 sampled still exist on disk. 601 of them carry
>   body text, so they are findable by search.** Measured read-only against today's backup while
>   investigating PJ-368; it is a different finding that the same query exposed.
>
>   What he would experience: searching turns up a result, he clicks it, and nothing opens —
>   601 times over. The rows are almost certainly the residue of the pre-MIG-108 layout (his
>   libraries lived under `E:\Cognitive Knowledge` before unification moved them under the
>   universe root); `Eisa Universe` registers five libraries and every one is under its own root.
>
>   **Why nothing has cleaned them up** — the part that makes this a design finding rather than
>   a stale-data annoyance: the boot reconcile prunes rows whose file is missing, but it is
>   documented as SKIPPING rather than healing rows that sit under no owned root. These sit under
>   no owned root. So the one mechanism that would remove them is the one that refuses to look at
>   them, and they have survived every boot since.
>
>   **INVESTIGATED & REPRODUCED on the LIVE db 2026-08-24.** Bigger than the headline: not 601
>   search results but **603 phantom notes** carrying **19,472 phantom note_links edges**, plus
>   603 rows each in `sky_nodes`, `review_schedule`, `note_body` and 127 `note_aliases` — all
>   under `E:\Cognitive Knowledge\...`, 601 confirmed absent from disk. The dead notes pollute the link
>   graph, Sky View and the Reviewer, not just search. **PROVENANCE CORRECTED 2026-08-24: `E:\Cognitive Knowledge`
>   is a SEPARATE legacy universe (bare root `universe.json`, name "Constellation Discovery"), NOT
>   pre-MIG-108 residue of THIS universe's libraries as I first wrote — I inferred that from the
>   path shape without reading the file (No-Guessing miss, caught by the design workflow and then
>   verified). Its content now lives in the linked universe "Eisa Cognitive Knowledge": 40/40
>   sampled phantom filenames have a same-named note there. The prune is therefore SAFER than
>   first thought — not this universe's own notes, content preserved elsewhere.**
>
>   **Mechanism confirmed (reconcile.rs step 3; search.rs:12371):** the boot reconcile is
>   DISK-FIRST — it walks each registered library's own root and checks the index against the
>   files found. Rows under NO walked root are never visited (it reports "0 rows without a file"
>   while 603 sit unseen), and step 3 **deliberately `continue`s** every row outside its
>   own+foreign roots for a load-bearing reason (WA#4): `path.exists()==false` cannot tell a
>   truly-deleted note from one on an UNMOUNTED drive, so a blind delete would destroy real
>   notes on an offline mount.
>
>   **The safety crux (why it is not a one-liner):** a correct prune needs a MOUNT-AWARE
>   discriminator — a true phantom is file-gone AND nearest-existing-ancestor-readable (mount is
>   live, so "gone" is trustworthy) AND under no registered library AND under no linked-universe
>   root. Verified live: `E:\Cognitive Knowledge` is a readable tree (8 entries) whose specific indexed
>   `.md` files no longer exist — drive up, files genuinely gone.
>
>   **Whole-Ecosystem tables to prune together:** note_meta, note_links, note_aliases, note_body,
>   sky_nodes, review_schedule, note_embeddings, notes_fts (via triggers), tag_counts (decrement)
>   — one funnel per path, ideally reindex_delete_note itself.
>
>   **Status: DESIGN, not built.** Destructive + safety invariant + offered-with-receipt
>   (cross-surface) → panel before any code or ruling (Panel-Speaks-First), then a plan to the
>   Boss. He selected it as the next work 2026-08-24.
>
>   **CONFIRMED AGAINST THE LIVE DATABASE 2026-08-23** (read-only, app running, nothing
>   disturbed): **601** searchable phantom rows, and **9** Linked-Universe rows (PJ-368) —
>   identical to the backup figures, so neither is a backup artefact. My own open item to verify
>   live is closed.
>
> ### 🛰 PJ-370…373 — the federation-honesty cluster (from the PJ-360 Architect pass)
> Filed under SO#9 because they surfaced in the Architect investigation and would otherwise have
> lived only in a session log, which is the miss the ledger exists to prevent. All four are
> preconditions for PJ-360 Phase 1 being *honest* rather than merely *scoped*.
>
> - **PJ-370** *(MED · Group 2 · federation)* — **unlinking a universe does not detach it.**
>   `remove_child_universe` skips `invalidate_search_state` exactly as `add_child_universe` does
>   (PJ-366), so an unlinked universe stays ATTACHED and keeps returning search results for the
>   rest of the session. The link direction leaves you unable to find notes that are there; the
>   unlink direction leaves you finding notes you have deliberately disconnected — which is the
>   worse of the two, because the user has explicitly said they no longer want that universe in
>   scope. One re-attach door serves both (see PJ-366).
>
> - **PJ-371** *(MED · Group 2 · federation · honesty)* — **the federation warning badge cannot
>   fire in time on his machine.** Warnings are polled at boot and once more at +3s, on an
>   in-code assumption that attach takes "tens-to-low-hundreds ms" — while `init_db` alone runs
>   ~15s on his universe, so the poll routinely completes before any warning exists. The
>   `federation:ready` handler re-fetches sky, graph, core, links, five-acts and bases, and never
>   re-fetches warnings. Net effect: the one channel that would tell him a universe failed to
>   attach is structurally unable to speak on the machine it matters on.
>
> - **PJ-372** *(MED · Group 2 · federation · honesty · LIVE ON HIS MACHINE)* — **a Linked
>   Universe whose folder has moved or been deleted is dropped SILENTLY, before the warning layer
>   exists.** `resolve_libraries_recursive` filters declared children through
>   `fs::canonicalize(...)` and `continue`s on failure, so a dead declaration never reaches
>   `federation_get_warnings` at all. **This is not hypothetical: `كون عيسى` currently declares a
>   child at `E:\Constellation Universes\Two universe UNIVERSE\Two Universe UNIVERSE`, which
>   does not exist on disk — verified 2026-08-23.** He has a broken federation link right now and
>   the app has never told him. Fix shape: a dead declaration is a WARNING, not a silent skip.
>
> - **PJ-373** *(MED · Group 2 · federation · silent-empty)* — **`verify_schema` checks five
>   `note_meta` columns and nothing else** (`path, name, library_name, created_at, modified`). It
>   never checks `notes_fts`, `note_links`, `note_embeddings` or `tag_counts`. A Linked Universe
>   with a stale schema therefore ATTACHES, counts as ready, and every query against it returns
>   an empty branch that is swallowed to zero rows — indistinguishable from "that universe has no
>   matches". Deepen the check, or make an empty branch from an attached universe reportable.
>
> - **PJ-374** *(LOW · Group 3 · federation · attribution)* — **`SearchResult` carries no universe
>   identity**, only `library_name`, which can collide across universes. So even on the ONE path
>   that does federate today, a result cannot say which universe it came from, and two libraries
>   sharing a name are indistinguishable. Prerequisite for PJ-360 Phase 1's coverage line, and
>   for the Boss ever being able to tell where an answer came from.
>
> ### 🔊 PJ-375 — the disk noise he heard, identified. NOT the loop.
> - **PJ-375** *(HIGH · performance · measured on his machine · Group 1)* — **`federation_prewarm`'s
>   FTS5 `optimize` on a Linked Universe runs for up to 78 SECONDS, repeatedly, and the code says
>   it should not.** The Boss answered YES to "did you notice the disk working continuously for
>   minutes while doing nothing" — the runtime observation I could not make myself. The evidence
>   says it is not the re-arm loop I feared: `links_backfill`'s repeated full recomputes in that
>   universe are from JULY, days apart, one per session, and today's log shows none at all. The
>   backstop never fired because it was never needed.
>
>   What it IS, from his own `diagnostics.log`: `[federation-prewarm] … FTS5 optimize OK in
>   **54084ms**` and `… in **77988ms**`, both today, against `Eisa Cognitive Knowledge`.
>   All-time in that log: **130 runs, 13 over ten seconds, worst 78s, 510s of disk churn total.**
>   The in-code comment says the optimize is "expensive (~30-60s for 7650 docs); on subsequent
>   invocations" cheap — the measurement contradicts it. Whatever the intended idempotency is, it
>   is not holding.
>
>   **Why this now outranks most of the queue:** it fires on the FEDERATED path, on every open of
>   a universe that has a Linked Universe attached — which is exactly the working pattern he has
>   just said he is adopting. Today it costs him a minute of thrashing per session; from tomorrow
>   it costs him that every time he works the way he intends to.
>
>   **INVESTIGATED 2026-08-24 (Reproduce-First). Three findings, and one honest unknown.**
>
>   **1. The fragmentation probe has never worked — not once in 130 runs.** The code reads
>   `SELECT MAX(segid) FROM notes_fts_data` before and after the optimize, "so we can see
>   optimize's effect in the diag log." That table's columns are `id` and `block` — **there is no
>   `segid` column.** The query errors every time and `.unwrap_or(-1)` swallows it, which is why
>   every line in his log reads `(segid -1 → -1)`. That is not "no change"; it is the probe
>   failing twice. **The instrument built to reveal this problem was itself broken and silent** —
>   the Charter's "a comment that became false" class, and the reason nobody noticed for months.
>
>   **2. A repeat optimize genuinely IS a no-op — the comment is right about that.** Measured on
>   a copy of his 2 GB `Eisa Cognitive Knowledge` index: three consecutive optimizes, **0.0 s
>   each**. So the 25–78 s runs are not a broken optimize; they are real merge work on an index
>   that was genuinely fragmented at that moment.
>
>   **3. Elapsed time does NOT predict the cost, which kills the obvious explanation.** From his
>   log: 0.0 s after 12.8 h, but **54.1 s just 17 minutes after a 34.4 s run**. So "it only
>   happens after a long gap" is false — the index can be re-fragmented heavily within minutes.
>
>   **THE UNKNOWN, named rather than guessed: what re-fragments it.** The obvious candidate is
>   that `Eisa Cognitive Knowledge` is not a read-only Linked Universe at all — it is his DAILY
>   universe, which he also opens and writes directly, and the prewarm's whole design assumes a
>   child that sits still. But I have not established that from evidence, so it is not a
>   conclusion. **The next step is to establish it, not to act on it.**
>
>   **Deeper investigation 2026-08-24 — three MORE hypotheses raised and FALSIFIED, so the
>   mechanism is still not identified and is recorded as such rather than guessed:**
>
>   - *Falsified: "the duplicate-cid self-heal re-writes the index each boot."* There are 11
>     distinct duplicate cids (one logged 212×), and the self-heal DOES write (`do_upsert("")`,
>     search.rs:8537). But 11 notes cannot produce 30–78s of FTS merge, and the pre-optimize
>     boot activity is near-identical before a 0s run and a 78s run (same duplicates, same
>     `reindexed=0`).
>   - *Falsified: "Eisa Cognitive Knowledge is also his daily universe, written directly."* It
>     is independently openable (own universe.json), so the theory was plausible — but its OWN
>     diagnostics.log has **zero** lines in the 10-hour window between a 0s optimize (08-23 18:45)
>     and a 78s one (08-24 05:29). It was not opened directly in that window, yet its index still
>     fragmented enough to cost 78s.
>   - *Falsified: "it fragments only after a long idle gap."* Already dead from the 54s-run-17-
>     minutes-after-34s data above.
>
>   **What IS now established (search.rs:11690–11760):** the prewarm opens the cUniverse db with a
>   plain read-WRITE `Connection::open` and runs `optimize`, which is a genuine FTS segment
>   rewrite — so the prewarm itself writes the index on every boot. The cost is real merge work;
>   what is NOT established is why `optimize` does not converge to a cheap steady state across
>   boots, given that three back-to-back optimizes in one process ARE free (#2).
>
>   **The honest close: I falsified every mechanism I could construct from the available
>   evidence, and did not manufacture one to fill the gap.** The next step requires evidence I
>   cannot get from logs — instrument the actual `MAX(segid)`-equivalent (via the REAL FTS5
>   introspection, `fts5vocab` or `notes_fts_data` row counts, since PJ-375 finding #1 proved the
>   current probe reads a non-existent column) before and after each boot's optimize on his live
>   machine, across several sessions, to see whether the segment count genuinely climbs between
>   boots and what it climbs in response to. That is a Boss-machine measurement, not a code read.
>
>   **Fix direction, unchanged and still sound regardless of the unknown:** repair the broken
>   probe (finding #1) so the code can DECIDE whether to optimize; #2 proves the skip path is
>   free, so a working probe caps the worst case at one honest merge when genuinely fragmented,
>   not one on every boot. Do NOT remove the prewarm. **The probe repair is designable now; the
>   convergence question is the measurement above and must precede any change to the optimize
>   cadence itself.**
>
> ### 🧹 PJ-376 — an error banner from one universe survives into another
> - **PJ-376** *(MED · Group 2 · cross-universe UI)* — **the rename-refusal banner persisted
>   across a universe switch.** His cleanup screenshot shows the "Container could not be renamed…"
>   banner still displayed while the app is in `Eisa Cognitive Knowledge` (19 libraries, 7,497
>   notes) — an error about a folder in a throwaway universe he had already left. `templateActionError`
>   is not cleared when the active universe changes, so a message about universe A is shown as
>   though it applies to universe B. Harmless today, misleading in general, and precisely the
>   kind of cross-universe state leak the Universe-of-Universes work is meant to eliminate.
>   Fix shape: clear the banner (and any other universe-scoped transient UI state) on switch.
>
> ### 🔬 PJ-321 — FOURTH corroboration, and the first CONTROLLED one
> The Boss CREATED TWO UNIVERSES today through the Universe Manager ("Folder Guard Test" and
> "Folder Guard Inner"), SWITCHED to one, worked in it, and triggered a refusal inside it. The
> registry afterwards: **277 bytes, mtime 2026-08-07, one entry (`كون عيسى`), sha256
> c20f9694… — byte-identical to every previous observation.**
>
> The three earlier corroborations were passive (the file did not change while the app ran). This
> one is a **controlled experiment**: the single action most certain to write a universe registry
> — creating a universe — was performed twice, and the file did not change. Meanwhile the app
> plainly knew about both universes: it listed them, switched to them, and titled its window with
> one of them.
>
> **The STOP order holds: observed, not diagnosed.** I did not go looking for where the app
> actually keeps this state, because that is the investigation PJ-321 reserves. Recorded as
> evidence and left alone.
>
> *(Cleanup note: this is also why removing the throwaway universes needed no registry edit —
> they were never in it. The folder was deleted; 25 files, one of them the auto-generated Five
> Acts note, nothing of the Boss's.)*
>
> ### Also this close
> - **PJ-300 escalation note**: three B1-cycle findings (pass-2 fallback degradation, the
>   routed-folder-fence blindability, the gate loops) all traced through the SAME degraded
>   session cache PJ-300 already files — its "needs its own pass" ranking is reinforced.
> - Stage obligation carried: the FULL Editor-Surface Gate checklist on a federated note (owed
>   to B6). The 2026-08-15 incomplete sweep scopes (notemodel-ownership,
>   cross-window-integrity, freeze-and-leaks) were covered by this sweep — that Charter item
>   can close.
>
> Ledger delta: B1 closed, Boss-validated and committed (`3f0f06a7`); **33 filed
> (PJ-344..376)**, plus a fourth and first-CONTROLLED PJ-321 corroboration, **of which PJ-358 is withdrawn by its own
> author within the same close** — 16 live; ► names PJ-344 (ruling owed) then B2, with
> **PJ-360 going to the panel first** as the most consequential filing of the day.

---

> **What changed in v1.95** (**MIG-111 Stage B6 SHIPPED, BOSS-VALIDATED — renaming a note inside a
> Linked Universe now works end-to-end; the fences hold for third universes**):
>
> **► NEXT ACTION — MIG-111 Stage B1** (the backfills' `recompute_*` FUNCTIONS take the registry
> from their callers' pinned scope — links_backfill / incoming twin / sky_backfill /
> name_fold_backfill, per the plan's B1 clause; the census annotations placed in B4 point at every
> site), then **B2** (DDL generation takes the vocabulary explicitly), then **B7** (pin the watcher
> fence with a test). Beside the stage: **PJ-326..331** queued; **PJ-341 + PJ-342 + PJ-343 Boss
> rulings owed**.
>
> ### ✅ MIG-111 Stage B6 — SHIPPED, Boss-validated 2026-08-23 (Stage 1 + Stage 2 A/B passed)
> The rename fences came down for the OWNER's own universe: `update_links_on_rename` resolves the
> owner once (`owner_scope_of`) and walks the OWNER's tree with the OWNER's vocabulary
> (seek disabled — disk truth); `rename_item_db_tail` + `rename_folder_db_tail` route their DB
> tails through **WriteScope (A3's first production wiring)** — migrate + rename-alias + reindex +
> maintenance land in the owner's own search.db. Third universes are fenced by the pure
> `routed_cascade_fence` rule (every universe root the active app knows, minus the owner and its
> ancestors). The Boss validated the headline (both link forms healed inside the Linked Universe,
> his own `inspires::` intact), the Phase-3 boundary (parent referrer text unchanged, click still
> resolves via the owner-stamped alias), and the own-universe regression. **Inspection cycle: 7
> diff-scoped passes, 16 CONFIRMED findings, 16 FIXED, zero parked** — incl. the pre-existing live
> wrong-DB alias write, the detached-tail switch races (a class fix inside `reindex_single_note`),
> strict owner attribution, and a latent clipboard-image overwrite. The tutorial pipeline caught a
> half-built exe (stale frontend bundle) before it reached the Boss.
>
> ### Filed this close (SO#9)
> - **PJ-343** *(LOW · wording · panel-recommended filing)* — the cascade-refusal error surfaces
>   as "could not be renamed" beside a rename that visibly succeeded (disk-first policy: the FILE
>   renames even when the link update refuses). Reword the surfaced message to say the rename
>   succeeded and the LINK UPDATE was refused, naming the universe. Group 4.
> - **Stage obligation recorded (not a PJ):** the FULL Editor-Surface Gate checklist on a
>   federated note (all 8 items) remains owed to B6 — the Boss test exercised the rename /
>   linked-probe-pair shape (item 6). Schedule before Stage B closes; sequencing is the Boss's.
> - **PJ-321 bundle: third corroboration** — the registry file byte-identical again after the B6
>   run's ~6 switches (hash c20f9694… unchanged since 2026-08-07). The STOP holds.
>
> **PJ ledger reconciled at the close of this job per SO#9** — B6 closed as shipped inside
> MIG-111 Stage B; 1 filed (PJ-343) + 1 stage obligation recorded; the ► line now names B1.

---

**Version 1.94 | 2026-08-22 (evening)** *(superseded by the v1.95 preamble above)*

> **What changed in v1.94** (**MIG-111 Stage B5 SHIPPED, BOSS-VALIDATED — the rename cascade
> carries the OWNER's vocabulary; the fences stay up until B6**):
>
> **► NEXT ACTION — MIG-111 Stage B6** (the fences come down for the owner's own universe, in
> their OWN commit: the SEEK-branch refusal and the `&foreign` exclusion, for the owner's universe
> only — a rename still refuses to cross into a THIRD universe (Phase 3 / R23). Verification per
> the plan: red→green on a child-only-type link (`[[refutes::Old]]` in a linked universe renames
> correctly where before B5 it was left broken) + the **Editor-Surface Gate checklist on a
> FEDERATED note**, including the linked-probe-pair shape (item 6)). Beside it: the PJ-326..331
> job stays queued.
>
> ### ✅ MIG-111 Stage B5 — SHIPPED, Boss-validated 2026-08-22 (all six test steps passed)
> `update_links_on_rename` resolves ONE owner registry per cascade (`registry_for_owner_of`,
> `?`-propagated) and threads it through BOTH branches into `rewrite_candidates` — ending the
> per-FILE process-global read inside the rayon closure (which could split one rename's rewrites
> across two vocabularies). Census: libraries.rs ABSENT. **Mutation-proved** (restoring the global
> read turns the new discrimination test red in 0.03s); fence-holds test added; suite 1537/0 ×3;
> diff-scoped inspection 1 pass / 0 findings (journal-verified). The Boss's live test used his own
> custom `inspires::` as the probe (panel-upgraded from builtin `supports::` — only a custom word
> can expose a wrong-list bug on screen) and both link forms healed exactly.
>
> ### Filed this close (SO#9)
> - **PJ-341** *(Boss ruling owed · federation UX)* — **the renamed-note-with-stale-links shape.**
>   B5's panel found (BLOCKING register correction) that `resolve_owner` verifies every declared
>   linked universe is readable BEFORE the active arm returns — so every rename runs that check
>   today (كون عيسى declares one child). If a linked universe's folder is unreadable, the note
>   renames but the cascade REFUSES naming the universe — referring notes keep the old title until
>   the cause is fixed and the rename redone. Deliberate (refuse-never-guess); the open question is
>   whether it deserves a retry/repair affordance. Group 3.
> - **PJ-342** *(product question · ties to the Backup & Recovery concept paper)* — Delete's
>   residue has no viewer: `.trash` (created on first delete, never shown in-app) and the
>   permanent MIG-104 delete-archive ledger record. Both deliberate; neither reachable from any
>   screen. Disclosed to the Boss in the B5 register; whether they get an in-app trash/history
>   viewer is his product call. Group 3.
> - **PJ-339 note** — the tension-status own-universe-gate question remains deferred to PJ-340's
>   design (recorded in both B4 and B5 panel registers so it cannot be lost).
>
> **PJ ledger reconciled at the close of this job per SO#9** — B5 closed as shipped inside
> MIG-111 Stage B; 2 filed (PJ-341, PJ-342); the ► Next-action line now names B6.

---

**Version 1.93 | 2026-08-22** *(superseded by the v1.94 preamble above)*

> **What changed in v1.93** (**MIG-111 Stage B OPENS with B4 SHIPPED, BOSS-VALIDATED — and a new
> top-principal ruling: the Universe of Universes**):
>
> **► NEXT ACTION — MIG-111 Stage B5** (the rename path takes the OWNER's vocabulary:
> `rewrite_wikilinks_in_text` + `update_links_recursive`; **the federation fences STAY UP in that
> commit** — B6 lowers them in a SEPARATE, later commit, never the same one). Beside it: the
> **PJ-326..331** job the Boss scheduled after Stage A remains queued.
>
> ### ✅ MIG-111 Stage B4 — SHIPPED, Boss-validated 2026-08-22 (all four test steps passed)
> Read-side analytics take an explicit `LinkTypeRegistry`. The seven ambient
> `structural_not_in_clause` sites answered or threaded (cache ×3 per-schema · sight active-by-
> construction · tension active-by-scope · the two SQL generators take `&LinkTypeRegistry`), plus
> three same-concern surfaces per the Whole-Ecosystem Fix Law: the 360 Inspector, the strata walk,
> and `scan_library_links` now resolve the OWNER's registry once per walk (`registry_for_owner_of`,
> with a pre-MIG-108 legacy-layout fallback); `scan_links_recursive`'s per-directory global re-read
> is gone. **`detect_tensions` now genuinely refuses non-own libraries** — the refusal its own
> comment had claimed since MIG-075; the Health tab renders its honest "Analysis unavailable" state.
> The census map carries every site's answer. **Per-build inspection: 3 passes, 7 CONFIRMED
> findings (3 MED / 4 LOW), ALL 7 FIXED before commit** — incl. two pre-existing sky back-fill
> defects (the wipe TOCTOU + the dropped re-arm) and the author's own federation-read designs
> corrected twice. Gates: Rust 1535/0 × ten runs (LL-050 fresh-relink applied once), vitest
> PJ-172 flake only (zero TS in the diff), release exe 11:50:18 verified to contain the fix.
>
> ### ⚖️ NEW TOP-PRINCIPAL — "Universe of Universes" (Boss-dictated 2026-08-22, at the B4 pass)
> *"…linking between notes, from the main universe and any linked universe(s), NOT to keep each
> universe in its own cocoon… Constellation should be a Universe of Universes in every aspect and
> concept."* Written into CLAUDE.md this commit. Generalizes the 2026-07-05 ONE-universe resolver
> ruling to EVERY aspect and concept: an honest "unavailable for a Linked Universe" is a stopgap
> that carries an obligation; the federated form of each feature is owed. Write sovereignty
> (MIG-111), MIG-108 layout, and the move-refusal are boundaries, not cocoons — they stand.
> **PJ-340 below is this ruling's first named deliverable.**
>
> ### 🔬 PJ-321 — evidence bundle recorded; the STOP holds
> The B4 test doubled as a controlled observation: the registry file received **zero writes**
> across register + two switches + relaunch on the new binary — byte-identical to the pre-run
> snapshot (277 B, mtime 2026-08-07, sha256 c20f9694…) — while sibling files in the SAME
> directory (write-journal.jsonl, app-prefs.json) are recently written, and `registry_path()`
> provably computes exactly this path. The Boss's next normal launch opened كون عيسى — which is
> what the STALE file's `active_id` points at. Snapshot + hash manifest:
> `lab/reports/pj321-evidence-snapshot-2026-08-22/`. **No mechanism asserted; no diagnosis; the
> instrumented reproduction starts from this bundle.**
>
> ### Filed this close (SO#9 — four new PJs)
> - **PJ-337** *(LOW · federation UX)* — a Linked Universe whose vocabulary file cannot be read is
>   SKIPPED on the per-note federated reads (Backlinks/Outgoing) with a notice that today reaches
>   only stderr + diagnostics.log (deduped). Owed: a panel-visible degradation hint — an IPC-shape
>   + UI decision (inspection pass-3 residual; pass-1's blanking fix chose skip deliberately).
>   Group 4.
> - **PJ-338** *(LOW · i18n wording)* — `tensionPanel.unavailable` says "switch tabs and back to
>   retry", which misleads for the PERMANENT Linked-Universe condition (retry always shows the
>   same message). Reword for both transient and permanent cases; ×15 locales. Group 4.
> - **PJ-339** *(MED-watch · tension surface)* — `note_tension_status` has NO own-universe gate;
>   it is honest for linked notes today only because its lookup is path-keyed (no row ⇒
>   `indexed:false`), i.e. by accident of branch order — the same wrong-universe concern its
>   sibling `detect_tensions` now refuses. Panel declined to set scope; filed here — fold into
>   PJ-340's design rather than patching twice. Group 2.
> - **PJ-340** *(FEATURE · the Universe-of-Universes ruling's first deliverable)* — **Federated
>   Knowledge Health**: the reserved MIG-063 family is elevated from "reserved" to "owed". The
>   Health tab's end-state for a Linked-Universe note is a REAL analysis performed against that
>   note's own universe (its own rows, its own vocabulary — the B4 machinery makes this
>   expressible), not the honest-unavailable stopgap. `/migration`-sized: touches read scopes,
>   the tension engine's name-keyed model, and the panel contract. Group 3, top of the group.
> - Also recorded: the B4 sentence for the Health panel exists in the EN help topic + EN manual;
>   the 14 translated manuals have NO Knowledge Health section at all to amend — that gap is
>   **PJ-336**'s existing scope, noted here so it is not double-filed.
>
> **PJ ledger reconciled at the close of this job per SO#9** — B4 closed as shipped inside
> MIG-111 Stage B; 4 filed (PJ-337..340); Group 3 re-ranked (PJ-340 tops it); the ► Next-action
> line now names B5. The v1.92 in-body next-action block below is superseded by this preamble.

---

**Version 1.92 | 2026-08-17** *(superseded by the v1.93 preamble above; ► lines below are historical)*

> **What changed in v1.92** (**MIG-111 Phase 1.2 OPENS — Architect + Plan done and Boss-approved. The Architect step found a defect that Phase 1.2 would have turned into a silent app-killer, and the Boss ruled it out as its own job first: PJ-302, FIXED. PJ-303 filed and FIXED — the test suite was not deterministic. The Architect doc was itself WRONG about the blast radius, and the red test corrected it**):
>
> **► NEXT ACTION — MIG-111 Phase 1.2 Stage A** (`MIG-111-PLAN-1.2.md`, steps A1…A8), ending with the `#[ignore]` removed from `routed_write_must_match_the_owners_vocabulary`. PJ-302 + PJ-303 await the Boss's test pass before commit. The Boss's ruling on **PJ-288** remains owed.
>
> ### ✅ MIG-111 Phase 1.2 — Architect + Plan (Boss-approved, 4 rulings taken)
> `MIG-111-ARCHITECT-1.2.md` + `-EVIDENCE.md` (1,808 lines; 14 agents) + `MIG-111-PLAN-1.2.md` (16 steps).
> - **The call-site count is 29, not the 26 the harness header claims** — and `sight.rs:113` is a false positive (`is_null_type` is a constant `matches!` that never reads the global). Amend the header when 1.2 lands.
> - **A connection-bound vocabulary is not expressible** — four of the six parse-chain readers are pure `&str → value` with no connection and no prospect of one, the deepest **five frames** below `index_note`. Whatever bundle exists at the top, below the connection layer the mechanism is a threaded `&LinkTypeRegistry`. Forced by the call graph, not chosen.
> - **Boss rulings:** approach approved as recommended · missing triggers ⇒ **REFUSE** naming the universe (PJ-232 stays closed) · the trigger defect is its **own PJ, fixed first** · **renames ARE in scope** (against the Architect's recommendation). Renames force a hard ordering: the vocabulary reaches `rewrite_wikilinks_in_text` FIRST (B5), the federation fence comes down SECOND (B6) — never the reverse, because `[[refutes::Old]]` under the wrong vocabulary silently fails to rewrite and breaks a link **on disk**.
>
> ### 🆕 PJ-302 — the foreign door STRIPPED a linked universe's triggers — **FIXED**
> `init_db_scoped` DROPped the `note_meta` sky trigger family unconditionally and recreated it only under `if owns`, so `federation::migrate::run_migrations_on` — reached whenever a linked universe's schema is stale, the ordinary state after an update — removed that universe's bookkeeping and did not put it back. Safe today only because of the sentence at `search.rs:5966-5968`, *"nobody writes through them, because the parent attaches a cUniverse read-only"* — **which Phase 1.2 is precisely the change that falsifies.** A routed write would then have produced no `sky_nodes` row, no stratum, no maturity, and `maintain_sky_after_save` could not repair it (it is an `UPDATE … WHERE path = ?1`; with no row it affects zero rows, returns `Ok(())`, and `maint.sky_failed` stays FALSE — **success reported**).
> **Fix, as a construction:** *the foreign door migrates SCHEMA and mutates NO trigger, in either direction.* Every trigger DROP/CREATE in `init_db_scoped` is now `owns`-gated. `InitScope`'s doc comment was false in **both** directions and is corrected: the `note_links_sky_ai/_ad/_au` block sat outside every gate while interpolating `snapshot()`, so the schema-only door **did** write registry-generated SQL into a foreign `sqlite_master` — harmless only because `structural_not_in_clause` is vocabulary-invariant by accident, not by the gate.
>
> ### 🔴 The Architect doc was wrong, and the red test caught it
> It claimed a parent-migrated child also loses its **outgoing-aggregate** triggers. It does not — `drop_outgoing_link_triggers` is the first line of `create_outgoing_link_triggers`, itself `owns`-gated, so that family is neither dropped nor created and **survives**. The true casualty list, printed by the test on its first run: `["note_meta_sky_ai", "note_meta_sky_stratum_au", "note_meta_sky_maturity_au"]`. I had asserted a blast radius from **reading** the gates instead of **executing** them, in a document whose own §7 warns against exactly that. Corrected in place, with the error recorded rather than overwritten.
> **Why the existing sibling test could not see it:** `schema_only_init_writes_no_vocabulary_triggers_into_a_foreign_db` starts from an EMPTY database, so `count == 0` passed *for the wrong reason*. The very next test's own doc comment already records the lesson for MIG-003 Step 1 — **a test whose subject cannot fire is not a test.** The new test seeds the triggers through a real owner-side `init_db` first.
>
> ### 🆕 PJ-303 — the test suite was NOT deterministic — **FIXED**
> Measured baseline: **six full runs → 1500/0 five times, 1499/1 once.** `arabic::fst_bake::tests::persist_then_try_load_cached_roundtrip` wrote a hand-built bundle to the **real per-machine cache path** and read it back, while the production initialiser `GenerativeFst::get` writes that same file on any cache miss — and `sample_bundle()`'s `[0xAA,0xBB,0xCC]` is not a valid `fst::Map`, so a miss is guaranteed. It also **deleted the developer's real Arabic cache on every run**. Reproduced deterministically by forcing `get()` into the window. Fixed with path-taking cores (`try_load_cached_at` / `persist_best_effort_at`); the test now uses `tmp_path`, whose own doc already said *"Avoids stomping on the real user cache during tests"*. Same disease as LL-047, one layer out: a shared mutable resource with a window.
> **This mattered beyond itself:** a non-deterministic suite makes every red→green claim in this migration probabilistic, and 1.2's entire deliverable is one red→green test.
>
> ### 🆕 PJ-304 — the H1 harness contaminated the suite with the very hazard it documents — **FIXED (interim)**
> A SECOND flake, distinct from PJ-303, and visible only once PJ-303's fix stopped masking it. `vocab_harness::index_under_vocabulary` called `link_types::set_active` and **never undid it** — so the first harness test to run left a 9-type vocabulary installed for **every subsequent test in the process**. Not a race window: permanent contamination, with test-scheduling order deciding whether it bit.
> It bit two tests asserting the empty-sentinel rank `9` (= `cognitive_ids().len() + 1`, correct only for a seeds-only registry; a custom type makes it 10 — the 1/2/4 ranks are unaffected, because a custom type sorts *after* the seeds, so **only the sentinel moves**): `links_backfill::tests::backfill_populates_existing_rows` and `search::tests_mig066_outgoing::outgoing_aggregates_maintained_by_triggers`.
> **Proven PRE-EXISTING, not caused by Phase 1.2 work:** reproduced on pristine `main` at `857530f5` with every Phase 1.2 change stashed — a 12-run sweep failed on run 3 with exactly that pair. It also retro-explains the day's very first baseline run, which showed **2** failures whose names were never captured: there were two independent flakes all along.
> **The irony is the lesson.** The harness was committed *specifically* to constrain Phase 1.2's design against LL-047 — *never install context into shared state for a duration* — and it did exactly that to the suite.
> **Fix:** an RAII `RestoreVocabulary` guard restoring seeds-only on `Drop`, including on panic. **Stated honestly as interim, and weaker than a fix:** it shrinks exposure from "the rest of the process" to "the duration of the call", but a non-harness test running concurrently can still read the mutated global. That residue is exactly what LL-047 says cannot be closed while the vocabulary is ambient — **Stage A removes it structurally**, by threading the vocabulary so the harness stops calling `set_active` at all. The guard is deleted then.
>
> ### 🆕 PJ-305 — the Arabic-overrides tests used SHARED fixed temp folders — **FIXED**
> Six persistence tests built their fixture at a fixed name (`constellation-overrides-atomic`, `-roundtrip`, `-sorted`, `-missing`, `-malformed`, `-unknown-fields`) and each opens with `remove_dir_all` on it, so two `cargo test` processes on one machine delete each other's fixture mid-test. **The unique-path idiom already existed further down the same file** (`constellation_overrides_test_activate_{nanos}`); these six simply were not using it. Now `unique_tmp(label)` — process id + nanos. Third instance today of one class (LL-049: a test sharing a mutable resource), after a file (PJ-303) and a process-global (PJ-304).
> **Honest provenance:** observed while running two suites concurrently — i.e. I triggered it. It is a genuine defect (verified in source: fixed path + `remove_dir_all`), but it was **not** one of the baseline flakes.
>
> ### 🆕 PJ-306 — the `lens` clock-boundary tests are fragile by construction — **LATENT, not fixed**
> `lens::query::tests::note_at_exact_boundary_included_after` inserts a note at `now() - 14 days`, then asserts a query — which recomputes its OWN "now" — still includes it. Its own comment says *"allow ±2s for current_unix_seconds rounding"*, but **the assertion carries no tolerance**: one tick between the two clock reads drops the row. Same shape in `lens::tests::recent_captures_excludes_older_notes`.
> **Stated at exactly the confidence the evidence supports:** these failed only under an artificial double-CPU load I created by running two suites at once. **There is NO clean evidence they flake in a normal single-process run** — 16 consecutive clean runs did not reproduce them. Filed as latent, not as an observed flake. **Not PJ-172**, which is a *vitest* file (`tests/sight-v6/perf.test.ts`) — different language, different location.
> Fix shape when taken: build the SQL first, read the threshold it computed, then insert the note exactly at that threshold — clock-drift-immune and still an exact-boundary test.
>
> ### 🆕 PJ-307 · PJ-308 — two MED silent-failure findings in `arabic/overrides.rs`, found by the diff-scoped inspection — **both FIXED**
> Neither was caused by this session's work (the PJ-305 change touched that file's **test module** only); both are pre-existing production defects the inspection surfaced because the file was in scope. Fixed in-pass per WA#6 rather than logged.
> - **PJ-307 (concurrency-race).** `set_active` took the write guard *inside* its swap statement — releasing it at the end of that statement — and stored the `ACTIVE_STORE_EMPTY` fast-path bit **outside** it, while its sibling writer `set_sovereign_layer` held its guard across both. **Two writers maintaining one invariant under two disciplines do not serialise.** Reachable: `set_active_universe` is `#[tauri::command(async)]` (runtime worker), `add_arabic_override`/`remove_arabic_override` are sync (main thread), and `switch_lock` serialises switch-vs-switch only. Either interleaving leaves the bit `true` over a **non-empty** store — the exact negation of the invariant documented on that static. `active_if_non_empty` then returns `None` from the atomic alone, so the FTS5 tokenizer stems every Arabic token as though no override existed, writing override-free stems into `notes_fts` — **silently**, with `active()` still returning the correct store so every `len()`/`layer_count()` diagnostic and the Settings panel look healthy.
>   **Fix:** extracted a single `publish()` that both writers call, holding the guard across bit-and-swap. The discipline now exists in **one** place, so "two writers, two disciplines" is no longer expressible — rather than repeating the ordering in both, which is the promise that was already broken once.
>   **The test written for it was DELETED, and this is the sharpest self-inflicted lesson of the session.** It failed on both counts: (1) it **did not reproduce the defect** — run against the pre-fix code it PASSED, because the interleaving needs a preemption inside a few-instruction window that 40 rounds × 120 writes never hit; and (2) **it broke its neighbours** — hammering the process-global store from two threads for the test's duration took down `set_active_replaces_prior_store_entirely`, `set_active_then_active_roundtrips`, `set_sovereign_layer_on_empty_active_creates_single_layer` and `set_sovereign_layer_preserves_child_layers` in **6 of 8** suite runs.
>   **I wrote a test that mutates shared state for a duration — in the same session, in the same file, as the fix for a bug whose entire nature is mutating shared state for a duration, and immediately after writing LL-049 about it.** The reason the fix stands is structural and readable, not a green test; the deletion and its reasoning are recorded in place at the call site so the absence is a stated decision rather than an oversight. A genuine red→green would need a test-only hook widening the window inside `publish` — a deliberate production change, to be decided openly rather than smuggled in behind a test that proves nothing. **PJ-307 therefore ships WITHOUT reproduction backing — a Reproduce-First exception that is flagged to the Boss, not assumed.**
> - **PJ-308 (false-success).** `reindex_arabic_overrides` was the ONE DB-touching command in that file without `ensure_search_db_ready`. `reindex_notes_matching_text` returns `Ok(0)` on a `None` connection (search.rs:13777) — byte-identical to the legitimate "no note contains this surface" — and the Settings panel renders any `Ok` as green success. `invalidate_search_state` NULLs `state.db` on every universe switch and the shell paints before the boot fan-out reinstalls it. So: add an override, be told the reindex completed, and every already-indexed Arabic note keeps its pre-override stems **forever**, because nothing re-runs it. Every sibling command already carried the line (libraries.rs:1712, :1753, :2634, :2709), each added by an earlier inspection against this same hazard. **Fix:** add it.
>
> ### ✅ PJ-302 — **BOSS-VALIDATED 2026-08-17**, on the strongest available corpus
> Tested on **Eisa Cognitive Knowledge** at the Boss's own suggestion — a better choice than the parent, and he was right about why: it is one of his two LINKED universes, so it is a universe that has actually been through the foreign door this fix repairs, on ~7,500 notes rather than 253.
> - **Stage 1 (the one that matters):** baseline **8023 nodes · 234061 edges · 7493 notes** → create one note → relaunch → **8024 nodes · 234061 edges · 7494 notes**. **Exactly +1 node, edges and MOCs unmoved** — the `note_meta_sky_ai` trigger firing on the active path after the change, across a full restart. (He named it `zamguon` because a `zarquon` already existed — the right call; a duplicate name would have made the +1 ambiguous.)
> - **Stage 2:** both linked universes present in the file tree with planet icons and bare counts (1 and 19); a note opened from inside **كون عيسى** with content and RTL intact; status bar **25 libraries · 9641 notes** — and the arithmetic confirms federation is genuinely counted (4 own + 1 universe-notes + 1 + 19 = 25), which per `aggregate_library_counts` only happens when the federated attach succeeded.
> - **Stage 3:** all four steps passed across both linked universes.
> **What this does and does not establish**, stated as it was to the Boss: opening a universe directly rebuilds its trigger set unconditionally, so a pass confirms the mechanism works and cannot prove the defect was ever present in his universes. That limit is inherent to the defect, not to the test.
>
> ### ✅ PJ-303 + PJ-309 — **BOSS-VALIDATED 2026-08-17.** Test 1 passed, then re-run after the PJ-309 fix and passed again, with a screenshot confirming it: no literal `<mark>`/`</mark>` anywhere, one clean single-strength highlight, Arabic reading normally. The screenshot also *demonstrates* why honouring the server's mark beat stripping it — in **الدحرجة** the highlight lands on **دحرج** INSIDE the longer word, which is the Arabic analyzer's real match and something a raw-query highlighter could never have located. **The `<mark>` defect was found by the Boss's eyes, not by any gate**: 1501 Rust tests, 997 vitest, svelte-check and four safety inspections were all green over it, because none of them look at rendered output.
>
> ### 🆕 PJ-309 — Search Hub snippets rendered literal `<mark>` tags — **found by the Boss's own PJ-303 test round; FIXED on his ruling**
> **Fix:** `highlightInText` now escapes first, then **restores the server's bare `<mark>`/`</mark>`** and, when the server marked the string, **skips the client pass entirely**. Two reasons the server's answer wins: it is the authoritative match position (FTS5 matched `دحرج` *inside* `دحرجة` through the Arabic analyzer — a raw-query regex cannot know that), and adopting it stops the hit being wrapped twice and rendering a double-strength background. Only bare tags are restored (none are ever emitted with attributes), so no other escaped text can become markup; a note whose body literally contains `<mark>` shows a highlight instead of the characters — inert, accepted, documented at the call site.
> **Whole-Ecosystem check:** the two other snippet surfaces are unaffected, verified not assumed — `RelatedCandidate.snippet` is a plain body excerpt rendered as text with no marks, and `IndexMention.snippet` uses a separate sentinel-char mechanism with its own `splitSnippet`. One producer (`search.rs:9051`), one consumer, one fix.
> Gates: svelte-check **0 errors** (268 warnings, baseline); vitest **996/997** — the single failure is `tests/sight-v6/tradition-perf.test.ts` at 19.37 ms against a 16 ms budget, the known **PJ-172** timing flake, confirmed **27/27 passing in isolation ×3**.
>
> ### 🔴 Original finding (kept for the record)
> Search دحرج and the Contents snippets read `ال<mark>دحرج</mark>ة` — the tag visible as text, *with* the highlight also applied. Traced end to end: Rust builds the snippet with HTML around the hit (`search.rs:9051`, `format!("{}{}<mark>{}</mark>{}{}", …)`, and FTS5 native `snippet()` on the other path), then the frontend runs `escapeHtml()` over the whole string before adding its OWN `<mark>` (`SearchHub.svelte:419-426`, `:425`). Escaping turns Rust's tag into `&lt;mark&gt;` — visible literal text — and the second highlight lands on top.
> **Pre-existing; nothing in this batch touches the frontend or snippet generation.** Not fixed in-pass because it carries a real design question rather than being mechanical: **Rust's mark is the AUTHORITATIVE match position** — in the Boss's screenshot it correctly identified دحرج inside دحرجة, which the frontend's raw-query regex would not reliably do for Arabic morphological matches. So the fix must *honour* Rust's marks (un-escape them after escaping the rest), not strip them, and it changes what the user sees — needing its own `tutorial-auditor` → `ui-inspector` round. **Boss ruling: fix now, or after Stage A?**
>
> ### 🆕 PJ-310 — `Open Existing Universe` may not refresh the window the way `Switch` does — **UNVERIFIED, filed rather than guessed**
> Surfaced by the `tutorial-auditor` while writing the PJ-302 Stage-3 steps, and filed because it was flagged rather than silently designed around. In `UniverseManager.svelte`, the **Switch** button calls `setActiveUniverse` and then the `onSwitch` prop — which the parent wires to `handleUniverseSwitch` (the full frontend refresh: library reload, stats, watcher, tab restore). **Open Existing Universe** calls `openExistingUniverse` and does **not** call `onSwitch`. Rust-side, `open_existing_universe` does set the active pointer and registry `active_id` in both its branches (`universe.rs:1212-1320`), so the backend switches while the frontend may not be told to re-derive.
> **Stated at the confidence the evidence supports:** it was NOT verified whether the frontend live-refreshes by some other route, so this is *possible* drift, not a confirmed defect. The Stage-3 test routes around it (prefer **Switch**; restart as the fallback) rather than asserting either way. Needs one read of the refresh path to settle. **Relevant to MIG-111**: a stale frontend after an active-universe change is precisely the class the Router work must not inherit. Group 2.
>
> ### 🆕 PJ-311 *(HIGH · freeze)* — a Search Hub query can trigger up to ~1,200 serial ONNX embeddings — **NOT fixed, needs a ruling**
> Found by the diff-scoped inspection on the PJ-309 frontend change — **pre-existing, not from that diff** (PJ-309 touched only `highlightInText`). Three legs, each verified in source by hand, not taken from the agent:
> 1. **The cap is PER CATEGORY.** `execute_universal_search` truncates titles/contents/tags/properties/wikilinks each to `limit` (search.rs:13541-13545), so `universalSearch(q, qEmbed, 200)` returns up to ~1,200 rows across six buckets.
> 2. **The summary `$effect` takes ALL of them.** `SearchHub.svelte:67-86` builds its path set from `allFlatResults` + `advancedGroups` with no slice and no cap, and the list is not virtualized. **Its own comment says "currently-visible result paths" — which is false**; `allFlatResults` is every result. Another comment asserting a property the code does not have.
> 3. **`nsc_get_summaries_for_notes` is get-or-COMPUTE, not read-only.** It loops SERIALLY over every path (`nsc/mod.rs:571-586`), each miss reaching `summarize_body` → the global `EmbeddingState.engine` mutex → an ONNX e5 batch, with errors swallowed to `eprintln!`. No cancel flag, no yield, no progress event, no cap — where its own sibling `nsc::backfill::run_backfill` has all four.
> **Reachable on the Boss's real corpus** (7,820 notes): a common-word search fans out to hundreds of cold notes, each a CPU-bound inference, invisible — `JobProgressStrip` watches only `nsc:backfill`, a different command. His دحرج searches did not hit it because they matched 3-4 notes.
> **Needs a ruling because the fix is a design choice, not mechanical:** make the search path **cache-read-only** (headlines appear only for already-summarised notes — my recommendation, since computing hundreds of summaries to fill a results list nobody asked for is the wrong trade), or cap/viewport-slice the request, or make it cancellable with a progress surface. **Group 1.**
>
> ### 🆕 PJ-312 *(MED · false-success)* — a failed search is indistinguishable from "no matches"
> `SearchHub.svelte:223` — a bare `catch` sets `response = null; filteredResults = []` with no message, no console line, and no wiring to the app's store-health surface. `constellation_search_universal` can legitimately `Err` ("Search index not available") during the boot/universe-switch window, and the user then sees an empty results area for a note they know exists. Pre-existing. **Group 1.**
>
> **Worth recording about the gate itself:** this inspection was arguably exempt under the standing order's "pure-UI tweaks with no write/index/lifecycle path" clause, and running it anyway surfaced a HIGH freeze. The exemption is more dangerous than it reads — a *file* in scope is not the same as a *diff* in scope, and the file is where the bugs were.
>
> ### 🆕 PJ-313 *(HIGH · index-corruption · REPRODUCED)* — adding an Arabic override CORRUPTS the search index — **NOT fixed, needs a ruling NOW**
> Found by the inspection the Boss asked for on the PJ-307 fix. `notes_fts` is an **external-content** FTS5 table with a custom tokenizer that reads the live override store per token. External-content `'delete'` requires supplying the original values so FTS5 can **re-tokenize and remove exactly the terms it added** — so the tokenizer must be the one that wrote them.
> The order is wrong: the panel `await`s `add_arabic_override` (which publishes the new store via `set_sovereign_layer`, overrides.rs:811) and only THEN calls the reindex (`ArabicOverridesPanel.svelte:145-149`). `reindex_notes_matching_text` (search.rs:13805-13823) then issues `'delete'` with the **NEW** tokenizer, removing a stem that was never in the index and **orphaning the old stem permanently**; the re-INSERT adds the new stem alongside it. `remove_arabic_override` is the exact mirror. Every add/remove compounds it.
> **The inspector reproduced this on a real external-content FTS5 table (SQLite 3.50.4):** after the mismatched delete+reinsert the old term still `MATCH`ed, appeared in `fts5vocab` with `doc=1`, and FTS5's own `integrity-check` returned **OK** — the corruption is invisible to SQLite's own checker. It also reproduced the escalation: once the affected note is later deleted, the orphan points at a missing content row and `snippet()`/`bm25()` — both on Constellation's search path (search.rs:8907) — raise **`database disk image is malformed`**.
> **The comment at search.rs:13806-13807 is the fifth false-comfort comment this session**: *"re-insert so the tokenizer runs again with the current ACTIVE_STORE in scope"* — right for the INSERT, silently wrong for the DELETE.
> **The guarding test cannot see it:** `override_and_reindex_flips_fts_token_set` (search.rs:14262) uses this same ordering and asserts only that the NEW stem is PRESENT — never that the OLD one is GONE. Green over a corrupting cycle.
> **⚠️ MY PJ-308 FIX INCREASED ITS REACHABILITY.** Before it, `reindex_arabic_overrides` could silently no-op (`Ok(0)`) when the DB was not ready; I added `ensure_search_db_ready`, so the reindex now reliably RUNS — converting "silently skipped" into "reliably performs the corrupting delete." PJ-308 was correct in isolation (a skipped reindex reported as success is a false success) and is **wrong to ship while PJ-313 stands**.
> **Fix shape:** delete-pass under the OLD store → publish → insert-pass under the NEW store. That means the reindex cannot remain a separate frontend call after the command; the ordering must be owned in one place. **Group 1 · ①.**
>
> ### 🆕 PJ-314 *(MED · index-divergence)* — linking or unlinking a cUniverse silently changes how every Arabic query stems
> `activate_layered_for_universe` (overrides.rs:720) stacks a child universe's override layer under the parent's on the next switch, with **no reindex and no signal**. The same tokenizer serves index writes and `MATCH` queries, so from that instant a query stems WITH the child's override while every stored row still holds the pre-override stem — a search returns **zero results for a word the user knows is in dozens of notes**. The CRUD path was given a reindex hook for exactly this hazard; this path has none, and `DriftReport` cannot see it (it compares mtime/row-presence and *"never compares content"*, reconcile.rs:610) — a tokenizer-verdict change moves no mtime. Permanent until a full rebuild. **Group 1 · ②.**
>
> ### 🆕 PJ-315 *(LOW · silent-degradation)* — a damaged `arabic-overrides.json` disables every override for the session, silently
> `activate_layered_for_universe` correctly propagates a parse error; its only production caller (universe.rs:1094-1097) swallows it into an `eprintln!` — and `main.rs` sets `windows_subsystem = "windows"`, so a release build has **no console** — then installs an empty store and returns `Ok(())`. The file is plain JSON the user is encouraged to sync via Git/Syncthing and hand-edit, so a merge conflict or half-synced copy reaches it with no bug on our side. Notes indexed during that session keep pre-override stems permanently, so repairing the JSON leaves a **permanent invisible recall hole**. Recoverable via Settings → Index full re-read — but nothing ever tells the user to run it. **Group 4.**
>
> ### 🆕 PJ-316 + PJ-316b — a partial FTS refresh reported as success, and the rollback my first fix forgot — **FIXED**
> `reindex_notes_matching_text` used to `continue` past a row whose FTS delete failed, count only rows whose insert succeeded, COMMIT unconditionally and return `Ok(count)` — so a note kept its PRE-override tokenization while the Settings panel painted a green "Reindexed N notes". **Reproduced before designing:** 3 rows, one failing statement, `Ok(2)` over 3 attempted, success shown, the untouched note still carrying its pre-override stems.
> **The previous panel called this "the only true app-killer anyone found", on the delete-succeeds/insert-fails "note vanishes from search" shape. That shape did NOT reproduce** — both statements carry the same payload so a value-based error hits the delete first, and the capacity route (`SQLITE_FULL`) also fails the COMMIT, which already propagated and rolled back safely. The demonstrated defect is the FALSE SUCCESS. Corrected the panel the same way it corrects me: by running it.
> **PJ-316b — my fix introduced something worse, and the panel caught it.** The abort path returned via `?` after a hand-issued `BEGIN IMMEDIATE`, with **no ROLLBACK anywhere in the function** — leaving the connection mid-write for the session, so every later `search.db` write looks saved, is invisible, and is **discarded at exit**: traversal counts, weights, confidence promotions, archived links. My comment claimed "`?` rolls everything back"; it did not — **the sixth false-comfort comment of the session, inside a fix for one of the other five.** This codebase had graded that exact shape HIGH ten days earlier and written `converge::commit_or_rollback` (converge.rs:364) so it could not recur; I had not looked. Now uses that helper and copies `run_tag_counts` (converge.rs:374-385) verbatim; the short-count check moved INSIDE the transaction so it can still roll back.
> **The test that missed it now pins it:** `conn.is_autocommit()` after the failure. Verified red→green twice — once on the false success (`Ok(0)` pre-fix), once on the open transaction.
>
> ### 🆕 PJ-310 — three doors half-switched the universe — **FIXED** *(re-scoped MED→HIGH by the panel)*
> `set_active_universe` is the only door that switches completely: pointer **plus** `invalidate_libraries_cache` (:1051), `invalidate_search_state` (:1061), `activate_layered_for_universe` (:1077). The two branches of `open_existing_universe` and `link_library_as_universe` moved the pointer and took the OS folder lock while calling **none** of the three. Without the middle one `ensure_search_db_ready` early-returns on `state.db.is_some()`, so **the connection to the PREVIOUS universe's `search.db` stayed open — the app reading and writing universe A's index while believing it was in B.** The class MIG-111 exists to end, shipped and reachable from the Universe Manager.
> **The shape came from the codebase, not from a new idea:** `create_universe`, the oldest and most-used door, never writes the pointer at all and leaves activation to its caller. The fix makes the other doors match the one that was already right.
> **Where I had to CORRECT the panel:** it prescribed deletion only, on the ground that "nothing downstream consumes them" — true in Rust, but tracing the callers showed `UniverseManager.handleOpenExisting` never called the correct door afterwards. Deletion alone would have turned "half-switches dangerously" into "does not switch at all". So: the three doors register only, **and** that one caller now routes through `setActiveUniverse` with the same departure flush `handleSwitch` performs (a switch that skips it can lose an unsaved edit in the universe being left). The other callers already did this.
> **The panel's exception confirmed with its mechanism, not on trust:** `migrate_legacy_data` KEEPS its pointer write, because `migrateLocalStorage()` runs between it and the real switch and invokes `save_universe_settings` / `migrate_universe_bookmarks` / `save_universe_workspaces` — all resolving through the active universe, each in a `catch` that keeps the local copy and logs. Remove the pointer and the user's settings, bookmarks and workspaces silently do not move.
>
> ### ⚖️ The harness fork — **RULED: DON'T** (and it should never have reached the Boss)
> Reproducing PJ-310 automatically would need a mocked `AppHandle`. The panel compiled the question: the concrete `tauri::AppHandle` appears **354 times across 57 files**; the generic form a mock requires appears **zero** times — so a mock harness reaches **zero** of the functions anyone wanted to test, and its true first step is a 57-file type migration. Its stated payoff was also already spent: the `resolve_owner` real-directory tests **shipped in `857530f5`**, two commits before the ruling that ordered them. Stage A's provability is unaffected — its acceptance harness (`federation/vocab_harness.rs`) already drives the real database and indexer with zero app-handle references. **I escalated a factual question as a judgement call; a four-line compile answered it.**
>
> ### 🆕 PJ-318 *(LOW · false-success)* — the panel's PJ-310 exception has a cost neither the panel nor I saw — **FILED, deliberately NOT fixed**
> Found by the diff-scoped inspection ON the PJ-310 fix. The panel ruled that `migrate_legacy_data` must keep its active-pointer write, and its stated reason is correct and independently confirmed (`migrateLocalStorage()` runs between it and the real switch, invoking three writers that resolve through the active universe). **What nobody traced is the consequence:** that pre-set trips the MIG-079 idempotence guard (universe.rs:882-895) on the caller's follow-up `setActiveUniverse(entry.id)`, which then returns `Ok(())` having done **none** of the activation work. The one materially skipped step is `ensure_universe_notes_folder` (:932) — which has exactly **two** call sites (:932, :1324), neither reachable on this path, and it is the ONLY code that registers the Universe-root-as-library entry (`is_universe_notes: true`). So for the whole first post-migration session, `libraries.json` lacks it, while the wizard reports success.
> **The inspector refuted its own escalation, and the refutation holds:** the path is gated on a cold boot with an empty registry, so `state.db` is None, the libraries cache is cold and no overrides are loaded — every other skipped step is a provable no-op — and the next boot (pointer `None`, guard cannot fire) repairs it. No data loss, no index divergence. **LOW.**
> **The right fix is a REORDER, not keep-or-delete:** move `migrateLocalStorage()` to AFTER `setActiveUniverse`, then delete the pointer write so all four doors match — satisfying the Whole-Ecosystem Fix Law properly instead of carving out an exception.
> **⚠️ MUST NOT BE CLOSED AS "deliberate and harmless" (panel, 2026-08-19).** Verified: `UniverseSetup.handleNext` calls `setActiveUniverse(entry.id)` **unconditionally**, outside the if/else (UniverseSetup.svelte:114) — so the divergence is LIVE on every legacy migration, not a theoretical branch. The carve-out is DEFERRED, not justified.
> **Deliberately not done in this pass**, and the reasons are stated rather than implied: it is LOW and self-repairing; it touches the **first-run legacy migration**, the highest-stakes and least-testable path an upgrading user has; it would **reverse a panel ruling**, which under the new law belongs to the panel, not to me; and the previous ruling explicitly forbade adding work to the front of this queue. **Panel ruling owed.**
>
> ### 🆕 PJ-321 *(UNKNOWN severity - observation only)* - the universe registry is not tracking reality, and **my first filing of this was WRONG**
> **RETRACTION, stated first because the wrong version was on record.** I filed this claiming `set_active_universe` never writes `registry.active_id` and never saves. **That is false.** It writes both, unconditionally, at the end of every genuine switch - `universe.rs:1102-1103`, from commit `c5b05f5c4` (2026-03-12). I concluded otherwise from an `awk` bounded to lines 897-1050; the write is at 1102, just outside the window I chose. **A truncated search reported as an absence** - the exact failure this session has been cataloguing, committed while filing a defect about the Boss's live machine. Caught by the `ui-inspector`, not by me.
>
> **What is actually OBSERVED, and remains unexplained:**
> - The registry file (AppData/Roaming/world.uconstellation.app/universes.json) is dated **08-07 09:56** and lists exactly one entry (kwn `كون عيسى`).
> - On 08-19 at **18:53:42** the Boss switched to that universe. The switch really happened - its own `.constellation/diagnostics.log` was written at 18:53:42, i.e. its search DB was initialised. **The registry file did not change.**
> - **`موسوعة عيسى`** (`E:/موسوعة عيسى`, ~1,073 notes, his real-life PKF, active earlier the same day) is **absent from the registry entirely**, although the registration paths do save it.
> - **2026-08-19, screenshot evidence from the Boss:** his **Universe Manager modal lists NINE universes** - كون عيسى (ACTIVE), Eisa Cognitive Knowledge, Eisa Universe, Constellation Test, Review Demo, Scratch, جوامع عيسى الشامسي, MIG108 Rehearsal, **and موسوعة عيسى**. At that same moment the file `list_universes` reads held **ONE** entry (277 bytes, mtime 08-07 09:56, re-read three times during the session including after his interaction).
> - Verified by direct reading, not by an agent: `UniverseManager.refresh()` calls `listUniverses()` -> `invoke('list_universes')` -> `load_registry` -> `app_data_dir()/universes.json`, and `load_registry` merges NOTHING (universe.rs:138-161). Process owner matches my user; identifier is `world.uconstellation.app`; only two `universes.json` exist anywhere under AppData and neither holds nine entries.
> **So the list the user sees does not come from the file the code says it comes from. Unresolved.**
> **Why this may be worse than cosmetic:** the write path (`load_registry_for_update`) loads that same one-entry file. If a registration writes it back, it persists ONE entry plus the new one - and the other eight the user can see in the modal would not be in the file at all. Whether they survive a restart is UNKNOWN and untested.
> - **2026-08-20, a CONTROLLED experiment, not an observation.** The Boss performed FOUR registry mutations in ~5 minutes: Remove (موسوعة عيسى), + Create New Universe, Remove again, then Open Existing Universe. Every one of those paths ends in `save_registry(&app, &registry)?` - an error-propagating write that CANNOT fail silently; a failure would have surfaced in the UI, and none did. **The file the code reads was unchanged throughout: 277 bytes, one entry, mtime 08-07 09:56.** A disk-wide sweep of AppData for anything written in the following 30 minutes containing a universe id or that name returned exactly ONE hit - a Windows Recent-files `.lnk` created by the folder picker. **No registry file was written anywhere findable.**
> - Also learned: **Universe Manager's Remove only UNREGISTERS; it does not delete files** (its own text says so). The `+ Create New Universe` run left `E:/موسوعة عيسى/موسوعة عيسى/` on disk - a full `.constellation`, a 462 KB index and one auto-scaffolded Five Acts note (`ensure_search_db_ready` -> `init_five_acts_system_notes`, search.rs:11819) - **nested inside his real PKF, contributing one row to its index.** Not a defect in the door PJ-310 fixed: `open_existing_universe` reads `universe.json` and errors when absent; it cannot create one.
> **STOP THEORISING. This has now produced two confident wrong explanations from me** (first: `set_active_universe` never saves - false, it does, at :1102; second: the file is authoritative - contradicted by a UI listing nine). The next person to touch this reproduces it under instrumentation or leaves it alone.
> Two independent facts that the code, as read, says should not both be true. **Mechanism UNKNOWN - do not close this as understood, and do not accept a second confident explanation without reproducing it.**
> **Why it matters regardless of cause:** boot activates from this registry, so his real-life universe is not in the list and must be re-opened by folder each time. **Group 1 - needs a reproduction, not another reading.**
>
> ### ✅ PJ-322 *(MEDIUM — **the reachable half FIXED 2026-08-20**; the rest re-scoped and open)* - the federation/registry readers report "nothing" when they mean "I could not look"
> **Panel-ruled 2026-08-20** (4 lenses + 4 independent skeptics + synthesis). **My reason for filing this instead of fixing it was factually wrong, and the panel proved it at file:line.** I wrote that fixing it "touches a DIFFERENT migration's journaled, snapshot-first engine … mid-cascade." It does not: `assemble_foreign_roots` has exactly two callers (`mig108.rs:2111` preflight, `:2349` `bring_in_library`), both already `Result`-returning `#[tauri::command]`s, and `mig108_execute` calls preflight at `:2231` **before** `Journal::new` at `:2232`. A refusal returns before a journal exists. **The reason I gave for parking it was the reason not to park it** — my fifth wrong structural call in three days, all the same shape: a confident claim made without opening the call graph.
>
> **An escalation to APP-KILLER was also REFUTED** (one lens claimed a boot dialog could propose relocating 8,031 notes). `set_active_universe` (universe.rs:960-980) resolves the id against the registry and errors when it misses, and it is one of only two writers of `active_path` — so at the moment `assemble_foreign_roots` runs at boot, the active universe is necessarily a registry entry. **Not live. No emergency, nothing for the Boss to do.**
>
> **The reachable hole was somewhere else than I filed it.** Not the child manifest — **`load_registry` (universe.rs:138-161), which returns an EMPTY registry on a path error (`:141-145`), a read failure (`:148-152`), and a parse failure (`:155-158`) alike.** An empty registry ⇒ empty `foreign_roots` ⇒ `foreign_reason` (mig108.rs) returns `None` for everything ⇒ **every** registered library that is a directory and not under the active root falls through `classify` to `EntryClass::Move`. Not "one child missing" — *no universe is foreign*, from one unreadable `universes.json`.
>
> **FIXED, the panel's minimum safe change — a structural guard, not the strict enumerator.** `classify` now asks the **disk**: a library that is, or sits inside, a directory carrying `universe.json` (or `.constellation/universe.json`) is `ForeignUniverse`, walking upward to the volume root. It reads a FACT, so it is immune to whichever of the four readers degraded; it is **monotone toward refusal** (it can only move an entry OUT of Move/Copy); and `classify` is a plain-data function with in-file unit tests, so red-first was reachable. Uses `fs::metadata`, **not `Path::exists()`** — `exists()` returns false for "absent" AND "unreadable", which would have reintroduced the same defect one layer down; an unreadable manifest counts as PRESENT. Shared predicate `carries_universe_manifest` now used by `bring_in_library` too (which had the `.exists()` bug), so the two surfaces cannot drift again. **4 tests, MUTATION-TESTED: disabling the guard turns 3 red and correctly leaves the over-refusal guard green.**
>
> **STILL OPEN, re-scoped by the panel — file under this number:**
> - **`load_registry`'s three-way collapse (universe.rs:138-161)** — the actual root cause, untouched. Its own doc comment already warns it is read-only and forbidden on write-back paths.
> - **The cache skew.** `mig108_preflight:2110` reads libraries from `load_all_libraries` (**cached**, libraries.rs:186-212) while `:2111` reads foreign roots **fresh**. A warm cache + a short fresh read = a complete library list with an incomplete foreign set in the same call. In a single fresh read the two degrade together and self-cancel; **the cache is what breaks that coupling**, and it is why the child-manifest half matters at all.
> - **The preflight error is INVISIBLE.** `Mig108UnifyDialog.svelte:103-106` catches it into `console.error` and never sets `visible`; devtools is dev-only in release. **A strict enumerator alone would convert silent misclassification into a silently absent proposal** — the same silent-failure class, relocated. Any strictness fix must ship with a visible surface. *(This is a Boss decision — see Housekeeping.)*
> - **`mig108_execute` re-runs preflight** at `:2231` against a plan the dialog computed earlier — a TOCTOU window.
> - **`remove_universe_from_registry` (universe.rs:1192-1200)** drops the entry and reassigns `active_id` but never clears `UniverseState.active_path`, so in-session the active universe can become unregistered. The boot dialog will not re-observe it, but `bring_in_library` will (invokable any time, calls `assemble_foreign_roots` at mig108.rs:2349).
>
> **NOT VERIFIED, stated rather than assumed:** the downstream behaviour of `run_db_rewrite` / `run_json_rewrites` on a mis-included entry — every lens stopped at the directory move, and so did the synthesis. **No reproduction exists for any part of this**, in the wild or otherwise.
>
> **UPDATE 2026-08-20, after the Boss's four rulings — the REPORTED half is now fixed too.**
> Boss decision 1: **"May the two directory-moving commands hard-fail on a transient registry read?" → YES.** Landed:
> - `universe::registered_universe_roots_strict` — reuses `load_registry_for_update`'s split verbatim rather than inventing a third policy: **`Unreadable` refuses** (transient; a retry succeeds), **`Corrupt` sets the file aside and proceeds from empty** — which is then TRUE rather than assumed. That split is the recorded scar at `universe.rs:120-126`; the first version of THAT fix returned `Err` for both and locked the user out of the app entirely.
> - `mig108::assemble_foreign_roots` is now `Result`-returning and uses BOTH strict readers. Propagated at its two call sites (`mig108_preflight`, `bring_in_library`). Confirmed safe by the panel: preflight runs BEFORE `Journal::new`, so a refusal can never land mid-cascade.
> - **The invisible `catch` is closed.** `Mig108UnifyDialog.svelte` gained a `blocked` state — title, plain-language body, the verbatim reason, and **Not now / Try again**. The retry exists because the refusal is usually transient. It is still never a gate: `dismiss` always releases the boot fan-out. 3 new i18n keys × 15 locales, **parity verified**.
>
> **Two independent guards now cover one concern, deliberately:** `classify` asks the DISK (immune to any reader degrading), and `assemble_foreign_roots` refuses when its REPORT is untrustworthy (catches what the disk walk cannot see).
>
> **Still open under this number:** the cache skew (`mig108_preflight` reads libraries from the CACHED `load_all_libraries` while foreign roots are read fresh — the cache is what breaks the self-cancelling coupling); the `mig108_execute` re-preflight TOCTOU window; `remove_universe_from_registry` not clearing `active_path`.
>
> ### 🆕 PJ-324 *(LOW — pre-existing, not caused by MIG-111)* - the Sight v6 render-budget tests flake on this machine
> **Measured 2026-08-20, both with and without the Stage-A changes.** `tests/sight-v6/perf.test.ts` asserts wall-clock budgets (`≤32 ms` for Hearst facet-count rebalancing on 7,636 notes; `≤16 ms` for others). On **pristine `main`, with nothing else running, 2 of them fail**; with the Stage-A diff, 3 fail; run in isolation, all 4 pass. **The count varies run to run and tracks machine load, not the diff** — verified by `git stash` on both sides.
> **Why it is filed rather than ignored:** a suite that fails 2-3 tests on a clean tree trains everyone to read "N failed" as noise, which is exactly how a real failure gets waved through. This is LL-049's subject (a green suite is a claim about a DISTRIBUTION) pointed at the frontend.
> **Not verified:** whether the budgets were ever met on this hardware, or whether something regressed the code they measure. **Establish that before touching either the budget or the code** — raising a threshold to silence a real regression is the failure mode here.
>
> ### 🆕 PJ-323 *(LOW — latent, not armed)* - `RunGuard` holds an owned `AppHandle` as a field
> `index_repair.rs` — `struct RunGuard { app: AppHandle }`, whose `Drop` requires it, constructed at one site inside the background repair worker. Same shape as the field removed from `WriteScope` (LL-050). **No test constructs it, so it is latent.** Align with the `WalkCtx` pattern (`search.rs`, `Option<&AppHandle>` + a pure split in `index_repair.rs`) when convenient. **Do not open a codebase-wide sweep** — the mechanism behind LL-050 is unproven, and two review agents ran that sweep, reported "population one, already compliant," and missed this very entry.
>
> ### ✅ CLOSED WITHOUT FILING - the missing `onnxruntime.dll`
> Investigated because it looked like a build-integrity defect. **Red herring, settled by measurement:** `onnxruntime.lib` in the ort cache is **305,821,070 bytes** — a static archive, not an import library. ONNX Runtime is statically linked (`DirectML.lib` and the ort `/LIBPATH` appear in the link line); **no DLL is expected and its absence is not a defect.** One review lens still recommended filing a PJ for it; that recommendation is refuted. Recorded here so nobody re-opens it.
>
> ### ✅ PJ-325 *(MEDIUM - FIXED 2026-08-20, same session it was found)* - the transient/permanent distinction was preserved and then thrown away one call later
> **Found by the concept panel, in code written the same morning.** `universe.rs::name_the_universe` deliberately preserves `PersistedError::{Unreadable, Corrupt}` under a comment stating that `Corrupt` is permanent and retrying is pointless - and `mig108::assemble_foreign_roots` then flattened both with `.map_err(|e| e.message().to_string())`. The unify dialog rendered the same card for both, telling the user *"this is usually temporary... Try again"* about a file that will never repair itself.
> **This is the Broken-Universe-Link concept's own thesis, operating inside the code written to serve it.** The best available evidence that the concept describes something real.
> **FIXED:** the kind crosses the IPC boundary as a machine-readable marker (`transient|` / `damaged|`, stripped before display - the frontend must never pattern-match a translated sentence to choose buttons); the dialog renders a distinct body for a damaged file; and **the Try-again button is not rendered at all** when retrying cannot work. New key `mig108.blockedBodyDamaged` x 15 locales, parity verified.
> **Also fixed in the same pass:** `universe.rs` carried a malformed user-facing sentence - *"Refusing to"* + 22 literal spaces + *"treat it as absent"* - which was **the only prose Constellation currently produces about a broken child link**, reaching the user verbatim through the unify dialog.
>
> ### 🆕 PJ-326 *(MEDIUM)* - you cannot unlink a child universe in the running app
> **Verified 2026-08-20.** A child universe can be ADDED from two places (`LibrarySwitcher.svelte:31`, `UniverseManager.svelte:128`). The only removal control is `handleRemoveChild` in `UniverseSetup.svelte:206` - the **first-run setup wizard**, reached only when no universe is configured or the last one was removed (`+layout.svelte:10605`). `UniverseManager.svelte:7` imports the `removeChildUniverse` wrapper and **never calls it** (checked for actual call sites, not just the import - the trap that produced a wrong conclusion earlier the same day).
> **Consequence:** a link to a universe that no longer exists cannot be retracted without editing `universe.json` by hand. This is why the Broken-Universe-Link concept v1 is **detection-only**: the most obvious button has no engine behind it.
>
> ### 🆕 PJ-327 *(MEDIUM)* - the federation warning badge can silently never appear
> **Verified 2026-08-20.** Warnings are fetched at boot and re-polled **once at ~3 seconds** (`+layout.svelte:3079-3106`). The `federation:ready` listener (`:3568`) re-fetches sky and graph but **not warnings** - the only three call sites of `getFederationWarnings` are `:2840`, `:3081`, `:3098`. **Any attach that settles after 3 s leaves the badge empty for the entire session**, so the one surface that reports an unavailable child universe is timing-dependent on a large universe.
>
> ### 🆕 PJ-328 *(MEDIUM)* - the federation reason string cannot be translated, and is an IPC-contract change
> **Verified 2026-08-20.** `+layout.svelte:10582` renders the raw Rust reason string with no `$t()`, and the `federation` i18n block has no key for reason text (only `warningBadge`, `popupTitle`, `cuniverseLabel`, `reasonLabel`). Any improvement to what that popup says ships **English-only in all 15 locales**, and any condition-derived buttons would have to pattern-match English prose to decide what to offer. **The fix is to cross the boundary as a code plus data, not a sentence** - the same shape PJ-325 just applied to the unify dialog.
> **Also here:** `LibrarySwitcher.svelte:131` hardcodes English `'library' : 'libraries'` outside `$t()`/`$tn()` - a live i18n break on the exact line this work would rewrite. And `UniverseManager.svelte:7` carries an unused import (PJ-326).
>
> ### 🆕 PJ-329 *(severity NOT ESTABLISHED)* - a dead child universe's libraries leak into the ACTIVE universe's library list
> **Found by the concept panel; in neither the concept paper nor two of its three reviews.** `resolve_libraries_recursive` loads a child's `libraries.json` at step 1 (`universe.rs:615-624`) **before and independently of** reading its `universe.json` at step 2 (`:634-651`).
> - **Manifest damaged:** the child's libraries load anyway, and `find_universe_root` (`attach.rs:112`) tests only that the manifest **exists**, never that it parses - so the child **attaches as though healthy and emits no warning.**
> - **No manifest:** the libraries still load into the merged list, the upward walk climbs past the child to the ACTIVE universe, and `attach.rs:88-91` discards the root. Net: **the dead child's libraries remain in the active universe's own library list with no database attached for them.**
> **NOT VERIFIED:** the downstream user-visible consequence across every consumer of that list. Every lens stopped at the trace. **Establish that before assigning severity** - and do not assume it is cosmetic.
>
> ### 🆕 PJ-330 *(LOW - open question, not adjudicated)* - a legacy-layout child universe may never be able to produce a warning
> `attach.rs:112` recognises a universe only by `.constellation/universe.json`, while at least four other readers also accept a legacy root-level `universe.json`. Raised by a panel skeptic; **the panel explicitly did not adjudicate it.** Trace before acting.
>
> ### 🆕 PJ-331 *(LOW effort, HIGH clarity - Boss ruling re-stated 2026-08-20)* - the app still calls a Linked Universe "cUniverse" and "Child Universe"
> **"We have decided to change the naming from 'cUniverse/Child Universe' to 'Linked Universe'. Have you forgotten?"** - Eisa, 2026-08-20.
>
> **The ruling was taken, never written into `CLAUDE.md` or the orientation doc, and was therefore lost.** Searched before answering: **"linked universe" appears 431 times across 75 files** as descriptive prose, so the term is long-established in practice - but `CLAUDE.md` still *defined* the level as "cUniverse (Child Universe)", and orientation v3.98 still carries a "canonical fact" line listing `cUniverse` among brand labels that "intentionally stay English." **The decision existed only in conversation, so the documents contradicted it and the panel, reading those documents, recommended the retired word.** Now recorded in `CLAUDE.md`, orientation **v3.99**, **Development Laws v1.5**, and memory.
>
> **A THIRD canonical document was found carrying the stale definition after the first report** - `docs/Constellation Development Laws v1.4.md` also defined it as "cUniverse (Child Universe)", with "linked Universe" appearing only as prose in the same sentence. Found by finishing a background search rather than stopping at the first answer. **All three canonical documents said the retired name; that is why no agent could have got it right from the record.**
>
> **The exact surface - measured, not estimated. 10 translated string VALUES:**
> `universe.setup.addChildUniverse` ("Add Child Universe") · `universe.setup.addLibrariesDescription` · `universe.setup.noLibrariesYet` · `universe.manager.children` ("Child Universes") · `universe.manager.addChild` · `secondScreen.dashboard.childUniverses` · `constellationMap.childUniverse` · `federation.warningBadge` ("cUniverses unavailable") · `federation.cuniverseLabel` ("cUniverse") · `styleSetter.labels.cuniverse`
>
> **Plus 2 user-visible labels that never went through `$t()` at all** (so they are English in all 15 locales today, independent of this rename): `ConstellationMap.svelte:271` - `node_type === 'child_universe' ? 'cUniverse'`, a hardcoded type label on the map; `StyleSetter.svelte:182` - `cuniverse: { name: 'cUniverse' }`, a Style Setter category name.
>
> **Scope discipline:** rename the **values** across 15 locales, not the **keys** (`addChildUniverse` etc. are not user-visible and renaming them is churn). Rust/TS identifiers (`add_child_universe`, `ChildUniverseInfo`, `resolve_child_universe_roots*`, `cuniverse_path`) are **out of scope** - optional and separate. **Do not rewrite historical records**: session logs and superseded orientation/ledger versions stay as written.
>
> **Related, and cheaper to do in the same pass:** PJ-328's `LibrarySwitcher.svelte:131` hardcoded English `'library' : 'libraries'`, which sits on the very line that renders the "Child Universes" section.
>
> **Priority: scheduled with the PJ-326..330 job after MIG-111 Stage A** (Boss accepted the panel's ordering, 2026-08-20).
>
> ### ✅ PJ-332 *(CLOSED 2026-08-22 · Boss-validated · HIGH · index-divergence · **PRE-EXISTING**, not introduced by MIG-111)* - the Sky back-fill thread has no universe identity
> **Confirmed by the 2026-08-21 diff-scoped safety inspection** (20 agents, refute-before-confirm). Candidate was raised as APP-KILLER and **downgraded to HIGH by its own verifier**, which explicitly declined to sustain the broader claim - recorded because a refuted escalation is as valuable as a confirmed one.
>
> `sky_backfill::run()` binds `app.state::<SearchState>()` once and **re-locks the SWAPPABLE `state.db` at every phase** (sky_backfill.rs :172, :183, :224, :313, :347, :378, :458), carrying **no path and no `federation_generation` token**, with no cancel check. Phase B (:301-305) is lock-free by design and performs up to 1000 `fs::read_to_string` calls - where nearly all the wall-clock sits. A universe switch inside that window (`invalidate_search_state` sets `*db = None` and bumps the generation; `ensure_search_db_ready` then installs the NEW universe's connection into the SAME mutex) silently redirects the thread.
>
> **Two independently reachable end states, both silent and permanent:**
> 1. The zombie thread's `finalize` (:456-471) stamps `schema_versions.sky` and DELETEs the cursor in the OTHER universe unconditionally on a drained batch. `is_needed` (:88) is version-only, so it returns false forever and that universe stays partially populated.
> 2. The zombie's `write_cursor` lands while the other universe's own thread is blocked behind its `ANALYZE`, so that thread's `read_cursor` picks up a foreign path and every note sorting below it is skipped - then stamped complete.
> Additionally **Phase E (:342-352) inserts `note_aliases` rows with NO existence guard**, writing one universe's paths into another's alias table, which `libraries.rs` and `map.rs` consult for wikilink resolution.
>
> **Nothing repairs any of this:** `index_repair.rs:415` states outright that nothing in the codebase rebuilds `sky_links`; `converge.rs:296` is a no-op on empty `sky_nodes`. Diagnostics log `[sky_backfill] completed`.
>
> **The guard already exists and every sibling uses it.** `ensure_search_db_ready` captures `init_gen` and discards on mismatch; `derived_heal.rs` re-checks the generation; `name_fold_backfill`, `links_backfill`, `incoming_links_backfill` and `review_backfill` each resolve a path ONCE and open their own connection. **`sky_backfill` is the lone outlier that reads the mutable "whichever universe is active NOW" handle.**
> **Fix shape:** capture `federation_generation` + `db_path` at thread start and abort on mismatch (or open its own connection like its siblings); add a completeness check before `finalize` stamps; add an existence guard to the Phase E alias insert.
> **Not fixed in this pass** - it is pre-existing, it is in a subsystem this diff only renamed one line in, and it is big enough to deserve its own reproduction. **Boss ruling owed on whether it precedes MIG-111 Stage B.**
>
> ### ✅ PJ-333 *(CLOSED 2026-08-22 · MED · index-divergence)* - `bring_in_library` does not ancestor-walk, so content nested inside an UNREGISTERED universe can be moved out of it
> **Confirmed by the same inspection.** PJ-322 gave `classify` an ancestor-walking manifest backstop (`universe_manifest_at_or_above`), and **deliberately did not give it to `bring_in_library`**, whose only structural guard is `carries_universe_manifest(src)` - the folder ITSELF. That choice is documented in the code as belonging to the Boss because widening it is a user-facing behaviour change.
>
> **The hole it leaves:** a second Constellation universe on disk that is NOT in this install's registry (synced from another machine, or removed from the list but kept). `assemble_foreign_roots` cannot name it, so `foreign_reason` answers `None` for every path under it. The user picks a plain subfolder inside it - which carries no manifest of its own - and Bring-In → Move succeeds. That universe's content is relocated out of it with no error.
>
> **Why it is MED and not higher:** it requires an unregistered universe on disk AND the user selecting a folder inside it. **Not verified:** what the other universe's own index does afterwards.
> **The fix is one line** - route `bring_in_library` through `universe_manifest_at_or_above` instead of `carries_universe_manifest` - but it changes what the app refuses, so it is **Boss's call**, exactly as the code comment says.
>
> ### ✅ PJ-334 *(CLOSED 2026-08-22 · Boss-validated · HIGH · index-divergence · **LIVE: 770 notes across 5 universes** · origin ESTABLISHED · fix RULED)* - indexed notes with no Sky node, and no path back
> **Full panel ruling: `docs/migrations/PJ-235-federation-boundary/PJ-334-PANEL-RULING.md`.** Four lenses, each adversarially attacked, read-only throughout. **It overturned the entry this replaces, corrected the author twice, corrected four of its own members, and established the origin.**
>
> #### The scope is 770, not 8
> | database | notes | sky rows | **missing** |
> |---|---|---|---|
> | **Eisa Universe** | 2,731 | 1,973 | **758 - 27.8% of that universe** |
> | Eisa Cognitive Knowledge | 8,031 | 8,024 | 7 |
> | Scratch / جوامع عيسى الشامسي / 3 nested كون عيسى | - | - | 1 each |
> | **موسوعة عيسى (the real PKF)** | 832 | 832 | **0** |
>
> **1,853 `sky_links` edges are drawn from nodes that do not exist.** Orphan sky rows: 0 everywhere.
>
> #### It is NOT only Sky View - it reaches the Reviewer
> `index_note` reads `SELECT CAST(stratum AS INTEGER) FROM sky_nodes … .unwrap_or(0)` (search.rs:8402-8406); `review.rs:1408-1412` does the same. **In every universe checked, `COUNT(review_schedule.stratum = 0)` equals the missing-sky count EXACTLY** (ECK 7/7, Eisa Universe 758/758, موسوعة عيسى 0/0). Every stranded note is parked permanently at the bottom of the review queue.
>
> #### ✅ ORIGIN - ESTABLISHED, and the leak is already sealed
> 1. Notes are indexed while `cid_cn` is still `''` (a bulk library import; the Five Acts system note written at universe init).
> 2. `note_meta_sky_ai`'s **`INSERT OR REPLACE`** collides on the then-**FULL** `UNIQUE INDEX idx_sky_nodes_cid_cn`. REPLACE = DELETE + INSERT, so **each cid-less note deleted the previous one's row - one survivor per cohort.**
> 3. The cid is injected later, which takes the **UPDATE** branch: `note_meta_sky_ai` never fires again, `note_meta_sky_au` only UPDATEs a row that is not there. **Permanent.**
> 4. **`2edc97d7` (2026-08-10 11:30:58 +0400)** made the index partial - sealing the producer - and added the §15 restore scoped `WHERE m.cid_cn = ''`. It healed everyone still blank (ECK 25, Eisa Universe 231, all sharing one `updated_at` second: **2026-08-10 05:06:31Z**) and **permanently skipped everyone who had already acquired a cid.**
>
> **Reproduced in miniature**, not merely inferred: `Eisa Universe\كون عيسى` (5 notes, still carrying the legacy FULL index) holds **exactly two cid-less notes and exactly one sky row between them.** The same file - `Five Acts\Observation — Recent Captures.md` - is the sole stranded note in **five** separate universes.
>
> **MIG-108 is EXONERATED** by a stronger test than any lens proposed: its own pre-run snapshot joined on **`cid_cn`, not `path`** (a path join proves nothing when the migration rewrites paths). Of the 758 stranded, **162 existed pre-run and 0 of them had a sky row.** MIG-108 destroyed nothing.
>
> #### ❌ TWO AUTHOR CLAIMS OVERTURNED
> 1. **"Nothing restores it" is FALSE.** `search.rs:6329` is PJ-207 §15's restore - top-level in `init_db`, not version-gated, running on **every boot of every universe** - and it demonstrably works (the 231-row cluster above). The entry quoted §15's post-mortem while missing §15's remediation twenty lines below it. **The correct framing is "the existing restore's predicate is one clause too narrow", and that reframing changes the entire answer.**
> 2. The proposed trigger fix was **inert three times over**, not once. The author doubted the `WHEN` guard and was right - *reproduced* in a `:memory:` harness built from the live trigger bodies: an ordinary edit touches none of `path`/`name`/`library_name`/`cid_cn`, and `content_hash` is not even in the upsert's `DO UPDATE SET` list. Worse: the AU **body** is UPDATE-only, so widening the guard alone still produces no row. **Decisively: all 770 stranded notes carry a NON-EMPTY `cid_cn`**, so the one non-rename arm of the guard can never fire for any of them. Only a rename/move could have healed anything - and three of ECK's seven are in `.trash` and will never be renamed.
>
> #### ⛔ THE WRITE-PATH TRIGGER IS REJECTED - and the panel reproduced the harm
> A heal trigger firing before `note_meta_sky_au` inserts a bare row at `NEW.path`; the rename's `UPDATE sky_nodes SET path = …` then hits **`UNIQUE constraint failed: sky_nodes.path`** (it is the PRIMARY KEY). `migrate_note_db_paths` runs statements through a **log-and-continue** runner (libraries.rs:1584-1596), so the file moves on disk, every other table migrates, and **`note_meta` is left at a dead path** - turning a cosmetic gap into silent index↔disk divergence. Reachable by renaming any of ECK's 31 or Eisa Universe's 231 blank-cid notes. SQLite also fires AFTER-UPDATE triggers in **reverse creation order** and guarantees no order by contract, so a "place it correctly" mitigation is not available.
> *(The latency objection was separately **rejected on its own terms** - a `NOT EXISTS` probe is microseconds against a ≥1500 ms debounce, and `note_meta_sky_maturity_au` already fires on every save. The trigger dies on the rename hazard and on being unnecessary, not on cost.)*
>
> #### ✅ THE RULING - widen the repair that already exists, in `init_db`. Change no trigger.
> Placed **after** `ensure_dependent_tables_mig003_indexes` (search.rs:6304) for §15's own stated reason. **Leave the shipped narrow arm at search.rs:6327-6344 exactly as it is**; add a count-gated two-phase arm inside the existing `if owns` block:
> - **GATE:** `COUNT(note_meta)` vs `COUNT(sky_nodes)`; proceed only when they differ.
> - **PHASE 1:** anti-join for the missing paths. **PHASE 2:** per-path `INSERT OR IGNORE … SELECT … WHERE path = ?` (PK seek), then stamp `stratum`/`maturity`.
>
> **Measured on the Boss's live databases, read-only:** gate **54.6 ms cold / 0.0 ms warm**; Phase 1 anti-join **806 ms cold** on ECK (8,031 notes / 1.6 GB) but paid **once, only on the boot that repairs**; 2.2 ms on Eisa Universe. **The trap, measured:** a single statement with the full column list plans `SCAN m` and drags 273 MB of `body_text` - the query plan must be asserted, not assumed.
>
> **Non-negotiable clauses, each with its reason:** `INSERT OR IGNORE` **never** `OR REPLACE` (REPLACE is the mechanism that CAUSED this) and never a bare `INSERT`; **widen the stamp with the insert** - the shipped stamp carries the same narrow predicate, and restoring 770 rows with `stratum` NULL would put stratum 0 straight back into the Reviewer, **re-creating the exact harm and reporting success**; keep the new arm **`owns`-gated** (the stratum/maturity SQL reads the active universe's link registry - Boss ruling 2 holds); **no `.trash` exclusion** (54 of 57 trashed notes already have sky rows); **three separate numbers** - candidates / inserted / stamped - to `diag_log`, because `OR IGNORE` can silently under-heal; **no `scan_*`/`rebuild_*`, no walk, no file I/O, `sky_backfill::is_needed` untouched** - the PJ-332 bar is cleared by construction.
>
> **Verification before it ships:** exactly the 7 named ECK paths restored and 0 elsewhere; **`SCAN m` must NOT appear in Phase 1's plan**; restored rows carry non-NULL stratum and maturity; a duplicate-cid candidate is skipped rather than destroying the incumbent, and is counted.
>
> #### ⏳ BOSS DECISION OWED - the only one
> **`Eisa Universe`'s Sky View will gain 758 nodes and 1,853 edges on its next launch, and three trashed notes will reappear in ECK's.** A visible change to a knowledge surface, unannounced. **Panel recommendation: ship it silently but loudly logged** - automatic repair, one status-bar line naming the number repaired, no button, no progress strip, no 15 locale files. Nobody answers "leave 758 of my notes invisible", so a permission door is disproportionate; a notification is not. **Panel explicitly declines to decide it.**
>
> **NOT VERIFIED / labelled by the panel:** Phase 2's write cost (~30 ms insert + ~190 ms stamp for 758 rows) is second-hand - measured by one member on a scratch copy, not re-run. The post-fix "zero exceptions after 2026-08-10 07:30 UTC" sample is only 19 notes: the date boundary is strong, the sample is small, **and the ruled design does not depend on it.**
>
> ### ✅ PJ-332b *(MED · concurrency-race · FOUND IN AND FIXED IN THE SAME PASS)* - the PJ-332 fix introduced a window, and its own comment claimed a guarantee it did not provide
> **Found by the diff-scoped safety inspection run on the PJ-332 diff itself**, before the commit. Raised HIGH; **its verifier refuted the candidate's mechanism and magnitude** and confirmed a narrower defect — recorded because the refutation is as valuable as the finding: the claimed multi-thousand-row abandoned band could not occur (thread 2 reads the HIGH-WATER cursor, and thread 1 exits at most one batch after the generation bump).
>
> **Two real defects, both fixed:**
> 1. **`maybe_schedule` had no in-flight guard** (pre-existing). `is_needed` is version-only, so an unstamped universe re-arms on every call, and an A→B→A switch calls `ensure_search_db_ready` → `maybe_schedule` again while the first thread is still inside the lock-free file-read phase. **`review_backfill.rs:29-52` carries exactly the `static RUNNING: AtomicBool` compare_exchange guard for this hazard** ("threads racing on the same WAL"); sky_backfill had none. Copied byte-for-byte, released in the thread tail so the universe-switch early return frees the slot too.
> 2. **`run` read the cursor TWICE** — once for the stratum/maturity wipe, once for the walk start — with `ANALYZE` and the wipe UPDATE in between, both contending for the write lock. A cursor advance committed in that window made the wipe cover `(C_old, C_new]` while the walk began at `C_new`, so that band kept NULL stratum/maturity permanently, and `.unwrap_or(0)` wrote rank 0 into the Reviewer for every note in it. **Now read once, used for both.**
>
> **And a false claim corrected where it was made.** The PJ-332 comment asserted that the generation stop prevents a second thread being spawned on a switch back. **It does not** — `still_ours()` is evaluated only at the loop top, so the thread keeps working for up to one more batch, which is exactly the window the second thread spawns into. The correction is recorded in place rather than quietly deleted: **a comment asserting a guarantee the code does not provide is worse than no comment**, and this one was written in the very fix that created the need for the guard.
>
> **Known residual, recorded not glossed:** a panic inside `run` skips the slot release and leaks it for the process lifetime (no back-fill until restart; no data loss — the walk is cursor-resumable). Left as-is because it matches `review_backfill` exactly, and one consistent shape across the back-fills beats a lone RAII variant. **Revisit for all of them together, or not at all.**
>
> ### ✅ CLOSE-OUT 2026-08-22 — PJ-332, PJ-332b and PJ-334 all shipped, Boss-validated
> **Boss test: PASS.** Went `tutorial-auditor` → `ui-inspector` (**APPROVED**, 27 claims) → panel (**SEND WITH 8 EDITS**, three of which would otherwise have produced a false failure report) → Boss. All eight applied.
>
> **The pipeline HELD the first attempt and was right to — it found a defect in the receipt wiring that no gate could have caught.** `loadSkyRestoreReceipt()` fired at `+layout.svelte:2837`, while the database only opens at `:2963` (`refreshLibraryCaches` → `ensure_search_db_ready`), and `take_sky_restore_receipt` returns `None` the moment `state.db` is still `None`. **The repair ran and the line announcing it could never appear** — a silent no-op in the half whose entire job is not being silent, and precisely the behaviour the Boss had approved. Caused by piggy-backing the read onto an existing boot call for convenience; the comment on that very line said "independent failure". Rust 1530/0, vitest 997/997 and svelte-check were **all green throughout** — it is a call-ordering fact between two lines 126 apart in one file, invisible to every suite. Fixed by moving the read to after the database opens, with the reasoning recorded in place.
>
> **What shipped:**
> - **PJ-332** — the Sky back-fill thread opens its own connection (as every sibling already did) and stops when the user leaves. **Reproduced first**: universe B stamped complete by a thread back-filling A. The original reproduction can no longer be written — the functions take a pinned connection, so there is no swappable handle to pass. The defect is inexpressible, not merely guarded.
> - **PJ-332b** — the single-run-slot guard copied byte-for-byte from `review_backfill`, one cursor read instead of two, and a **false claim corrected where it was made** (the generation stop does NOT prevent a second thread; `still_ours()` is evaluated only at the loop top).
> - **PJ-334** — the boot restore widened past the `cid_cn = ''` clause that stranded 770 notes. Count-gated, two-phase, `INSERT OR IGNORE`, stamp widened with the insert, `owns`-gated, `.trash` included, three-number receipt. **Mutation-proved**: disabling the stamp turns the test red, enforcing the panel's sharpest clause — *restored complete or not restored*.
> - **The receipt** — one faint centred status-bar line naming the number repaired, dismissible, shown only on the launch that repaired (the Rust side clears it as it reads). 2 keys x 15 locales, parity verified. **Deliberately not a `JobProgressStrip`** although four exist: there is no job to watch, the repair finished before the window painted.
>
> **Two things the tests taught me, recorded because both were my errors:** `stratum` is stored as TEXT (which is why every reader CASTs it, and why a missing value becomes rank 0 rather than an error); and my duplicate-cid collision test was **unconstructible** — `note_meta.cid_cn` is itself UNIQUE, which is exactly why the panel measured zero duplicates. Rewritten to assert the invariant that closes the hazard.
>
> **Still open from this family:** PJ-333 (MED, `bring_in_library` does not ancestor-walk — one line, but it changes what the app refuses, so it is the Boss's call) and the **origin of the missing rows** is established for the *permanence* but not for the *first event*.
>
> ### ✅ PJ-335 *(CLOSED 2026-08-22 · Boss-validated · three findings in `bring_in_library` — ALL PRE-EXISTING)*
> **Found by the diff-scoped inspection run on the PJ-333 change.** None was caused by that change; the inspection found them because the change made it look at that file. All three are reachable from the **Bring In** button.
>
> **1. HIGH · silent-data-loss — the registered-library guard FAILED OPEN.** It read `load_all_libraries`, the LENIENT reader, whose own doc says every caller must be read-only. It swallows a read failure AND a parse failure into an empty list and then **caches that empty answer for the process lifetime**, so the `any()` check was vacuously false and a registered external library could be relocated — registry entry left pointing at an empty folder, every index row stranded, and `add_library` afterwards reporting either success or a registry error reading as "nothing happened", **after an irreversible move**. The strict twin sat two lines above (`assemble_foreign_roots`, PJ-322, Boss decision 1: *a plan that moves directories must refuse rather than guess*). **Fixed:** `try_load_libraries`, refusing on a degraded read. *(The lenient read in `mig108_preflight` is left alone — there an empty answer yields "nothing to move", which is fail-SAFE.)*
>
> **2. MED · content-loss — the cross-volume Move deleted the source unverified.** `copy_dir_recursive` skips symlinks and junctions with a bare `continue` and no log, so a junction'd subtree was not copied and was then **destroyed at the source**, with `Ok` returned.
> **⚠️ The fix the inspection prescribed would NOT have caught its own scenario, and that is recorded because it matters.** It asked for `run_move_phase`'s src/dst file-count compare — but **`count_files` skips symlinks too, by the same rule**, so the counts MATCH while the subtree is missing: the check would have passed and the source would still have been deleted. Verified by reading both walkers side by side. **Fixed on the thing that actually differs:** if the source contains any reparse point the original is KEPT, and the user is told which link, and where both copies are. The count compare is kept as a second, independent check for a non-symlink shortfall.
>
> **3. MED · content-loss — a failed copy left its debris under the universe root, which IS a library.** The dialog said "Failed to copy file: …" and the user reasonably concluded nothing had happened, while several hundred valid `.md` files sat under the root where the indexer and watcher take them as real notes; a retry de-collides to "Name 2", so the orphan persists as a duplicate indexed set. **Fixed:** best-effort cleanup of the partial copy, and the message says whether the cleanup succeeded — never silently.
>
> **All three share one shape:** an operation that moves the user's files treating "I could not read that" as "there is nothing there". Same class as PJ-322 and PJ-334. **Gates: Rust 1532/0. BOSS TEST PASSED 2026-08-22** — the refusal fired and named the universe. `tutorial-auditor` → `ui-inspector` (**APPROVED**, 24 claims, zero findings) → panel (**SEND WITH 6 EDITS**, two blocking). **The blocking one is worth recording: the Boss's INSTALLED Constellation is dated 2026-06-13 and contains no `bring_in_library` at all** — had he run it, the button would not exist, and the nearest equivalent would have produced exactly the sight the tutorial's first failure mode told him to report as an app-killer. A false emergency, averted by a gate that asked which binary the instructions assumed.
>
> ### 🆕 PJ-336 *(LOW effort · documentation coverage gap · PRE-EXISTING)* - the 14 translated help sets are missing 22 of the 43 English topics
> **Measured 2026-08-22 while doing SO#2 for this session's user-visible changes.** `docs/help.uConstellation.World/` (English) carries **43 topics**; `docs/help.ar/` and its 13 siblings carry **21** - and the missing ones include **Universe**, **Libraries**, and every topic this session's changes belong in. So the Bring In refusals and the Sky View self-repair were documented in English only, because there is no translated topic to put them in.
>
> **This is a pre-existing gap, not something this session created** - the topics were never translated. Recorded rather than silently skipped: SO#2 says update the help files in all languages, and this session could not, because 22 of the trees do not exist. **Creating 22 topic trees x 14 languages is its own job**, not a tail-end of a bug fix.
>
> **Related:** PJ-328 (Rust error strings reach the user untranslated) - the Bring In refusals documented above are themselves English-only strings from `mig108.rs`, so translating the help topic without translating the message would leave a manual describing text the user never sees in their language.
>
> ### 📋 Housekeeping
> - **Gates.** Rust **1501 / 0** (20 ignored, incl. the 1.2 acceptance test) — and this time the number is a **distribution, not a sample**: 16 consecutive clean single-process runs after PJ-303/PJ-304, then **9 more consecutive green** on the final state after PJ-307/PJ-308 and the deletion of the harmful test. (1501 not 1502 because that test was removed. One sweep run was lost to a transient Windows `LNK1104` linker lock — a file-lock on the freshly-linked exe, not a test failure.) **Release binary built 2026-08-17 17:02**, after every fix in the batch. Against a baseline failing ~1 in 6, that is ~5% likely by luck alone, so the runs corroborate; the proof is that both mechanisms were reproduced deterministically and removed. Diff-scoped inspection on the PJ-303 diff: **0 confirmed findings**; a combined re-inspection of the full five-file diff was run because the first PJ-302 inspection spanned the stash used for the pristine test and could have read the wrong bytes.
> - **A measurement-discipline note, recorded because it is the same disease as the bugs.** I twice ran two `cargo test` loops concurrently and once edited source mid-loop, then read the resulting failures as evidence. They were artefacts of my own concurrency — shared mutable state with a window, created by the person cataloguing shared mutable state with a window. The contaminated runs were discarded and re-measured clean. **Rule for the rest of this migration: one suite at a time, no source edits mid-sweep.**
> - Boss tutorial for PJ-303 went `tutorial-auditor` → `ui-inspector` → **REJECTED** (wrong Settings label; and it claimed the Arabic cache rebuilds at app-open, which it cannot — `GenerativeFst::get` is a lazy `OnceLock` fired by indexing/search, and `.setup()` makes no Arabic call) → corrected → re-inspected.
> - Filed in passing, unnumbered: **`tension.rs:88-92` states a false claim** — that `validate_path_in_any_library` refuses cUniverse paths. It does not; `libraries.rs:727-728`'s own doc says "including child universe libraries".
> - Build cost, for planning: a cold `cargo test` compile is **~8–18 min** on this machine; incremental runs ~15–25 s.
>


> **What changed in v1.91** (**MIG-111 Phase 1.1 SHIPPED and the H1 harness is committed BEFORE the code it constrains. The inspection found a HIGH in Phase 1.1 — the §0.4 dead-guard defect, one layer up, with all nine tests green over it. PJ-301 filed. PJ-294's missing documentation closed in 15 languages**):
>
> **► NEXT ACTION — MIG-111 Phase 1.2 (the routed context pool).** Its acceptance condition is already committed and `#[ignore]`d in `federation/vocab_harness.rs`; **removing that attribute is the definition of done.** The Boss's ruling on **PJ-288** remains owed and can be taken at any point.
>
> ### ✅ MIG-111 Phase 1.1 — `resolve_owner` (R2) — CLOSED (commit `660cfda8` + this commit's fix)
> The Router's first question, alone in `federation/owner.rs`: longest-match wins (attack H3), unknown is an `Err` and never the active universe (attack H2), roots come from the federation tree and never from library lists. **12 tests.**
>
> ### 🔴 The finding that mattered — Phase 1.1 shipped with the §0.4 defect, one layer up
> The per-build inspection returned **one CONFIRMED HIGH** in the phase I had just called clean, and it is the same disease §0.4 already paid for:
>
> `resolve_child_universe_roots_recursive` builds the federation list with `fs::canonicalize` — Windows' verbatim `\\?\E:\…` — while the active root arrives raw from the registry. `norm()` folded slashes and case and knew nothing of verbatim prefixes, so:
>
> - a **nested** linked universe — **the DEFAULT shape under MIG-108** — resolved to the **ACTIVE PARENT** with `is_active: true`. That is attack H3, defeated in the pure function and reintroduced by the wrapper. Routing on that answer writes a child universe's rows into the parent's database, no error, every row count still correct.
> - a linked **sibling** failed the other way and became **permanently unroutable**.
> - `Owner.root` returned raw from one branch and verbatim from the other — **one universe, two identities**. For a lock key that is not a weaker lock; it is no lock.
>
> **All nine tests were green over it**, because every one drove the pure function `resolve_owner_in` with hand-built RAW paths and nothing exercised the form the app actually supplies. Reproduced with real directories and real `fs::canonicalize` (3 new tests, red→green), then fixed in `norm` — **stripping the verbatim prefix so the comparison is total over path forms**, rather than by promising callers pass the right form. That promise has now been broken twice in one migration. The module header's claim that the roots "are raw" was false on the federation side on the day it was written; corrected in place. **→ LL-048.**
>
> Whole-Ecosystem sweep of every `canonicalize` comparison: the rest canonicalize BOTH sides, which is safe. `universe_lock::canon` is deliberately left alone — see PJ-301.
>
> ### ✅ The H1 harness — `federation/vocab_harness.rs`, committed before Phase 1.2
> One note indexed under two vocabularies through the REAL `init_db` + `index_note` + `maintain_incoming_after_save`, diffing **aggregate VALUES, not rows** — because a vocabulary mismatch leaves every row count correct while changing what the rows SAY. Two things it found before any routing code exists:
> - `index_note` alone never reaches `maintain_incoming_after_save` (that runs on the save path), so a harness built the obvious way would have observed edges and none of the incoming aggregates H1 is actually about.
> - **Its determinism test failed on the first run.** `link_types::REGISTRY` is a **process-global** read at CALL time by all 26 of its call sites; a sibling test's `set_active` reached into an already-open database and changed what it produced. **This eliminates the design Phase 1.2 was most likely to reach for** — *open the child's connection, `set_active` the child's types, write, restore* — because the debounced save, a backfill tick or the watcher lands in that window and computes with the wrong vocabulary, with every row count still right. A routed write must carry its vocabulary **explicitly**. **→ LL-047**, pinned by `a_vocabulary_swap_reaches_back_into_an_already_open_database`.
>
> ### ✅ PJ-294's documentation — CLOSED (it shipped without any)
> The Hotkeys screen shipped with no user documentation at all. Now: a new **Hotkeys** help topic, and **User Manual §17 in all 15 languages** gains the `Ctrl+Shift+T` row and a *Customising your shortcuts* section — each translation quoting **its own** locale's labels for Settings → Hotkeys, *Filter commands*, *Press keys…*, *Not set*, *Reset* and *Clear*, so the manual names each control exactly as that reader sees it. Documented accurately from source: `Ctrl+Q`, `Ctrl+W` and `F5` are **not** reserved and were nearly written in from memory.
>
> ### 🆕 PJ-301 — filed from this step
> - **PJ-301** *(MED · identity-ambiguity · **deliberately not fixed in-pass**)* — **`universe_lock::canon` can return two different identities for one universe.** It is `fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf())`, so it yields the verbatim form when the syscall succeeds and the raw form when it does not — while its own doc-comment states that "path identity must have ONE answer app-wide." If two call sites ever take different branches for the same root, `lock_path` differs and two processes each believe they hold the universe's owner lock. **Not fixed here because `canon` also BUILDS the lock file's path**, and the verbatim form is exactly what lets a deeply-nested universe exceed Windows' 260-character limit — so stripping it trades a latent ambiguity for a real long-path regression I have not measured. The reconciliation contract is documented on `Owner::root` in the meantime (`canon(stripped) == canon(verbatim)` when the directory exists, so everything that needs identity must go through `canon` rather than comparing strings). Needs a measured decision. **Group 2.**
>
> ### 📋 Housekeeping
> - **Session-log gap closed.** No log existed for **2026-08-12, 08-15 or 08-16**; `SESSION-LOG-2026-08-17.md` covers that whole stretch and says so at the top rather than hiding it.
> - **Gates.** Rust **1500 / 0** (20 ignored, incl. the 1.2 acceptance test). Diff-scoped inspection on every commit, and the fix above was **re-inspected** before commit.
>

> **What changed in v1.90** (**MIG-111 PHASE 0.4 SHIPPED — the writers on the federation boundary are guarded, and Phase 0 CLOSES. Seven writers, not the five the plan named. Four findings filed: PJ-297…PJ-300**):
>
> **► NEXT ACTION — the Boss's ruling on PJ-288**, then **MIG-111 Phase 1 (the Router)** — Phase 0's foundations are complete, which is the gate the rest of the migration was waiting on.
>
> ### ✅ MIG-111 Phase 0.4 — the unguarded writers onto the boundary (R1) — CLOSED
>
> **The plan named five. There were seven**, and the two extra were found only because the tests were rewritten to FIND writers instead of listing them:
>
> | writer | what it did unguarded |
> |---|---|
> | `ensure_cid_cn_cmd` | runs on the note-**open** path and writes identity into frontmatter — so simply opening a linked universe's note rewrote that universe's file. It had no `AppHandle`, which is *why* it had no guard: there was nothing to ask. |
> | `write_conflict_sidecar` | created a `.conflict` file INSIDE a corpus Constellation only reads |
> | `sources_set_manual` + `content_type_set_manual` | rewrote a foreign note's frontmatter; the guard now lives in the two write HELPERS, so all four commands are covered by construction |
> | **`sources/bulk_ops.rs` Approve-All** *(not in the plan)* | reached disk through its OWN `gate_rmw`, so per-card Accept correctly REFUSED a linked note while the bulk path silently wrote to it |
> | `update_base_columns` / `update_base_order` | fixed in the SHARED `validate_base_path`, covering six writers rather than the two named |
> | **`create_base`** *(not in the plan)* | authorised against the FEDERATED resolver and created `.base` files inside linked universes |
>
> **Rust 1485/0** (+8 wiring tests).
>
> ### What the gate found in this step's own diff — five rounds
>
> Recorded because the pattern is the lesson, and it is one pattern: **I kept proving properties over the sample I happened to look at.**
>
> 1. **The base guard was a DEAD NO-OP.** It compared a `fs::canonicalize`d path — Windows' verbatim `\\?\E:\…` — against registry roots normalised only for slashes and case. The comparison could never match. Every base writer still wrote into linked universes. **And the test passed the whole time**, because it compared SOURCE-TEXT byte offsets: it asserted the call was *written* first, never that it *fired*. Now the test drives the real comparison with both path forms.
> 2. **Approve-All bypassed the boundary** — and the test named *"no sources command writes around the helpers"* counted `gate_rmw` occurrences in `mod.rs` ALONE, so it read green over a bypass in the file next door. It now walks the directory.
> 3. **`create_base` was outside the boundary entirely** — missed because the test asserted the two writers the plan NAMED. It now searches for `.base` writers, and on its first run immediately found a sixth (`create_workspace_base`), which is exempt **by construction** (its destination derives from the active universe, so there is no caller-supplied path to point elsewhere) — stated as a construction, not a name.
> 4. **A regression I introduced**: `create_template` applied the base guard to the SOURCE note rather than the destination, so "Save as template" on a linked universe's note broke — dialog closing as though it worked, no file, no error. Making a template FROM something you can read is exactly what federation is for; the boundary belongs on where it LANDS.
> 5. **My own comment was false.** The sidecar guard claimed "the conflict itself is still surfaced — the refusal stops the WRITE, not the warning." It did not: the frontend recorded a conflict only on success and otherwise logged to a console that does not exist in release. **The refusal had turned "a stray file appears in a corpus we only read" into "your external edit is clobbered and you are never told."** The conflict row now appears either way, says the copy could not be kept, and omits the two actions that need a file.
>
> ### 🆕 PJ-297 … PJ-300 — filed from this step, all PRE-EXISTING
> - **PJ-297** *(MED · index-divergence)* — **`canonical.rs` renames notes with no index migration.** It writes to a new path and deletes the old — a rename in all but name — with no `migrate_note_db_paths` and no reindex, while every write it makes is watcher-suppressed. Runs **unattended at boot** behind a localStorage flag PJ-110 proved non-durable. For the rest of that session search hits open nothing and backlinks are dead. Session-bounded: the next boot's reconcile relocates rows by `cid_cn`. **Group 1 · ②.**
> - **PJ-298** *(MED · swallowed-write-error)* — the same path does `let _ = fs::remove_file(...)` after writing the new copy, so a failed delete leaves **two files carrying the same `cid_cn`**, counted as restored with nothing pushed to `errors`. **Group 1 · ②.**
> - **PJ-299** *(MED · freeze)* — `move_item_db_tail` holds the single global SearchState writer mutex across `migrate_note_db_paths` for EVERY descendant of a folder move: the exact unbatched hold its sibling `rename_folder_db_tail` was rewritten to avoid (measured 19.79 ms/note → ~15.9 s for 803 notes). **Group 2.**
> - **PJ-300** *(MED · fail-open · **filed deliberately, with a ruling owed**)* — **the federation cache can hold a DEGRADED resolve for a whole session.** `resolve_libraries_recursive` swallows an unreadable child registry three ways and contributes zero libraries for it; `load_all_libraries` then caches that reduced list until the next registry mutation. `foreign_library_roots` under-reports, and **every §0.4 guard silently degrades to a no-op**. The fix shape is known — refuse to cache a degraded resolve and fail CLOSED, the "absent is a fact; unreadable is an unknown" rule already used by `try_load_libraries` — but it changes the failure semantics of the resolver the WHOLE app reads through. Not tacked onto the end of a long step. **Group 1 · ① · needs its own pass.**
>

> **What changed in v1.89** (**PJ-294 SHIPPED — the Hotkeys screen binds keys for real, and New Tab is a command. Twelve gate rounds; LL-046 written on why. PJ-295 and PJ-296 filed from what it surfaced**):
>
> **► NEXT ACTION — the Boss's ruling on PJ-288**, then **MIG-111 Phase 0.4** (the five unguarded writers, which completes Phase 0).
>
> ### ✅ PJ-294 — CLOSED (Boss said "ship"; committed, walkthrough follows)
>
> **The defect.** Settings → Hotkeys listed every command, offered to record a key, and threw the keystroke away — *"For now just display the captured shortcut (hotkey persistence is a future feature)"*. It did not even display it. Anyone who tried to rebind a key in Constellation had it silently do nothing. And **New Tab was not a command at all**: it existed only as the `+` button beside the tab strip, so it had no shortcut, no palette entry, and no row in the Hotkeys list.
>
> **What shipped.**
> - **New Tab** registered as a first-class command (`Ctrl+Shift+T`, verified free), appearing in the palette and the Hotkeys list automatically.
> - **Capture persists**, through `eventToShortcut` — the SAME function the dispatcher matches against. The old hand-rolled version emitted modifiers in a different ORDER, so even had it saved, a three-modifier binding could never have fired.
> - **Conflict detection** naming the command that already holds a combination, comparing CURRENT bindings and unioning the full command set itself (so a caller's partial list cannot produce a false "free").
> - **Reset** (removes the override, so a future change to a default still reaches the user) and **Clear** (an empty binding meaning genuinely none).
> - **Refusals** for a bare key, for Escape in any form, for combinations the dispatcher answers itself, and for **every key the editor owns** — derived from the six keymaps NotePane installs.
> - **macOS**: the stored form was already neutral (`eventToShortcut` maps ⌘ and Ctrl to one token, so settings sync between machines without migration); only the display needed a Mac branch — `⌘⇧T`.
> - All 15 locales, parity-gated. **vitest 966/966** (45 for this feature), svelte-check 0 errors.
>
> **Twelve inspection rounds — see LL-046.** Round one looked finished and was dead for a third of the commands, dead on macOS, dead for any arrow binding, one click from disabling every shortcut in the app, and capable of giving away the editor's own keys. The through-line: every round, what I had written was *shaped* like a derivation and was really a **list** — first literally a list, then a loop reading one field of a four-field structure, then a canonicalisation describing the keymap's LABEL rather than the keystroke that triggers it. The findings shrank as the sourcing got honest, not as cases got patched.
>
> ### 🆕 PJ-295 — palette commands with empty actions that consume editor keys
> Three shipped commands — `toggle-italic` (Ctrl+I), `toggle-comment` (Ctrl+/), `select-next` (Ctrl+D) — have `action: () => {}`. The dispatcher matches them, calls `preventDefault`, runs nothing, and CodeMirror then refuses the already-defaulted event: the key is consumed and neither the command nor the editor's binding happens. `nav-back`/`nav-forward` (Alt+←/→) also sit on editor keys but are real commands, where the shadowing may be intended. **The decision is whether these should be real commands or not commands at all** — which is why it is filed rather than folded into PJ-294. **Group 2 · ②.**
>
> ### 🆕 PJ-296 — the one reservation that cannot be derived
> CodeMirror binds `Shift-Mod-\`, and on a US layout that keystroke arrives as `Ctrl+Shift+|` — the character depends on the user's keyboard layout, so no table computed from keymap LABELS can predict what `eventToShortcut` will record. Reserving the label form does not protect the real keystroke, and a user could bind a command to it and lose `cursorMatchingBracket`. Stated rather than guessed at, per LL-046's last rule. Options: a layout-aware probe at capture time, or refusing punctuation keys that shift-map. **Group 2 · LOW.**
>

> **What changed in v1.88** (**PJ-287 CLOSED — a write composed from a model that no longer exists is no longer reported as success. Six gate rounds, five findings, two of them mistakes inside the fix for the previous one. PJ-288 goes to the Boss as a RULING, not a task**):
>
> **► NEXT ACTION — the Boss's ruling on PJ-288** (below), then **MIG-111 Phase 0.4** (the five unguarded writers, which completes Phase 0).
>
> ### ✅ PJ-287 — CLOSED (pending the Boss's pass; committed only after it)
>
> **The defect.** A tab id is a SLOT, not a note. `markSaved`/`noteDiskSynced` had been lineage-guarded since PJ-207 §15 and both correctly refused a mid-flight re-seed — **but a guard that returns `void` tells its caller nothing**, so `saveUnchained` cleared the net, ran `onSuccess` (reindex, re-embed, broadcast) and returned success anyway. Reachable ordinarily: a save fails on a locked `.md`, the ~10 s auto-retry parks on `await`, and the user clicks **"Discard my changes"**. The bytes they discarded went to disk; the version they kept survived only in a model reporting clean, so nothing ever wrote it back; the banner cleared, reading as success. **An explicit user decision, silently inverted.**
>
> **The fix.** `lineageHolds` is now THE predicate, and the two mutators guard on it rather than repeating its comparisons, so a guard and a caller's check can never disagree. `saveUnchained` ASKS it, and on a break asks one further question — *does the slot's model actually disagree with what was written?* — with three outcomes: divergence → baseline as recovery + stash the kept version + report via `onSuperseded`; agreement → baseline only; model gone → nothing. The compare-and-clear runs in every case.
>
> **Reproduce-First honoured**: red on demand before anything was designed, via a writer the test releases by hand so the interleaving is deterministic rather than timed.
>
> **The five gate findings, recorded because the pattern matters more than the fix** — every one was a consequence of the previous round's repair, not the same defect resurfacing, and **two were mistakes inside the fix for the one before**:
> 1. the defect itself — a `void` guard feeding a caller that ratified anyway;
> 2. **cross-note bleed (HIGH)** — "lineage broke" covers THREE states (gone · re-seeded · **different note**); the repair called `setDiskBaseline`, the one model mutator with no identity guard, stamping note A's bytes onto note B. Fixed AT THE MUTATOR: "every caller remembers" is the promise the next caller breaks (the `read_state` lesson, MIG-111 §0.3);
> 3. the residue (MED) — refusing to ratify left the kept version in memory only, destroyed on tab close, with nothing recording the split;
> 4. phantom recovery (MED) — when the re-seed read back the very bytes just written (`write_note` async, `read_note` sync), the repair flagged already-durable content as work disk never had: a stale net entry that beats a NEWER file on the next open. **PJ-181 re-armed by a repair**;
> 5. dropped compare-and-clear (HIGH) — **an exception I invented**, on reasoning that did not hold (it clears ONLY IF the net still holds exactly the bytes just written, which are on disk by then). The fix was to DELETE the exception; one test changed too, because it was asserting the wrong reasoning back.
>
> Round six: **zero findings.** A line was drawn before round five — a fifth finding meant asking whether the approach was wrong rather than patching again — and honoured; the rethink found the fix *removed* an exception rather than adding a guard, and the branch ended with fewer special cases than at round three.
>
> **Verification.** vitest **921/921** (82 files, 6 new), svelte-check **0 errors**, Rust **1477/0**, diff-scoped inspection clean. **Boss-validated**: Steps 1, 3, 4, 5 PASS; Step 2's content assertion confirmed transitively (Step 3's pre-state — "zarquon one" intact after the tab round-trip — held, and it passed).
>
> ### 🆕 PJ-293 — the (+) new-tab button's tooltip is hardcoded English
> `+layout.svelte:8618` — `title="New tab"`, a literal rather than a `$t()` key, against the standing i18n rule that every user-facing string goes through `$t()` in all 15 locales. Found because the Boss had to use that button after Ctrl+click turned out to be a multi-select gesture. Trivial to fix; filed rather than folded in, since it is unrelated to PJ-287's diff. **Group 2 · i18n.**
>
> ### ⚖️ PJ-288 — A BOSS RULING IS OWED, NOT A TASK
>
> `flushIfDirty` and its callers ask `isDirty` alone, so content recovered from the crash net — born CLEAN by construction, and holding text disk has never had — is never flushed by any departure, universe switch, or app close, and is not counted by the residual-dirty close marker. Its only copy is the localStorage net that **PJ-110 proved non-durable**.
>
> A 30-site audit (`durability-arbiter-audit`, every site read in context) says the honest predicate is `isDirty || hasUnsavedRecovery` and that **22 sites must move together** — every store-side gate delegates to one choke point, so changing any single one is either inert or produces a *false* success. There is no small version.
>
> **The trade the Boss must rule on**, stated plainly because it is a design decision and not an engineering one:
> - a locked `.md` could **block navigation** on a note the user never edited (today the nav proceeds and the recovery is silently dropped);
> - a **checkbox tick, a Base cell edit, or "link this mention" would be REFUSED** on a restored note whose recovery cannot be written (four surfaces);
> - **app close** does more work inside Rust's ≤5 s hold.
>
> Honest refusals replacing silent losses — but new refusals the Boss will see. **Group 1 · ① · ruling owed.**
>
> ### 🆕 PJ-292 — dead UI strings in NotePane: a save indicator and a placeholder that cannot render
>
> **Two dead surfaces, found in two consecutive gate rounds on one test draft**, both by `ui-inspector` applying LL-045's reachability half rather than checking that a string exists.
>
> **(b) The empty-note placeholder never renders.** `notePane.placeholder` / `notePane.bodyPlaceholder` ("Start writing...") have **zero call sites** in any `.svelte` or `.ts` file — they live only in the 15 locale JSONs. What a blank note actually shows is the CM6 `TemplateDoorWidget` button reading **"Start from a template…"** (`NotePane.svelte:478-535`, `en.json:2372`). The component's own comment records why: CM6's native `placeholder` extension "never fired" for a freshly created note (doc length 1, not 0), so it was abandoned — and the keys were left behind in every language. Delete them across all 15 locales, or wire a real placeholder.
>
> **(a) The "Saving..." indicator is dead code, and it was described to the Boss twice.**
> `NotePane.svelte:1565` renders `{#if saving}…{$t('notePane.saving')}…{/if}`, but `saving` is a `$props()` default (`:134`) that **nothing ever sets**: the sole mount site (`NoteEditor.svelte:792-832`) does not pass it, and the component never reassigns it. So the label cannot render in the running app, and there is **no on-screen indicator of any kind for an ordinary successful save** — nothing visibly changes (the only save-related UI is the failure-path `SaveHealthBanner`). Found by `ui-inspector` applying LL-045's reachability half to a test draft. **It reached the Boss once already**: the PJ-278 batch's Stage 1 Step 1 told him he "may briefly see a small Saving... label" — he passed the test regardless, because nothing depended on it, but the claim was untrue when made and the gate approved it that round by verifying the string existed rather than that it could appear. Decide: wire the prop (a real save indicator, which the debounce arguably wants) or delete the dead branch, its i18n key and its CSS. **Group 2 · ② + doc-drift.**
>

> **What changed in v1.87** (**The ①+② remediation pass — PJ-278 … PJ-283 ALL FIXED in one batch, six of them in the Boss's stated priority. PJ-286 filed: the frontend perf gate cannot tell a regression from machine noise**):
>
> **► NEXT ACTION — MIG-111 Phase 0.4** (the five unguarded writers onto the boundary), which completes Phase 0. The remediation that preceded it is closed below.
>
> ### ✅ PJ-278 … PJ-283 — CLOSED (pending the Boss's pass; committed only after it)
>
> Six PJs, one concern: **silent failure on and around the save path.** Taken ahead of Phase 0.4 because six of the nine tenth-sweep findings sit in the Boss's ①+② priority — knowledge actively lost, and the index lying about your notes — and WA#6 forbids shipping a known finding.
>
> - **PJ-278 — CLOSED.** Three durable-save paths reindexed but never re-embedded: **Focus mode** (the designated fast-capture surface), the save-failure **Retry** (which also never told the second screen, while the user watched the banner clear), and **conflict merge** (which replaces the whole body by construction). One `afterDurableSave` now owns the routine, so it is no longer something seven call sites must each remember. 5 vitest cases.
> - **PJ-279 — CLOSED.** `resolve_structural_conflict` now readies the search DB **before** the write (refuse rather than half-do it — the retry was otherwise a dead end, since a second click finds the file already resolved and skips the reindex forever) and logs a reindex failure instead of discarding it. 2 source-assertion wiring tests.
> - **PJ-280 — CLOSED.** The review write pair is pinned at BOTH halves; a row that would land in a different Universe's database is refused with a message naming both. The comparison is the pure `open_db_belongs_to`. 3 tests, incl. fail-closed on an unidentifiable DB. *Re-routing (rather than refusing) needs MIG-111 Phase 1's Router — noted there, not deferred silently.*
> - **PJ-281 — CLOSED, and it was the severe one.** A transient read failure of a legacy `workbench.json` returned "you have no collections"; the frontend then persisted that emptiness, **permanently closing the one-shot adoption gate** — the user's entire Workbench set gone, silently, from one virus-scanner file lock during one boot. A **second defect in the same block** was found while fixing it: it wrote the legacy bytes *before* parsing them, so a damaged file was copied into place and locked collections out entirely. Now routed through `read_persisted_json` (only NotFound is trustworthy emptiness) and parse-before-write. 4 tests.
> - **PJ-282 — CLOSED.** CECE's per-cataloger timeout **could not time out** — the worker ran inside `std::thread::scope`, which joins before returning — while the trail told the user the cataloger had been "isolated by orchestrator". Now a detached thread. 1 test: an 8 s cataloger against a 500 ms budget returns control in 0.51 s.
> - **PJ-283 — CLOSED.** "Background classification = on save" dispatched an event whose only listener lives in a mounted `SourceReviewPanel`; in the normal typing state nothing listened and the setting did nothing, silently. The event is now `cancelable`, an open panel claims it, and unclaimed the editor makes the call the setting's own contract promises. No automated test — a hand-off between two mounted screens; Stage 2 of the Boss test is its verification, stated as such in the test.
>
> **The gate caught a cross-note bleed in the PJ-278 fix itself.** `afterDurableSave` first read the note model AFTER the awaited write — but a tab id is a SLOT, not a note, and `openNoteTab` reuses it in place without serialising behind an in-flight write of a *clean* model. Note A's write could resolve and embed **note B's body under A's path**; with the tab closed, `?? ''` force-embedded an empty body over the real vector. Same content-integrity class as BUG-023. The identity-correct value was already being handed to the callback and ignored — the helper now derives the body from the written bytes and performs no lookup at all. Re-inspection of the corrected code: **zero findings**.
>
> **Verification.** Rust **1477/0**, vitest **915/915**, svelte-check **0 errors**. The Boss test was **rejected twice** by `ui-inspector` before approval — round 2 caught the coverage claim being false (PJ-279 and PJ-280 had *no* tests; they were written rather than the absence merely disclosed) and PJ-281 omitted from the Boss's view entirely.
>
> ### 🆕 PJ-286 — the frontend perf gate cannot tell a regression from noise
> `tests/sight-v6/perf.test.ts` and `tradition-perf.test.ts` assert wall-clock budgets (≤16 ms per tradition switch, ≤400 ms for a full cycle, ≤32 ms for facet rebalancing) and fail a **different number of tests run to run**: 3 failures on a clean tree, 1 with an unrelated batch applied, 12 under concurrent load. Every run therefore requires a stash-and-compare to establish whether anything real broke — which is the opposite of a gate. They also guard **Sight, disabled in core since MIG-038**. Decide what they should measure (a relative budget, a benchmark harness outside the unit suite, or deletion while the feature is disabled) rather than patching the thresholds upward. **Group 2 · test infrastructure.**
>
> ### 🆕 PJ-287 … PJ-291 — from PJ-284's unswept scopes, and the first one is an APP-KILLER shape
>
> PJ-284 filed the tenth sweep's three unswept hunt groups. Two of them — **note-model ownership** and **cross-window integrity** — have now been swept (`noteModel.ts`, `noteSession.ts`, `secondScreen.ts`, both second-screen components) and returned **five confirmed findings**. All are PRE-EXISTING; none is in the PJ-278…283 batch (verified: that batch touches none of these files). **PJ-284 remains OPEN for its third scope, freeze-and-leaks**, which does not decompose into a file list and needs its own cross-cutting pass.
>
> - **PJ-287** *(**HIGH · false-success · silently inverts an explicit user decision**)* — **`saveUnchained` (`noteSession.ts:269`) ratifies a write whose model lineage has broken.** `markSaved`/`noteDiskSynced` correctly refuse on a generation mismatch, but the function still runs `clearNetIf` + `onSuccess` and still returns `ok:true`. The reachable path: a save fails (locked `.md`) → the banner appears → the ~10 s auto-retry composes and parks on `await write` → **the user clicks "Discard my changes"** → a new model is minted from disk → the parked write then lands. Result: **the bytes the user explicitly discarded are written to disk, and the version they chose to keep is destroyed** — surviving only in an in-memory model that reports CLEAN and will therefore never be written back. The write is watcher-suppressed so nothing adopts it; `afterDurableSave` then reindexes and re-embeds from the discarded content. No banner, no sidecar, no surface. Fix shape: compare the returned generation against the live model before ratifying success, and/or serialise `discardFailedSave` against an in-flight save. **Group 1 · ① — this is the top of the backlog after PJ-278…283 commits.**
> - **PJ-288** *(MED · silent-data-loss)* — **`flushIfDirty` (`noteSession.ts:299`) is blind to `netUnsaved`.** A model seeded on restart from the write-ahead net holds content disk has never had, yet is born CLEAN by construction (version === savedVersion), so `isDirty()` is false. No departure, no universe switch, and no app-close flush ever schedules a write for it, and the residual-dirty close marker does not count it. PJ-207 §15 taught four other arbiters to ask `hasUnsavedRecovery` alongside `isNoteDirty` and left these. The only remaining copy is the localStorage net that **PJ-110 already proved non-durable**. **Group 1 · ①.**
> - **PJ-289** *(MED · cross-note-bleed)* — **`SecondScreenCockpit.svelte:108`**: on a note switch the cockpit renders the NEW note's title and body immediately while still showing the PREVIOUS note's backlinks, outgoing links and review gauges until two async IPCs resolve — and the `loading` flag it computes **is never rendered**, so there is no cue that the panels belong to a different note. **Group 1 · ②.**
> - **PJ-290** *(MED · index-divergence)* — **`SecondScreenCockpit.svelte:80`** requests backlinks with `aliases: []` while the main window passes the note's real aliases, so **every alias-routed backlink is silently absent** from the second screen's knowledge graph. Two windows, two answers, no indication. **Group 1 · ②.**
> - **PJ-291** *(LOW · index-divergence)* — **`SecondScreenPage.svelte:150`**: `followExternalRename`/`followPeek` move the second screen's tab path and title but never refresh its CONTENT, and the channel the code's own comment defers to (`screen:note-saved`) is never emitted for a rename — so the second screen renders the note's pre-rename frontmatter indefinitely. **Group 2.**
>

> **What changed in v1.86** (**MIG-111 Phase 0.3 SHIPPED — the ledger's cross-process lock, with FOUR inspection findings fixed before the commit. The tenth whole-app sweep ran INCOMPLETE; its nine other findings are FILED here as PJ-278…PJ-283, and a v1.85 miss is corrected as PJ-285**):
>
> **► NEXT ACTION — the remediation pass on PJ-278…PJ-283, then MIG-111 Phase 0.4** (the five unguarded writers, which completes Phase 0). The remediation goes first because six of the nine findings sit squarely in the Boss's ①+② priority — knowledge actively lost, and the index lying about your notes — and WA#6 forbids shipping a known finding.
>
> ### ✅ MIG-111 Phase 0.3 — the `link_life` ledger cross-process lock (R5 · Architect condition 5)
>
> The ledger's `FILE_LOCK` was a process-local mutex — invisible to a second instance, so a compaction over there could rename the tail out from under an append over here, and a lost *decision* is not merely absent: the next boot's fold carries the pre-decision value and **writes it back**. Now a two-lock guard (in-process mutex + exclusive OS lock on `<dir>/ledger.lock`, fs4). Verified with two REAL processes: every line from both survives intact, exactly once. Rust **1467/0** (+4 tests).
>
> **Four findings fixed before the commit, each caught by the per-build gate** (three of them defects the 0.3 diff itself introduced — recorded plainly, because a gate that only ever confirms other people's bugs is not being read honestly):
> 1. the OS wait was **unbounded** (`lock_exclusive` inside the process-global mutex) — one stuck foreign holder would have frozen every ledger write in the process forever. Now `try_lock_exclusive` on a 5 s budget.
> 2. `read_state` read its **two files under no lock** (pre-existing) — a compaction landing between them makes the reader fold the old snapshot against an absent tail, and an absent tail is "empty store", not an error. The guard now lives inside `read_state`.
> 3. both fallbacks announced the degradation with **`eprintln!`**, which does not exist in a Windows GUI release build — erasing the only evidence that a documented loss path had been re-opened. Now `diag_log`.
> 4. compaction **proceeded without cross-process exclusion**. The append path's "proceed anyway" reasoning is circular for the rename that CREATES the aside copy nothing reads back. It now returns `Refused` — free, since compaction is optional and re-runs every boot.
>
> ### 🆕 PJ-278 … PJ-283 — the tenth sweep's findings, individually numbered
> - **PJ-278** *(MED · index-divergence · THREE sites, one class)* — **the embed-parity class has three more holes.** PJ-207 §15 gave four save paths a re-embed and documented that a skipped embed never self-corrects (the boot backfill only embeds notes with NO vector; repair and reindex never touch `note_embeddings`). Three durable-save paths were missed: **`commitFocusSave`** (`+layout.svelte:1795` — Focus is the *designated fast-capture editing surface*, so a body written there leaves semantic search answering from the pre-Focus text for good), **`retrySaveFailure`** (`store.ts:528` — which also never notifies the second screen, so the banner clearing reads as full success while a companion view stays stale), and **`resolveConflictMerge`** (`store.ts:6182` — a merge replaces the whole body by construction). Whole-Ecosystem Fix Law: **one shared after-durable-save helper**, not three patches. **Group 1 · ② "the index lying about your notes".**
> - **PJ-279** *(MED · false-success · index-divergence)* — **`resolve_structural_conflict` (`libraries.rs:2641`) swallows its reindex error** with `let _ =` and never calls `ensure_search_db_ready` — the one gated-RMW sibling that missed both 2026-08-01 treatments (the adjacent branch three lines below *does* log its skip). The frontmatter lands on disk, the watcher is suppressed, the command returns Ok and emits `cascade:rewrote`, and the index keeps the old parent/contains. **Retrying is a dead end**: the disk is already resolved, so `gate_rmw` returns `OkUnchecked` and the early-return skips the reindex forever. **Group 1 · ②.**
> - **PJ-280** *(MED · cross-window-clobber)* — **the review write pair is half-pinned** (`review.rs:763`, same shape at :791 and :818). The 2026-08-01 fix pinned the `review-pulse.json` RMW to the universe the click was composed in; `sync_action_to_row` still resolves the **ACTIVE** universe's `SearchState`. A universe switch between the two halves puts a ghost row for A's path into B's `review_schedule` (snooze's UPDATE silently matches 0 rows), and since `upsert_schedule_row` re-derives from the ROW, not the pulse, the note **keeps resurfacing as due** in A — PJ-187's exact symptom, reintroduced through the unpinned half. **Group 1 · ②.**
> - **PJ-281** *(MED · silent-data-loss · PERMANENT)* — **`read_universe_collections` (`universe.rs:1889`) swallows the legacy adoption read.** On the first boot after upgrading a pre-MIG-092 universe, one transient read failure of `workbench.json` (AV/sync sharing violation — the exact Windows case `read_persisted_json` was hardened against) returns `Ok([])` instead of an error. The frontend treats a resolved empty array as a successful read, seeds a Starred collection, and persists it — **creating `collections.json` and closing the `!path.exists()` adoption gate forever.** The user's entire Workbench collection set never appears again; `workbench.json` sits on disk unadopted with no error and no `.migrated` marker. Two earlier fixes hardened this block's WRITE side and left the read. **Group 1 · ① "actively losing knowledge".**
> - **PJ-282** *(MED · illusory guard)* — **CECE's per-cataloger timeout cannot unblock** (`cece/orchestrator.rs:153`). The worker runs inside `std::thread::scope`, which joins before returning, so after `recv_timeout` fires the function still blocks for `classify()`'s full duration. The V3-§8.r4.4 "hang fix" and Architect §10 invariant 12 ("ensemble timeouts are bounded") are **silently false**, and the abstain trail claims the cataloger was "isolated by orchestrator" when it was not. Measured over-budget by ~16× on real data (a ~32 s embed against a 2 s budget; the batch embed holds the engine mutex across every inference). No data loss — a false claim of a bound. **Group 2.**
> - **PJ-283** *(LOW · fire-and-forget)* — **the on-save background classification setting is a silent no-op in the normal typing state** (`NoteEditor.svelte:333`). It dispatches a window event whose ONLY listener lives in a mounted `SourceReviewPanel`; with no Source Review panel and no Cataloger open, nothing runs, nothing logs, and the review queue never fills — contradicting the setting's own description. The dependency is provably known: both sibling dispatch sites force-open the Cataloger and defer a frame first. **Group 2.**
>
> ### 🆕 PJ-284 — the tenth sweep must be FINISHED
> The whole-app cycle sweep of 2026-08-15 **did not complete**: 3 of 14 hunt groups (**notemodel-ownership, cross-window-integrity, freeze-and-leaks**) and 18 verify agents terminated on a model quota. Those scopes are **UNSWEPT, not clean**, and the cycle cannot be declared closed on a partial register. Re-run them before the MIG-111 cycle close. Register banked as `SWEEP-2026-08-15-tenth-whole-app-INCOMPLETE.json`. **Group 1 · Charter.**
>
> ### 🩹 PJ-285 — a v1.85 MISS, corrected
> The ninth sweep (Phase 0.2) recorded "two pre-existing `migrate_legacy_data` findings join the triage pile" **in the session log only — they were never filed here.** That is precisely the drift SO#9 exists to make structurally impossible, and it happened at the immediately preceding step. Filed now: **`universe.rs::migrate_to_constellation` / `migrate_legacy_data` swallow bases-copy errors**, so a one-shot legacy migration can report success having silently not copied part of the user's data. Re-verify both against current source before fixing (the sweep that found them is two commits old). **Group 1 · ①.**
>

> **What changed in v1.85** (**MIG-111 PLAN APPROVED — seven of eight §0.5 rulings taken; the build cascades. PJ-224 RULED after weeks gated. PJ-262 sequenced INTO MIG-111, before its Phase 2**):
>
> **► NEXT ACTION — MIG-111 Phase 0, step 0.1 (the live-WAL `fs::copy` ban — "fixed FIRST").** R36's repeals table is awaiting the Boss's item-by-item ratification after his "provide more details" — the details are delivered in chat; ratification closes §0.5 fully. Phase 1 may proceed on the taken rulings meanwhile (the Plan's gate is satisfied for everything except the Phase-4 text repeals R36 governs).
>
> ### ⚖️ THE §0.5 RULINGS (Boss, 2026-08-12)
> 1. **Plan APPROVED** — Plan-Approval-Equals-Build-Approval is in force.
> 2. **C1: YES** — Undo replaces confirmation ceremonies. Every cross-universe operation ends in a quiet receipt with genuine, engine-level Undo (R22).
> 3. **C2: persistent quiet line** on a locked universe's read-only note.
> 4. **C3: DELEGATED to the Art Director & team** (per the 2026-07-10 ruling) — tab identity spec is theirs to settle in Phase 1.5.
> 5. **R31: "Linked Universe" ADOPTED** — the Boss: *"Linked Universe is more logical… but you have a lot of renaming to do, from cUniverse to Linked Universe."* Scope recorded honestly: every USER-VISIBLE string ×15 locales, help files, User Manual ×15, and docs — filed as **PJ-277** (Phase 1.5 carries it). Code-internal identifiers (`cuniverse` iconKind, `cu0..cu24` ATTACH aliases, struct names) are NOT renamed in the same pass — churn across load-bearing code for zero user-visible gain; noted here so the decision is never silent. If the Boss wants the code identifiers renamed too, say so and it becomes its own hygiene PJ.
> 6. **R35: YES — PJ-262 (the Living-Link disk layer) ships BEFORE MIG-111 Phase 2.** The sequence is now: Phase 0 → Phase 1 (Router) → **PJ-262 as its own `/migration`** → Phase 2 (transfer engine, structurally smaller: earned knowledge becomes files that travel) → Phase 3 → Phase 4.
> 7. **R36: details requested** — delivered in chat for item-by-item ratification.
> 8. **PJ-224: RULED — YES**, the plain search box spans the umbrella by default. **PJ-224 CLOSES** after weeks gated; **PJ-207 §13 is UN-GATED** (its implementation folds into MIG-111 Phase 4 with the search-federation work). 
>
> - **PJ-277** *(filed per ruling 5)* — the cUniverse → **Linked Universe** renaming: every user-visible string in all 15 locales, `docs/help.uConstellation.World/`, User Manual + 14 translations, concept papers' user-facing text. Lands in MIG-111 Phase 1.5 (R31), one commit, i18n-parity-gated. **Group 1 · MIG-111.** (**MIG-111 Plan DRAFTED and delivered for Boss approval — concept VALIDATED 5/5 by the Boss-mandated panel (37 binding requirements), uniqueness confirmed by prior-art, both plan drafts attacked to SOUND-WITH-AMENDMENTS and amended**):
>
> **► NEXT ACTION — the §0.5 Boss gate: approve the MIG-111 Plan + the seven rulings** (C1 ceremonies · C2 locked-universe presentation · C3 tab identity · R31 naming · R35 PJ-262-first? · R36 repeals · PJ-224). `docs/migrations/PJ-235-federation-boundary/MIG-111-PLAN.md`. **Phase 0 foundations (fs::copy ban, the real owner lock, ledger lock, five writers on-boundary) may proceed on Plan approval; Phase 1+ waits on the rulings.**
>
> ### 🏛️ MIG-111 — Plan phase complete (2026-08-12)
> The Boss mandated panel validation: **5/5 chairs VALIDATED-WITH-REQUIREMENTS** (inspector, auditor, UX, Art Director & team, knowledge-formulation) — the safety chair's reframing: *the status quo IS the dangerous state* (22 documented broken silent crossings). 37 requirements bind the Plan; 3 chair conflicts go to the Boss. Prior-art (WA#5): **no shipping PKM product does this** — Obsidian "will remain impossible", Notion's cross-workspace move is a lossy copy, DEVONthink/Tana each solve half. The philosopher's line heads the Plan: *"sovereignty with seamlessness — each corpus keeps its own truth; the mind over them is one."*
>
> Two adversarial attacks on the drafts, both SOUND-WITH-AMENDMENTS, every amendment folded in — notably H1 (routed maintenance would have RUN with parent vocabulary, writing parent aggregate values into child rows — the harness now diffs VALUES), H2 (the owner resolver would have admitted any unlinked universe on disk — now intersected with the federation tree, fail-closed), H4 (Windows mandatory locks would have made the refusal message unreadable — owner info moved outside the locked range), B3 (child prep never on the debounced save), B4 (routing ships before any door; the Architect §6.3 wave order is corrected).
>
> Panel + plan evidence banked: `MIG-111-CONCEPT-PANEL.md`, `MIG-111-PLAN-EVIDENCE.md` (112 KB), `MIG-111-PLAN.md`. (**MIG-111 allocated and its ARCHITECT PHASE COMPLETE — full cross-universe operations, per the Boss's reframing ruling. Option A (route-to-owner) recommended, viable-with-conditions. Phase 2 awaits five Boss decisions**):
>
> **► NEXT ACTION — MIG-111 Phase 2 (Plan): the Boss's five decision points** in `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT.md` §6 — confirm the repeals table, the door's UI shape, the wave order, the two-instance policy, and PJ-224 (now in frame). **No build until the Plan is approved.**
>
> ### 🏛️ MIG-111 — Architect phase closed (2026-08-12)
> The Boss's governing ruling, at the PJ-235 interim's pass: *"I want to be able to conduct full functions/operations between universes… why did I design Constellation to have a cUniverse(s) if I wasn't planning to have full access and/or operations among them? … That's why Constellation are unique."* Federation is ONE knowledge space with full agency — completing his 2026-07-05 "It is ONE universe" ruling; the read-only contract was the implementation's assumption, never the design.
>
> **The mapping** (12 agents, every claim file:line-verified; evidence banked at `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT-EVIDENCE.md`, 196 KB): the per-universe DB model and the four existing second-DB open sites; the 22-site write register re-verified; the 30-table earned-data census (the transfer cargo manifest); cid_cn's per-universe uniqueness and the collision→invisible-note→severed-rows chain; cross-universe links living only in the author's DB (the target's universe never learns — and inbound links break silently on rename, the PJ-253 family, now in scope); three pre-existing cross-process hazards that become critical under any write-capable federation (process-local ledger lock; the WAL false-negative instance probe; fs::copy backup/restore in federation/migrate.rs).
>
> **Options, adversarially attacked:** **A route-to-owner — VIABLE-WITH-CONDITIONS, RECOMMENDED** (the only shape keeping universes self-contained; 7 blocking conditions scheduled into the Plan). **B provenance-index** — viable but carries two app-killer cross-process holes and a permanent two-copy sync contract; not recommended. **C single-store — NOT VIABLE AS WRITTEN**: FTS5 breaks structurally (notes_vocab has no per-universe form; BM25 statistics go machine-global — registering universe B would change ranking inside universe A; top-K loses its index-only path) and no durable universe identity exists to key rows by.
>
> **PJ-276 is subsumed** into MIG-111 (move = Wave 1 of the operation surface). PJ-270…275 become the migration's correctness cases. PJ-253's cross-universe half folds into Wave 3. (**PJ-235 + PJ-254 PARTIAL — committed as interim by PANEL VERDICT (2–1), not closed. The migration is accepted and Architect input is banked. PJ-270…PJ-275 filed for the residue**):
>
> **► NEXT ACTION — the PJ-235/PJ-254 `/migration` Architect phase** (Boss-accepted 2026-08-11), reading `docs/migrations/PJ-235-federation-boundary/ARCHITECT-INPUT-federated-write-sites.md` — a verified enumeration of **22 federated write sites**. **PJ-207 §13 still GATED on PJ-224. PJ-253, PJ-219, PJ-260 still BLOCKED on Boss rulings.**
>
> ### ⚖️ BOSS RULING at the interim's pass (2026-08-12) — the migration's goal is REFRAMED
> Passing Parts A and B, the Boss asked: *"Why can't I move files to/from cUniverses? I would like to be able to do that."*
>
> **Filed as PJ-276, and it becomes the migration's headline requirement.** The goal is no longer "seal the federation boundary" — it is **"seal the SILENT crossings, and build a proper door":** cross-universe move as a first-class, deliberate operation. What made the old behaviour an app-killer was never the crossing itself; it was that the crossing was an *accident* with broken bookkeeping — the note's earned knowledge (link weights, confidence, traversal counts, archival decisions, review schedule) lives ONLY in the source universe's `search.db`, so it did not travel; the destination indexed the note as brand-new; the source kept a ghost row serving stale text. A real cross-universe move must carry the earned data across both databases, update both sides' indexes, re-resolve links, and be explicit in the UI. The interim just committed is the wall; PJ-276 is the door. The Architect phase now designs BOTH (and decides the UI shape — e.g. whether the Move picker re-lists linked universes under a clearly-marked section with confirmation, or a separate "Move to another Universe…" command — as a Boss-ruled option set in the Plan).
> - **PJ-276** *(FEATURE · Boss-wanted 2026-08-12 · migration headline)* — **deliberate cross-universe move, with full both-sides bookkeeping.** Subsumes PJ-270's defect framing (the unguarded OUT-of-linked-universe direction becomes this feature's correctness case rather than a hole to plug). **Group 1 · migration.**
>
> ### ⚖️ The disposition was ruled by a PANEL, at the Boss's direction
> The engineer's fix was found wrong twice by adversarial review (a guard that could not refuse the nested-cUniverse case its own test then documented; a Move-picker narrowing that erased the federation walk boundary; the same boundary error repeated in `canonical.rs`). The Boss accepted the request that the remaining work become a full `/migration`, and mandated that "the inspectors and auditor choose" what happened to the interim diff. **Three independent reviewers ruled 2–1: COMMIT AS INTERIM**, under blocking conditions — all executed before the commit:
> 1. **Pure guard + wiring test** — `require_own_library_in(own, foreign, path)`; the test drives the GUARD, and deleting its foreign-root check turns the suite red. (The dissent's sharpest finding: the previous tests exercised only the primitives, so the whole suite stayed green with the check deleted.)
> 2. **False comments corrected** — three sites claimed `auto_canonicalize_all` "runs at STARTUP". Verified false: it has NO caller; the startup path is `repair_external_libraries_on_startup` (+layout.svelte:3466). The correction is recorded IN the comment, with the panel catch dated.
> 3. **`invalidate_libraries_cache()` added to `add_child_universe` AND `remove_child_universe`** — without it the foreign-root set stays empty for the whole session in which a universe is linked, i.e. the guard was inert exactly when first needed.
> 4. **Honest scope everywhere** — `move_item`'s comment now states the source side is NOT guarded; the two silent `None` reindex arms now `diag_log` per the file's convention.
>
> ### ✅ What the interim commit actually delivers (and what it does NOT)
> - `move_item` refuses a foreign DESTINATION — external and nested topologies both, via own-prefix + foreign-root set.
> - The Move picker and the template destination picker offer OWN libraries only, with the federation-aware walk boundary intact (`walk_exclusions`).
> - **Eight write/reindex tails** moved off the federated resolver, incl. three `canonical.rs` surfaces that RENAME files, one of which is on the localStorage-gated startup path.
> - **NOT delivered** (the migration's scope): the tails are still prefix-only (blind to a nested cUniverse); `move_item`'s SOURCE is unguarded for plain notes/folders; the foreign set is best-effort and library-rooted; ~7 commands still authorise through the federated resolver; `constellation_search_reindex` trusts a frontend-supplied library name.
>
> ### 🆕 PJ-270 … PJ-275 — the residue, individually numbered (panel condition)
> - **PJ-270** *(HIGH · silent-data-loss)* — **`move_item`'s SOURCE is unguarded for plain notes/folders**: a note can be moved OUT of a linked universe into this one, destroying its earned link/review data (search.db-only, not recomputable). `guard_no_foreign_library_under` refuses only a source that IS/CONTAINS a registered library. **Group 1 · migration.**
> - **PJ-271** *(HIGH · index-divergence)* — **the eight write tails are prefix-only**: a cUniverse nested under the active root still resolves to `universe_notes`, and the row now carries an OWN library name. Pinned by `pj235_a_cuniverse_nested…` (the deliberate "documents the hole" assertion). **Group 1 · migration.**
> - **PJ-272** *(HIGH · fail-open)* — **the foreign-root set is best-effort and library-rooted**: an unreadable child registry yields an empty set (fail-open, pinned by the guard's own empty-set test), and a linked universe with no library at its root is invisible to it. The migration should consider cUniverse ROOTS (`resolve_child_universe_roots_recursive`) or an explicit provenance column. **Group 1 · migration.**
> - **PJ-273** *(HIGH · unguarded write door)* — **~7 commands still authorise through `validate_path_in_any_library`** (write_note, create_note, create_folder, rename_item, delete_path, quick_capture, canvas/import/base writers — full register in the Architect input), and **`constellation_search_reindex` takes its library name from the frontend unchecked** (~20 callers pass `tab.libraryName`). **Group 1 · migration.**
> - **PJ-274** *(MED · perf)* — `load_libraries` is UNCACHED (the cache serves only `load_all_libraries`), so the converted tails re-read `libraries.json` per call, and a transient read failure yields `vec![]` → a logged-but-silent reindex skip. The migration's loader design should give the own set the same cache + the same fail-closed posture. **Group 2 · migration.**
> - **PJ-275** *(MED · drift)* — **three independent "is this foreign" predicates** (Rust `foreign_library_roots`; frontend `ownUniverseLibraries`/`isChildUniverseLib`; the builder's inline filter) with separate failure modes; and `load_libraries_pub` is a misleading alias of the FEDERATED loader with one caller (`create_base`) using it for write auth. One predicate, one name — migration scope. **Group 2.**
>
> **What changed in v1.81** (**PJ-234 + PJ-240 CLOSED and Boss-validated — the blank-line block-drop is dead at FOUR surfaces, and the wrong predicate is deleted from the codebase. PJ-269 filed: a third frontmatter parser, found by the gate**):
>
> **► NEXT ACTION — `PJ-258`, then the `PJ-235 + PJ-254` federation-boundary family** — continuing the Boss's ①+② priority (M2). **PJ-207 §13 still GATED on PJ-224. PJ-253, PJ-219, PJ-260 still BLOCKED on Boss rulings.**
>
> ### ✅ PJ-234 + PJ-240 — CLOSED, Boss-validated
>
> **The rule.** When a writer replaces a list-valued property it must first delete the old list's lines. The rule it used — `is_block_value_line` = *"a `- item` or an indented line"* — is **false for a BLANK line**, so a blank inside the list ended the deletion early and stranded every item after it under the new scalar. That is a sequence with no key: **unparseable YAML**, which is precisely the state in which every LATER property edit on that note is silently discarded. One bad write disables that note's property panel permanently.
>
> **Reproduce-first.** RED reproductions written before any fix, at every site; the first run returned `topics: gamma` followed by an orphaned `- beta`, exactly as predicted.
>
> **FOUR surfaces, not three.** The Whole-Ecosystem sweep for the three known sites found a fourth carrying the same concern in a different shape:
> 1. `bases.rs::update_frontmatter_property` — replacing a property (reached by a **Base table cell edit**)
> 2. `bases.rs::remove_frontmatter_property` — removing a property
> 3. `libraries.rs::set_frontmatter_parent` — setting a note's structural parent (PJ-240)
> 4. `libraries.rs::merge_initial_frontmatter` — **creating a note FROM A TEMPLATE**: it dropped the template's filtered identity key (`kind:`/`title:`/`cid_cn:`/`created:`) but KEPT that key's list items, so a brand-new note was born with unparseable frontmatter. Flagged MED and unnumbered in the seventh sweep; closed here rather than left as the one that got away again.
>
> **The wrong answer is DELETED.** `is_block_value_line` had zero callers after the sweep and is **removed from `yaml_lines.rs`**, with a comment in its place explaining why. A wrong predicate left in the codebase is how this defect reached its fourth and fifth shapes; `ends_dropped_block` is now the only answer, shared by all four writers plus the two `sources/mod.rs` loops PJ-207 §15 had already converted. **Five drop loops in Rust, one rule.**
>
> **Gates:** Rust **1452/0** (7 new tests) · vitest **941/0** · release binary **2026-08-11 14:46** · Boss-validated Stage 1 (Base-table cell edit, disk + panel). `tutorial-auditor` → `ui-inspector`: **REJECTED ×2, APPROVED on the third** — one rejection was a byte-level catch (the "broken" illustration omitted a blank line the pre-fix code really emits), one was my own correction introducing a promise Steps 4–5 made impossible.
>
> ### 🆕 PJ-269 — the indexer stores every block list as an EMPTY STRING
> *(HIGH · index-divergence · found by the `ui-inspector` gate while verifying the PJ-234 Boss test)*
>
> **`search.rs::parse_frontmatter` (:6630) skips every sequence item** and then stores the bare `topics:` key with its empty inline value, so `note_meta.properties_json` records **`"topics": ""`** for a note whose file holds three items. Everything reading that column shows blank: **Base tables**, lens queries, filters on a list property. `tags:` is special-cased and unaffected; every OTHER list key is not.
>
> **This is the PJ-252 disagreement in Rust, and it is the THIRD parser:** `bases.rs::parse_frontmatter` (:150-171) DOES join block-list items into `"alpha, beta, gamma"`; `search.rs::parse_frontmatter` returns `""`; the TS side has its own. **Filed, not fixed in this pass** — it is a read/indexer change whose blast radius is every Base table and lens query and which needs a reindex to take effect. **Group 1 · ② "the index lying about your notes".**
>
> ### 📌 Process notes from this close
> - **The Boss ruled the sequence**: *"I want us to tackle 1 + 2 as a priority."* The readiness plan is amended — **M2 first**, then M3, then M1 (**PJ-262**) and the M0 scope ruling (**PJ-263**). Recorded in the plan's own BOSS RULING block.
> - **v1.80 was edited in place after being committed**, which the SO#9 discipline forbids (never overwrite; the trail is durable). Corrected: v1.80 restored to its committed state and this v1.81 carries the delta. The slip is recorded rather than quietly fixed.
> - `propertyEditor.valuePlaceholder` is **dead i18n** — present in all locale files, referenced nowhere in `src/`. Hygiene, folded under PJ-261.
>
> **What changed in v1.80** (**PJ-252 CLOSED and Boss-validated across five test rounds — the APP-KILLER is dead, and the fix caught a regression of its own before the Boss ever saw it. Two more blank-line siblings fixed. Two more whole-app sweeps (54 confirmed) banked**):
>
> **► NEXT ACTION — `PJ-234 + PJ-240`, per the BOSS RULING of 2026-08-11: "tackle 1 + 2 as a priority".** The readiness plan (`docs/Constellation Readiness Plan v1.0.md`) is the authority on sequence and is **amended**: **M2 runs FIRST** — ① what actively corrupts or loses knowledge, and ② what misplaces notes or makes the index lie — then M3 (PJ-264 triage), then M1 (**PJ-262**, the Living Link disk layer) and the M0 scope ruling (**PJ-263**). **PJ-207 §13 still GATED on PJ-224. PJ-253 (inside ②), PJ-219 and PJ-260 still BLOCKED on Boss rulings.**
>
> ### ✅ PJ-252 — CLOSED, Boss-validated
>
> **One classifier.** `yamlDoc.classifyDoc` / `classifyFrontmatterValues` is now the single answer to *"what is this key's value?"* (`list` · `structured-list` · `block` · `scalar`), read by all three places that used to answer it separately: `store.parseFrontmatter` (the panel's projection, **including the list's item VALUES**), `composeFrontmatter`'s `immutableBlockKeys` (the write path's refusal), and `projectProps`. `composeFrontmatter` now parses the document ONCE — it previously parsed twice, for the H1 gate and again for the refusal — so the classification and the refusal cannot come apart.
>
> The refusal is stated **closed** (`WRITABLE_KINDS`): a fifth `FmValueShape` added to the union arrives REFUSED rather than silently writable. `seqCarryingComments` carries an edited list's comments across the splice-and-append, so making those lists editable did not trade a destroyed list for a destroyed comment.
>
> **Two more shapes of the same disagreement, found by RUNNING the path rather than reading it** — both fixed and pinned: an inline `- history   # why` was projected with the comment **inside the tag value** and written back as the literal quoted tag `"history   # why"`; and `whatever: [alpha, beta]` under a key `detectPropertyType` does not know was typed `text` valued `alpha, beta`, so editing the note wrote the sequence back as that string.
>
> ### ⚠️ The regression this fix nearly shipped — and what caught it
>
> Routing the projection through the `yaml` library made **how the YAML text is extracted** load-bearing for the first time. `parseFrontmatter`'s `rawYaml` is `yamlLines.join('\n')`; on a CRLF note every line still ends in `\r`, and `join` puts a separator only BETWEEN elements, so **the last line's `\r` was unterminated and the library read it as DATA**. The final property of a Notepad-saved note came back as `snurfle\r` and was written as the quoted tag `"snurfle\r"` — on an ordinary plain tags list, far commoner than any shape PJ-252 set out to fix.
>
> **The `ui-inspector` gate found it by running the real chain, and REJECTED the test.** Its attribution ("pre-existing") was the one thing wrong: reproduced against HEAD, the same note round-trips clean before the change. It was ours. Fixed by handing the classifier `splitFrontmatter(content).yaml` — the exact bytes `composeFrontmatter` composes from — rather than a second extraction of the same region. **This is the case for the test pipeline stated in one line: the gate is not ceremony, it caught an APP-KILLER-class regression that every one of 939 green tests had missed.**
>
> ### 🧹 The blank-line family — three siblings, one concern
> Seen on screen in the Boss's own probe note, then traced to **two independent causes**:
> - **`canonical.rs::ensure_cid_cn`** re-emitted the frontmatter slice under a fresh `---\n` while the slice itself BEGAN with the newline — a blank line above the first property on the one pass that injects a note's identity, i.e. the first time any note is ever opened. The third sibling of the defect PJ-207 §15 fixed in `update_frontmatter_title` and `set_frontmatter_parent`, and the only one it had not reached. Split out as a pure `inject_cid_cn` so the shape is testable without disk; **3 Rust tests**. *(Boss ruling: `cid_cn` is Constellation's note-identity system working as designed — the fix moves the newline and touches neither the value nor the naming.)*
> - **`composeFrontmatter`** — splicing the block's FIRST key empties the CST residue, and `+= eol` on an EMPTY string invented the line. **Pre-existing**: measured at HEAD, byte-identical. The subtlety worth keeping: at that point a blank line the user typed and one the splice left are *indistinguishable* — the old code preserved the user's only by the same accident that fabricated one. The user's blank is now restored from `rawYaml`, the file's own bytes, which is the only place it still exists. Both cases pinned.
>
> ### 🆕 NEWLY FILED — PJ-258 … PJ-261
> - **PJ-258** *(HIGH · content-corruption · from the seventh sweep)* — **`propRow.ts:101`** `listItemsOf`'s scalar fallback splits on a raw comma (`String(p.value).split(',')`) instead of the quote-aware `splitFlowSeqItems` that `parseFrontmatter` uses for the identical job. **Directly relevant to PJ-252's close:** `/simplify` proposed routing `serializeLine`'s list fallback through `listItemsOf`; that was skipped for a behaviour-change reason, and the sweep then confirmed the helper carries its own defect. **Group 1.**
> - **PJ-259** *(MED · altitude residue from PJ-252)* — the one classifier settles *kind-of-YAML-node*, not *kind-of-property*: `detectPropertyType` (`store.ts:199-236`) and `PropertyEditor.svelte:469-485` still independently answer "is this key a list". **Block EXTENT** is also still line-decided (`blockExtent`), so kind can no longer disagree but extent still can. Neither is reachable as data loss today (the write path refuses from the file), and changing them changes behaviour — hence filed, not folded in. **Group 2.**
> - **PJ-260** *(LOW · hygiene · BOSS RULING PENDING)* — the Rust frontmatter writers hardcode `\n` for the fence and injected lines regardless of the note's own endings, so a CRLF note ends up with **mixed line endings**. Measured in the Boss's live probe note. Nothing renders differently in Notepad, Constellation or git. Surfaced to the Boss with an offer; **no ruling yet**. **Group 4.**
> - **PJ-261** *(MED · doc-drift)* — `deriveTabName` (`store.ts:3230`) carries the comment *"For canonical files, extract title from frontmatter"* but applies the frontmatter `title:` to **every** file, canonical or not. Surfaced when the Boss asked why a note named `pj252-test-1` displayed as "Zarquon Test One" — the behaviour is correct and by design; only the comment is wrong. **Group 5.**
>
> ### 🎯 READINESS — PJ-262 … PJ-268, and a re-ranking of the whole backlog
> The Boss commissioned a holistic readiness evaluation (`docs/Constellation Readiness Review
> v1.0.md`) and a plan (`docs/Constellation Readiness Plan v1.0.md`). **The plan re-ranks every
> open PJ by distance-to-publishable rather than by severity, and is the authority on sequence
> from here.** Seven numbers were allocated for readiness work that had no owner:
>
> - **PJ-262** *(APP-KILLER-class · `/migration` · Boss-directed 2026-07-24)* — **the Living Link
>   disk layer.** Verified again today: **no code writes a LINK file**; `weight`, `confidence`,
>   `traversal_count`, `last_traversed` and `status='archived'` live ONLY in `note_links` inside
>   `search.db`. So **File Over App is violated for the earned half of every link**, and *"every
>   link operation must be reversible"* is **false** — rebuilding the index resurrects every
>   archived link. `search.db` is a system of record, not a cache. **The only CONCEPT failure in
>   the product, and rank 1 of the whole backlog.** **Group 1 · Charter.**
> - **PJ-263** *(BOSS RULING REQUIRED, then execution)* — **surface-area certification or cut.**
>   28 concept papers say *"Enabled in bring-up: no"* while `store.ts:7012` ships ~25 functions
>   ON. The bring-up acceptance program was never completed. **The highest-leverage decision
>   available** — it prices M4, M6 and much of M2. **Group 3.**
> - **PJ-264** *(process)* — **triage the ~100 unnumbered sweep findings** across six registers
>   (177 confirmed: 3 APP-KILLER — all closed — - 59 HIGH, 79 MED, 36 LOW). De-duplicate, drop
>   what is fixed, number and group the rest. Until it is done, every rank below it is a ranking
>   over the findings we NUMBERED, not the findings we HAVE. **Group 1.**
> - **PJ-265** — **kill-mid-index recovery**, boot ship-gate criterion 5, *not implemented*.
>   **Group 1.**
> - **PJ-266** — **idle RSS ≤ 350 MB**, boot criterion 3, never measured. **Group 2.**
> - **PJ-267** — **macOS: build → sign → notarize → smoke-test.** CI is `windows-latest` only;
>   `bundle.macOS` is null; never built, never launched. App code is mostly platform-neutral
>   (proper `#[cfg]` arms; only 3 bare `ctrlKey` sites vs 29 handling both) — infrastructure work,
>   not a rewrite. **Group 3.**
> - **PJ-268** — **backup & recovery system**, Boss-wanted 2026-06-21; concept paper already
>   written. **Group 3.**
>
> **Readiness exit criteria: 2 of 7 met** (no known-live app-killer ✅; everything else open).
>
> ### 🆕 PJ-269 — the indexer stores every block list as an EMPTY STRING
> *(HIGH · index-divergence · found by the `ui-inspector` gate while verifying the PJ-234 Boss
> test, 2026-08-11 — it ran the chain and got `{"topics": ""}` where the file holds three items.)*
>
> **`search.rs::parse_frontmatter` (:6630) skips every sequence item** (`if is_seq_item(line) {
> i += 1; continue; }`) and then stores the bare `topics:` key with its empty inline value. So
> `note_meta.properties_json` records **`"topics": ""`** for a note whose file says
> `- alpha / - beta / - gamma`. Everything reading that column shows blank: **Base tables**
> (`discover_base_properties` → `resolve_dim` → `json_extract`), lens queries, and any filter on
> a list property. `tags:` is special-cased and unaffected; every OTHER list key — `topics`,
> `aliases`, `authors`, any custom one — is not.
>
> **It is the PJ-252 disagreement in Rust, and this is the THIRD parser:** `bases.rs::parse_frontmatter`
> (:150-171) DOES join block-list items into `"alpha, beta, gamma"`; `search.rs::parse_frontmatter`
> returns `""`; the TS side has its own. Two Rust readers of one file format, disagreeing.
> **Filed, not fixed in the PJ-234 pass** — it is a read/indexer change whose blast radius is every
> Base table and lens query and which needs a reindex to take effect, so it earns its own pass
> rather than riding along. **Group 1 · ② "the index lying about your notes".**
>
> ### 📋 Two more sweep registers, durable in the repo
> `lab/reports/sweeps/SWEEP-2026-08-11-sixth-whole-app.json` (**30** confirmed) and `SWEEP-2026-08-11-seventh-whole-app.json` (**24** confirmed). **Neither run was diff-scoped** — `args.files` did not reach the script both times, so both went whole-app; the frontmatter write path was covered in full either way, and **not one confirmed finding lies in the PJ-252 diff**. Their headline items are PJ-258 and the already-filed PJ-254/PJ-235 family. As with the fourth and fifth registers, **the remainder are not yet individually numbered** — the honest state, and that triage is now a **four-register** backlog.
>
> **What changed in v1.79** (**PJ-249 CLOSED and Boss-validated — rename ~50 s → 216 ms, after discovering the index it shipped had NEVER been used. Two more whole-app sweeps (54 confirmed). One APP-KILLER filed UNFIXED with its exposure measured. Ultracode**):
>
> **► NEXT ACTION — PJ-252 (the APP-KILLER: adding a tag deletes the tags already there), then the PJ-234 + PJ-240 pair, then PJ-235.** PJ-249 is closed. PJ-252 outranks the carried Group-1 queue because it is an *active silent-data-loss path* in the frontmatter write layer — the same layer whose three siblings were closed at §15 — and because it is already **reproduced and pinned**; the fix is one shared predicate, not an investigation. Its live exposure is **1 of 10,077 notes**, measured, so it is urgent by class, not by blast radius. **PJ-207 §13 remains GATED on PJ-224 (Boss ruling).**
>
> ### ✅ PJ-249 — CLOSED, Boss-validated (and it did not close the way v1.78 described)
> v1.78 filed PJ-249 as *"normalise `note_links.target_name`"*. **The Boss ruled a different shape** at the Phase-1 gate — a **new clean column** (`target_base`, option d) rather than normalising the existing one, with **folder-qualified links in scope** — because rewriting `target_name` in place would have destroyed the wikilink's own spelling, which the rewriter needs. Built §1–§6h, four-phase, Boss-validated at every stage.
>
> **The close is worth reading, because the migration's own headline number was false for a week.** The seek shipped, the Boss's *second* rename measured 44 ms, and it looked finished. `EXPLAIN QUERY PLAN` on his live database then showed **`SCAN note_links USING INDEX idx_link_source`** — a full scan of 31,368 rows. **The index had never been used since the day it was created.** The 44 ms was the same scan served from the page cache; his *first* rename of each session paid **2,579 ms**. Cause: `sqlite_stat1` still reported `target_base` as having ONE distinct value across every row — true when collected (the column was all-NULL), falsified by the §4 back-fill itself, and never re-collected. `PRAGMA optimize` at boot cannot see it (it re-analyses on row-count movement) and in fact *manufactures* it, by photographing the column while it is still uniform.
>
> Fixed structurally rather than statistically: the index carries `source_path` so it **covers** the query — verified to hold with `sqlite_stat1` **deleted entirely** — plus a scoped `ANALYZE` so the false statistic is not left in the database for the next query written against the column.
>
> **Measured on the Boss's universe, end to end: `~50,000 ms → 2,878 ms → 1,673 ms → 216 ms`.** Final breakdown: `seek-query 20 ms | freshness-map 2 ms | freshness-net 71 ms [382 dirs, 2,109 .md, read_dir 16 ms, metadata 0 ms] | rewrite-done 216 ms`.
>
> ### 🆕 NEWLY FILED — PJ-252 … PJ-257
>
> **APP-KILLER**
> - **PJ-252** *(APP-KILLER · silent-data-loss · REPRODUCED, UNFIXED)* — **`src/lib/editor/yamlDoc.ts:225` vs `store.ts:2582`.** Adding a tag to a note whose frontmatter list carries a **comment line** or an item **wrapped across two lines** DELETES the entries already in that list, from the `.md`, with no error. Two classifiers disagree: `store.parseFrontmatter` is LINE-based (a comment fails its all-bare-items test → projected read-only with an EMPTY `value`), while `yamlDoc.immutableBlockKeys` asks the `yaml` library (comment attached as a comment, wrapped item folded to one scalar → "all scalars" → **not protected**). The mutator then rebuilds from `p.listItems ?? (p.value ? split : [])` = `[]`. Same route destroys a typed-link block via `addTypedLinkToProps` (`store.ts:1468`). **Reproduced against shipped code and pinned as `it.fails` in `tests/pj-249/yamlCommentInSeqDestroysBlock.test.ts`** — those three turn RED the moment the fix lands. **Exposure measured, not assumed: 1 of 10,077 live notes** across both universes (a probe note, on `authors`). **The fix is ONE shared predicate** — this is the fourth shape of a block the write path must refuse (2026-07-24 closed seq-of-maps; PJ-182 closed block scalars), and it is open precisely because each closure re-answered the question in a second place. **Group 1 · Charter.**
>
> **HIGH — ruling required before it can be built**
> - **PJ-253** *(HIGH · silent-miss · BOSS RULING REQUIRED)* — **the cascade's two halves disagree about case.** The seek folds (`target_base_of` → `fold_match_key`), so `[[meeting notes]]` IS returned as a candidate when renaming "Meeting Notes"; `cascade_pattern` (`libraries.rs:7205`) matches literally via `regex::escape`, so the link is read and **left naming a title nothing owns**, and the rename reports success. **NOT a PJ-249 regression** — the pre-PJ-249 walk used the same regex and missed it identically. It needs a ruling because fixing it **changes which links get rewritten on disk**. **Group 1.**
>
> **HIGH — from the two new whole-app sweeps**
> - **PJ-254** *(HIGH · index-divergence)* — **`libraries.rs:1776`** (and `:1849`, `:2654`, `create_note` at `:1334`) every rename / move / create write-path tail resolves its reindex library through **`load_all_libraries`, the FEDERATED resolver**, so touching a linked universe's note silently adopts it into the ACTIVE universe's index. The file states the contract against itself at `libraries.rs:266-269` (*an attribution resolver that "feeds a write or a reindex" MUST use `load_libraries`*), and `reindex_changed_paths` obeys it. **The exact class PJ-207 §8 closed at six other surfaces.** One family with **PJ-235** and the Ctrl+N picker bug fixed 2026-08-10 — *reading a universe-wide list is right for RESOLVING a name and wrong for CHOOSING where to write.* **Group 1 · Charter.**
> - **PJ-255** *(HIGH · cross-window-clobber)* — **`libraries.rs:1851 / :2655 / :6889`** none of the detached DB tails carries a universe/generation token, so a universe switch mid-tail writes the DEPARTED universe's note paths into the NEWLY-opened universe's `search.db`. `SearchState.federation_generation` exists and is bumped on switch; no tail checks it. Same shape as PJ-244/245/246 — **one generation-guard helper for all six, per the Whole-Ecosystem Fix Law.** **Group 1 · Charter.**
> - **PJ-256** *(MED · perf + hygiene)* — **no back-fill in Constellation re-collects statistics for the table it fills.** The only two `ANALYZE` sites (`sky_backfill.rs:138`, `links_backfill.rs:152`) both run at the START of their pass — the one position from which they cannot help — and `PRAGMA optimize` is structurally blind to a back-fill (it re-analyses on row-count movement). PJ-249 fixed the one instance that was *measurably* mis-planned; **14 back-filled columns are unindexed today and the first index or predicate added to any of them lands on a statistic nothing refreshes.** Recommended rule: a shared `finalize_stamp(conn, module, version, table)` so the scoped `ANALYZE` and the `schema_versions` stamp cannot be separated. Full register: `lab/reports/sweeps/AUDIT-2026-08-10-stale-statistics-class.json`. **Group 2.**
> - **PJ-257** *(MED · false-success · seen live in the Boss's log, twice)* — **`props_reparse` FAILS on every boot and re-arms itself forever** over 2 rows: *"completeness check failed: 2 row(s) still carry the phantom signature — not stamping, the next boot re-runs."* A repair that can never succeed and never stops trying, with no surface. **Group 2.**
>
> ### 📋 The two new sweep registers, durable in the repo
> `lab/reports/sweeps/SWEEP-2026-08-10-fourth-whole-app.json` (**25** confirmed) and `SWEEP-2026-08-11-fifth-whole-app.json` (**29** confirmed). PJ-252/254/255 are their headline items; **the remainder are NOT yet individually numbered** — that is the honest state, and triaging them into PJs is itself the next ledger job after PJ-252. They are recorded in the repo rather than in a scratchpad precisely so that remains possible.
>
> ### ⚖️ One new law, and a register defect corrected
> **LL-044** — *a back-fill invalidates the statistics that describe the column it fills, and a correctness test cannot see a query PLAN.* And: **LL-037 and LL-038 had each been issued TWICE** (both by me, in this arc, both filed in numerical position so nothing looked wrong). Renumbered to **LL-042** / **LL-043**, every citation updated in `search.rs` and two session logs. A lesson with a colliding number cannot be cited.
>
> **What changed in v1.78** (**PJ-207 CLOSED except §13 — the cycle sweep ran three times over the whole app, 101 confirmed findings, 72 fixed including FIVE app-killers, 29 open; the two slowest things the Boss does every day got fast; 18 new PJs filed so nothing from the sweep lives only in a scratchpad — or only in the Charter. Ultracode**):
>
> **► NEXT ACTION — the third sweep's APP-KILLER pair (PJ-234 + PJ-240), then PJ-235, then open `/migration` Phase 1 on PJ-249.** PJ-207 §15 is closed and **§13 is still GATED on PJ-224 (Boss ruling required)** — so the migration cannot advance and the queue is now the sweep's own residue. PJ-234/PJ-240 are one rule applied to three writers that never got swept (`bases.rs:495`, `bases.rs:584`, `libraries.rs:2370` still call `is_block_value_line`; only `sources/mod.rs` was routed through `yaml_lines::ends_dropped_block` — **verified in the tree today**), and their output is unparseable YAML, which is the precondition for the frontmatter-discard app-killer this same sweep fixed. PJ-235 is a **data-placement** hazard: a note can be physically moved into a linked universe. PJ-249 (normalise `note_links.target_name`) is the gate on the last big user-visible slowness AND on the index being a faithful referrer list — **/migration-sized**, so it starts with an Architect doc, not a patch.
>
> ### ✅ PJ-207 §15 — CLOSED (the cycle sweep, three passes, and a fourth on the held items)
> **101 confirmed findings across three whole-app runs — 32 + 32 + 37; 72 fixed, none deferred; 29 open and filed below** (Boss ruled "Fix all three", "Fix the remaining", "Fix all 27, then one test", "fix all 5"). **Five app-killers**: frontmatter edits silently discarded on malformed YAML (now REFUSED at the model, all six mutators — the sixth, `setProps`, was the second pass's catch); workspaces carried across a universe switch; the `sources:` block-strip eating a comment or continuation (and, at the second pass, a **flush-left** comment the first fix's own test had pinned as working); a note whose bytes begin before its `---` fence getting a SECOND fence — **28 notes in the live universes already have that shape**, several Arabic, and an ordinary Accept in Source Review was the trigger; and the crash-recovery net erasing itself when a recovered note was merely looked at and the tab switched. Full narrative: `lab/reports/SESSION-LOG-2026-08-09.md` §15 (four passes).
>
> **The five items held for a ruling are all fixed** — the cascade now walks **every own library in the Universe** (rooted at the Universe root per MIG-108, stopping at a linked universe via `foreign_library_roots`), the cascade pattern now matches `[[type::Old]]` and `#heading` / `^block` anchors, `sky_nodes.cid_cn`'s UNIQUE index is partial, the `sky_links` rename is identity-qualified (measured: retitling one of seven same-titled notes had been rewriting **4,359 rows**, ~4,300 belonging to the other six), and `save_universe_bookmarks` exists again.
>
> **Boss test: 4/4 steps passed** on the widened rename cascade.
>
> **The lesson the pass paid for three times, stated once:** *a rule applied to the half of the shape you happened to test is not a fix, and a comment is a claim with a shelf life.* Two of the five self-inflicted findings came from believing a neighbouring comment instead of reading the statement it described — including a comment claiming the cross-library widening had happened when the call below it still passed one library. **The tutorial-auditor caught that one** by refusing to write a Boss test for behaviour the code did not have.
>
> ### ⚡ The performance investigation — the two slowest daily gestures, measured and fixed
> **Measured end to end: note create ~54 s → near-instant; rename ~50 s → under one second.** The instrumented breakdown behind those numbers, all against the Boss's real universe, all measured rather than reasoned:
> - **The tree walk was opening every file.** `extract_frontmatter_status` was called per-`.md` inside `read_dir_recursive`, and `read_library_tree` runs **twice per create**. Byte-for-byte Rust replicas against the Boss's real universe: `Constellation PKM` (803 notes / 130.6 MB) cold **3.67 s**; the same walker with only that call removed, same directory, same state: **0.017 s**. The per-file open *was* the entire cost. Removed.
> - **The rename cascade was reading 140.8 MB from 2,105 files, every time.** Live from the Boss's own write journal: walks of **8,303 ms / 8,511 ms**; isolated replica **8.73–8.90 s** cold (read 7.8 s · stat 0.75 s · regex **0.02 s** — the regex was never the cost). One folder, `Constellation PKM/Constellation Working Docs`, is **91 %** of every rename's I/O. Fixed by (a) one `entry.file_type()` for both the symlink and directory questions plus dropping a redundant `path.exists()` — measured stat cost **752.7 ms → 2.3 ms** — and (b) parallelising the per-file read (`rayon`; each file already takes its own per-path gate lock, so there is nothing to serialise on).
> - **Refuted, on evidence, rather than assumed:** the day's two prime suspects — the new `target_cid_cn` back-resolution statement and the widened `note_meta_sky_au` WHEN clause — contribute **under 20 ms** of the 14 s. Both are index seeks by `EXPLAIN QUERY PLAN`; the widened WHEN adds **nothing** on the rename path because `OLD.name IS NOT NEW.name` already fired that trigger.
>
> ### 🆕 NEWLY FILED — PJ-234 … PJ-251 (the third sweep's 29 open findings + 2 named performance next steps + 1 Charter orphan)
> The third whole-app sweep confirmed **37**; 8 were fixed in-pass or by the held-5 (`sky_nodes` UNIQUE, the cross-library cascade, `save_universe_bookmarks`). **29 remain open and are filed below** — durable register copied into the repo at `lab/reports/sweeps/SWEEP-2026-08-10-third-whole-app.json` (origin: the session scratchpad's `sweep3.json`, which does not survive the session). Severities are the sweep's own, after adversarial refutation. **Line numbers are as at the sweep and have drifted where the §15 fixes landed** — every anchor below was re-verified against the tree today unless marked otherwise.
>
> **APP-KILLER**
> - **PJ-234** *(APP-KILLER · content-corruption)* — **`bases.rs:584`** (and its sibling at **`bases.rs:495`**) `update_frontmatter_property` ends the dropped block at a **BLANK line** (`is_block_value_line("")` is false), so every sequence item after an interior blank is re-emitted orphaned under the new scalar and folded into its value. `yaml_lines::ends_dropped_block` (`yaml_lines.rs:176`) is the corrected predicate §15 wrote for exactly this and it was swept into `sources/mod.rs` **only** — verified today: outside its own definition in `yaml_lines.rs`, `ends_dropped_block` is referenced from no file but `sources/mod.rs`. Output is unparseable YAML, i.e. the precondition for the frontmatter-discard app-killer. **Group 1 · Charter.**
>
> **HIGH**
> - **PJ-235** *(HIGH · silent-data-loss)* — **`libraries.rs:2518`** `move_item` authorises its **DESTINATION** with `validate_path_in_any_library`, whose `load_all_libraries` resolves through `resolve_universe_libraries` — the **FEDERATED** set. Verified today by reading the chain. So a note or folder can be physically moved **out of the active Universe into a linked universe's library** with no refusal — the mirror image of the case `guard_no_foreign_library_under` (`libraries.rs:445`) exists to refuse, and which `move_item` already applies to its *source*. **Group 1 · Charter.**
> - **PJ-236** *(HIGH · silent-data-loss)* — **`store.ts:357`** the write-ahead crash-recovery net persists to a **single `constellation-wab` localStorage blob, never pruned and never size-capped**, and the `setItem` is wrapped in an empty `catch {}`. Once the blob crosses the WebView2 per-origin quota, every later net stash silently fails and crash recovery is dead app-wide with no surface. Adjacent to **PJ-110**. **Group 1 · Charter.**
> - **PJ-237** *(HIGH · content-corruption)* — **`store.ts:1107`/`:1121`** `reloadTabsFromDisk` matches caller paths against tab paths with **raw string equality** and keys `byPath` by the raw caller path, while its sibling walker `adoptExternalChangeIntoTabs` was normalised with `normPath` on both sides for exactly this reason. A differently-spelled path makes the whole post-rewrite adopt a silent no-op, and the stale model's next save reverts the on-disk rewrite. **Group 1 · Charter.**
> - **PJ-238** *(HIGH · silent-data-loss)* — **`store.ts:5091`** `preserveWorkBeforeVacating` selects `isGone(t.path) && isNoteDirty(t.id)`, so a tab whose model holds **write-ahead-recovered** content (clean by construction, `netUnsaved`) is invisible to it; the function returns "safe to destroy recovery state" and the caller then wipes that path's net while the model is disposed unsaved. `hasUnsavedRecovery` exists — §15 wired it at three arbitration sites and this is a fourth. **Group 1 · Charter.**
> - **PJ-239** *(HIGH · index-divergence)* — **`embeddings.rs:469`** (twin call site **`search.rs:12854`**) `INSERT OR REPLACE INTO note_embeddings (…, cid_cn)` against the **full** UNIQUE index `idx_note_embeddings_cid_cn` (`search.rs:4254`), with `''` the documented default for a note carrying no identity. **The exact shape `sky_nodes` just shed** at §15 pass four: a second cid-less note's embedding silently deletes the first's. Same fix (partial index). **Group 1 · Charter.**
> - **PJ-240** *(HIGH · content-corruption)* — **`libraries.rs:2370`** (sweep: `:2341`) `set_frontmatter_parent`'s block-drop loop uses the same blank-line-blind rule, so replacing a `parent:` block list containing an interior blank leaves the remaining `- "[[X]]"` items orphaned under the new **quoted scalar** — unparseable, not merely wrong. **Same root as PJ-234; fix them together, through the one shared predicate.** **Group 1 · Charter.**
> - **PJ-241** *(HIGH · false-success)* — **`+layout.svelte:7076`** `addTagToNote`'s CLOSED-note branch composes through `composeUpdatedContent`, whose H1 branch (`yamlDoc.ts:354`) returns the **original** frontmatter bytes verbatim when the YAML does not parse — so the tag is never added, yet the function writes, reindexes and returns `true` with no notice bar. `addLinkToNote`'s closed branch (`store.ts:1563`) has the same shape **for a typed Living Link**. **Group 1 · Charter.**
> - **PJ-242** *(HIGH · silent-data-loss)* — **`universe.rs:1656`** (sweep: `:1659`) `read_universe_settings` returns `Ok({})` whenever `path.exists()` is false — which is false on **any** metadata-level failure, not only absence — and `loadSettings` (`store.ts:7302-7307`) latches `settingsLoaded = true` on any non-throwing result, so the 300 ms debounced `saveSettings` atomically writes **default settings over the user's real `settings.json`**. Verified in the tree today. **Group 1 · Charter.**
> - **PJ-243** *(HIGH · silent-data-loss)* — **`universe.rs:1738`** (sweep: `:1708`) `read_universe_workspaces` has the identical `path.exists()` shape. **§15 made this MORE load-bearing, not less**: `loadWorkspaces` now deliberately adopts an empty successful read (that was app-killer #2's fix, and the comment in `store.ts:7565-7573` says so), which is correct **only if an empty answer means empty**. The third reader, `read_universe_property_types` (**`universe.rs:1894`**, MED), shares the shape → a near-empty registry written over every library's property-type assignments. **One fix for all three: distinguish NotFound from every other metadata error.** **Group 1 · Charter.**
> - **PJ-244** *(HIGH · cross-note-bleed)* — **`review_backfill.rs:72`** captures the **departing** universe's `.constellation` directory once, then keeps writing `review_schedule` rows through the swappable `state.db`, with no generation guard and an unconditional stamp. **Group 1 · Charter.**
> - **PJ-245** *(HIGH · index-divergence)* — **`sky_backfill.rs:456`** drives an in-memory cursor across batches against the swappable `state.db`, has no `federation_generation` guard, and `finalize` stamps `schema_versions.sky` unconditionally — a universe switch mid-run permanently marks a **partly-built** universe complete. **Group 1 · Charter.**
> - **PJ-246** *(HIGH · index-divergence)* — **`links_backfill.rs:447`** identical unguarded shape in the outgoing-link aggregate back-fill; `finalize` stamps `links_outgoing` + `links_vocab` with no completeness check. **PJ-244/245/246 are one shape in three files — one generation-guard helper, per the Whole-Ecosystem Fix Law.** **Group 1 · Charter.**
> - **PJ-247** *(HIGH · freeze-hang)* — **`cece/orchestrator.rs:153`** `run_one_safe`'s per-cataloger timeout is **unenforceable**: `std::thread::scope` blocks until every thread spawned inside it finishes, so after `rx.recv_timeout(timeout)` gives up the scope still waits for the hung cataloger. The function's own stated purpose ("the original only caught panics, not hangs") is defeated. **Group 1 · Charter.**
>
> **The rest of the sweep, grouped**
> - **PJ-248** *(13 MED + 2 LOW — 14 distinct issues; two entries are the same `commitFocusSave` defect)* — full text and reproduction detail in **`lab/reports/sweeps/SWEEP-2026-08-10-third-whole-app.json`**. One line each below, so that none is lost even if the register ever is: **(1)** `+layout.svelte:6294` the semantic-index loop drives one **SYNC** `read_note` invoke per note over ~7,800 notes on the WebView2 IPC dispatch thread, starving every other invoke. **(2)** `+layout.svelte:1787` `commitFocusSave` — the only durable-save path for FocusPane text — reindexes but never `reembedNote`s, so a Focus-edited note keeps its **pre-edit** semantic vector forever (the exact omission §15 fixed at the four other save sites). **(3)** `NotePane.svelte:1177` and **(4)** `FocusPane.svelte:322` — the title is committed only from the input's `onblur`; teardown and `beforeunload` flush the **body** only, and removing a focused input from the DOM fires no blur, so a typed-but-unblurred rename is discarded with no trace. **(5)** `store.ts:1830` `saveTabContent`'s cascade gate returns with no write, **no write-ahead net entry and no `pendingPropSaves` record** — the three-way loss the sibling `saveLocks` gate two lines below was hardened against by §15. **(6)** `store.ts:2491` `parseFrontmatter` never unquotes a key, so a **quoted** key never matches `yamlDoc`'s CST lookup: SET falls through to ADD and appends a junk duplicate; REMOVE silently does nothing. **(7)** `universe.rs:1894` `read_universe_property_types` (see PJ-243). **(8)** `lens/query.rs:393` five `.base` writers still use a bare `fs::write` — the last persisted-state writers not on `atomic_write`. **(9)** `libraries.rs:561` deleting a folder that contains a **registered library** permanently removes its registration from `libraries.json`, recorded only by an `eprintln!` that does not exist in a release build — even in the recoverable trash modes where the files come back. **(10)** `store.ts:528` `retrySaveFailure` reindexes but never `reembedNote`s. **(11)** `SenseMakingCanvas.svelte:221` the teardown save deregisters from the app-close flush registry and then fires an **un-awaited** write whose only failure surface is a component already destroyed. **(12)** `GraphMindView.svelte:378` Sky View's "Compute Semantic Links" invokes **`read_file`, a command registered nowhere**, and swallows the rejection into `content: ''` — every note embedded as an empty document, meaningless similarity links, reported as success. **(13 · LOW)** `GlobalTasksView.svelte:151` `toggleTaskReconciled` writes watcher-suppressed and two of its three call sites send no `screen:note-saved`. **(14 · LOW)** `sources/mod.rs:696` `rewrite_note_sources_on_disk` returns Ok unconditionally even when the malformed-frontmatter bail-out changed nothing, so the caller mirrors values into `note_meta.sources` and clears the suggestion row for a write that never happened. **Groups 1–4 by item · Charter.**
>
> **From the performance investigation — the two named next steps, filed explicitly**
> - **PJ-249** *(/migration-sized · perf + index fidelity)* — **normalise `note_links.target_name`.** Measured on the Boss's real database (`perf.db`, 31,367 rows): **290 rows store something other than the bare title** — 75 contain `#` (an anchor) and 215 contain `::` (a predicate head). That is what **blocks driving the rename cascade from the index**: `SELECT DISTINCT source_path FROM note_links WHERE target_name = ?` is `SEARCH note_links USING INDEX idx_link_target` and times at **0.05–1.79 ms** against a **3.4–8.5 s** filesystem walk, and the referrer distribution is p50 = 1, p90 = 7, p99 = 31, max = 353 — **the median rename would open ONE file instead of 2,105**. Switching the cascade to the index *before* normalising would silently stop rewriting those 290 links, which is a worse defect than the slowness. Two ordered steps: **(1)** normalise the stored value + backfill; **(2)** only then drive the cascade from the seek, keeping the walk as an explicit fallback. Step 1 changes a **stored derived view and a write path**, so it is `/migration` by the project's own rule. **Group 1 (ranked high — it is the gate on both the cost and the index's fidelity as a referrer list) · leads the Group-2 perf lane.**
> - **PJ-250** *(perf · design decision)* — **the full-universe refresh fires after every note create.** `refreshLibraryCaches()` (`+layout.svelte:4225`) is re-armed 800 ms after every watcher flush and the `note-created` listener fires that flush; it re-runs `cache_boot_snapshot_core` (`cache.rs:172`) in full — `SELECT name, path, library_name FROM note_meta` across **all federated schemas**, shipping **10,752 note objects** over IPC. The Boss's own `boot-perf.history.jsonl` reports the Rust phase at **689 / 628 / 705 / 785 ms** across the last four boots; the SQL itself is a covering-index scan, so the ~0.7 s is materialisation + serialisation, not a table scan. **A boot-shaped full refresh used as an incremental update.** Not mechanical: the same function repopulates tags and aliases, and the universe-switch and index-repair callers legitimately need the whole thing — **scoping it per-caller is a design decision.** **Group 2.**
>
> **From the Charter — one item that said "filed as its own job" and never got a number**
> - **PJ-251** *(HIGH · index-divergence — Charter W2-14, `search.rs:1365`)* — the save path's incoming diff keys on target **NAMES only**, so **retyping a link** (same target, different link type; or cognitive ↔ structural) never recomputes B's `incoming_link_types(_json)` / `incoming_top_rank` / `incoming_count`. §15 annotated it as *"still open as a WRITE-PATH defect, but no longer unreachable — the repair now heals it and the user can now run that repair"*, and said it was **filed as its own job** — but no `PJ-NNN` was ever allocated, so it existed only inside the Charter. **The fix belongs in the save path's incoming diff, not in the repair.** **Group 1 · Charter.** *(Charter **W2-15** — FocusPane mounts with no `ontitlechange` — is the same defect as PJ-248 items 3–4 and folds into that fix.)*
>
> ### ✅ WHAT v1.77 CARRIED THAT IS NOW SETTLED (three closes, one correction, two "already recorded")
> - **PJ-221** *(APP-KILLER, `bases.rs:796` `format_yaml_value`)* — **CLOSED by §15.** The quoter now covers a leading `- `, `*`, `&`, `!`, `@`, backtick, `|`, `>`, `%`, `?`, `,`, `}`, `]`, bare `true/false/null/yes/no` and leading/trailing whitespace, at parity with the TS `quoteIfNeeded` it shadows. Verified today at `bases.rs:807-831`, with the parity test module `pj207_s15_quoter_parity` beneath it.
> - **`store.ts loadWorkspaces` APP-KILLER** — **CLOSED by §15** (app-killer #2). The `if (data.length > 0)` that contradicted its own comment is gone; an empty successful read is adopted, and `resetWorkspacesForUniverse()` is wired into `handleUniverseSwitch`. Verified today at `store.ts:7561-7582`; 3 vitest pins. **Its Rust half is now the exposure → PJ-243.**
> - **PJ-223** — **formally CLOSED** (repaired live at §11 Stage 1, 830 → 0 missing; the formal close was §15's, and this is it). The out-of-repo memo `project_index_rebuild_button_decision.md` is **marked overturned** — its own 2026-05-04 reopen condition fired and was measured at 60/7,824 drift with 798 never indexed.
> - **Charter W2-9 — NOT closed, deliberately, and v1.77 predicted otherwise.** v1.77's next-action line said §15 would close W2-9 "evidence §8/§13". It did not, and the Charter says why in its own words: the plan expected the evidence to be §8 **and** §13, **§13 was never built** (blocked on the PJ-224 ruling), so the **half-closed** status stands — *"marking it closed on §8 alone would be exactly the false-completion this Charter exists to catch."* The automatic class is closed (six passes scoped to the active universe's own libraries; 13 foreign rows measured before the fix); the **user-action class stays open → PJ-219**. Recorded here so the ledger does not inherit an anticipated close as a real one.
> - **PJ-228 · PJ-229** (closed in v1.73) and **PJ-230 · PJ-231 · PJ-232** (closed in v1.74) — **already recorded; no change.** Listed here only so the §15 close is not read as re-closing them.
> - **PJ-226** (walker-classification sweep, ≈25 walkers) — **advanced, not closed.** One walker done: `update_links_recursive` now takes a single `entry.file_type()` and drops the redundant `path.exists()`, measured **752.7 ms → 2.3 ms** of stat cost per rename. The remaining ~24 are untouched.
>
> ---
>
> ## The five priority groups
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> ~~1. **PJ-252**~~ — **CLOSED 2026-08-11**, Boss-validated across five test rounds (Stage 1 + Stage 2 Parts 1–4). One classifier; two further shapes found by running; a CRLF regression of its own caught by the `ui-inspector` gate; the blank-line family closed in both Rust and TS. See the v1.80 preamble.
> ~~2. **PJ-234 + PJ-240**~~ — **CLOSED 2026-08-11**, Boss-validated. FOUR surfaces, one rule; the wrong predicate deleted. See the v1.81 preamble.
> 2a. **PJ-258** — `listItemsOf`'s raw-comma split, the quote-aware sibling of a defect PJ-207 §15 already closed elsewhere.
>
> ---
> **► Next action (2026-08-22, after PJ-332 / 332b / 334 shipped Boss-validated):**
> **MIG-111 Stage B** — the rename path, read-side analytics, the watcher fence. Its hard ordering rule
> stands: the vocabulary reaches `rewrite_wikilinks_in_text` FIRST (B5); the rename fence comes down in
> a LATER commit (B6), never the same one.
> Beside it: **PJ-333** needs a one-line Boss ruling, and the **PJ-326..331** job the Boss already
> scheduled for after Stage A.
> ---
> 2b. **PJ-269** — the indexer stores every block list as an empty string; a THIRD frontmatter parser.
> 3. **PJ-235 + PJ-254** — the federation-boundary family: `move_item` can move a note **into** a linked universe, and every rename/move/create tail files a linked universe's note into THIS universe's index. **One concern, two surfaces — fix together.**
> 3a. **PJ-253** — the cascade's case-fold miss. **BLOCKED: Boss ruling required** (it changes which links are rewritten on disk).
> 3b. **PJ-255** — six detached tails with no generation guard (folds into PJ-244/245/246's helper).
> ~~3. **PJ-249**~~ — **CLOSED 2026-08-11**, Boss-validated. Rename ~50 s → **216 ms**. Built as a new `target_base` column (Boss-ruled option d), not the `target_name` normalisation this ledger originally described. See the v1.79 preamble.
> 4. **PJ-236 · PJ-237 · PJ-238** — the three recovery/adopt HIGHs on `store.ts` (unbounded net blob · raw-string path match · recovery-blind vacate guard).
> 5. **PJ-241** — write + reindex + `return true` on frontmatter that never changed.
> 6. **PJ-242 · PJ-243** — the `path.exists()` empty-success readers, and the frontend latches that trust them.
> 7. **PJ-244 · PJ-245 · PJ-246** — three back-fills with no generation guard across a universe switch.
> 8. **PJ-239** — `note_embeddings`' non-partial UNIQUE, the shape `sky_nodes` just shed.
> 9. **PJ-247** — a timeout that cannot time out.
> 10. **PJ-251** — retyping a link never recomputes the target's incoming aggregates (Charter W2-14, finally numbered).
> 11. Carried: **PJ-222** (`collect_md_paths` boundary) · the 2026-07-30 inspection's **25 lost candidates** · the **38-finding register** (`wbxz23bdr`) · PJ-248's Group-1 members (items 3–5, 9, 11).
>
> **Group 2 — Architecture & performance debt**
> **PJ-259** (the PJ-252 altitude residue: `detectPropertyType` + `PropertyEditor` still answer "is this key a list" independently, and block EXTENT is still line-decided) · **PJ-256** (no back-fill re-collects its table's statistics — the class behind PJ-249's headline defect) · **PJ-257** (`props_reparse` fails every boot and re-arms forever) · **PJ-250** (boot-shaped refresh used as an incremental update) · **PJ-226** (≈24 walkers still on `path.is_dir()`) · **PJ-225** (9 hand-rolled `mtime_secs` copies) · **PJ-110** (localStorage durability — `app-prefs.json` is the home for the next tenants; `constellation-wab` needs a **durability design**, not a JSON file — see PJ-236) · **PJ-233** (the registry `universes.json` lists only `كون عيسى` and points `active_id` at it while the app demonstrably runs **Eisa Universe** — proven by the app-generated timestamp in its own `boot-perf.latest.json`, its logged boot sequence, its federation manifest and direct file counts; the mechanism could **not** be reconstructed from source and was **not** invented) · PJ-248 item 1 (the sync `read_note` loop).
>
> **Group 3 — Feature completion**
> **PJ-207 §13** — GATED on **PJ-224** (Boss ruling: the ordinary search box does not federate; no removal offer may ship before that is ruled on). **PJ-219** — the user-action write class awaits its design ruling. **PJ-227** — a linked universe's phantom rows are permanently exempt from dead-row removal post-§8 (9 live rows). **PJ-220** — the `{name:}` workflow form + args delivery (CRLF proven for the `scriptPath` form).
>
> **Group 4 — Polish / i18n / small bugs**
> PJ-248 items 13–14 (LOW) · **PJ-172** Sight timing flakes (reproduce under CPU load, pass in isolation) · **PJ-260** (mixed line endings in Rust-written frontmatter — **Boss ruling pending**).
>
> **Group 5 — Documentation & hygiene**
> **PJ-261** (`deriveTabName`'s comment says "for canonical files" but the frontmatter `title:` is applied to every file) · **Doc-drift watch** — the translated manuals are partial and drift in vocabulary; §12 found **67** wrong panel names in `fa`/`ur` and fixed them. A periodic term-consistency sweep of each manual against its own locale's `i18n` JSON is worth a PJ if it recurs. The `.md` help/manual coverage of the §15 fixes is otherwise not user-facing.
>
> ---
>
> **Gates (v1.80, at the PJ-252 close):** vitest **941 passed / 0 failed** (82 files — the 3 `it.fails` are now `it`, plus 8 new cases incl. two CRLF and two blank-line) · Rust **1445/0** (1442 + 3 new `inject_cid_cn`) · svelte-check **0 errors**, 268 warnings (baseline) · release binary **2026-08-11 10:11**, Boss-validated. `tests/sight-v6/perf.test.ts` flakes under concurrent build load and passes 3/3 in isolation — the known **PJ-172** timing flake, no Sight file in the diff.
>
> **Gates (v1.79, at the PJ-249 close):** vitest **927 passed + 3 expected-fail** (82 files — the 3 are PJ-252's pinned reproduction, green *because* the defect is live) · Rust **1442/0** · svelte-check **0 errors** · release binary **2026-08-10 22:35**, Boss-validated. Commit `904bccbc`.
>
> **Gates (v1.78):** vitest **926/926** (81 files) re-verified **2026-08-10 10:27**. Rust **1410/0** · svelte-check **0** · i18n **15/15 in parity** as recorded at the §15 fourth pass; the performance pass's Rust gate is recorded with its commit in `lab/reports/SESSION-LOG-2026-08-09.md`.
>
> **PJ ledger reconciled at the close of this job per SO#9** — 3 items closed that v1.77 carried as open, **18 filed**, one anticipated close (Charter W2-9) corrected to the truth, Group 1 re-ranked, the sweep register preserved in-repo.
>
> ---

**Version 1.77 | 2026-08-09**

> *(See `Constellation Pending Jobs v1.77.md` — the trail is durable, never overwritten.)*

### 🚨 PJ-430 *(HIGH — Group 1 — terminology, user-visible on core surfaces in 11 languages)* — the app calls a Library a "vault" in ≥251 strings, inconsistently with its own correct word

**CLAUDE.md's FIRST convention:** *"Terminology: Use 'Library' everywhere, never 'vault' (except for
Obsidian import compatibility)."*

**The shape of it, which matters more than the total.** Every affected locale ALREADY has the right
word and already uses it for the majority of its Library strings. The vault word is the MINORITY
everywhere. This is an inconsistency to reconcile, not a retranslation to commission — measured over
the 120 keys whose English says "library" (repo/keyring senses excluded):

| locale | correct word (uses) | vault variant (uses) |
|---|---|---|
| zh | `库` 107 | `仓库` 27 |
| fa | `کتابخانه` 108 | `خزانه` / `صندوق` 3 |
| fr | `bibliothèque` 102 | `coffre` 11 |
| he | `ספרי…` 79 | `כספת` / `כספות` 33 |
| pt | `bibliotec…` 79 | `cofre` 33 |
| ja | `ライブラリ` 78 | `保管庫` 21 |
| ko | `라이브러리` 78 | `보관함` 21 |
| ur | `لائبریری` 78 | `خزان…` 18 |
| ru | `библиотек…` 77 | `хранилищ…` 33 |
| tr | `kütüphane` 65 | `kasa` 33 |
| hi | `लाइब्रेरी` 52 | `तिजोरी` 18 (and `पुस्तकालय` 21 — a THIRD word, correct but a different register) |

**≥251 strings, 11 locales.** A floor, not a final figure: some strings the method flags are
legitimate paraphrases (`Search across all libraries` → *"search all notes"*), so closing this needs
a per-locale read, not another grep.

**FIXED IN THIS PASS — only `plurals.libraries`** in 8 locales (fr, hi, ja, ko, pt, ru, ur, zh),
because PJ-428's new notice interpolates it and the clash was audible inside one sentence: the count
said *"2 coffres"* while the body said *"la bibliothèque"*. Three call sites, one the permanently
visible **status bar**. **Verified afterwards that each replacement matches its locale's own dominant
word** (table above) rather than introducing a fourth variant — that check was owed, because the
first version of the Hindi fix could easily have picked `पुस्तकालय` and split the file three ways.

**THE COUNT WAS WRONG FOUR TIMES. That is the durable lesson, not the number.**
1. **211** — a `قبو` probe matched Arabic `قبول` ("accept"); word-boundary misses elsewhere.
2. **222** — still counted three strings using the vault word CORRECTLY (`zh githubTokenDesc` is a
   GitHub *repo*; `ru apiKeyProtectionDesc` is the OS keyring).
3. **219** — **the search was CASE-SENSITIVE.** Turkish button labels read `Kasa ekle`; a lowercase
   `kasa` could not see them (tr 13 → 33), and the same blindness hid capitalised hits in ru, pt, fr.
   The six locales whose scripts have no letter case were right *by accident*.
4. **248** — **Persian was not in the word-list at all**, because the list was built from the words
   another reviewer had named, and it had not named Persian. `fa` carries `خزانه` (treasury) and
   `صندوق` (chest).

Every one of the four ran correctly and returned a clean result. None could have found what it was
missing. The instrument that finally worked asks a different question — *does this locale use its own
dominant Library word here?* — and cannot be defeated by not knowing the wrong word in advance.

**Found because** the `ui-inspector` compared an interpolated `{noun}` against freshly written body
text and saw two words for one thing. **Nothing in the pipeline checks translation CONSISTENCY** —
`i18n-parity.mjs` verifies keys exist and plural categories match, never that the words agree. Parity
is not correctness. A consistency check belongs in that script; filed with this entry.

### 🆕 PJ-432 *(MED — Group 2 — editor-lifecycle — the flush an ordinary switch runs, a removal skips)* — removing the active universe bypasses the pre-transition dirty-tab flush

`UniverseManager.svelte`'s `handleSwitch` calls the `onBeforeSwitch` flush hook before changing
universe; `confirmRemove` does not (compare the two functions). Removing the ACTIVE universe — which
is also the route to the first-run screen, since emptying the list fires
`onRemoveLast` in `+layout.svelte` — therefore transitions without the explicit flush that every
ordinary switch performs. PJ-103 exists because app-close flushes dirty models to disk; this is the
same concern on a different departure.

**Found** by the `ui-inspector` while it was checking whether the PJ-428 test could safely walk the
Boss to the first-run screen. It could — and this is why it should not. Recorded as the real reason
in the test rather than the vaguer "would need a separate first-run scenario", which was untrue.

**Not fixed in-pass**: it is an editor-lifecycle change, and CLAUDE.md routes those through
`/migration` with the BUG-015 forensics review. Verify before fixing whether the frontend's own
universe-switch path flushes downstream of this hook anyway — if it does, the severity drops.

### ✅ PJ-407 — **BOSS-TESTED AND PASSED 2026-08-29.** PJ-409 closed with it.

Both steps. Evidence from his own screen: the notice reading exactly "2 notes"; after the rename the
bar gone and the status bar moving **7494 → 7496**; the sidebar listing **NET** and **NET Framework**;
`.NET` open — 3,521 words, 14 properties, a note the app could not see that morning.

Verified afterwards against his index with the app's own resolution join: **`.NET` 16 incoming links,
`.NET Framework` 11** — all 27 dead before the rename. Files renamed cleanly (23,262 B / 35,613 B),
**no `NET.md.md`**: the trap the review panel caught in the instruction did not fire.

### 🆕 PJ-431 *(MED — Group 2 — index↔disk divergence the drift check cannot see)* — FIXED 2026-08-29 — the identity written on first open never reached the index

Found while verifying the one PJ-407 claim the Boss had not exercised. Both recovered notes carry a
`cid_cn` in their frontmatter while `note_meta.cid_cn` is empty, and all 27 incoming links hold
`target_cid_cn = ''`. Confirmed against the live DB **with the WAL replayed**, not an `immutable`
snapshot.

`ensure_cid_cn` injects the identity into the FILE through `gate_write`, which suppresses the watcher
by design; nothing then updates `note_meta`, so `note_meta_sky_ai` (guarded on `NEW.cid_cn <> ''`)
correctly declines and `note_meta_sky_au` — whose own comment says it exists for exactly this lazy
injection — never fires. **Both triggers are right; the write path never announced itself.**

Fixed in `canonical.rs::ensure_cid_cn_cmd` — re-index when the content actually changed, so the
ordinary open path pays nothing. Test `stamping_a_notes_identity_also_tells_the_index`, proven RED.

**Worse than it looks, and measured:** both rows are IN STEP by mtime, so the drift check — whose job
is spotting index/disk disagreement — is blind to it, because it compares timestamps and not content.
It does not self-heal. The remedy that works already exists and is named for this case:
**Settings → Index → Full re-read**. The Boss's two notes still need one run.

**Owed follow-up:** whether the drift check should be able to see a content-level divergence at all is
a real question this exposes, and it is not answered here.

## 🧭 Filed 2026-08-29 — the "A UNIVERSE MAY LIVE ANYWHERE" audit

*Boss ruling (CLAUDE.md, Knowledge Hierarchy): a universe may live wherever the user keeps it — a
different drive, a USB stick, a share. Clarified by him: **"identify" means explicit registration must
work anywhere**, NOT auto-discovery. A four-lens audit with adversarial refutation found the
**capability already fully honoured** — there is no location constraint anywhere in production code
(every `Constellation Universes` literal in Rust is in a test or a comment; zero in the frontend and
all 15 locales; no picker default anywhere; `ensure_under_active_root` called at exactly two sites,
both LIBRARY paths). **Nothing to remove.** What it found instead is that the app describes the
freedom wrongly. Two fixed in-pass on his instruction; the rest filed here.*

**FIXED IN-PASS:** the User Manual + help portability passages (they promised "Constellation will
automatically detect and fix all internal paths" — measured false: only `libraries.json` is rewritten,
while `note_meta` holds **8,033 rows, all 8,033 absolute** under the old location), and the first-run
button that read **"Bring In a Library — copy or move an existing folder of notes into this
Universe"** while its handler (`link_library_as_universe`) copies and moves **nothing** and creates a
*Universe* in place. Relabelled ×15 to say what it does. **That button is the ruling's own operation,
and it was labelled as its opposite.**

### ✅ PJ-433 *(CLOSED 2026-08-31 — the Boot Chooser; Boss-passed on all seven live stages; session log §1–§10; ledger v2.09 preamble carries the close evidence)* — an unreachable universe failed silently at boot, and the fallback was then saved as the user's choice

**Shipped shape (Boss-ruled 2026-08-31, panel-unanimous):** when the recorded universe cannot be
activated at boot, the app opens NOTHING and persists NOTHING. `BootChooser.svelte` — a sibling of
the setup wizard under the same pre-`appReady` gate — names the universe, its path and the reason,
lists every registered universe with live reachability (3s mount-watch, read-only), and offers
Retry / **"It's back — Open"** (lit by the watch, never auto-opening — the Boss's ruling 3) /
per-entry Open / **Open from folder…** / Create-new (with a context-only **Back**). The Boss also
ruled **no Remove on the boot screen** (a missing drive is usually just unplugged).

**A′, folded in per the Whole-Ecosystem Fix Law:** `remove_universe_from_registry` no longer guesses
a successor (`active_id = None`, never `entries.first()`), and the confirm dialog NAMES the universe
it will open — one computation, captured before the dialog clears, so the promise and the act cannot
drift. Verified live: *"Constellation will then open: Eisa Cognitive Knowledge"* → that is what
opened; files intact on disk afterwards.

**Two defects cured in passing, both pre-existing:** a universe created through the first-run wizard
used to run with **no file-watcher and no federation listener until restart** (the wizard called
`initializeApp` alone; every cold-start door now runs the shared `finishBoot` tail); and the
"Welcome to Constellation" wizard no longer appears when your universes merely can't be reached —
the chooser lists them instead.

**Original filing, kept for the record:** `+layout.svelte`'s boot loop was a bare
`catch { continue; }` — the error never bound. With other universes registered, the loop opened one
of them and the user got a **normal-looking window on the wrong universe**, no notice;
`set_active_universe` then wrote that fallback to `active_id`, so it kept opening the fallback after
the drive returned. Location-agnostic in mechanism, but **the population it hit was precisely the one
the may-live-anywhere ruling invites**: a universe on a USB stick, an external drive or a share.

**Coverage honesty:** four states ship code-verified but never seen live — all-unreachable,
no-recorded-choice, the inline pick-failure path, and the Unreachable chip on a list row. See the
v2.09 preamble.

### 🚨 PJ-434 *(HIGH — Group 1 — Whole-Ecosystem violation already in the tree)* — an unreachable Linked Universe is reported as present and empty

`get_child_universes` (`universe.rs:1806+`) tests only `.exists()`; on failure it synthesises a name
from the folder and reports `library_count = 0`. Forty lines away `resolve_libraries_recursive`
(`:641-644`) DOES check (`canonicalize` → `Err(_) => continue`). **Two walkers of one concern
disagree.** Live today: `كون عيسى` declares a child at `E:\Constellation Universes\Two universe
UNIVERSE\Two Universe UNIVERSE`, which does not exist, rendering as an ordinary member on four
surfaces (`LibrarySwitcher.svelte:131`, `DashboardView.svelte:216`, `+layout.svelte:8536`,
`OrgChart.svelte:159`). `ChildUniverseInfo` has no field that could carry "unavailable".

**Escalation:** `remove_child_universe` has no reachable UI caller outside the setup session that
created the link — so a dead link **cannot be retracted without hand-editing `universe.json`.**

### ✅ PJ-435 *(CLOSED 2026-08-31 — Boss-passed twice: three-stage live test + post-inspection smoke re-test; session log §26–§28; ledger v2.07 preamble carries the close evidence)* — moving a universe leaves the index at the old location, AND the remedy the app recommends destroys the link graph’s age

**SCOPE CORRECTED 2026-08-29 after a philosophy panel — the original framing rested on a stale
rulebook.** CLAUDE.md asserted that earned link data lives nowhere on disk. **False since MIG-104
Slice 6**: `.constellation/earned.jsonl` is identity-keyed, travels with the folder, and restores
walks / trust / retirements / review priority on every boot. The Boss's universe carries 53 such
records. That section has been corrected; **two panels were briefed wrongly by it before anyone
noticed.**

**What a move ACTUALLY costs, measured 2026-08-29 — narrow, and worse than "stale paths":**
- **Every link's birth date.** `created` is absent from `earned.jsonl`; **234,917 of 234,917 live
  rows carry one.** The Living Link Architecture names *Created* as one of the eight properties.
- **The review schedule.** `review_schedule` is keyed on `path` — 8,033 rows, all absolute — so a
  move orphans it outright.
- **And the app's own recommended remedy destroys both.** A Full re-read re-indexes each note by its
  OLD location, finds nothing to preserve, and writes every link fresh with today's date — four
  minutes, reports success, silently flattens the age of the whole link graph. **The documentation
  that recommended it (written earlier the same day) has been corrected to say the opposite.**

**APPROVED PLAN (Boss: "Proceed", 2026-08-29):**
1. ~~Correct the CLAUDE.md storage section~~ **DONE** — the panel called it the highest-value item.
2. **No new engine.** Drive `mig108`'s existing rewrite with a one-entry journal: it already takes a
   verified backup, rewrites in one transaction, and proves conservation before committing.
3. **The notice must REPLACE the existing alarm, not sit beside it.** Today a move makes the app say
   *"8,033 notes in your libraries are not in the search index"* — true words, false impression,
   nothing is missing — beside a button that starts the destructive rebuild. *"An app that tells you
   your life's notes are gone when they are all fine, and then offers a button that quietly damages
   them, is not Constellation."*
4. **Carry the two survivors through the repair** — link `created` and the review schedule. Discovered
   and measured, so WA#6 applies: fixed in-pass, not filed.
5. Reproduce-First: fixtures CLEAN and DIRTY before any production code.

**REJECTED by the panel** (two of its own members proposed it): reusing the per-note "follow the moved
file" mechanism. It discards ~80 index rows with no usable identity stamp — **12 of them ordinary user
notes** — and reads 8,033 file headers at boot, which Rule 8 forbids by name.

*Original filing, kept for the record:*

Opening a moved universe rewrites `libraries.json` only. Every `note_meta` row keeps its absolute old
path; `collections.json`, `review-pulse.json` and `settings.templateFolder` likewise. The cold-start
re-index cannot repair it because its gate keys on library **name**, not path
(`index_repair.rs:846-851`).

**Nothing is lost** — `reconcile.rs:782` refuses to remove rows once more than `max(200, 10%)` look
stale, treating it as a disconnected drive rather than vanished notes, which is exactly right. But
search and links point at the old location until the user runs **Settings → Index → Full re-read**,
and the app never prompts for it. The documentation now says so plainly; **making it automatic is the
real fix** and is what would let a universe genuinely "just work" from a USB stick, which is the
ruling's stated intent.

### 🆕 PJ-438 *(LOW — Group 3 — double work on first open, found by an inspector tracing a test claim)* — a never-indexed universe may be indexed TWICE at first open

Two boot paths fire on the same trigger ("zero indexed notes, registered libraries present"):
the PJ-065 §8 cold-start loop (`+layout.svelte:3153-3174` → `Scope::ColdStart` — silent by design)
AND the BUG-022 recovery check inside `loadAllStats().then` (`:3182-3208` →
`constellation_search_init` → `Scope::Full` — emits the progress strip). `ColdStart.covers(&Full)`
is false, so the Full QUEUES behind the running ColdStart and re-reads the whole universe again the
moment it finishes. Cost: one redundant full pass on first open, and a progress strip that may or
may not appear depending on the race. Not data-unsafe (both passes are idempotent); filed rather
than fixed because the fix touches the boot fan-out hours before a Boss test, and the redundancy is
pre-existing (neither path is from PJ-435). The Stage-2 test now reports which path wins as data.
**Found by the `ui-inspector` refusing to verify a "no progress bar" sentence it could not prove.**

### 🧭 PJ-437 *(Group 2 — owed DIRECTION, not a task)* — the index addresses notes less portably than the durable layer beneath it

`earned.jsonl` keys on `cid_cn` — identity — and therefore travels with a universe wherever the user
puts it, exactly as the Boss's 2026-08-29 portability ruling requires. `note_meta`, `note_links`,
`review_schedule` and eleven sibling columns key on **absolute paths**. So the DERIVED layer is
addressed less portably than the source it derives from, and that asymmetry — not the missing repair
— is the actual root of PJ-435.

Make note addressing identity-relative and a move stops being an event: no notice, no click, no
repair, and the onboarding document's long-standing promise *"move it to another machine and the
entire workspace follows"* becomes literally true.

**Filed as direction so that "one click per move, forever" does not become the permanent answer by
default.** Full `/migration`; not today's work. Recorded in the same commit as PJ-435's plan, per the
panel's explicit condition for endorsing the smaller fix.

## 🧹 Filed 2026-08-31 — from the PJ-433 close (three `/simplify` findings deliberately not taken mid-migration, two pre-existing Rust gaps found while mapping)

### 🚨 PJ-440 *(MED — Group 2 — the un-generalized half of a mechanism this migration DID generalize)* — the "leave a universe" sweep runs at one door and not the others

PJ-433 gave every cold-start door ONE enter tail (`finishBoot`). The **leave** half was left as three
hand-rolled variants: `handleUniverseSwitch`'s ~50-line residue sweep (workspaces latch — PJ-207 §15,
link-type latch, sky epoch, drift counts, tree selection MIG-091 P2, and more, **each line there a
measured leak**), against `enterUniverse`'s two-line flush. The **remove-last → wizard → create** door
therefore changes universe with **none** of that sweep. Pre-existing, but now the asymmetry is
visible: one door's invariant is enforced by fifty commented lines and another's by nothing.
**The fix is one leave-sweep + one enter tail, composed by all four doors** — `finishBoot`'s header
comment was re-scoped to stop overclaiming until then. *(Altitude review, 2026-08-31.)*

### 🆕 PJ-441 *(LOW — Group 3)* — `appReady` is gated per handler, not at the dispatch seam

The PJ-433 guards on `handleToggleSecondScreen` / `handleSendToSecondScreen` are correct but local.
The real exposure is **mid-switch**: `handleUniverseSwitch` sets `appReady = false` while the global
keydown listener stays live, and `handleGlobalKeydown` checks `isLocked` and `$hotkeyCaptureArmed`
but never `appReady` — so every OTHER palette command remains dispatchable with no universe active.
One gate at the dispatch seam (Escape/overlay-close exempted) instead of per-handler opt-in.

### 🆕 PJ-442 *(LOW — Group 3 — reuse)* — the Boot Chooser's row kit is a copy-adapt of the Universe Manager's

Two surfaces render the same universes; `.bc-entry*`/`.bc-btn*`/`.bc-chip` were typed from
`.um-entry*`/`.um-btn*`/`.um-badge`, including a second hardcoded `#22c55e`. Metrics were re-aligned
at the close so measured drift is zero TODAY, but the CLAUDE.md rule ("extract into a shared
component; never copy-paste and adapt") wants one shared universe-row. Not taken mid-migration
because it restyles a Boss-validated surface hours before his test.

### 🆕 PJ-443 *(LOW — Group 3 — a half-activation in the legacy path)* — `migrate_legacy_data` sets the active pointer without the invalidation chain

`universe.rs` (the `migrate_legacy_data` body) writes `registry.active_id` AND sets the in-memory
`active_path` directly, skipping `invalidate_libraries_cache` / `invalidate_search_state` / the
Arabic-override activation that `set_active_universe` performs. Pre-existing; reachable only on the
legacy-migration path. Surfaced while enumerating every writer of `active_id` for PJ-433's A′.

### 🆕 PJ-445 *(LOW — Group 3 — a narrow contradiction of PJ-433's own promise, found by the Phase-4 audit)* — a FAILED "Open from folder…" still moves the recorded choice

`open_existing_universe` writes `active_id` and saves it **before** activation (PJ-310's
register-then-activate design; the PJ-435 moved-universe repoint branch relies on the same shape).
The Boot Chooser's **Open from folder…** door therefore has a seam: if the *register* succeeds and
the *activation* then fails, the user's recorded choice has moved even though their click did not
open anything — a narrow contradiction of "nothing is remembered until you choose."

**Bounded and self-announcing:** the next boot shows the honest chooser naming that universe, so the
user is never lied to and can pick again; nothing is lost. **Not fixed in-pass** because the write
order belongs to PJ-310/PJ-435's shared path and altering it hours after a passed Boss test would
put a validated repoint mechanism at risk for a recoverable seam. The clean fix is to defer the
`active_id` write until activation succeeds — which is also the shape PJ-437's direction wants.

## 🛡️ Filed 2026-08-31 — the PER-CYCLE whole-app Safety Inspection at the PJ-433 close (65 agents; 19 confirmed → de-duplicated to 17 distinct: **8 new, 8 already-filed, 1 refuted-stale**)

*Its FIRST attempt returned an empty findings list with all 14 hunters dead on a model rate limit —
recorded as a NON-RESULT, not a pass, and re-run. The re-run's 19 were then de-duplicated
finding-by-finding against this ledger + the Charter + both prior sweep registers, and panelled.*

**The panel's headline:** *"Nothing in this register can lose Eisa's work today; the worst of it is
that opening a note the app has never stamped with an identity can briefly freeze the window — and
the two genuinely dangerous items were already found and filed by earlier sweeps and are still
sitting open."*

**🔁 THE STRUCTURAL FINDING — the ledger works as a NET and fails as a QUEUE.** Eight of seventeen
were already filed, some since 2026-08-11. Nothing was lost — every one was findable — but nothing
comes OUT. The mechanism is two umbrella entries: **PJ-264 (~100 unnumbered sweep findings) and
PJ-378 (58)**. A defect inside them is filed but *invisible*: F9 and F13 are confirmed defects this
ledger technically carries that no human can see. **This sweep spent most of its budget re-proving
known bugs. The cure is not a better sweep** — it is a drain. Put to the Boss as a cycle-shape
question.

**Re-ranking (visibility corrections, no filed item proved MORE severe than its entry):** PJ-378's
F4 stays HIGH/Group 1 despite this sweep labelling it MED · PJ-346 keeps MED over this sweep's LOW ·
**F13 should be promoted out of PJ-264's bucket to its own number** (unsaved property edits lost on
rename must not be invisible) · F7 and F14 are each double-filed at two severities (PJ-348+PJ-378;
PJ-248 item 13+PJ-346) — consolidate to one row each.

**Already-filed (no new numbers):** F3→PJ-396 · F4→PJ-378 · F7→PJ-348 · F8→PJ-347 · F9→PJ-378 ·
F13→PJ-264 · F14→PJ-248 item 13 · F15→PJ-346. **Refuted-stale (already fixed in tree):** F12.

### ✅ PJ-446 *(CLOSED 2026-09-01 — Boss-passed; found by THREE independent hunters)* — `ensure_cid_cn_cmd` re-introduced the PJ-066 app-freeze class, six days ago

**FIX SHIPPED:** `#[tauri::command]` → `#[tauri::command(async)]` at `canonical.rs:1477`, matching
the siblings, with the reasoning recorded in a doc-comment at the site so the next reader cannot
re-introduce it. No contract change: the promise still resolves on completion and awaited callers
stay correct. Suite **1,616/0**.

**BOSS-PASSED 2026-09-01** on the 19:16:19 binary. **The panel's proposed test was rejected and
replaced first** — it asked for "open three never-opened notes in his daily universe", but that
universe is 27/8,033 unstamped (14 exempt templates + 13 candidates), so three random notes would
reach the fixed branch with probability ≈13/8,033 each: **the test would have passed whether or not
the fix worked.** Replaced with a fixture reproducing the real exposed population — three unstamped
notes (one 41 KB, 400 wikilinked sections) brought into an already-migrated universe, which is the
only shape that reaches the branch (a NEW universe gets everything stamped at first boot by
`mig003`, gated at `search.rs:4987`). All three opened with no stall and responsive typing.

`src-tauri/src/canonical.rs:1477` is a bare `#[tauri::command]` — **no `(async)`** — and since
**PJ-431's fix landed at `4aee6ea2` (2026-08-29)** its body calls `reindex_single_note` at `:1513`,
taking the SQLite writer lock and running a full `index_note` **on the IPC dispatch thread**. It is
awaited on the note-OPEN path (`store.ts:3371`, `:3714`). Every sibling that reindexes is explicitly
`(async)` — `search.rs:12699` (its doc-comment cites the measured **30–50 s** PJ-066 connect
freeze), `bases.rs:397`, `shape.rs:259/277`, `libraries.rs:1693`.

**Exposed population is not an edge case:** `mig003_backfill_cid_cn` runs ONCE PER UNIVERSE (gated
on `schema_versions`), so a library brought in later from an external/Obsidian vault into an
already-migrated universe is never backfilled and **every note in it hits the branch on first
open**. While it holds, no other IPC is served: tab bar, search, watcher flush, debounced saves all
park. No spinner, no error — the window simply stops responding, then resumes.

**Fix: one token**, matching four documented siblings. Bound, stated honestly: `ensure_cid_cn`
early-returns for notes that already carry an identity, so this is one re-index per note per
lifetime — **a stall, not corruption**. The frontend already solved this same seam
(`store.ts:3716` fires the async command fire-and-forget); PJ-431 re-implemented it server-side
synchronously. *(Prior sighting never promoted: `lab/reports/sweeps/PERF-2026-08-10-create-rename.json:47`.)*

### 🚨 PJ-447 *(HIGH — Group 1 — silent data loss; defeats a Boss-approved ruling)* — a colliding property key silently overwrites the other property's value on disk

`src/lib/editor/propsCommit.ts:110`: a row whose key collides with an existing key falls through to
`:118-124` and is emitted as `{op:'set'}` **on the existing key** — routing around BOTH refusals
built for this: `noteModel.addProp:370` never runs, and `noteModel.renamePropKey:405` — whose own
doc records **the Boss-approved ruling that a rename onto an existing key is reported to the user,
never resolved by last-wins** — is not called by this path at all. `apply()` reports the set as
TAKEN, so `refused` is empty and PJ-187's "could not be saved" banner never fires. **Measured by
running the shipping `plan`/`apply`/`composeUpdatedContent`.**

Two corroborating facts the hunter added: `renamePropKeyIn` (`noteSession.ts:148`) has **zero call
sites in `src/`** — the guard is unreachable from the panel — and `PropertyEditor.svelte:693`'s
`updateKey` does no duplicate check, so a colliding row is reached **by ordinary typing** (Ctrl+;
then type an existing key). **Not fixed in-pass** on the panel's ruling: the fix spans
PropertyEditor + propsCommit + noteModel AND requires deciding what the user is *told* on a
collision — a multi-file content-integrity change at the end of a long session is how regressions
ship. **Top of Group 1.**

### 🆕 PJ-448 *(MED — Group 2 — silent data loss on a path PJ-435 shipped yesterday)* — the post-repair reload skips the final-flush handshake

`+layout.svelte:658` — `window.location.href = '/'` unloads the webview **without** running the
`session:final-flush` handshake, so the four debounced writers that handshake exists for never
flush. This is the PJ-435 post-repair reload path (itself an inspection fix from that job), so the
window is narrow — a repair completes and reloads — but the handshake exists precisely because
those writers are debounced.

### 🆕 PJ-449 *(MED — Group 2 — false success)* — Approve-All reports a clean batch over swallowed failures

`src-tauri/src/sources/bulk_ops.rs:286`: every per-note accept failure is swallowed into a state
field **the UI never reads**, then the batch reports done with no error.

### 🆕 PJ-450 *(MED — Group 2 — content-corruption; a Whole-Ecosystem asymmetry)* — the rename cascade's SEEK arm lacks the WALK arm's boundary guards

`libraries.rs:7876-7906`: the SEEK loop applies four guards to index-derived candidates (dedupe,
`exists()`, the foreign fence, `cascade_excluded`) — but its source
`cascade_candidates_via_index:8474` returns raw `source_path` strings from `note_links` with no
filtering, and **the WALK arm's `collect_cascade_candidates:8317` skips symlinks and dot-entries
while the SEEK arm never does.** Two halves of one concern, unequal — the exact shape the
Whole-Ecosystem Fix Law names.

### 🆕 PJ-451 *(MED — Group 2 — false success)* — `adoptDisk` re-seeds without minting a new `gen`

`src/lib/editor/noteModel.ts:666`: a full re-seed (props, body, base, diskBaseline,
version/savedVersion) with no new `gen`, so a save composed BEFORE the adopt can still land.

### 🆕 PJ-452 *(LOW — Group 3 — write-time-derivation gap)* — `sky_nodes.maturity` is a clock function recomputed only on write

`src-tauri/src/search.rs:270`: `maturity` derives from `days_since_created` / `days_since_modified`
but is only recomputed when the note is written, so it goes stale for an untouched note. (Rule 8
territory: the derived view is maintained at write time, but this one's input is *time itself*.)

### 🆕 PJ-453 *(LOW — Group 3 — cross-note bleed)* — outgoing-link headlines keyed by target string, never cleared

`OutgoingLinksPanel.svelte:129`: `summaryHeadlines` is keyed by the raw wikilink TARGET STRING
rather than the resolved note path, is never cleared on a note switch, and its async fetch can land
on the wrong note.

### 🚨 PJ-454 *(HIGH — Group 1 — breaks a Boss ruling; found by the BOSS's own remark, 2026-09-01)* — a template OUTSIDE the Templates folder gets stamped, and every note cast from it inherits its birth

**The Boss, 2026-09-01:** *"Templates shouldn't have a constellation cid_cn stamp, never, when
created by a user. Because if Constellation assigns a `cid_cn` stamp to a template at the time and
date of creation, it will carry it all the time, and if the user tries to use any template to
create a new note out of it, Constellation will assume it was created at the time and date of the
`cid_cn` stamp."*

That is **MIG-TPL §1 / his ruling of 2026-07-19**, and `store.ts:4866-4871` states it in exactly
those terms. **The ruling is honoured for templates INSIDE the templates folder and broken for
templates outside it**, because the two halves of the app disagree on what a template IS:

| | test for "is a template" |
|---|---|
| **Rust** (`search.rs:4327-4332`) | `kind == "template"` **OR** path under `templates_dir` — two arms, and `search.rs:17098-17100` has a test fixture named **`stray`** for exactly the outside-the-folder mold |
| **Frontend** (`store.ts:4875-4888`, `isTemplatePath`) | **path only** |

The frontend guard is the one that gates the stamping call (`store.ts:3369`, `:3713` →
`ensure_cid_cn_cmd`), so a `kind: template` file outside the folder **is stamped on first open** —
the mold acquires an identity, and `rebrandCopyFrontmatter` (`store.ts:566`) only strips `cid_cn`
on the *recovered-copy* path, not on a cast from a template.

**Reachable by ordinary use**, no edge case required: moving a template out of the folder;
**changing `appSettings.templateFolder`** (every existing template becomes a stray at once);
setting `kind: template` on a note via the Properties panel; importing templates from another
vault into a different folder.

---

### 🔬 PANEL VERDICT + BOSS RULINGS (2026-09-01, workflow `wf_77c3801f-411` — 8 agents: 4 investigations, 3 lenses, chair)

**IT HAS ALREADY HAPPENED — 102 stamped molds on his disk, measured, not theorised.** The chair
re-measured all 13 universes personally *because the first two maps disagreed*:

| Universe | Template files carrying an identity stamp |
|---|---|
| **Eisa Universe** | **82** |
| **موسوعة عيسى** | **19** |
| **Eisa Cognitive Knowledge** (daily) | **1** |

Read to be certain: `موسوعة عيسى\الموارد الرئيسة\القوالب\1 Base Template (up, related, created).md`
carries `created: "{{date}}"` — a placeholder waiting to be filled at casting — sitting directly
beneath a hard stamp dated **2026-04-14**.

**THE CORRECTION THAT MATTERS, in the Boss's favour: no cast inherited a stamp.** `create_note`
strips identity keys from template-supplied frontmatter before writing
(`libraries.rs:1666-1686`, with a regression test). His feared consequence is real in MECHANISM
but has not reached a single note. What IS wrong today: a stamped mold misreports its own birth
wherever identity is read — and identity **overrides** a correct `created:` line beside it
(`cockpitGraphData.ts:136`).

**Why the scale — ten doors, two guarded.** The territory map found **ten** paths that can write a
`cid_cn`; the frontend location guard covers **two** (`store.ts:3369`, `:3713`). The primitive
itself (`canonical.rs:1449` `ensure_cid_cn`) has **no template check at all**, and three of the
unguarded sites are BULK passes over every note in a universe — which is how 82 molds in one
universe acquired stamps *within the same second*. Exactly one stamping site in the codebase
honours the kind arm: `mig003_step3_soft_rebackfill` (`search.rs:4417`).

**RECOMMENDED FIX — the Two-Signal Choke Point.** Move the template test INTO the single engine
that writes the identity line, and have it ask **both** questions: does the file declare itself a
template (`kind: template`), **OR** does it sit under a templates folder? Keep strip-at-creation
as an independent second layer.
- *Reason 1:* the guarantee must live where no future caller can bypass it — ten doors, one gate.
- *Reason 2:* **the data chose the shape.** **ZERO of the 102 damaged files declare themselves
  templates** — they are Obsidian-era molds Constellation never marked — so a content-only rule
  would have prevented **none** of them; and location-only is what already failed. Both, OR'd.
- *Dissent (one lens):* self-declaration should be the sole authority as the File-Over-App-pure
  answer. **The disk refuted it**; it becomes the second arm, not the only one.

**Prior art (WA#5) is unanimous with the Boss:** Notion mints a new identity per page created from
a template; Obsidian's unique-note tooling never puts an id in the template (it generates at use);
Rails blanks id + timestamps on copy; `git init --template` copies structure, never repository
identity; **Word marks template-vs-document by a marker INSIDE the file, not by folder** — the
exact correction this fix makes.

### ⚖️ THE BOSS RULED (2026-09-01)

1. **Repair the 102 molds — YES.**
2. **Show him the exact file list for approval FIRST** — no automatic pass. *(The panel's own
   folder-based scan flagged ordinary notes with template-like names; stripping identity from a
   real note silently severs its earned reading history and leaves its links pointing at a dead
   identity, with nothing on screen. Misidentification is permanent and silent.)*
3. **Build it in the DRAIN cycle**, not at the tail of the 2026-08-31 session. **PJ-454 is the
   drain cycle's first item.**

**BRIEF FOR THE DRAIN CYCLE — how the list must be produced** (written now so the next session
does not improvise the risky half): identify candidates by **both** arms and **report them
separately**, never merged — (a) files under a resolved templates folder, (b) files whose own
frontmatter declares `kind: template`, (c) files matching NEITHER but flagged by name heuristics —
**bucket (c) is not a repair candidate, it is a review list.** For every candidate show: full path,
the stamp value and its embedded date, whether `earned.jsonl` holds records keyed on that
`cid_cn`, and whether any `note_links.target_cid_cn` points at it. **A mold with earned records or
inbound identity links is NOT a mold** — it is a note that was treated as one, and it must be
excluded and shown to him separately. Repair only after his approval, snapshot-first.

### 🆕 PJ-444 *(LOW — Group 3 — honesty gap on a corrupt registry)* — an unparseable `universes.json` shows the first-run wizard

`load_registry`'s lenient read falls back to an EMPTY registry, so a corrupt file presents to the
boot flow as "no universes at all" → the first-run wizard, while `set_aside_corrupt` shunts the file
aside on the next write path. The user is told nothing about the corruption or the set-aside copy.
PJ-433's chooser deliberately did NOT absorb this (different mechanism, and the boot flow must not
guess) — but it is the same honesty family: **never present a failure as an absence.**

**CLOSED 2026-08-30 — the full arc above shipped and Boss-passed.** Detection persisted
(`relocation.json`; copy never arms, second moves chain, move-back disarms), the honest banner ×15
replacing the destructive alarm, the one-click snapshot-first journaled repair (mig108 engine,
conditional destination purge), and the count refresh. Both fields the entry names as casualties
were carried through and MEASURED intact (1,000/1,000 created dates; 501/501 review rows). What
this entry does NOT close: the REBUILD exposure (`created` still absent from `earned.jsonl`) — that
is PJ-437's direction, restated in CLAUDE.md's amended storage section.


### 🆕 PJ-436 *(LOW — Group 3)* — a documented "Open Folder" action that was never wired

`Universe.md:93` documents it; `universe.manager.openFolder` exists and is translated ×15; grep in
`src/` returns nothing — orphaned, with `.path` and `.created`. Given the ruling, the right fix is to
WIRE it on every universe row (≈5 lines reusing `LibraryManager`'s `plugin:opener|open_path`) rather
than delete the string: a user keeping universes in scattered places is exactly who needs "show me
where this one actually is".

### 🆕 PJ-429 *(MED — Group 2 — the design ruling PJ-428 deliberately did not take)* — should a DECLARED library be content of this universe at every fence, or at none?

MIG-112's exemption — *"an explicit declaration beats a filesystem inference"* — lives at exactly one
address (`reconcile.rs` step 3). Roughly **22 other fence sites** across `canonical.rs`, `canvas.rs`,
`embeds.rs`, `inspector360.rs`, `libraries.rs`, `reconcile.rs` and `search.rs` have no equivalent, so
a registered library behind a manifest is "ours" for row-retention and "not ours" for every walk.

PJ-428 removed the two ways that could *destroy* or *lie*. What remains is a genuine choice, and it
reverses or extends a Boss-ruled contract either way:

- **Declaration wins everywhere** — thread the exemption through a shared helper so a registered
  library is content at every surface. Re-opens, by design, part of the door MIG-112 closed.
- **Fence wins everywhere** — drop the exemption; a manifest above you means your rows go, registered
  or not. Consistent, but silently de-adopts a library the user explicitly registered, which is what
  the exemption was added to prevent.

Not urgent: **measured latent** — 52 library entries across 8 universes, zero meet the precondition,
and PJ-428 has now closed the door that could create it from inside the app. Resolve with **PJ-419**
(bare `reindex_single_note` across a universe switch), which lives in the same fence contract.


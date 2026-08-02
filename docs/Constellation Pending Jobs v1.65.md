# Constellation Pending Jobs

**Version 1.65 | 2026-08-02**

> **What changed in v1.65** (**Four fixes closed — and the three outstanding inspection registers were finally triaged: 95 raw entries are 31 actual defects, 17 of them already dead. Ultracode**):
>
> **► NEXT ACTION — Boss's call on the triage batches.** `lab/reports/inspection-triage-2026-08-02.md` recommends **§2 as one batch of eight** (4 app-killers + 4 high; six are small, and four share a single root). Then **§3 as a second batch**, with the unreachable repair pass moved to its front. **No item in the register is migration-sized** — all 31 are local fixes. The standing queue behind them is unchanged: MIG-105 Phase-2 Plan (awaiting approval), MIG-104 Slices 9–15, MIG-109 (reserved, not scheduled).
>
> ### ✅ CLOSED by this session
> - **MIG-104 Slice 8 + 8b** (`92e55a29`) — archive-before-purge. The delete archives the note and its earned state *before* `DELETE FROM note_meta`, because the FK `ON DELETE CASCADE` fires there; a failed append **refuses the purge**. Five delete reasons distinguished. Boss Steps 1–2 passed; Step 3 surfaced the tree bug below.
> - **The boot regression** (`1b29e2b4`) — the probe asks `WHERE cid_cn = ''` and the only partial index covered `cid_cn != ''`, its exact complement: **109.7 s** of full scan. One mirror index closed it.
> - **The standing defrag rule** (`99769c2e`, `c3b7bdc1`) — after any mass rewrite the database compacts itself, gated by a pure tested predicate (25,000 pages **and** 10%). Boot **122.7 s → 4.7 s**; freelist 0. Made visible via MIG-041's existing `MigrationProgressStrip`; an interruption costs 10 min, not the 24 h a real failure earns.
> - **The deleted note that stayed in the file tree** (`6f22ff47`) — Boss-found. First-match library resolution + a conditional refresh; MIG-108 made it reachable. Six sites answering one question three ways now share `owningLibrary()`. 14 tests + in-suite RED proof.
> - **The ~27–34 s cold boot that predated today** — **closed by the defrag rule, not by separate work.** Bucketing all 896 recorded boots by idle-gap showed it was the cold-cache bucket (median **26.7 s** across 304 cold boots); the first cold boot after the vacuum measured **1.2 s**. *Caveat: n=1 — see Orientation §17.*
>
> ### 🔍 THE TRIAGE — three registers reconciled at last
> **95 raw entries → 31 distinct defects. 17 already fixed** by the 1–2 August work (verified at the symbol, not the line — line numbers had drifted up to 370 lines). Severity of the 31: **4 app-killers · 13 high · 12 medium · 2 low**. Fix sizes: 20 one-line, 45 small, 9 medium, **0 migration-sized**.
>
> The 95→31 collapse is the headline: the same defect was reported independently by up to **thirteen** agents across different files. Four settings files alone accounted for 13 entries.
>
> ### ☠️ The four still-live APP-KILLERS
> **(1) Four settings files treat "couldn't read this" as "you have none"** — `universe.rs:113-124` (write-backs at 701/706, 976/981, 1055/1061, 1250/1253), `review.rs:826-833`, `stylePresets.ts:128`, `link_types.rs:516`. One momentary file lock and the next click saves the emptiness over the real file: **every registered Universe**, the whole review history, the custom link vocabulary, every saved style. *I verified this one myself end-to-end at `universe.rs` — and the sting is that a previous audit hardened that write to be ATOMIC, which guarantees the destructive overwrite completes cleanly.* **(2) "Overwrite" on a name clash can destroy the surviving note** — the replaced note is trashed but its tab stays open and writes itself back at app close. **(3) After a failed save, restart discards the app's own rescue copy** — re-labelled "already safe on disk", then deleted (LL-040 verifier-blind-spot). **(4) If the editor falls back to simple mode, nothing typed is ever written** — `NotePane.svelte:723-743` omits the change listener that exists only at `:633`. *I verified this one myself: `EditorView.updateListener` appears exactly once in the file, in the primary state.*
>
> ### ⚖️ THE PATTERN THAT MATTERS MOST — our own law, violated by the passes enforcing it
> **"Half a sweep" — a fix applied to one branch and not its identical twin — explains a third of the register.** Eleven verified instances: the refuse-to-overwrite cure exists in one settings file and not four siblings · delete closes tabs, overwrite doesn't · collections resets on universe switch, workspaces doesn't · the rename fix landed on one branch of one function and not the branch below it · note-rename runs heavy work detached, folder-rename runs it inline · one Base command is non-blocking, its sibling isn't · one Tasks surface reverts a failed checkbox, the other doesn't.
>
> **This is the Whole-Ecosystem Fix Law being violated by the very passes run to enforce it.** Proposed change: *"one of N identical sites changed" becomes an automatic inspection finding*, and a fix is done only when every sibling is brought along in the same commit behind a shared helper.
>
> Five further classes, each with a concrete rule: **② "couldn't read it" → "you have none"** (one shared settings helper so a new store cannot omit the latch) · **③ success reported for work that didn't happen** (8 items; ban the bare `return` and the discarded Rust write result on any write path) · **④ write first, check second** (5 items; the check comes first, always) · **⑤ frontmatter hand-rolled at every writer** (correct shared helpers exist and are ignored; the frontend quoter still lacks the backslash fix the Rust one has) · **⑥ comments that promise a guarantee the code does not provide** — *the most dangerous, because a confident comment stops the next reader from checking.* Four found, including `"the 2nd screen never writes"` on a path that writes to the user's Markdown.
>
> ### 🆕 FILED — PJ-200…207
> - **PJ-200** — the four settings files (app-killer ①). One shared helper, not four patches.
> - **PJ-201** — the fallback editor with no change listener (app-killer ④).
> - **PJ-202** — the discarded rescue copy on restart (app-killer ③).
> - **PJ-203** — Overwrite destroying the surviving note (app-killer ②).
> - **PJ-204** — the frontmatter block-boundary + quoting corruption family (§2 items 5/6/7 — one root).
> - **PJ-205** — the save/flush-path group (dropped save batches, the 0.8 s property window, tab-switch recovery gap).
> - **PJ-206** — the rename-path group (single-library cascade contradicting the ONE-universe ruling; Base rename stripping its extension; folder-rename freeze).
> - **PJ-207** — **the unreachable repair pass.** Recommended to lead §3: several other fixes are only safe *because* they assume a self-heal that is not wired, and its error message names a Rebuild button that does not exist.
>
> ### ⚠️ NOT TRIAGED — stated, not buried
> **The 2026-07-30 run lost 25 candidates to server errors.** That register says so itself: *"those candidates died unverified — an under-count, not an all-clear."* **They are not in the 31 and nobody has ever looked at them.** Re-running that scope is owed. Also: the 17 "already fixed" were spot-checked at 3 of 17, not all re-verified; and **nothing here was reproduced on the running app** — per Reproduce-First, that is the first step of each fix, not of this triage.
>
> ### 🔁 Process observations
> - **The registers were never the backlog — the triage is.** Three feeds totalling 95 entries described 31 problems, and 17 were already dead. Left untriaged another week, the ledger would have carried ~64 phantom items.
> - **A gate number goes stale inside one day.** I nearly wrote "791 tests / 69 files" into the Orientation doc from memory; the actual gate was **875 / 74**. Re-run, never recall.
> - **The Boss's steer beat the search.** *"Instead of exploring for a solution, try to see how we overcame the same issue before"* turned an open-ended boot investigation into reading our own MIG-079 record, where the primitive was already written down.
>
> **Gates at close:** vitest **875/875** (74 files) · svelte-check **0 errors** · i18n parity **15/15 ✓** · Rust green.
>
> ---

**Version 1.64 | 2026-08-01**

> **What changed in v1.64** (**Stage-B SHIPPED and Boss-validated — the universe is one folder. Then the Search Hub was opened up: the concept was missing, the Boss stated it, and the cross-check found the bridge half-built along a language axis. MIG-109 allocated, not scheduled. Ultracode**):
>
> **► NEXT ACTION — Boss's call.** MIG-109 is **reserved, not scheduled** (Boss: *"to be dealt with in the right time"*). The standing queue behind it is unchanged: **MIG-104 Slice 8 + 8b**, then the two inspection triage feeds (24-item 2026-08-01 + 27-item 2026-07-30) and PJ-187's 19 M-cost register.
>
> ### ✅ CLOSED — MIG-108 "One Universe, One Location"
> **Stage-B ran on the live universe and PASSED** (~27 min on resume). Independent post-check: 7,827 notes · 234,236 links · 7,827 schedules · 1,577 aliases — all exact — and **zero stale paths across all 13 path-bearing tables**. Earned weight moved +3.4657, which is exactly five wikilink traversals (5 × ln 2) from the Boss's own testing during the run — nothing lost. The first attempt failed after 45 minutes and rolled back correctly: the Phase-4 audit had me widen the VERIFY to 12 tables while leaving the SWEEP at 5, so the check could count stale rows nothing could repair (14 orphaned `note_embeddings` + 6 `note_body`). Fixed at `f5ca0279`, RED-proven, and the journal now records WHY it stopped rather than only that it did. **MIG-108 is closed.**
>
> ### 🐎 The Search Hub had no horse — the Boss supplied it
> The **Semantic** result group was built with **no concept paper**, which under `00-MASTER` means it should not have existed. Three independent readers searched every concept paper, migration doc, audit, session log, MoCh, Lessons-Learned and all 63 prior ledger versions for a purpose statement, a recall goal or any ruling on it: **nothing**. Boss-stated concept, 2026-08-01: *"If a user searches for something, it will help them find every note that matches their search query, **regardless of the language of the related notes**… an **aerial view of their knowledge**."* Recorded as **`docs/concept-papers/33-search-aerial-view.md`**, which now governs `06-search-hub.md` §2.
>
> ### 🔍 What the cross-check found (Boss-directed: "cross-check the Search Hub with the Index")
> **Both surfaces miss the dictionary, for opposite reasons.** The Hub feeds it the RAW typed string (`المعرفة` — nothing strips `ال`); the Index feeds it a VOCABULARY term (`معرف` — a stem, not a dictionary word). Measured live: the vocabulary holds `معرف` (448 docs) and neither `معرفة` nor `المعرفة`; the dictionary holds `معرفة` and not `معرف`. **One side stems Arabic, the other stores dictionary-forms, and for Arabic they can never meet.**
>
> **This is why it tested clean and still failed.** English stems to itself, so `knowledge` is in both the vocabulary (1,937 docs) and the dictionary — the M12 example `tree → شجرة` genuinely works. The capability is **half-built along a language axis**: English→Arabic bridges (`knowledge` → `علم`, 778 docs); Arabic→English cannot. The Boss's recollection that his approval test passed is **correct** and is now explained rather than doubted.
>
> **The answer is present and unreachable:** every `c:knowledge` bridge term is live in his universe — knowledge 1,937 · علم 778 · cognition 19 · Wissen 16 · connaissance 13 · savoir 12. Only the lookup fails.
>
> ### 🆕 ALLOCATED / FILED
> - **MIG-109 — "Search as Aerial View."** RESERVED, **not scheduled**. Scope: the normalisation contract between indexer, query parser and lexicon corpus (§4.3); the bridge reaching the default route (§4.2); the relative cutoff (§4.1). Architect phase opens with §1 of the concept paper and must answer its four §7 rulings first.
> - **PJ-196** — the Semantic group's cutoff keeps only results within 0.03 of its own best score, so a strong match *suppresses* the rest: `المعرفة` returned **2 of 7,750** because the Boss owns a note with that title. Shipped 2026-04-10 (`8f21f657`), never since changed, never designed. Also empties `constellation_search_similar` by construction. **In MIG-109's scope.**
> - **PJ-197** — Arabic **fused tokens** in the FTS vocabulary: `معرفغرض` (36 docs), `معرفعلم` (25), `علممعرف` (18), `دارمعرف` (16). Two words welded into one token. Upstream of MIG-109 and of any "completeness" claim; **own investigation**, not folded in.
> - **PJ-198** — `notes_vocab` under-delivery: the `≈ similar` lookup table holds 538,813 entries against a real vocabulary of 6,498,791 (**~92% empty**) so *"valid suggestions are silently dropped."* Discovered in the Boss's own `FTS-Health-Forensics-2026-06-23` §A.1–A.3, a fix was recommended and ordered, it was **never built and never filed** — a WA#6 breach sitting since June. Now filed.
> - **PJ-199** — the `via {lemma}` badge is rendered in a view where all six result constructors set it to `None`: structurally unreachable. Cosmetic today; it is also the record's own evidence that the default route was *intended* to bridge.
>
> ### ⚠️ NOTED — the Hub's acceptance gate cannot currently pass
> `06-search-hub.md` §10 lists *"cross-lingual `via {lemma}` bridges work"* as an unticked acceptance box, and §11 records **"Enabled in bring-up: no"** for BOTH the Search Hub and the Index panel. Neither surface has passed bring-up; the concept paper above is the precondition for the Hub's.
>
> ### 🔁 Process observations from this job
> - The 0.03 cutoff predates the Migration Rule, the Safety-Inspection standing order and WA#5. **No audit, inspection or migration has read it since 2026-04-10** — the safety sweeps hunt silent *failures*, not silent *design gaps*, and a function with no concept paper is invisible to both.
> - **The parity tool accepted an orphan.** `searchBadges.concept` — correctly flagged for deletion by PJ-019 after the 2026-05-05 revert — was re-classified as live and translated into 13 locales on 2026-08-01. A key with no reachable owner passed the new guard.
> - `docs/User Manual.md:709` documents the Search Hub as **five** groups; the Semantic group the Boss was looking at is absent from the list while line 665 documents its badge. The manual contradicts itself. Fold into MIG-109's docs pass.
>
> ---

**Version 1.63 | 2026-08-01**

> **What changed in v1.63** (**the Stage-B gate is CLEARED — 101 confirmed safety findings fixed, none deferred. Plus i18n parity across all 15 bundles + a guard. Ultracode**):
>
> **► NEXT ACTION — Slice 7 / Stage-B: the live unification of `E:\Constellation Universes\Eisa Cognitive Knowledge`.** Both gate halves are now GREEN: the complete whole-app inspection (58 confirmed, all fixed) and the MIG-108 Phase-4 audit (31, incl. 3 BLOCKERs, all fixed), plus a per-build verification sweep over the remediation diff (36 more — 12 in-diff regressions fixed before commit, 24 pre-existing filed). Remaining order after Stage-B: **MIG-104 Slice 8 + 8b**, then the two triage feeds.
>
> ### ✅ CLOSED by this session
> - **The Stage-B hard gate** (carried since v1.62) — both halves discharged; evidence: SESSION-LOG 2026-08-01 §1–§5, Orientation v3.80, commit `4de3a585`.
> - **58 whole-app inspection findings** + **31 MIG-108 Phase-4 audit findings** + **12 in-diff regressions**. Four APP-KILLERS among them, one introduced by the remediation itself and caught only by the verification sweep — which is the argument for running it.
>
> ### 🆕 FILED
> - **`lab/reports/inspection-2026-08-01-remaining.md`** — 24 pre-existing findings (13 HIGH, 8 MED, 3 LOW) surfaced by the verification sweep. **Triage owed**, and dedupe first against the 2026-07-30 feed (27 items) and PJ-187's 19 M-cost register. Highest-signal: `universe.rs:118` collapses an unreadable `universes.json` into an empty registry which four write paths then save back (**silently deleting every registered Universe** — the fix `libraries.json` already has); `set_active_universe` holds the path mutex across a DB lock (whole-app freeze); and **`reconcile_filesystem` — named by many fixes as "the authoritative self-heal" — has no user-reachable trigger at all**, which silently upgrades a family of "eventually consistent" defects into "permanent".
> - **The POSIX-path test blind spot** — the watcher-adopt suite drove `/n.md` paths where `normPath` is the identity function, so a Windows-only total failure of the external-change subsystem kept 803 tests green. Any test exercising a path comparison must use a backslash path.
>
> ### ✅ i18n locale parity — 15/15, and now machine-checked
> `scripts/i18n-parity.mjs` (+ `npm run i18n:parity`) + `tests/i18n/locale-parity.test.ts` (55 tests). **806 translated strings** across 13 locales, **11 keys added to en.json**, 3 orphans deleted, and **three real CLDR plural defects fixed**: `ru` had **no `other` category at all**, `es`/`fr`/`pt` had no `many`, `ar` had no `zero`. None of these crashed — `resolvePluralForm` falls back category → other → one, so they rendered the **wrong grammatical form silently** since MIG-087. Full record: SESSION-LOG 2026-08-01 §1, Orientation v3.80.
>
> **Two premises of the incoming brief were wrong and are corrected in the tooling** — (1) **en.json is the SEVERE direction**: `t()` falls back active → en → **raw key**, so a key missing from en renders the literal key path in **all 15 languages**, while a key missing elsewhere merely renders English; (2) the `plurals.*` "gaps" were **correct CLDR, not drift** — `two` exists only in ar/he, `few`/`many` only in ar/ru, and a union-based reference would have forced permanently-dead keys into English. The tool therefore governs ordinary keys by the **union minus exemptions** and `plurals.*` by **`Intl.PluralRules`** — the same engine the runtime uses.
>
> ### FILED
> - **PJ-194 — `L()` in StyleSetter treats an intentionally-empty string as a miss.** `styleSetter.labels.an` is `""` in he/ja/ko *on purpose* (no indefinite article in those languages), but `L()`'s `!v || v === key ? en : v` falls back to the English, so the bold-text preview renders **"An りんご" / "An 사과"**. **No locale-data value can fix this** — empty renders English, non-empty renders a word those languages don't have. One-line fix in `StyleSetter.svelte` distinguishing "absent" from "intentionally empty"; deliberately NOT taken because the task scoped out component code. Held behind a per-entry allowlist + a stale-waiver test. **Ruling owed.**
> - **PJ-195 — the Orientation doc is 7,715 lines against SO#6's ~1,500-line split threshold.** SO#6 says "if it grows past ~1500 lines, split into `docs/orientation/` sub-documents". It is 5× over and every version bump copies the whole file. Long-standing; recorded now rather than left implicit.
>
> ### NOTED
> - **`git stash` mutates working-tree line endings on this repo.** A stash round-trip (used to prove the Sight v6 perf flakes pre-existing) rewrote LF → CRLF via `core.autocrlf`; vite's `.mjs` pipeline then failed with a bare `SyntaxError` where Node and esbuild were fine. Normalising back to LF left the diff byte-identical. Verify line endings after any stash round-trip.
> - **Sight v6 perf tests are confirmed flaky, not broken.** Identical code gave 854/854 then 852/854 on consecutive full-suite runs; a **clean-tree** run failed **3** assertions in the same family. Adjacent to PJ-172's serial lane — worth folding in when PJ-172 is next touched.
> - **SO#2 checked, no change required:** the User Manual already claims "15 languages, RTL-native" (line 5) and "All operators work in 15 languages" (line 175). This work makes the **existing** claim true rather than changing documented behaviour. No help topic documents locale coverage. PJ-146 (help topics English-only) is untouched and still open.
>
> **STILL OPEN:** MIG-108 Slice 7 (⏸ gated) + Slice 8 audit half · **PJ-187 (19 M-cost + the 27-item feed, triage owed)** · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105 (raised)** · PJ-164 · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (9th strike)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 · **PJ-174** · PJ-176 · PJ-177 · **PJ-180** · **PJ-183–186** · **PJ-188–191** · **PJ-193** · **PJ-194** · **PJ-195** · **PJ-137 (strike six)** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-136 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.62 | 2026-07-30**

> **What changed in v1.62** (**MIG-108 Slices 0–6 SHIPPED; Stage-A PASSED with three defects caught-and-killed; PJ-192 CLOSED; the One-Location law is standing behaviour. Ultracode**):
>
> **► NEXT ACTION — after the weekly limit resets (Aug 1): the COMPLETE safety inspection (whole-app; the 2026-07-30 run covered only 3 of 14 scopes) + the MIG-108 Phase-4 three-agent audit → then Slice 7 (Stage-B: the live unification of the Boss universe) → then MIG-104 Slice 8 + 8b.** Stage-B is HARD-GATED on the complete inspection — recorded when the truncation happened, holds now.
>
> ### ✅ MIG-108 through Slice 6 — see Orientation v3.79 for the build record
> Stage-A (Boss, full scratch copy): ALL PASS. Three defects caught by the rehearsal + Boss: the 25-min sky-trigger rewrite (→ 455 s measured after two fix rounds), 70 orphaned copy-class rows (the verify shared the decision's blind spot — LL-040's fourth appearance this week), and the stale registry cache duplicating every library in the tree (webview-only reload + bypassed invalidation hooks). All fixed, RED-proven where a harness exists. Honest timing ("several minutes") now promised ×15.
>
> ### CLOSED
> - **PJ-192** — `move_to_trash` demoted from command to pub(crate); its one Rust caller passes the universe root, which is the only meaning the collapsed setting has. (Slice 3.)
> - The "Consolidate trash" build (superseded): `consolidate_trash` ships INSIDE the migration flow + as a standalone idempotent pass.
>
> ### RE-RANKED / NOTED
> - **MIG-105 priority RAISED** (Boss connected it live during Stage-A): post-MIG-108 nested libraries are the UNIVERSAL shape — "which library owns this subtree" being read-time-derived (prefix + exclusion set + process-lifetime cache) is now load-bearing everywhere, forever. The Stage-A duplication was NOT the MIG-105 gap (it was the engine bypassing cache invalidation — fixed), but it is the class MIG-105 retires.
> - **Filed in-pass:** the future relative-paths portability migration (move a universe = move one folder + one registry entry; MIG-108 created its precondition — one shared prefix) — folded into the MIG-105/PJ-145 complex rather than a new number.
> - `link_library_as_universe` double-entry registration fix deferred to the Slice-8 registry-normalization note (conforms to One-Location already).
>
> **STILL OPEN:** MIG-108 Slice 7 (⏸ gated) + Slice 8 audit half · **PJ-187 (19 M-cost + the 27-item feed, triage owed)** · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105 (raised)** · PJ-164 · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (9th strike)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 · **PJ-174** · PJ-176 · PJ-177 · **PJ-180** · **PJ-183–186** · **PJ-188–191** · **PJ-193** · **PJ-137 (strike six)** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-136 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.61 | 2026-07-30**

> **What changed in v1.61** (**the S-SWEEP LANDED, Boss-validated end to end — Stage 1 all-Pass. Two Boss rulings queue the next migration. Ultracode**):
>
> **► NEXT ACTION — `/migration` Phase 1 (Architect): "One Universe, One Location" (+ one central `.trash` at the universe root).** Boss-ruled 2026-07-29/30. Measured scope: 18 of 20 libraries outside the root (ALL are TEST libraries, Boss-confirmed — the ideal rehearsal), 7,684 notes / 297.8 MB, same volume. Reverses the "read in place, never copied" invariant — amend CLAUDE.md IN the migration. `search.db` keys 7,672 rows by old absolute paths and is the system of record for 234,234 earned weights + 7,825 review schedules → **in-place SQLite path rewrite, never drop-and-rebuild**. Then **MIG-104 Slice 8 + 8b** (archive hook BEFORE the `DELETE FROM note_meta` at search.rs:9845). The PJ-187 register triage with the Boss is still owed.
>
> ### ✅ PJ-187 S-SWEEP — LANDED (33 fixes: 29 register + 4 in-flight)
> Boss Stage-1 validation in full: deleted-notes-stay-deleted · trash de-collide via Delete AND via Overwrite · Escape keeps a retyped title · Move doesn't freeze · the 7-step regression sweep — **all Pass**. Gates at landing: Rust **1287/0** · vitest **67/717** + Sight **5/84** (PJ-172 serial lane) · svelte-check **0**. Reports: `PJ-187-S-SWEEP-2026-07-29.md` (+ addenda), SESSION-LOG 2026-07-29 §7–§8, 2026-07-30 §1.
>
> ### ★★ The pre-commit inspection caught SIX defects in the sweep itself — fixed before landing
> Headline APP-KILLER: the new cascade gate returned `{ok:true}` for a refused write, so every departure silently destroyed a mid-cascade edit (LL-040's shape, FOURTH consecutive build). Plus: collections latch never reset on universe switch (cross-universe overwrite); stale-payload save race (now single-flight); `drainCidEnsure` missing its reindex (Whole-Ecosystem law violated INSIDE the sweep, second instance); both nav paths consuming the incoming note's recovery net on an aborted nav. All RED-proven. **27 pre-existing findings** → `lab/reports/pj187-inspection-2026-07-30-remaining.md` (1 APP-KILLER, 10 HIGH, 13 MED, 3 LOW); dedupe against the 19 M-cost register at the next triage. NOTE: 25 verify agents died to server errors — the feed is an UNDER-count.
>
> ### CLOSED / FILED / RE-RANKED
> - **CLOSED:** the 29 S-cost register findings (evidence: the sweep report + Boss Stage-1) · the trash-destination divergence (Boss-found) · the two-implementation trash de-collide (Boss-found).
> - **FILED: PJ-193 — the in-app Trash browser/restore UI.** No way to see or restore `.trash` inside Constellation (grep: zero UI); the manual promises "recoverable" with no in-app route. Boss chose file-and-decide-later. Obsidian has the same gap (third-party Trash Explorer plugin; Obsidian Sync skips `.trash`) — decide deliberately, not by inheritance.
> - **SUPERSEDED:** the approved "Consolidate trash" Settings button → becomes the migration's back-fill (Boss ruling 1 collapses the scope setting entirely).
> - **PJ-192** (Template-Studio undo trashes to universe root from Rust, setting-blind) — stands, ruling owed; becomes trivial after the migration.
> - **PJ-166 — NINTH strike:** invoked diff-scoped over 17 files, ran whole-app (85 agents, ~36 min). It caught the app-killer, AGAIN — but the per-build gate the standing order requires still does not exist.
>
> **STILL OPEN:** **PJ-187 (19 M-cost + the 27-item feed, triage owed)** · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105** · PJ-164 · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (9th strike)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 · **PJ-174** · PJ-176 · PJ-177 · **PJ-180** · **PJ-183–186** · **PJ-188–193** · **PJ-137 (strike six)** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-136 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.60 | 2026-07-29**

> **What changed in v1.60** (**PJ-187's 29 S-cost sites fixed in ONE sweep, per Boss ruling. Uncommitted, held for a single consolidated test. Ultracode**):
>
> **► NEXT ACTION — the consolidated Boss test of the PJ-187 S-sweep**, then land it as per-family commits. After that: **MIG-104 Slice 8 + 8b** (the archive hook MUST precede the `DELETE FROM note_meta` at `search.rs:9845` — FK enforcement fires the CASCADE there, so a hook at the later purge archives **nothing**; `tests_stage0_delete_order_defect`). Then Slices 9–15, then **MIG-105 Phase 2**. **PJ-187's remaining 19 M-cost sites still owe a Boss triage.**
>
> ### ✅ PJ-187 S-SWEEP — 29 of 48 register findings CLOSED in one pass
> Boss ruling (2026-07-29): *"All 29 small ones in one sweep."* Full write-up, family breakdown and proof table: `lab/reports/PJ-187-S-SWEEP-2026-07-29.md`. **Nothing deferred, nothing noted-and-shipped** (WA#6).
>
> **The six families.**
> **A · concurrent writers destroying each other's data (2)** — `universe.rs:117` (APP-KILLER) gets the pid+counter unique temp `write_gate` already had, so two windows saving at the same instant can no longer scramble your settings/session/collections/library-list into a file that then reads as empty. `libraries.rs:6753` — the trash de-collide check runs OUTSIDE the gate's lock, so a colliding delete was refused, the reason discarded, and a copy+remove fallback **overwrote the file already in the trash**.
> **B · a failed READ presenting as "you have nothing" (2)** — collections now refuse to write until a read has actually succeeded (an unreadable file showed as empty, and the next star wrote that emptiness over **every collection the user had**), and `saveCollections` retries once and raises a visible error rather than a `console.error` release builds discard.
> **C · a failure that says nothing at all (7)** — `sync_action_to_row` returns `Result`; `apply` returns `{ changed, refused }`; four panels (Review status, Style Setter, Global Tasks, Properties) gained a visible failure line, ×15 locales; the watcher's rejected re-index requeues (bounded) instead of leaving search stale until restart; bulk-accept ANNOUNCES the disk write even when the SQLite mirror fails — without that, the open note's next save erased the blocks the user had just accepted.
> **D · a stale read written back over fresher bytes (5)** — `flushOutgoing` gains the `isCascading` gate (NoteEditor refuses at four sites; the departure path was the fifth and open) · `addTagToNote` gains both guards its `addLinkToNote` sibling carries · `moveItem` gains `renameItem`'s flush-before-cascade envelope · `loadTabHistoryEntry` is reordered to `openNoteTab`'s audited ordering (it flushed BEFORE the incoming disk read, so keystrokes typed during that read were discarded by the model re-seed) · `cascadeFreeze` is refcounted, so the first of two overlapping renames no longer lifts the read-only overlay while the second is still rewriting files.
> **E · index↔disk divergence (4)** — `mark_with_parent` at all four gated sites (every note save was waking the watcher into a full library re-scan; the helper existed with ZERO callers — the LL-035 shape) · a re-index after the first `cid_cn` injection · `collect_md_paths` skips dot-directories, so **deleted notes stop coming back** in search and link suggestions · sky maintenance is no longer gated by the INCOMING back-fill stamp.
> **F · blocking the window, and leaked memory (5)** — `list_universe_folders` and `write_note` move to `#[tauri::command(async)]` · three model-disposal gaps closed (`deleteWithSetting`, the second screen's peek pane and its workspace restore) · the reliability sweep no longer deletes the `.tmp` a live save is writing through.
> **G · hand-built YAML lines (2)** — both now go through `quoteIfNeeded`, which itself had a gap (it never quoted a leading `- `) found by the proof.
>
> ### ★★ A PROOF REJECTED ONE OF THE FIXES — the fourth build in a row
> The trash-collision fix as prescribed (retry ONCE) still failed: the fresh name can be taken again while you reach for it. The concurrency harness surfaced it as a real sharing violation, not a theory. **Measured pre-fix loss: 13 of 24 concurrently-deleted files destroyed.** Only the bounded retry loop passes, five runs out of five. LL-039, LL-040, LL-041 and now this: every one of these fixes contained a defect that only a proof caught — never the suite.
>
> **Verification.** Rust **1285 passed / 0 failed** (5 new pj187 tests) · vitest **69 files / 782 tests** (8 new) · svelte-check **0**. Every new proof RED-proven by reverting its own mechanism, then restored. **Three fixes carry no automated proof** — the Escape-commits-title fix and the four visible-failure panels — because this repo has no component-test harness; named here rather than hidden. Whole-Ecosystem check run for the three multi-surface concerns (watcher suppression, tab-drop→dispose-model, hand-built frontmatter lines).
>
> ### ★★ STAGE-1: THE BOSS FOUND THE SWEEP'S OWN WHOLE-ECOSYSTEM VIOLATION
> Test 2 asked the Boss to create two notes with the same title — which Constellation refuses **by design**, as the User Manual states three lines above the paragraph this very sweep edited. Boss: *"We designed Constellation to NOT accept title duplication. You should know that."*
>
> Finding the correct path exposed the real defect: **the trash de-collide concern has TWO independent implementations and the sweep touched one.** `move_into_trash_folder` (**Delete**) was fixed; `move_to_trash` — the collision dialog's **Overwrite** and the PJ-088 conflict-sidecar path — kept its own private copy, so a name claimed between the check and the rename surfaced to the user as *"An item with this name already exists at the destination"*: an error naming a trash filename they never chose and cannot see, for an operation they had already confirmed. The law's canonical shape, **committed inside the sweep whose report claimed the concern was swept.**
>
> **Fixed** with ONE shared helper (`trash_move_decolliding` + the re-runnable `free_trash_name`); both call sites route through it so they cannot drift again. Two more proofs; Rust **1287 passed / 0 failed**. Lesson: *a per-concern grep is only as good as the concern's NAME* — the check ran against the concerns I had already assigned the fixes to, and never asked whether "trash de-collide" was itself multi-surface, because the triage entry named a single line.
>
> ### ★★ STAGE-1 PART B — PASSED, and finding the files exposed a SECOND real bug
> Part B passed exactly as designed (`Overwrite Test.md` = VERSION ONE, `Overwrite Test 1.md` = VERSION TWO, nothing clobbered, no refusal). It read as a failure only because **my instruction named the wrong folder** — and the reason it was the wrong folder is a defect: **four paths displace a note and they did not agree on where it goes.** Delete honours **Settings → Deleted files**; Overwrite-on-create, Overwrite-on-rename and the PJ-088 conflict sidecar hardcoded `<library>/.trash` and never read the setting at all. On the Boss's universe — whose libraries live OUTSIDE its root — those are different TREES, not neighbouring folders. With the DEFAULT `trashDestination: 'system'` it is worse: Delete uses the Recycle Bin while Overwrite silently creates a `.trash` folder inside the library the user never opted into.
>
> **Fixed** by extracting `resolveTrashDestination(path)` from `deleteWithSetting` and rerouting `moveToTrash` through it + `deletePath` — one implementation for every displacement path. Predecessor → Replacement entry written BEFORE the edit (SESSION-LOG §7.5); no IPC command retired (`move_to_trash` keeps its Rust caller). Proof `tests/pj-187/trashDestination.test.ts` (7 assertions), RED-proven: 4 of 7 fail against the pre-fix body.
>
> ### NEWLY FILED
> - **PJ-192 — the Template-Studio undo path** (`universe.rs:2426`) trashes to the universe root by design, ignoring `trashFolderScope`. It is Rust-side with no access to the frontend setting, and the existing comment shows the choice was reasoned. **Surfaced for a Boss ruling rather than changed silently** — the only displacement path not yet unified.
>
> ### RE-RANKED
> - **PJ-187** drops from 48 open findings to **19** (all M-cost), still owed a Boss triage. The five APP-KILLERs are now: `yamlDoc.ts:311`, `universe.rs:85`, `+layout.svelte:3407` — `universe.rs:117` and the headline are CLOSED.
> - **PJ-166 — still the oldest unfixed process defect** (eighth strike, unchanged this pass).
>
> **STILL OPEN:** **PJ-187 (19 M-cost sites)** · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105** · PJ-164 · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (8th strike)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 · **PJ-174** · PJ-176 · PJ-177 · **PJ-180** · **PJ-183** · **PJ-184** · **PJ-185** · **PJ-186** · **PJ-188** · **PJ-189** · **PJ-190** · **PJ-191** · **PJ-137 (strike six)** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-136 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.59 | 2026-07-29**

> **What changed in v1.59** (**PJ-187's headline APP-KILLER closed and Boss-validated — the third app-killer of the day. One new law, LL-041. Ultracode**):
>
> **► NEXT ACTION — MIG-104 Slice 8 + 8b.** The archive hook MUST precede the `DELETE FROM note_meta` at `search.rs:9845` (FK enforcement fires the CASCADE there; a hook at the later purge archives **nothing** — `tests_stage0_delete_order_defect`). 8b adds the note BODY. Then Slices 9–15, then **MIG-105 Phase 2**. **PJ-187's remaining 49 sites still owe a Boss triage.**
>
> ### ✅ PJ-187 headline — CLOSED. Cross-note property bleed.
> Edit a property in the **right-sidebar** Properties panel, then click a wikilink within the 800 ms debounce. Measured on note B's disk: **B gained A's `secret_a` key, and B's own `stage` was overwritten with A's edited value.** Silent, durable, no error.
>
> **Every guard was present, passing, and asking the wrong question.** The chain is thoroughly identity-guarded — every intent takes an `expectPath` and refuses a mismatch — but those guards verify *"do this tab id and this path refer to the same note?"*, and after an in-place navigation they DO (same tab id, new path, model re-pointed). Nothing checked whether the **rows** belonged to that note.
>
> Why the neighbours all missed it: the nav-flush is gated on `isNoteDirty` and a pending PANEL edit never reached the model, so the model was clean and the flush correctly skipped · `tabChanged` is `tabId !== prevTabId` and the id never changed · MIG-107's `localEditPending` guard then blocked the props-changed re-seed too, correctly preserving rows that happened to be the wrong note's · and the panel's own onDestroy identity gate never ran, because the sidebar instance is mounted **without a `{#key}`**. Its twin inside `NoteEditor` IS keyed and was safe throughout — **that asymmetry was the bug report.**
>
> **Fixed** with `rowsBelongToTarget(seededForPath, targetPath)` in `propsCommit.ts` — provenance, enforced at the commit rather than by keying the mount, and placed where it can be tested without a component harness. The pending edit is DROPPED, not redirected (by then the model has already been re-pointed); that matches what the teardown path already does.
>
> ### ⚖️ ONE NEW LAW — LL-041
> *A guard that two IDENTIFIERS agree is not a guard that the PAYLOAD belongs to them.* Data that outlives the thing it was read from needs PROVENANCE, not just a destination check. Enforce at the point of damage, not by keying the mount. Put the decision where it can be tested. And when two instances of one component exist and only one is keyed, that asymmetry is the bug report.
>
> ### NEWLY FILED
> - **PJ-190 — flush pending PANEL edits at navigation time.** The nav already flushes a dirty MODEL; it has no notion of an uncommitted panel edit, so such an edit is now safely dropped rather than saved to its own note. Saving it properly is the better outcome and needs the panel to expose "I have a pending edit" to the nav path.
> - **PJ-191 — key the right-sidebar PropertyEditor** on `sidebarTab.id + '|' + sidebarTab.path`, for symmetry with the already-protected embedded twin. Not required now (the commit guard holds however it is mounted) but it would make the whole class unrepresentable for this panel.
>
> **Verification:** vitest **64 files / 724 tests** (was 63/722) · svelte-check **0** · Rust untouched. RED-proven by neutering the predicate, with a no-navigation control that stays green. Docs updated ×15 **in this commit, not as a catch-up**.
>
> ---

**Version 1.58 | 2026-07-29**

> **What changed in v1.58** (**PJ-181 CLOSED and Boss-validated — and the build's own inspection found an APP-KILLER IN THE FIX before it shipped. One new law, LL-040. Ultracode**):
>
> **► NEXT ACTION — MIG-104 Slice 8 + 8b.** The archive hook MUST go BEFORE the `DELETE FROM note_meta` at `search.rs:9845`: FK enforcement fires the CASCADE there, so a hook at the later explicit purge archives **NOTHING** (`tests_stage0_delete_order_defect`). 8b adds the note BODY (Boss decision #6). Then Slices 9–15, then **MIG-105 Phase 2**. **Owed first, though: a Boss triage of PJ-187** (5 APP-KILLERs, headline `PropertyEditor.svelte:974`).
>
> ### ✅ PJ-181 — CLOSED. A merely-VIEWED note overwriting a newer external edit.
> View a note, type nothing, close it — `NoteEditor`'s teardown still stashes a write-ahead entry, and nothing clears it for a closed note. The note is then edited OUTSIDE Constellation (Syncthing / another device / `git pull`; the watcher ignores it because the note is closed). `cid_cn` is the note's IDENTITY and never changes with an edit, so `resolveNoteContent`'s cid check passed, the stale view won the screen, the model was born DIRTY with it, and the first tab switch wrote it over the newer file. **Measured: the flush returned `{ok:true}` and the externally-added paragraph was gone from disk.**
>
> **The root:** the net entry recorded WHAT it held and never WHY. It now carries `snapshot?: boolean` — "this content was already durable when stashed" — and `resolveNoteContent` refuses a snapshot whose content no longer matches disk. A genuine recovery copy (PJ-102 Recipe S) is untouched, and so is a legacy entry with no flag: unprovable → treated as real work, the direction that never discards.
>
> ### ★★ THE INSPECTION FOUND AN APP-KILLER **IN THE FIX** — measured, before it shipped
> The flag was first hard-coded `true` in the `!needsDiskSave` branch, on the reasoning that reaching there means "nothing changed since the last durable save". **It does not.** `needsDiskSave` is NotePane's view-level `dirty`, cleared at save-REQUEST time (`NotePane.svelte:340`) and never restored on failure. So after a FAILED save any teardown re-stashed the user's ONLY copy flagged *already durable*, and the new branch rejected and **deleted** it. The fix would have destroyed exactly what the net exists to protect. Now derived from the MODEL, which owns durability.
>
> **And two of the three new tests were worthless when written** — one passed for a harness bug (a path where a save ENV was expected), one passed with the flag hard-coded. Both replaced; the surviving assertion is proven load-bearing against the old predicate.
>
> ### ⚖️ ONE NEW LAW — LL-040
> *A flag named for an INTENT is not a fact about DURABILITY — read every assignment site, and derive safety-critical predicates from the layer that owns the truth. And a test of the CONSEQUENCE cannot pin a DECISION whose consequence is ambiguous: assert the predicate at the point of decision, then prove the assertion load-bearing by replacing it with the old value.*
>
> ### NEWLY FILED
> - **PJ-188 — the write-ahead net's localStorage blob is unbounded.** Never pruned, never capped, and `setWriteAhead` swallows a quota exception with an empty `catch` — so on quota exhaustion the crash-recovery net silently stops persisting. Pre-existing; surfaced while sizing the PJ-181 fix (which is why that fix carries a boolean and not a second copy of every note's bytes).
> - **PJ-189 — net entries from the PREVIOUS build carry no `snapshot` flag**, so they keep the pre-PJ-181 behaviour until the note is next opened-and-closed under the new build. Deliberate (unflagged → treated as real work → never discards), but the fix self-heals per note rather than instantly.
>
> **Verification:** vitest **63 files / 722 tests** (was 62/716) · svelte-check **0** · Sight perf SERIAL 31/31 · Rust untouched (frontend-only). Boss-validated twice — once on the original fix, once on the corrected build.
>
> ---

**Version 1.57 | 2026-07-29**

> **What changed in v1.57** (**PJ-182 CLOSED — and it was 20 surfaces across two languages, not one function. `/simplify` then found three more defects in the fix itself. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-181** (APP-KILLER, `store.ts:2448`): the write-ahead net is restored on a `cid_cn` match with **no freshness check against the disk bytes it just read**, and a net entry is stashed for merely-VIEWED notes. View → close → the note is edited externally (Syncthing / second device / git pull; the watcher ignores it because the note is closed) → reopen shows the **stale** content with the model born dirty → the first tab switch writes it over the newer file. `restoreSessionTabs` (`store.ts:2930-2940`) already solves exactly this on the sibling path — copy that arbitration. Then **MIG-104 Slice 8 + 8b** (the archive hook must precede `DELETE FROM note_meta` at `search.rs:9845` — FK enforcement fires the CASCADE there, so a hook at the later purge archives **nothing**; `tests_stage0_delete_order_defect`). Then Slices 9–15, then **MIG-105 Phase 2**.
>
> ### ✅ PJ-182 — CLOSED. A reproduced content-loss bug that was six times larger than filed.
>
> Filed as *"a zero-indent YAML block list projects as an empty list — `store.ts:2009`"*. Verified by running, then swept ecosystem-wide per the Whole-Ecosystem Fix Law (38-agent workflow, every candidate adversarially refuted): **20 confirmed surfaces**, including **six APP-KILLERs not previously known**. Boss ruling: *fix the whole thing now*, five slices.
>
> **The single root cause.** Eight independent sites answered *"is this line a continuation of the previous key's block sequence?"* from **leading whitespace**, when YAML's rule is *"the trimmed line begins with a dash"* — a mapping key can never begin with one. `search.rs::parse_frontmatter` had carried the correct rule all along, in a comment that states it explicitly. **The rule was written down in exactly one of the nine places that needed it.**
>
> **What it cost the user.** The frontend READ the list as empty, so the panel showed nothing and the next write to that key deleted every item from the `.md` — silently, re-parsing cleanly afterwards. The Rust WRITERS were worse: renaming a note, editing a Bases cell, or resolving a contested parent spliced their own indented item in beside the user's column-0 ones and produced frontmatter that **no longer parses at all**, after which every future property edit on that note was silently discarded forever. `aliases:` was hit identically (breaking every backlink through that alias), and typed links written this way never reached `note_links` at all — the Living Link graph silently lost those edges, while the structural-link guard simultaneously failed OPEN.
>
> **Fixed:** one shared predicate per language (`isYamlSeqItem` + `yamlSeqItemValue` in `store.ts`; the new `src-tauri/src/yaml_lines.rs`), routed through **every** surface — 14 sites, not the 8 the sweep reported. Three further shapes closed in the same pass: **block scalars** (`desc: |` projected as editable text valued `"|"`, and the cache wrote `desc: "|"`), **flow sequences on the next line** (read-only, so the user could not edit their own tags), and **comments inside a block**. Plus, found in-pass and indentation-independent: **every ikhtilāf edit was a silent no-op**, because `nestedObjects` — the property's actual content — was compared by nothing and carried by no intent.
>
> **Verification:** every fix RED-proven by removing it, each with a control proving the indented form is untouched. Suite **678 → 716** (vitest 62 files) · Rust **1261 → 1277** · svelte-check **0** · Sight perf SERIAL 31/31.
>
> ### ★★ `/simplify` FOUND THREE DEFECTS IN THE FIX ITSELF — all three reproduced, all three fixed
> 1. A **block scalar was outside `immutableBlockKeys`**, so Slice 4 gave it the read-only widget and not the write-path refusal. Omitting the row from a props array **deleted the prose from the file**.
> 2. **`nestedObjects` was threaded through three layers and stopped at the fourth** — `composeFrontmatter`'s unchanged-check still decided from the display summary, so a row deletion was dropped. *A chain is exactly as strong as its last link.*
> 3. **The Rust twin had no comment concept**: widening the continuation-skip meant a `#` line among the items ended the skip and orphaned everything after it. The JS twin had always been right — **only the Rust half was wrong.** (LL-038 rule 4.)
>
> And the reuse review found **eight more sites of the same concern still hand-rolling the predicate** — including the very function the new module quotes as the rule's origin, and the direct siblings of two functions that WERE routed. **The Whole-Ecosystem Fix Law caught me inside the change made to obey it.** All now routed.
>
> ### NEWLY FILED
> - **PJ-183 — the Rust frontmatter BLOCK WALKER.** Eleven sites still own a private state machine for "consume/track the block under key K" (`skipping_list_items`, `in_alias_list`, `in_old_list`, `in_list`, `in_aliases_block`, `in_aliases`, `skip_block` ×2, `current_key`, `current_structural`, `in_tags`). The shared *predicate* stopped the bleeding; the shared *walker* is the end-state. Two of those state machines are verbatim copies of each other (`sources/mod.rs`).
> - **PJ-184 — give a block scalar its own `PropertyType`.** It currently borrows `nested-map`, smuggles its `|` indicator through `value`, puts a prose preview in `nestedKeys`, and is labelled **"Nested map" to the user in 15 languages**. Form-Aligns-To-Purpose: the row must express what the thing is. Small — the read-only rendering already exists.
> - **PJ-185 — `parseFrontmatterDoc` returns `props: []` on a parse error.** If PJ-137 ever swaps the projection, a note with broken frontmatter would show NO rows where it shows readable ones today. That is a Boss ruling, not a silent consequence.
> - **PJ-186 — the flow-sequence branch covers only a single-line `[...]`.** A multi-line flow sequence, and a flow MAPPING on the next line, still fall to read-only. Correct and inert, so not urgent.
> - **PJ-187 — the 2026-07-29 whole-app inspection register.** 50 unique confirmed sites (5 APP-KILLER · 10 HIGH · 29 MED · 7 LOW), owed a per-cycle triage with the Boss. `lab/reports/SAFETY-INSPECTION-2026-07-29-pj182.md`. Headline: **APP-KILLER `PropertyEditor.svelte:974`** — the right-sidebar Properties panel is never `{#key}`-remounted, so a pending 800 ms debounce that survives an in-place navigation (click a wikilink within 800 ms of a property edit) writes note **A**'s properties onto note **B**, durably and silently. Its NotePane twin is protected by a `{#key}`; the sidebar one is not. Also inside it: **APP-KILLER `yamlDoc.ts:311`** — `composeFrontmatter`'s malformed-YAML passthrough discards every property edit on such a note and reports the save as **successful** (`hasErrors` has zero consumers outside `yamlDoc.ts`), and **`yamlDoc.ts:362`** — the CST splice+append deletes YAML comments attached to the edited key, on the app's single most common frontmatter edit (adding a tag).
> - **PJ-166 — EIGHTH strike.** This build's inspection was invoked diff-scoped with `args.files` and returned `mode: "whole-app"` again: 88 agents, ~10.8 M tokens, ~30 minutes for a 13-file gate. It earned its cost *again* (it caught the half-routed `sources/mod.rs`), but **the per-build gate the standing order actually requires still does not exist, eight attempts in.** This is now the oldest unfixed process defect in the ledger.
>
> ### RE-RANKED
> - **PJ-137** (retire the hand-rolled `store.parseFrontmatter` for one YAML-document authority — `/migration`-sized) is now at **strike SIX**: PJ-136, MIG-101, the 2026-07-24 seq-of-maps inspection, and now PJ-182's three shapes all originate in the parser split. Recorded for whoever opens it: `projectProps` / `parseFrontmatterDoc` are **test-only today** — a spec-compliant projector already sits beside the hand-rolled one, dark, which is the cheapest possible starting point. The gaps it must close first are type detection (`date`/`datetime`/`link`/`nested-object-list` + key-driven typing), date normalisation, ikhtilāf rows, and PJ-185's error behaviour.
> - **PJ-180** gains evidence: `buildFullContent` is no longer *destructive* on the RICH fixture (PJ-182 fixed that at the source), but it still re-serializes — `'He said: "hi"'` comes back double-quoted. The `tests/g4/composeUpdated.test.ts` case that used to assert the destruction now asserts the re-quoting, because asserting the old damage would have pinned a defect in place.
> - **PJ-146** unchanged and now more visible: the Properties help topic is **English only**; the 14 translated help dirs do not carry it. The User Manual's Properties section WAS updated in all 15 languages this pass.
>
> **STILL OPEN:** **PJ-181** (► next) · **PJ-187** (the new whole-app register — 5 APP-KILLERs, owed a Boss triage) · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105** · PJ-164 (= MIG-104 Slice 12) · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (8th strike — the per-build inspection gate still does not exist)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 (→ MIG-104 Slice 14) · **PJ-174** (the earlier sweep register) · PJ-176 · PJ-177 · **PJ-180** · **PJ-183** · **PJ-184** · **PJ-185** · **PJ-186** · **PJ-137 (strike six)** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-136 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.56 | 2026-07-29**

> *(See `Constellation Pending Jobs v1.56.md` — the trail is durable, never overwritten.)*

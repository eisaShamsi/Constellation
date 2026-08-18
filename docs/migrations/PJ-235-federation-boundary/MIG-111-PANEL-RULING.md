# THE RULING — Inspection Panel, MIG-111 Phase 1.2

**To: Eisa. You delegated this decision; here it is, decided.**

---

## 1. The ruling, in one paragraph

**Nothing in the register displaces MIG-111 Phase 1.2. It resumes, and it resumes soon** — but not today, because two things inside Stage A's own footprint are unfinished and one live defect deserves a morning first. In order: **(1)** commit the corrected Arabic-overrides fix that is already written and sitting in the working tree; **(2)** fix three silently-swallowed errors in the Arabic re-index that can drop a note out of search entirely while telling you it succeeded — this is the only true app-killer anyone found in this whole review and it had no number; **(3)** fix "Open Existing Universe", which today makes the app *believe* it has switched universes while it keeps reading and writing the previous universe's search index — a live rehearsal of the exact mistake this migration exists to prevent; **(4)** reconcile the Pending Jobs ledger, which has drifted in four places; **(5)** amend the approved Phase 1.2 plan in two small ways so its own definition of done means what it says; **(6)** then cascade Stage A A1–A8 as approved, and Stage B after it. **Everything else in the register is deferred, explicitly, with a named reopen condition.** Two items the panel proposed as "one-line heals" are **forbidden as written** — I explain why below, because one of them would have been committed without review and would have dropped and rebuilt a 2.03 GB search index on your machine with no measurement and no way to stop it.

---

## 2. The ordered work list

| # | Item | Disposition | Why | Rough cost |
|---|---|---|---|---|
| **1** | **PJ-307b** — commit the corrected `arabic/overrides.rs` (soften one comment first) | **DO NOW** | Verified correctness fix already written, compiles, suite green (1521 run / 1501 pass / 0 fail). The version currently in `main` is the regressed one. Leaving it uncommitted is strictly worse than landing it. | 15 min |
| **2** | **PJ-316 (NEW)** — three swallowed errors + an unbounded writer-lock hold in `reindex_notes_matching_text` | **DO NOW** | `search.rs:13812-13814` skips a failed FTS delete, `:13819-13821` skips a failed insert, `:13823` commits anyway, and the panel paints it green. Delete-succeeds/insert-fails **removes a note from search while reporting success**. This is the Charter's app-killer class. Nobody filed it. | 3 h |
| **3** | **PJ-310 (RE-SCOPED, MED→HIGH)** — "Open Existing Universe" leaves the app half-switched | **DO NOW, reproduce first** | Verified: `open_existing_universe` (`universe.rs:1246-1250`, `:1314-1317`) sets the active pointer and calls **none** of `set_active_universe`'s three fan-outs (`universe.rs:1051`, `:1061`, `:1077`). Third door: `link_library_as_universe` delegates to it (`:1350`). Frontend calls neither the departure flush nor the reload (`UniverseManager.svelte:136-148`). | ½ day + Boss test |
| **4** | **SO#9 ledger reconciliation**, in the same commit | **DO NOW** | Four verified drifts in a twelve-item Group-1 list. Details in §3. | 1 h |
| **5** | **Plan amendment A4** — fourth refusal (fail-closed federation read) + real-directory tests on `resolve_owner` | **AMEND THE PLAN, then build inside Stage A** | `resolve_owner` (`owner.rs:149`) has **zero call sites and zero tests** — I grepped; the only other hit is its test module name. A3 makes it the router for every routed write, and it rests on a resolver that returns an empty list on a missing/unreadable/unparseable manifest (`universe.rs:682-694`). | 3 h |
| **6** | **Plan amendment A8** — extend `Aggregates` to diff `notes_vocab`/`notes_fts` | **AMEND THE PLAN, then build inside Stage A** | `Aggregates` (`vocab_harness.rs:59-69`) holds only link rows, edges and incoming counts. The test named *"routed write must match the owner's vocabulary"* cannot currently see vocabulary. Ten lines. | 2 h |
| **7** | **MIG-111 Phase 1.2 Stage A, A1–A8** | **THE MANDATE — resume, cascade** | Plan approved. Definition of done unchanged: the `#[ignore]` comes off `routed_write_must_match_the_owners_vocabulary`. With #5 and #6 in, that removal means what the plan says. | Per approved plan |
| **8** | **PJ-317 (NEW)** — gate the foreign FTS drop + rebuild on `owns` | **DO INSIDE STAGE A, not before** | `search.rs:4718` (drop) and `:6221` (rebuild) carry no `owns` gate, and `InitScope`'s comment (`search.rs:4600-4607`) explicitly blesses this on the false premise that the rebuild "involve[s] no process-global state" — it re-tokenizes through the Arabic global at `libraries.rs:4653`. This is PJ-302's own rule, applied where PJ-302 missed it. | 2 h |
| **9** | **PJ-320 (NEW)** — children already stripped of their sky triggers are never repaired | **FILE NOW, FIX AS STAGE A "A0"** | PJ-302 stopped the stripping. It did not restore what earlier builds removed, and I found no `sky_nodes`/`sky_links` repair in `derived_heal.rs`. A4's first refusal is a trigger-capability probe — so on the day Stage A ships, **every routed write into a previously-migrated linked universe refuses by name**, and A8 cannot catch it because it builds both universes fresh. | ½ day |
| **10** | **Measure** the FTS rebuild on the 7,820-note universe | **BEFORE any decision on PJ-313's cleanup** | The only cost figure in the codebase is an *expectation* written in April (`search.rs:6209-6215`), and it explicitly asks for this measurement. `search.db` is **2,026,405,888 bytes (2.03 GB)**, verified. Nobody has ever run it. | 30 min |
| **11** | **PJ-313 + PJ-314 + PJ-315** — one job, MEDIUM | **AFTER Stage A** | Fix the delete/publish ordering; fan the Arabic re-activation out to `add_child_universe`/`remove_child_universe`; surface the loader's parse failure instead of an invisible `eprintln`; add the negative assertion the guarding test lacks. **No schema-version bump** until #8 and #10 are done. | 1 day |
| **12** | **MIG-111 Stage B, B1–B7** | **AFTER Stage A** | Per approved plan, with PJ-300 landed before B5/B6, PJ-222 folded into B6, PJ-245/246 folded into B1. | Per approved plan |
| **13** | **PJ-312** (search's swallowed errors + blank panel + stuck spinner) | **DEFER — first out** | Half a day, `SearchHub.svelte`. Promote its **error-surfacing** half if Stage B slips. | ½ day |
| **14** | **PJ-311** (Search Hub headline pass) — DOWNGRADED from HIGH | **DEFER** | Performance hygiene, not safety. | 1–1.5 days |
| **15** | The rest of Group 1 + PJ-306 | **DEFER as a block** | See §3. | — |

---

## 3. Per-item disposition — every open item

### The two the coordinator called HIGH. Neither survives as a HIGH.

**PJ-313 — Arabic override edits leave garbage in the search dictionary.**
**DOWNGRADE HIGH → MEDIUM. Rewrite the entry. Fix after Stage A. No schema bump.**
Three of its filed claims do not survive reproduction and must be struck: it does **not** corrupt the index (SQLite's integrity-check passes in every state), it does **not** break search (a query re-stems through the same live tokenizer and always lands on the current stem — the orphaned entry is unreachable), and the "database disk image is malformed" escalation **could not be reproduced** by running the exact stated sequence. What it actually is: permanent junk in the search vocabulary dictionary that no user-reachable path removes, leaking on **every** add *and* every remove, compounding as you tune a word. Its one visible surface is the **Index panel**, which reads that dictionary directly with a "appears 5+ times" gate (`libraries.rs:5549-5552`) — a real ghost inherits the whole pre-override count, clears that gate, and shows you a word that exists in no note. That is the honest case for fixing it, and it is a MEDIUM.
*Promotes if:* someone reproduces an actual query returning a wrong answer, or a genuine `SQLITE_CORRUPT`.

**PJ-311 — Search Hub's headline pass.**
**DOWNGRADE HIGH → MEDIUM. Defer to after Stage B.**
The measurements are right — ~1,200 paths, two writer-lock takes per path, a full untruncated body read, a full-byte hash even on a cache hit, all on the writer connection while an unused reader connection sits beside it. But "invisible stall / blocks the next search and any save" is wrong: the lock is taken and released **per path**, never held across the pass, and the command runs off the UI thread. It is contention and wasted work — nothing lost, nothing wrong, nothing frozen. It does not outrank an approved migration.
*Promotes if:* you feel a measurable delay on your own corpus and say so. **Before it is designed, run the one query nobody has run:** how many of the 217 summary-less notes actually compute to an empty summary — misses are deliberately never cached, so those notes pay the full cost on every search, forever, in the libraries you write in.

### The live defects

**PJ-310 — RE-GRADED to HIGH, fixed in the first batch.** The panel filed it as "may not refresh the frontend." It is not a refresh bug. After that button the app's search connection, its Arabic vocabulary, and your open tabs all still belong to the **previous** universe, while the app's pointer and its library list belong to the new one. The library cache does self-heal (it is keyed by universe path, `libraries.rs:181-198` — Lens A had this wrong, Lens C had it right); the search connection does not, because it short-circuits on a flag that only the missing call clears (`search.rs:11536`). **Fix:** one shared activation helper both commands call, plus the departure flush and reload on the frontend. **Reproduce it first** — Reproduce-First applies, and nobody has run this, only read it.

**PJ-316 (new) — the reindex's three swallowed errors.** Fix in the first batch. This is the single most serious finding in the review and it arrived with no number, discovered while checking whether PJ-308 should be reverted.

**PJ-307 / PJ-307b — commit the correction.** See §4(a).

**PJ-308 — LEAVE SHIPPED.** See §4(b).

**PJ-314** — a cUniverse link/unlink changes how words are stemmed with no re-index. Verified: the Arabic re-activation has exactly one caller (`universe.rs:1077`), so linking a child changes the verdict only at the next restart. Fix with PJ-313 (item 11) — same concern, and the Whole-Ecosystem Fix Law forbids fixing one and leaving the twin.

**PJ-315** — a damaged `arabic-overrides.json` silently installs an empty store. The codebase documents this against itself (`overrides.rs:407-412`): that file is *the only on-disk record of your overrides; nothing rebuilds them*. Filed LOW; it is the only **irreversible-loss** shape in the register. Fix with item 11, and put the error on screen.

**PJ-306** — test-only, seen only under artificial load. Defer. *Promotes if:* it fails a clean single-process run.

### The pre-existing Group-1 backlog

**Close in the ledger now — both are fixed in code and still carried as open:**
- **PJ-247** (item 9) — `run_one_safe` now uses a detached thread (`cece/orchestrator.rs:175`); documented in place as PJ-282.
- **PJ-251** (item 10) — the incoming signature now keys on name **and** type (`search.rs:2564-2565`).

**Re-word, don't re-do:**
- **PJ-235 + PJ-254** (item 3) — both named halves are guarded (`libraries.rs:2702`, `:2770`, `:1921`). Only the documented PJ-270 residue remains.

**Correct the metadata:** PJ-236 lives at `src/lib/libraries/store.ts`, not `src/lib/stores/store.ts`. PJ-242 has moved to `universe.rs:1673-1683`.

**Scheduled inside the migration, not carried separately** (these touch the same function bodies the plan already edits — doing them apart means editing twice and inviting exactly the drift the Whole-Ecosystem Fix Law forbids):
- **PJ-300** → land before **B5/B6**. Verified *not* a Stage-A blocker (the owner resolver deliberately bypasses that cache), but a hard blocker for B6, which removes the rename fence — at that moment an under-reported foreign set stops being wasted work and becomes real mis-routing.
- **PJ-222** → fold into **B6**. `collect_md_paths` (`libraries.rs:2830-2855`) skips symlinks and dotted entries and nothing else; a linked universe root is an ordinary folder under yours.
- **PJ-245 / PJ-246** (and PJ-255) → fold into **B1**.

**Deferred as a block until Stage B closes** — verified still live where checked, none touching the federation write path: **PJ-258, PJ-269, PJ-236/237/238, PJ-241, PJ-242/243, PJ-239, PJ-244**, plus PJ-248's Group-1 members and the carried sweep registers. *The reason is not their severity — it is shape.* Interleaving unrelated fixes with an in-flight migration on the same write path is how a half-migrated state ships, which this project has already paid three sessions to undo. *Promotes if:* Stage A stalls for more than a week, in which case take PJ-258 (already flagged "► Next action") and PJ-312 while it is blocked.

**PJ-301** — Group 2, but flag it: it is the universe **lock** identity ambiguity, and Phase 1.4 turns that lock into enforcement. It needs its measured decision **before 1.4**, not before 1.2.

**PJ-253, PJ-224, PJ-260, PJ-288** — awaiting your rulings, unchanged. Not blocking this sequence.

---

## 4. The three decisions you were asked for — decided here

### (a) PJ-307's Reproduce-First exception: **accept it for PJ-307b, and close the door behind it.**

Plain language: the coordinator changed some locking code by *reasoning* that it was wrong rather than *demonstrating* it was wrong. The rule says don't do that. It did it anyway, and the change it shipped introduced a **new** bug of exactly the kind it was trying to fix.

I accept the exception for the correction now sitting uncommitted, on one specific ground: **it is a restoration, not a new speculative fix.** It puts the code back into the shape it had before, and it makes the compiler enforce that shape — the publish function now physically cannot be called without holding the lock (`overrides.rs:624-627`). A restoration you can verify by reading the diff is not the thing Reproduce-First was written against. Commit it.

**Two conditions.** First, soften the comment in the code. It claims the regression was "strictly worse than the bug being fixed." That will not survive a challenge — both defects need the same microsecond-wide timing collision. The true and sharper statement is directional: *the fix took a function that was already safe and made it unsafe, in order to correct a flag-ordering problem in its neighbour.* Second — and this is the real ruling — **no further change to that publish path without a reproduction.** That ground is burned once. The exception is granted for this restoration and does not become precedent.

### (b) PJ-308: **leave it shipped. Do not revert. The question is closed.**

The panel split; one member ruled revert. I rule against, and the evidence is one-sided.

Plain language: PJ-308 made sure that when you add an Arabic word-override, the re-index that follows actually runs instead of silently doing nothing. The worry was that the re-index is the thing that leaves junk in the dictionary (PJ-313), so maybe it was better off skipped.

It was not. **Without** the re-index, the note keeps its old form while your searches use the new one — so the note becomes **unfindable by the very word you just taught the system**. The existing test proves this itself: it asserts zero matches *before* the re-index runs. Reverting would trade an unreachable ghost dictionary entry for a genuine hole in finding your own notes. And the revert would barely bite anyway: the situation PJ-308 guards against exists only in a narrow window right after startup or a universe switch — in normal running the re-index already ran before PJ-308 existed.

**But correct the register: "PJ-308 — correct in isolation" is wrong.** PJ-308 hardened the *door* to a function that still swallows three errors *inside* it. That is item #2 in the work list, and it is why this question was worth asking even though the answer is no.

### (c) Stage B rename scope: **within the owning universe only. The coordinator's assumption is upheld.**

Your ruling was "renames are in scope." The coordinator read that as: *renaming a note that lives in a linked universe works properly, doing its bookkeeping and its link-rewriting in that universe, using that universe's own vocabulary* — and **not** the full cross-universe cascade where renaming in universe A automatically heals every reference in universe B. It flagged that reading for correction and was never corrected. I confirm the narrow reading, for three reasons:

1. **The wide reading is a different engine, not a bigger step.** Your own Concept Panel requirement R23 defines the cross-universe cascade as: heal every referrer in every *reachable* universe, turn unreachable ones into **durable pending repairs that complete later**, and state both counts in the receipt. That needs a persistent pending-repair store that does not exist. It is called "the acceptance test of the whole migration" in your own documents and is scheduled as Phase 3. It cannot be a row in a Stage-B table.
2. **The wide reading is not safe yet.** The rename's federation boundary is computed from a resolver that silently under-reports when a manifest cannot be read (PJ-300). A cross-universe cascade running on an under-reported boundary would quietly repair *some* universes and skip others — with no error. That is worse than not cascading.
3. **The narrow reading already delivers the thing the fence broke.** Today, a link written as `[[refutes::Old]]` inside a linked universe is left broken by a rename because the parent does not know the word `refutes`. B5/B6 as scoped fixes exactly that, which is what "renames are in scope" was for.

**One addition I am ordering, because the narrow scope creates a visible gap.** After B6, a rename inside a linked universe will succeed while references to it *from other universes* stay broken — silently. That is unacceptable under your own "silent breakage forbidden" requirement. So: **B6 must state, in its receipt, what it healed and what it did not** ("renamed; 12 references updated in this universe; references in other linked universes are not updated in this phase"). Honest, cheap, and it turns a silent gap into a stated limitation until Phase 3 closes it.

---

## 5. What the coordinator must NOT do

The failure pattern this session is unmistakable and it is one pattern: **every error came from reasoning about the code instead of running it; every correction came from running it.** So:

1. **Do NOT bump `FTS_SCHEMA_VERSION` from 1 to 2.** Three of the four lenses recommended this as a "one-line heal." It is not one line — it is a schema migration that **drops and rebuilds the entire search index**, on a 2.03 GB database, inline during startup, under the lock nearly every command waits on, with no cancel and no progress bar, on a cost figure that is an unverified guess written in April against a smaller corpus. And it is not gated to your own universe, so it can reach linked universes' databases. It is forbidden until the rebuild is gated (item #8) **and** measured (item #10). One-line changes are the ones that get committed without review, which is precisely why this one is dangerous.

2. **Do NOT thread the Arabic vocabulary through the connection layer as part of Phase 1.2.** Three lenses wanted this; it sounds right and it is wrong on both cost and correctness. Cost: the tokenizer registration takes no vocabulary parameter today, so this is ~17 call sites across 8 files. Correctness: the same word-processing function is **also the query-side stemmer** (`libraries.rs:5843`, `:5944`, `:5971`), so freezing it per-connection at index time while queries keep using the live one would *create* a new mismatch of exactly PJ-313's shape. And PJ-313 is a *timing* mismatch inside one universe — a per-connection vocabulary does not fix it at all. **File it as PJ-319 for Phase 1.3.** In 1.2 it gets a refusal in A4 and visibility in A8, and that is all.

3. **Do NOT revert PJ-308**, and do not re-open the question.

4. **Do NOT design any fix for PJ-310 or PJ-313 before reproducing it.** Both were sold to me on source reading. Source reading was enough to establish the *state*; it is not enough to establish the *damage*, and the damage is what the fix must target.

5. **Do NOT run more than one `cargo test` sweep at a time, and do not edit source mid-sweep.** Several wrong conclusions this session were artefacts of concurrent runs. A contaminated green is worse than a red.

6. **Do NOT add work to the front of this queue.** Four separate proposals each grew the pre-Stage-A list — one to seven items. The panel's job was to cut, and it kept adding. If something new surfaces during the batch, file it; do not schedule it ahead of Stage A without coming back here.

7. **Do NOT batch the ledger reconciliation to the end.** SO#9 says the ledger is the *first* file opened at every close. Following the earlier proposals would have violated it four times in a row, in the file that exists to prevent exactly the drift this review found.

8. **Every build in this sequence is Boss-tested before commit**, and every test tutorial goes `tutorial-auditor` → `ui-inspector` → you. Items 1–4 batch into **one** test round; items 5–7 into a second.

---

## 6. What I do not rule on

Two things, and only two — both genuinely yours because they are about what you are willing to pay, not about what is true.

**(i) If the FTS rebuild measurement (item #10) comes back expensive, where does the cost land?** If rebuilding the search index on your 7,820-note universe takes seconds, the cleanup is trivial and I have already ordered it. If it takes a minute, the choice is: accept a one-time slow startup, or build a resumable background rebuild with a progress strip. That is a risk-appetite call about your own boot experience, and the code's own comment says to bring it to you. **Take this decision after the number exists, not before.**

**(ii) Phase 1.3's scope.** I have ruled the Arabic vocabulary out of 1.2 and filed it as PJ-319. Whether 1.3 is "thread the second vocabulary" or something else entirely is product sequencing — it depends on how much more of the federation boundary you want closed before you go back to building features. I will not decide how long you stay in this migration.

Everything else in the register I have ruled on above. Nothing is logged-and-shipped; every deferral has a stated reason and a stated reopen condition.

---

**Verification note, so you know what stands on what:** every file:line in this ruling I read myself in this session, including the ones where the four lenses disagreed. The claims I did **not** independently re-run are: the PJ-313 reproduction (one lens ran it in the test suite; nobody refuted it), and the count of summary-less notes in PJ-311 (nobody has measured it, and I have not asserted a number).
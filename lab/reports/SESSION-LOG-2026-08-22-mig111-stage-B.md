# SESSION LOG — 2026-08-22 — MIG-111 Stage B (B4)

**Session start:** head `919c1b7a`, tree clean, gates at handover: Rust 1532/0/19 ignored ·
vitest 997/997 · svelte-check 0 errors · 15/15 locales.

> **Working on: MIG-111 Stage B, step B4 — the read-side analytics' vocabulary** (the seven
> ambient `active_universe_vocabulary().structural_not_in_clause("link_type")` reads at
> cache.rs:516/548/1288, sight.rs:77, tension.rs:277, search.rs:189/267, plus the false
> federation claim at tension.rs:88-92).

**Concept (the horse):** *a read that classifies a universe's links must classify them with
that universe's own vocabulary — the reader names whose vocabulary it holds, instead of
silently reaching for whichever universe is active.*

---

## §B4 — read-side analytics take an explicit registry

### Scope, and why it is wider than the brief's seven sites

The brief named seven sites + the tension comment. The investigation (all in-source, this
session) added three surfaces by the Whole-Ecosystem Fix Law — one of them ordered by the
census itself:

1. **inspector360.rs:133/:285** — the census's own 2026-08-21 annotation says "**B4 must
   thread this**": `get_360_view` accepts Linked-Universe paths (`validate_path_in_any_library`
   iterates `load_all_libraries`, federated included) and classified them with the ACTIVE
   vocabulary. Its gap list (`ids()`) had the same defect.
2. **strata.rs:98** — same shape, and *reachable in normal operation*: the Sky enrichment loop
   (`enrichNodesBackground`, +layout.svelte:4506) calls `compute_note_strata` for EVERY
   federated library.
3. **libraries.rs:4040 (`scan_links_recursive`)** — same shape, PLUS it still re-read the
   process-global **once per directory** — the per-walk drift A5 fixed in its two siblings
   (strata / inspector360) and missed here.

### The census answers (written before the map was touched)

| site | whose vocabulary? | action |
|---|---|---|
| cache.rs:516/548/1288 | The universe each SCHEMA belongs to — these are per-schema readers looped over `main` + `cuN` | **Threaded.** `registry_for_schema` (new): `main` → active; `cuN` → `registry_for_root` on the attached root, STRICT. Resolved BEFORE any connection lock (`registries_for_schemas`) — `registry_for_schema` locks `state.federation`, and holding `federated_conn` across it would have nested the two mutexes for the first time in the codebase. |
| sight.rs:77 | ACTIVE — right **by construction**: no path parameter, reads only `state.db`, sx read under the same `db.lock()` as the query | Answer recorded at the site + census. Federated Sight = reserved MIG-063 family. |
| tension.rs:277 | ACTIVE — right **by scope**, *once the refusal is real* | `detect_tensions` now refuses non-own libraries via `require_own_library` (the shared own-scope decision, fail-closed). The :88-92 comment had always CLAIMED this refusal; the code never implemented it. |
| search.rs:189/267 | Hidden inside the generators — the exact "no caller can see" shape A5 deleted | **Threaded.** `stratum_sql_expr(&reg)` / `maturity_sql_expr(&reg)`; every caller answers at its own line. |
| inspector360 / strata / scan_library_links | The universe that OWNS `library_path` | **Threaded** via `registry_for_owner_of` (new): `resolve_owner` → active ⇒ active registry; linked ⇒ `registry_for_root`, strict. One value per walk (the B3 rule). |

### Generator callers, each with its answer

- **search.rs DDL + PJ-334 restore arms** — `owns`-gated (PJ-232 / Boss ruling 2), so the
  active answer is right there; explicit `ddl_reg` / `restore_reg` reads with comments. B2
  threads the DDL layer itself.
- **search.rs save/de-index tails** (`maintain_sky_after_save` now takes `reg`;
  `recompute_incoming_and_sky_for_target`; the delete tail) — each hoists ONE read shared by
  its incoming+sky pair, so one operation can no longer use two vocabularies. Phase 1.3
  revisits the index tail for routed saves.
- **name_fold_backfill.rs:184/189** — passes its own pinned-root `&vocab`. **This completes
  the 2026-08-21 inspection fix**: that fix resolved conn+vocab from the same root "so the
  pair cannot drift", but the two generator calls 100 lines later still read the global
  internally. Found because B4 made the generators take a parameter.
- **sky_backfill.rs `process_batch`** — hoists ONE read per batch shared by the Phase-A sx and
  Phase-D exprs (was THREE reads at three moments — a vocabulary change mid-batch could mix
  vocabularies within one batch). B1 threads it from the PJ-332 pinned root.
- **links_backfill.rs `recompute_sky_range`** — explicit read + B1 annotation.
- **search.rs:1090 test + cache.rs tests** — explicit `seeds_only()` (deterministic under
  concurrent tests, LL-049; clause registry-invariant per A2).

### The honesty note (verification is at the resolution layer)

`structural_not_in_clause` is **registry-invariant today**: A2's merge pin locks the
structural lane to `{contains, parent}` in every constructible registry (search.rs:4581's own
comment: "harmless only by the accident that structural_not_in_clause is vocabulary-invariant").
So B4's SQL output is byte-identical everywhere, and "each universe's own types" cannot be
proven at the clause layer. The proof lands at the RESOLUTION layer with a custom-type marker:

- `a_cu_alias_resolves_to_that_linked_universes_own_vocabulary` — cu0 resolves to the CHILD's
  own disk (its custom `refutes` is the marker; guard asserts seeds alone cannot pass). The
  attached root is `fs::canonicalize`d — the exact form `attach_all` builds (LL-048).
- `an_unknown_schema_alias_refuses_instead_of_guessing` — names the alias.
- `a_cu_alias_whose_vocabulary_is_corrupt_refuses_naming_the_universe` — corrupt ⇒ refusal,
  never a silent fallback to the active vocabulary.

### A regression caught before it shipped — the pre-MIG-108 legacy layout

`resolve_owner` is root-containment over {active} ∪ {federation}. An OWN library registered
at an EXTERNAL path (the legacy layout, still live until a universe accepts its unification
proposal — PJ-330's subject) is under neither, so the first version of
`registry_for_owner_of` would have turned three WORKING surfaces (360 / strata /
scan_library_links) into errors for legacy universes — where the active vocabulary is
genuinely correct. Fixed: the own-set membership check (STRICT `try_load_libraries` +
`owning_own_library_name_in`) runs **only in the `Err` branch** — run first, the prefix
resolver would hand a NESTED linked universe to `universe_notes` (whose path IS the active
root), the exact trap documented on `require_own_library_in`. The fallback's answer is a
fact from the own registry, not a guess. (Composition of three individually-tested
functions; the wrapper itself needs an AppHandle and has no unit test — same status as
`resolve_owner`'s own wrapper, noted honestly here.)

### Live behavioral deltas (everything else is byte-identical)

1. **`detect_tensions` refuses non-own libraries.** Before: a Linked Universe's library passed
   the gate and the detection ran over the ACTIVE database's rows under that library's NAME —
   an empty fake-healthy report, or the active universe's namesake library on a collision.
   After: refusal; the frontend's existing catch renders the Health tab's built-in
   **"Analysis unavailable"** state (TensionPanel.svelte:150-153 — "Never an eternal
   Loading…"). `note_tension_status` was checked and left alone: it queries by PATH, so a
   linked note honestly reads `indexed: false`.
2. **Strict errors naming the universe** where a Linked Universe's vocabulary cannot be read
   or an owner cannot be resolved (360 / strata / scan_library_links / the federated cache
   readers). Previously: silent active-vocabulary fallback. Frontend surfaces all catch and
   degrade (backlinks → empty + console; enrichment loop → per-library skip; 360 → error
   state). Recorded as the through-line's fourth-cousin: "I could not read that" ≠ "use the
   active universe's answer".
3. **Dead code removed:** cache.rs `read_links` single-schema wrapper (zero callers).

### Files touched

cache.rs · inspector360.rs · libraries.rs · link_types.rs · links_backfill.rs ·
name_fold_backfill.rs · search.rs · sight.rs · sky_backfill.rs · strata.rs · tension.rs
(11 files, +377/−103). No frontend change (all error paths already handled).

### Gates

- **Rust: 1535 / 0 / 19 ignored — four consecutive runs** (LL-049: distribution, not a run).
  +3 over the handover's 1532 = the three new resolution tests.
- **Binary identity (LL-050):** `constellation_lib-48488a16ffaa15f4.exe`, 86,622,208 bytes,
  mtime 2026-08-22 10:54:42.
- **Census red→green:** went red on touch (its job); map updated only after each site's
  answer was written. Post-B4 census: cache.rs ABSENT (was 3) · inspector360 ABSENT (was 2) ·
  strata ABSENT (was 1) · libraries 1 (was 2 — the rename rewriter stays for B5/B6) ·
  link_types 6 (+2: the two resolvers' active arms) · links_backfill 8 (+1, B1) ·
  search 16 (net +1: generators −2, comments −2, DDL/restore +5) · sky_backfill 1 · sight 1 ·
  tension 1.
- **Line-endings note:** the Python edit scripts normalized the touched files to LF in the
  working tree; `core.autocrlf=true` normalizes on commit, so the committed content is
  unchanged in convention. Diff verified content-only (+377/−103).
- **vitest:** full run under concurrent inspection-workflow load: 996/997 with
  `tests/sight-v6/perf.test.ts` + `tradition-perf.test.ts` timing budgets over — the known
  **PJ-172** flake shape verbatim ("flakes under concurrent build load, passes in
  isolation, no Sight file in the diff" — v1.92 gates note). Re-run in isolation: **31/31**.
  This diff contains zero TS. Full clean re-run after the workflow completes, below.
- **Per-build diff-scoped safety-inspection — pass 1 (`wf_abe7b5f1-f79`, 9 agents, 13.5 min):
  4 CONFIRMED findings, ALL FIXED before commit (WA#6):**

  1. **MED · index-divergence · cache.rs (B4-NEW, mine)** — `registries_for_schemas` failed
     wholesale, so ONE corrupt/held Linked-Universe `link-types.json` blanked Backlinks +
     Outgoing for EVERY note — the active universe's own rows included — and the frontend's
     catch swallowed the named refusal into empty arrays. **Fixed: per-schema skip-with-notice**
     (the `attach_all` skip_unavailable model): only the unreadable universe's rows are
     withheld, the eprintln names it, and NO fallback to the active vocabulary. Surfacing
     federation-vocabulary health in UI filed to the ledger (PJ-326..331 warning-badge family).
  2. **MED · toctou · sky_backfill.rs:217 (pre-existing, exposed by the hunt)** — a thread
     scheduled for unstamped universe A could pin freshly-activated stamped universe B
     (`set_active_universe` flips `active_path` 18 lines before the generation bump) and run
     the unconditional stratum/maturity WIPE over a completed universe; a mid-walk switch then
     abandoned the NULL band with the stamp intact — rank 0 in the Reviewer, permanent, silent.
     **Fixed three ways:** `run()` re-checks `is_needed` on the PINNED connection before the
     wipe; the wipe transaction atomically CLEARS the 'sky' stamp (interrupted walks now
     re-arm — `is_needed`'s documented cursor-row contract finally real); `is_needed`
     distinguishes no-row (run) from read error (fail closed — a transient error must never
     authorize a destructive pass).
  3. **MED · fire-and-forget · sky_backfill.rs:88 (pre-existing)** — a switch to an unstamped
     universe during another's drain window hit the RUNNING CAS and was dropped for the whole
     session (search.rs's `maybe_schedule` call sits behind the db_ready fast path; sky had no
     second re-arm site, unlike review_backfill). **Fixed:** the exiting thread re-invokes
     `maybe_schedule` after releasing the slot, gated on clean exit (an `Err` keeps today's
     no-retry — no hot-loop).
  4. **LOW · index-divergence · name_fold_backfill.rs:85 (B4-adjacent)** — the strict
     vocabulary read gated Phase A (the vocabulary-INDEPENDENT name_lower fold), so a
     corrupt vocab file silently blocked the false-orphan repair every boot. **Fixed:** the
     strict read moved to just before Phase B, its first consumer; root still pinned once at
     the top; a Phase-B refusal leaves the module unstamped for retry.

  Post-fix gates: forced fresh relink (LL-050 — a transient one-error compile was observed and
  not shrugged off: exe deleted, relinked at 11:20:36, 86,622,208 bytes), then **1535/0 ×2**.

- **Pass 2 (`wf_d891da29-5d0`, 3 agents) over the four re-touched files: 2 CONFIRMED, both
  LOW, both FIXED:**

  1. **LOW · index-divergence · cache.rs** — my skip-annotation claimed skipped rows
     "reappear the moment the file is readable again"; TRUE for the per-note readers
     (re-resolve every call), FALSE for the MEMOIZED `cache_full_links` consumer:
     `ensureFullLinks` latches `linksReady=true` on any Ok, so a skip during its single
     fetch froze an incomplete edge list for the session (legacy `perNoteLinkQueries=false`
     path only — non-default, no UI toggle). **Fixed with a per-consumer split:** the skip
     form is documented per-read-consumers-only; `cache_full_links` uses a new STRICT
     variant — an error keeps the `!linksReady` 3s retry armed, so the common transient
     hold self-heals on the next attempt.
  2. **LOW · comment-integrity · sky_backfill.rs** — my fresh B4 comment asserted
     present-tense "B1 threads this... the pair cannot drift at all" while B1 has not
     landed — the third false-guarantee comment in that exact file (the pattern PJ-332b's
     own corrections name). Runtime reachability verified nil by the refuter (the
     interlocks: `load_active` after the generation bump; the pinned `is_needed` re-check;
     the RUNNING CAS). **Fixed:** reworded future-tense, naming the interlocks and warning
     against weakening them on the strength of the comment.

  Post-fix: **1535/0 ×2** again, binary 11:36:10.

- **Pass 3 (`wf_0572aed2-900`, 2 agents) over cache.rs alone: 1 CONFIRMED LOW, fixed:**
  the skip form's only notice was stderr — invisible in a Windows release build — so a note
  living in a broken Linked Universe shows badge N vs 0 panel rows with zero surfaced
  notice. The verifier judged the skip model itself correct ("deliberate and per-consumer
  correct") and the residual "non-blocking." **Fixed in-pass:** the notice also goes to
  `diagnostics.log` via `diag_log`, deduped once per (schema, message) per session (it
  fires per note-switch while a file is broken — unbounded repeats would flood the log).
  The panel-visible degradation hint is an IPC-shape + UI decision → **filed to the ledger
  (PJ, v1.93)**, not bolted on mid-step; one caveat from the verifier recorded honestly:
  its candidate's claim that a "Stage-1 blocked card" independently surfaces vocabulary
  corruption is NOT verifiable in shipped code and was not counted.

  **Inspection cycle total: 3 passes, 7 confirmed findings (3 MED, 4 LOW), 7 fixed before
  commit. Zero parked.**

  Final gates: **1535/0 ×2** (ten green suite runs today), release
  `target/release/constellation.exe` rebuilt **2026-08-22 11:50:18**, 95,670,272 bytes,
  containing the final code (new Rust strings verified in the exe; frontend `build/`
  rebuilt 10:56 — no frontend change in this diff).

### The test pipeline run (tutorial-auditor → ui-inspector → panel)

- **tutorial-auditor** built the Stage-1 tutorial (14 source-verified claims, incl. live
  universe data + the June-installed-binary trap addressed in Stage 0).
- **ui-inspector: REJECTED ×1** — the draft called the left-dock dashboard icon
  "heart-shaped"; it is the two-lobe **brain** glyph (verified twice: the SVG paths and the
  command palette's 🧠 for the same toggle). Corrected; **re-inspection APPROVED** (24 claims
  verified total, including binary-contains-fix, discrimination reachability, and that both
  live universes' vocabularies add the identical custom type `inspires`).
- **Panel (3 lenses + synthesis, `wf_7ca92ddc-f35`): SEND_WITH_AMENDMENTS — 10 edits**, the
  material ones:
  - The draft **misdescribed the old behavior**: for a linked note the OLD binary shows the
    inactive "Add more links… 0 / 50" state, not "nothing wrong found." Both lenses traced
    both binaries; the old-binary signature is now Step 3's named failure mode, making a
    stale-exe report unambiguous.
  - **PJ-321 protection turned into evidence**: the Universe Manager may LIST "Eisa
    Universe" (the phantom nine-row list) — the tutorial now forbids clicking the row
    (different code path) and uses only Open Existing Universe. The registry evidence file
    + the two corroborating artifacts were **snapshotted hash-verified BEFORE the run**
    (now durable at `lab/reports/pj321-evidence-snapshot-2026-08-22/`, mtimes preserved:
    universes.json c20f9694…, boot-perf a50f93df…, diagnostics f669532c…). Two free PJ-321
    observations attach to this test: (1) I re-stat the registry right after the Boss's
    Step 1; (2) new Step 4 asks which universe opens on his next normal launch.
  - Step 3 now pre-explains that the message's "retry" hint never succeeds on a linked
    note (permanent by design) — **PJ to file: the wording is misleading for a permanent
    condition**; plus a 5-second Backlinks glance as a canary for the strict full-links
    gate.
  - Honesty additions: the three background failure-path repairs are named in "not
    covered" with their dormancy VERIFIED against his live data (all universes sky-stamped,
    zero NULL-stratum, all vocabulary files parse); the register now carries the
    filed-not-built panel-hint item and the PJ-172 vitest flake.
- **Panel declined to rule** (the Boss's calls): Stage 2 scope; the retry-wording PJ's
  priority; whether the PJ-321 observations change its priority; and whether
  `note_tension_status`'s missing own-universe gate (tension.rs — same concern its sibling
  now refuses, masked today by branch order) is B5/B6 scope or a new PJ — **to resolve at
  the v1.93 ledger reconciliation**.

**Obligations standing before commit:** re-stat `universes.json` after the Boss's Step 1 and
record the outcome here; file at v1.93: the panel-hint PJ, the retry-wording PJ, the
note_tension_status gate item.

## §B4 — BOSS-VALIDATED (2026-08-22, all four steps passed)

- **Step 1** pass (Open Existing → Eisa Universe).
- **Step 2** baseline: own note "Pending and Follows-up" (Constellation PKM) → "No tensions
  detected for this note — it's well-connected." (screenshot on file).
- **Step 3** headline: linked note "Acropolis" (Eisa Cognitive Knowledge → History) →
  **"Analysis unavailable — switch tabs and back to retry."** exactly. Backlinks canary:
  39 linked mentions / 0 unlinked / 21 outgoing with typed pills — normal.
- **Step 4**: switched back to كون عيسى; next normal launch opened كون عيسى.

### PJ-321 — the two free observations, recorded WITHOUT diagnosis (the STOP stands)

1. **The registry file received ZERO writes across the whole test.** Post-run:
   `%APPDATA%/world.uconstellation.app/universes.json` is byte-identical to the pre-run
   snapshot — 277 bytes, mtime 2026-08-07 09:56:01, sha256 `c20f9694…` — after a sequence
   that per `universe.rs` (`open_existing_universe` → `save_registry`; `set_active_universe`
   ×2) should have written it at least three times, on the NEW 11:50 binary.
2. **Step 4's answer is explained by the stale file itself**: its `active_id`
   (`universe_189df371a599143c8d18` = كون عيسى) is what the next launch opened — no evidence
   the final switch persisted anywhere this file can show.
3. Bounded fact-set gathered (observations only): `registry_path()` =
   `app_data_dir()/universes.json`; tauri identifier = `world.uconstellation.app` (so the
   frozen file IS the path the code computes); the only other `universes.json` on disk is
   the OLD identifier `com.notesconstellation.app` (stale since 2026-03-14, active_id
   differs); **the same directory IS actively written** — `write-journal.jsonl` mtime
   2026-08-20, `app-prefs.json` 2026-08-10 — while `universes.json` alone stays frozen.
   Pre-run snapshot preserved at `lab/reports/pj321-evidence-snapshot-2026-08-22/` (three
   files, hash-manifested). **No mechanism is asserted. PJ-321's STOP holds; this is the
   evidence bundle its future instrumented reproduction starts from.**

### NEW TOP-PRINCIPAL RULING — "Universe of Universes" (Boss-dictated at the B4 pass)

> "…it was based on the idea of linking between notes, from the main universe and any linked
> universe(s), NOT to keep each universe (main and linked) in its own cocoon. That's why
> Constellation should be a Universe of Universes in every aspect and concept, and you should
> make it happen. Write this statement as a rule, and you shall develop the app based on this
> concept." — Eisa, 2026-08-22

Written into CLAUDE.md as a top-principal in this commit (full scoping there: generalizes
the 2026-07-05 ONE-universe resolver ruling to every aspect and concept; honest refusal is a
floor with an obligation, federated function is the target; write sovereignty / MIG-108 /
the move-refusal are boundaries, not cocoons, and stand). Ledger v1.93 files the owed
federated Knowledge Health accordingly. **B4's refusal stays** — it replaced a wrong answer
with an honest state — and the federated form is now on the books as the end-state.

---

## §B5 — the rename cascade takes the OWNER's vocabulary (fences UP)

> Working on: **MIG-111 Stage B5** — `rewrite_wikilinks_in_text` + `update_links_recursive`
> take the OWNER's registry, threaded from the top of `update_links_on_rename`. **Both
> federation fences untouched** — B6 is a separate, later commit.

**Concept:** a rename's "is `[[refutes::Old]]` a typed link or a target name?" decision must
be made with the vocabulary of the universe that owns the files being rewritten.

**What the investigation established before the edit:**
- The one ambient read sat inside `rewrite_candidates`' rayon closure — read **per file**,
  so a vocabulary change mid-cascade could split one rename's rewrites across two
  vocabularies (the LL-047 window, per candidate), and it was always the ACTIVE vocabulary.
- Both branches (index-seek and walk) funnel into `rewrite_candidates` — one threading
  point covers the cascade.
- Unlike B4's clause-invariant sites, the rewriter's `reg.is_known(head)` genuinely differs
  between vocabularies — but under the fences a linked cascade rewrites ZERO files in both
  branches (the seek's foreign skip; the walk's foreign boundary), and an active-universe
  cascade resolves owner = active. So B5 is behavior-preserving for every reachable path
  today, while making the OWNER's grammar the one B6 will act on.

**The change:** `update_links_on_rename` resolves ONE registry per cascade
(`registry_for_owner_of(&app, &library_path)`, `?`-propagated — a linked universe's corrupt
vocabulary is an ERROR naming it, never a rewrite pass under the wrong grammar);
`update_links_recursive` and `rewrite_candidates` take `reg`; the per-file global read is
gone. Census: libraries.rs ABSENT (was 1), answer recorded in the map.

**Verification:**
- `cascade_rewrites_a_child_only_typed_link_under_the_childs_vocabulary_and_not_the_parents`
  — the plan's clause as a discrimination pair, entered through `update_links_recursive`
  (LL-048), child registry built by the PRODUCTION writer + strict reader: child registry ⇒
  `[[refutes::Old]]` → `[[refutes::New]]`; seeds ⇒ the typed form untouched (a target name)
  while plain `[[Old]]` still rewrites. **MUTATION-PROVED:** restoring the global read
  inside the closure turns it red (0.03s fail), reverted.
- `cascade_still_refuses_to_cross_a_foreign_root_after_b5` — the fence holds: a matching
  foreign-root file stays byte-identical while its sibling rewrites.
- Suite **1537/0 ×3** (two new tests), binary 2026-08-22 17:39:11.
- **Per-build inspection `wf_54802e43-408`: 1 pass, 0 CONFIRMED findings** — the empty
  result was verified against the journal, not assumed (the hunter's trace covers the
  frontend caller `handleRenameComplete` — which always passes the ACTIVE universe root
  and surfaces a cascade `Err` via `templateActionError` + a journal marker, answering the
  half-complete-rename question — both fences, the rayon shared-reference use, and the
  census consistency; its verdict: "strictly narrows a real prior hazard (the LL-047
  mid-cascade vocabulary-split window). Clean.").

### The B5 test pipeline

- Release exe rebuilt with B5: **17:49:03**, 95,669,248 bytes, verified newer than both
  changed sources (Rust-only diff; the 11:02 frontend bundle is current).
- **tutorial-auditor** built the regression tutorial (12 verified claims — B5 is
  behavior-preserving, so the test is an ordinary two-probe rename: plain + typed referrer
  links must BOTH heal, `supports::` surviving intact being the exact decision the changed
  code makes).
- **ui-inspector: REJECTED ×2, then APPROVED (18 + 5 + 1 claims across three passes):**
  1. The draft's "the universe row is at the very top of the sidebar" is FALSE against the
     Boss's LIVE sidebar — his كون عيسى renders "Five Acts" and "Bases" sections above the
     universe row (verified on his disk), and those headers carry no context menu.
  2. My correction then claimed right-clicking those headers "shows no menu at all" —
     ALSO false: without an `oncontextmenu` suppressor the webview's NATIVE menu appears
     (the app's own comment on the sibling handler documents this). The gate caught the
     fixer; the inspector's source-verified wording was adopted verbatim.
  Deep checks that passed: the reopen path cannot show stale pre-cascade content (the
  write-ahead snapshot logic re-verified), Ctrl+A genuinely needed (the rename input is
  not pre-selected), no template auto-fills a new note (his live settings read).
- **Panel `wf_129dc803-24c` (3 lenses + synthesis): SEND_WITH_AMENDMENTS — 9 edits.** The
  material ones:
  1. **BLOCKING register correction — my "unreachable for active-universe renames" claim
     was FALSE.** `resolve_owner` runs `resolve_child_universe_roots_recursive_strict`
     BEFORE the active arm of `registry_for_owner_of` returns, and كون عيسى declares one
     linked child (Two Universe UNIVERSE) — so **every rename the Boss makes today runs
     the strict federation-readability check**. An unreadable linked-universe folder now
     makes the cascade REFUSE (note renamed, links stale until retried) where pre-B5 it
     shrugged and continued. Deliberate (refuse-never-guess, Boss ruling 2026-08-17), but
     a NEW shape the Boss hears about in the register, not discovers. The
     retry/repair-affordance question is his (panel declined to rule).
  2. **The probe upgraded from `supports::` to the Boss's own custom `inspires::`**
     (live-verified in كون عيسى's link-types.json): a builtin exists in every possible
     registry including seeds-fallback, so only a custom word can expose a wrong-list bug
     on screen — the test gains real discriminating power against vocabulary-resolution
     failures.
  3. **The cascade freeze overlay pre-framed**: the tree shows the new name BEFORE the
     cascade dispatches, and `CascadeFreezeOverlay` ("Updating links…") blocks the ⋯
     button until cascade + tab-reload settle — a stable false failure is impossible, but
     an unexplained spinner would read as a glitch.
  4. **Delete honesty**: `trashDestination:'local'` live — Step 6 CREATES `.trash` in his
     everyday universe root (no in-app viewer) and open probe tabs self-close; both now
     stated, with the optional Explorer cleanup. Register also discloses the permanent
     MIG-104 delete-archive envelope, alongside the panel's line-by-line verification
     that the in-app purge (aliases from the rename included) is complete in one
     transaction.
  5. **Venue RULED**: كون عيسى is the right place (only registered universe; probes
     self-created/self-deleted; blast radius = exactly the one probe referrer). One lens
     had read the WRONG universe's settings; the panel re-checked the right one — the
     refutation layer working on the panel itself.
  Declined to rule (the Boss's): the build verdict itself; the stale-links-shape
  follow-up; the .trash/archive-viewer product question; the tension-gate scope (already
  slated for v1.93+ reconciliation).

## §B5 — BOSS-VALIDATED (2026-08-22, all six steps passed)

- Steps 1–4 pass (probe creation under كون عيسى, both link forms authored via the
  suggestion list — screenshot shows the live-preview "Inspires" pill on the typed link —
  rename executed).
- **Step 5 (the check): PASS** — Source mode shows exactly `[[Rename Probe Renamed]]` and
  `[[inspires::Rename Probe Renamed]]`; both lines the new title, the Boss's own custom
  `inspires::` intact before it (screenshot on file). The cascade's decisions demonstrably
  carry his vocabulary through the owner-resolved path.
- Step 6 cleanup done (probes deleted; `.trash` behavior disclosed in the tutorial).
- Incidental observation (recorded, not acted on): the probes were created under the
  `Templates` folder of كون عيسى (the New-Note Location default from the row he
  right-clicked) — immaterial to the test; the cascade heals referrers wherever they live.

### B5 close bookkeeping

- **Help/manual (SO#2): no user-facing delta** — B5 is behavior-preserving; the
  stale-links refuse shape gets documented when PJ-341 is ruled (documenting an error
  path the ruling may change would be churn).
- Ledger **v1.94** (B5 closed; PJ-341 + PJ-342 filed; ► = B6) and orientation **v4.5**
  in this commit. MoCh-2026-08-22-1430 written.

### Open follow-ups filed into this stage's later steps (not new PJs)

- B1: thread the backfills' `recompute_*` functions (links_backfill / sky_backfill annotations
  point at it).
- B2: DDL generation takes the vocabulary explicitly (the `ddl_reg` sites).
- B5/B6: the rename rewriter (libraries.rs census entry, count 1) — vocabulary FIRST, fences
  down in a LATER commit, never one.
- Phase 1.3: the index tail (`maintain_incoming_after_save`'s caller) for routed saves.
- Known limit carried (not new): `require_own_library`'s foreign set is best-effort for a
  linked universe with no library at its root — already recorded on the function, filed as
  migration scope.

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

---

## §B6 — the fences come down for the owner's own universe (build phase)

> Working on: **MIG-111 Stage B6** — the SEEK-branch foreign skip + the walk's foreign
> exclusion, lowered for a rename whose owner is a Linked Universe; the rename DB tail
> routed through `WriteScope` (the FIRST production wiring of A3). A rename still refuses
> a THIRD universe (Phase 3 / R23).

**Territory mapped first** (4-agent parallel sweep, evidence-only): rename is OFFERED on
linked notes through SIX frontend entry points, all funneling into `handleRenameComplete`;
`rename_item` ADMITS linked paths today — the file renames + frontmatter rewrites in the
foreign universe while the 'rename' alias lands in the ACTIVE DB (a live wrong-DB write)
and the owner's index goes stale; the cascade walked the wrong tree (frontend hardcoded
the active root); WriteScope had zero production callers; `maintain_sky_after_save` was
private; the cascade reindex tail carried an adoption landmine (foreign path fallback →
active DB).

**The design (one owner decision):** `owner_scope_of` → (routed_root, registry).
- Active owner: walk root = active universe root (backend-derived now), fence = the
  pre-B6 federated foreign set — byte-identical behavior.
- Routed owner: walk root = the OWNER's root; fence = the owner's own children via the
  STRICT resolver (unreadable manifest ⇒ the cascade REFUSES — a third universe that
  cannot be enumerated cannot be fenced); seek disabled (the active index cannot hold the
  owner's referrers — walk = disk truth); rewritten referrers reindex via
  `WriteScope::routed_at` on the owner's connection with the owner's vocabulary +
  maintenance pair (empty-old full recompute, the A8 shape). Refusals are DISK-FIRST:
  files are the source of truth, already healed; a scope refusal skips only derived
  state, loudly, and NEVER falls back to the active DB.
- `rename_item_db_tail`: routed arm runs the 11-table `migrate_note_db_paths` + the
  rename-alias INSERT + reindex on the OWNER's connection — the wrong-DB alias write is
  gone; legacy external own libraries (for_note Err + own-registry claim) keep the ACTIVE
  tail (the B4-class regression, caught by self-review pre-inspection).

**The B6 fence test caught a REAL production defect red-first:** the strict child resolver
canonicalizes (Windows verbatim prefix), and the fence normalization missed the prefix —
the grandchild's file was rewritten in the test's first run (the §0.4 / LL-048 class).
Fixed by sharing `owner::strip_verbatim` (now pub(crate)) — one identity per path, one
function, no second copy to drift.

**Verification:** `b6_routed_cascade_heals_the_owner_and_fences_the_grandchild` (real
universe.json child declaration, production resolver; owner referrer heals typed+plain,
grandchild byte-identical) and `b6_routed_rename_tail_lands_in_the_owners_db_under_the_owners_vocabulary`
(production init_db child fixture; routed_at through the A4 refusals; asserted against an
INDEPENDENTLY-opened child DB: the `refutes`-typed link renamed and classified under the
child's vocabulary, the rename alias + note_meta migration in the CHILD DB). Suite
**1539/0 ×2** (two new tests), binary 19:20:42. Inspection `wf_b605fa1c-9e8` launched;
verdict below.

### B6 inspection pass 1 (`wf_b605fa1c-9e8`): 2 CONFIRMED, both FIXED

1. **HIGH · content-corruption** — my routed fence enumerated only the OWNER's declared
   children, so a third universe physically nested under the owner but declared elsewhere
   in the federation (the active app knows it; the owner's manifest doesn't) would be
   walked, rewritten under the owner's vocabulary, and planted into the owner's index —
   silent cross-universe file corruption, unreachable pre-B6. **Fixed with a better
   rule:** `routed_cascade_fence` (pure, unit-tested with the exact C-inside-B topology)
   — every universe root the ACTIVE app's STRICT recursive federation knows, minus the
   owner and its ancestors. Subsumes the owner's children; closes undeclared nesting;
   unreadable federation ⇒ the cascade refuses.
2. **MED · content-loss** — the legacy pre-MIG-108 arm walked only the active root,
   silently skipping referrers inside the renamed note's own EXTERNAL library. **Fixed:**
   `walk_roots` — the legacy arm walks both the active root and the external library;
   net + walk iterate the vector.

Post-fix: **1540/0 ×2** (the fence-formula test added), binary 19:40:46. **Pass 2
(`wf_97663781-6a6`)** launched over libraries.rs (the only file changed since pass 1) —
verifying the fixes themselves (segment-boundary fencing, dual-root bookkeeping, the
strict federation read against the Boss's live layouts, active-arm byte-identity).

### B6 inspection pass 2 (`wf_97663781-6a6`): 3 CONFIRMED, all FIXED

1. **HIGH · concurrency-race (largely PRE-EXISTING, exposed by the sweep)** — the four
   detached ACTIVE-arm DB tails (folder rename, move, single rename, cascade reindex)
   re-acquire `SearchState.db` per batch with NO generation check; a universe switch
   mid-tail writes the departing universe's notes into the NEW universe's database —
   silent, durable (reconcile skips-not-heals such rows), the PJ-332 class on the rename
   family. **Fixed:** federation-generation switch guards on all four (capture at start,
   re-check per batch/note, STOP loudly; the departing universe's own reconcile heals the
   remainder). The ROUTED tails need no guard — pinned connections by construction.
2. **LOW · swallowed-write-error** — the 'rename' alias stamp was `let _`-dropped in both
   arms; a failed stamp is a PERMANENT resolution gap (the old title exists nowhere on
   disk afterwards; reindex preserves-but-never-creates rename aliases). **Fixed:** the
   active arm logs the failure; the routed arm propagates it as an Err.
3. **LOW · index-divergence** — `move_item_db_tail`'s attribution-miss arm skipped the
   reindex with zero log (its sibling folder tail logs the identical arm). **Fixed:**
   logged + counted in the summary.

Post-fix: **1540/0 ×3** (one more transient link error answered with the LL-050
forced-relink guard; binary 19:57:18). **Pass 3 (`wf_716eba67-ce4`)** launched over the
guard code itself (TOCTOU residue, interleavings, early-STOP consistency).

### B6 inspection pass 3 (`wf_716eba67-ce4`): 3 CONFIRMED, all FIXED

1. **HIGH · index-divergence (Whole-Ecosystem asymmetry — mine)** — I routed the NOTE
   rename tail and left the FOLDER tail active-only: a folder rename inside a Linked
   Universe (reachable — the folder guard refuses only folders CONTAINING a linked
   library root) stranded the owner's whole subtree at dead paths in ITS database, the
   exact hazard the note tail closed. **Fixed:** `rename_folder_db_tail` routes by owner
   exactly like the note tail — per-descendant migrate + reindex + maintenance on the
   pinned routed connection, owner-library attribution, legacy fallback, loud refusals.
   (The MOVE tail is exempt by construction: moves inside a linked universe are already
   refused end-to-end by PJ-235's `require_own_library`.)
2. **MED · toctou** — my loop guards checked the generation BEFORE parking on the writer
   lock; a switch completing while parked still slipped one write into the new universe's
   DB. **Fixed as the CLASS (Solve-the-Class):** `reindex_single_note` itself now
   captures the generation before parking and refuses after waking if it moved — every
   caller (the four tails AND the save path) is guarded at the one place the park
   happens; the loop-level guards remain as cheap early stops.
3. **LOW · index-divergence** — my legacy dual-root fix covered only the CALLER's
   external library; a SIBLING external own library's referrers were still silently
   skipped on the walk path — the same defect one door over. **Fixed:** the legacy arm
   walks the active root + EVERY own library at an external path (deduped).

Post-fix: **1540/0 ×2**, binary 20:14:32. **Pass 4 (`wf_c2a11058-edb`)** launched over
the folder tail's new routed write path + the reindex guard (old-path reconstruction
forms, interleavings, multi-root double-visitation, happy-path byte-identity).

### B6 inspection pass 4 (`wf_c2a11058-edb`): 2 CONFIRMED, both FIXED

1. **MED · index-divergence (mine)** — my routed tails resolved owner-library attribution
   through the LENIENT `own_libraries_for_root` ([] on read error, no log) and fell back
   to the literal "universe_notes" — which matches NO real owner library (the owner's
   root library carries the universe's DISPLAY name): durable misattribution stamped into
   the owner's note_meta, silent, unhealed by mtime-gated reconcile. **Fixed:** new
   `owner_libs_strict` (absent = a fact; unreadable/corrupt = Err); all three routed
   tails skip the attribution-dependent REINDEX half loudly on Err/miss (migrate + alias,
   which are attribution-free, still run) — never a fabricated name.
2. **LOW · index-divergence (mine, the third recurrence of one shape)** — the routed walk
   covered only the owner's ROOT; a routed owner's own EXTERNAL (pre-MIG-108) libraries
   had their referrers silently skipped — the identical skip pass 3 fixed on the legacy
   arm. **Fixed with the shared-helper law:** `cascade_walk_roots(base, libs)` now serves
   BOTH arms, so the external-library coverage cannot drift between them again.

Post-fix: **1540/0 ×2 force-relinked** (binary 20:36:17; one more transient link error
answered with the LL-050 guard, no error in the fresh run). **Pass 5 (`wf_67d6dd69-8aa`)**
launched over the deltas — loop-until-dry: the cycle closes on a DRY round, not a
converging one.

### B6 inspection pass 5 (`wf_67d6dd69-8aa`): 2 CONFIRMED, both FIXED — both in my guards

1. **HIGH · concurrency-race (mine)** — the rename tail's STOPPED branch logged and then
   FELL THROUGH to Step 6, whose fresh prefix-resolve against the NEW universe's registry
   would adopt the departing child's note into the parent's index as a lexically-own row
   that reconcile keeps forever (worse than foreign_rows — it never even counts). The
   guard detected the hazard and then did the hazardous thing anyway. **Fixed:** STOPPED
   now returns, ending the whole tail.
2. **MED · toctou (mine)** — the move tail's migrate guard was VACUOUS: captured and
   re-checked back-to-back BEFORE the park; a switch completing while parked handed the
   pre-DELETEs the wrong universe's connection. The correct after-lock pattern sat twenty
   lines away in my own diff. **Fixed** in the move tail AND the folder tail's migrate
   batches (same shape, Whole-Ecosystem): the generation check lives INSIDE the acquired
   lock everywhere now.

Post-fix: **1540/0 ×2 force-relinked** (binary 20:55:16). **Pass 6 (`wf_e915d722-e6e`)**
launched as the DRY check over the three-guard delta.

### B6 inspection pass 6 (`wf_e915d722-e6e`): 1 CONFIRMED LOW, FIXED

**LOW · content-corruption (a pre-existing degraded mode B6 WIDENED)** — with
libraries.json transiently unreadable at rename time, the fence collapses to {} (the
documented one-boot fallback), and B6's backend-derived walk root gave EVERY rename
whole-active-root reach — so a linked universe nested under the active root could have
its notes' [[OldTitle]] wikilinks rewritten by the active universe's rename. **Fixed with
defense-in-depth the B6 machinery makes cheap:** the ACTIVE arm's fence now also carries
the UNIVERSE-ROOT fence (the same `routed_cascade_fence` formula over the active
federation) — derived from universe.json, which fails independently of libraries.json,
so one locked file no longer strips both boundaries. Best-effort on the active arm
(refusing a plain own rename on a bad manifest would be a regression); the routed arm
keeps its strict refusal.

Post-fix: **1540/0 ×2 force-relinked** (binary 21:12:45). **Pass 7 (`wf_de8820fe-0a6`)**
launched over the one additive change — the dry candidate at 15 findings / 15 fixed.

### B6 inspection pass 7 (`wf_de8820fe-0a6`): the B6 delta is DRY — the cycle closes

Pass 7 confirmed NOTHING against the pass-6 fence change. Its single finding is an
UNRELATED, LATENT, pre-existing LOW spotted in passing: `save_clipboard_image` named
attachments at 1-second resolution and wrote with bare `File::create` — a silent
same-second overwrite (permanent image loss, both embeds rendering the survivor),
unreachable today (zero frontend call sites; the paste wiring is future work) but armed
the moment it lands. **Fixed in-pass** (WA#6): exclusive `create_new` (the OS's atomic
O_EXCL — no exists-check race) with a de-collision suffix — the binary sibling of the
`gate_create_exclusive` discipline every text create path in the file follows.

**THE B6 INSPECTION CYCLE: 7 passes, 16 CONFIRMED findings, 16 FIXED, zero parked.**
Severity trajectory: HIGH/MED → HIGH/LOW/LOW → HIGH/MED/LOW → MED/LOW → HIGH/MED → LOW
→ (unrelated LOW; delta dry). Six of the sixteen were defects in MY OWN fixes for
earlier findings — the loop caught the fixer every time it needed to. Final gates:
**1540/0 ×2 force-relinked**, binary 21:26:00.

### The B6 test pipeline — the auditor's pre-flight caught a stale-bundle binary

The tutorial-auditor's pre-flight (its own initiative) found `build/` dated 11:02 while
B6's frontend fix landed at 19:10 — my 21:29 release rebuild ran only
`cargo build --release`, re-embedding the eight-hour-stale bundle
(`feedback_frontend_build_before_cargo`, the exact recorded trap). **The 21:29 exe
contained only the RUST half of B6**; a test on it would have silently exercised the old
dispatch and produced a false failure indistinguishable from a real regression. My
earlier statement that "the fix is genuinely in the binary" was true of the Rust half
only — corrected here. Rebuilt properly: `npm run build` (build/ 21:50:40) →
`cargo build --release` (exe **21:53:30**, 95,668,736 bytes, newer than the fresh
bundle). The draft (staged: Stage 1 headline + Stage 2 boundary/regression/cleanup, 26
verified claims incl. the load-bearing cross-universe alias-resolution chain for Stage
2A) is with the ui-inspector.

### The B6 pipeline verdicts

- **ui-inspector: REJECTED ×1 then APPROVED** — the draft called the sidebar
  Linked-Universe icon "multi-colored"; it is a single-indigo planet-with-orbit outline
  (the multi-colored icon is the status-bar button). 31 claims verified incl. the
  load-bearing Stage-2A cross-universe alias-resolution chain, live reads of all three
  universes, and the fresh two-half binary (build/ 21:50:40, exe 21:53:30).
- **Panel (`wf_97f19b36-611`): SEND_WITH_AMENDMENTS — 8 edits.** Material ones: the
  tutorial undersold the Boss's time (TRUE switch count six, not three; two switches
  stall 10–20 s per the live boot-perf records — now stated); a machine-side cleanup
  option added (WA#1 — none of the cleanup genuinely needs a human; watcher-index-
  freshness reconciles); Stage-1 gets a self-contained stop-here ending; the second
  sidebar group (كون عيسى) pre-framed; NOT-COVERED gains the switch-race guards and the
  owed full federated Editor-Surface Gate run; the REGISTER gains the admission that an
  early B6 version WIDENED the degraded-registry mode before the cycle closed it, and
  PJ-342 restored to the open list. Panel corrections of my premises: كون عيسى holds SIX
  notes (not ~9,600 — the corpora are ECK 8,031 / EU 2,731); venue RULED approved (ECK is
  the only real federated pair; the tutorial itself is the ask, probes self-created).
  Blast radius walked at source: real notes are READ ONLY. PJ-321: the standing bundle
  suffices; machine-side post-run re-stat ordered. Housekeeping ordered and DONE: the
  mid-switch error string's garbled spacing collapsed (suite 1540/0 after; the path is
  untouched by this test so the 21:53 exe stays valid); this rejection/approval cycle
  recorded here per the panel's order.
- Panel declined to rule (the Boss's): cleanup path choice; PJ-341; PJ-342; the
  misleading cascade-refusal wording (fix-now-or-file — panel recommends filing at PCS);
  scheduling of the owed full federated Gate run; the cross-universe text-healing
  phase's priority.

## §B6 — BOSS-VALIDATED (2026-08-23, all stages passed)

- **Stage 1 PASS** (screenshot on file): a rename performed through the Eisa Universe view
  on a note inside Eisa Cognitive Knowledge healed BOTH referrer lines —
  `[[Zelkovine Target Renamed]]` and `[[inspires::Zelkovine Target Renamed]]`, the custom
  typed head intact. The Boss heard the disk walk ("PC hard disk thrashing") — the routed
  cascade's deliberate disk-truth scan of ~8,000 notes; no banner appeared (no open note
  pane in the way — "may appear" per the tutorial).
- **Stage 2 Part A PASS** — the parent-universe referrer's text stayed old (the honest
  Phase-3 boundary) AND clicking it opened the RENAMED note: the cross-universe old-name
  resolution through the owner-stamped alias works live, same session.
- **Stage 2 Part B PASS** — the own-universe regression rename healed both forms.
- **Cleanup:** the Boss took the recommended path (deleted the Florzeth pair, closed);
  I deleted the three Zelkovine probe files from their verified paths (0 remain outside
  `.trash`).
- **PJ-321 third corroboration:** the registry file is byte-identical AGAIN
  (277 B · mtime 2026-08-07 · sha256 c20f9694…) after the full B6 run's ~6 switches on
  the 21:53 binary — the frozen-file observation now stands across the B4 and B6 test
  rounds. Recorded; the STOP holds.
- **Boss question answered in-chat:** why tests use "Open Existing Universe" over the
  Universe Manager list — test discipline (the list's ids are PJ-321's unexplained side;
  the folder route pins the universe by path); his daily list habit is fine and is itself
  evidence the app reads a reliable store that is not the watched file.

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

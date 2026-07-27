# Constellation Pending Jobs

**Version 1.53 | 2026-07-27**

> **What changed in v1.53** (**MIG-105 Architect LOCKED + Stage 0 shipped + MIG-104 Slices 0–6 shipped and Boss-validated — the earned layer now survives losing the index. TWO new laws (LL-035, LL-036). Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — MIG-104 Slice 7** (`earned.snapshot.jsonl` + the 2 MB-threshold compactor; bounds the fold permanently). Then **Slice 8 + 8b** — *the Boss's time machine substrate*: **archive-before-purge** (must go BEFORE the `note_meta` DELETE — the CASCADE fires there, proven by `tests_stage0_delete_order_defect`) **+ the note BODY** (Boss decision #6). Then Slice 9 (continuous note-history mirror, Boss decision #4) · 10 (cascade pre-delete archive) · 11 (restore rejoin) · **12 = PJ-164/C8** (child tables → `ON UPDATE CASCADE`; unblocked by R2=archive-first) · 13 (gated `index_note` overlay) · 14 (adjacent defects, Q3=fix all four) · 15 (docs ×15). **Then MIG-105 Phase 2 Plan** (Core Organizer + loose content → "Eisa Test"). MIG-106 (LINK authoring surface) opens after Slice 6 validates — **it now has** → PJ-169.
>
> **CLOSED THIS JOB:**
> - **PJ-145 / MIG-105 Architect** — design LOCKED by Boss rulings (Core Organizer; no icon; Universe MOC; closed kinds; TOTAL root rule; loose content → Eisa Test). Docs: Architect v2 + AD verdict v2 + Inspectors verdict v2.
> - **PJ-149** (11-table path cascade + delete purge; its idempotence test caught a latent landmine that deleted freshly-migrated rows) · **PJ-151** (the 1,591 silent relocate failures — root cause found: FK `ON UPDATE NO ACTION`; fixed with `defer_foreign_keys`) · **PJ-153** (cid_cn emitters + healer; 3 real notes injected) · **PJ-154** (the invisible note; 8/8 stranded pairs healed) · **PJ-155** (second-screen mislabelling) · **PJ-156** (last three first-match resolvers → one shared helper) · **PJ-157** (vitest allow-list → globs) · **PJ-161** (NOT an app defect — MSIX AppContainer shadow; protocol recorded).
> - **MIG-104 Slices 0–6** — baseline + harness · `.constellation` watcher predicate (fixed a LIVE stall) · determinism (**RED-proven: 6 fake history rows → 0**) + dot-segment guard · the appender/reader/contract · the 6 write hooks · the seed · **the restore — Boss-validated on live data: 34/34 written, 34 exact matches, 0 mismatches, a retired link returned still retired.**
>
> ### ★★ THE TWO LAWS THIS JOB PRODUCED
> - **LL-035** — *"Feature X is off" must be PROVEN BY RUNNING IT; grepping for its enabler proves nothing. And never log a success you did not verify.* Born of: FKs asserted inert by grep while enforced by default (1,591 silent failures), plus TWO false-success lines shipped in one build ("14 relocated" on a boot where nothing moved).
> - **LL-036** — *When you clone a proven pattern, clone its PRECONDITIONS; the comment explaining why it is safe is part of the code. And build fixtures at the PRODUCTION caller's privilege level.* Born of: the restore's bare connection lacking the FTS tokenizer (100% write loss, silent) while 52 tests passed because every fixture used `init_db`, which registers it.
>
> **NEWLY FILED:**
> - **PJ-167** *(optional)* — per-library earned stores instead of one per Universe (18 of 20 libraries live outside the Universe root, so earned data would travel with a detached library). Deferred by Boss decision #3.
> - **PJ-168** — 236 orphan `note_links` rows whose `source_path` is absent from `note_meta` (235 under the retired `E:\Cognitive Knowledge` root). **[UNVERIFIED]** why reconcile never purged them.
> - **PJ-169** — **MIG-106: the LINK authoring surface** (links as openable, annotatable, searchable objects — CLAUDE.md's "first-class knowledge objects"). Boss decision #7: sibling migration, opens now that Slice 6 has validated.
> - **PJ-170** — `note_links.target_name` is stored lowercased, so the Outgoing/Backlinks panels show `earth`/`france` instead of the notes' real titles.
> - **PJ-171** — **CI/pre-commit guard**: `release.yml` runs ZERO tests. The vitest + cargo suites should run automatically (Plan ruling R4, non-blocking).
> - **PJ-172** — **PJ-132 is now a gate, not a nuisance.** The Sight perf tests assert wall-clock budgets (`≤16 ms`) inside a *parallel* runner, so they measure the machine. Since PJ-157 made the suite glob-driven, a green suite gates every slice — and a load-sensitive test makes that gate lie **in both directions**. Move them to a serial lane or make them load-invariant.
>
> **STILL OPEN:** **MIG-104** (Slices 7–15) · **PJ-145 / MIG-105** (Plan → Build → Audit; PR-F1–F6, R3, R7, the stored-name token) · **PJ-164** (C8, now MIG-104 Slice 12) · PJ-150 (its *conclusion* was wrong — the work is PJ-164) · PJ-152 (`UniverseMeta` catch-all — `custom_stages` still destroyed by rename/attach/detach) · PJ-158 (RTL chevrons · "vault" ×10 locales · cursive captions · English-only const) · PJ-159 (939 MB orphan DB + uncapped logs) · PJ-160 (appearance/i18n surface never inspected) · PJ-162 (`.base` YAML parsed as JSON) · PJ-163 (review-pulse RMW wipe) · PJ-166 (PJ-124 struck a 3rd time; 23 inspection verifiers died on a session limit — unverified, not cleared) · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-137 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.52 | 2026-07-26**

> *(See `Constellation Pending Jobs v1.52.md` — the trail is durable, never overwritten.)*

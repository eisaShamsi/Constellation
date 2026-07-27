# Constellation Pending Jobs

**Version 1.52 | 2026-07-26**

> **What changed in v1.52** (**MIG-105 STAGE 0 SHIPPED + Boss-validated — nine live defects fixed, and the 3-week-old silent-failure root cause found: foreign keys ARE enforced. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — (1) the R2 ruling + C8** (child-table rebuild to `ON UPDATE CASCADE`; upgraded from optional hardening to the proper structural fix by the FK discovery). **(2) MIG-104** (durable earned-link data on disk — Boss-ruled to precede the MIG-105 content move so link data becomes recoverable). **(3) MIG-105 Phase 2 Plan** (the Core Organizer restructure + the loose-content move to "Eisa Test"), presented for approval before any code. Then the PJ-140 backlog sequencing ruling · §1 use-side remainder · D4.
>
> **CLOSED THIS JOB (Boss-validated on the running binary @21:42; verified in the DB, not the log):**
> - **PJ-149** — `migrate_note_db_paths` **5 → 11 tables**; `reindex_delete_note` purges the same five; all statements de-silenced. Its new idempotence test caught a **latent landmine**: a repeat call with the same (old,new) deleted the freshly-migrated rows → note indexed NOWHERE. Guarded.
> - **PJ-151** — the 1,591 silent "relocate deferred" failures: **root cause found and fixed** (see the FK discovery below). Error capture + honest messages + bounded logging + forced boot summary.
> - **PJ-153** — `cid:` → `cid_cn:` at all three canonical emitters (value-preserving legacy migration) + inject-capable boot healer + template-scoped probe. Live result: `stale=17 templates=14 injected=3 still_empty=0`.
> - **PJ-154** — the invisible note. `Testing opened note.md` is indexed and searchable; **8/8** stranded pairs healed, 0 orphaned child rows, no duplicate rows created.
> - **PJ-155** — second-screen duplicate/mislabelled notes (`collect_library_notes` exclude-set).
> - **PJ-156** — the last three first-match resolvers → one shared `owning_own_library_name` (own libraries only, MIG-065 §J); callerless `get_library_mode` deleted; scoped_paths boundary; 2 frontend fixes (`addLinkToNote` wrong-library write, sidecar-trash boundary).
> - **PJ-157** — `vitest.config.ts` allow-list → globs + reason-commented excludes. A test file can never again silently not-run. (Audit: 0 masked regressions existed.)
> - **PJ-161** — **NOT an app defect.** The "registry disagreement" was an observer artifact: Claude sessions read a stale MSIX AppContainer shadow of `%APPDATA%`. Standing protocol recorded in the Architect doc §5.
> - **PJ-145 / MIG-105 Architect phase** — design LOCKED by Boss rulings, committed `adc7da42`.
>
> ### ★★ THE DISCOVERY — foreign keys ARE enforced (overturns the PJ-150 diagnosis)
> `PRAGMA foreign_keys = 1` on **every** production connection — rusqlite enables it by default, so no PRAGMA appears in our source and a grep for the enabler could never find it. The child tables are `ON UPDATE NO ACTION`, so SQLite **refuses** the parent `note_meta.path` UPDATE for any note owning a summary / state-history / suggestions row: **every rename, move and relocate of such a note had been silently failing for ~3 weeks (1,591 logged)**. Unreproducible in every replica replay because the replicas ran FK-off. Fixed by running the cascade under `PRAGMA defer_foreign_keys` inside a transaction (owned when the caller has none) — **red proven** (disabling the pragma reproduces the Boss's live log verbatim), green with it, and pinned permanently as `tests_pj150_fk_enforcement_reality`. → **LL-035**.
>
> **NEWLY FILED:**
> - **PJ-164** *(APP-KILLER class, gates on R2)* — **C8: rebuild `note_state_history` / `note_summaries` / `sources_suggestions` with `ON UPDATE CASCADE`** (+ a shared connection-pragma helper and a `foreign_key_check` gate that quarantines rows, never the DB file). The deferred-FK cascade makes moves *work*; the declaration is still wrong, and every future path-writer inherits the trap. **Boss ruling R2 required first:** on a genuine delete, does `note_state_history` die with the note (CASCADE as declared) or get archived first?
> - **PJ-165** *(hygiene)* — the two false-success bugs introduced and fixed in this same build are now **LL-035**; sweep for the same shape elsewhere: any function that delegates to a deliberately best-effort helper and then reports success without re-reading state (candidates: the other `let _ =`-style cascades and any `Ok(())` after a swallowed batch).
> - **PJ-166** *(process)* — **PJ-124 struck a THIRD time** (safety-inspection ignored `args.files`, ran whole-app). Additionally, **23 verifier agents died on a session limit** in `wf_75a9d203-e96` — their candidates are *unverified*, not cleared, and must be re-run before that register is treated as complete.
>
> **STILL OPEN:** **PJ-164** (C8 — R2) · **MIG-104** (3 questions; sequenced before the MIG-105 move) · **PJ-145 / MIG-105** (Plan → Build → Audit; PR-F1–F6 + R3/R7 + the stored-name token inside it) · **PJ-150** (superseded in substance by the discovery — its *conclusion* was wrong; the remaining work is PJ-164) · PJ-152 (`UniverseMeta` serde catch-all — `custom_stages` still destroyed by rename/attach/detach) · PJ-158 (RTL chevrons · "vault" in 10 locales · cursive captions · English-only const) · PJ-159 (939 MB orphan DB + uncapped logs) · PJ-160 (appearance/i18n/Style-Setter surface never inspected) · PJ-162 (`.base` YAML parsed as JSON) · PJ-163 (review-pulse RMW wipe) · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-137 · PJ-135 · PJ-132 · PJ-125–139.
>
> ---

**Version 1.51 | 2026-07-26**

> *(See `Constellation Pending Jobs v1.51.md` — the trail is durable, never overwritten.)*

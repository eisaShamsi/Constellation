# Constellation Pending Jobs

**Version 1.51 | 2026-07-26**

> **What changed in v1.51** (**MIG-105 Architect RUN + LOCKED by Boss rulings — the root library is re-founded as the Core Organizer (the Universe's head). Two adversarial cycles (69 agents) + Art Director & Team + Inspectors, both verdicts on file. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — MIG-105 Stage 0** (Boss-approved 2026-07-26 "Proceed"): fix the nine live defects P1–P9 (PJ-149…157 below) that silently lose data today and that the migration cannot be built on. Own build + Boss test. **Then** (2) MIG-104 remaining open questions → build (Boss-recommended sequence: MIG-104 lands before the MIG-105 move so earned-link data becomes recoverable). **Then** (3) MIG-105 Phase 2 Plan (presented for approval; stored-name token + R3 confirmed there). Then the PJ-140 backlog ruling · §1 use-side remainder · D4.
>
> **CLOSED THIS JOB:**
> - **PJ-145 / MIG-105 Architect phase — DONE, design LOCKED** (`docs/migrations/MIG-105-Architect-root-library-vs-flat-universe.md` v2; companion AD + Inspectors verdicts v2). Boss rulings 2026-07-26: the entity **remains** as **"Core Organizer"** (kind `core`, `_Core` leaf, stored like a library / never presented as one) · **no icon** · front page = the **Universe MOC** (one note, charter+map, offered never auto-created) · contents CLOSED to MOC + Templates + Five Acts · **TOTAL root rule** (no user notes at the root; home-Library asks; first note in a fresh Universe creates its first Library) · **all loose root content → "Eisa Test"** (58 indexed + 1 unindexed + canvas; Templates/Five Acts → `_Core`; `.trash` stays). Inspectors verdict: SAFE-AFTER-PREREQUISITES, amendment TOTAL + FEDERATION-STAGED (PR-F1–F6 tracked inside MIG-105). The migration itself (PJ-145) stays OPEN through Plan → Build → Audit.
>
> **NEWLY FILED (Stage 0 — all verified live this session):**
> - **PJ-149** *(APP-KILLER class)* — path cascade covers **5 of 11** path-bearing tables (`migrate_note_db_paths`, libraries.rs:1063): every rename/move orphans `note_body` / `note_summaries` / `note_state_history` / `sight_v3_layout` / `shape_history` / `sources_suggestions` rows (1,312 state-history rows exposed on the root notes alone).
> - **PJ-150** — `PRAGMA foreign_keys` enabled ONLY inside one `#[test]` (cece/history.rs:433); every declared `ON DELETE CASCADE` is inert in production. Impact review before enabling.
> - **PJ-151** — reconcile self-heal failing silently at scale: **1,577** "relocate deferred" lines, the `rusqlite::Error` discarded at reconcile.rs:192. Reproduce-First: instrument + diagnose from the live DB (suspect: stale destination-row PK conflict).
> - **PJ-152** — `UniverseMeta` has no serde catch-all (universe.rs:13-22): the Boss's `custom_stages` (zero readers/writers — verified) is destroyed by `rename_universe` / `add_child_universe` / `remove_child_universe`. `#[serde(flatten)]` + same guard on `LibraryInfo`.
> - **PJ-153** — 17 of the root entry's 126 notes have NULL/empty `cid_cn`; reconcile DELETES rather than relocates such rows on an external move. Stamp before any move.
> - **PJ-154** — `Testing opened note.md` on disk at the Universe root with NO index row (invisible to search). Diagnose the leak, fix the class.
> - **PJ-155** — second-screen mis-attribution: `collect_notes_names_recursive` (libraries.rs:5610) has no exclude set; its only caller (SecondScreenPage.svelte:427) stamps a fixed library name → duplicated + mislabelled notes live today.
> - **PJ-156** — three surviving first-match resolvers (`bases.rs:382`, `shape.rs:161`, `tasks.rs:529`) → `library_name_for_path`; delete callerless `get_library_mode` (libraries.rs:436, raw-prefix bug).
> - **PJ-157** — `vitest.config.ts test.include` is a curated allow-list (~45 named files): an unlisted test file silently never runs. Audit + fix before the MIG-105 harness.
>
> **NEWLY FILED (other):**
> - **PJ-158** *(i18n/RTL batch, found by the AD designs on the never-inspected surface)* — `.v-chev` never RTL-flips (only two flips exist in +layout.svelte, against CLAUDE.md's rule) · "vault" shipped in `universe.manager.*` in 10 of 15 locales · `.section-label` uppercase+tracking breaks Arabic/Farsi/Urdu cursive joining · `RECENT_CAPTURES_CONTENT` (system_notes.rs:47) ships English-only.
> - **PJ-159** — the 939 MB orphan `.constellation/Constellation SV Test.db` (mtime Apr 23) + unbounded `boot-perf.history.jsonl` (2.4 MB) / `diagnostics.log` (1.5 MB). Delete-after-code-reference-check + cap the logs. (Architect R8; deletion confirmed at Stage-0 Boss test.)
> - **PJ-160** — the **appearance / naming / i18n / Style-Setter surface has never been safety-inspected** (proven: the first designs to touch it found the PJ-158 batch). Run its own inspection before MIG-105 Phase 3.
> - **PJ-161** — the Roaming `universes.json` registry disagrees with the live universe (lists only كون عيسى while ECK is in active use). Instrument `save_registry` / read the write path — a step-1 blocker for the MIG-105 Plan.
> - **PJ-162** — `.base` files are YAML but `scan_bases_dir` parses them with `serde_json` (bases.rs:614): every base's in-file name is unread (falls back to filename); `LensDefinition` has no catch-all. One parser.
> - **PJ-163** — `review-pulse.json` read-modify-write with an empty-default fallback (review.rs:772-776 + the three RMW commands): one "✓ Reviewed" click after a transient read failure replaces the entire review history. Three-state load contract (Loaded | Absent | Unreadable) at the shared boundary (Inspectors PR-9).
>
> **STILL OPEN:** **PJ-145 / MIG-105** (Architect DONE; Stage 0 → MIG-104 → Plan → Build → Audit; PR-F1–F6 + R3/R7 + token format ruled inside it) · **MIG-104** (location + re-type settled; 3 questions left; **sequenced before the MIG-105 move**) · PJ-140 (~37; editor-lifecycle cluster = own migration) · PJ-142 · PJ-143 · PJ-144 · PJ-146 (help-dir 14-language sync) · PJ-147 (resolver consolidation — partially subsumed by PJ-156's Rust fixes; frontend half stands) · PJ-148 · PJ-137 · PJ-135 · PJ-124 (inspection ignores `args.files` — struck again 07-26) · PJ-132 · PJ-125/126/127/128/129/131/133/138/139.
>
> ---

**Version 1.50 | 2026-07-25**

> *(See `Constellation Pending Jobs v1.50.md` — the trail is durable, never overwritten.)*

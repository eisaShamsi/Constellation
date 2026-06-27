# Session Log — 2026-06-27

**Theme:** The Boss-prioritized 7-item backlog (order 1-2-5-4-7-6-3), Ultracode, one item at a time with staged Boss tests + full SO close-out between items. **All 7 items COMPLETE.** (Item 1 = MIG-087 §D landed 2026-06-26; items 2–7 landed today.)

---

## Items 2–6 (committed earlier today — summary)

| Item | Commit | What shipped |
|---|---|---|
| 2 — live properties count | `cbc9d4b6` | MIG-087 §E: one-way display-only `onLiveProps` observer (PropertyEditor → +layout), mirrors §C; all-noun Arabic singular "one" (واحد/واحدة) ×27. svelte-check 0, MIG-076 harness 28/28, Boss PASS. |
| 3 — PJ-065 concept | `31426042` | Structural parent/TOC link-type concept paper — Boss RATIFIED (build deferred to /migration). |
| 4 — PJ-067 concept | `2b30a9df` | Living Link Relationship Model v2 concept paper + R4 wargame (3 defence papers) → Boss ruled `problematizes/answers` = its own INTERROGATIVE FAMILY. RATIFIED, build nothing. |
| 5 — broader i18n | `15b0987c` | Triage-first → Boss ruled thorough sweep; 854 (key,locale) live-English strings translated ×13 locales. svelte-check 0, Boss PASS. |
| 6 — StyleSetter labels | `5b1e03f3` | Data-only i18n: 65 `styleSetter.labels` ×13 locales (the `sky_view` category was English in 13/14). svelte-check 0, Boss PASS. |

---

## Item 7 — PJ-066 connect-freeze (the final item) — SHIPPED + Boss-validated

**Function in hand:** the *Connect* action (right-sidebar Suggested Connections / Reviewer → add a typed link), on a link-dense note.

**Concept (the horse):** connecting two ideas must be instant — the act of forging a relationship is the core of Knowledge Formulation; a multi-second freeze on it is a betrayal of the "every keystroke instant" law applied to the most important gesture in the app.

### Reproduce-First — 5 instrumented measurement rounds
Heaviest note: **Ancient history** (533 outgoing links). Instrumentation = release-safe `diag()` sink → `<universe>/.constellation/diagnostics.log` + a slow-IPC detector (≥150 ms SYNC dispatch) + reindex phase timers + caller tags + affected-count split.

- The filed "~3 s residual" was wrong by an order of magnitude — **a single connect froze the app ~47 s.**
- **Refuted hypothesis A** (writer-lock-read) and **B** (frontend re-render) — both prior guesses.
- **Refuted the "reindex storm" hypothesis** (Boss-chosen path): the caller-tagged trace proved **one connect = exactly one reindex** (`prop_save`), no flush reindex, no `reloadTabsFromDisk` loop. The "7 reindexes" seen in an earlier round were the Boss clicking connect repeatedly *during* the freeze.
- **The decisive measurement:** `maintain_incoming TOTAL 31,362 ms` with `affected=1` — so the 31 s was *before* the UPDATE loop, inside the name-resolve. Confirmed in SQL on the live DB: `SELECT path FROM note_meta WHERE COALESCE(name_lower, LOWER(name)) = ?` → **SCAN note_meta = 21,915 ms** (the COALESCE defeats the index; the scan drags the wide `body_text` rows of a 2 GB DB).

### Four root causes → four fixes
1. **#1 — `constellation_embed_notes` async** (`embeddings.rs`). e5 ONNX inference (~32 s) was SYNC on the IPC thread. → `#[tauri::command(async)]`.
2. **#2 — skip redundant re-embed** (`store.ts`). Frontmatter-only connect re-embedded the whole (unchanged) body. → `saveTabContent(…, bodyUnchanged=true)` → `force:false`.
3. **#4 — the full-table scan** (`search.rs`). → new covering index `idx_note_name_lower(name_lower, path)` (in `init_db`, `IF NOT EXISTS`) + index-seeking `UNION` rewrite of `resolve_incoming_target_paths`. **21,915 ms → 0.06 ms**, verified identical over 200 real names.
4. **#3 — `scan_unlinked_mentions` off the writer lock** (`libraries.rs`). Was SYNC + took `db.lock()` → blocked the IPC thread for the reindex's full duration. → `#[tauri::command(async)]` + `with_read_conn` (read-only WAL reader).

### Verification (Boss + trace)
- Boss: **"Instant and no freeze at all."**
- Trace: `maintain_incoming` 31,362 ms → **154 ms**; **zero** `[PJ066-SLOW]` blocks; reindex (11 s cold / 2.5 s warm) entirely background.
- svelte-check **0 errors**; clean ship binary built (instrumentation fully reverted; `git grep PJ066` empty).

### Impact review (WA#4)
- Index: additive, `IF NOT EXISTS`, one-time ~21 s build on a large existing DB (same pattern as `idx_note_boot_snapshot`); free thereafter. The Boss's DB already has it (created during de-risk test). No write-path regression (one narrow index on per-note writes).
- Query rewrite: semantically identical (verified). 3 callers, all save/delete-path maintenance.
- `scan_unlinked_mentions` async + read_db: eventual-consistency correct for a suggestion panel; `with_read_conn` falls back to `db` pre-init.

**Commit:** _(this commit)_ — `PJ-066 §C5 (item 7): connect-freeze root-cause fix`.

---

## Standing Order
- Orientation bumped **v3.14 → v3.15** (Item 7 preamble) — lands in the same commit as the fix.
- MoCh: `docs/MoCh/MoCh-2026-06-27-*.md`.
- Handover: `docs/handover/Handover-2026-06-27.md`.
- Help/User Manual: no change — Item 7 is a performance fix; the Connect feature's usage is unchanged (just instant now).

**THE 7-ITEM BACKLOG IS COMPLETE.**

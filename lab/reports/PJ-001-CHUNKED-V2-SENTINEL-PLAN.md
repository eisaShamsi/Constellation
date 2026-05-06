# MIG-015 Plan v1 — Chunked v2 sentinel migration with progress UI

**Architect**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`
**Status**: Pending Eisa's "Plan approved." Cascade starts at §1A on go.
**MIG ID**: MIG-015 (closes PJ-001).

---

## Open questions resolved (per Eisa, 2026-05-06)

| # | Question                       | Resolution                                                    |
| - | ------------------------------ | ------------------------------------------------------------- |
| 1 | Chunk size                      | 100,000 rows (Claude's call; speed is the constraint)         |
| 2 | Status-bar placement            | **Center** — new `.sb-center` group                            |
| 3 | i18n locales                    | **All 15 locales now** — no PJ-014 deferral                    |
| 4 | MIG numbering                   | MIG-015                                                        |

---

## Phase rollout

| Phase | Scope                                                                      | Visible? | Boss test? |
| ----- | -------------------------------------------------------------------------- | -------- | ---------- |
| §1A   | Rust — `count_pending_v2_sentinel_rows` + `sentinel_bigram_rows_chunked`. Internal refactor; preserves existing behaviour. | No       | No         |
| §1B   | Rust — `init_db` defers v2 step. Async task spawn in `lib.rs::run`. Tauri event channel `migration:term_vocab_v2`. | No       | No         |
| §1C   | Frontend — `MigrationProgressStrip.svelte` + i18n (15 locales). Listener wired into `+layout.svelte` status-bar center. | **Yes**  | **Yes**    |
| §1D   | Three-agent audit (invariants / drift / migration-path).                    | No       | No         |

---

## §1A — Rust helpers (chunked + count)

### Goal
Add two pure Rust helpers next to the existing `sentinel_bigram_rows`. No behaviour change yet — `init_db` still calls the old function in this phase. The helpers exist for §1B to wire.

### Files touched
- `src-tauri/src/search.rs`

### Algorithm

```rust
/// Count term_vocab rows still requiring the v2 bigram sentinel.
/// Cheap query — exists only to populate the `total` field of the
/// initial progress event so the status-bar strip can show
/// "Migrating term index — 0 / N".
fn count_pending_v2_sentinel_rows(conn: &Connection) -> rusqlite::Result<u64> {
    conn.query_row(
        "SELECT COUNT(*) FROM term_vocab \
         WHERE bridge_concept_id IS NULL \
           AND term LIKE '%' || CHAR(31) || '%'",
        [],
        |row| row.get::<_, i64>(0).map(|n| n as u64),
    )
}

/// Chunked variant of `sentinel_bigram_rows`. Each chunk processes
/// up to `chunk_size` rows; loops until 0 rows affected. The
/// `on_progress` callback fires after each chunk with cumulative
/// completed-row count.
///
/// Crash-recoverable by construction — the WHERE clause excludes
/// already-sentinelled rows, so a re-entry from a prior partial
/// run picks up where it left off.
fn sentinel_bigram_rows_chunked<F>(
    conn: &Connection,
    chunk_size: u32,
    mut on_progress: F,
) -> rusqlite::Result<u64>
where
    F: FnMut(u64),
{
    let mut completed: u64 = 0;
    loop {
        let affected = conn.execute(
            "UPDATE term_vocab \
                SET bridge_concept_id = '-' \
              WHERE rowid IN ( \
                SELECT rowid FROM term_vocab \
                 WHERE bridge_concept_id IS NULL \
                   AND term LIKE '%' || CHAR(31) || '%' \
                 LIMIT ?1 \
              )",
            rusqlite::params![chunk_size],
        )? as u64;
        if affected == 0 { break; }
        completed += affected;
        on_progress(completed);
    }
    Ok(completed)
}
```

The subquery + `rowid IN (… LIMIT N)` shape is the SQLite-idiomatic way to chunk an UPDATE — `UPDATE … LIMIT N` is **not** supported by stock SQLite (only by builds with `SQLITE_ENABLE_UPDATE_DELETE_LIMIT`).

### Verification
1. `cargo build --release --lib` clean (warnings ≤ baseline).
2. `git diff src-tauri/src/lexicon/` empty (M11 zero-diff).
3. `init_db` still calls the existing `sentinel_bigram_rows` (no behaviour change yet).

### Commit message skeleton
```
MIG-015 §1A — chunked v2-sentinel helpers

Adds `count_pending_v2_sentinel_rows(conn)` and
`sentinel_bigram_rows_chunked(conn, chunk_size, on_progress)` next
to the existing single-statement `sentinel_bigram_rows`. No
behaviour change — `init_db` still uses the old bulk UPDATE.
The new helpers are wired into the boot path in §1B.

100k chunk-size choice: at SSD UPDATE throughput of 250-500k
rows/sec, a 100k chunk completes in 200-400ms. On a 5.7M-row
migration that's 50-100 progress updates total — visible motion
in the status bar, no event-channel flooding.

Crash-recoverable by construction: the WHERE clause excludes
already-sentinelled rows, so any re-entry resumes cleanly.
```

---

## §1B — `init_db` defers v2; async task wired

### Goal
`init_db` no longer runs the v2 UPDATE inline. Instead it detects v2 is pending, returns immediately, and the Tauri app's main entry spawns a one-shot async task that runs the chunked migration with progress emit. The schema-version stamp moves into the task's success path.

### Files touched
- `src-tauri/src/search.rs` — `init_db` change.
- `src-tauri/src/lib.rs` — task spawn after `app_handle.manage(...)` setup, before main window setup.

### Algorithm

**`init_db`** (search.rs ~line 1318):
```rust
// Step v1→v2 — the actual UPDATE no longer runs here. We only
// detect that it's pending. The Tauri app's main entry spawns a
// background task that runs the chunked migration with progress
// emit. The schema-version stamp moves into the task's success
// path so a crash mid-migration leaves us still at v1 and the
// next boot picks up where we left off.
if stored_term_vocab_bridge_version < 2 {
    diag_log(path, "[search] init_db: term_vocab v2 sentinel migration deferred to async task");
    // Stamp v0→v1 ALONE here. The v1→v2 stamp lands in the
    // async task on success. (Skip stamping if we're already at
    // v1 — the column-add helper above takes care of it.)
}
```

**Tauri app entry** (lib.rs):
```rust
// After init_db has set up the schema, spawn a one-shot task that
// runs the v2 sentinel migration if pending. This keeps the boot
// path non-blocking on pre-MIG-013 DBs (~5.7M rows would otherwise
// freeze the splash for 30-90 seconds).
let app_handle = app.handle().clone();
tauri::async_runtime::spawn_blocking(move || {
    if let Err(e) = run_term_vocab_v2_migration_if_pending(&app_handle) {
        eprintln!("[search] term_vocab v2 migration task failed: {}", e);
    }
});
```

The task's body (in search.rs):
```rust
pub fn run_term_vocab_v2_migration_if_pending(app: &tauri::AppHandle) -> Result<(), String> {
    let db_path = …; // resolve from active universe
    let conn = …;    // open connection
    let stored: i64 = conn.query_row("SELECT version FROM schema_versions WHERE module = 'term_vocab_bridge'", [], |r| r.get(0)).unwrap_or(0);
    if stored >= 2 { return Ok(()); } // already done

    let total = count_pending_v2_sentinel_rows(&conn).map_err(|e| e.to_string())?;
    if total == 0 {
        // No work to do — just stamp.
        conn.execute("INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('term_vocab_bridge', 2, strftime('%s','now'))", []).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let _ = app.emit("migration:term_vocab_v2", serde_json::json!({
        "phase": "start", "total": total
    }));

    let on_progress = |completed: u64| {
        let _ = app.emit("migration:term_vocab_v2", serde_json::json!({
            "phase": "progress", "completed": completed, "total": total
        }));
    };

    sentinel_bigram_rows_chunked(&conn, 100_000, on_progress).map_err(|e| e.to_string())?;

    conn.execute("INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('term_vocab_bridge', 2, strftime('%s','now'))", []).map_err(|e| e.to_string())?;
    diag_log(&db_path, &format!("[search] term_vocab v2 sentinel migration completed in async task; {} rows sentinelled", total));

    let _ = app.emit("migration:term_vocab_v2", serde_json::json!({
        "phase": "done", "total": total
    }));

    Ok(())
}
```

### Verification
1. `cargo build --release --lib` clean.
2. M11 zero-diff.
3. With `schema_versions.term_vocab_bridge = 1` manually set on a copy of a pre-MIG-013 DB and the binary launched: `diag_log` shows the deferred-task entry, init_db returns within milliseconds, and a Tauri event listener (smoke test via DevTools console) receives `migration:term_vocab_v2 { phase: 'start', … }`. (Frontend wiring lands in §1C.)

### Commit message skeleton
```
MIG-015 §1B — init_db defers v2; async task with emit

init_db no longer runs the v2 sentinel UPDATE inline. It only
detects pending, then returns. The Tauri app entry spawns a
one-shot async task (tauri::async_runtime::spawn_blocking) that
runs the chunked migration with Tauri event emit on each chunk.

Schema-version stamp moves into the task's success path —
crash-recoverable by construction: a kill mid-migration leaves
schema_versions.term_vocab_bridge at 1; the next boot picks up
where it left off via the WHERE clause.

Tauri event channel: `migration:term_vocab_v2` with phases
`start { total }`, `progress { completed, total }`, `done { total }`.
```

---

## §1C — Frontend `MigrationProgressStrip` + 15-locale i18n

### Goal
Show a thin status-bar strip in the center of the status bar: `Migrating term index — N / M`. Hide it 4 seconds after `done`. All 15 locales updated.

### Files touched
- New `src/lib/components/MigrationProgressStrip.svelte`.
- `src/routes/+layout.svelte` — add `<MigrationProgressStrip />` to a new `.sb-center` group between `.sb-left` and `.sb-right`. Existing `.status-bar` flexbox stays.
- `src/lib/i18n/{en,ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — add `migrationProgress.termVocabV2` translation strings. Two keys:
  - `migrationProgress.termVocabV2.label` = "Migrating term index"
  - `migrationProgress.termVocabV2.done` = "Term index migration complete"

### Algorithm

```svelte
<!-- MigrationProgressStrip.svelte -->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { t } from '$lib/i18n';

    interface ProgressEvent {
        phase: 'start' | 'progress' | 'done';
        total: number;
        completed?: number;
    }

    let visible = $state(false);
    let total = $state(0);
    let completed = $state(0);
    let phase = $state<'start' | 'progress' | 'done' | null>(null);
    let hideTimer: ReturnType<typeof setTimeout> | null = null;
    let unlisten: UnlistenFn | null = null;

    const fmt = (n: number) => n.toLocaleString();

    onMount(async () => {
        unlisten = await listen<ProgressEvent>('migration:term_vocab_v2', (ev) => {
            const p = ev.payload;
            phase = p.phase;
            total = p.total;
            if (p.phase === 'progress') completed = p.completed ?? completed;
            if (p.phase === 'start') { visible = true; completed = 0; }
            if (p.phase === 'done') {
                completed = total;
                if (hideTimer) clearTimeout(hideTimer);
                hideTimer = setTimeout(() => { visible = false; phase = null; }, 4000);
            }
        });
    });
    onDestroy(() => { unlisten?.(); if (hideTimer) clearTimeout(hideTimer); });
</script>

{#if visible}
    <div class="mig-progress-strip" role="status" aria-live="polite">
        <span class="mig-progress-label">
            {phase === 'done' ? $t('migrationProgress.termVocabV2.done') : $t('migrationProgress.termVocabV2.label')}
        </span>
        {#if phase !== 'done'}
            <span class="mig-progress-counts">— {fmt(completed)} / {fmt(total)}</span>
        {/if}
    </div>
{/if}

<style>
    .mig-progress-strip {
        display: flex; align-items: center; gap: 6px;
        font-size: 0.75rem; color: var(--text-muted);
    }
    .mig-progress-counts {
        font-variant-numeric: tabular-nums;  /* steady width as the number climbs */
    }
</style>
```

In `+layout.svelte` status-bar:
```svelte
<div class="status-bar">
    <div class="sb-left"> … </div>
    <div class="sb-center"><MigrationProgressStrip /></div>
    <div class="sb-right"> … </div>
</div>
```

CSS for the new center group (matches existing flex layout):
```css
.sb-center { flex: 1; display: flex; justify-content: center; }
```

### i18n strings (15 locales)

| Locale | `label` | `done` |
| --- | --- | --- |
| en | Migrating term index | Term index migration complete |
| ar | جارٍ ترحيل فهرس المصطلحات | اكتمل ترحيل فهرس المصطلحات |
| de | Begriffsindex wird migriert | Migration des Begriffsindex abgeschlossen |
| es | Migrando índice de términos | Migración del índice de términos completada |
| fa | در حال انتقال نمایه واژه‌ها | انتقال نمایه واژه‌ها تکمیل شد |
| fr | Migration de l'index des termes | Migration de l'index des termes terminée |
| he | מעביר אינדקס מונחים | העברת אינדקס מונחים הושלמה |
| hi | टर्म इंडेक्स माइग्रेट हो रहा है | टर्म इंडेक्स माइग्रेशन पूरा हुआ |
| ja | 用語インデックスを移行中 | 用語インデックスの移行が完了しました |
| ko | 용어 색인 마이그레이션 중 | 용어 색인 마이그레이션 완료 |
| pt | Migrando índice de termos | Migração do índice de termos concluída |
| ru | Миграция индекса терминов | Миграция индекса терминов завершена |
| tr | Terim dizini taşınıyor | Terim dizini taşıma tamamlandı |
| ur | اصطلاحات کی فہرست منتقل ہو رہی ہے | اصطلاحات کی فہرست کی منتقلی مکمل ہو گئی |
| zh | 正在迁移术语索引 | 术语索引迁移完成 |

(Translations are best-effort literal renderings of the en source. If any locale wants a more idiomatic phrasing later, it can be revised in a separate doc-only commit.)

### Boss-test (Stage 1)

> **What this is**: when you install Constellation on a library that hasn't been "marked" yet (a pre-MIG-013 backup, for example), the app needs to add a tiny housekeeping mark to ~5.7 million rows in its term index. Until today, this happened in one bulk operation that froze the splash screen for 30–90 seconds with no feedback. Now the marking runs **after** the app paints — you see the full window immediately, and a thin status-bar strip in the center shows progress: `Migrating term index — N / M`. The strip disappears 4 seconds after the marking completes.
>
> **Stage 0 — install**: close Constellation, run the new MSI, reopen. Build timestamp will be in the Boss-test message.
>
> **Step 1 — simulate a pending migration**: this is the tricky part. Your library's already migrated, so the marking won't fire on a normal boot. To force it, I'll provide a small SQL command you can run in DB Browser for SQLite to roll the schema version back to 1 (no data loss; the column-add stays at v1). Then close + reopen Constellation.
>
> **Expected**:
> - The app paints fully and immediately. Sidebar, tabs, editor — everything is interactive within a couple of seconds.
> - The status-bar strip in the center shows `Migrating term index — 0 / 5,729,974` (your actual total).
> - The completed count climbs steadily — about 50–100 visible jumps over the full migration.
> - You can edit notes, search, switch tabs while it runs (the database lock window per chunk is a fraction of a second).
> - When complete, the strip changes to `Term index migration complete` and disappears 4 seconds later.
>
> **Step 2 — verify it doesn't re-fire**: close + reopen Constellation. The strip does NOT appear (schema version is now 2 again).
>
> **Step 3 — crash recovery**: roll schema back to 1, restart Constellation, force-kill the app while the strip is mid-migration (e.g. completed = 1,500,000). Reopen. Expected: the strip reappears, but starting from `Migrating term index — 0 / 4,229,974` (the remaining ~4.2M, NOT the original 5.7M). Already-marked rows aren't re-touched.
>
> **If you see this instead**:
> - **Frozen splash on Step 1** → §1B's deferred-task wiring isn't firing. Tell me; check Constellation's developer console for the boot trace.
> - **Status-bar strip in left or right group instead of center** → CSS issue; I'll move it.
> - **Completed count goes backward or stalls at zero** → progress event payload is wrong; tell me what the strip actually shows.
> - **Strip never disappears after `done`** → the 4-second hide timer broke; tell me.

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. M11 zero-diff.
4. Boss test passed.

### Commit message skeleton
```
MIG-015 §1C — MigrationProgressStrip + 15-locale i18n

New status-bar component listens for `migration:term_vocab_v2`
events and renders a thin center-group strip:
  "Migrating term index — N / M"
On `done`, switches to "Term index migration complete" and hides
4 seconds later. Hidden when no migration is in flight.

Plumbing: new .sb-center flex group between .sb-left and .sb-right.

i18n: 15 locales (en + ar + 13 others) — Eisa's call to ship all
upfront rather than queue via PJ-014.

Boss test passed Stage 1.
```

---

## §1D — Three-agent audit

Three parallel agents:

1. **Invariant agent** — verifies: (a) the migration is one-shot per DB (re-running yields zero rows affected); (b) crash-recoverable (manual interrupt + restart resumes from the WHERE clause); (c) end state matches the bulk UPDATE byte-for-byte; (d) chunk size is constant (no off-by-one / overflow drift); (e) M11 zero-diff; (f) no synchronous `invoke()` on the keystroke path; (g) Tauri event channel emits exactly the three phases.

2. **Drift agent** — checks no other surface assumes the v2 migration runs synchronously. Likely places: `ctse::search::ctse_search_terms_by_concept` reads `term_vocab.bridge_concept_id` filters but only on rows with non-NULL values, so mid-migration mixed state is fine. Verify that path. Search index reindex paths also touch `term_vocab` — verify they don't conflict with the chunked UPDATE's `rowid IN (… LIMIT N)` lock window.

3. **Migration-path agent** — checks: (a) fresh DB with empty term_vocab — no event ever emits, schema stamps to v2 immediately; (b) DB already at v2 — no event emits, no work done; (c) DB at v1 with 0 pending rows (column added but no bigram rows yet — possible on edge-case mid-MIG-013 backups) — emits `start { total: 0 }` then `done { total: 0 }`, strip flashes briefly; (d) crash mid-migration — resume works; (e) DB at v1 with rows interleaved between users sentinelled and not (corrupt half-state somehow) — verify the WHERE clause's `bridge_concept_id IS NULL` filter handles cleanly.

P0/P1 fixed before close. P2/P3 logged as memory follow-ups.

### Commit message skeleton
```
MIG-015 §1D — three-agent audit closes MIG-015

Audit report at lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md.

Findings: [filled in after audit run]

PJ-001 → SHIPPED. Pending Jobs v1.x bumped.
Constellation Development Laws stay as-is (no new laws produced).
Orientation v1.45 bumped inline per SO #6.
```

---

## Closing the cascade

After §1D:
- `Constellation Pending Jobs v1.x.md` — PJ-001 status: SHIPPED via MIG-015.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — phase commits + state-of-standing record.
- `docs/Constellation Orientation & Onboarding v1.45.md` — bumped inline.
- MoCh next-block file written.

---

**Awaiting Eisa's "Plan approved" before §1A.**

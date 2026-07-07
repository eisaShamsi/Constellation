export const meta = {
  name: 'safety-inspection',
  description: 'Constellation Safety & Integrity Audit — adversarial hunt for silent app-killer bugs (whole-app or diff-scoped)',
  whenToUse: 'PER-BUILD: pass args.files (the changed files) for a diff-scoped hunt before a commit. PER-CYCLE: no args for the full whole-app sweep. Hunts the silent app-killer classes; every candidate is adversarially refuted before it is confirmed. See docs/Constellation-Safety-Audit-CHARTER.md.',
  phases: [{ title: 'Hunt' }, { title: 'Verify' }],
}

// ── Invocation ────────────────────────────────────────────────────────────────
// Workflow({ name: 'safety-inspection' })                       → whole-app cycle sweep
// Workflow({ name: 'safety-inspection', args: { files:[...] } }) → per-build diff sweep
const files = (args && Array.isArray(args.files)) ? args.files.filter(Boolean) : []
const DIFF_MODE = files.length > 0

const HUNT = {
  type: 'object', additionalProperties: false,
  required: ['scope', 'candidates'],
  properties: {
    scope: { type: 'string' },
    candidates: { type: 'array', items: {
      type: 'object', additionalProperties: false,
      required: ['klass', 'file', 'line', 'summary', 'silent_failure_scenario', 'severity'],
      properties: {
        klass: { type: 'string', enum: [
          'silent-data-loss','fire-and-forget','swallowed-write-error','false-success',
          'content-corruption','content-loss','cross-note-bleed','cross-window-clobber',
          'index-divergence','init-ordering','reactivity-loop','concurrency-race','toctou',
          'freeze-hang','resource-leak','deadlock','other' ] },
        file: { type: 'string' }, line: { type: 'integer' },
        summary: { type: 'string' }, silent_failure_scenario: { type: 'string' },
        severity: { type: 'string', enum: ['APP-KILLER','HIGH','MED','LOW'] },
      },
    } },
  },
}
const VERDICT = {
  type: 'object', additionalProperties: false,
  required: ['verdict', 'reason', 'final_severity'],
  properties: {
    verdict: { type: 'string', enum: ['CONFIRMED','REFUTED','NEEDS-REPRO'] },
    reason: { type: 'string' },
    final_severity: { type: 'string', enum: ['APP-KILLER','HIGH','MED','LOW','NONE'] },
  },
}

// The app-killer taxonomy — what an inspection HUNTS. An "app-killer" = a SILENT
// failure that damages the user's knowledge or the index with NO surfaced error.
const TAXONOMY = [
  'CONSTELLATION SAFETY & INTEGRITY INSPECTION. App: Tauri v2 + Rust + SvelteKit/Svelte 5 (runes), SQLite/rusqlite WAL, CodeMirror 6. Source: src-tauri/src (Rust) + src (Svelte).',
  'AN APP-KILLER = a SILENT failure that loses/corrupts a source-of-truth (a note .md on disk; the search index note_meta/notes_fts/sky_links/note_links/note_aliases/review_schedule/note_embeddings; a persisted JSON like universe.json/libraries.json/collections/settings) OR the on-screen note content, with NO error the user or a test would notice. Recoverability is irrelevant; SILENCE is the defining trait.',
  'HUNT THESE CLASSES (ground every candidate in file:line with a CONCRETE failure scenario — exact state/timing → silent damage):',
  '1. SILENT DATA-LOSS / DURABILITY — a source-of-truth write via a fire-and-forget spawn/spawn_blocking (JoinHandle dropped), no durability/retry; lost on conn-None / lock-contention / app-close (the canonical MIG-098 rename bug + LL-033).',
  '2. SWALLOWED WRITE-ERROR — `let _ = conn.execute(...)` / `.ok()` / `unwrap_or` on a fallible DB or FS write that matters; `.catch(()=>{})` on an invoke/write in TS.',
  '3. FALSE-SUCCESS — returning Ok(())/Ok(x) or resolving a promise after CONDITIONALLY SKIPPING the work (e.g. reindex_single_note returning Ok on a None conn); marking a model "saved" BEFORE the write resolves.',
  '4. CONTENT-INTEGRITY (BUG-012/015/019/023 + LL-014) — a note acquiring ANOTHER note content on screen/disk, or losing its own: editor lifecycle ({#key} teardown, tab/mode switch, zombie flush), save-composition, the rename wikilink cascade, cross-window edits, two tabs/models for one note, nav-before-flush.',
  '5. INDEX↔DISK DIVERGENCE (Rule 8) — a derived surface silently out of sync with the .md source; a write path missing its reindex/cascade; a derived note_meta column with no trigger and no reconcile.',
  '6. INIT / ORDERING — a write that can run before the DB/state is ready (conn None) and silently no-op.',
  '7. FREEZE / HANG — an AWAITED Tauri invoke that parks on an unbounded writer-lock wait (the §B2-4 class) or does heavy sync work on the IPC dispatch thread (missing `(async)`); invoke() on the CM6 keystroke hot path.',
  '8. CONCURRENCY / REACTIVITY / LEAKS — TOCTOU / racing writers / a stale-result guard that still applies; a $effect that reads+writes the same state or watches a prop it mutates (echo loop); listen()/timer/EditorView/addEventListener not cleaned in onDestroy; unbounded growing caches.',
  'A candidate MUST be a real, reachable, SILENT source-of-truth loss/divergence/corruption (or a real freeze/leak). Speculative, cosmetic, or already-guarded (line-change guard, seq token, reconcile, onDestroy cleanup present) do NOT qualify.',
].join('\n')

// The whole-app subsystem map (the per-cycle sweep). Mirrors Waves 1–3.
const WHOLE_APP = [
  { key: 'rename-move-delete-gate', hint: 'libraries.rs (rename/move/delete/create + rename_item_db_tail), write_gate.rs (gate_write/gate_rmw_rename, the journal + identity/staleness checks). Detached-tail durability, swallowed writes, folder-cascade gaps, the gate ever letting wrong/stale content land on a file.' },
  { key: 'note-save-index', hint: 'store.ts writeNote/saveNoteSession + NoteEditor/NotePane/FocusPane save paths + write_note/index_note/the note_meta triggers. Does every write path pair with a reindex? Any write that leaves note_meta/FTS stale, or marks-saved-before-write, or has no write-ahead net?' },
  { key: 'notemodel-ownership', hint: 'noteModel.ts (open/repath/externalChange/markSaved/compose, savedVersion/reloadVersion, the write-ahead buffer) + store openNoteTab/loadTabHistoryEntry. Two tabs/models per note; nav-before-flush edit loss; version races; freshness-gating gaps.' },
  { key: 'editor-lifecycle', hint: 'NoteEditor/NotePane/FocusPane + the {#key} teardown in +layout (tab/mode switch, onDestroy flush). Zombie/destroyed editor flushing stale/empty doc over the wrong/renamed file; Focus↔NotePane round-trip integrity.' },
  { key: 'rename-cascade-integrity', hint: '+layout handleRenameComplete + the wikilink cascade (flushAllTabsInLibrary/updateLinksOnRename/reloadTabsFromDisk/cascadeFreeze/markCascading). Cascade reload stomping an open tab; the freeze/gate airtight across every tab + the second screen.' },
  { key: 'cross-window-integrity', hint: 'SecondScreenPage + secondScreen.ts + the +layout screen listeners + externalChangeNoteModel. Main AND second screen writing the same note; adopt-without-remount → stale editor clobbers; missing broadcastNoteSaved; Display-not-Domain violations.' },
  { key: 'derived-index-triggers', hint: 'search.rs (index_note, the ai/ad/au triggers, incoming/outgoing/word_count/link_types maintenance) + sky + review.rs + connectivity/tension + reconcile.rs. Derived note_meta columns maintained only at index time with no reconcile; a link change in another note leaving B stale.' },
  { key: 'frontmatter-property-writes', hint: 'store.ts parseFrontmatter/reconstructFrontmatter + PropertyEditor + addTagToNote/stage/sources/aliases writes. YAML round-trip that DROPS keys (block scalars/nested maps) or CORRUPTS values (quote escaping); a splice that races the body; cid_cn/title/aliases preservation on every mutation.' },
  { key: 'persisted-json-state', hint: 'universe.rs/libraries.rs/collections.rs/review.rs — saves of universe/libraries/collections/bookmarks/review_schedule/settings/property-types. Non-atomic truncate-then-write (crash window); swallowed write/rename errors; fire-and-forget saves.' },
  { key: 'cece-sources-derived', hint: 'cece/ + sources/ + classifier + review_rehearse.rs. Best-effort mutations of note frontmatter / note_meta columns — swallowed errors, non-durable, racing writes that lose user data.' },
  { key: 'frontend-write-callers', hint: 'store.ts + +layout + editor components — invoke() WRITES (rename/move/delete/save/tag/frontmatter/stage) with `.catch(()=>{})` or not-awaited so a failure is swallowed and the UI proceeds as if it succeeded.' },
  { key: 'boot-init-ordering', hint: 'ensure_search_db_ready/init_db + the SearchState db Mutex<Option<Connection>> + federation attach. User-triggerable WRITES that can run while state.db is None and silently no-op; commands that `if let Some(conn)` and skip vs error.' },
  { key: 'reactivity-concurrency', hint: 'All Svelte components: $effect that writes a $state it reads, or watches a prop it mutates (echo loop), or fires without a line-change guard. Rust: TOCTOU / racing writers on a file/row; single-flight lock gaps; stale-result guards that still apply.' },
  { key: 'freeze-and-leaks', hint: 'Rust: any #[tauri::command] awaited by the UI that acquires the writer lock then does heavy work, or a sync command doing a full scan on the dispatch thread (missing (async)). Svelte: invoke() on the CM6 keystroke path; listen()/timer/EditorView/rAF/addEventListener not cleaned in onDestroy; unbounded caches.' },
]

// Diff mode: chunk the changed files so each agent audits a focused set + the
// data-flow it touches. ~4 files per agent, capped.
function chunkFiles(list) {
  const code = list.filter(f => /\.(rs|ts|svelte|js)$/.test(f) && !/\.(test|spec)\./.test(f))
  const perChunk = 4
  const chunks = []
  for (let i = 0; i < code.length && chunks.length < 24; i += perChunk) {
    chunks.push({ key: 'diff-' + (chunks.length + 1), files: code.slice(i, i + perChunk) })
  }
  return chunks
}

const targets = DIFF_MODE ? chunkFiles(files) : WHOLE_APP
if (!targets.length) {
  log('safety-inspection: no code targets to audit (diff had no source files).')
  return { mode: DIFF_MODE ? 'diff' : 'whole-app', confirmed_findings: [] }
}
log('safety-inspection: ' + (DIFF_MODE ? ('DIFF mode over ' + files.length + ' changed file(s)') : 'WHOLE-APP cycle sweep') + ' — ' + targets.length + ' hunt group(s).')

const results = await pipeline(
  targets,
  (t) => {
    const where = DIFF_MODE
      ? ('AUDIT THESE CHANGED FILES (read them + the data-flow they touch): ' + t.files.join(', ') + '\nHunt ONLY defects introduced or exposed by THIS code; trace how a caller/state reaches a silent failure.')
      : ('HUNT SUBSYSTEM: ' + t.key + '\nWHERE TO LOOK: ' + t.hint)
    return agent(TAXONOMY + '\n\n' + where + '\n\nReturn scope + candidates[] (each grounded in file:line with a concrete silent-failure scenario). Empty candidates if clean. Do NOT re-report the known register (MIG-098 rename-tail; the documented Charter findings) unless this code newly reintroduces it.',
      { label: 'hunt:' + t.key, phase: 'Hunt', schema: HUNT, effort: 'high' })
  },
  (hunt, t) => {
    const cands = (hunt && hunt.candidates) || []
    const scope = DIFF_MODE ? (t.files || []).join(',') : t.key
    if (!cands.length) return { scope, confirmed: [] }
    return parallel(cands.map(c => () =>
      agent('CONSTELLATION SAFETY INSPECTION — adversarially VERIFY this candidate. Your default is REFUTED unless you can construct a CONCRETE, reachable failure (exact state/timing → silent source-of-truth loss/divergence/corruption, OR a real freeze/leak, with NO surfaced error), grounded by reading the ACTUAL code at and around the site. Refute if: the write is awaited/durable; the error is surfaced elsewhere; a guard/freeze/version-gate/trigger/reconcile/onDestroy-cleanup prevents it; the path is unreachable; or it self-corrects visibly. CONFIRM only a real, silent defect.\n\nCANDIDATE (' + scope + '):\n' + JSON.stringify(c, null, 2) + '\n\nReturn verdict + reason + final_severity.',
        { label: 'verify:' + (c.file || '').split(/[\\/]/).pop() + ':' + c.line, phase: 'Verify', schema: VERDICT, effort: 'high' })
        .then(v => ({ ...c, ...(v || {}), scope }))
    )).then(arr => ({ scope, confirmed: arr.filter(Boolean).filter(x => x.verdict === 'CONFIRMED' || x.verdict === 'NEEDS-REPRO') }))
  }
)

const all = results.filter(Boolean).flatMap(r => r.confirmed || [])
const order = { 'APP-KILLER': 0, HIGH: 1, MED: 2, LOW: 3, NONE: 4 }
all.sort((a, b) => (order[a.final_severity] ?? 5) - (order[b.final_severity] ?? 5))
return {
  mode: DIFF_MODE ? 'diff' : 'whole-app',
  scanned: DIFF_MODE ? files : WHOLE_APP.map(s => s.key),
  by_scope: results.filter(Boolean).map(r => ({ scope: r.scope, confirmed: (r.confirmed || []).length })),
  confirmed_findings: all,
}

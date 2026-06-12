# MIG-076 — Write Integrity ("Single Write Authority") — Phase 1: ARCHITECT

**Date:** 2026-06-11 (evening) · **Mandate (Eisa, verbatim):** *"We have to STOP EVERYTHING, and put our ALL EFFORTS into solving this BUG, once and forever. It is an app KILLER. … My target is 200% guarantee that you will solve it."*

**Trigger:** BUG-023 — a title rename left `Rename Probe v2.md` carrying `Probe Pointer`'s entire frontmatter (title + `cid_cn` identity) with an empty body. Fourth incident of one class.

Territory mapped by four parallel agents (W1 writer map · W2 identity model · W3 lifecycle + in-repo law · W4 industry prior art), 2026-06-11. Their full reports are in the session transcript; this document is the distilled, decision-bearing record.

---

## 1. The corruption class, named precisely

Every editor-originated disk write is **assembled at write time from three independently-mutating identity sources**:

| Ingredient | Source | Mutates when |
|---|---|---|
| properties | `freshProps()` — store lookup **by `tab.id`** | any store content update (reload, watcher, direct mutation) |
| body text | the CM6 pane's doc / a `body` prop | pane epoch changes ({#key}), prop reactivity |
| target path | `mountedFilePath` (frozen at pane mount) or live `tab.path` / `filePath` props | rename, move, tab reuse |

`buildFullContent(properties, body)` → `writeNote(path, …)` (store.ts:981–989). When any transition (tab reuse, rename, cascade reload) lands between ingredient reads, the write is a Frankenstein: **note X's properties + note Y's body → note Z's path**.

**The four incidents of the class:** BUG-012/§140 (stale write-ahead buffer resurrected a deleted note's identity into a new note — the session log's own words: "the corruption Boss reported") · BUG-015/§115 (value-sync `$effect` raced `{#key}` destroy; target body overwritten with source body) · the F2 cascade-stomp family (editors reverting cascade rewrites; patched by the `isCascading` gates) · BUG-023 (above). Each fix guarded **one window**; the assembly mechanism survived every time. **LL-014 (never patch the same bug three times — find the root cause) is invoked: this is strike four.**

## 2. Territory facts (verified, file:line)

### 2.1 Writers (W1 + W4)
- **Frontend compositions:** NoteEditor `handleSave` :174 / `handleFlush` :227 / `handlePromote` :155; PropertyEditor `debouncedSave` :692–693 and `onDestroy` flush :436–437; NotePane `doSave`/`doFlush` (text + frozen `mountedFilePath`); store `saveTabContent` :536–596; `flushAllTabsInLibrary` :519–534 (writes **WAB content only**).
- **`write_note` call sites (frontend):** store.ts:988, +layout.svelte:3498/3715/3786, BacklinksPanel.svelte:167, NotebookNavigator.svelte:244 — **six**, each composing content its own way.
- **Rust .md writers:** `write_note` libraries.rs:330–354 (**path-validation + plain `fs::write` — no identity check, no atomicity, no serialization**); `create_note` :757; `rename_item` :962–969 (frontmatter-title rewrite at the OLD path, then `fs::rename`); `move_item` :1443; cascade walker `update_links_recursive` :4351 (watcher-suppress TTL ~2.5 s, per-file `fs::write`); `ensure_cid_cn` canonical.rs:1230/:1248 (lazy identity injection **on the read path**); `base_edit_cell` bases.rs:737; tasks.rs:467. An atomic temp+rename precedent already exists in-repo (cece/reliability.rs:124).
- **No locking anywhere**: concurrent writers to one path are possible today (main window × second screen share `openTabs`; cascade × editor flush; PropertyEditor × NotePane).

### 2.2 Identity model (W2)
- `openNoteTab` **reuses the active tab object** for a different note (store.ts:1298–1321): same `tab.id`, path+content+name swapped. `tab.id` is therefore **not** a note identity — it is a *slot* identity. Every `freshProps()`-by-id lookup is built on the wrong key.
- **Six direct `tab.content =` mutation sites** outside any single owner: PropertyEditor :436/:692, NoteEditor :149/:231, FocusPane handler (+layout:6072), second-screen `onNoteSaved` listener (+layout:2739) — plus the store-level updates (openNoteTab, reloadTabsFromDisk :490, watcher :2646, history :709).
- **The WAB identity check FAILS OPEN** (store.ts:1172–1174): stale buffered content is rejected only when `wabCid && diskCid && wabCid !== diskCid`; if either cid is unreadable, **the stale buffer wins**. Legacy/no-cid notes turn the WAB into a time bomb.
- **Two PropertyEditor instances**: one inside NotePane (dies per {#key} epoch), one in the right sidebar (+layout:6487) that **survives tab switches**; its re-seed `$effect` (PropertyEditor:311–334) keys on `tabId` — which never changes on tab reuse — and skips re-seeding while `saving` is true.

### 2.3 Lifecycle + in-repo law (W3)
- Flush triggers: 1.5 s debounce, 30 s idle, visibilitychange, beforeunload, onDestroy (timers cleared before the destroy-flush; doc read before `view.destroy()`).
- The laws already on the books: NotePane spec **§2.1** (CM6 owns the doc; one-way flow), **§2.2** (store updated only on close/switch/reload), **§2.6** (no `$effect` touches editor content); Rename Concept Paper **D2** (open editors are the most fragile state; three named failure modes), **D6** (recreate-or-imperative-dispatch, never reactive sync), **P3/P4** (body sacred; open-editor coherence), **F2** (body corruption named as the failure to never repeat).
- The `isCascading` gate is **path-keyed at mark time** — a rename inside the window re-keys the tab while the gate stays on the old path (the gap is real; today it is narrowly survived only because timers are cleared on destroy).

### 2.4 BUG-023 reconstruction — what is proven, what remains
- **Proven (disk + screenshots + code):** "Rename Probe" never had body text, so the corpse = *frontmatter swap only*. The fatal write is the ordinary flush shape — `buildFullContent(store-props-by-id, pane-doc)` → renamed path — fired **after the tab's store content had become Probe Pointer's pre-cascade content** (screenshot 1 shows the store holding it). Every guard behaved as designed; the *input* was poisoned.
- **Unpinned:** which bridge put Probe Pointer's content into the tab's store entry. Ranked candidates (W2): the WAB fail-open restore; a stale-reference direct mutation (PropertyEditor class); the second-screen `onNoteSaved` path-matched reload; watcher path-match in the rename window; split-view duplicate instances. **The write journal (this design) pins it at first recurrence in dev — and every layer below kills the class regardless of which bridge it was.**

## 3. Prior art (W4 — Working Agreement #5)

VS Code (one `TextFileEditorModel` per URI; snapshot+`versionId` saves; per-model `TaskSequentializer` — "never ever must 2 saves execute at the same time"; mtime+size ETag → `FILE_MODIFIED_SINCE` → Compare/Overwrite dialog) · Obsidian (`Vault.process()` atomic read-transform-write; synchronous critical section) · IntelliJ (`FileDocumentManager` sole flush authority under a global write action) · Git (lockfile `O_CREAT|O_EXCL` + atomic rename; **racy-git**: equal mtimes ⇒ re-hash content) · CouchDB `_rev`/409 + sibling revisions (never destroy either side) · Kubernetes `resourceVersion`/409 · SQLite WAL (**single-writer serialization makes races impossible rather than detected**) · Dropbox/Syncthing conflict-copy naming · VS Code Local History (bounded per-file save journal).

**The convergent industry principle:** one owner per file; writes composed from one point-in-time snapshot; writes *conditional* on what the owner last knew; refusal is surfaced to the human with a path forward; refused/conflicting content is never destroyed.

## 4. The design — five locks, a journal, and a recovery surface

**The 200% structure: each lock alone would have stopped BUG-023; together they close the class, its stomp-variant, torn writes, and the unknown-future-bug case.**

- **L0 — Per-path single-writer gate (Rust).** A `WriteGate` module: per-path async mutex every .md write funnels through (all ten Rust writer sites). Serialization first — CAS without it is check-then-act (TOCTOU). This also turns the quiesce protocol (L4) from courtesy into enforcement: the cascade holds the same locks the flush needs.
- **L1 — Atomic replace.** Inside the gate: same-dir temp → `sync_all` → `ReplaceFile`-class swap, bounded AV retry (5 × 50–200 ms on sharing violations), `watcher_suppress` marking **both** temp + final path. Kills torn/zero-length files (today's plain `fs::write` is exposed) independent of the race class.
- **L2 — Identity + freshness CAS at the boundary.** Every content write carries `{expected_cid_cn, expected_base (mtime+size, hash on ambiguity — racy-git)}`. Gate re-reads the target under the lock: identity mismatch → **refuse + quarantine**; freshness mismatch (same note, newer disk) → **refuse + Compare/Overwrite/Keep-both UX** (VS Code semantics, not a dead toast). Creation is a distinct **create-exclusive** verb (`If-None-Match: *` / `O_EXCL` semantics) — "write to path" can never create. Rollout in **shadow mode first** (log would-be refusals, allow) → enforcement flip after a clean soak.
- **L3 — Single-snapshot composition + single store writer (frontend).** A write request becomes one immutable value `{path, cid_cn, base, content, docVersion}` composed **inside the owning pane at one instant** — `buildFullContent(freshProps-by-id, pane-text)` joins are abolished. One `updateTabContent` authority replaces the six scattered `tab.content =` mutations. The WAB check flips **fail-closed** (buffer wins only when both cids present *and* equal). `docVersion` (VS Code `versionId`) prevents marking-clean over keystrokes typed during a save.
- **L4 — Quiesce protocol for rename/move/cascade.** The sequence becomes: freeze pane (read-only overlay) → final flush through the gate → rename + cascade as one gate-locked operation → deliberate remount. The title-rename feature (the BUG-023 trigger) re-lands **only** on top of this, closing the original §13 gap safely.
- **Journal + quarantine.** `write_journal` (SQLite): ts, path, expected/found cid, content hash, bytes, outcome, surface — bounded like Local History; quarantined payloads at `.constellation/quarantine/` with Dropbox-grade self-describing names (`{stem} (refused {surface} {ts} expected-{cid8} found-{cid8}).md`). Nothing refused is ever lost; every anomaly is attributable in minutes.

### Known risks (named now, owned in the Plan)
1. **The cid-absent population** is L2's biggest hole: legacy notes without `cid_cn` degrade CAS exactly where the class lives → a **resumable background backfill** (Rule-8 shape, status-bar progress) must bring the universe to 100% cid-carrying; until then shadow mode logs rather than refuses.
2. **Over-strict refusal kills trust in the guard** (externally edited files, BOM/CRLF/duplicate keys): robust frontmatter parsing + the Compare/Overwrite path for freshness conflicts; hard refusal reserved for identity mismatch.
3. **Perf invariants (hard):** zero keystroke-path change (writes stay ≥1.5 s debounced; one extra pre-read per write under the lock is microseconds); boot untouched; no new `$effect` on editor content (spec §2.6 stands).
4. **Second screen** shares `openTabs` — L0 serializes cross-window writes at the Rust boundary regardless of frontend state.

## 5. Invariants that must hold through the migration
I1 typing latency unchanged · I2 boot time unchanged · I3 CM6 ownership laws (spec §2.1/2.2/2.6) untouched · I4 cascade contract (a)–(e) preserved, now lock-enforced · I5 no user write silently dropped (refusals always journaled + surfaced) · I6 no legitimate save blocked (shadow-soak before enforcement) · I7 every quarantined payload recoverable · I8 second-screen display-not-domain preserved · I9 all 15 locales for every new user-facing string · I10 reversible rollout (enforcement behind a flag until Boss-validated).

## 6. Phase skeleton (detail in the Plan)
§A WriteGate (L0+L1) + journal, all Rust writers routed, tests · §B CAS tokens (L2) shadow mode + create-exclusive + cid backfill · §C frontend snapshots + single store writer + WAB fail-closed (L3) · §D quiesce rename/cascade (L4) + title-rename re-land · §E refuse/recovery UX + i18n ×15 · §F enforcement flip after soak + lifecycle regression suite (the BUG-023/015 interleavings as automated tests) + /simplify + 3-agent audit + Boss stages.

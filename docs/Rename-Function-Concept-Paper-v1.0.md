# Constellation Rename Function — Concept Paper

**Version 1.0 | 2026-05-02**

**Author of facts**: Eisa ALSHAMSI (project owner, designer, IT Boss)
**Maintainer**: Claude (consultant / engineer / SME)

This paper defines the **Rename** function as a first-class part of Constellation. It is the analog of [`docs/360.3D-Concept-Paper-v1.0.md`](docs/360.3D-Concept-Paper-v1.0.md) for the rename surface — what it is, why it exists, what it promises, what it must not do, and the design principles every implementation pass must satisfy.

It exists because of a specific failure: the §115 commit (MIG-006 §3 expanded, 2026-04-25) shipped a cascade-reload mechanism that referenced "an existing prop-change handler" that didn't exist. To make the reference real, the commit added a `$effect` in NotePane that synced the parent's `value` prop into the CodeMirror doc. That `$effect` raced with `{#key}` `onDestroy` on tab navigation and corrupted target body content with source body content. The plan fabricated the existing-handler reference; the plan misled itself into BUG-015. This concept paper exists so that future plans for the Rename function can be checked against a real definition rather than against a migration document that sits at the technical layer.

---

## 1. What Renaming is

Knowledge formulation is an ongoing act. The first time you write a note, you name it from where you stand at that moment. Six months later your understanding has moved — the original name is too narrow, too broad, in the wrong language, or simply wrong. Renaming the note is part of formulation, not metadata maintenance.

In Constellation, **renaming a note is a conceptual revision** — a re-signing of the relationship the rest of the network has with that note. The mechanical surface is a filesystem operation: a `.md` file's name changes, and every wikilink across the library that pointed at the old name has to be rewritten. The cognitive surface is something different: you've decided that the concept this note represents is better named X than it was named Y, and the entire library has to follow.

The wikilink rename cascade is the mechanism that keeps the network honest after the re-signing.

---

## 2. Why Rename exists

A knowledge tool that does not support renaming reliably forces calcification. The user hesitates to rename because they fear what might break. The library accumulates names that diverge from current understanding. The user works around naming instead of fixing it. Names that should mature get frozen.

A tool that supports renaming **unreliably** is worse than one that doesn't support it at all. The user renames, the wikilinks dangle, the user loses trust, the user stops renaming, and from then on every revision happens silently in the user's head while the library drifts further from it.

Constellation's design promise is the third state: **renaming is safe**. Whatever the user knew about the note's relationships before the rename, the user still knows after. No connections lost. No body content lost. No silent failures. Names are allowed to grow with understanding.

---

## 3. The Rename Promise (8 invariants)

What the user can rely on when they trigger a rename:

**P1. Identity preservation.** The note's persistent identity (`cid_cn` token in frontmatter, set at creation) is never touched by rename. Renaming changes the human-readable filename (post-MIG-003) and the frontmatter title; the cid_cn is immutable. References by cid_cn (second-screen sync, IPC events, search FTS5) keep working through the rename.

**P2. Wikilink integrity.** Every wikilink across the library that referenced the old title is rewritten to reference the new title. Coverage includes:
- `[[OldTitle]]` — bare
- `[[OldTitle|display text]]` — with display alias
- `[[OldTitle|supports]]` and the other typed forms — typed annotation preserved
- `[[OldTitle|alias|supports]]` — both display and type preserved
- `![[OldTitle]]` — embed/transclude

The cascade rewrites the **target token** only. Everything from the first `|` onward is preserved verbatim. The `!` prefix is preserved.

**P3. Body content sacred.** Source notes' body content **outside the wikilink target token** is never touched. The cascade rewrites wikilink targets only — never paragraphs, never list items, never code blocks, never frontmatter outside the title field, never typed annotations, never display aliases.

**P4. Open-editor coherence.** If a source note (containing `[[OldTitle]]`) is open in a tab while the rename happens, the editor's view stays consistent. The user does not lose typing in flight; does not see a stomp where their next autosave reverts the cascade; does not see ghost characters from a stale buffer.

**P5. Per-file atomicity.** Each source rewrite is atomic at the filesystem level (tempfile + atomic rename). If the process crashes mid-cascade, every file on disk is either fully old or fully new — never half-written.

**P6. Cascade-level visibility.** When the cascade is non-trivial (>100 inbound links), the user sees progress and can cancel. Cancellation is well-defined: already-rewritten files stay rewritten; pending files are skipped; the user knows the count of each.

**P7. Index reflects disk.** After the cascade completes, the SQLite index (`note_links.target_name`, FTS5 vocabulary, alias table) reflects the post-rename state. Searches, strata derivations, and panel reads are consistent with the rewritten wikilinks on disk.

**P8. Reversibility.** Renaming back from the new title to the old title runs the inverse cascade. The library round-trips cleanly: rename A → B → A leaves the library byte-identical to before A → B, modulo filesystem timestamps and the alias-table history (which records both renames).

---

## 4. Scope

The Rename function this paper defines covers **note title rename** only.

| Operation | In scope | Reason |
|---|---|---|
| Rename a note (title change → filename change → wikilink cascade) | ✅ | This paper. |
| Rename a folder | ⏸ | Folder renames change file paths, not wikilink target names. The wikilink cascade does not fire. The file-watcher's renamed-event triggers a path-update for affected notes' open tabs. Separate concern. |
| Rename a library | ⏸ | Library-level identity is settings, not content. Wikilinks reference target titles within the library, not library names. Separate concern. |
| Edit the title in note body | ⏸ | That's wikilink retitling within prose, not a Rename. The cascade does not fire because no wikilink TARGET changed. |
| "Save as" / fork | ⏸ | Creates a new note with a new identity; the source is unchanged. |
| Rename across libraries within a Universe | ⚠ Open question — see §10 |

This paper covers note rename only. Folder, library, and cross-library concerns are tracked separately.

---

## 5. Design principles

**D1. The user's content is sacred.** The cascade only modifies wikilink target tokens within wikilink syntax. It never modifies prose, never modifies frontmatter outside the title slot, never modifies typed annotations or display aliases. If a future feature requires modifying user content beyond wikilink targets, it is a different function and gets its own concept paper.

**D2. Open editors are the most fragile state.** The cascade must coordinate with them, not race them. Three known failure modes the design must close:
- *Pre-cascade staleness*: the source's debounced autosave hasn't fired; the walker reads stale disk.
- *Post-cascade stomp*: the source's NEXT autosave overwrites the cascade with its still-pre-cascade in-memory copy.
- *Watcher loop*: the walker's `fs::write` bubbles back through the file watcher as an external edit, racing the editor's read-back.

**D3. Per-file atomicity, multi-file partiality.** Each file's rewrite is filesystem-atomic. The cascade as a whole is **not** transactional across files. Partial cascades are valid intermediate state. When a single file fails (locked, permission denied), the cascade continues with the rest and surfaces the failure to the user. Retry is per-file.

**D4. Visibility scales with scope.** Single-file renames are silent (the rename itself shows; the cascade is invisible). Cascades up to 100 inbound run synchronously. Cascades >100 inbound run asynchronously with a progress toast and cancel button. The user is never surprised by the duration.

**D5. Reversibility means undamaged round-trips.** A rename A→B followed by a rename B→A leaves the library byte-identical to its state before A→B (allowing for filesystem timestamps and alias-table history). The cascade has no destructive non-deterministic behavior.

**D6. Reactive coherence over reactive convenience.** The §115 / BUG-015 incident proved that the rename + reload pipeline must NEVER use `$effect` to sync editor body content from a parent prop. CodeMirror's editor doc is owned by the EditorView; reactive sync into it races lifecycle. Two acceptable mechanisms for closing the open-editor coherence loop:
- **Recreate the editor** via Svelte's `{#key}` block — the parent bumps a key value, the editor destroys and re-mounts with fresh content. Cursor and scroll are lost; correctness is guaranteed.
- **Imperative dispatch** through a known, lifecycle-aware ref — the parent calls `view.dispatch({ changes: ... })` directly, never via `$effect`. Cursor and scroll preserved; lifecycle ordering must avoid dispatching into a destroying view.

Never an `$effect`-driven `view.dispatch`. NotePane spec §2.6 codifies this prohibition.

**D7. The cascade is one user action.** From the user's POV, rename is one click in the file tree. Internally, that one click triggers (1) frontmatter title write, (2) filename change on disk, (3) tab-state update, (4) flush of dirty source tabs in the affected library, (5) walker rewrite across the library, (6) reindex of each rewritten source, (7) reload of each affected open tab. All seven steps must succeed or all seven must report a coherent failure to the user. The user must never be left wondering "did the rename actually happen?"

**D8. The walker is alias-aware.** Pre-MIG-006 stale wikilinks (where the body still says `[[OldTitle]]` because the rename happened in a session before the cascade was reliable) are addressable via the `note_aliases` table's rename rows. The user can opt into a backfill that replays the cascade for all historical renames (Settings → Files → "Rewrite stale wikilinks…").

---

## 6. What Rename interacts with

| Subsystem | Interaction |
|---|---|
| File watcher | The cascade marks each path as a recent write before `fs::write`; the watcher's external-edit emit path skips when marked. Without this, the watcher loops: cascade → external-edit emit → reload → cascade. |
| Open editor | Flush-before-cascade + reload-after-cascade. The reload mechanism is one of the two D6-allowed primitives (key-bump or imperative dispatch). Never an `$effect`. |
| `note_links` index | Reindex each rewritten source via `index_note(conn, path, library_name)` so `target_name` reflects the new title post-cascade. Without reindex, search results show stale targets even though the wikilinks on disk are correct. |
| `note_aliases` table | Each rename writes a row with `source='rename'`. This row is the historical record (used for backfill) and a defense-in-depth read-side resolver (used by MIG-004 alias-aware reads for typos / partial / Arabic-normalized matches during the brief reconciliation window between cascade and reindex). |
| Sky View, 360.3D Inspector, Backlinks, Outgoing | All depend on `note_links`. The reindex from D7 keeps them coherent post-cascade. |
| Living Link properties (per [`docs/Living-Links-Guide-v1.0.md`](docs/Living-Links-Guide-v1.0.md)) | The cascade preserves all 8 link properties — `weight`, `confidence`, `created`, `last_traversed`, `traversal_count`, `annotation`, `link_type`, `direction`. The link's identity is its `(source_path, target_path)` pair; the cascade updates `target_path` (when MIG-003 filenames track titles) but never touches the other properties. |
| External sync (Git, Syncthing, iCloud) | The cascade produces N file-content writes (one per affected source) plus one rename. Sync sees this as a multi-file commit. The user may want to commit the rename + cascade as a single Git commit; that's a UX layer the cascade design itself doesn't constrain. |
| MIG-003 (human-name filenames) | When MIG-003 lands, filenames track titles for new notes. The rename then changes both filename and title in lockstep. The walker's frontmatter-title lookup (MIG-006 §1) remains correct for legacy canonical-named notes; for human-name notes the lookup is a no-op. |

---

## 7. The cognitive role

The Five Acts of Knowledge Creation (per [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) Part V) span timescales from minutes (capture) to years (deepening understanding). The names you give your notes are the most visible artifacts of where you are in that arc.

**A note's name should be allowed to change.** Knowledge tools that punish renaming punish learning. Constellation's Rename function exists so that renaming is friction-free and consequence-free.

The cascade is the mechanism that delivers the consequence-free part. Without it, every rename creates orphan wikilinks and the user starts avoiding renames. With it, the user renames freely; the network adapts.

---

## 8. The Rename failure modes

What goes wrong when the function isn't right:

| # | Failure | Symptom | Address in MIG-006 § |
|---|---|---|---|
| F1 | Dangling wikilink | Body still says `[[OldTitle]]` after rename. Click goes nowhere. | §1 (oldName from frontmatter title), §2 (walker correctness) — both shipped. |
| F2 | Body corruption | Cascade or its open-editor-coherence layer overwrites body content with content from another note. | §3 (open-editor coherence). The original §3 burned this; redo must not. |
| F3 | Watcher loop | Cascade `fs::write` bubbles as external edit; reload triggers cascade; etc. | §3 (RECENT_WRITES Rust map). |
| F4 | UI freeze on hub renames | Sync cascade walks 1000 inbound links on the main thread; UI unresponsive. | §6 / §7 (sync/async dispatch + progress events). |
| F5 | Index stale post-cascade | Wikilinks on disk are new; `note_links.target_name` still says old. | §4 (reindex via `index_note`). |
| F6 | Annotation loss | `[[OldTitle\|supports]]` becomes `[[NewTitle]]` (annotation stripped). | §2 — shipped. |
| F7 | Display alias loss | `[[OldTitle\|My Display]]` becomes `[[NewTitle]]` (alias stripped). | §2 — shipped. |
| F8 | Embed loss | `![[OldTitle]]` not rewritten because the parser missed the `!` prefix. | §2 — shipped + 11 cascade tests. |
| F9 | Partial cascade with no failure surface | Cascade rewrites some files, fails on others, user never sees the failure list. | §7 (completion event with `failed[]` payload). |
| F10 | Pre-MIG-006 stale wikilinks | Body still says `[[OldTitle]]` from a rename that happened before the cascade was reliable. | §10 (backfill command). |
| F11 | Cascade interrupted by another rename | User renames again before the first cascade completes. | §6 (rename_id + cancellation primitives + serialized cascade dispatcher). |

The §3 redo addresses F2, F3, and the open-editor side of F11. §1 + §2 (already shipped) address F1, F6, F7, F8. §4 (queued) addresses F5. §6 / §7 (queued) address F4, F9, F11. §10 (queued) addresses F10. §9 (queued) addresses crash-mid-cascade integrity for all of them.

---

## 9. Done means…

The Rename function is "done" — in the same sense the 360.3D Inspector is "done" after Stage 3 — when:

1. **All eight P1–P8 invariants hold** under the worst test cases:
   - Open dirty editor on a hub note while renaming the target.
   - Cascade interrupted by another rename mid-walk.
   - Cascade racing concurrent autosaves.
   - Cascade on a 1000-inbound note (async path).
   - Cascade on Arabic-titled notes (no normalization mismatch).
   - Cascade in mixed RTL/LTR libraries.
   - Cascade with a tab open in the second screen, not just the main window.
2. **A Rename Reading Guide exists** for the user — analog of the [Matrix Reading Guide](docs/360.3D-Matrix-Reading-Guide-v1.0.md) — explaining what the function promises, what edge cases it handles, what the user should expect on a hub-note rename, and what to do if a rename appears to misbehave.
3. **All MIG-006 §3–§11 work has shipped**, plus Phase 4 audit closure.

§3 is the most architecturally complex of the eleven; it is the one that closed BUG-015.

---

## 10. Open questions

Not blockers for the §3 redo. Define the perimeter of the function.

1. **Cross-library renames.** A wikilink whose source and target are in different libraries within the same Universe. Today the cascade is library-scoped; cross-library wikilinks stay dangling. Is this acceptable, or does the cascade need to span the Universe?
2. **Rename initiated from the second screen.** The second screen is a display, not a domain. Verified that rename-from-second-screen triggers the cascade in the main library, or does it bypass?
3. **Rename + sync conflicts.** Two devices rename the same note before sync. Merge semantics?
4. **Rename cancellation mid-cascade — recovery path.** Cancelling leaves some files rewritten, some not. The library is in a partial state. Is this user-visible? Is there a "complete the cascade" button later?
5. **Rename of a note while it's being indexed.** Race between rename and an in-flight `index_note` call. Behavior?
6. **Rename of a note that is the target of an inline embed `![[Foo]]` from a note in a different library** (subset of cross-library question, but specifically for embeds).

These six are the perimeter. They should be answered before the function is declared "done." None of them block the §3 redo.

---

*End of v1.0. Maintained per Standing Order #6 — when any P1–P8 invariant changes, when the scope expands (e.g. cross-library renames ship), or when a new failure mode is added to §8, bump this paper to v1.1 in the same commit that lands the change. Older versions are preserved.*

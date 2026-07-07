# MIG-099 — Create-Latency Fix: Index-Backed Name Resolution (Architect)

**Status:** Architect complete → awaiting Plan approval
**Opened:** 2026-07-07 · **Owner:** Claude (autonomous remediation) · **Boss:** Eisa
**Precedes:** G4 (frontmatter parser) — Boss gated this latency fix before G4.
**Analysis:** multi-agent workflow `wf_cdcc99bc-1a7` (11 agents, 1.05M tokens): call-site census · index schema/currency + federation · invariant enumeration · WA#5 proven-pattern research · design synthesis · adversarial refutation.

---

## Concept (the horse)

> **A note-name's "does this already exist / where does it point?" question must be answered from the always-current index (`note_meta` / `note_aliases`), not by reading every `.md` file on disk.**

This is Rule 8 (Write-Time Derivation) applied to name resolution. The folded match key is *already* maintained on the write path (`index_note` UPSERTs `name_lower = fold_match_key(name)` and re-inserts `note_aliases` on every create/save/rename/move). The read must simply become the cheap lookup — the field's standard answer — instead of re-walking 2 GB.

---

## 1. Confirmed root cause (measured, not theorized)

Live universe **Eisa Cognitive Knowledge** (2 GB, ~7,700 notes), `diagnostics.log`, 2026-07-07:

```
21:22:37  resolve_wikilink_cross_library took 13575 ms (matched=false)   ← the CREATE
21:22:56  resolve_wikilink_cross_library took   801 ms (matched=false)   ← the RENAME (OS cache warm)
          read_library_tree 0–32 ms · update_links_on_rename 2 ms        ← NOT the cost
```

**Mechanism:** `resolve_wikilink_cross_library_impl` (`libraries.rs:1779`) resolves in two stages:
1. `find_note_by_name` — filename-**stem** match via `read_dir`, **no file reads** (cheap).
2. Only if stage 1 finds nothing → `find_note_by_title_or_alias` (`libraries.rs:1905`) — `fs::read_to_string` of the **whole** `.md` of **every** note in **every** library, to test frontmatter `title:`/`aliases:`.

A **brand-new note name matches no stem anywhere** → stage 2 runs over all libraries → full 2 GB **cold** read → **13.6 s**. Rename was fast only because (a) the second scan hit the warm OS cache and (b) `update_links_on_rename` found 0 linkers. The create runs this from `createNoteWithTemplate` (`+layout.svelte:4042`).

---

## 2. Call-site census (who calls the resolver)

Rust command `resolve_wikilink_cross_library` (`libraries.rs:1760`) · JS wrapper `resolveWikilinkCrossLibrary` (`store.ts:2692`) · one raw `invoke` (`+layout.svelte:4953`). Returns `ResolvedLink { path, library_name, library_path, fragment }`.

**Write-path existence checks (the 13.6 s victims) — need exists+locate:**
- `createNoteWithTemplate` (`+layout.svelte:4042`) — MIG-076 §E1b create-time title-collision guard → the CREATE.
- `handleRenameComplete` (`+layout.svelte:6040`) — rename-time title-collision guard (self-match excluded) → the RENAME.

**Read-path resolution — need resolved path (+library+fragment):**
- `handleWikilinkHover` (`+layout.svelte:4953`) — hover page-preview (debounced 400 ms).
- `CodeMirrorEditor.headingCompletion` (`:778`) — **ON THE KEYSTROKE PATH** (`[[note#` completion source can re-fire per keystroke; a title/alias-only ref triggers the full scan → a live Rule 3 violation today).
- `OutgoingLinksPanel` (`:129` up to 120 resolves/tab-open, `:173` click), `NoteEditor.handleLinkClick` (`:407`), `PropertyEditor onNoteClick` (`:8164`).

**Also same-class (candidate follow-ups, out of MIG-099 core scope):** `trails.rs:203 find_note_recursive` (via `resolve_note_path`) walks + reads canonical-note titles.
**NOT the class (all O(1) `.exists()`):** `create_note` gate, `create_folder`, `rename_item`, `move_item` single-path collision guards; `resolve_filename_collision` (single-dir `read_dir`, bounded).

---

## 3. The index (the Rule-8 source of truth)

`note_meta(path PK, name, name_lower, library_name, modified, …)` + `note_aliases(path, alias_lower, source, …)`.

- `note_meta.name` = frontmatter `title:` when non-empty, else file stem (`search.rs:5464`) → **a canonical-filename note IS findable by its human title via the index** (adversarial claim 1 — **HOLDS**).
- `name_lower = fold_match_key(name)` (`search.rs:5641`): `NFC → to_lowercase → NFC`. Full-Unicode fold, **not** ASCII `to_lowercase`. Backed by covering index `idx_note_name_lower(name_lower, path)` — index-only seek, measured elsewhere at **0.06 ms** vs a 21,915 ms full scan.
- `note_aliases.alias_lower = normalize_alias_for_match(x) = normalize_arabic_for_search(fold_match_key(x))` — `fold_match_key` **plus** Arabic tashkeel/tatweel strip. **Asymmetry:** the query must fold the target *per column* (`fold_match_key` for `name_lower`; `normalize_alias_for_match` for `alias_lower`).
- **Currency:** maintained write-time by the Rust `index_note` path (create/save/rename/move) — not a trigger. `reindex_delete_note` removes `note_meta`/`note_links`/etc. but **not** `note_aliases` (orphan-alias gap).

---

## 4. WA#5 — proven-pattern cross-check ("don't reinvent the wheel")

**Verdict: the indexed-lookup is the battle-tested pattern; there is no better one, and the file scan is the discarded anti-pattern.**

- **Obsidian** — `MetadataCache` + `getFirstLinkpathDest` / `uniqueFileLookup` (in-memory keyed index; nearest-then-root precedence).
- **Logseq / Roam** — DataScript/datom name index in memory.
- **Dendron** — markdown-on-disk + **SQLite/FTS5 index maintained by triggers** (migrated off in-memory scanning, measured **~30×**). *Architecturally identical to Constellation's `note_meta`.*
- **TiddlyWiki** title hashmap · **Foam** workspace index. None touch the filesystem per link.
- **SQLite multilingual:** pre-fold once into an indexed key column + B-tree seek is *the* recipe. **`COLLATE NOCASE` is ASCII-only — disqualified for Arabic.** The one load-bearing rule: **identical fold on write and query** (mismatch = silent miss). Constellation already satisfies this via `fold_match_key`.

---

## 5. Design options

| | Approach | Speed | Effort | Risk |
|---|---|---|---|---|
| **A** | Pure index replacement (drop the walk); add a `stem_lower` column + backfill migration; cross-schema attached-cuN unions | Fastest (sub-ms even for stems) | **High** (schema migration + cross-schema) | **High** (silent stem-link loss; federation attach holes; a `/migration` of its own) |
| **B** | Bounded 1–2 KB frontmatter read (keep the walk) | Partial — still O(files) `read_dir`; seconds cold | Low | Low mechanically, **fails the sub-10 ms goal** |
| **C ✅** | **Keep stage 1 (stem `read_dir`) untouched; replace ONLY stage 2 (the full-file scan) with an indexed `name_lower`+`alias_lower` seek for OWN-universe libraries; bounded walk for FEDERATED libraries** | Sub-10 ms for own libraries (the 2 GB); federated cost bounded to the small federated tree | **Medium** (surgical, no schema change; reuses existing indexes) | **Low** (stem invariants = untouched code; ordering preserved by reusing the loop; folding change is a correctness improvement; federation explicit) |

**Recommended: C.** The measurement pins 100 % of the 13.6 s to stage 2; stage 1 is already cheap. Replacing *only* stage 2 turns 13.6 s → sub-10 ms while leaving the stem-resolution code — and all its invariants — literally unchanged. No schema migration. Option A is the theoretical end-state but is its own `/migration` with silent-link-loss risk; B misses the bar.

---

## 6. Adversarial corrections (the build MUST incorporate these — 4 of 5 claims refuted)

The independent refutation pass found five concrete, code-grounded failure scenarios in the naïve C. Each is now a **build requirement**:

**C1 — Federation origin must be explicit, NOT path-vs-root. [CRITICAL]**
`main.note_meta` never authoritatively holds federated rows (each cUniverse has its own `search.db`, attached read-only as `cu0..cuN`; `main` reflects federated notes only stale-as-of-the-last `cache_reconcile`). `LibraryInfo` carries **no** origin field and `resolve_libraries_recursive` (`universe.rs:384`) flattens+dedups own+federated into one Vec. **A cUniverse can be rooted UNDER the active universe root** (e.g. `E:/U/shared`) → "path under root ⇒ own" **misclassifies it as own → indexed query on `main` misses it → silent dead link + false-negative collision.** Conversely an external OWN library sits outside the root → misclassified federated → slow-scan perf regression.
→ **Fix:** carry an explicit `is_federated` marker (or a federated-path set) from `resolve_libraries_recursive` so the boundary survives flattening. Route **only** libraries whose rows are authoritative in `main.note_meta` (own) to the indexed seek — **trusting an index miss as "does not exist."** Keep the **live disk walk for every federated library regardless of physical location** (always correct; immune to attach state / 25-cap / schema drift). *(Phase 4 may later upgrade federated to attached-cuN indexed queries — optional, Boss-gated.)*

**C2 — Synchronous, propagated reindex on create (freshness). [CRITICAL]**
`create_note`/`write_note` do **not** reindex; only the debounced watcher does. So an index-miss can only be trusted as "does not exist" once the note is indexed. The retained stem stage **cannot** cover a title whose filename stem diverges via reserved-char rewrite (`note_display_filename`: `"Ratio A/B"` → stem `"Ratio A B"`). → A same-session second create of `"Ratio A/B"` before the first is indexed: stem miss + index miss → **duplicate title silently created, defeating MIG-076 §E1b.**
→ **Fix:** make `create_note` (and the Overwrite recreate path) call `reindex_single_note` **synchronously with a propagated (non-`let _ =`) error**, so `note_meta.name_lower` is authoritative the instant the file exists. This is also the architecturally-correct Rule 8 move (index in the same step as the write). *(All existing `reindex_single_note` sites swallow errors with `let _ =` — `libraries.rs:1559/1618/1624`, `bases.rs:404`, `tasks.rs:532`; the new create-path call must not.)*

**C3 — Byte-length tie-break, not char-length. [Arabic-critical]**
The walk's tie-break is Rust `sort_by_key(|p| p.to_string_lossy().len())` = **UTF-8 byte** count. SQLite `length(path)` = **character** count. For Arabic paths (the app's core Language-First case, 2 bytes/char) these **invert** → the same wikilink opens a **different** note.
→ **Fix:** `ORDER BY length(CAST(path AS BLOB)) ASC LIMIT 1` (BLOB length = bytes), and ensure the stored `path` form matches the walk's `to_string_lossy()` separator style.

**C4 — Fold with `fold_match_key`/`normalize_alias_for_match`, never `COLLATE NOCASE` / plain `LOWER`.**
The query key must pass through the identical write-side fold, and use the pre-MIG-085 fallback shape `COALESCE(name_lower, LOWER(name))` as two index-seeking arms (`WHERE name_lower=?1 UNION … name_lower IS NULL AND LOWER(name)=?1`) — as `resolve_incoming_target_paths` (`search.rs:1407`) already does.

**C5 — `fold_match_key` is an intentional behavior CHANGE (flag, don't claim parity). [Boss ruling]**
`fold_match_key` unifies Unicode canonical equivalents (a strict superset of the walk's byte-exact `to_lowercase`). It will (a) make some currently-broken accented/NFD/Arabic-diacritic links **start** resolving (improvement, aligned with **Language-First**), and (b) unify canonically-equal titles the walk kept distinct (possible tie-break flip). Parity and the NFD fix are **mutually exclusive** — we choose the fix. → Document as an intentional canonical-equivalence change; **Boss confirms** (expected: yes, it is the Language-First-correct behavior).

**Plus the design's own gaps (fold into Phase 3):** stat-`exists()` the resolved path before returning (guards orphan `note_aliases` rows + unmounted-lib rows); Overwrite path must `reindex_delete_note` the trashed note (no phantom collision); verify `index_note` alias extraction covers every YAML shape `has_alias` handles.

---

## 7. Phased plan (each step = one commit + verification clause)

**Phase 1 — Indexed title/alias resolver helper (additive, no call-site change).**
New `resolve_title_alias_indexed(conn, library_dir, folded_name, folded_alias) -> Vec<PathBuf>`: `note_meta WHERE name_lower=?1 [COALESCE fallback] AND path LIKE library_dir||'%'` UNION `note_aliases` join on `alias_lower=?2` under the same prefix, `ORDER BY length(CAST(path AS BLOB)) ASC`. Predecessor→Replacement: none (additive).
*Verify:* `cargo build` green; unit test seeds `note_meta`+`note_aliases`, asserts shortest-**byte**-path match for a title key, an alias key, an Arabic-vs-ASCII sibling tie-break, and empty on miss; safety-inspection diff-scoped on `libraries.rs`.

**Phase 2 — Wire own-libraries to the index; keep bounded walk for federated (C1).**
Thread the federation-origin marker from `resolve_libraries_recursive`; thread a read connection via `with_read_conn`. Fold target with `fold_match_key` (name) / `normalize_alias_for_match` (alias). Per library the loop already visits in current-first / Vec-order: **own** → Phase-1 helper (trust a miss); **federated** → existing `find_note_by_title_or_alias` bounded to that cUniverse's tree. Stage 1 (`find_note_by_name`) stays first, unchanged. Predecessor→Replacement: same function, own-arm swapped to index, federated-arm keeps walk (no relocation).
*Verify (Reproduce-First on the live 2 GB universe):* `diagnostics.log` create drops ~13,575 ms → sub-10 ms; `[[title-only]]`, `[[alias]]`, and `[[stem]]`-with-distinct-title all open the correct note; a name in two libraries opens the **same** note as before; an Arabic sibling-folder tie-break matches the old winner; safety-inspection diff-scoped.

**Phase 3 — Close index-currency gaps (C2 + orphan/trash/exists).**
`exists()` the resolved path before returning; Overwrite path `reindex_delete_note`s the trashed note; `create_note` + Overwrite-recreate `reindex_single_note` **synchronously, error propagated**.
*Verify:* delete a titled note then create a new note with that exact title → no phantom collision; Overwrite flow → trashed title not re-flagged; rapid double-create of `"Ratio A/B"` → blocked on the second; safety-inspection diff-scoped on the create/trash write paths.

**Phase 4 — (OPTIONAL, Boss-gated) fast federated via attached `cuN` schemas.**
Replace the federated bounded-walk with indexed queries against attached `cu0..cuN` (`COALESCE(name_lower, LOWER(name))` per schema; guard `note_aliases` existence per schema), walk-fallback when a cUniverse is unattached / schema-incomplete / over the 25-cap.
*Verify:* large federated cUniverse resolves sub-10 ms; detach → still resolves via walk fallback.

**Phase 5 — Alias-shape parity + remove instrumentation + docs.**
Confirm `index_note` alias extractor covers inline-array / scalar / bare-`- item` block with identical folding (test if divergent). Remove the perf probes (`9bcd590a`/`d318b904`). Update Orientation (SO #6, same commit) + LL entry. Whole-app safety-inspection as the per-cycle sweep before declaring closed.

---

## 8. Open questions → defaults (Boss may override at approval)

1. **Federated resolution speed:** bounded-walk-of-federated forever (correct, cheap) vs Phase 4 now. **Default: ship without Phase 4** unless you routinely federate very large cUniverses.
2. **Create-collision scope:** should a new title clashing ONLY with a **federated** note (a different universe you cannot overwrite) still fire the dialog? **Default: detect across the one-universe** (own via index, federated via bounded walk) per the "one universe" ruling, but offer **Overwrite only for own-library** clashes (Change-name/Cancel for federated).
3. **Stem indexing:** keep the `read_dir` stem stage (cheap, preserves every stem invariant free) vs a `stem_lower` column later. **Default: keep** the stem stage; revisit only if measured slow.
4. **Folding upgrade (C5):** switching to `fold_match_key` makes some accented/Arabic/NFD links start resolving (improvement) — **confirm desired** (expected: yes).

## 9. Invariants that must not break (audit checklist)

stem-first precedence · stem-with-distinct-title resolvability · current-library-first · other-library Vec-declaration order · shortest-**byte**-path tie-break · `library:note` prefix scoping (return None on miss, don't fall through) · `#fragment` passthrough · fold parity (write==query) · alias YAML-shape parity · federation one-universe span · nonexistent-library skip · dotfile/trash exclusion · empty-library-chip guard (`libraries.rs:1826`) · no create-time false-negative collision.

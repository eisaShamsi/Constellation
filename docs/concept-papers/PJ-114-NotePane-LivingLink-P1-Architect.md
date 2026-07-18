# PJ-114 Phase-1 ARCHITECT — "NotePane owns the living link"

*/migration Phase 1 · 2026-07-18 · `wf_5bb4abd1-ca4` · read-only, no code · awaiting Boss picks*

**Concept (the horse):** NotePane is the gate to knowledge cognition — a link's *meaning* (kind, reasoning) must be editable where the thinking happens, not only in raw syntax.

---

## 1. The territory — and three findings that change the plan

Phase 1 touches: the live editor right-click (`NotePane.getEditorMenuItems`), the Backlinks/Outgoing rows (each mounted **6×** in `+layout.svelte`), a new per-link inspector, one setting — and, for 1.1/1.2/1.5, **the user's `.md` files**.

**FINDING 1 — the documented dual-layer model does not exist.** **No LINK file is ever written** (`LINK` is only a classifier kind in `file_kinds.rs`). The real source of truth for **Type and Annotation is the `[[…]]` text in the note body**; `note_links` is a *derived index* rebuilt by `index_note` on every save. Confidence/Weight/Created/Last-Traversed/Traversal-Count are DB-only sidecars with no syntax. So the 8 properties split **2 body-derived / 6 sidecar** — and **a DB-only `setLinkAnnotation` would be silently reverted at the next save.** Framing 1.2 as "an IPC" was the wrong shape.

**FINDING 2 — link identity is `(source_path, target_name, link_type)`** (`UNIQUE`, `search.rs:3469`). There is **no occurrence index**. Repeated identical tokens in one note collapse to one record at parse time (`search.rs:5187`) — **the 2nd token's annotation is already silently dropped today.**

**FINDING 3 — a LIVE silent-data-loss bug sits directly under 1.2.** The indexer's preserve-set (`search.rs:6058`) ignores `confidence`, and the identical-edge test (`:6107`) doesn't compare it. Any annotation edit DELETEs + re-INSERTs the row with hardcoded `'hypothesis'` — **editing an annotation wipes a user-set confidence on an untraversed link.** Blocking prerequisite, not a follow-up (WA#6).

---

## 2. Forked decisions

### A. How re-type + annotate WRITE
| Option | Speed | Effort | Risk | Verdict |
|---|---|---|---|---|
| **(i) Body-text rewrite is the write; `index_note` derives the row** | Med | Med | Med — shares the BUG-015/cascade clobber class | ✅ **RECOMMEND** |
| (ii) Sidecar record write via IPC (mint LINK files + `note_links`) | Slow | Large — builds a second layer that doesn't exist | High — two sources of truth; contradicts File-Over-App; needs full backfill | ✗ |
| (iii) Hybrid: body write + optimistic DB mirror | Med | Med+ | Med-High — mirror diverges; duplicates the parser | ⚠️ only as UI-latency polish |

**Recommend (i)** — the only option consistent with File-Over-App and `addLinkToNote`'s contract. **Mandatory conditions:** the rewrite goes **through the note model / `saveTabContent`**, never a disk read-modify-write; **refused while `cascadeFreeze`/`isCascading()`**; surfaces success/failure (no `catch {}`).

*Also:* `annotation` is **overloaded** — `[[Target|display]]` puts *display text* in the same slot as *reasoning*. **Recommend reusing the slot** (predicate-first `[[type::Target|reason]]` is unambiguous) rather than inventing a second channel.

### B. Inline-gesture link identity
**Recommend:** resolve to the single row via `(source_path, fold_match_key(target), link_type)` parsed from the token under the cursor. Upgrade `NotePane.svelte:1140`'s line regex to the existing **`findWikilinkAtLineOffset`** (`linkAtPos.ts`). Where N identical tokens exist, **say so honestly** ("this link appears 3× — editing the relationship"). Frontmatter-declared links have no body token → fall back to panel/inspector depth. *(Occurrence-ordinal identity rejected: schema change + backfill + breaks the UNIQUE key the indexer rests on.)*

### C. Per-link inspector surface
**Recommend a popover anchored at the link** (shared component, opened from all three D2 depths) — matches the picker precedent; one component serves editor + rows. *(Extending `Inspector360` is a category error — it's note-scoped.)* Needs **read-widening only**: add `created`, `status` to `cache.rs:549` SELECT + `NoteLink` structs (`weight` exists but is dropped by the row map). No schema change, no backfill.

### D. Density setting
**Recommend an enum `'minimal' | 'rich'`** (room for a third tier), default `'minimal'`, living in **Settings → Files → Living Link Lifecycle** (where decay/half-life already live; it has an `updateLinkLifecycle()` helper).

### E. "Dead" items — audit premise HALF-REFUTED
All 5 "dead" menu items are **live and wired in `NotePane`**; they are dead only in `EditorContextMenu.svelte` — **which nothing mounts.** **Recommend DELETE** `EditorContextMenu.svelte`, `CodeMirrorEditor.svelte`, `FormattingToolbar.svelte` (~2.4k lines) after confirming no dynamic import / orphaned table components. *(Verified independently: nothing imports CodeMirrorEditor; NotePane builds its own `EditorView` at `:445/:608` and its own menu at `:1132`.)* The **3 align buttons are end-to-end complete in code** — suspected `bidiPlugin` line-decoration override; **Reproduce-First: no fix designed until it fires live.**

---

## 3. Invariants that must not break

Typing latency untouched · zero `invoke()` on the keystroke path · write-time derivation · **body is authoritative for type/annotation** (never a DB-only write) · no clobber with `updateLinksOnRename`/`cascadeFreeze`/`markCascading` · single content ownership (LL-014) — mutate via the note model only · **row ≠ link** (panels dedupe by target; the DB doesn't) — never extend `set_confidence`'s bulk, type-unscoped `WHERE` to type/annotation · Editor-Parity (new link items reach FocusPane's host-built menu too) · RTL + `$t()` ×15 · macOS: menus/pickers, no new Ctrl chords · **no silent failure** — check affected row counts, surface errors, fix `ConfidencePicker`'s two `catch {}`.

---

## 4. Back-fill / migration / rollback

No schema migration as scoped. Legacy rows may have `created = ''` → the inspector must render an honest "unknown" (measure the count first). Absent density key → `'minimal'`. **Rollback asymmetry is the key point:** P0/1.3/1.4 are read-only and roll back free; **1.1/1.2/1.5 mutate `.md` files and do NOT.** A partial write must never leave a token rewritten with the save unflushed — **the body edit and the save are one unit or neither.**

---

## 5. Recommended step order

1. **P0a** — delete the three unmounted components (+ 4 stale comments). *Dead-code confirmation pass.*
2. **P0b** — reproduce the align buttons live; fix only if it fires.
3. **S1** — read-widening (`created`/`status`/`weight`) + extract shared `LinkStateChips`/`fmtTraversed` (kills an existing 3-copy divergence). *Light inspection.*
4. **S2** — 1.3 badge + density setting.
5. **S3** — 1.4 inspector popover (+ i18n/Style-Setter the hardcoded `STAGE_META`).
6. **S4 — BLOCKING FIX:** confidence-preservation in the indexer (`search.rs:6058`). **safety-inspection.**
7. **S5** — the one new engine: body-token rewrite helper + inline link identity (B). **safety-inspection.**
8. **S6** — 1.1 re-type on top of S5. **safety-inspection.**
9. **S7** — 1.2 annotate on top of S5. **safety-inspection.**
10. **S8** — 1.5 supersedes (S6 + existing `archiveLink`). **safety-inspection.**

**Steps 1–5 are one landable, rollback-free phase; 6–10 are the write phase and should be separately Boss-gated.**

---

## 6. BOSS RULINGS — 2026-07-18 ✅

- **A → (i) BODY-TEXT IS THE WRITE.** Re-type/annotate edit the `[[…]]` token in the user's `.md` (File-Over-App). Mandatory guards: through the note model / `saveTabContent` (never raw disk RMW) · refused while `cascadeFreeze`/`isCascading()` · surfaces success/failure (no `catch {}`) · body-edit + save are ONE unit or neither.
- **Annotation slot → REUSE** the post-`|` slot with the predicate-first form `[[type::Target|reason]]` (unambiguous); no second channel.
- **B → identity via `(source_path, fold_match_key(target), link_type)`** parsed from the token under the cursor using the shared `findWikilinkAtLineOffset`; N-duplicates surfaced honestly; frontmatter links fall back to panel/inspector depth.
- **C → POPOVER anchored at the link** (one shared component, opened from all three depths). Read-widening only (`created`, `status`, `weight`).
- **D → enum `'minimal' | 'rich'`, default `'minimal'`, in Settings → Files → Living Link Lifecycle.**
- **E → DELETE** `CodeMirrorEditor.svelte`, `EditorContextMenu.svelte`, `FormattingToolbar.svelte` (~2.4k lines) after a no-dynamic-import/orphan confirmation. *(Independently verified: nothing mounts CodeMirrorEditor; NotePane builds its own `EditorView` at `:445/:608` + its own menu at `:1132`.)*
- **SEQUENCING → ONE CONTINUOUS MIGRATION** (steps 1–10 in one plan), every build Boss-tested before its commit; `safety-inspection` diff-scoped on the write steps (S4–S8).

**Correction logged:** §0.2 (`findWikilinkAtLineOffset` extraction) refactored `CodeMirrorEditor.svelte` — a file nothing mounts. The helper is sound and is genuinely used by FocusPane (11 tests green), but the "proven against the main editor" claim was false; the Boss's Ctrl-click test passed via NotePane's own handler. The real live duplicate regex is `NotePane.svelte:1140` — switched to the shared helper in S5.

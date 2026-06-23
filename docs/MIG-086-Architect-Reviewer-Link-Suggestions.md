# MIG-086 — Architect Doc: Reviewer Link Suggestions ("Connect to [[X]]")

**Status:** Architect (Phase 1) · **Date:** 2026-06-23 · **Author:** Architect agent
**Function in hand:** the Reviewer detail pane (`ReviewerView.svelte`), specifically the orphan/fragile PRESCRIPTION block where MIG-086 turns the diagnosis "connect it to a related note" into named, one-click-actionable candidates.

---

## 1. Concept (the horse)

> **When the Reviewer tells you an orphan or fragile note needs a link, it must also tell you *which* note to link — and let you create that typed link in one click — so a diagnosis becomes an action instead of a homework assignment.**

The carriage (the function) is a ranked "Connect to:" candidate list with one-click link insertion. The horse (the concept) is *closing the loop between diagnosis and remedy*: today the Reviewer says "connect it to a related note" and names no candidate, leaving the user to go hunt for the relative themselves — which is exactly the friction that lets orphans stay orphans. The cognitive purpose is **healing under-connected knowledge at the moment of diagnosis**, where the user is already looking at the note and already motivated to act.

---

## 2. The relatedness signal

### Recommendation: **FTS5 BM25 "More Like This"** over the note's top-IDF terms.

Three candidate signals were weighed against Constellation's constraints (local-first, no cloud, Rule 8 write-time derivation, 7,600-note corpus, query-time-cheap):

| Signal | Quality | Cost | Rule-8 fit | Verdict |
|---|---|---|---|---|
| **Exact-title unlinked mentions** (Obsidian/Roam/Logseq baseline) | Brittle — useless for a note whose title isn't a common phrase; the *exact* orphan case it must serve | Free | ✅ | **Insufficient alone** — the orphan is precisely the note no string points to |
| **FTS5 BM25 MoreLikeThis** (top-IDF terms → OR-MATCH → `ORDER BY rank`) | Good, *explainable* (shared distinctive terms ARE the reason) | ~2 cheap reads, O(K posting lists) | ✅ already-current FTS5 index | **RECOMMENDED** |
| **Term co-occurrence / PMI** | Term-term relatedness, not doc-doc | O(terms²) materialization | ⚠️ needs a pair matrix or per-query aggregation | Complementary *enhancement*, not the core |
| **Local embeddings** (bge-micro / MiniLM, cosine) | Best semantic; finds no-shared-vocab relatives | New model bundle + ONNX runtime + per-note vector table + back-fill + write-path maintenance | ⚠️ a whole subsystem + `/migration` | Future opt-in **Wing** only if BM25 proves shallow |

**Why BM25 MLT wins for Constellation specifically:**

- **Proven, not invented (WA#5):** it is Lucene/Elasticsearch `MoreLikeThis`, ported to the FTS5 index Constellation already keeps current. It is the same query-time concept-expansion family WA#5 already endorsed (§1D backfill lesson).
- **Rule 8 by construction:** the `notes_fts` index and `fts5vocab` dictionary are maintained write-time by the `note_meta_ai/ad/au` triggers (`search.rs:2935`). "Related" is a cheap *read*, never a boot rescan, never a precomputed similarity matrix that goes stale (the O(N²) ≈ 58M-pair / LL-XXX-OOM anti-pattern CLAUDE.md forbids).
- **No model, no bloat:** consistent with the standing decision not to ship cloud/embeddings.
- **Explainable:** the matched distinctive terms *are* the reason two notes relate — exactly the legible connection a knowledge-*formulation* tool wants the user to see. (This drives the UI's "shared terms" chips, §5.)
- **The existing co-occurrence primitive proves the cost envelope:** `read_cooccurring_terms` (`libraries.rs:3890`) already samples-MATCH + re-tokenizes for a common term in <100ms.

### Concrete FTS5 SQL shape

The note already exists in `note_meta`; let `:nid` = its rowid, `:total` = `SELECT count(*) FROM note_meta`.

**Step 1 — extract the source note's top-K distinctive terms (tf × idf).** We do NOT need to scan the corpus: per the findings (`libraries.rs:3865` doc-note), the established pattern is *re-tokenize the one note's `body_text` in-process* (mirroring `read_cooccurring_terms`'s `process_word_for_fts` at `libraries.rs:3944-3978`), then look up each term's document-frequency from the persisted ledger. Constellation already has the IDF half persisted: **`term_vocab(term, doc_count, total_count)`** (`search.rs:2816`), maintained write-time by `ctse::hooks::on_note_indexed`. So:

```
for each distinct stem in tokenize(source.body_text):           -- in-process, same 'constellation' tokenizer namespace
    tf  = count in source note
    df  = term_vocab.doc_count   (or notes_vocab.doc)           -- single indexed lookup
    if length(stem) < 4 or stem in stopwords: skip              -- minWordLen / stopword guard
    if tf < 2: skip                                             -- Lucene minTermFreq
    if df < 2: skip                                             -- minDocFreq: drop hapax (only this note)
    if df > :total * 0.5: skip                                  -- maxDocFreq: drop near-ubiquitous terms
    score = tf * (ln(:total / (df + 1)) + 1)                    -- Lucene idf
keep top 25 by score                                           -- Lucene maxQueryTerms
```

(idf computed in Rust — no SQL `log()` math-extension dependency. Defaults `minTermFreq=2`, `minDocFreq=5→here≥2`, `maxQueryTerms=25` are the canonical Lucene constants; the `df ≥ 5` floor can be a tunable, see §9.)

**Step 2 — build the OR-MATCH query, rank by BM25, self-exclude, dedup against already-linked.** Each surviving term is phrase-quoted via the existing `fts_quote_phrase` (`libraries.rs:3670`) to neutralize FTS5 operators, joined with `OR`:

```sql
SELECT m.path, m.name,
       bm25(notes_fts, 10.0, 1.0) AS rank,                     -- title 10×, body 1× (the established weighting; search.rs:8709)
       snippet(notes_fts, 1, CHAR(2), CHAR(3), '…', 12) AS snip
FROM notes_fts
JOIN note_meta m ON m.rowid = notes_fts.rowid
WHERE notes_fts MATCH :mlt                                     -- '"epistemic" OR "pramana" OR "masadir" OR ...'
  AND notes_fts.rowid <> :nid                                  -- (a) self-exclusion
  AND m.path NOT IN (                                          -- (b) dedup against already-linked (either direction)
        SELECT target_path FROM note_links WHERE source_path = :spath
        UNION
        SELECT source_path FROM note_links WHERE target_path = :spath
      )
ORDER BY rank                                                  -- ascending: most-related first (BM25 more-negative = better)
LIMIT :limit;                                                  -- default 5 (see §9)
```

Cost: one in-process re-tokenize of a single note's body + one MATCH touching only the ~25 OR'd posting lists — **not** all 7,600 notes. Sub-100ms, bounded, no boot regression. Confirm `note_links` column names (`source_path`/`target_path`) at wire time — the dedup shape is invariant regardless. Aliases: if a note's title appears verbatim in the candidate, BM25's title-10× weighting already surfaces it; an explicit alias-MATCH pass can be layered later (§9) but is not required for v1.

---

## 3. Reuse map (Predecessor: what MIG-086 stands on)

| MIG-086 needs | Reuses (file:line) | Must NOT duplicate |
|---|---|---|
| FTS5 index + dictionary | `notes_fts` schema + triggers (`search.rs:2912`, `:2935`); `notes_vocab` / `term_vocab` (`search.rs:2963`, `:2816`) | No new precomputed similarity table; no boot rebuild |
| Body re-tokenization | `read_cooccurring_terms` pipeline `process_word_for_fts` + `is_cooccurrence_boundary` (`libraries.rs:3944-4005`) | Don't fork the tokenizer — same `'constellation'` namespace (`fts5_tokenizer.rs`) |
| IDF / distinctiveness | `term_vocab.doc_count` (`search.rs:2816`) | Don't recompute df by scanning |
| BM25 ranking | `bm25(notes_fts, 10.0, 1.0) ORDER BY rank` convention (`search.rs:8709`) | — |
| Phrase-quoting terms | `fts_quote_phrase` (`libraries.rs:3670`) | — |
| Candidate "notes containing term X" pattern | `run_mentions_query` (`libraries.rs:3803`) | — |
| **Link creation** | wikilink insert via CM6 dispatch (`completions.ts:41-53`, insert strings `:97`/`:120`/`:111`) → `writeNote` (`store.ts:1083`) → `constellation_search_reindex` (`store.ts:664` → `search.rs:8196`) → `index_note` derives `note_links` (`search.rs:5153`) → `maintain_incoming_after_save` (`search.rs:8259`) | **CRITICAL: do NOT add any `create_link`/`insert_link` Rust command.** Confirmed none exists; a link is born as `[[wikilink]]` body text, then the save→reindex path derives the `note_links` row. A parallel writer would violate the Living Link single-writer invariant. |
| Reviewer detail pane | `ReviewerView.svelte` (the only component; prescription block `:338-342`, action row `:394-405`, the inert orphan "🔗 Connect" button `:397`) | Don't create a new detail component; don't create the Connect button from scratch — it exists, it's a no-op (opens editor), MIG-086 makes it real |
| Callback wiring | `+layout.svelte:6466-6497` (`onNoteClick` / `onOpenWithTab` pattern) | — |
| In-place invoke pattern | `act()` / `commitPriority` (`ReviewerView.svelte:217-239`) | — |
| Row data | `struct DueNote` (`review.rs:34-62`); TS mirror (`ReviewerView.svelte:16-32`) | No new field on `get_due_notes` — suggestions are a *separate* command keyed on `note_path` |

**The one genuine gap to build:** a *per-note* "distinctive terms → related notes" command. Its halves all exist (body tokenization, `term_vocab` IDF, BM25 MATCH) but no single command wires them. That is the net-new backend work — and the *only* net-new backend work.

---

## 4. IPC contract

**New command:** `suggest_related_notes` — registered in `src-tauri/src/libraries.rs` (alongside `read_cooccurring_terms` / `read_term_mentions`, the surfaces it reuses) and added to the `invoke_handler` generate-list in `src-tauri/src/lib.rs` (or wherever the existing `read_cooccurring_terms` handler is registered).

**Signature:**
```rust
#[tauri::command]
fn suggest_related_notes(
    library_path: String,
    note_path: String,
    limit: Option<usize>,      // default 5
) -> Result<Vec<RelatedCandidate>, String>
```

**Output struct** (new, near `CooccurringTerm` at `libraries.rs:2564`):
```rust
#[derive(Serialize)]
struct RelatedCandidate {
    path: String,          // target note path — the key for the connect action
    name: String,          // display title
    score: f64,            // |bm25| normalized 0..1 for the UI bar (optional; raw rank ok)
    shared_terms: Vec<String>,  // the matched distinctive terms — the "why" chips (explainability)
    snippet: String,       // STX/ETX-marked snippet for preview (reuse the snippet() output)
}
```

**Behavior:** runs Step 1 + Step 2 from §2. Returns `[]` (not an error) when the note has no distinctive terms (too short / all-stopword) or no candidates survive the filters — the UI renders the empty state (§5). `shared_terms` is populated by intersecting the source note's top-K terms with each candidate's body (cheap: the OR-query terms are known; mark which fired per row via the same STX/ETX snippet-scan `find_match_via_marked` uses at `libraries.rs`).

**Frontend wrapper:** `suggestRelatedNotes(libraryPath, notePath, limit)` in `src/lib/libraries/store.ts` (alongside `readCooccurringTerms` at `store.ts:2895-3036`), returning `Promise<RelatedCandidate[]>`.

**Rule 8 / hot-path compliance:** called *on demand* when the Reviewer detail pane opens an orphan/fragile note (one invoke per detail-open, not per keystroke, not on boot). No CM6 ViewPlugin invoke, no polling.

---

## 5. UI — candidates in the Reviewer detail

The candidates render **immediately after the prescription card** (`ReviewerView.svelte:342`, between `.rv-d-rx` and the priority box at `:344`) — the prescription tells you *to* connect; the list shows you *what*. New block `.rv-d-suggest`:

```
┌─ Prescription ──────────────────────────────────┐
│ Connect it to a related note — or mark it        │   (existing, kept in place)
│ deliberately standalone.                         │
└──────────────────────────────────────────────────┘

  Connect to:                                          ← reviewer.suggestLabel
  ┌──────────────────────────────────────────────┐
  │ Pramāṇa and valid knowing      [ 🔗 Link ]    │   ← name + one-click link button
  │   shared: epistemic · proof · valid            │   ← shared_terms chips (the "why")
  ├──────────────────────────────────────────────┤
  │ Aristotelian maturity gradient [ 🔗 Link ]    │
  │   shared: maturity · gradient                  │
  └──────────────────────────────────────────────┘
```

**Data flow:** on detail-open for `isOrphan(n)` OR `n.reason === 'fragile'`, fire `suggestRelatedNotes(libraryPath, n.note_path, 5)` once (guarded by `n.note_path` so it doesn't re-fire on every render; cache per selected note). Loading state shows a one-line skeleton.

**The one-click "Link" action** (the core gesture) — reuses the existing writer path entirely, NO new Rust write command:
1. Open the target note's body via the existing open/write path, **or** (preferred, no editor-mount needed) read → append → write headlessly: fetch the source note body, append `\n\n[[Target Name]]` (or `[[type::Target Name]]` if a type is chosen — see §9), via the existing `writeNote(notePath, content, origin)` (`store.ts:1083`).
2. Call `constellation_search_reindex(notePath, libraryName)` (`store.ts:664`).
3. That alone derives the `note_links` row (`index_note`, `search.rs:5153`) and bumps the *target's* `incoming_count` via `maintain_incoming_after_save` (`search.rs:8259`) — so the target drops out of the orphan lens and the *source* gains an outgoing link.
4. After success: remove that candidate from the list; if the source was an orphan and now has ≥1 outgoing link, optionally re-fetch `get_due_notes` (the `act()` pattern at `:217` already refreshes after in-place actions) so the row's diagnosis updates live.

This follows the `act()` invoke pattern (`ReviewerView.svelte:217-227`) for the reindex, but the insert is the body-write, not a link command. The existing inert orphan **"🔗 Connect" button** (`:397`) is repurposed/demoted: instead of just opening the editor, it scrolls to / focuses the new suggestion list (or, with the list always visible, becomes redundant and is removed — Boss decision §9).

**Fragile gets the list too** (today fragile falls into the `{:else}` "Reviewed" branch with no Connect button, `:401`). For fragile, the heading reads "Shore it up — connect to:" and the default link type is `derives-from` rather than `associative` (the fragile prescription literally says "Add a supporting (derives-from) link", `reviewer.rx.fragile`) — see §9.

**Empty state** (no candidates): render an honest empty line, not a fabricated suggestion (BASIC RULE) — `reviewer.suggestEmpty` = *"No strong relatives found in your Library — this note may be genuinely novel. Connect it manually, or mark it standalone."* This keeps the existing "Mark standalone" escape valve (`:403`) meaningful.

**i18n keys** (new, under `"reviewer"` in all 15 files `ar de en es fa fr he hi ja ko pt ru tr ur zh`, `en.json:3714`):

| Key | English | Notes |
|---|---|---|
| `reviewer.suggestLabel` | "Connect to:" | orphan heading |
| `reviewer.suggestLabelFragile` | "Shore it up — connect to:" | fragile heading |
| `reviewer.suggestSharedTerms` | "shared:" | chip-row prefix |
| `reviewer.suggestLinkBtn` | "Link" | the one-click button |
| `reviewer.suggestLinking` | "Linking…" | in-flight |
| `reviewer.suggestEmpty` | "No strong relatives found in your Library — connect it manually, or mark it standalone." | empty state |
| `reviewer.suggestLoading` | "Finding related notes…" | skeleton |
| `reviewer.suggestLinked` | "Linked ✓" | post-success toast/inline |

RTL: the block uses `dir="auto"` on note names (matching the component's existing convention, `:341`) and flips via the existing `:global([dir="rtl"])` rule (`:452`). Per [feedback_full_localization_everything], use native equivalents in each locale, not transliterations.

---

## 6. Predecessor lookup

**Predecessor → Replacement (in place):**

- **Where it lives now:** the orphan/fragile prescription strings — `ReviewerView.svelte:246` (`reviewer.rx.orphan` = *"Connect it to a related note — or mark it deliberately standalone."*) and `:247` (`reviewer.rx.fragile` = *"Add a supporting (derives-from) link to ground it."*) — rendered in the `.rv-d-rx` block at `:338-342`. The inert orphan "🔗 Connect" button is `:397`, routing to `onNoteClick` (opens the editor; creates nothing).
- **Where the replacement lives:** **the same place.** The candidate list slots directly beneath the prescription card (`:342`); the connect action upgrades the existing `:397` button. No new component, no relocation, no new panel. The prescription text is **kept** — it states the principle; the list makes it actionable underneath.
- **What gets cut:** the orphan "🔗 Connect" button's no-op `onNoteClick` behavior (it opened the editor and changed nothing). **What gets kept:** the prescription strings, the "Mark standalone" / "Dismiss" escape valve (`:403`), the "Open in editor / 360 / Classify" hand-off row (`:407-412`) — all untouched.

No Tauri command is removed, no settings entry dropped, no writable store retired. The only *addition* is `suggest_related_notes` (§4) living next to its kin in `libraries.rs`.

---

## 7. Invariants & risks

1. **False-positive handling.** (a) **Never suggest the note itself** (`rowid <> :nid`). (b) **Never suggest an already-linked note**, either direction (the `note_links` dedup subquery, §2). (c) **Never suggest a trivial-overlap note** — the `minTermFreq=2`, `df ≥ 2`, `df ≤ 0.5×total`, `length ≥ 4`, stopword filters kill common-word noise (the canonical Obsidian unlinked-mentions failure class). (d) **Honest empty state** over a fabricated suggestion (BASIC RULE) — if nothing clears the bar, say so.

2. **Rule 8 cost on 7,600 notes.** One in-process re-tokenize of a single note's body (the `read_cooccurring_terms` envelope, <100ms) + one MATCH over ~25 posting lists. No full-FS-walk, no boot rebuild, no precomputed matrix, no per-keystroke invoke. Called once per Reviewer detail-open. **Hard constraint:** must not regress boot or typing — measured on a 7,600-note Universe before commit (CLAUDE.md Rule 8 hard constraint).

3. **Living Link single-writer invariant.** The connect action creates a link ONLY by inserting `[[wikilink]]` body text + the existing `writeNote → constellation_search_reindex → index_note` path. **No parallel `note_links` writer.** This is the load-bearing invariant: `index_note` (`search.rs:5153`) is the sole writer that re-derives every edge from the body and preserves earned weight/traversal. A second writer would corrupt link weight/lifecycle.

4. **Link type for one-click connect.** The 8-type cognitive vocabulary is Constellation's differentiator (no surveyed PKM types its suggestions). Default proposed: **orphan → `associative`** (the neutral default — "these relate," user can retype later) and **fragile → `derives-from`** (the fragile prescription literally prescribes a derives-from grounding link, `reviewer.rx.fragile`). Whether one-click forces a type-picker vs. uses a sensible default with a later retype is a Boss decision (§9).

5. **Reindex-after-create refreshes the orphan lens.** Confirmed: inserting `[[X]]` + reindex makes X gain a backlink (`maintain_incoming_after_save`, `search.rs:8259`), X's `incoming_count` bumps, X exits the orphan lens; the *source* note gains an outgoing link and (if it was an orphan needing outgoing connection) its diagnosis updates on the next `get_due_notes` refresh. The Reviewer should re-fetch after a successful connect (the `act()` refresh pattern).

6. **Suggestion staleness within a session.** Cache suggestions per selected `note_path`; invalidate when a connect succeeds (remove the linked candidate) — don't re-fire the whole query on every Svelte re-render.

7. **i18n completeness.** All 8 new keys land in all 15 locale files in the same commit (CLAUDE.md i18n rule). Component `|| 'fallback'` defaults are belt-and-suspenders, not a substitute.

---

## 8. Phased plan (each step landable as one commit, each with a verification clause)

**§A — Backend: `suggest_related_notes` command.**
Build the command in `libraries.rs` (Step 1 in-process re-tokenize + `term_vocab` IDF; Step 2 OR-MATCH + BM25 + self-exclude + `note_links` dedup), the `RelatedCandidate` struct, register in the invoke handler.
*Verify:* a Rust unit/integration test on a small fixture corpus — assert (1) a planted related note ranks #1, (2) the source note never appears, (3) an already-linked note never appears, (4) an all-stopword note returns `[]`. `cargo test` green. (Note: per the Reproduce-First / Editor-Surface gate, static tests are NOT runtime verification for editor-lifecycle bugs — but §A touches no editor lifecycle, so unit tests suffice here.)

**§B — Frontend wrapper + read-only render.**
Add `suggestRelatedNotes` to `store.ts`; render the `.rv-d-suggest` block (heading + candidate names + shared-terms chips + snippet) beneath the prescription card — **display only, Link button disabled.** Add all 8 i18n keys × 15 locales.
*Verify (Boss-testable):* open the Reviewer, select an orphan → a "Connect to:" list of named candidates appears under the prescription, each with shared-term chips; select a note with no relatives → the honest empty state shows. No link is created yet.

**§C — The one-click Link action.**
Wire the Link button to: read source body → append `[[type::Target]]` → `writeNote` → `constellation_search_reindex` → on success remove the candidate + re-fetch `get_due_notes`. Apply the §9 link-type decision. Repurpose the inert `:397` Connect button per the §9 ruling.
*Verify (Boss-testable, full tutorial below):* clicking Link inserts the wikilink, the link appears in the note, the candidate disappears, and the row's diagnosis updates.

**§D — Fragile parity + polish + SO.**
Confirm fragile notes get the list with the `derives-from` default and the "shore it up" heading; verify RTL flip; `/simplify` the diff; session log + Orientation v-bump + MoCh + help/manual (all 15) in the same commit.
*Verify:* the Editor-Surface Gate items relevant to a body-write-then-reindex (NotePane persists; tab switch-away+return; rename-probe-pair unaffected; on-screen === disk after the connect) all pass.

### Boss test (articulated per the Testing-Instructions Rule) — lands with §C

**What this feature is and why it matters.** Constellation's Reviewer flags two kinds of under-connected notes: *orphans* (substantial notes nothing links to) and *fragile* notes (heavily-relied-on notes that themselves rest on too little). Until now the Reviewer told you *to* "connect it to a related note" but never said *which* note — so you had to go hunting. This update fills in the blank: it suggests the specific related notes already in your Library and lets you create the link in one click. A diagnosis becomes a one-click cure.

**Before you start.** You'll need a Library with at least a few orphan notes that share distinctive vocabulary with other notes. If you don't have one handy: create a note titled "Pramāṇa" with a paragraph about *epistemic proof and valid means of knowing*, and a second note titled "Masādir" with a paragraph about *kinds of proof and epistemic sources* — but do NOT link them. The shared words ("epistemic", "proof") are what the suggester keys on.

**Step 1 — open the Reviewer.** Open the left-dock Reviewer (the universe reviewer surface). *Expected:* the master list shows due notes; your unlinked "Pramāṇa" note appears flagged as an orphan (nothing links to it).

**Step 2 — select the orphan.** Click "Pramāṇa" in the list. *Expected:* the detail pane shows the diagnosis, then a **Prescription** card reading "Connect it to a related note — or mark it deliberately standalone," and — new — immediately beneath it a **"Connect to:"** list. "Masādir" should appear as a candidate, with a small "shared: epistemic · proof" row showing *why* it was suggested. *If you see no list or an empty "No strong relatives found" message instead* — the two notes don't share enough distinctive words; add more overlapping vocabulary to both and re-open.

**Step 3 — one-click connect.** Click the **🔗 Link** button next to "Masādir." *Expected, in order:* the button briefly shows "Linking…", then the "Masādir" candidate disappears from the list, and "Pramāṇa"'s diagnosis updates (it now has an outgoing link, so it's no longer a bare orphan). *If you open "Pramāṇa" in the editor* — its body now ends with `[[Masādir]]` (or `[[associative::Masādir]]` if typed-by-default was chosen). *If you open "Masādir"* — its backlinks panel now shows "Pramāṇa" linking in.

**Step 4 — verify the lens healed.** *Expected:* if "Masādir" was itself an orphan, it has now gained a backlink and should no longer appear in the orphan lens on the next Reviewer refresh. Content of both notes is otherwise untouched — only the one wikilink was added to "Pramāṇa," nothing was removed from either file.

**Step 5 — the empty case.** Select an orphan that shares no vocabulary with anything (e.g. a one-line note). *Expected:* instead of a candidate list you see "No strong relatives found in your Library — connect it manually, or mark it standalone." *This is correct behavior, not a bug* — the suggester refuses to invent a weak match.

---

## 9. Open Boss decisions

1. **Default link type for one-click connect.** Three options: **(A)** orphan → `associative` (neutral, retype later), fragile → `derives-from` (matches the fragile prescription) — *Architect's recommended default*; **(B)** every one-click opens the 8-type picker first (more deliberate, one extra click, surfaces the cognitive vocabulary Constellation is proud of); **(C)** always `associative`, never typed at connect time. The Living-Link philosophy ("links are living vessels carrying type/confidence") argues for surfacing type; one-click speed argues for a sensible default. *Recommend A, with the chip retypeable in the editor afterward.*

2. **How many candidates (`limit` default).** Field practice: Smart Connections shows ~10, but the Reviewer detail pane is compact and the goal is *decisive action*, not browsing. *Recommend 5*, with the BM25 floor doing the quality gating (a weak corpus simply returns fewer). Boss may prefer 3 (tighter) or "top relevant, capped at 8."

3. **Do fragile notes get a *different* suggestion than orphans?** Both use the same BM25 relatedness signal, but fragile notes specifically lack *grounding* (derives-from) links, not just any link. Options: **(A)** same candidate list, only the default type differs (`derives-from`) and the heading differs ("Shore it up — connect to:") — *recommended, simplest*; **(B)** for fragile, additionally bias candidates toward more-mature notes (`maturity` field on `DueNote`, `review.rs:51`) since a grounding link should rest on something solid. *Recommend A for v1, B as a future refinement.*

4. **Should this affordance also appear outside the Reviewer?** The same "suggest related + one-click typed link" is valuable in the NotePane (a "Related notes" sidebar tab) and in the 360° inspector. MIG-086 scopes to the Reviewer (the diagnosis→action loop). Options: **(A)** Reviewer-only for v1, extract the candidate-list into a shared component for later reuse (per [feedback_reuse_components]); **(B)** ship it in both Reviewer and NotePane now. *Recommend A — build the shared `<RelatedCandidates>` component so the NotePane/360 reuse is a later wiring task, not a rebuild.*

5. **(Minor) Fate of the existing inert "🔗 Connect" button** (`:397`). With an always-visible candidate list, the button is redundant. Options: remove it; or keep it as a "scroll to suggestions" focus aid. *Recommend remove* (the list IS the connect affordance) — but flagging since it's an existing user-facing element (Predecessor Rule).

---

*Architect doc complete. Next phase: Boss rules on §9, then Phase 2 (Plan) with the 4 steps of §8 as commit boundaries.*

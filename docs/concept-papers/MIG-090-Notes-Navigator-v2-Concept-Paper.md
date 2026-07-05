# The Notes Navigator, Reborn — Concept Paper (MIG-090 v2)

**Date:** 2026-07-05 · **Status:** DRAFT — awaiting Boss ratification · **Boss directive (verbatim):** *"Let's re-write the Navigator concept, based on the original 'Note Navigator'. What we should do is to upgrade it to be smart, fast, and to launch from where others ended."*
**Research:** workflow `wf_50227623-e2e` — 5 dossiers (the namesake plugin · the note-list masters · the PKM frontier · interaction science · Constellation's launchpad) + frontier synthesis. Every claim below is sourced there.

---

## 1. The concept (the horse)

> **The Navigator is where I *process* my notes, not just find them: a two-pane browser whose left rail speaks Constellation's cognitive vocabulary and whose list is fast enough to think at.**

The File Explorer answers *where things live*. Search answers *what matches a query*. Bases answer *show me a table I define*. The Navigator answers the question none of them own: **"what is in my hands right now, in what state, and what does it need next?"** — browsing as an act of formulation, not retrieval.

## 2. The paradigm — kept, deliberately

The original two-pane shape stays: **facets on the left, a rich note list on the right.** The research validates it: the namesake plugin (Obsidian's *Notebook Navigator*, 691k downloads, 5.0 stars) proved the paradigm scales to 100k+ notes and that users experience it as "the best implementation of a file explorer in any app." Its maintainer defends strict two-pane as a design boundary (closed-wontfix on merging panes). The paradigm is not the problem; the *vocabulary* and the *data path* were.

## 3. Where others ended (the launchpad)

**Table stakes** (the floor — everyone serious has these; the new Navigator must too):
virtualized list smooth at 100k+ notes · render-first boot from a persisted cache, diff-not-rescan in background · rich rows (title, one-line preview, date, pinned section, date grouping) · filter-as-you-type ≤50 ms with no debounce (local data) · declarative rule filters with any/all combinators (Apple Notes' 11-filter Smart Folders is the best shipped) · saved searches as durable sidebar objects · per-facet sort/view memory (UpNote + Evernote each hold half) · full keyboard navigation, inline rename, multi-select batch actions, drag-to-facet · native full-text search (the namesake NEVER built this — it outsources to a plugin; our FTS5 is already write-time-maintained) · 15-locale + RTL + non-ASCII-correct matching.

**Where each lineage ended — the edges we launch from:**
- **The namesake:** render-first architecture (IndexedDB→RAM write-through cache, diff-not-rescan) — the Rule-8 shape in plugin form. Its wishlist (= where it ran out of road): undo for moves/tags, batch tag manipulation, full content search, stable file IDs.
- **Apple Notes:** the most complete rule system — that never met text search, can't be saved *from* a search, carries no view.
- **Bear:** the smartest query grammar (`@task`, `@lastXdays`, `-#tag`) — that evaporates on Escape; zero persistence.
- **Tana:** the query as a first-class object, pinnable, multi-view — with the category's most honest confession: read-time evaluation caps at 2,500 nodes and collapsed queries don't run.
- **Notion/Mem:** feeds and "smart" surfaces — ranked by *recency only*, because recency is the only signal they collect; Mem's automation removes user agency and needs the cloud.
- **Superhuman/Linear (the FAST frontier):** instant = the work was already done before the interaction (local pool, optimistic writes); budgets measured honestly (<50 ms actions, % of events under target).

**The unsolved lanes (nobody, anywhere):** (1) rule-facet + full-text + saved view + own sort as ONE persisted object; (2) facets by *knowledge state* rather than administrative metadata; (3) live smart lists at scale without result caps (write-time, not read-time); (4) review/resurfacing *inside* the browser instead of a separate room; (5) a feed ranked by epistemic signals instead of recency; (6) undoable organization + durable manual order (blocked everywhere by missing stable file IDs — **Constellation's canonical filenames ARE stable IDs**); (7) AI-*suggested*, user-*ratified* facets computed locally.

## 4. FAST — the contract (numbers, not adjectives)

- **Every interaction ≤50 ms; 100 ms is the hard ceiling** (Superhuman's internal budget; RAIL). Scroll at 10 ms/frame. Filter-as-you-type streams results with **no debounce** — matching is local (the nucleo pattern: ~3M items in ~33 ms, correct non-Latin scoring).
- **The data path is the fix the Architect already prescribed:** rows come from `note_meta` / the boot snapshot (the 7,600-note corpus reads in low-millis via the covering index) — **never a filesystem walk**. Previews come from persisted NSC headlines (`note_summaries`), not file reads. Tags from `tag_counts`. The old path (~15,000 file reads per open) is deleted.
- **Virtualized list** (Rule 3), fixed row heights per density mode (sidesteps the variable-height fling problem the whole industry hand-tunes).
- **Live, not stale:** the Navigator joins the mutation-event ecosystem (`note-created`, save/rename/delete events) — the very precondition the Boss set on 2026-06-29 for re-enabling its right-click. One data domain, shared with the tree.
- **Measured before shipped:** the Superhuman method (event-timestamp → paint, % of events under budget) on the 7,600-note Universe; boot budget untouched (the Navigator stays off the boot path).

## 5. SMART — launching from where they ended

Every item below is powered by data Constellation **already maintains at write time** — no new scanning, no cloud. Ranked by cognitive value:

1. **The knowledge-state facet rail.** The left pane facets by the four questions, not folders alone: **Connection** (orphans, hubs, one-way notes — from `incoming_count`/`outgoing_count`), **Development** (stage, maturity — seed→wilting), **Altitude** (stratum), **Origin** (sources, received/discovered) — each a live count, each one click from a filtered list. *No competitor stores this vocabulary at all.* Folders and tags remain as facets — the administrative axis doesn't vanish; it stops being the only axis.
2. **The Tension queue.** A standing facet: *unresolved contradictions* — notes carrying a `contradicts` link whose confidence is still hypothesis/contested. The Five Acts put Tension at the center; no other tool can even express it as data.
3. **The Dormancy & Renewal feed.** A sort/facet ranked by epistemic decay, not recency: link weights decaying without traversal, links never traversed, thinking going stale. Constellation is the only system that *records traversal* — the whole category ranks by recency because that's all they collect.
4. **The Smart List — one object.** Rule-facet + full-text clause + view config + its own sort, persisted as a single first-class sidebar item, evaluated against the write-time index with **no result cap** (past Tana's 2,500-node wall). This is the unsolved union — and it should share bones with the Bases definition format rather than invent a second one.
5. **Review-due as a facet.** The Review Pulse's due list (already an indexed <100 ms read) becomes a rail facet + list badge + sort order — resurfacing as a property of *browsing*, not a separate room.
6. **The vocabulary rail.** The corpus's actual terms (the FTS5 dictionary the Index panel already reads) as a live, multiscript facet — browsing by what you *wrote*, not only what you tagged.
7. **The formulation timeline.** "Changed stage this month" / "stuck at hypothesis for 6 months" — from `note_state_history`, which records *what kind* of change happened, not just mtime.
8. **Suggested facets, user-ratified.** The Navigator proposes ("14 notes share concept X — make it a list?") from local index data; nothing applies without the user's yes. The empty middle between Mem's agency-removing automation and fully-manual schemas.
9. **Durable manual order + undoable organization.** The namesake's two oldest wishlist items, blocked there by missing stable IDs — our canonical filenames dissolve the blocker.

## 6. What it is NOT

- **Not a table.** Tables are Bases (Boss ruling, 2026-07-05). The Navigator is a *list you process*, not columns you configure.
- **Not automatic.** It suggests; the user ratifies. Formulation is the user's act.
- **Not a file manager.** Rows lead with the note's *standing* (state chips), not its byte size.
- **Not a second data domain.** It reads the same write-time index as everything else and hears the same mutation events — the "separate data domain" era is what killed the old one.

## 7. Open Boss decisions (before Architect/Plan)

1. **v1 scope of SMART:** which of §5 ship in the first cut? (Proposal: #1 facet rail + #5 review-due + table-stakes floor in v1; #2/#3/#4 in v2; #6–#9 later.)
2. **Batch operations:** keep multi-select batch tag/move/delete in v1 (through shared write paths), per the earlier port ruling?
3. **The second screen:** the old SS mount is back after the revert; its fate ties to PJ-068 — decide here or defer to PJ-068's reopening?
4. **Keyboard triage** (the email-grade j/k + one-key verbs + auto-advance lane — nobody fused it with notes): v1, v2, or out?

## 8. Process from here

Ratify (or edit) this concept → **Architect delta** (the v1 surface mapped onto the existing index reads; the old Architect's diagnosis §§1–5 carries over) → **Plan** (commit-sized, harness-gated where content is touched) → build. The old component keeps running untouched until the validated swap.

# MIG-094 — The Orphan/Fragile Connectivity Vocabulary — Architect

**Date:** 2026-07-06 · **Status:** **ARCHITECT — awaiting the Boss definition ruling before the Plan.** PJ-069 Step 1 (the first answer-duplication cluster). Built from workflow `wf_86d66ad7-c11` (5 agents: 2 current-state mappers + 2 WA#5 researchers + 1 synthesizer), verified against HEAD `cd18cdf3`.

**Predecessor / scope:** the orphan/fragile "answer-duplication" cluster from the PJ-069 concept paper — the one flagged as producing *contradictory answers the user can see*. This Architect maps it, researches the proven vocabulary (WA#5), and designs the "one home per capability" end-state. No code proposed until the ruling + Plan are approved.

---

## 1. The horse (the concept)

> **Each named structural-connectivity verdict a note can receive — "nothing points at me," "I float alone," "many lean on me but I rest on one thread" — gets exactly ONE authoritative implementation, computed at write time and read from `note_meta` columns, so a note's answer to a given question is identical on every surface. Surfaces that legitimately ask *different* questions keep their own named predicate. The dedup is one-implementation-per-concept, not one-number-for-all-surfaces.**

This is "one home per capability" applied to a cluster where the capabilities were never cleanly separated — and it is held in check by **Form-Aligns-To-Purpose**: we do *not* flatten genuinely-different questions into one number.

## 2. The finding — sharper than "5 definitions of orphan"

Verified against HEAD, the sites touch **three genuinely-distinct graph concepts + a substance axis + one higher-order concept.** Some divergence is *intentional*; most is *accidental drift*.

**ORPHAN class — three definitions, only some spread intentional:**
- **DEF-1 "no incoming + has substance"** = `incoming_count==0 && word_count>20`. Reviewer orphan lens (`review.rs:351` — the ONE fully-canonical site), Reviewer note-tab badge (`:555`), 360 (`inspector360.rs:236` — inbound canonical, but `word_count` from the in-memory fs-walk, not `note_meta`), Tension (`tension.rs:363` — `word_count` from `note_meta` but **inbound re-computed in-memory, alias-UNAWARE**). The word_count gate is *intentional* (exclude stubs); the inbound-source divergence is *accidental drift*.
- **DEF-2 "no incoming, no floor"** = `linkCount===0`. Sky View ring (`graphEngine.ts:2193`), CNS stat (`ConstellationSight2:1391`). **Correction verified in `cache.rs:1157`:** the sky payload's `link_count` is bumped only on the *target* row → it is **incoming degree, not total degree** as the map claimed. So DEF-2 is DEF-1 *minus the word_count floor* — a mostly-accidental divergence (Sky/CNS never chose to count 5-word stubs; they just never gated).
- **DEF-3 "fully isolated"** = `in==0 && out==0`. Search "Orphans" filter (`search.rs:6675` — rebuilds a `_incoming_targets` **temp table** per query, a direct Rule-8 violation), Collections "Unlinked" chip (`collectionChips.ts:46` — the ONE frontend site already Rule-8-clean). This is a **stricter, genuinely-different question**: a note that links *out* but has no backlinks is a DEF-1/DEF-2 orphan but NOT DEF-3. **This difference is intentional and must be preserved as a distinct named concept.**

**Sky View contradicts itself (accidental):** the `showOrphans` render-filter (`graphEngine.ts:365`) keeps a node if it is source OR target of any edge (total-degree presence), while the orphan ring (`:2193`) tests incoming-only. A note that links out with no backlinks is rendered *and* wears the orphan ring. Pure drift.

**FRAGILE/SPOF — ONE concept, FOUR copy-pasted implementations (all accidental):** `incoming>=5 && derives-from-support<=1`, at `review.rs:386`, `review.rs:555-556`, `inspector360.rs:238`, `tension.rs:436` — each computing the derives support differently (SQL subquery ×2, in-memory edge count ×2) and the `>=5` gate differently (canonical `incoming_count` vs alias-unaware in-memory). **Key:** the derives support is *already* a write-time value — `note_meta.outgoing_link_types_json` (trigger-maintained `{type:count}` map) — so `json_extract(...,'$."derives-from"')` makes the whole predicate a pure `note_meta` read. None of the four use it today.

**FALSE MEMBERS — excluded per Form-Aligns-To-Purpose (they are NOT orphan verdicts):**
- `cece/catalogers/graph.rs:97` `degree<2` — a classifier **abstain guard** ("too few classified neighbors to vote"). Rename so it never reads as "orphan."
- `livePreview.ts:1044` `not-in-sky-set` — a **graph-membership / load-state** check gating the "Open in CNS" button. Rename to a render-eligibility concept.
- `search.rs:7189` `weak_foundations` — a **link-confidence** lens (hypothesis links, weight>2.0). Out of class.
- CNS Blind Spots / `clusterEngine` gaps / `tension.rs` structural_gaps — **inter-community** under-connection, not per-note degree. Out of class.

**The user-visible bug:** the same note gets three different answers — orphan in Reviewer only if it clears 20 words; orphan in Sky/CNS regardless of length; NOT "Unlinked" in Collections/Search if it links out; and can flip between Tension (alias-unaware) and Reviewer (alias-aware) purely from alias resolution. Fragile flips across its four copies from alias-awareness and derives-count mechanism.

## 3. The research (WA#5 — proven vocabulary, before designing)

The field has exact names for every concept here, and a clear rule about when to unify vs name:

- **Graph-theory canon:** *isolated vertex* (degree 0) · *source node* (in-degree 0, may link out) · *sink/dangling node* (out-degree 0) · *leaf/pendant* (degree 1) · *hub/authority / structural-hole broker* (the fragile family).
- **The exact real-world analog of our bug:** Obsidian's **CLI `orphans` command (no-incoming) disagrees with its own Graph View (isolated-vertex)** — filed as a **defect** (forum 112063). Logseq's "All-pages orphan = empty" vs "Graph-view orphan = no-links" is the same overload (issue #5382). **"Orphan" is fatally overloaded** — the field's lesson is: do *not* ship a bare "orphan" as a shared computation label.
- **Substance is an orthogonal axis:** Logseq proves `word_count`/emptiness is independent of degree — so the shared definition should expose *degree cleanly* and let each surface AND-in its own substance filter.
- **Fragile is the OPPOSITE of orphan:** network-science *articulation point* / *bus-factor* / k-core "high-degree-but-weakly-embedded," and the Toulmin "many dependents, thin warrant" — a heavily-relied-upon note that derives from almost nothing. A genuinely separate named signal.
- **The split verdict (the whole point):** **UNIFY** when it is one concept computed many ways (the drifted orphan implementations — the field treats that drift as a bug). **NAME-DISTINCTLY** when views mean genuinely-different questions (Obsidian keeps orphan vs unresolved-link separate; linters keep *unused* vs *unreachable* separate; observability keeps metrics/logs/traces as separate named signals). Never one universal "needs-attention" number.

## 4. Options

| Option | What it does | Speed | Effort | Risk |
|---|---|---|---|---|
| **A — One canonical predicate everywhere** | Pick ONE `is_orphan` (`incoming_count==0 && word_count>20`), flatten every site to it. | Fastest to reason about | Medium | **HIGH** — erases the *intentional* differences (strict `in==0&&out==0` isolation; Sky/CNS no-floor stat). The research explicitly warns against collapsing distinct structural questions. **Rejected.** |
| **B — Small NAMED VOCABULARY (RECOMMENDED)** | One shared helper per *named* concept over `note_meta`; each surface declares which it shows. | More design | Medium-high (3 Rust helpers + 1 frontend module + ~9 re-points + fix the sky payload source) | **LOW** on cognitive integrity; **MODERATE** on the one genuine verdict-change question. Matches the field's proven pattern. |
| **C — Named vocabulary, degree-only, defer the substance axis** | Same as B but don't harmonize `word_count` at all. | Fastest vocabulary option | Medium | Leaves the DEF-1-vs-DEF-2 floor divergence documented-but-unfixed → **violates Fix-What-You-Discover** unless the Boss rules the floor a permanent per-surface lens. |

## 5. Recommendation — Option B

A fixed vocabulary of named predicates, each with ONE implementation reading `note_meta` columns (Rule-8-clean), grounded in the graph-theory canon:

1. **UNREFERENCED** (graph "source node", in-degree 0) = `incoming_count==0`. "Nothing points here." Surfaces AND-in their own substance floor (Reviewer/360/Tension add `word_count>20`; Sky/CNS don't — the floor is a *per-surface lens*, not baked in). Unifies DEF-1 + DEF-2's inbound onto the canonical column.
2. **ISOLATED / UNLINKED** (graph "isolated vertex", degree 0) = `incoming_count==0 && outgoing_count==0`. "Floats alone." Surfaces: Collections chip, Search filter. Unifies DEF-3, kills the `search.rs` temp-table re-walk.
3. **FRAGILE / SINGLE-POINT-OF-FAILURE** (articulation-point / bus-factor / weakly-warranted load-bearing claim) = `incoming_count>=5 && COALESCE(json_extract(outgoing_link_types_json,'$."derives-from"'),0)<=1`. One implementation replacing the four copies.

The non-cognitive thresholds (cataloger `degree<2`, livePreview membership) are **renamed out** of the orphan family. Per the research, the bare word **"orphan"** stays a user-facing synonym ONLY on the surface matching users' Graph-View mental model (ISOLATED); precise labels ("Not linked to", "Only linked out", "Loose end") elsewhere.

## 6. The shared-helper design

**Rust — three helpers, reading ONLY `note_meta` columns (zero fs-walk, zero `note_links` subquery, zero in-memory degree map):** `is_unreferenced` = `incoming_count==0`; `is_isolated` = `incoming_count==0 && outgoing_count==0`; `is_fragile` = `incoming_count>=5 && COALESCE(json_extract(outgoing_link_types_json,'$."derives-from"'),0)<=1`. Provided as **both** a WHERE-clause fragment (scan sites) and a `fn(&NoteMetaRow)->bool` (single-note sites) so the two call shapes share one definition. **Frontend** — one predicate module mirroring `collectionChips` (`isUnreferenced`/`isIsolated`/`isFragile`), sourced from `note_meta` facts (Collections is already the template).

**Per-surface re-pointing + verdict-parity (WA#4):**
- `review.rs` orphan + fragile lenses + note-tab badge → helpers. **UNCHANGED** (already canonical; fragile unchanged *iff* the JSON-map count == the subquery count — verified at build).
- `collectionChips.ts` Unlinked → `is_isolated`. **UNCHANGED** (already reads the columns).
- `inspector360.rs` orphan (`word_count`→`note_meta.word_count`) + SPOF (`out_derives`→JSON map). **VERDICT CHANGE** (aligns 360 with Reviewer — a correctness win, but a displayed change).
- `tension.rs` orphans + SPOF → helpers. **VERDICT CHANGE (intended)** — inbound goes alias-UNAWARE → alias-AWARE; alias-linked notes flip.
- `search.rs:6675` Orphans filter → `is_isolated` (drop the temp table). **VERDICT CHANGE** — alias-awareness flip (+ the Q1 ruling if it moves to UNREFERENCED).
- Sky View ring + filter + CNS stat → one UNREFERENCED source; **re-source the sky payload** by JOINing `note_meta.incoming_count/outgoing_count` in `cache.rs` so Sky/CNS stop carrying a parallel degree tally. Meaning preserved; reconciles Sky's internal self-contradiction.
- `cece/graph.rs` + `livePreview.ts` → **renamed** out of the orphan family, behavior identical.

## 7. Invariants (must not break)

1. Each surface's cognitive purpose preserved (dedup = one impl per named concept, NOT one number).
2. `note_meta.incoming_count` stays canonical (DISTINCT source, alias-aware, structural/PJ-065 lane excluded, archived excluded). No helper re-derives inbound any other way.
3. No perf regression on 7,600+ notes: every predicate is an O(1) column read or indexed scan — no read-time fs-walk, no per-query temp-table, no in-memory full-graph map, no `note_links` correlated subquery.
4. The fragile derives-count from JSON must be occurrence-count-equivalent to the current `note_links COUNT(*)` — **verified at build before any swap**, or the `<=1` boundary shifts silently.
5. Sky's render-filter and orphan-ring read the SAME source after the change.
6. Every displayed-verdict change is Boss-approved BEFORE the swap and help/manual updated in the same commit (WA#4 + Testing Instructions Rule) — no silent flip of what a user triages.
7. No surface labels a non-cognitive threshold "orphan" after the rename.

## 8. The Boss ruling needed (before the Plan)

1. **One orphan concept or two?** Is "links out but zero backlinks" an orphan (UNREFERENCED = `incoming_count==0`) or NOT (only ISOLATED = `in==0 && out==0`)? *Recommendation: keep BOTH as separately-named concepts* (the research is emphatic; they are different triage questions).
2. **The word_count floor:** permanent per-surface lens (Reviewer/360/Tension only), or should Sky/CNS also gate so a 5-word stub reads identically everywhere? *Recommendation: per-surface lens (substance is an orthogonal axis).*
3. **Verdict changes:** approve the four correctness fixes (360 word_count source; Tension → alias-aware; Search → alias-aware; Sky payload re-sourced) even though they change what those surfaces list?
4. **Naming:** the internal computation labels are graph-theory-precise (UNREFERENCED / ISOLATED / FRAGILE); the *user-facing* labels get the full 15-locale native-first treatment at build. Any label you want fixed now?

## 9. Migration path (each landable as one commit with a verification clause)

- **P0 — Harness:** a read-only diagnostic that logs each surface's *current* verdict + disagreements on the 7,600-note universe. The before-snapshot for verdict-parity. (Reproduce-First — this class of change must be measured, not assumed.)
- **P1 — Helpers dormant:** land the 3 Rust helpers + frontend module. **Build-gate:** a test asserting `is_fragile` via JSON map == via the old `note_links` subquery on a real snapshot (zero mismatch, or characterized + Boss-ruled).
- **P2 — No-change swaps:** re-point Reviewer + Collections (byte-parity; harness shows zero diff).
- **P3 — Verdict-change Rust (Boss-approved):** 360 + Tension; help/manual in the same commit; harness shows the diff == exactly the approved flip set.
- **P4 — Search filter:** drop the temp-table re-walk; measure perf ≤ prior.
- **P5 — Sky internal + payload re-source:** reconcile ring/filter; JOIN `note_meta` into the sky snapshot; no boot/frame regression.
- **P6 — Rename false members + close:** cataloger + livePreview renamed; dead `buildSkyData` fallback removed if confirmed; SO#6 orientation v-bump + session log + help/manual.

## 10. Open risks

- Q1 gates P4/P5 — if the Boss collapses UNREFERENCED and ISOLATED, re-read the P0 harness for the new verdict set.
- The `outgoing_link_types_json` derives semantics are *assumed* occurrence-count + active-only + structural-excluded — **P1's parity test is the gate**; if it fails, a dedicated maintained column may be needed.
- The sky payload re-source (P5) must not regress the ~17s SKY read (MIG-079 §C.2d) — measure; fall back to reconciling only the two internal sites if it does.
- Tension/Search alias flips must be shown in a tutorial-framed before/after or they read as a regression.
- Whether `supports` should count as a "foundation" link alongside `derives-from` in the fragile predicate is a latent design question — **out of scope** (preserve current behavior); flag for a future concept paper, do NOT silently widen.

---

**This session's deliverable ends at this Architect + the §8 ruling.** Plan follows ratification.

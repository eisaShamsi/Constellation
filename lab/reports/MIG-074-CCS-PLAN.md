# MIG-074 — CCS, the Constellation Circulatory System (Plan / Phase 2)

**Status: PLAN (Phase 2) — produced 2026-06-10. Awaiting Eisa's approval → then the build cascades
(Plan-Approval = Build-Approval), stopping only at the ★ Boss-test clauses.**
**Architect: `MIG-074-CCS-ARCHITECT.md` — RATIFIED 2026-06-10, all Q1–Q8 as recommended.**

## Locked decisions (Eisa, 2026-06-10)

- **Q8/D1 — Option B**: extend `link_stats_cache` with additive `ccs_*` keys inside the existing
  background recompute; the panel reads ONE snapshot IPC; drill-downs are bounded on-demand queries.
- **Q2/D2 — 5-tier usage census** (fresh · emerging · established · load-bearing · stale) for
  "The Life of a Connection" (new `ccs_tiers` key — the SQL port of `linkLifecycle()`, store.ts:2320).
- **Q3 — YES**: repair the shared 6-stage `lifecycle` key's dormancy bucket to the **derived** definition.
  KH's census numbers change honestly (dormancy stops reading ~0; growth/maturity stop hiding stale links).
- **Q1**: dock button **adjacent to CNS** (directly after it); **ECG pulse-waveform** inline-SVG icon;
  gate `enabledFeatures.ccs !== false` + a Settings → Plug-Ins entry (the Cataloger pattern).
- **Q4**: per-register trends **deferred to v2** (no history table).
- **Q5**: KH coordination = **mutual deep-links** (CCS header → KH overlay; KH header → CCS). No embedding.
- **Q6**: `crossLibrary` + `broken` LinkDashboard sections **drop with the panel**, documented in help/manual.
- **Q7**: `detect_tensions` **untouched** (queued for the future CNS MIG).
- **Scope trims inside the ratified frame** (stated, not silent): v1 write surface = **unarchive only**
  (set-confidence stays in its existing homes; CCS adds it in a later iteration if lived-in use asks);
  facts-rest (I6) ships as **invitation framing** (no warnings/red badges on Cooling/Conviction) — the
  stratum-scoping *filter* is deferred alongside trends; the `data-style-target` Setter category is deferred
  (no Core Plug-in has one today — the Cataloger ships without; CCS uses pure theme vars so every Style
  applies). Each is a one-commit add later; none blocks the concept.

## The new cache keys (§A's contract — all additive, computed in the SAME single background pass)

| Key | SQL semantics (status + idle are NULL-safe: unparseable `last_traversed` counts as warm — the store.ts:2324 least-destruction principle) | Payload |
|---|---|---|
| `ccs_living` | active ∧ tc>0 ∧ idle≤90d, ORDER traversal_count DESC, last_traversed DESC, LIMIT 20 | `{total, rows}` |
| `ccs_load_bearing` | active ∧ tc>0 ∧ idle≤90d, ORDER weight DESC, LIMIT 20 | `{total, rows}` |
| `ccs_cooling` | active ∧ tc>0 ∧ idle>90d, ORDER last_traversed ASC (coldest first), LIMIT 20 | `{total, rows}` |
| `ccs_contested` | active ∧ confidence='contested', ORDER weight DESC, LIMIT 20 | `{total, rows}` |
| `ccs_tiers` | census over status≠'archived': fresh (tc=0) · emerging (1–2 ∧ ≤90d) · established (3–9 ∧ ≤90d) · load-bearing (≥10 ∧ ≤90d) · stale (tc≥1 ∧ >90d) | `{fresh, emerging, established, load_bearing, stale}` |
| `ccs_retired` | archived, ORDER last_traversed DESC, LIMIT 20 + total | `{total, rows}` |

`rows` reuse the existing `FormulationInsight` shape (source/target/type/weight/confidence/tc/last_traversed/
library) — no new struct family. **Q3 repair** in `compute_lifecycle_distribution` (search.rs:5531):
`dormancy = status='dormant' OR (active ∧ tc>0 ∧ idle>90d)`; `growth`/`maturity` gain `∧ idle≤90d` —
buckets stay mutually exclusive, totals preserved. Existing 6 keys' shapes are otherwise **frozen** (I9).

---

## §A — Backend: ccs_* keys + Q3 dormancy repair + the CCS snapshot IPC (one commit; not Boss-visible)

1. Extend `recompute_link_stats_cache` (search.rs:5633) with the 6 keys above (same transaction, same
   dedicated background connection).
2. Apply the Q3 bucket repair in `compute_lifecycle_distribution` (NULL-safe `julianday` idle predicate).
3. New IPC **`constellation_ccs_snapshot`** — same SWR mechanics as the KH snapshot (shared helper): reads
   `stats` + `lifecycle` + the 6 `ccs_*` keys; **its own** key-completeness check (missing ccs keys →
   `{ready:false}` + background populate + `kh-snapshot-ready`); the KH IPC keeps its own 6-key check —
   **KH can never regress to not-ready because of CCS keys**. Register in `lib.rs`.

**Verify (me, on a COPY of the 1.7 GB universe DB):** `cargo check` 0 errors; recompute populates 12 keys;
`constellation_ccs_snapshot` returns in <50 ms with no `note_links` scan; lifecycle census **sum unchanged**
vs pre-repair (links only move between buckets; spark+birth byte-identical; dormancy > 0 iff stale links
exist); the KH snapshot's 6 payloads byte-shape-identical; recompute duration measured before/after
(bounded growth — same pass, +6 indexed aggregates).

## §B — Frontend: the CCS surface + dock + the seven registers (read-only) + i18n ×15 ★ Boss test

1. **`src/lib/components/CCSView.svelte`** — full-page surface on the Cataloger overlay pattern: header
   (title `ccs.title` = "Constellation Circulatory System (CCS)" mirroring CNS's ×15 title pattern +
   **"Knowledge Health →"** deep-link + close), then the seven registers in CCS §6 order, each titled with
   its *question* phrasing. Lists render source → **`LinkTypePill`** → target + weight/traversals/last-walked
   meta; The Acts of Inquiry orders by the **Link-Type Registry** with `relates`/`associative`/untyped
   aggregated into an **"Open inquiries"** line (guardrail 1); Conviction & Doubt = 4 confidence bars +
   the contested list; The Life of a Connection = the 5-tier census bars. Invitation copy throughout (I6);
   honest `computing` (first-population) and per-register empty states. Row click opens the source note —
   **never fires `_link_traverse` (I2b)**.
2. **Data flow (the KHD P3 pattern):** register the `kh-snapshot-ready` listener BEFORE the first fetch;
   ONE `constellation_ccs_snapshot` invoke; stale renders instantly and updates in place; unlisten +
   interval cleanup on destroy (Rule 4). Zero IPC while closed (LL-022).
3. **+layout wiring:** `showCCS` + sticky `ccsEverOpened` (reset on universe switch, like the others);
   `.ccs-overlay` mount next to the Cataloger's; dock button **directly after the CNS block**
   (+layout:4880–4891) with the ECG-waveform inline SVG, tooltip `ribbon.ccs`, gate
   `enabledFeatures.ccs !== false`; mutual-exclusion with the other full-page surfaces per the existing
   dock-onclick pattern; a command-palette entry (the KH precedent, +layout:1826); the Settings → Plug-Ins
   list gains the CCS entry (SettingsModal:148-150 region).
4. **KH side of Q5:** `KnowledgeHealthDashboard` header gains "Open CCS →" (an `onOpenCcs` prop wired in
   +layout: closes the overlay, opens CCS).
5. **i18n:** full `ccs.*` namespace (~35 keys: title, 7 register titles + questions, meta labels, empty
   states, computing, openInquiries, actions, KH-link) in **all 15 locales in this same commit**
   (merge-script + per-file delta verification; CRLF en/ar, LF others); RTL-correct rows (`dir="auto"` names,
   `text-align: match-parent` lesson); pure theme vars (every saved Style + dark/light applies).

**Verify (me):** `svelte-check` 0 errors; 10-char rapid-type check (Rule 7); locale deltas machine-verified.
**★ Boss test Stage 1 (tutorial at the stop):** open CCS from the new dock button beside CNS — first open
may say "computing" once, then seven registers render instantly on every later open; numbers agree with
Knowledge Health where they overlap (total/type/confidence); EN + AR passes (RTL rows correct); the KH
overlay still opens instantly and now shows a **real dormancy count** (the Q3 repair — expected change).

## §C — Retired Reasoning actions + the live drill-down ★ Boss test

1. "Show all" in Retired Reasoning loads the full archived list via the existing `listArchivedLinks` IPC
   (explicit user action — the I1 carve-out); each row gets **Restore** → `unarchiveLink` → the row leaves
   the list locally; counts true-up on the next SWR refresh (single-link writes ride SWR, the MIG-073 P3
   as-built propagation).

**Verify (me):** archive→restore round-trip on a test link preserves all 8 properties (I8).
**★ Boss test Stage 2:** restore an archived link from CCS → it disappears from Retired Reasoning and
reappears in the note's Backlinks/Outgoing; within ~2–3 minutes (or one reopen) the register counts reflect it.

## §D — Retire the Link Dashboard + re-point the MIG-007 hub (one atomic commit) ★ Boss test

**Predecessor Lookup re-confirmed in the session log immediately before this commit** (the §2.4 map).

1. Remove the right-sidebar **`links`** tab button + the `LinkDashboard` mount (+layout:6472) + the import
   (+layout:100); delete `LinkDashboard.svelte`; sweep every `panelPlacements.links` reference (tab strip,
   placement Settings UI) — a leftover stored value must be inert, never a crash.
2. Re-point the hub: the listener (+layout:2300) becomes `constellation:open-ccs` → opens CCS; the dispatch
   site (SettingsModal:1391) renamed in the same commit; button label → `settings.links.ccsBtn`
   ("Open the Circulatory System →") ×15. The hub never dangles (I11).
3. Drop `linkDashboard.*` ×15 (machine-verified −N/+0/~0 per file); add `settings.links.ccsBtn` ×15.
4. Check `allLinks`/`allNotes` prop plumbing for other consumers before trimming (the /simplify pass
   re-verifies).

**Verify (me):** repo-wide grep: zero `LinkDashboard` / `open-link-dashboard` references; svelte-check 0.
**★ Boss test Stage 3:** the right sidebar no longer shows the old Links tab; Settings → Links → the hub
button now opens CCS full-page; archived links remain reachable (CCS → Retired Reasoning). The cross-library
and broken-links lists are retired with the panel (documented — Q6).

## §E — Docs + PCS

EN help topic **"Constellation Circulatory System"** (seeded from Concept §5 pitch) + User Manual section
(replaces the Link Dashboard section; notes the Q6 drops and the Q3 dormancy-number change); orientation
**v2.66** rows/preamble; session log; MoCh; milestone tag `milestone/mig-074-ccs` + ZIP after the audit
passes. The 14-language help translation is the batched follow-up (per the standing translation-debt ruling —
no piecemeal patching).

## §F — /simplify + Phase-4 audit (Migration Rule)

- `/simplify` over the full MIG-074 diff range.
- Three parallel audit agents: **invariants** (I1–I14 — incl. I2b no-observer-effect, I9 KH-frozen,
  Q3 bucket exclusivity/total-preservation, Rule-2/Rule-4 on the new component); **drift** (the now-two
  listeners of `kh-snapshot-ready` are *intended* — flag any third; single emitter; single IPC caller;
  locale parity 15/15; no orphaned `linkDashboard.*`/`panelPlacements.links` stragglers); **migration path**
  (existing-universe first boot: 6-key cache → CCS `{ready:false}` → self-heals; pre-MIG-074 binary on a
  post-MIG-074 DB: extra rows inert; mid-recompute interrupt: per-key INSERT OR REPLACE self-heals;
  rollback: revert §D→§A + optional `DELETE FROM link_stats_cache WHERE stat_key LIKE 'ccs_%'` — KH
  untouched at every point).
- Boot/typing/IPC measured before/after on the 7,661-note / 234k-link universe (I13).

## Risk register (from Architect §4/§6, with mitigations)

| Risk | Mitigation |
|---|---|
| Q3 changes KH's visible census | Boss-test tutorial states the expected change up front; sum-preservation verified in §A |
| CCS keys missing on existing universes | per-key ready logic + self-healing populate (the Scenario-2 pattern, extended) |
| KH regression via shared plumbing | KH IPC + its 6 key shapes untouched; its ready-check ignores ccs_* keys; audited in §F |
| `panelPlacements.links` leftovers | §D sweep checklist + drift agent |
| Cold-read on drill-downs | only explicit user actions hit live queries (bounded, indexed); registers always render from cache |
| Recompute pass grows | +6 indexed aggregates in the same transaction; duration measured in §A on the real-size DB |

## Rollback

Every § is one revertible commit; the cache keys are derived and droppable; KH is untouched throughout, so
rollback at any depth restores today's behavior exactly (worst case: also delete the `ccs_%` rows — inert
either way).

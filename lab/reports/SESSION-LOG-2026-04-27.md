# Session Log — 2026-04-27

---

## §89 — Backlinks/Outgoing dedup + annotation-redundancy suppression + M badge confirmation + orientation v1.4

**User-visible bugs closed:**

1. **Source-note duplication in Backlinks panel.** A note with both
   `[[X]]` (regular) and `[[X|supports]]` (typed) targeting the same
   active note rendered twice — once with no badge, once with the
   `supports` badge. Now grouped by source path into ONE row whose
   `linkTypes[]` array carries both the regular-and-typed badges.
   The LINKED MENTIONS count reflects unique source notes, matching
   the user's mental model of "how many notes engage with this one".
2. **Same source-note duplication in Outgoing Links panel** when a
   note targets the same other note via both regular AND typed
   wikilinks. Now grouped by target.
3. **Redundant annotation prose under the typed-link badge.** The
   parser stores the typed-link keyword in BOTH `link_type` and
   `annotation` slots when the user writes `[[Note|supports]]`.
   The badge reads from one, the annotation prose reads from the
   other → "supports" was rendered twice. `displayAnnotation` now
   suppresses the annotation when it's exactly the typed-link
   keyword. Real prose annotations (`[[Note|supports my health goal]]`)
   are unaffected — they don't match `KNOWN_LINK_TYPES`.

**Implementation:**

- `src/lib/libraries/store.ts` — `dedupeBySource<T>(rows, keyFn)`
  helper. `getBacklinks` calls it with `r => r.path`; `getOutgoingLinks`
  with `r => r.target`. `displayAnnotation(l, displayedType)` helper
  added beside `displayLinkType`. Each grouped row exposes a new
  `linkTypes: string[]` field.
- `src/lib/components/BacklinksPanel.svelte` — renders `bl.linkTypes`
  with `{#each ... as lt (lt)}`; falls back to wrapping the legacy
  single `linkType` for any non-deduped caller.
- `src/lib/components/OutgoingLinksPanel.svelte` — same pattern.
- `src/routes/+layout.svelte:1044-1045` — inline state types updated
  to allow the new `linkTypes?: string[]` field.

**Badge taxonomy update**: M = Mutual link confirmed by project owner
2026-04-27. Moved out of Unresolved into the link-relationship table
in `docs/Badge-Taxonomy.md`. **No more pending badge letters.**

**Doc bump**: `docs/Constellation Orientation & Onboarding v1.4.md`
written as new file alongside v1.0/v1.1/v1.2/v1.3 per SO #6
versions-stack rule. v1.4 §13.1 reflects the M = Mutual addition;
§17 unknowns reduced (M removed); §88 + §89 noted in changelog.

**Verified by user 2026-04-27 ~17:30**: Apple Tree Fruit's Backlinks
shows 3 LINKED MENTIONS, Lunch Plan once with the `supports` badge,
no italic redundancy.

**Other typed links** (contradicts / causes / exemplifies / generalizes
/ derives-from / part-of / associative): identical code path —
`KNOWN_LINK_TYPES` covers all of them, so the same dedupe + annotation
suppression applies. Spot-test with any of them yields the same
single-row result.

**Open / next**: SecondScreenPage.svelte buildSkyData still alias-blind
(2-arg form) — pending. Snapshot-path bypass forensics — pending.

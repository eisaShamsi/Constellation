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

---

## §90 — Unlinked Mentions: strip wikilinks + frontmatter-title label

**User-visible bugs closed (item 6 of the option-(e) backlog):**

1. Apple Tree Fruit's UNLINKED MENTIONS row showing
   `20260426T140940Z_NOTE_1...` with preview
   `My eating choices [[Apple Tree Fruit|supports]] my health go...`
   was a false positive. The scanner was matching "Apple Tree Fruit"
   inside the typed wikilink markup because the legacy
   "skip if `[[NoteName]]` substring is present" check matched only
   the bare wikilink form, missing every typed (`|supports`) /
   alias / embed form.
2. The same row showed the canonical filename instead of the human
   title because source_name fell straight to `path.file_stem()`.

**Implementation:**

- `src-tauri/src/libraries.rs::scan_unlinked_recursive` now
  strips ALL wikilinks (`!?\[\[[^\]]*\]\]`) from content before
  applying the plain-text word regex. Side-effect: every form of
  wikilink — regular, typed, alias-target, embed — is correctly
  excluded from "unlinked" classification.
- Source label now reads `extract_frontmatter_title(content)` first,
  falls back to `file_stem()` only when title is missing. Same helper
  the rename path (libraries.rs:838) already uses, so canonical
  notes display "Lunch Plan" instead of `20260426T140940Z_NOTE_11B4`.
- No frontend changes required — `BacklinksPanel.svelte`'s Unlinked
  section consumes the corrected data verbatim.

**Verification (2026-04-27 ~18:00):** user reported "Pass" for both
the no-false-positive and human-title-label expected behaviors.

**Doc bump**: `docs/Constellation Orientation & Onboarding v1.5.md`
written alongside v1.0..v1.4 per SO #6 versions-stack rule. v1.5
changelog at top notes §90 closure of item 6.

**Backlog items remaining** (unchanged from v1.4 except item 6):
- MIG-007 — Links Settings tab.
- Constellation Map perf / tooltip / search backlog
  (canonical-filename label there is still pending — Map uses a
  different code path, not `scan_unlinked_mentions`).
- SecondScreenPage.svelte buildSkyData alias threading.
- Snapshot-path bypass forensics.
- MIG-005 Steps 4-8 (alias-aware tension/inspector360/LinkDashboard).

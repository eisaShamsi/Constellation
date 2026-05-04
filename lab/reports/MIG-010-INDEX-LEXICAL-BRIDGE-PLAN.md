# MIG-010 — Phase 2: Build Plan

**Companion to**: `MIG-010-INDEX-LEXICAL-BRIDGE-ARCHITECT.md`
**Phase**: 2 (Plan) → cascades to Phase 3 (Build) on Boss approval
**Build steps**: 5 commits + 1 simplify checkpoint + 1 audit doc

---

## §0 · Reading guide

Each Build step lands as one commit with the format `§NNN (MIG-010 §Build.N) — short title`. Each step has a **verification clause** — what observably proves the step worked. Boss-testable steps pause for tutorial-style test instructions per the Testing Instructions Rule; internal-only steps cascade.

The plan is intentionally small (5 build steps, ~half-day end-to-end) — the Architect deliberately picked the lowest-surface options at every fork (3.A, 3.D, 3.G).

---

## §Build.1 — Promote bridge helpers to `pub(crate)`

**Surface**: `src-tauri/src/search.rs`
**Change**: change visibility on three items from private to `pub(crate)`:
- `expanded_match_query` (~line 2662)
- `LexicalExpansion` struct (~line 2637) with both fields
- `find_bridge_lemma_in_snippet` helper (~line 2689)

No semantics change. Compile-only commit.

**Verification**: `cargo check` clean. The existing search-side tests at search.rs:4820+ still pass (`cargo test --lib --no-run search`).

**Boss-testable**: No — internal visibility change. Cascade to §Build.2.

---

## §Build.2 — Extend `read_term_mentions` with optional Bridge expansion

**Surface**: `src-tauri/src/libraries.rs`

**Changes**:
1. Add parameter: `expand_cross_language: Option<bool>` (defaults to `false`).
2. Extend `IndexMention` struct with `via_lemma: Option<String>` (None on direct hits, Some on cross-language hits with the matched bridge lemma).
3. When `expand_cross_language == Some(true)`:
   - Call `crate::search::expanded_match_query(&term)` (with the term first normalized through the same path search.rs uses).
   - If `Some(expansion)` returned: use `expansion.match_expr` as the FTS5 MATCH clause; per row, scan the snippet via `find_bridge_lemma_in_snippet(&snippet, &expansion.bridge_terms_lower)` and populate `via_lemma`.
   - If `None` returned (term not in corpus or no real OR-expansion): fall back to today's exact-phrase path. `via_lemma` is None on every row.
4. When `expand_cross_language` is `None` or `Some(false)`: behaviour identical to today (one-line guard at top of function).

**Tests** (added in same commit):
- `read_term_mentions_no_expand_returns_exact_only` — toggle off + a Bridge-eligible term returns no `via_lemma`.
- `read_term_mentions_expand_out_of_corpus_returns_exact_only` — toggle on + "Xzyqwop" returns exact-only.
- `read_term_mentions_expand_in_corpus_returns_via_lemma` — toggle on + "tree" against a fixture with an Arabic mention returns `via_lemma: Some("شجرة")` on that row.

**Verification**: `cargo test --lib libraries::tests::read_term_mentions_*` all pass. Manual compile check via `cargo check`.

**Boss-testable**: No (no UI surface yet). Cascade to §Build.3.

---

## §Build.3 — Settings: new "Index" section + `indexExpandCrossLanguage` key

**Surface**:
- `src/lib/libraries/store.ts` — `AppSettings` interface (line ~2826) gets new field `indexExpandCrossLanguage: boolean`. `DEFAULT_SETTINGS` (line ~3010) sets it to `false`.
- `src/lib/components/SettingsModal.svelte` — new section `'index'` added to the section list; new branch under the `{#if/else if}` chain (~line 902+) rendering a single labeled toggle.
- `src/lib/i18n/en.json` + 14 other locales — new keys:
  - `settings.index.title` — "Index"
  - `settings.index.expandCrossLanguage.label` — "Expand mentions cross-language"
  - `settings.index.expandCrossLanguage.description` — "When you click a term in the Index panel, also surface notes containing its translations from the Lexical Bridge. Off by default — turning it on adds a 'via {lemma}' badge to each cross-language match so you can always tell which mentions are direct vs. bridged."
  - `indexPanel.viaLemma` — "via {lemma}" (string template, used in the badge)

**Verification**: open Settings → "Index" section appears; toggle renders; flipping it persists across modal close/open and across app restart (manual). `notifySettingsChanged()` fires on flip (verified by Boss test in §Build.5).

**Boss-testable**: Yes — Boss verifies the toggle visually exists and persists. **Tutorial test instructions emitted at this step.**

---

## §Build.4 — Wire IndexPanel: pass setting to IPC + render badge

**Surface**:
- `src/lib/libraries/store.ts` — `readTermMentions` wrapper accepts `expandCrossLanguage?: boolean` and forwards it; `IndexMention` TS type gets `viaLemma?: string` (note: Tauri auto-converts `via_lemma` snake → camel `viaLemma`).
- `src/routes/+layout.svelte` — the `loadMentions` callback passed to `<IndexPanel>` reads `$appSettings.indexExpandCrossLanguage` and passes it through.
- `src/lib/components/IndexPanel.svelte` — mention row rendering: when `mention.viaLemma` is present, append a small chip after the note name reading "via {lemma}" using `$t('indexPanel.viaLemma', { lemma: mention.viaLemma })`. Visual style: muted color, small, dir-auto for RTL safety.

**Verification**: with toggle off, click a term — no badges. Flip toggle on, click an Arabic term ("شجرة") that has English equivalents in your library — English notes appear in the mentions list with "via tree" (or similar) badge.

**Boss-testable**: Yes — full end-to-end Index panel verification. **Tutorial test instructions emitted at this step.**

---

## §Build.5 — `/simplify` checkpoint

Run three review agents on the §Build.1 → §Build.4 diff: code-reuse, code-quality, efficiency. Address Tier 1 + Tier 2 findings inline, surface Tier 3 for Boss.

**Verification**: agent reports clean OR all Tier 1+2 findings resolved + Tier 3 explicitly deferred with rationale.

**Boss-testable**: No — internal review pass. Cascade to Audit.

---

## §Audit — Phase 4 (separate doc: `MIG-010-AUDIT.md`)

Three audit lenses:
1. **Invariant verification** (I1–I11 from Architect) — each checked against shipped code + Boss test result, table-formatted.
2. **Drift audit** — compare shipped to plan; flag any unintended deviation.
3. **Code surface check** — Rust diff size; frontend diff size; locale coverage check (15/15); migration-path check (no schema change, no on-disk format change, settings additive only).

**Verification**: all 11 invariants PASS; no unintended drift; migration path confirmed safe; closure declared.

---

## §X · Boss approval gates

| Gate | What Boss approves | When |
|---|---|---|
| **G1 — Plan approval** | This plan as written; treats §Build.1 → §Build.5 + §Audit as one autonomous cascade per the Plan-Approval-=-Build-Approval rule. | Now (Phase 2 → Phase 3). |
| **G2 — Settings UI sanity (§Build.3)** | "Index" tab appears, toggle visible, label readable in Boss's Arabic interface. | After §Build.3 commit lands. |
| **G3 — End-to-end (§Build.4)** | Full toggle-off → toggle-on → cross-language mentions surface with correct "via {lemma}" badges. RTL clean. | After §Build.4 commit lands. |
| **G4 — Closure** | Audit doc verified; MIG-010 marked closed in memory + orientation. | After §Audit. |

Stops at G2 + G3 only. §Build.1, §Build.2, §Build.5 cascade without pause.

---

## §Y · Out-of-scope (deferred to follow-on MIGs)

- Migrating the term-exclusion list from `localStorage` (current IndexPanel-internal) into the new Settings → Index section. Logical fit but separate scope.
- Per-term ad-hoc expansion (right-click an Index term → "expand this once"). Boss explicitly chose global toggle, not per-term.
- Domain-pack-aware expansion (when domain packs ship per the M11 follow-on plan).
- Synonym-edge expansion (when route-(a) cross-concept synonyms ship).

These are real features but each composes on top of MIG-010, not inside it.

---

**Phase 2 closes here. Awaiting Boss's G1 approval — then cascade through §Build.1 → §Audit autonomously, stopping only at G2 (after §Build.3) and G3 (after §Build.4) for Boss test.**

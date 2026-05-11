# V3-§10 — User-Facing Surfaces — Plan Document (Option C — Full)

**Date:** 2026-05-11  
**Author:** Claude (Plan phase per /migration workflow)  
**Status:** awaiting Boss approval to begin Build phase  
**Predecessor:** V3-§10 Architect doc, Boss picked **Option C — Full** (Settings + en/ar i18n + EN help topic + EN User Manual chapter + 13-locale i18n backfill + 14-language help topic translations + 14-language User Manual chapter translations)

Plan-Approval-Equals-Build-Approval applies once Eisa approves: I cascade through phases autonomously, stopping only at user-testable verification clauses (Gate 3 Boss-test), genuine architectural surprises, or plan completion.

---

## §0 — Scope summary

Seven build phases (A → G), structured to allow checkpoint commits at meaningful boundaries. Total estimated effort ~12-15 hrs of agent time + Eisa's Gate 3 Boss-test session at the end.

| Phase | What | Files | Commit | Self-verify |
|---|---|---|---|---|
| **A** | Settings UI structure + new IPC + appSettings.cece sub-object | `SettingsModal.svelte` + new IPC + `appSettings.ts` + new TS calibration helper | `V3-§10.A` | `cargo test cece::reliability::tests` (IPC unit-tested via existing helpers) + svelte-check |
| **B** | en + ar i18n for new Settings strings | `en.json` + `ar.json` | `V3-§10.B` (or folded into A) | grep $t() keys against the JSON files |
| **C** | EN help topic + EN User Manual chapter | new `Source Review.md` topic + edit `User Manual.md` | `V3-§10.C` | proof-read pass; cross-references resolve |
| **D** | 13-locale i18n backfill for the cece.* keys | 13 locale JSONs | `V3-§10.D` | parse-validity check (each file remains valid JSON) |
| **E** | Help topic translations into 14 non-English help dirs | new `Source Review/Source Review.md` per locale | `V3-§10.E` | each file present + has the AI-translated disclaimer header |
| **F** | User Manual chapter translations into 14 `User Manual.md` files | edit each translated User Manual | `V3-§10.F` | each file present + has the new chapter |
| **G** | NSIS rebuild + orientation v1.93 + Gate 3 Boss-test ready | docs + build artifacts | `V3-§10.G` | **✅ Boss-test Gate 3** |

Each phase commits with its session-log entry; orientation bump to v1.93 lands inside the Phase G commit per SO #6.

---

## §1 — Phase A: Settings UI structure + IPC + appSettings.cece

**Goal:** wire the user-facing Settings surface for CECE, including a new read-only per-Library calibration view that exposes the V3-§9.C.2 reliability data.

**New IPC:**

```rust
/// V3-§10.A — Read the active Library's reliability profile for UI display.
/// Returns the entire ReliabilityProfile struct (which serializes to the
/// same shape as the JSON file). UI converts to display rows.
#[tauri::command]
pub fn cece_get_reliability_for_active_library(
    app: tauri::AppHandle,
) -> Result<ReliabilityProfile, String> { ... }
```

Located in `src-tauri/src/cece/reliability.rs`; registered in `lib.rs::invoke_handler`. Implementation: resolve the active Library root via the same `library_root_for_note` pattern that `cece_record_correction_for_card` uses (with the active note path or — if no note is open — fall back to the first Library root from `list_libraries`). Returns empty default profile if no reliability JSON exists yet.

**Settings schema additions:**

```typescript
// src/lib/stores/appSettings.ts
type CECESettings = {
  reasoningTrailVisibility: 'always' | 'on_disagreement' | 'never';  // default 'on_disagreement'
  backgroundScan: 'off' | 'on_save' | 'on_startup';  // default 'off'
};
```

Defaults preserve current behavior. Existing users with no `cece` sub-object in their saved settings get the defaults on first read (per the existing migration pattern in appSettings).

**Frontend integration:**

- New `cece` reactive on the SourceReviewPanel reads `$appSettings.cece.reasoningTrailVisibility`. The existing `isTrailOpen()` helper updates to honor the user's pick:
  - `always` → trail always open
  - `on_disagreement` → current behavior (trust-cal banner + Split/StrongMajority cards open by default)
  - `never` → trail always closed unless user explicitly clicks the toggle
- Background scan integration: `on_save` listens for the existing 1500ms-debounced save event from NotePane and triggers `classifier_suggest_for_note`. `on_startup` triggers `classifier_scan_start` once on app boot. `off` is a no-op (manual scan only).

**Settings UI markup** (in `SettingsModal.svelte`, new section under Intelligence):

```html
<div class="setting-section-heading">
  {$t('cece.settings.heading') || 'Constellation Epistemic Content Engine'}
</div>
<p class="section-intro">
  {$t('cece.settings.intro') || 'The cataloger ensemble that classifies your notes along Source × Content Type axes. Six lenses, each with its own evidence; the engine combines their votes into a synthesis. Local-only — no notes leave your device.'}
</p>

<!-- Reasoning Cataloger model status -->
<div class="setting-item">
  <div class="setting-info">
    <div class="setting-name">{$t('cece.settings.reasoningModel') || 'Reasoning Cataloger model'}</div>
    <div class="setting-desc">
      {$t('cece.settings.reasoningStatus') || 'Not downloaded — local AI judgment lens deferred to V3-§7.b. When llama.cpp wiring ships, you\'ll be able to download Qwen3-4B Q5_K_M from this panel.'}
    </div>
  </div>
  <button class="test-btn" disabled>{$t('cece.settings.downloadDisabled') || 'Coming soon'}</button>
</div>

<!-- Reasoning trail visibility -->
<div class="setting-item">
  <div class="setting-info">
    <div class="setting-name">{$t('cece.settings.trailVisibility') || 'Reasoning trail visibility'}</div>
    <div class="setting-desc">{$t('cece.settings.trailVisibilityDesc') || 'When to auto-expand the per-cataloger reasoning trail on each Source Review card.'}</div>
  </div>
  <select bind:value={$appSettings.cece.reasoningTrailVisibility}>
    <option value="always">{$t('cece.settings.trailAlways') || 'Always show'}</option>
    <option value="on_disagreement">{$t('cece.settings.trailOnDisagreement') || 'On disagreement only (default)'}</option>
    <option value="never">{$t('cece.settings.trailNever') || 'Always hide'}</option>
  </select>
</div>

<!-- Background scan -->
<div class="setting-item">
  <div class="setting-info">
    <div class="setting-name">{$t('cece.settings.backgroundScan') || 'Background classification'}</div>
    <div class="setting-desc">{$t('cece.settings.backgroundScanDesc') || 'When to auto-classify notes that don\'t yet have sources or content type set.'}</div>
  </div>
  <select bind:value={$appSettings.cece.backgroundScan}>
    <option value="off">{$t('cece.settings.scanOff') || 'Off — manual scan only (default)'}</option>
    <option value="on_save">{$t('cece.settings.scanOnSave') || 'On note save'}</option>
    <option value="on_startup">{$t('cece.settings.scanOnStartup') || 'On app start'}</option>
  </select>
</div>

<!-- Per-Library calibration view (collapsible) -->
<div class="setting-item">
  <details>
    <summary>{$t('cece.settings.calibrationHeading') || 'Per-Library calibration'}</summary>
    <PerLibraryCalibrationView />
  </details>
</div>
```

**New helper:** `src/lib/cece/calibrationView.svelte` (or .ts + a `<PerLibraryCalibrationView>` component that fetches via the new IPC + renders a table per the Architect §9 mockup).

**Self-verification:**
- Boot the app fresh; open Settings → Intelligence; confirm CECE section renders.
- Change `reasoningTrailVisibility` to `always`; reload; confirm trails open on every card.
- Change `backgroundScan` to `on_save`; type 100 chars rapidly in NotePane; confirm 0 IPC calls during typing (only one IPC ~1500ms after typing stops).
- Open the calibration view on a Library with reliability data; confirm the table shows per-cataloger per-axis counts.
- `cargo test cece::reliability::tests` — existing 12 tests pass + 1 new test for the IPC's helper if applicable.
- `npx svelte-check` — zero new errors on SettingsModal.svelte.

**Pass criteria:** all 4 setting rows render; calibration view shows real data; defaults preserve existing behavior; no regressions.

**Commit:** `MIG-021v3 V3-§10.A — Settings UI + IPC + appSettings.cece sub-object`

---

## §2 — Phase B: en + ar i18n for new Settings strings

**Goal:** populate ~15 new `cece.settings.*` keys in `en.json` + `ar.json` per the Phase A markup.

**Files:**
- `src/lib/i18n/en.json` — new `cece.settings` block
- `src/lib/i18n/ar.json` — same in Arabic

**Self-verification:** `grep "cece.settings" src/lib/components/SettingsModal.svelte` lists all keys used; each appears in both en.json and ar.json.

**Pass criteria:** every `$t('cece.settings.*')` call in SettingsModal has a translation in both en + ar; no inline EN fallback fires.

**Commit:** `MIG-021v3 V3-§10.B — en+ar i18n for CECE Settings strings`

(This phase may be folded into Phase A's commit if it lands cleanly; separate phase is for clarity in the Plan.)

---

## §3 — Phase C: EN help topic + EN User Manual chapter

**Goal:** ship the user-facing English documentation for the Source Review surface.

**New file: `docs/help.uConstellation.World/Source Review/Source Review.md`**

Sections:
1. **What is the Source Review panel?** — one paragraph plain language explaining CECE in user terms ("six lenses examining each note along two axes").
2. **The two axes** — Source (where knowledge comes from) + Content Type (what kind of knowledge).
3. **The six catalogers** — one paragraph each: Your frontmatter, Citations & structure, Wordstems & lexicon, Linked notes, Similar notes, AI judgment (with the lens-color guide blue/rose/amber/teal/violet/green).
4. **Three confidence regimes** — Unanimous / StrongMajority / Split with worked examples.
5. **The dot cluster** — what the colored dots mean; voiced/silent/dissent states.
6. **Sibling Disambiguation** — when it appears + how to use it.
7. **The reasoning trail** — what the friendly rule chips mean; when to read them.
8. **The queue composition filter** — five filter chips explained.
9. **Trust calibration period** — first 50 reviews + what changes after.
10. **Accept vs Reject vs Edit vs Disambig pick** — when to use each.
11. **Per-Library calibration** — what the Settings panel data means.

**Edit: `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md`**

Add a one-paragraph cross-reference at the end pointing to the new Source Review topic.

**Edit: `docs/User Manual.md`**

Add a new chapter "**The Source Review Workflow**" between the Knowledge Strata chapter and Search. Approximately 800-1500 words covering the same ground as the help topic but at User Manual depth (less screenshot-detail, more workflow narrative). Cross-reference to the help topic for click-by-click instructions.

**Self-verification:** read-through pass; cross-references resolve; no internal component names (no "SourceReviewPanel.svelte" etc. — user-facing labels only).

**Pass criteria:** topic + chapter both present; consistent vocabulary across them; help topic discoverable from the help system's index.

**Commit:** `MIG-021v3 V3-§10.C — EN help topic + EN User Manual chapter`

---

## §4 — Phase D: 13-locale i18n backfill for cece.* keys

**Goal:** translate all `cece.*` keys (~90 keys total: 75 from V3-§8/§9 + 15 from V3-§10.B) into the 13 non-en/non-ar locales.

**Files:** `src/lib/i18n/{de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — each gets a new `cece` block matching en.json's structure with translations.

**Translation discipline:**

Per the BASIC RULE (don't make things up + native-speaker review):

- I produce best-effort translations using a consistent glossary. The glossary lives in a comment block at the top of each translated `cece` block, capturing the core technical terms (cataloger / axis / regime / split / unanimous / disambiguation / trust calibration) so future edits stay consistent.
- Each translated block ends with a `_translation_note` key: `"AI-generated translation; native-speaker review pending. Please file corrections via the project repository."`
- Languages I have higher confidence on (de/es/fr/pt/it-style romance + ar+fa+ur+he Arabic-script + tr) get full translations.
- Languages with non-Latin scripts I'm less confident on (ja/ko/zh/hi) get translations with a more cautious note + a slightly larger correction window.

**Self-verification:**
- Each JSON file parses cleanly (`node -e "JSON.parse(require('fs').readFileSync(...))"` for each).
- Each `cece` block has the same key set as `en.json::cece` (deep-equality on key paths, not values).
- The `_translation_note` field is present in each.

**Pass criteria:** all 13 locale files have a populated `cece` block; same key shape as en.json; translation-quality disclaimer present.

**Commit:** `MIG-021v3 V3-§10.D — 13-locale i18n backfill for cece.* keys`

---

## §5 — Phase E: Help topic translations into 14 non-English help directories

**Goal:** make the new Source Review help topic discoverable in all 14 non-English help directories.

**Files:** 14 new files at `docs/help.{ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}/Source Review/Source Review.md`.

**Translation discipline:**

Per the V3-§10 Architect §5 risk register: each translated help topic ships with a header note:

```markdown
# Source Review

> **Translation note:** This help topic is an AI-generated translation
> from the canonical English version at
> `help.uConstellation.World/Source Review/Source Review.md`. Native-
> speaker review pending. Please file corrections via the project
> repository.

[ … translated body … ]
```

For locales where the translation directory has only stub structure today (most of them per the audit — only help.ar + a few others have substantial topic coverage), the new file lands as the first new V3-§10 topic in that directory.

**Self-verification:** each of 14 files exists; each has the disclaimer header; each has the same section structure as the English version (verified via grep of `## ` heading lines).

**Pass criteria:** 14 translated files present; consistent structure; disclaimer header on each.

**Commit:** `MIG-021v3 V3-§10.E — Help topic translations (14 locales)`

---

## §6 — Phase F: User Manual chapter translations

**Goal:** add the new Source Review Workflow chapter to all 14 translated User Manuals.

**Files:** edit each `docs/help.{ar,de,...,zh}/User Manual.md` — append the new chapter at the same position as the English version (between Knowledge Strata and Search, or at end if those chapters don't exist in that translation).

**Translation discipline:** same disclaimer header pattern as Phase E. The chapter content is roughly 800-1500 words — translation work is real but bounded.

**Self-verification:** each User Manual file has the new chapter heading + disclaimer; word count per chapter is in the 800-1500 range (proxy for "actual translation, not a stub").

**Pass criteria:** 14 User Manual files updated; new chapter present; disclaimer on the chapter.

**Commit:** `MIG-021v3 V3-§10.F — User Manual chapter translations (14 locales)`

---

## §7 — Phase G: NSIS rebuild + orientation v1.93 + Gate 3 Boss-test

**Goal:** ship a build Eisa can install + run the Gate 3 Boss-test verification clause for V3-§10.

**Files:**
- `docs/Constellation Orientation & Onboarding v1.93.md` (NEW — bump from v1.92, document V3-§10 close-out)
- `lab/reports/SESSION-LOG-2026-05-11.md` — entry summarizing all 7 phases
- NSIS build artifact: `Constellation_0.3.4_x64-setup.exe`

### ✅ Boss-test Gate 3 — V3-§10 user-facing surfaces

Per the Testing Instructions Rule, every stage articulates the feature first, then walks through interaction by interaction.

**Stage 0 — Verify the new build is installed**

1. Close Constellation if running.
2. Run installer.
3. Launch + open the same Library you've been Boss-testing.

**Stage 1 — Settings UI section**

*Feature:* Phase A added the "Constellation Epistemic Content Engine" section in Settings → Intelligence with 4 setting rows + a collapsible per-Library calibration view.

1. Open Settings (Cmd/Ctrl+,).
2. Navigate to the **Intelligence** tab.
3. Scroll to find the new section heading **"Constellation Epistemic Content Engine"**.

**Expected:** section header + intro paragraph + 4 setting rows: (a) Reasoning Cataloger model status (with disabled "Coming soon" button), (b) Reasoning trail visibility dropdown (default "On disagreement only"), (c) Background classification dropdown (default "Off"), (d) Per-Library calibration collapsible.

**Stage 2 — Reasoning trail visibility setting**

1. In the new section, change **Reasoning trail visibility** to **"Always show"**.
2. Open the Source Review panel. Look at any card.

**Expected:** the reasoning trail is auto-expanded on every card (not just Split/StrongMajority cards). The "▾ Hide reasoning" / "▸ Why this classification?" toggle still works for manual override per-card.

3. Change to **"Always hide"**. Look at any card.

**Expected:** trail is collapsed on every card; manual click on the toggle still works to expand.

4. Change back to **"On disagreement only"**.

**Expected:** behavior matches the pre-V3-§10 trust-cal-end state — trail expands on Split/StrongMajority cards only.

**Stage 3 — Background classification: keystroke perf check**

*Feature:* Phase A added the background scan toggle. The "On note save" mode triggers classification on the existing 1500ms debounced save — NOT on every keystroke.

1. Set **Background classification** to **"On note save"**.
2. Create a new note, open it in NotePane.
3. Type 100 characters rapidly without pausing.

**Expected:** typing remains instant (no perceptible lag). After you stop typing, ~1500ms later, ONE classification IPC fires (visible as a brief Source Review queue update).

**Failure mode:** if typing stutters or you see multiple Source Review queue updates per second of typing, the integration is misfiring on keystroke instead of on debounced save. STOP and tell me.

4. Set back to **"Off"**.

**Stage 4 — Per-Library calibration view**

*Feature:* Phase A surfaces the V3-§9.C.2 reliability data as a read-only table.

1. In the CECE settings section, click the **Per-Library calibration** collapsible to expand it.

**Expected:** table renders with one row per cataloger that has voiced on this Library, showing horizontal + vertical accuracy ratios. Catalogers with fewer than 20 corrections show "(uniform)" label. Empty state if no corrections yet logged.

**Failure mode:** if the table shows blank rows or stale data, the IPC isn't returning the right Library's profile. Tell me which Library you're in vs what's displayed.

**Stage 5 — i18n in 13 other locales**

*Feature:* Phase D translated 90 cece.* keys into 13 non-en/non-ar locales.

1. Switch the app's interface language to **German** (Settings → Appearance → Language → Deutsch).
2. Open Settings → Intelligence → CECE section.

**Expected:** the section header + 4 setting names render in German (per the translated `cece.settings.*` keys).

3. Open the Source Review panel.

**Expected:** the count strip, dot cluster tooltips, trail toggle text, etc. all render in German.

4. Switch back to **English** to confirm fallback works.

5. (Optional) Repeat for 1-2 other locales of your choice (Hebrew or Persian for an RTL pair; Japanese or Chinese for a CJK script).

**Stage 6 — Help topic discoverability**

1. Open the help system from the Constellation menu (or wherever the help is accessible — confirm the help directory contents are wired into the in-app help browser if there is one; otherwise navigate the file system).
2. Find the new **"Source Review"** topic in the English help directory.

**Expected:** topic exists; opens cleanly; renders the 11 sections from Phase C.

3. Switch to a non-English locale and confirm the translated help topic is present (per Phase E).

**Expected:** translated file with the AI-translation disclaimer header.

**Stage 7 — User Manual chapter**

1. Open `docs/User Manual.md` (in Constellation NotePane or any text editor).
2. Find the new chapter **"The Source Review Workflow"** between Knowledge Strata and Search.

**Expected:** chapter present, ~800-1500 words, with cross-references to the help topic.

3. Open one of the translated User Manuals (e.g. `docs/help.de/User Manual.md`).

**Expected:** new chapter present at the same position with the AI-translation disclaimer.

### Gate 3 PASS criteria

- Stage 1 confirms Settings section renders correctly
- Stage 2 confirms trail visibility setting actually changes behavior across all 3 modes
- Stage 3 confirms background scan doesn't fire on keystroke (perf preserved)
- Stage 4 confirms per-Library calibration data surfaces correctly
- Stage 5 confirms i18n fallback works across at least 2 non-en/non-ar locales
- Stage 6 confirms help topic exists in en + at least 1 other locale
- Stage 7 confirms User Manual chapter exists in en + at least 1 other locale

If all 7 stages pass, **Gate 3 closes** and we move to V3-§11 (final integration audit + close-out of MIG-021v3 entire).

**Commit:** `MIG-021v3 V3-§10.G — NSIS build + orientation v1.93 + Gate 3 ready`

---

## §8 — What V3-§10 does NOT do (re-confirming non-scope)

- ❌ Wire llama.cpp / download Qwen3-4B (V3-§7.b territory; Reasoning model status text says "deferred")
- ❌ Add per-cataloger weight overrides in Settings (advanced-user feature; backlog)
- ❌ Add a "reset reliability data" button (correction is the canonical path; reset would be confusing)
- ❌ Surface the V3-§9.D axis-aware GBNF in Settings (interface lock-in for V3-§7.b; not user-facing today)
- ❌ Add per-Library Reasoning Cataloger preferences (no preferences exist yet — Reasoning is global)
- ❌ Final V3-§11 integration audit (separate cascade)

---

## §9 — Risk register (mitigated)

| Risk | Phase | Mitigation in this Plan |
|---|---|---|
| Background scan fires on keystroke | A | Hook on existing 1500ms debounced save, NOT on `oninput`. Boss-test Stage 3 perf check. |
| Settings flag default changes existing user behavior | A | All defaults preserve current behavior (`on_disagreement` = trust-cal-end behavior; `off` = no auto-scan) |
| Calibration view confuses users when no corrections logged | A | Empty state copy: "No corrections logged yet on this Library. Calibration data will appear here after you Accept / Disambig at least one card." |
| Translation quality varies wildly across 13 locales | D, E, F | AI-generated disclaimer header on every translated file. Translation glossary comment in each i18n block. Native-speaker review framed as a follow-up activity. |
| Translated JSON files break the build (parse errors) | D | Per-file `JSON.parse` validation step before commit |
| Help topic name "Source Review" shadows the panel name | C | Subtitle the topic with "(Constellation Epistemic Content Engine — CECE)" so search finds both |
| User Manual chapter drifts from help topic | C | Both share opening paragraphs verbatim; differ only in depth |

---

## §10 — Approval request

**Boss, please approve this Plan.**

Once approved, per Plan-Approval-Equals-Build-Approval I'll cascade through phases A → G autonomously, stopping only at:
- The Stage 0–7 user-testable verification clauses in Phase G (Gate 3 Boss-test)
- Genuine architectural surprises (will surface and pause)
- Plan completion (summarize close-out + propose next step toward V3-§11)

The Standing Order session-log discipline applies between steps; I'll log each `V3-§10.X` commit as it lands. Orientation bump to v1.93 lands inside the Phase G commit per SO #6.

Estimated wall-clock time: 12-15 hrs of agent build/test/commit + Eisa's Gate 3 Boss-test session at the end.

---

# Session Log — 2026-06-20 (MIG-082 docs i18n catch-up)

> **Function in hand:** the **MIG-082 docs translation catch-up** — bring User Manual §14 (the clickable, 8-calendar Calendar) and the help-site Calendar topic into the ×14 non-EN languages, discharging the full-localization Standing Order now that the in-app i18n is complete ×15.
> Branch `main`. Picks up from `lab/reports/HANDOVER-2026-06-20-mig082-complete.md` (MIG-082 COMPLETE + audited + `/simplify`-clean, HEAD `5925e783`).

## Session-start ritual
- `git pull origin main` → already up to date.
- Read orientation v2.93 preamble (full MIG-082 picture) + HANDOVER-2026-06-20 + SESSION-LOG-2026-06-19 (the shipped record).
- Boss chose direction (AskUserQuestion): **docs translation catch-up**.

## Pre-flight findings (verified, not assumed)
- **Handover Option 3 ("Boot perf — defer the Sky read off boot") is STALE.** Confirmed via `git log` (`23fdd45f`→`cff8f827`→`2f3c8aa0`) + orientation v2.89: that shipped as **MIG-079 §C.2d** (cold `graph_ready` ~11 s → ~1.5 s, Boss-validated). No open boot-perf "defer the Sky read" task. Corrected memory `project_mig079_boot_wtd`.
- **Docs structure mapped:** EN master `docs/User Manual.md` §14 (current — reflects MIG-082); ×14 translated manuals `docs/help.{lang}/User Manual.md` each with a STALE ~10-line calendar section (pre-MIG-081 placeholder); EN help-site topics in `docs/help.uConstellation.World/` (35 topics); translated help dirs `docs/help.{lang}/` are 13-topic SUBSETS with **no Calendar topic**.
- **⚑ Handover inaccuracy found + corrected.** The handover said "the help-site calendar topic was updated EN-only this session." NOT TRUE — `help.uConstellation.World/Calendar/Calendar.md` was last git-touched **pre-MIG-080** and still described the Calendar as a *right-sidebar panel* with only purple/red dots, no cultural calendars. Translating it as-is would have pushed stale info into 14 languages. → Rewrote the EN topic from the validated §14 FIRST.

## Work done
### 1. EN help-site Calendar topic — brought current (author task)
- Rewrote `docs/help.uConstellation.World/Calendar/Calendar.md` (was 57 lines / pre-MIG-080 → now 90 lines): left-dock full-page view; gold/purple/red clickable dots; click empty→daily note (never duplicates); task-dot→open-at-line; complete-from-popover; eight cultural calendars (incl. Chinese/Korean lunisolar); Hijri calculation-method + month-correction; Chinese/Korean year-display + native/phonetic month names; Style-Setter theming; Gregorian-ISO filename invariant; daily-note Hijri stamp + Properties "+ X" converter; RTL note retained.

### 2. Manual §14 ×14 + help-site topic ×14 — Workflow fan-out
- **Workflow `wf_62dfd578`** — 14 native-translator agents (one per locale: ar/de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh), `agentType: general-purpose`, each owning ONLY its locale's two files (no shared files → no positional-mapping risk, the §B.3 caution avoided structurally).
- **Task 1 per locale:** replace the stale calendar section body in `help.{lang}/User Manual.md` (heading + section number preserved → TOC anchors intact, no renumbering) with a faithful translation of the current EN §14.
- **Task 2 per locale:** create the NEW `help.{lang}/Calendar/Calendar.md` translating the now-current EN help-site topic (frontmatter keys English, values translated; tables + `[!tip]` callouts preserved).
- **Translation rules:** localize calendar-system names + UI words into native equivalents; KEEP VERBATIM the CJK/Hangul glyphs, sexagenary/Dangi year glyphs, romanizations, Arabic phonetics, code tokens, example values, `al-Tawfīqāt al-Ilhāmiyyah`. Phonetic example labels "English"/"Arabic" translated; both romanizations kept. No invention.
- **Result:** 14/14 manuals + 14/14 topics, 14 agents, ~1.08M subagent tokens, 210 s.

## Verification (Claude-side — verified, NOT trusted)
- `git diff --stat`: 14 manuals (+32 each, uniform) + 1 EN topic (+92) + 14 new `Calendar/Calendar.md` files. Exactly the expected surface.
- **Glyph sweep** (verbatim tokens across all translated files): 五月 / 5월 / 闰六月 / 윤6월 / 丙午年 / 병오년 / 단기 4359 / Wǔyuè / Owol / وُو-يوي / al-Tawfīqāt al-Ilhāmiyyah / `2025-06L-17` — present in 14/14 manuals + 14/14 topics. The single `단기 4359` manual "miss" = Korean's faithful `**단기** 4359` (bold markup split the literal grep) — confirmed correct.
- **Structural uniformity:** all 14 new topics = 90 lines, frontmatter delimiter ×1, `aliases:` ×1, `description:` ×1, H1 ×1, `[!tip]` ×3. Byte-uniform.
- **Read directly:** German manual diff (faithful, fluent, heading `## 15. Kalender` preserved, no renumber); Arabic manual section + Arabic topic (native, publication-quality — الميلادي/الهجري/العبري/الهندي شاكا/البوذي/الصيني/الكوري, الأشهر الحُرُم, شهر كبيس with 闰六月/윤6월 intact, Boss-verified phonetics وُو-يوي/أوه-وُل preserved, heading `## 16. التقويم` + `## 17.` intact).
- No app code touched (docs only) → no svelte-check / cargo needed; Editor-Surface Gate not in scope.

## PCS
- Orientation **v2.94** (NEW file, v2.93 retained as preamble) — SO #6 triggers: help topic restructured + shipped ×14, doc-drift item cleared. Landed in the SAME commit (feedback_orientation_inline_with_commit).
- This session log; memory `project_mig079_boot_wtd` corrected.
- Single commit on `main`; pushed to origin.
- **MoCh:** brief block; deferred to session close (not yet ~3 h of direct chat after MoCh-2026-06-20-0900).

## State / next
- MIG-082 fully closed, audited, `/simplify`-clean, AND now fully localized (in-app ×15 + docs ×15). Standing calendar docs debt RESOLVED.
- **Open (Boss's pick next):** the deferred inspector360 Settings-UI bug; or a new feature. (Boot perf is DONE — §C.2d.)
- Note: the other ~21 EN help-site topics remain untranslated in the help.{lang} subsets (a pre-existing standing debt, NOT calendar-specific) — flagged, not in this task's scope.

## STATE-OF-STANDING SNAPSHOT (Boss: "Where are we standing? What next?" — 2026-06-20, SO #5)

**Worktree:** clean. **HEAD** `4f474c94` on `main`, pushed to `origin/main`. Nothing uncommitted, nothing in-flight.

### (a) Verified-shipped & protected
- **MIG-082** — the clickable, 8-calendar Calendar (Gregorian/Hijri/Solar-Hijri/Hebrew/Indian/Buddhist/Chinese/Korean): interactive day cells, per-calendar year + native/phonetic month names, daily-note Hijri stamp, Gregorian→cultural converter. COMPLETE, Phase-4 audited (3×P1 fixed incl. a real BUG-015 F2 fix), `/simplify`-clean, Boss-validated. **Now fully localized** (in-app ×15 + docs ×15 as of this session, `4f474c94`).
- **MIG-081** — Calendar's 4 functions (month-correction, calc-mode, Style-Setter "Calendar" tab, retired daily-launcher) + per-element theming + ×15 i18n. COMPLETE, audited, Boss-validated.
- **MIG-079 (boot WTD)** — §A/§B/§C.1/§C.2a/§C.2b/§C.3/§C.2c/**§C.2d** ALL shipped+validated. **Boot perf is DONE** — cold `graph_ready` ~11 s → ~1.5 s (Sky read deferred off boot, §C.2d). No open boot-perf task.
- **MIG-078 §B1** — thundering-herd init_db race + frontmatter `\n---` parser fix. Shipped+validated.

### (b) At-risk / in-flight / uncommitted
- **None.** Tree clean; last work (docs i18n) committed + pushed.

### (c) Known-broken
- **inspector360 Settings-UI bug** — deferred across recent sessions, out of MIG-082 scope, still OPEN. **Specifics NOT yet known to me** — not reproduced, not scoped. (inspector360 = the 360.3D matrix inspector; concept paper `docs/concept-papers/16-inspector360.md`.) Reproduce-First applies before any fix.

### (d) Pending, not started
- **MIG-080 §B–§F** — right-rail → note-context-only cascade. Architect + **Plan approved** (`docs/MIG-080-Plan.md`); only §A/§A.2 shipped (Calendar relocated to left dock). The largest queued migration.
- **inspector360 Settings-UI bug** fix (see (c)).
- **Help-site translation parity** — ~21 EN topics still untranslated in the help.{lang} 13-topic subsets (pre-existing standing debt; calendar now done).
- **Bring-up program §D** (phase-by-phase function re-enable behind flags) — `project_bringup_concept_papers`; concept papers 02–32 are the checklist.
- **Documented deferrals (non-blocking):** MIG-079 §C.2c items (split-pane ×N traversal chips P1; `status` predicate nit; §C.2b cleanup once `perNoteLinkQueries` permanent); MIG-014 §2F 6×P2/P3 doc/render polish (`project_mig014_audit_p2_p3_followups`).

### (e) Documentation drift
- **Cleared this session:** the calendar docs translation debt (manual §14 ×14 + help-site topic ×14); the stale EN help-site Calendar topic; the stale `project_mig079_boot_wtd` memory + handover boot-perf note.
- **Remaining (minor):** orientation BODY has no dedicated calendar §4.x section yet (preamble + Plan/Architect carry it); other-feature manual ×14 translations ride the standing debt; the ~21 untranslated help topics (see (d)).

**Recommendation:** with boot perf done and MIG-082 fully closed, the two substantive forward moves are **MIG-080 §B–§F** (approved Plan, largest queued, real product surface) or the **inspector360 bug** (reproduce-first). The help-site parity sweep is the low-risk "harness-warm" option.

---

# MIG-080 §B–§F — cross-check before cascade (Boss chose it 2026-06-20; SO #8 + WA#4)

> **Function in hand:** MIG-080 §B–§F — the right-sidebar → note-context-only cascade. Plan `docs/MIG-080-Plan.md` (canonical lettering; §A/§A.2 shipped). Plan is PRE-MIG-081/082 → 5 parallel read-only cross-check agents established current ground truth before any code.

## Cross-check verdict: Plan VALID (none of §B–§F shipped), with refinements + a re-discovered open defect
- **Shared spine (current, verified):** `NOTE_SCOPED_TABS` (`+layout.svelte:391`) = `{properties, backlinks, star, tasks, health, provenance, inspector360}`. `'calendar'` correctly dropped (§A done). `review`/`sourceReview` NOT yet in (added by their phases). Right-rail tabs render at `:7140–7435` (all 10 present incl. Tags toggle `:7224`, Source Review force-visible `:7210`). `PanelId`/defaults at `store.ts:3354/4062`; `sourceReview` not in PanelId union.
- **§A inspector360 Settings-UI bug — STILL OPEN (the "deferred inspector360 bug" = this).** `SettingsModal.svelte:2147–2158` panel-placement list shows 9 IDs (backlinks/outgoing/properties/tags/sky/tasks/health/provenance/review) — **missing `inspector360` AND `sourceReview`**, though both are in the schema + tab strip. The Plan folded this into §A; it was skipped at §A ship. **~2-line fix; clears the separately-deferred bug.**
- **§B Tags — not shipped; SIMPLER than planned.** "All tags" toggle live (`+layout:7224–7227`, `tagView 'note'|'all'`); `DashboardView.svelte:104` still `scanAllLibraryTags()` (per-lib filesystem walk, `tagUtils.ts:14`). `tag_counts` table shipped (MIG-079 §C.1, `tag_counts.rs`; boot read-flip `cache.rs read_tags_in_schema`) → boot `allLibraryTags` is ALREADY tag_counts-derived → **no new `get_all_tags()` Rust command needed; swap Dashboard `scanAllLibraryTags()` → `allLibraryTags`** (Rule 8 satisfied). `#tag`→Search Hub routing intact (`handleTagClick`, `:4458`).
- **§C Tasks — §C.a DONE, defect REAL, + new defect.** Right-rail note-Tasks already note-scoped (`scanNoteTasks`, `:1692`). Left agenda NOT built (GlobalTasksView is a full-page overlay only; Calendar dots cover per-date grouping). **`toggle_task` (`tasks.rs:440–503`) calls `gate_write` (`:500`) but NEVER reindexes → search/backlinks/tags drift — DEFECT STANDS** (MIG-082 §A.3 `toggleTaskReconciled` is a different layer: frontend model reconcile, not index). **NEW latent defect found:** `SecondScreenPage.svelte:1344/1477` call unreconciled `toggleTask()` (BUG-015 F2 class — could be reverted by autosave). → fold the SecondScreen toggle into the reconciled path within §C.
- **§D Source Review — not split; LIGHTER.** `SourceReviewPanel` loads `sources_list_pending_suggestions` (universe-wide, `mod.rs:688`) regardless of note; has `activeNotePath` prop used only for the "Classify open note" button. `sources_get_suggestions(notePath)` (`mod.rs:671`) reads the per-note record → **likely serves the per-note need without a new IPC**. CatalogerView wired left-dock (`+layout:5583`) ✓ universe home exists.
- **§E Knowledge Health — not split.** `detect_tensions` (`tension.rs:67`) library-wide; no per-note variant; `TensionPanel` has no `noteContext` prop → right-rail Health renders LIBRARY tensions (the INV-1 violation). `KnowledgeHealthDashboard` (cached snapshot, `search.rs:6701`) ✓ universe home exists. As planned.
- **§F Review Pulse — NOT started; defect confirmed.** `record_note_visit` (`review.rs:135`) = **DEAD CODE (zero callers, whole repo)** → notes resurface forever. `get_due_notes`/`mark_reviewed`/`snooze`/`dismiss` exist; `get_note_review_status` does NOT. `ReviewPulsePanel` renders universe queue (not note status); no `ReviewStatusPanel`, no `ReviewerView`, no Dashboard "N due" card, `openNoteTab` doesn't call visit. As planned (heaviest).

## Refined build order (proposed — pending Boss confirm of fold-ins)
0. **§A-completion: inspector360 (+sourceReview) Settings-UI fix** — clears the deferred bug; ~2 lines; first quick win.
1. **§B Tags → Dashboard** (lightest; reuse `allLibraryTags`, no new Rust).
2. **§C Tasks** — left agenda + `toggle_task` reindex fix + fold SecondScreen toggle into `toggleTaskReconciled` (Editor-Surface Gate).
3. **§D Source Review** (reuse `sources_get_suggestions`; wire panel per-note; Cataloger home).
4. **§E Knowledge Health** (per-note tension IPC + `TensionPanel` noteContext; KH-Dashboard home).
5. **§F Review Pulse** (heaviest: per-note IPC + `ReviewStatusPanel` + `ReviewerView` + `record_note_visit` wire + Dashboard card).
6. **§G** final `NOTE_SCOPED_TABS` reconcile + `/simplify` + 3-agent audit + Editor-Surface Gate for §C.

## Boss confirmed (AskUserQuestion 2026-06-20): build order **§0 then §B→§F**; **fold the SecondScreen toggle defect into §C**.

## §0 — inspector360 Settings-UI fix — BUILT (frontend-only; pending Boss test)
- **`SettingsModal.svelte`** (panel-placement list, ~:2157): added `['inspector360', $t('settings.panels.panelInspector360'), $t('settings.panels.panelInspector360Desc')]`. inspector360 was already a valid `PanelId` with a `right-sidebar` default (`store.ts`) → the existing value/onchange logic works unchanged. (`sourceReview` placement deferred to §D — it's not yet a `PanelId`; render-gated "force-visible".)
- **i18n ×15:** `settings.panels.panelInspector360` ("360.3D Inspector") + `panelInspector360Desc` ("360° matrix view of the current note"). EN added inline; ×14 via one native-aware translator (keyed output — no positional risk) + Python textual insert after `panelReviewDesc` (preserves formatting). All 15 parse + values verified. ar = مُفتِّش 360.3D / عرض مصفوفي 360° للملاحظة الحالية.

## §B — Tags "All tags" → Dashboard — BUILT (frontend-only; pending Boss test)
- **`+layout.svelte`:** added `'tags'` to `NOTE_SCOPED_TABS` (`:391`) → the right-rail Tags tab is now note-scoped (empty-guard shows "no note selected" with no note). Removed the `tagView` state + the "All tags"/"This note" toggle + the universe `TagsPanel` branch; the Tags branch now renders ONLY the open note's tags (`activeNoteTags` → `handleTagClick` → `#tag` Search Hub, unchanged). Removed the now-dead `TagsPanel` import. Passed `allTags={allLibraryTags}` to `<DashboardView>`.
- **`DashboardView.svelte`:** new `allTags` prop; `dashboardTags` is now `$derived` from it (`Object.entries(allTags).map(...).sort(desc by count)`) instead of `$state` loaded via `scanAllLibraryTags()`. Removed the `scanAllLibraryTags` import + the filesystem-walk call in `loadDashboardData`. **Rule 8 satisfied** — the universe tag list reads the write-time `tag_counts`-derived `allLibraryTags` (MIG-079 §C.1), no per-library scan. `selectTag`/`notes_by_tag` click-through unchanged.
- **Note:** `scanAllLibraryTags` (`tagUtils.ts:14`) is now orphaned → `/simplify` candidate at §G. Orphan i18n keys `panels.tagsThisNote`/`tagsAll` left in place (harmless; §G cleanup).
- **Verify:** `svelte-check` **0 errors / 321 pre-existing warnings**. `git diff --stat`: SettingsModal +2, DashboardView, +layout (tags branch), 15 locales +2 each. Release build (npm → cargo) — binary 20:01:21, embed verified fresh (binary linked after build/). Committed `f767fca3`.
- **§0+§B Boss test: "All Pass."** One observation → §0b.

## §0b — 360.3D right-sidebar side margin (Boss observation) — BUILT
- **Symptom:** the 360.3D inspector content (the per-type bars' count column — "100.0%" / the "—" dashes) sat flush against the right panel edge; Boss wants ≥5px side margins.
- **Root cause:** `Inspector360.svelte` `.i360-bar-row` is a grid `130px 1fr 60px` with **fixed** label+count columns. On a narrow right sidebar those fixed columns overflow the (12px-padded) `.i360.compact` container, pushing the right-aligned count past the padding to the edge (the truncated "Derives Fr…" label confirms the 130px column was saturated). The 12px inset existed but was being overflowed.
- **Fix:** `grid-template-columns: minmax(0, 130px) 1fr 60px` — the label column can now shrink below 130px (it already ellipsises), so the row never overflows and the `.i360.compact` 12px side inset is restored. Count stays 60px → bars still align across rows. CSS-only; no regression when the sidebar is wide.
- **§0+§B+§0b PUSHED** (`0920c5f9` → origin/main). Boss "Pass" on the margin re-check.

## §H — Style-Setter "Text size" control for the right sidebar (Boss-requested) — BUILT
- **Boss ask (post-§0b test):** resize the right-sidebar text via the Style Setter. AskUserQuestion → **text-only scale** (not zoom), **this before §C**.
- **Why text-only is involved (mapped, not guessed — Workflow `wf_6cc60bb3`, 3 agents):** the ~10 sidebar panels are **95% `rem`** (root-relative) + some `px` → a wrapper `font-size` CANNOT scale them (rem ignores the wrapper, looks at `<html>`). (Caught + corrected a Workflow agent's CSS error claiming rem is ancestor-relative — it is NOT.) The global interface-font setting already scales everything via the root; an INDEPENDENT sidebar control needs either `zoom` (whole-sidebar) or per-font-size `calc` (text-only). Boss chose text-only.
- **Mechanism (scoped + safe):** `.rs-inner { --rs-scale: calc(var(--rs-text-scale, 100) / 100); }` — the scale is scoped to the right-sidebar wrapper; descendant panels read `--rs-scale` only inside `.rs-inner` (the SAME shared panel rendered elsewhere — e.g. PropertyEditor inline in NotePane — has `--rs-scale` unset → fallback 1 → unchanged). **181 font-sizes** rewritten `font-size: X` → `font-size: calc(X * var(--rs-scale, 1))` across 10 components (PropertyEditor 32, Inspector360 36, SourceReview 36, Backlinks 14, Outgoing 12, Provenance 12, ReviewPulse 11, Tasks 9, Tension 9, LocalSky 2) + 8 `.rs-*` rules in `+layout`. **rem/px ONLY — `em` deliberately skipped** (em already inherits the parent's scaled size → wrapping would double-scale). Deterministic Python transform (idempotent; `!important` preserved).
- **Style Setter:** added a "Text size" range control (`--rs-text-scale`, 70–140, step 5, default 100, unit '') to the **existing `cRightSidebar` element** (Components → Right sidebar — a two-zone LIVE category, so dragging scales the real sidebar in real time). Reused the existing translated label `text_size` (×15, ar = حجم النص) → **zero new i18n**. (Discarded an initial separate top-level "Right Sidebar" category — would have duplicated the `right_sidebar` label.)
- **Verify:** `svelte-check` **0 errors**. **Adversarial review (general-purpose) — PASS on all 6 invariants:** (1) all 181 wraps use the `, 1` fallback → pixel-identical at default; (2) `--rs-scale` set ONLY on `.rs-inner`, `--rs-text-scale` only written by the Style Setter to `<body>` — no leak (verified PropertyEditor's inline NotePane instance stays unscaled); (3) zero `em` wrapped (no double-scale); (4) no `calc(calc(`; (5) status-bar/federation/modals/spacing untouched; (6) the /100 math sound.
- **§H global Boss test: "Pass" — BUT Boss pivoted on reflection:** one sidebar-wide size can't suit panels with different content → **per-panel size tokens** (the Option-3 he'd earlier declined). Built but NOT committed; superseded by §H per-panel below. The 181 wraps are REUSED (they read a generic `--rs-scale`).

## §H (revised) — PER-PANEL right-sidebar text size — BUILT (pending Boss test)
- **Boss:** "You resize one panel to discover it is not suitable for another. So, let's have it per-panel size tokens." → each sidebar panel gets its OWN text size.
- **Elegant reuse — only the SOURCE of `--rs-scale` changed, not the 181 wraps.** Removed the single `.rs-inner { --rs-scale: … }`; `.rs-inner`'s markup now sets `--rs-scale` INLINE from the **active tab's own token**: `style="--rs-scale: calc(var(--rs-text-scale-{rightSidebarTab}, 100) / 100)"`. Only one panel renders at a time, so this scopes the scale to whichever panel is open — one dynamic binding, no per-wrapper plumbing. Each panel's component keeps its generic `var(--rs-scale, 1)` wraps.
- **Style Setter:** removed the global "Text size" from `cRightSidebar`; added a new **`cRsText` element ("Right Sidebar Text")** in the Components category with **10 per-panel sliders** (Properties / Backlinks / Tags / Sky View / Tasks / Health / Provenance / Review / 360.3D / Source Review), each writing `--rs-text-scale-<tab>` (70–140, def 100). Two-zone LIVE.
- **i18n ×15 (zero fabrication, max reuse):** control labels = panel names → COPIED the already-translated `settings.panels.panel*` values into `styleSetter.labels` (properties/backlinks/tags/tasks/health/provenance/review; `sky_view` already existed); `360_3d` = literal "360.3D"; element name "Right Sidebar Text" + "Source Review" (the latter was EN-only in the 14 locales — a pre-existing Source-Review i18n debt) translated natively ×14. JSON round-trip verified clean; all 15 parse; 168 insertions.
- **UX wrinkle (documented for the test):** because the live binding follows the ACTIVE tab, dragging a panel's slider shows live ONLY when that panel's tab is the open right-sidebar tab (you size the panel you're looking at).
- **Verify:** `svelte-check` **0 errors**. Release build (npm → cargo) — binary 05:11:41, embed verified.
- **§H per-panel Boss test: "Perfectly done! Thank you."** + 2 findings (both in the Review Pulse panel) → §H.2.

## §H.2 — Review Pulse polish (Boss findings during §H test) — BUILT
- **Finding 1 — the tab COUNT badge ("62") was clipped by the sidebar's top edge.** Root: `.rs-tab-badge` is `position:absolute; top:-2px` (overhangs the flush top) AND §H was scaling it (`calc(0.55rem * var(--rs-scale))`) → sizing the Review panel up grew the badge and worsened the clip. **Fix:** the badge is tab-strip CHROME, not panel text → REMOVED it from §H scaling (back to plain `0.55rem`); nudged `top:1px; inset-inline-end:1px` so it sits inside the tab (not clipped). (§H wrap count 181 → 180; the badge exclusion is intentional.)
- **Finding 2 — the Review lists were capped ("+32 more" non-clickable; resurfacing silently capped at 20).** **Fix (`ReviewPulsePanel.svelte`):** the cap rows ("+N more") are now **clickable buttons** that expand the FULL list (`showAllNeverReviewed` / `showAllResurfacing`, toggling back via "Show less"). Both the never-reviewed (cap 30) and resurfacing (cap 20) sections. `.rp-more` → a real `<button>` (cursor/hover/underline). ×15 i18n for `reviewPanel.more` / `reviewPanel.showLess` (native; ar = أخرى / عرض أقل). (Opt-in expand keeps the default fast; §F will properly split this panel — note status vs a full-page reviewer — per the MIG-080 plan.)
- **Verify:** `svelte-check` **0 errors**. Release build (npm → cargo) — binary 09:38:33, embed verified.
- **§H.2 Boss test: "Better. But I want the whole list expanded by default, no +N more."** → §H.3 (the click-to-expand is replaced by full show-all).

## §H.3 — Review Pulse: show the FULL list by default (Boss) — BUILT
- **Boss:** "I want the whole list expanded by default, no need for the +00 more."
- **Verified the constraint first:** `get_due_notes` (`review.rs`) has **NO cap** — it pushes EVERY note older than 1 day that's never been reviewed, so "never reviewed" can be the WHOLE library in a fresh universe. Rendering all of that as plain DOM = the §C.2c freeze (Rule 3). Boss's actual list is 62 (modest).
- **Fix:** removed the cap + the "+N more" entirely → **all items shown by default**. Each section renders via a shared `reviewItem` snippet through a `section` snippet that picks the strategy: **plain `{#each}` (all items) below `VLIST_THRESHOLD` (80)** — the natural sidebar scroll, what Boss wants for normal lists — and **`VirtualList` (the §C.2c windowed pattern) only above 80** (a `.rp-vlist-wrap` max-height:60vh scroller), so a pathological all-unreviewed library can't freeze. Reuses the existing `VirtualList`. `rpRowHeight` ≈ 30px estimate. svelte-check 0 (snippet params typed `DueNote`/`string`/`boolean`).
- **Cleanup:** removed the `showAll*`/`CAP_*` state + the `.rp-more` button/CSS. (The §H.2 `reviewPanel.more`/`showLess` i18n keys are now unused-but-harmless ×15 — left in place; §F may reuse.)
- (§F will still split this panel properly — note status vs a full-page reviewer — per the MIG-080 plan.)
- **Verify:** `svelte-check` **0 errors**. Release build (npm → cargo) — binary 10:20:32, embed verified.
- **§H.3 Boss test: "Perfect!"** + 1 final ask → §H.4.

## §H.4 — remove the redundant Review tab count badge (Boss) — BUILT
- **Boss:** the "Never Reviewed" section header already shows the count on the right, so the count pill on the Review tab ICON is redundant — remove it.
- **Fix:** the `.rs-tab-badge` (`+layout.svelte:7202`, `{#if dueNotes.length > 0}<span class="rs-tab-badge">{dueNotes.length}</span>`) was the ONLY tab badge → removed the markup + its now-dead CSS (the earlier §H.2 badge-positioning fix is thereby moot, removed with it). `dueNotes` still flows to the panel (only the icon pill went). svelte-check 0; frontend-only.
- **§H.4 Boss test: "Pass."** Binary 10:32:41, `rs-tab-badge` gone from `build/`.

## PCS — §H increment (Boss: "PCS + Orientation", 2026-06-21)
- **Boss-validated, all on `main` (pushed): §0/§B (`f767fca3`), §0b (`0920c5f9`).** This PCS commits the uncommitted **§H / §H.2 / §H.3 / §H.4** (per-panel right-sidebar text size + Review-Pulse polish + badge removal) — 10 panel components + StyleSetter + `+layout` + ReviewPulsePanel + 15 locales + session log.
- **Orientation v2.95** (NEW file, v2.94 retained) — MIG-080 §0/§B/§H, SO #6 (subsystem ships features), in the SAME commit (feedback_orientation_inline_with_commit).
- **Help/Manual (EN):** User Manual §15 (right-sidebar = note-context; Tags note-only; inspector360 placeable; the new per-panel "Right Sidebar Text" sizing) + help-site Panels topic. ×14 manual translations ride the standing debt.
- **MoCh** `docs/MoCh/MoCh-2026-06-20-2000.md` (this session: MIG-082 docs i18n + MIG-080 §0–§H).
- **Editor-Surface Gate:** UNTOUCHED across §0/§B/§H (settings/styling/panel-render/i18n only). §C is the first write-path phase.
- **Carried debt (noted):** `reviewPanel.more`/`showLess` i18n keys now unused ×15 (harmless; §F may reuse); the dead §B toggle CSS (`.rs-tag-toggle`/`.rs-tags-total`/`.rs-tags-header`); `scanAllLibraryTags` orphaned in `tagUtils.ts`. All → §G `/simplify`.
- **NEXT — §C (Tasks):** design analysis `wf_29c533af` (toggle_task reindex Editor-Surface-Gate safety + freeze-risk; left agenda; SecondScreen fix) → build plan + Editor-Surface Gate test.

## ⚑ STATE-OF-STANDING + BRING-UP CROSS-CHECK (Boss redirect 2026-06-21, SO #5 + SO #8)
> Boss re-raised the **bring-up method** (the 2026-06-15 image: disable all but the editor → measure → re-enable one-by-one fixing lag/wiring) and asked to cross-check §C against the pending unfinished tasks. Read: MoCh-2026-06-15-1400, `docs/concept-papers/00-MASTER-Bring-Up-Charter-and-Checklist.md`, session logs 06-16→06-20.

**The bring-up program (Boss-directed 2026-06-15) — foundation BUILT, §D UNSTARTED:**
- §A single-owner activation ✓ · §B `safeBootMode` flag ✓ (store.ts:3604, default false; gates the 4 boot IPCs) · §C graph-WTD = MIG-079 §C.1–§C.2d ✓ (the 30 s killer GONE).
- **Editor baseline MEASURED (00-MASTER §6):** minimal mode `paint 452 · hydrated 588 · graph_ready 603 ms` — the editor+tree boots in **~0.6 s**; the ~32 s was ALL satellites.
- 31 concept papers (02–32) + core + charter ✓. **§7 Debt Register** ✓: Rule-8 recompute ×~14 functions; right-click gaps ×~19 (MIG-077 incomplete); hardcoded English ×~17; 5 confirmed defects.
- **§D (phase-by-phase re-enable, each validated vs its concept-paper checklist) — NEVER EXECUTED.**

**What happened INSTEAD of §D (06-16→06-20):** MIG-079 §C.2b–§C.2d (boot perf — legitimately the bring-up §C) → **MIG-081/082 (calendar features)** → **MIG-080 (right-sidebar note-context)**. The calendar + right-sidebar are FEATURE work; they also ADDED surface that now must pass the bring-up's per-function discipline. §H (per-panel text sizing) was the furthest drift — a Style-Setter feature, neither note-context nor bring-up.

**§C ↔ bring-up cross-check — they are the SAME work, two framings:**
- MIG-080 **§C** (toggle_task reindex) **IS Debt Register E.2** ("Tasks toggle_task bypasses the Editor gate → search/backlinks/tags drift").
- MIG-080 **§F** (Review Pulse) **IS Debt E.4** (`record_note_visit` dead code) + the Review-Pulse Rule-8 (`scan_due_recursive` reads every note/open, Debt A).
- The Tasks panel ALSO has: Debt A (`scan_library_tasks` re-walks per open), Debt B (no right-click), Debt C (i18n), Debt D (no `enabledFeatures` gate). MIG-080 §C only touches E.2 + the left agenda — a SUBSET of the Tasks bring-up.
- **Conclusion:** §C is NOT a detour from the bring-up; it's a corner of the Tasks bring-up (Phase 5). Doing it as a MIG-080 sub-step addresses 1 of ~5 Tasks debts; doing it AS the Tasks bring-up (full concept-paper checklist) addresses all.

**The honest decision point for Boss (see chat):** the boot CRISIS that opened the bring-up is SOLVED (MIG-079); the bring-up's remaining value TODAY is the systematic per-function CORRECTNESS pass (Rule-8 / right-click / i18n / the 5 defects), not boot discovery. Options: (A) finish MIG-080 §C–§G as features; (B) pivot to bring-up §D and fold MIG-080's note-context design into each panel's bring-up; (C) defects-first prioritized subset. Recommended in chat: **reframe — do the remaining panel work the bring-up way** (full per-function checklist), starting with the confirmed defects (E.2 Tasks = §C).
- **BOSS CHOSE: (C-adjacent) "Finish MIG-080 first"** — complete §C–§G, THEN bring-up §D fresh (banked: §C = Debt E.2).
- **Boss also banked the right-click targets:** Obsidian's Note/Folder/Link/editor-empty menus → `docs/concept-papers/Right-Click-Reference-Obsidian.md` + memory `project_rightclick_obsidian_targets`. For the right-click bring-up (Debt B / concept-paper §5). LATER, not now.

## §C.1 — Tasks: toggle_task reindex (Debt E.2) + SecondScreen reconcile — BUILT (pending Editor-Surface Gate test)
- **Freeze-safety VERIFIED (the Editor-Surface-Gate risk):** `index_note` (search.rs:4737–4752) computes the would-be edge set vs stored and **skips the `note_links` DELETE+rebuild (+ the MIG-001 trigger cascade) when `unchanged`**. A checkbox toggle changes no `[[links]]` → `unchanged=true` → rebuild skipped → **no ~40 s freeze** (the `fca3f194` guard). So reindexing on a toggle only does the cheap note_meta/FTS refresh — exactly the drift fix.
- **`tasks.rs` `toggle_task` (Debt E.2):** after `gate_write`, resolve `lib_name` (the bases.rs:722 canonicalized-prefix pattern) + `crate::search::reindex_single_note(&search_state, &file_path, &lib_name)` (best-effort). Fixes the FTS/body drift after a toggle. `cargo check` clean.
- **`SecondScreenPage.svelte` (BUG-015 F2 fold-in, Boss-approved):** `:1345`/`:1478` raw `toggleTask()` → `toggleTaskReconciled` (import moved to `$lib/libraries/store`; removed the now-unused `toggleTask` import). Consistent with the calendar/Tasks-panel/GlobalTasksView callers; the new Rust reindex fires.
  - **⚑ HONEST residual (flagged, NOT half-fixed):** the second screen is a SEPARATE window context → its `openTabs` store is its own instance, so `toggleTaskReconciled` reconciles the SECOND SCREEN's tabs, not the MAIN window's. The swap fixes the second-screen-local dirty case + is strictly better than raw toggleTask, but the full cross-window race (MAIN window holds the dirty tab) needs a display-not-domain refactor — the second screen should ROUTE the toggle to the main window (emit→listen→main reconciles). → flagged for the **Second-Screen bring-up (concept paper #26)**; NOT a rushed partial.
- **Editor-Surface Gate (the §C write-path test):** to run on the binary — toggle a task → FTS/search reflects it (no drift); toggle a task in a note with UNSAVED edits → the toggle sticks AND edits preserved; type-burst persists; Focus round-trip; tab switch; body intact; second-screen toggle round-trip.
- **NEXT in §C:** §C.2 the left task agenda (reuse GlobalTasksView/scanLibraryTasks).
- **§C.1 Boss test: "All pass."** + a NEW finding → §C.2 (natural-language dates) jumped the queue (the left agenda is now §C.3).

## §C.2 — Natural-language task due dates (Boss finding during §C.1 test) — BUILT (pending test)
- **Boss:** tasks don't recognize "today/tomorrow/next week/etc." as dates. **Decisions (AskUserQuestion):** PIN to a real FIXED date (converted visibly on commit, NOT a save-path rewrite); keywords = today/tomorrow/yesterday + next week/next month + weekday names + "in N days/weeks"; recognized **with OR without** the 📅 marker ("so be it" on edge cases).
- **Reproduce-First:** `extract_due_date` (tasks.rs:65) only parsed explicit `YYYY-MM-DD` after 📅/`due::` — no natural language → tasks got no due date.
- **Design (safe — content-integrity class):** "pin" means writing the resolved date into the file. Did it in the **EDITOR** (a visible, undoable edit on Enter-commit), NOT the dangerous save-composition path.
  - **`src/lib/editor/taskDates.ts`** (NEW, PURE): `findTaskDateConversion(line, now)` — detects the earliest NL date keyword in a TASK line, resolves it relative to `now`, returns the span + `📅 YYYY-MM-DD` replacement (absorbs an existing 📅/`due::`; skips non-tasks, explicit dates, and the ✅ completion stamp). **Unit-tested ×8 (`tests/mig-080/taskDates.test.ts`, added to vitest include) — all pass** against now=Sun 2026-06-21 (today→06-21, tomorrow→06-22, next week→06-28, Monday→06-22, next Friday→07-03, in 3 days→06-24, in 2 weeks→07-05, marker-absorb, skip-cases, earliest-wins).
  - **`NotePane.svelte`:** a `convertTaskDateOnEnter` command added to the **`Prec.highest` Enter keymap** (beside `calloutExitOnEnter`); converts then **returns false** so the normal newline/list-continue still runs. NotePane only — **FocusPane stays plain (Rule 1)**. One-line regex scan/Enter → no keystroke cost (Rule 1/3).
- **Verify:** `svelte-check` **0 errors**; vitest 8/8. Release build (npm → cargo re-embed) in progress.
- **Honest caveats (for the test):** (1) trigger is **Enter (line commit)** — typing a date then clicking away without Enter won't convert yet (v1); (2) no-marker matching can false-positive on a sentence ("…plan for next week" → converts) — Boss accepted "so be it"; will tighten if it annoys. (3) The Rust `extract_due_date` still only reads explicit dates — fine, since the editor pins NL → 📅 date BEFORE the scanner sees it.
- **§C.2 v1 Boss test: passed** — but Boss asked to RESEARCH proven methods first (WA#5), to deal with the two caveats.

## §C.2 — WA#5 research (Workflow `wf_cd63316f`, 4 web-research agents) + v2 redesign
- **Research verdict (cited: Obsidian nldates `github.com/argenos/nldates-obsidian`, Obsidian Tasks docs, Todoist/TickTick/MS-To-Do help, Things 3 culturedcode, `github.com/wanasit/chrono`):** the proven systems NEVER disambiguate prose-date-vs-due-date by NLP heuristics. They use a **structural gate** — a trigger char (nldates `@`), a task-line gate + signifier (Obsidian Tasks), a dedicated field (Things), or highlight + one-click-undo + a global toggle (Todoist/TickTick). chrono itself has NO prose filter. **My v1 (whole-line auto-convert on Enter) is the one approach none of them use** — risky in free-flowing note prose.
- **Boss decision (AskUserQuestion):** switch to **autosuggest-you-accept** AND add the Obsidian `@` trigger ("@today" etc.) — "both are correct; if a user forgot @, the autosuggest will suffice." + **a Settings toggle** (table-stakes per the research).

## §C.2 v2 — autosuggest (@ trigger + bare fallback) + Settings toggle — BUILT (pending test)
- **`taskDates.ts`:** replaced `findTaskDateConversion` (auto-convert) with **`taskDateCompletions(before, now)`** — pure: (A) `@`+partial → the keyword menu filtered by the partial (today/tomorrow/yesterday/next week/next month/weekdays/next-weekdays + dynamic "in N days/weeks"); (B) a COMPLETE bare keyword at the cursor → one resolved suggestion (the "forgot @" fallback). **Unit-tested ×10** (`tests/mig-080/`, all pass): @tomorrow→06-22, @to→{today,tomorrow}, @friday→06-26 vs @next friday→07-03, bare "next week"→06-28, "discuss the plans"→null, partial "tomo"→null-until-complete.
- **`completions.ts`:** `createTaskDateCompletion(isEnabled)` — a thin CM6 `CompletionSource`, **gated on `TASK_RE`** (cursor on a `- [ ]` line → prose lines inert, the false-positive solved STRUCTURALLY) + the Settings toggle; `filter:false` (pre-filtered by the @-partial; labels are dates). Offers `📅 YYYY-MM-DD`; accept → inserts. **REMOVED** the v1 `convertTaskDateOnEnter` keymap from NotePane.
- **`NotePane.svelte`:** `taskDateCompletion` added to the autocompletion `override` (first), gated on `$appSettings.naturalLanguageTaskDates ?? true`. NotePane only (FocusPane plain).
- **Settings:** `appSettings.naturalLanguageTaskDates` (default true; `{...DEFAULT_SETTINGS,...parsed}` merge → no migration) + a **Settings → Editor toggle** ("Natural-language task dates") + i18n ×15 (`settings.editor.taskDates`/`taskDatesDesc`; @tokens + 📅 kept literal; ar = تواريخ المهام باللغة الطبيعية).
- **This resolves BOTH caveats:** the task-line gate makes prose false-positives structurally impossible; the suggestion (accept-to-pin) means nothing converts unless the user picks it. `svelte-check` 0; vitest 10/10. Release build in progress.
- **§C.2 v2 Boss test: "WOW… great job… one of our milestones."** → MILESTONE PCS: commit `585443c2` (§C.1+§C.2, 27 files) + orientation v2.96 + push + **tag `milestone/mig-080-tasks-nldates`** + ZIP backup (E:/Backups, 167 MB).

## Backup & Recovery system — WANTED FEATURE banked (Boss 2026-06-21)
- Boss: "Create a backup system… users have peace of mind that, in the worst-case scenario, they have a safety net to secure their PKM/PKF." → banked (not started; after MIG-080, via /migration + WA#5). **`docs/concept-papers/Backup-System-Concept-Paper.md`** (intent + the hard constraints: File-Over-App restorable-without-Constellation, Local-First offline/user's-cloud-choice, the SQLite index rebuilt-on-restore-not-backed-up, archival-not-deletion; options local-snapshots/Git/per-note-version-history/soft-delete; open questions; prior art) + memory `project_backup_system_wanted`.

## §C.3 — universe Task agenda → left dock (relocated out of the note-context right rail) — BUILT (pending test)
- The note-context right rail keeps the OPEN note's Tasks (§C.a, done); the UNIVERSE/library task agenda relocates LEFT, per the MIG-080 disposition. **Reuse, not reinvent:** `GlobalTasksView` already renders the universe tasks (filters + date-grouping) but was only reachable via the command palette. Added a **left-dock "Tasks" button beside the Calendar** (`+layout.svelte` dock, modeled on the Calendar button; toggles `showGlobalTasks`, resets the other full-page views). Reused the existing `commands.globalTasks` label (×15 — **zero new i18n**) + an inline checklist SVG. `svelte-check` 0; frontend-only. Binary 13:31.
- **§C.3 Boss test: "Pass."** + 2 asks → §C.3b.

## §C.3b — Global Tasks: follow the universe theme + a Style-Setter tab (Boss asks) — BUILT
- **Bug (Reproduce-First + diagnosed):** the Global Tasks view rendered **dark regardless of the universe theme**. Root cause: its background used `var(--bg-primary, #1a1a1a)` — but **`--bg-primary` is NOT defined** in `theme.css` (the real var is `--background-primary`) → it always fell back to the dark literal. `--border-faint` was also undefined. The other 6 vars resolved fine.
- **Fix (the Calendar-Style-Setter pattern, one deterministic rewrite — 32 replacements):** every surface → **`var(--gt-X, var(--real-theme-var, literal))`** — fixes the broken names (`--bg-primary`→`--background-primary`, `--border-faint`→`--border-light`) so it **follows the universe theme**, AND adds the `--gt-*` override layer. Tokens: `--gt-bg / surface / text / muted / accent / border / hover / overdue / today`.
- **Style Setter "Global Tasks" category** (`StyleSetter.svelte`): a `globalTasks` element (9 colour controls on the `--gt-*` tokens) + a two-zone LIVE category. No local `--gt-*` declaration on `.global-tasks` (inline-var on each surface) so the body-level Style-Setter override wins (the BUG-015 single-writer + the Calendar `--cal-*` precedent). 
- **i18n:** labels via the `L()` fallback (works in EN now; shared labels Background/Text/Accent/Border already ×15). The 6 new labels (Global Tasks / Surface / Muted text / Row hover / Overdue date / Due-today date) → ×14 in the §C PCS.
- **Verify:** `svelte-check` 0; frontend-only. Binary 14:00. **§C.3b Boss test: "Pass."** + ask → §C.3c.

## §C.3c — Global Tasks text resizing (Boss ask) — BUILT
- Boss: add Text resizing to Style Setter → Global Tasks. Same proven pattern as the right-sidebar text scale: **14 font-sizes** wrapped `calc(X * var(--gt-scale, 1))` (rem only; no em); `.global-tasks { --gt-scale: calc(var(--gt-text-scale, 100) / 100) }`; a **"Text size" range control** (70–140, def 100) added to the StyleSetter `globalTasks` element — **reuses the `text_size` i18n label** (already ×15). Text-only; default 100 = identical. `svelte-check` 0; frontend-only. Binary 14:16. **§C.3c Boss test: "Pass." → §C COMPLETE.** PCS'd: commit `039e3ddf` (21 files: dock agenda + theme fix + Style-Setter tab + text size + 7 labels ×15 + backup paper + orientation v2.97), pushed.

## §D — Source Review split (note-context right rail vs the universe Cataloger) — BUILT
> **Function in hand:** MIG-080 §D — the right-rail Source Review shows THIS note's pending source suggestions; the universe queue stays in the Cataloger.
- **Verified the IPC semantics first (not assumed):** `sources_get_suggestions(note_path)` → `Option<SuggestionRecord>` (this note's record via `read_suggestions WHERE note_path=?`); `sources_list_pending_suggestions()` → the universe `Vec`. The right-rail mount (`+layout.svelte:7426`) ALREADY passes `activeNotePath={sidebarTab?.path}`; `loadQueue` just ignored it. CatalogerView mounts the panel WITHOUT `activeNotePath` (universe — confirmed, only a comment mentions it).
- **Change (small, frontend-only, no new IPC):** `loadQueue()` branches on `activeNotePath` — set → `sources_get_suggestions` (`queue = rec ? [rec] : []`); null → the universe list (Cataloger). + a `$effect` that reloads when the open note changes (first run skipped — onMount loads; Cataloger keeps `activeNotePath` null so it never reloads). + `'sourceReview'` added to `NOTE_SCOPED_TABS` → the right rail shows "no note selected" when none open and renders per-note otherwise, via the SAME proven gate as properties/backlinks/tasks (no duplicate render — single `{#if}` chain). Fixed the stale "(tags/…/sourceReview) are universe-wide" comment (drift: §B made tags note-scoped, §D makes sourceReview note-scoped).
- **Verify:** `svelte-check` 0; binary 16:56.

### §D adversarial review (ultracode — 4-lens workflow, then verify each finding) — caught 2 P1s BEFORE the Boss test
> WA#4 + secure-don't-muddle: §D touches a read path + reactivity + a cross-surface gate, so it got a parallel review (reactivity / read-path / gate / regression lenses → adversarial verify). **7 findings, ALL confirmed high-confidence (0 dismissed).** §D-as-built was UNSAFE to test. All 7 fixed:
> 1. **P1 (DESTRUCTIVE) — universe-wide "Reject all" in the per-note rail.** The bulk Approve-all/Reject-all buttons rendered for any non-empty queue with NO `activeNotePath` gate; `sources_reject_all_pending` = `DELETE FROM sources_suggestions` (no WHERE) → one note's rail could wipe EVERY pending suggestion in the Universe. **Fix:** `{#if !activeNotePath}` around `.srp-bulk-actions` (+ gated the filter-chip row & trust-cal banner the same way — universe-only chrome). Per-note rail = per-card Accept/Reject only.
> 2. **P1 — right-click "Suggest sources" broke.** After §D made the rail note-scoped, the file-tree right-click (arbitrary note, maybe not open) either no-op'd (panel unmounted, no listener) or contaminated a different note's scoped rail. **Fix:** `handleSuggestSourcesForNote` now routes to the **Cataloger** (the universe queue's home post-§D), mirroring the `case 'cataloger'` reset + classify-and-show dispatch.
> 3. **P2 — contamination via the global `classify-and-show` listener.** **Fix:** `handleClassifyAndShow` now ignores foreign events on a note-scoped instance (`if (activeNotePath && notePath !== activeNotePath) return`); the universe Cataloger instance (activeNotePath null) still prepends — cross-instance sync intact (verified `classifyActiveNote` self-dispatches with `notePath === activeNotePath`).
> 4. **P2 — rapid note-switch race.** `loadQueue` is async with no sequencing → a stale response could overwrite a newer note's. **Fix:** monotonic `_srpLoadSeq` cancel-previous guard (the pattern CLAUDE.md already mandates for async loads).
> 5. **P3 — stale render-branch comment** ("works with or without an active note") → updated to the note-scoped reality.
> 6. **P3 — `read_suggestions` swallowed real DB errors as None** (blanket `.ok()`) → a locked/corrupt DB looked like "Nothing to review". **Fix:** match `QueryReturnedNoRows` → `Ok(None)`, propagate other `rusqlite::Error` → `Err`.
> 7. (the right-click finding had a P1 + a P3 facet — both covered by #2 above.)
- **Verify:** `svelte-check` 0; frontend + Rust release. Binary 17:18. **§D Boss test (2 stages): "Pass."** §D COMPLETE (wiring + all 7 review fixes). + ask → §D-text.

## §D-text — Cataloger text-resize slider (Boss ask) — BUILT
- Boss: add a text-resize slider to the **Cataloger** Style Setter (the category already existed with just an `accent` colour). **Key reuse:** the embedded `SourceReviewPanel` font-sizes were ALREADY `calc(X * var(--rs-scale, 1))` (from §H's right-sidebar per-panel scale). So: `.cataloger-view { --cat-scale: calc(var(--cat-text-scale,100)/100) }`; the 8 CatalogerView chrome font-sizes wrapped `calc(X * var(--cat-scale,1))`; **`.cataloger-queue { --rs-scale: var(--cat-scale) }`** drives the embedded panel's existing scale — so the slider scales the cards too. The RIGHT-RAIL SourceReviewPanel is a separate instance with its own `--rs-scale` (from `.rs-inner`) → unaffected (verified the two wrappers are distinct). StyleSetter: a `cataloger` element (Text size) added to the existing `cataloger` category (`elements: ['accent','cataloger']`). **Zero new i18n** — both `cataloger` + `text_size` labels already ×15. `svelte-check` 0; frontend-only. Binary 17:44. **Cataloger text-size Boss test: "Pass."** PCS'd: commit `9b9a129b` (§D wiring + 7 review fixes + Cataloger slider + orientation v2.98), pushed.

## §E — Knowledge Health split (note-context Health tab vs universe KH Dashboard) — BUILT
> **Function in hand:** MIG-080 §E — the right-rail Health tab shows THIS note's tensions; the universe Knowledge Health stays in the full-page KnowledgeHealthDashboard.
- **Grounded against the plan + code first.** The right-rail `health` tab renders **TensionPanel** (not KH Dashboard), fed by `loadTensionReport` which called `detect_tensions` = **library-wide** tensions (cached per-library). §E makes it note-scoped.
- **Rust — new IPC `detect_note_tensions(library_path, library_name, note_path)`** (tension.rs): reuses `load_notes_from_db` + `detect_from_notes`, then post-filters — `contradictions/orphans/single_points` by `note_path ==`, `structural_gaps` by whether the cluster's note-NAMES include this note (lower-cased from the loaded map). `total_linked_notes/total_notes/active` stay library-wide (the "<50 linked" inactive state still reads). Registered in lib.rs.
- **Frontend** (`+layout.svelte`): `loadTensionReport` → `detect_note_tensions`; cache guard per-NOTE (`_tensionNotePath`); **added a cancel-previous `_tensionSeq` guard proactively** (the §D race lesson). The mount passes `noteContext={sidebarTab?.name}`.
- **TensionPanel**: new `noteContext` prop + a `noteHasTensions` derived + a note-aware "Analyzing this note's health…" string + a **positive "✓ No tensions detected for this note" state** when note-scoped & clean (instead of four empty headers). Library mode unchanged. 2 new strings (`analyzingNote`, `noteHealthy`) ×15.
- **Verify:** i18n ×15 merged; build clean (binary 18:16). **Adversarial review (4-lens, 15 agents) → 10 confirmed findings — exposed that the PLAN's server-side per-note filter was the wrong shape. Redesigned to Option B before any Boss test.**

### §E review → Option B redesign (the server-side filter had 2 real flaws)
> The plan's `detect_note_tensions` (post-filter in Rust, per note) drew two real defects:
> 1. **Correctness (P2):** `structural_gaps` filtered against `GapItem.notes` — but that's `.take(5)` of a **HashMap-ordered** cluster, so a note genuinely in a >5-member gap was dropped **non-deterministically** → could show the "✓ healthy" state while in a gap.
> 2. **Perf regression (P2, Rule 8):** the full library detection re-ran on **every note-switch** (was cached per-library before §E). The concept paper `23-tension-provenance-source.md` already flags the per-focus recompute as a bring-up item; §E made it per-note-switch.
> 3. + 3× "detect_tensions now orphaned" (P3) — an artifact of the server-side approach.
>
> **Option B (the correct shape):** detect the **library** report ONCE (cached per-library, as before §E — `detect_tensions` stays the live IPC, resolving the orphan findings), and **slice to the note client-side** (`filterTensionsForNote`, a cheap array filter — Rule 3 OK, it's filtering a computed report, not parsing notes). For correct gap membership, Rust `GapItem` gained a **full `member_names`** list (the client filters on it, not the truncated `notes`); the 5-name display is now **sorted (deterministic)**. `tensionReport` is a **`$derived`** of `(tensionReportFull, openNote)` → re-slices on note-switch with **zero** new Rust calls. This removes the §E perf regression (back to per-library cache) and the correctness bug.
> - **Deferred (documented, not §E-blocking):** the *pre-existing* per-library-focus recompute (the real write-time-derivation fix is a tracked bring-up item); target-of-contradiction shows on the source note; cache not invalidated on same-note edit (read-only panel); not-in-index note reads as clean (P3 edges).
- **Verify:** Option B build clean (binary 18:31). Then Boss established a **TOP PRINCIPAL: "if we discover it, we fix it"** (interrupted to say it) → fixed ALL remaining findings, no deferral. New memory `feedback_fix_all_discovered`.

### §E — ALL 10 findings fixed (Boss principle: fix all discovered)
- **#1/#3 (gap truncation):** `GapItem.member_names` (full, sorted, deduped) + client filters on it; display `notes` sorted-then-take(5) (deterministic). ✓
- **#2 (perf recompute):** Option B — `detect_tensions` cached per-library, `tensionReport` is a `$derived` client-side slice (`filterTensionsForNote`). ✓
- **#4/#6/#8 (orphaned IPC):** `detect_tensions` is the live IPC again. ✓
- **#5 (target-of-contradiction):** Detection 1 now emits BOTH perspectives — source `contradicts X` + target `contradicted by Y` (TensionItem doubled per pair; consumed only note-scoped). A contradicted note is no longer "healthy". Test updated (was `len==1` → now `len==2`, asserts both `detail_kind`s). ✓
- **#7 (no refresh on edit):** `+layout` listens `screen:note-saved`; when the OPEN note (Health tab active) saves → invalidate `_tensionLibPath` + reload report + status. ✓
- **#9 (unindexed → false healthy) + #10 (duplicate-title → false healthy):** new O(1) IPC `note_tension_status(libraryName, notePath) → {indexed, ambiguous_title}` (two indexed lookups, NOT the detection); `TensionPanel` gains a `noteStatus` prop → honest "not analyzed yet" / "shares its title — can't reliably assess" states instead of the ✓. ✓
- **Discovered-while-fixing (also fixed, per the principle):** the tension **detail strings were hardcoded English** (the only un-i18n'd user text on the Health surface). Added `detail_kind`+`detail_args` to TensionItem (4 detections set them; `detail` kept as EN fallback + test oracle); `TensionPanel.localizedDetail()` renders via `$t`. **6 new strings ×15** (`noteNotAnalyzed`, `noteAmbiguous`, `detail.{contradicts,contradictedBy,orphan,singlePoint}` — placeholders `{0}`/`{1}` preserved).
- **Verify:** i18n ×15; `svelte-check` 0; `cargo test tension` 5/5; build clean (binary 19:02).

### §E fix-pass RE-REVIEW (4-lens, 11 agents) → 7 more findings, ALL fixed (the principle compounds)
> Reviewing the fixes found 7 more (2 P2, 5 P3) — fixed every one:
> - **P2 #1 — stale-status flash:** `tensionReport`/`noteContext` are synchronous `$derived` but `noteStatus` is async → on a same-library note-switch the panel briefly judged the NEW note against the OLD note's status (false ✓ / false ⚠). **Fix:** reset `tensionNoteStatus = null` before the await + a **pending branch** (clean + status-null → "Analyzing…", never a premature verdict).
> - **P2 #2 — Rule-8 re-detect per autosave:** `screen:note-saved` fires every ~1.5 s while typing; the #7 listener re-ran the FULL library `detect_tensions` each tick. **Fix:** a 2.5 s trailing **debounce** (cleared in cleanup) coalesces the burst into one re-detect after edits settle.
> - **P3 #3** TS interfaces gained `detail_kind/detail_args/member_names` (dropped the `any`). **P3 #4** ambiguous-titled note WITH tensions now shows a `⚠` caveat banner above the rows (reuses `noteAmbiguous`, no new i18n). **P3 #5** `note_tension_status` logic extracted to a pure `compute_note_tension_status` + a unit test (indexed / unique / shared-title / cross-library-not-ambiguous). **P3 #6** save-listener guard gained `&& isHome` (matches the trigger `$effect`). **P3 #7** self-`contradicts` note now emits ONE row, not two (+ a test).
- **Verify:** `svelte-check` 0; `cargo test tension` 7/7; build clean (binary 19:23). **§E Boss test (2 stages): "Pass." → §E COMPLETE.** PCS'd: commit `ef93c742` (Option B + 17 fixes + 2 principles + orientation v2.99), pushed.

## Concept-Before-Function catch → MIG-083 (Review Pulse Rule-8 redesign) ARCHITECT
> **Boss principle set mid-session (2026-06-21):** *"A concept is like a horse pulling a carriage; a function without a concept is a carriage without a horse."* Canonised in CLAUDE.md (Architecture Principles) + memory `feedback_concept_before_function`. (Also #6 "Fix what you discover" → Working Agreement #6 + memory.)
- **Applied immediately:** before building MIG-080 §F (Review Pulse split), Boss asked **"do we have a concept paper?"** → `docs/concept-papers/22-review-pulse.md` exists + is sound, AND it flags a **Rule-8 blocker** the §F plan ignored: `get_due_notes` re-walks the whole library FS (+ a full content read of every `.md` for `#assumption`/`#model` checkpoints) on every call. So §F's full-page reviewer would be a carriage with no horse. **Boss ruled: fix Rule 8 first → MIG-083.**
- **MIG-083 Architect** (`docs/MIG-083-Architect-Review-Pulse-Rule8.md`, via a 5-agent `/migration` Architect workflow + WA#5 research): RECOMMENDED **Option A** — a derived `review_schedule` SQLite table (indexed `due_days`), maintained **write-time in `index_note`** exactly like `tag_counts` (MIG-079 §C.1); reads become `SELECT … WHERE due_days <= today` (<100ms vs the FS walk); resumable post-paint back-fill (`sky_backfill` cursor template); `review-pulse.json` stays the durable action source (File-Over-App). Rejected B (widen `note_meta`) + C (cache in JSON). Matches the proven Anki/SM-2/FSRS indexed-`due` pattern. 8 invariants (I1–I8) + a `rehearse_against_live_copy` harness.
- **Boss decisions (before the Plan):** (1) **checkpoint = `tags_json`** — resolved via a **live-corpus diff**: scanned all 7,609 notes of `E:/Cognitive Knowledge`, found **0** `#assumption`/`#model` checkpoints under either definition (today's scan reads 7,611 files for nothing — pure Rule-8 waste); `tags_json` is the risk-free, future-correct superset. (2) **Fix all 3 quirks in the swap** (real stratum · 1·3·7·14·30 ladder · the `stale` reason) — so the rehearsal harness asserts the CORRECTED behavior, not parity.
- **NEXT:** the MIG-083 **Plan** (phase-by-phase, landable + verifiable) → Boss approves → Build → Audit. Then MIG-080 §F (the split, reading the cheap schedule) → §G.
- **Plan written + Boss-APPROVED** (`docs/MIG-083-Plan.md`, commit `87fff1c1`): 5 phases (§A schema+corrected logic · §B write-time maintenance · §C back-fill · §D read swap+harness+Boss test · §E cleanup+audit).

## MIG-083 build §A — STOPPED on architectural surprise → the `stale` quirk had no concept → defined Mode 2 (WA#5-validated)
> **Concept-before-function caught its own migration.** Reading `review.rs` for §A: the "fix all quirks" decision split into 2 buildable (ladder `1·3·7·14·30`; real stratum from `sky_nodes.stratum`/MIG-002) + **1 underspecified — `stale` (Mode 2) was never implemented, only declared, with no concept.** Per the principle, stopped + surfaced. Boss chose **"define the staleness concept now."**
- **Mode 2 defined (Boss, 2026-06-22):** a note is STALE when a **load-bearing OUT-dependency** (`supports`/`contradicts`/`derives-from`/`part-of`/`supersedes`; NOT `associative`) had its **content change** (not a touch) **after its last explicit ✓ review**; all maturities; 1-hop; ranked+explained; a **separate lens** from Mode 1.
- **WA#5 research (4-angle workflow) VALIDATED + REFINED it:** it's **changed-dependency invalidation / Truth-Maintenance-Systems** (Doyle/de Kleer) applied to a knowledge graph — **novel in PKM** (no SR tool triggers on a neighbour changing) with a literal analog in **dosu.dev's "Git Age Delta."** The research caught that the Boss's first answers (either-direction · any-change) would **over-fire** (the backlink-noise/badge-blindness death) → scoped to: OUT-dependencies, load-bearing types, **content-hash** (not mtime — autosave/sync false-fire), 1-hop. 4 Boss decisions confirmed: out-only · load-bearing-only · **explicit-✓-only `last_reviewed`** (→ `record_note_visit→openNoteTab` §F wiring **dropped**) · **two separate lenses**.
- Recorded: concept paper §12 (`22-review-pulse.md`); Plan updated (§B `content_changed_at` write-time signal; §D the staleness JOIN; §F record_note_visit dropped). **NEXT: build §A** (now with the corrected ladder + real stratum + the Mode-2 predicate as tested pure logic).
- **§A BUILT + committed `c30431e6`** — inert `review_schedule` table + corrected pure logic (4 tests). **§B-1 BUILT + committed `73a79745`** — gated DB-maintenance fns + corrected `mark_reviewed` ladder + `record_note_visit` removed (5 tests; 18 review tests total). Everything INERT (gated on `schema_versions.review`, unstamped until §C).

## ⏹ SESSION CLOSE (state of standing, 2026-06-22) — full PCS done
> Closed at MIG-083 §B-1 on my recommendation: §B-2 is the `index_note` hot-path wiring and deserves fresh eyes (secure-don't-muddle), with multiple open sub-questions.
- **Verified-shipped + protected:** MIG-080 §B–§E (note-context right sidebar, Boss-validated, 17 review fixes); 2 new top-principals (Concept-Before-Function, Fix-What-You-Discover) in CLAUDE.md + memory; MIG-083 §A + §B-1 (committed, tested, **inert** — gated off).
- **In-flight / next:** MIG-083 §B-2 (hot-path hooks + action-writer row-sync) — fully specified in `lab/reports/HANDOVER-2026-06-22-mig083-b2.md` (incl. open sub-questions: `sky_nodes.stratum` TEXT format; `content_changed_at` → §D). Then §C→§D (Boss test)→§E→ MIG-080 §F→§G.
- **Known-broken:** none. **Uncommitted:** none (everything pushed).
- **Doc-drift to fix in §E close-out:** MIG-080 §F help/manual updates (the note-context panels are user-facing) were not written this session — owed at MIG-080 §G.
- **Close-out artifacts:** orientation **v3.00**; MoCh `MoCh-2026-06-22-0900.md`; handover + next-session prompt `HANDOVER-2026-06-22-mig083-b2.md`; this state-of-standing.

## ▶ Session CONTINUED past the close (Boss "Proceed" ×N) — MIG-083 §B-2 + §C built
- **§B-2 committed `2200ddc0`** — wired the gated `review_schedule` write-time maintenance into the hot path: `index_note` upsert (stratum via `CAST(sky_nodes.stratum AS INTEGER)`), `reindex_delete_note` drop, and the action-writer row-sync (mark/snooze/dismiss → row via `app.try_state::<SearchState>`). `record_note_visit` already removed in §B-1. 11 tests.
- **§C committed `a98b52e7`** — `review_backfill.rs`: resumable post-paint back-fill (cursor, 1000-row batches, 50ms sleep, idempotent `INSERT OR REPLACE`), scheduled at boot; **`finalize` stamps `schema_versions.review=1` + wipes cursor in one txn = the go-live moment** (flips §B from inert to live; reads stay legacy until §D). 12 tests.
- **Still INERT** — nothing stamps until a shipped build runs the back-fill; reads don't swap until §D.
- **RESUME AT §D** (read swap + Mode-2 staleness JOIN + `content_changed_at` + the live-copy rehearsal harness + **Boss test**). Spec + ready-to-paste prompt in `HANDOVER-2026-06-22-mig083-b2.md` ("RESUME HERE — §D"). Re-closed here: §D is the biggest, user-facing, Boss-test-bearing phase + the novel staleness feature — deserves fresh eyes (secure-don't-muddle).

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

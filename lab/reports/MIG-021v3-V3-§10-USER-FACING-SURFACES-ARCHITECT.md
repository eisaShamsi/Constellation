# V3-§10 — User-Facing Surfaces — Architect Document

**Date:** 2026-05-11  
**Author:** Claude (audit + design)  
**Status:** Architect Phase — awaiting Boss approval to proceed to Plan phase  
**Predecessor:** V3-§9 (vertical-axis activation, Gate 2 PASS, orientation v1.92)

---

## §1 — Where we actually are (current user-facing state audit)

V3-§1 → V3-§9 built the engine. V3-§10's job is the user-facing surfaces around it: Settings, i18n, Help docs, User Manual. The Boss now has a 6-cataloger ensemble producing classifications with friendly per-cataloger trails and a Library-scale review queue with composition filter — but the only place the engine is mentioned in the app's user-facing chrome (outside the Source Review panel itself) is one Settings button labeled "Run classification scan."

**What's already in the app today:**

| Surface | Current state | Gap |
|---|---|---|
| **Source Review panel** | Full chrome shipped through r5/r6/r7/r8 + V3-§9 — dot cluster, friendly rule chips, trust-cal banner, queue Split chip + filter, Sibling Disambig form, Approve All Split-aware, Legacy pill | None — it's the engine's primary surface and it's complete |
| **Right-click context menu** | "Suggest sources & content type" entry wired (`+layout.svelte` dispatches `constellation:classify-and-show`) | None — covers both axes |
| **Settings UI** | One section ("Sources & content type classifier") at `SettingsModal.svelte:1779-1796` with a single "Run classification scan" button | No CECE-specific framing, no Reasoning Cataloger model status, no reasoning trail visibility toggle, no per-Library calibration view, no background scan toggle |
| **i18n** | `cece.*` keys (~75 keys: cataloger labels, regimes, trail, rules, trustCal, badge, queueSplit, queueFilter, disambiguation) populated in `en.json` + `ar.json` only | 13 other locales (de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh) fall back to inline EN defaults |
| **Help docs** | `Cognitive Engine` topic exists but has zero CECE content. No dedicated CECE / Source Review / Cataloger Ensemble topic | Need a new help topic explaining the six catalogers + three regimes + disambiguation form + when to trust |
| **User Manual** | `docs/User Manual.md` has zero CECE content | Need a CECE chapter |
| **User Manual translations** | 14 translation directories exist in `docs/help.{lang}/`. Most have stub structure (Getting Started + Appearance + a few core topics). Per PJ-014 pattern, full translation backfill is queued as separate work | Same — translations of the new CECE chapter would need to land separately |

**Notable findings from the audit:**

1. **Per-Library reliability data has no visualization surface.** V3-§9.C wired the JSON updates; V3-§9.C.2 closed the dual-axis gap. The data is there. But there's no UI showing the user "Linguistic is 95% accurate on horizontal in this Library, 60% on vertical" — that's the per-Library calibration view the original V3-§10 plan mentioned.
2. **Reasoning Cataloger model is deferred but the Settings UI has no acknowledgment.** Today the green Reasoning dot is silent on every card (per Gate 2 Stage 4). A user discovering CECE for the first time has no way to know "the AI judgment lens isn't running yet because llama.cpp isn't wired" — which would let them set expectations and watch for the V3-§7.b update.
3. **Reasoning trail visibility is hardcoded to the trust-cal heuristic.** Currently r5.3 auto-expands trails for the first 50 reviews, then collapses. A user who wants trails always-visible (advanced user) or always-collapsed (rubber-stamp mode) has no override.
4. **Background scan is manual-only.** The "Run classification scan" button is a one-shot trigger. No "auto-classify on note save" or "auto-classify at startup" toggle, even though the scan is resumable + non-blocking.
5. **Help topic naming convention.** Existing help topics use the singular noun form (`Cognitive Engine`, `Sky View`, `Lens`). The new topic name should follow — `CECE` is too acronym-heavy for the help directory; better as `Source Review` (the user-facing surface) or `Constellation Epistemic Content Engine` (the formal name) or extend the existing `Cognitive Engine` topic with a CECE section.

---

## §2 — Invariants that must not break

(Most are inherited from V3-§8 + V3-§9.)

1. **No new feature regresses boot time, typing latency, or IPC responsiveness on a 7,600+ note Library.** Settings UI changes don't fire IPCs on the keystroke path; per-Library calibration view reads the JSON file once on Settings open, not on every paint.
2. **No new feature changes existing Source Review behavior.** Every change ships behind a Settings flag where the existing default behavior is preserved unless the user explicitly opts in.
3. **i18n complete on user-facing strings** — every new Settings label / Help text / User Manual snippet goes through `$t()` with EN + AR populated from day one. Other 13 locales fall back to inline EN defaults (existing pattern from V3-§8.r5).
4. **Honest accuracy framing per Architect §10 invariant 10**: no overpromising. The per-Library calibration view says "Linguistic correctly classified 47 of 52 horizontal-axis notes you've reviewed in this Library" — explicit, evidential, no hand-waving. The Reasoning Cataloger status says "Not downloaded — local AI judgment lens deferred to V3-§7.b" — honest.
5. **Settings flag persistence**: any new flag goes through the existing `appSettings` store with a default that preserves the current behavior. New flags are documented in the Settings help.
6. **Help docs follow the existing topic style**: paragraph-driven explanations with one feature per `## H2`, examples in fenced code blocks, plain language (no internal component names like "SourceReviewPanel.svelte" or "synthesis.rs"; user-facing labels only).
7. **User Manual chapter integrates with the existing structure**: follows the manual's existing chapter conventions (intro paragraph, then numbered subsections, then "Common workflows" appendix).
8. **No regressions on the per-Library calibration data**: the read view is read-only. Editing reliability counters from the UI is out of scope — that's what corrections are for (V3-§9.C wired updates from Accept/Disambig).

---

## §3 — Three scope options for Boss decision

### Option A — Minimal (Settings only, ~2-3 hrs, 2 commits)

- **Settings UI section** "Constellation Epistemic Content Engine" inside Intelligence:
  - Reasoning Cataloger model status (read-only line: "Not downloaded — local AI judgment lens deferred to V3-§7.b. When llama.cpp wiring ships, you'll be able to download Qwen3-4B Q5_K_M from this panel.")
  - Reasoning trail visibility (radio group): Always show / On disagreement only (default — current behavior) / Always hide
  - Background scan trigger (auto-classify): Off (default — current behavior) / On note save / On app start
  - Per-Library calibration view (collapsible read-only section): renders the per-cataloger per-axis accuracy from the reliability JSON. "Linguistic — Horizontal: 12 correct / 2 wrong (86%); Vertical: 8 correct / 4 wrong (67%)" etc. Note count required for stable accuracy: 20 (matches `MIN_SAMPLES_FOR_WEIGHTING`).
- **i18n keys** for all new Settings strings in `en.json` + `ar.json` (~15 new keys).
- **No help docs / User Manual**: deferred to Option B.

**What this yields:** the engine's user-facing surface is feature-complete in Settings. Users can see per-Library calibration data + control Reasoning trail visibility + opt into background scanning.

### Option B — Standard (Option A + help topic + User Manual chapter, ~5-6 hrs, 4 commits)

Includes Option A plus:

- **New help topic** `docs/help.uConstellation.World/Source Review/Source Review.md` covering:
  - What CECE is (one paragraph plain language)
  - The six catalogers explained for a non-expert user (one paragraph each, no jargon, with the lens-color guide)
  - Three confidence regimes (Unanimous / StrongMajority / Split) with examples
  - Sibling Disambiguation form walkthrough
  - The queue composition filter (5 chips)
  - Trust-calibration period (first 50 reviews)
  - When to Accept vs Reject vs Edit vs Disambig pick
  - Reasoning trail interpretation (the friendly rule chips)
- **User Manual chapter** in `docs/User Manual.md` covering the same ground at User Manual depth (cross-references to the help topic for screenshot-level detail).
- **Naming decision**: help topic is "Source Review" (user-facing surface name) NOT "CECE" (acronym).

**What this yields:** users can self-serve answers to "what does this badge mean?" / "why is this card Split?" / "how do I disambiguate?" / "what does it mean when only Linguistic is voiced?" without having to ask Boss.

**What this leaves on the table:** translations of the help topic + User Manual chapter. Per PJ-014's existing pattern, those land later.

### Option C — Full (Option B + 13-locale i18n backfill + 14-language User Manual translations, ~12-15 hrs, 6+ commits)

Includes Option B plus:

- **Translate all 75 `cece.*` i18n keys** into the 13 other locales (de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh). Currently fall back to inline EN defaults.
- **Translate the new help topic** into all 14 non-English help directories.
- **Translate the new User Manual chapter** into all 14 translations.

**What this yields:** complete i18n parity across all 15 locales for the new V3-§10 surfaces.

**Cost:** 13 + 14 + 14 = 41 translation passes. Each translation pass needs Eisa's review (or a bilingual translator's). The agent-time cost is moderate (~1hr per locale for high-quality translation), but the *Eisa-review* cost is high (Eisa is the reviewer for AR + EN; he can't validate ja/zh/he/etc. translations himself, so they'd need to ship as best-effort agent translations subject to community correction).

This is the same trade-off PJ-014 identified — full translation backfill is its own MIG, not a phase of V3-§10.

---

## §4 — Recommended option

**Option B.** Rationale:

- **Option A alone** ships the Settings without the documentation backstop. New users hitting CECE for the first time would have to figure it out from the UI alone (the panel is well-designed but six catalogers + three regimes + disambig form is a lot of new vocabulary).
- **Option C** (full translation) is the right end-state but the breadth makes it a separate effort. Per the existing pattern (PJ-014 for User Manual translations, PJ-019 for some i18n backfills, etc.), translation work is its own discipline that deserves a focused MIG with translator review cycles.
- **Option B** lands the engine's user-facing surface complete in en + ar (the two locales Eisa actively maintains). Translation backfill becomes V3-§10.x or PJ-NNN, scheduled when there's translator bandwidth.

The Settings UI work in Option A is the highest-impact / lowest-risk part. The help topic + User Manual chapter give users self-serve documentation. Translations are the long tail that doesn't block any critical workflow.

---

## §5 — Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Settings flag default change accidentally regresses the Reasoning trail visibility behavior | Low | Medium (user surprise) | New radio group defaults to "On disagreement only" — exactly the current trust-cal-end behavior |
| Per-Library calibration view exposes confusing data when no corrections have been logged yet | Medium | Low (cosmetic) | Empty state copy: "No corrections logged yet on this Library. Calibration data will appear here after you Accept / Disambig at least one card." |
| Background scan toggle (auto-classify on note save) fires on every keystroke during note creation | High if wrong; Low if implemented correctly | High (performance) | Implement as "on save" = on the existing 1500ms debounced save, NOT on every keystroke. Verify with a stress test (type 100 chars rapidly in a fresh note → ensure 0 IPC calls during typing) |
| Help topic name "Source Review" collides with the panel name and is confusing | Low | Low | Subtitle the topic with "(Constellation Epistemic Content Engine — CECE)" so search finds both names |
| User Manual chapter introduces vocabulary inconsistent with the help topic | Medium | Low (drift) | Both files share the same opening paragraphs verbatim; differ only in depth. Cross-reference in both directions |
| 13 untranslated locales render Settings labels in English mid-AR/JA/etc. UI | Medium | Low (existing pattern) | Same as the V3-§8.r5 fallback approach: inline EN defaults via `$t() \|\| 'EN string'` in the Svelte. Document in PJ filing |
| Reasoning Cataloger status text becomes stale when V3-§7.b ships | Certain (deliberate) | Low | The status string is a single line that V3-§7.b updates as part of its wiring commit |

---

## §6 — Files in scope (Option B recommended)

For Option B:

**Settings UI:**
- **EDIT** `src/lib/components/SettingsModal.svelte` — new "Constellation Epistemic Content Engine" section header + 4 setting rows (Reasoning model status, trail visibility radio, background scan radio, per-Library calibration collapsible view)
- **EDIT** `src/lib/stores/appSettings.ts` — add `cece` sub-object with `reasoningTrailVisibility`, `backgroundScanTrigger` flags
- **NEW** `src/lib/cece/calibrationView.ts` — small TS module that fetches the per-Library reliability JSON via a new IPC + formats it for the Settings view
- **NEW** Tauri IPC `cece_get_reliability_for_active_library() -> ReliabilityProfile` in `src-tauri/src/cece/reliability.rs` (the data is already there; the IPC just exposes it for UI rendering)

**i18n:**
- **EDIT** `src/lib/i18n/en.json` — ~15 new `cece.settings.*` keys
- **EDIT** `src/lib/i18n/ar.json` — same in Arabic

**Help docs:**
- **NEW** `docs/help.uConstellation.World/Source Review/Source Review.md` — full topic per §3 Option B
- **EDIT** `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` — add a one-paragraph cross-reference at the end pointing to the new topic

**User Manual:**
- **EDIT** `docs/User Manual.md` — new chapter "The Source Review Workflow" between the Knowledge Strata chapter and Search

**Plan + orientation:**
- **NEW** `lab/reports/MIG-021v3-V3-§10-USER-FACING-SURFACES-PLAN.md`
- **EDIT** `docs/Constellation Orientation & Onboarding v1.92.md` → bump to v1.93 documenting V3-§10 close-out

**Session log:** continuing entries in today's `lab/reports/SESSION-LOG-2026-05-11.md`.

---

## §7 — What V3-§10 does NOT do

- ❌ Translate to 13 other locales (Option C scope; defer to PJ-NNN)
- ❌ Translate User Manual chapter to 14 languages (defer to PJ-014 follow-up)
- ❌ Wire llama.cpp / download Qwen3-4B (V3-§7.b territory; Reasoning model status text says "deferred")
- ❌ Add per-cataloger weight overrides in Settings (advanced-user feature; backlog)
- ❌ Add a "reset reliability data" button (correction is the canonical path; reset would be confusing)
- ❌ Surface the V3-§9.D axis-aware GBNF in Settings (interface lock-in for V3-§7.b; not user-facing today)
- ❌ Add per-Library Reasoning Cataloger preferences (no preferences exist yet — Reasoning is global)

---

## §8 — Decision request

**Boss, please pick:**

- **(A)** Option A only — Settings UI + i18n (en + ar). Defer help docs / User Manual. ~2-3 hrs.
- **(B)** Option B (recommended) — Settings + help topic + User Manual chapter, en + ar. ~5-6 hrs.
- **(C)** Option C — Option B + full 13-locale i18n + 14-language User Manual translations. ~12-15 hrs (heavy translation breadth; needs translator review cycles).
- **(D)** Skip V3-§10 and jump to V3-§11 (final integration audit + close-out of MIG-021v3 entire). The CECE engine is feature-complete; user-facing chrome can ship as a follow-up MIG.

Once you pick, I write the Plan with phase-by-phase commits + verification clauses, and you approve the plan before any code lands.

---

## §9 — Appendix: per-Library calibration view mockup (Option A/B/C)

The visual the Architect doc proposes for the Settings panel:

```
─────────────────────────────────────────────────────────────
  Constellation Epistemic Content Engine

  The cataloger ensemble that classifies your notes along
  Source × Content Type axes. Six lenses, each with its own
  evidence; the engine combines their votes into a synthesis.
  Local-only — no notes leave your device.

  Reasoning Cataloger model
    Status: Not downloaded — local AI judgment lens
            deferred to V3-§7.b. When llama.cpp wiring ships,
            you'll be able to download Qwen3-4B Q5_K_M from
            this panel.
    [download disabled — coming in a future update]

  Reasoning trail visibility
    ( ) Always show — every card auto-expands its trail
    (•) On disagreement only — auto-expand for Split /
        StrongMajority cards; collapse for Unanimous (default)
    ( ) Always hide — manual click required to see trails

  Background classification
    ( ) Off — manual scan only (default)
    ( ) On note save — re-classify each note ~1.5s after
        you stop typing
    ( ) On app start — scan unclassified notes once per
        launch

  Per-Library calibration                                [▾]
    Active Library: arab-literature
    20 corrections required for stable accuracy data.

       Cataloger          Horizontal      Vertical
       ─────────          ──────────      ────────
       Your frontmatter   12/12 (100%)    4/4 (100%)
       Citations          18/22 (82%)     6/8 (75%)
       Wordstems          24/28 (86%)     20/26 (77%)
       Linked notes       3/4 (uniform)   2/3 (uniform)
       Similar notes      14/19 (74%)     12/19 (63%)
       AI judgment        — (not running) — (not running)

    "(uniform)" = below 20-correction threshold; cataloger
    contributes uniformly weighted votes until enough data
    accumulates.
─────────────────────────────────────────────────────────────
```

The exact visual is open to refinement — what matters is the data shape. The Plan phase will pin down the actual Svelte markup.

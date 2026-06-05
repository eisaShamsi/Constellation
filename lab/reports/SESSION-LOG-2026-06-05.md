# Session Log — 2026-06-05

## HANDOVER / STATE OF STANDING — MIG-070 §C (Style Setter unification), mid-migration

Written at the close of a long multi-day run (≈33 commits across 2026-06-02 → 06-05) so a fresh session can resume cold. Companion handover doc: `docs/MIG-070C-HANDOVER.md`. Plan: `docs/MIG-070C-PLAN.md`. Audit/invariants: `docs/MIG-070-style-merge-AUDIT.md`. Detailed commit trail: `lab/reports/SESSION-LOG-2026-06-02.md`.

**Position:** commit `b49658e1`, branch `main`, working tree clean (only machine-local `.claude/settings.local.json`). Binary mtime `2026-06-04 20:22:27` (current; docs-only changes since the last build).

### (a) Verified-shipped & protected — DO NOT REGRESS
- **Persistence spine (Phase 0/1):** per-Universe `appSettings.styleOverride` merged on top of the theme in the shared `+layout` apply `$effect` (tracked in `_lastStyleSettingsKeys`; survives theme switch; clears cleanly). Setter `apply()`→`mergeStyleOverride`, `resetDraft()`→`clearAllStyleOverride`, seeds draft from saved look on open. Boss-validated.
- **Every Markdown element (§3A):** H1–H6 (own colour+size, shared weight), bold, italic, strikethrough (line colour+thickness), inline code (bg/text/size), blockquote text — all read CSS vars (`NotePane` `markdownHighlightStyle` + `livePreview` theme, in parity).
- **Interface elements (§3B):** file-tree per-row-type (Library/Folder/cUniverse over a master), Status bar, Universe bar (+ text size); tab/library-label/breadcrumb follow **interface**; note title+body follow **note** (`--editor-text-color`).
- **Categories rail** (Interface · Components · Editor · Global · Sky View · OrgChart · Index · Cataloger · Shell).
- **Global category** (backgrounds/text-shades/status/accent; type & rhythm; shape) and **Components category** (dock/toolbar/layout-bar/tabs/right-sidebar/buttons/tags/shell).
- **Saved colour swatches** (per-Universe palette; auto-save; click-apply / right-click-remove; cap 24).
- **Focused per-element preview (#4):** centre replicates only the selected element.
- **Fonts (Phase 4.1/4.2):** 14 curated Latin stacks + Code font; **per-script fonts** (Arabic/Hebrew/CJK/Devanagari/Cyrillic) language-smart via `CnSetterText`/`CnSetterUI` virtual families — Arabic note shows its Arabic font even in an English interface; chrome follows the interface font.
- **Decisions locked:** (1) interface-language selector stays in **Settings → Language** (removed from the Setter; commit `b49658e1`); (2) Setter UI localization (15 langs) happens AFTER all content is final.

### (b) At-risk / in-flight / uncommitted
- **Nothing uncommitted** beyond machine-local `.claude/settings.local.json`. No in-flight code edit. Clean handover point.
- The last commit (`b49658e1`) was a control-removal with **no Boss test outstanding** (per-script fonts already passed).

### (c) Known-broken / known-interim
- **`note_links.link_type` globally `'relates'`** (memo `project_note_links_link_type_relates_bug.md`) — foundational, separate from MIG-070; Phase 5 must not assume link_type is correctly populated. Not a MIG-070 fix.
- **Second screen** does NOT yet sync `styleOverride` / full style (Phase 8 builds the mirror). Until then, the Setter's look applies to the main window only.
- **Old Appearance + Style-Settings tabs remain LIVE** (intentional — they stay until the Phase 9.1 parity gate).
- **Deferred Phase 2** (catalog parity for ~17 Setter-only vars) — folded into Phase 9 (it's parity plumbing for the soon-retired tab; the Setter works via `styleOverride` regardless).

### (d) Pending — not started
- **Phase 5 — link colours** (NEXT): 8 typed-link colours via the shared link-type save path (`LinkTypesEditor.svelte` / `linkTypeRegistry.ts` — confirm the exact save fn) + display toggles (`colourTypedLinks` L3242, `showTypedLinkLabels` L3244) + pill shape (`linkPills` L3337).
- **Phase 6** — unify Themes + MIG-069 Presets into one "Styles" gallery (read-time non-destructive).
- **Phase 7** — 4 no-UI gaps (accent picker · dark/light/system · custom-CSS editor · per-library appearance + its MISSING apply path, ⚠ LL-023 clean clear-down).
- **Phase 8** — second-screen full-style sync + live re-sync.
- **Setter UI localization** (15 languages) — after content final.
- **Phase 9** — retire old tabs at the parity gate (+ deferred Phase 2). LAST.
- **Deferred small:** swatch rename (memory #2); full installed-fonts enumeration.

### (e) Documentation drift / housekeeping
- **Orientation bumped v2.51 → v2.52** (NEW file this handover; preamble captures §C Phases 0–4.2 + swatches + focused previews + per-script fonts + the two decisions + the precise remaining roadmap). v2.51's preamble was stale (described only the kickoff). v2.51 retained as historical record.
- **MoCh** `docs/MoCh/MoCh-2026-06-05-0848.md` written (font/language arc + handover).
- **Handover doc** `docs/MIG-070C-HANDOVER.md` written (self-contained resume + copy-paste prompt).
- The running detailed trail lives in `SESSION-LOG-2026-06-02.md` (dated 06-02 but appended through 06-04); this 06-05 file is the clean handover snapshot.

---

### Handover docs prepared (this entry)
- `docs/MIG-070C-HANDOVER.md` — centerpiece, self-contained, includes the copy-paste resume prompt.
- `docs/Constellation Orientation & Onboarding v2.52.md` — orientation bump.
- `docs/MoCh/MoCh-2026-06-05-0848.md` — conversational trace.
- `lab/reports/SESSION-LOG-2026-06-05.md` — this state-of-standing.

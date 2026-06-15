# 16 — Inspector 360 (360.3D) (Concept Paper)

> A Cognitive-Engine satellite, not core. It reads the Editor's open note and answers *"what surrounds this idea?"* in one glance. Its bring-up gate is a hard **Rule 8** problem: today it recomputes the whole library on every open. See [00-Constellation](00-Constellation-Core-Concept-Paper.md) §Rule 8 and the [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) template.

## 1. Function in hand
**Inspector 360 / 360.3D** — `src/lib/components/Inspector360.svelte` (the "المنظار الكروي" / 360.3D panel). Renders in two surfaces: a **compact** right-sidebar scorecard tab (`rightSidebarTab === 'inspector360'`) and a **full-window** Stratification-Matrix overlay (`showInspector360`). Backed by the Rust command `get_360_view` in `src-tauri/src/inspector360.rs`, returning a `Note360View`. CE Phase 12.

## 2. Purpose
Give the single open note its **360° context in one view**: who links to it and how (8 typed acts + untyped), at what stratum (1 Datum → 8 Worldview), its maturity, origin/trust depth, stage, review status, trail and tag membership — and the **gaps** (blind-spot link types, orphan, fragile/single-point-of-failure, active tensions). It serves **Connection** and **Tension** (Acts 2 and 3 of the Five Acts): it makes the shape of a note's relationships, and the contradictions sitting on it, *visible* so the user can act on them. Justified: the typed-link vocabulary is Constellation's cognitive core, and a per-note instrument that surfaces which acts are *missing* is exactly the "diagnostic instrument for intellectual life" the core paper calls for.

## 3. What it is NOT
- **Not** the whole-universe view — that is Sky View (bubbles) / Constellation Map (sunburst). Inspector 360 is **single-note scope** (memory: 360.3D = one note; Sight = whole universe — mutually exclusive).
- **Not** an editor — it never writes note content; clicking a dot/title *navigates* (opens that note), it does not mutate.
- **Not** a persisted index — it owns no stored table today (this is the bring-up problem, §7).
- **Not** Sight — it does not classify epistemic sources; it reads link/stratum structure already implied by the files.

## 4. Wiring
- **Inputs (stores read):** the focused tab's `path` + `libraryPath` (`inspector360Path` / `inspector360LibPath` in `+layout.svelte`); the Link-Type Registry (`getLinkTypes()` for column order/colours/labels); `lookupStageEmoji` (stage icons).
- **Inputs (IPC):** one call — `invoke('get_360_view', { libraryPath, notePath })`, debounced 200 ms, sequence-guarded against overlapping fetches, last-key-guarded to skip re-fetch of the same note.
- **Outputs (events):** none direct from the component; navigation goes through the host's `onNoteClick` callback in `+layout.svelte`, which pushes onto `inspector360BackStack` and opens the target note (multi-hop back-nav). `onClose` / `onBack` are host callbacks.
- **Outputs (IPC / writes):** **none** — read-only; it never persists.
- **Consumers:** only the `+layout.svelte` host (the back-stack + overlay/sidebar visibility). Nothing downstream depends on it.
- **Connection to the Editor (the gate):** it attaches **by reading the focused tab** the Editor owns. When the open note changes, the `$effect` re-fires `get_360_view` for the new path. It is a pure *downstream reader* of the Editor's current-note selection — it does not feed the Editor and is lazy-mounted (`inspector360EverOpened`, LL-022) so it costs nothing until first opened.

## 5. Right-click / context menu
- **Has one? NO.** Grep of `Inspector360.svelte` for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` returns **zero matches**. All interaction is left-`onclick` (navigate to note), `onmouseenter`/`onmouseleave` (dot tooltip), and the overflow `+N` expand toggle.
- **Shared vs hand-rolled:** N/A — there is no menu at all (neither shared MIG-077 `<ContextMenu>` nor hand-rolled).
- **Actions reachable only by right-click:** none.
- **GAP — flagged.** A note dot / title is exactly the kind of target that *should* offer a right-click action set (Open, Open in new tab / split, Reveal in file tree, Copy wikilink, Create link of type X). Today the only affordance is left-click=navigate. Bring-up should add the **shared `<ContextMenu>`** (per MIG-077, never hand-rolled) on dots and list-items, reusing the file-tree/note context-menu builder so there is one source of truth.

## 6. Multilingual
- **Mostly localized.** User-facing labels flow through `$t('inspector360.*')` with a `tr()` fallback chain (active locale → en.json → English literal). The `inspector360` key block exists in **all 15 locale files** (ar de en es fa fr he hi ja ko pt ru tr ur zh), and Arabic is a real native translation (`المنظار الكروي`; strata as `بَيانة / معلومة / رأي / مفهوم / مبدأ / نظرية / منظور / رؤية شاملة`) — not transliteration, per the full-localization standing order.
- **RTL / dir:** note names and the back-name use `dir="auto"` (the dot tooltip, card name, header name, list names). Layout uses logical properties (`inset-inline-end`, `text-align: start`) in several places.
- **HARDCODED ENGLISH — flagged.** Non-`$t()` strings remain on **`title=` tooltips and `aria-label`s** (not yet localized): e.g. `title="Close"`, ``title={`Back to ${previousNoteName}`}``, ``title={`Return to ${previousNoteName}`}``, `title="Blind spot — typed direction not used"`, `title="Tensions — active contradicts pointing here"`, `title="Fragile — load-bearing on thin foundation"`, `title="Collapse"`, ``title={`Show all ${cellNotes.length}`}``. The `STRATUM_FALLBACK` map and the `|| 'English'` defensive literals are English-only (acceptable as last-resort fallbacks, but the `title`/`aria-label` strings are user-facing and must move to `$t()` ×15 in bring-up).

## 7. Boot behavior
- **Runs at boot? NO** — lazy-mounted; the IPC fires only when the panel is first opened (sidebar tab or full overlay).
- **Rule 8 status: ❌ RECOMPUTES-on-read — VIOLATION.** `get_360_view` is the canonical anti-pattern the core paper warns against. On every open it **re-walks the entire library from disk** (`scan_all_notes`: recursive `fs::read_dir` + `fs::read_to_string` for every `.md`, regex-parsing links and tags), then `precompute_all_strata` over the whole graph, scans **all** notes for inbound links, walks the derives-from provenance chain, and does a **second** full disk walk for trail membership (`scan_trails_for_note`). Nothing is persisted; the derived view is rebuilt from scratch each time. This is exactly the "re-walk the Universe to produce a derived view" shape Rule 8 forbids — the same class as the term-index that OOMed (the LL-XXX cited in the core paper). The correct end-state: persist per-note 360 facts (or read them from the existing FTS5 / `note_links` index) and maintain them at write time via the note-save reindex hook; the panel read should be a cheap lookup, not a universe scan.
- **Cost:** **unknown — verify in bring-up.** Estimated O(N + total_links) per open on a cold scan of a 7,600-note universe (full disk read of every `.md`, twice counting the trails pass) — likely hundreds of ms to seconds. Must be measured before re-enable; the 200 ms debounce hides repeat opens of the same note but not the first scan.

## 8. Flag / gate & bring-up position
- **Gate today:** `$appSettings.enabledFeatures.inspector360` (boolean, **default `true`**). It is *not* gated behind any `SIGHT_*` flag; it is its own CE-Phase-12 feature flag. Visibility also gated by `NOTE_SCOPED_TABS` (requires an open note) and lazy-mount (`inspector360EverOpened`).
- **Bring-up phase:** **after the Editor (the gate) and the write-time link index.** It depends on (a) the Editor's focused-note selection, (b) the Link-Type Registry being boot-seeded, and (c) — for the Rule 8 fix — a persisted/maintained link+stratum index it can read instead of disk-walking. Re-enable should be **deferred until the Rule 8 violation is converted to a write-time-derived read.** Default-on should be reconsidered until then.

## 9. Budget
- **Boot budget:** zero — lazy-mounted, fires no IPC at boot. (Must stay zero.)
- **Interaction budget:** first-open scan must come in **under a perceptible threshold on a 7,600-note universe** — target to be set in bring-up (the current full-disk-walk almost certainly fails this on a large universe; the persisted-read rewrite is the path to meeting it). Repeat opens of the same note: instant (last-key guard).
- **Regression guard:** measure `get_360_view` latency on a large universe before/after the Rule 8 rewrite; assert no boot-time or keystroke-path cost (it must never `invoke` on edit); confirm the 200 ms debounce + sequence guard still discard stale fetches on rapid tab switches.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** opening a note shows correct typed/untyped link counts, stratum, maturity, origin, stage, review, trails, and the gap/orphan/fragile/tension flags.
- [ ] **Serves Constellation's core purpose:** makes Connection + Tension (Acts 2–3) visible per note; blind-spot link types read as a first-class signal.
- [ ] **Wires correctly to the Editor:** changing the focused note re-fetches; navigation via dot/title click opens the target and pushes the back-stack; close/back behave.
- [ ] **Right-click present + correct:** add the **shared `<ContextMenu>`** (MIG-077, not hand-rolled) on note dots/list-items with Open / Open-in-split / Reveal / Copy-wikilink actions.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** move all `title=`/`aria-label` strings to `$t('inspector360.*')` in all 15 locales; verify `dir="auto"` on every note-name surface; RTL layout intact.
- [ ] **Within budget:** first-open latency measured and under the bring-up threshold on a 7,600-note universe; zero boot/keystroke cost.
- [ ] **Obeys Rule 8:** `get_360_view` reads a persisted/maintained derived view (write-time index), **not** a full library disk-walk. No `scan_*`/recompute on read.
- [ ] **Holds its invariants:** read-only (never writes note content); same `Note360View` shape rendered by compact and full surfaces; back-stack consistent; sequence guard discards stale fetches.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured; Rule 8 violation outstanding)**
Notes: **The central bring-up blocker is Rule 8** — `get_360_view` recomputes the entire library (two full disk walks: `scan_all_notes` + `scan_trails_for_note`) on every open, owning no persisted state. Convert to a write-time-derived read (reuse the `note_links` / FTS5 index + a maintained per-note stratum/fact table) before re-enable. Two secondary gaps: **(a) no right-click menu** on note dots/titles (should use the shared `<ContextMenu>`), **(b) hardcoded English** on `title`/`aria-label` tooltips (the visible `$t('inspector360.*')` labels are localized ×15; the tooltips are not). First-open cost on a large universe: **unknown — verify in bring-up.**

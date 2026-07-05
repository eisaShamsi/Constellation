# PJ-068 — The Second Screen as a Contextual Companion (Concept Paper)

**Date:** 2026-07-04 · **Status:** Parked (Boss ruling: "We will set it aside for now… and tackle it in due time") · **Supersedes nothing; extends** [26-second-screen.md](26-second-screen.md) (the bring-up stub) with the full history, the ratified concept, and the 2026-07-04 governing rulings.

---

## 1. The concept (the horse)

> The Second Screen is **an extension of the main screen** onto a second monitor: it shows the **context around the work in hand** — never a second copy of the work itself.

**The two governing rulings (Boss, 2026-07-04, verbatim intent):**
1. **"The SS should be contextual to the main screen."** What it displays must be *about* what the main window is doing right now.
2. **"It shouldn't replicate what is already displayed on the main screen."** Duplication is not context. A bigger copy of a main-window surface serves nothing.

**The ratified frame (same day, Boss):** *"the SS are designed as an extension to the main screen"* · *"it is conditional on the availability of a second monitor. And it is up to users to decide whether to use it or discard it."* — optional by design, hardware-gated by design, user-controlled by design.

These rulings are not new law — they are the **original law restated**. On 2026-04-05, after defining Constellation's core purpose as *"an extension of the mind,"* the Boss set 7 redesign principles for the SS, including **context-aware** and **"User in control: SS never initiates — it serves the deliberate workflow"** (SESSION-LOG-2026-04-04.md:303-313). PJ-068 exists because parts of the implementation drifted from that law, and one part of the *documented contract* itself blesses replication (see §5).

**The three-way test every SS surface must pass (the PJ-068 razor):**
- **Contextual** — does it respond to what the main window is doing *now*?
- **Complementary** — does it show something the main window is *not* showing?
- **Chosen** — does it appear because the user deliberately opened the SS, never initiating on its own?

A surface failing any prong is out — either redesigned to pass, or retired.

## 2. History (birth → today, the load-bearing arc)

| Date | Event |
|---|---|
| 2026-03-13 | **Born** (`48eb3f01`): a standalone mode-switcher window (grid/graph/detail). Same day (`68911ca5`): the architecture that still stands — a static entry (`screen.html` + `screen-entry.ts`) outside SvelteKit routing; the window pre-declared hidden; close = hide. |
| 2026-03-19 | **First contextual companion** (`cbd80f78`): Sky View hover=glimpse / click=pin, backlinks/forward-links/tags/local-graph beside the main graph. The SS finds its true vocation. |
| 2026-03-21 | **Mode-based companion model** (`0191438c`): SS adapts to the main sidebar mode; Universe Dashboard mode; session tracking. |
| 2026-04-04 | **The founding principle** (`8e9b0f74`): SS's own save/load logic competed with the editor and interrupted edits → **"screens are displays, not domains"** enters CLAUDE.md. The SS never re-implements operations again. |
| 2026-04-05/06 | **Boss's 7 principles + the redesign** (`1c407a98`, `1edb9720`, `f26f3b76`): monitor detection + auto-position; right-sidebar panels *migrate* to the SS (main becomes a clean writing space); split comparison; **gated behind 2+ monitors**; never auto-opens. |
| 2026-04-13 | Becomes a **plug-in toggle**, ON by default (`a9cdc113`). |
| 2026-06-09 | **Release-blank fix** (`1b67f036`): the SS had rendered blank white in EVERY release build since birth (no production build step for its entry) — found by a Boss screenshot. Dedicated second Vite pass added. Also MIG-072 §5: palette parity + fill-the-centre-zone (Boss: *"we are not taking advantage of the SS center zone"*). |
| 2026-06-12 | **MIG-076 single-ownership**: SS edit sync adopts the freshness-gated note model; SS is a first-class member of the Editor-Surface Gate (item 7: second-screen edit + sync). |
| 2026-06-15 | **The double-init culprit closed** (`256d7c5f`, MIG-079 §A): SS's `setActiveUniverse` call removed — it had been re-initializing the search DB (~34 s double boot). SS now only *reads* the active universe name. Boss: *"way better, no freezes."* |
| 2026-06-21 | MIG-080 §C.1: SS task toggles → `toggleTaskReconciled` (Boss-approved fold-in). |
| 2026-07-04 | **F2′**: SS gains its own `note-created` listener (creations reach its displays; zero new operations). **Boss finds `Ctrl+Shift+2` dead** → the Shift+digit shortcut class had been dead since birth; fixed (`1676a28f`). **Boss concept check** → the rulings above; rework parked as **PJ-068**. |

**The pattern history teaches:** every SS defect that mattered came from the SS acting like a *domain* (own saves, own universe activation) or from it being a *second build artifact* nobody verified (release-blank; dead shortcut). Every SS success came from it being a faithful, well-fed *display* (sky companion, panels migration, split compare). PJ-068 continues the winning direction.

## 3. Current truth (code, 2026-07-04)

- **Architecture:** a standalone Svelte app in its own WebView (`screen-entry.ts` mounts `SecondScreenPage.svelte`, 2,311 lines; own store instances; no `+layout`). All main↔SS coupling is the **Tauri event vocabulary** in `src/lib/secondScreen.ts` (12 main→screen events, 4 screen→main, 1 bidirectional). **The PJ-068 rework is therefore an event-vocabulary + mode redesign, not a window redesign.**
- **Mode dispatch:** one 14-branch first-match-wins if-chain (`SecondScreenPage.svelte:990-1732`): dashboard-note → dashboard-tag → index-term → index-compare → map → split → editor-panels → sky-graph → navigator → orgchart → dashboard → fallback tab-strip editor → empty.
- **Sync contract:** settings/theme/fonts/locale propagate instantly; SS edits flow back via `screen:note-saved` (freshness-gated adoption); creations via `note-created`; the main right sidebar auto-hides while the SS is open (its panels *migrate*).
- **Two-monitor gate:** entry points (dock button, `Ctrl+Shift+2`, palette) hidden on a single monitor; the right sidebar carries the same panels inline. Boss-ratified.

## 4. The replication audit (the heart of PJ-068)

Per-mode verdict against the razor (evidence: `SecondScreenPage.svelte` + `+layout.svelte` senders; workflow `wf_42ec73f0-794`):

| SS mode | Verdict | Evidence |
|---|---|---|
| **Navigator companion** | **REPLICATES** | When the main sidebar is in list mode, the SS renders the *same* `NotebookNavigator` component the main sidebar is showing at that exact moment — same data, bigger. The canonical violation. |
| **OrgChart ("Sky View tree")** | **REPLICATES + UNREACHABLE** | Mirrors the main sidebar's OrgChart branch — but no code path ever sets `sidebarMode='skyview'`, so neither side can enter it from the UI. Dead mode, still documented in the manual. |
| **Sky View graph companion** | **COMPLEMENTS** | Main shows the bubble graph; SS shows what the graph doesn't: preview, backlinks/forward-links, tags, local subgraph, peek editor, history. The 2026-03-19 original. Minor overlap: the LocalSkyView panel re-draws visible nodes. |
| **Split comparison** | **COMPLEMENTS** | Main shows the split notes' bodies; SS shows per-note metadata columns (properties/backlinks/tags/local-graph/tasks) — a surface main has nowhere to render. |
| **Map companion** | **MIXED** | Mini-map drill-down grid re-renders hierarchy the main sunburst already shows (replication-heavy); the note-click leg opens duplicate editors in both windows; the sole complement is the context mini-map beside the SS editor. |
| **Index term exploration** | **MIXED** | Term click expands mentions inline in the main panel AND sends the same list to the SS (duplicate); the full editor with term highlight is the complement. The multi-term compare leg leans COMPLEMENTS (main shows only the intersection; SS shows per-term columns). |
| **Editor panels (migrated right sidebar)** | **COMPLEMENTS by construction** | The panels are *moved*, not mirrored — the main right sidebar hides while the SS carries them. Nothing shown twice. The model to generalize. |
| **Universe Dashboard** | **MIXED (ambient)** | Not simultaneous duplication, but the same `DashboardView` the main home screen renders, ambient to the active note — fails the *contextual* prong, not the replication prong. |
| **Fallback tab-strip editor** | **NON-CONTEXTUAL** | An SS-local tab strip + editor driven by SS-local navigation — a freestanding second editor. Fails the contextual prong; the third failure class. |

**Structural findings (fold into the rework):**
- **Mode shadowing (static read, not runtime-reproduced):** once editor-panels mode activates (any main tab activity), nothing deactivates it except SS close — by template order it shadows the sky/navigator/orgchart/dashboard companions for the rest of the session. The documented mode model and the executing dispatch have drifted.
- **Dead wire:** `screen:open-note` is emitted from 4 main-window sites into a no-op listener; dead imports (`writeNote`, `renameItem`, …) linger in the SS file.
- **Rule-8 re-walks:** the SS re-derives backlinks/local-sky from fresh `scan_library_links` walks per hover/click/split-note (async since Batch W `d9f8bd80`, so no longer freezing — but still recompute-on-read; should read persisted derived views). `buildSkyData` calls are alias-blind (rename-drops-edges, logged since 2026-04-25).
- **Cross-window flush gap (honest, open):** a main-window-dirty note renamed from the SS relies on Rust path locks for disk integrity; display adoption rides events (2026-07-03 scope note).

## 5. Doc drift to resolve with the rework

1. **CP-26 + 00-MASTER are stale on the #1 violation:** both still list `setActiveUniverse` as an open breach; it was removed 2026-06-15 (MIG-079 §A).
2. **The help topic contradicts itself:** "always starts closed — never auto-restored" (line 31) vs "Workspaces save and restore the second screen state" (line 245). The state-request machinery exists; the two claims can't both be true as written.
3. **The manual's mode table documents the unreachable OrgChart mode** and promises the replicating "Full Navigator view" — the manual itself confesses the replication: two of its seven rows promise duplicates, five promise context. **The PJ-068 scope line runs exactly between those two groups.**
4. The help topic already states the contextual principle ("Instead of duplicating the main editor, it provides contextual views…") — then blesses full-editor replication in four modes. The documented contract must be made self-consistent with the rulings.
5. Two different help sections are both titled "Sky View Companion"; one help line leaks internal jargon ("CodeMirror 6").

## 6. Rework scope (when PJ-068 is taken up — needs Boss rulings, then /migration)

**Keep and strengthen (already pass the razor):** Sky View graph companion · Split comparison · Editor-panels migration · Index *compare* leg.

**Redesign or retire (fail the razor) — each needs a Boss ruling:**
- Navigator companion (replicates) — retire, or redesign into something the main list can't show (e.g. selected-note context).
- OrgChart mode (unreachable) — delete, or make reachable *and* complementary.
- Map companion (replication-heavy) — keep only the complementary legs?
- Universe Dashboard (ambient) — keep as the explicit "no note open" idle state, or make contextual?
- Fallback tab-strip editor (non-contextual) — is a freestanding second editor ever wanted, or does every SS note view arrive from a main-window action?

**Engineering items riding along:** fix the editor-panels shadowing (a real mode *state machine* instead of the if-chain); retire the dead `screen:open-note` wire + dead imports; Rule-8 persisted reads for companion data; alias-aware `buildSkyData`; the hardcoded-English sweep (×15) + shared right-click on SS lists; re-sync manual + help topic + CP-26; the cross-window flush gap ruling.

**What PJ-068 must NOT do:** add operations to the SS (displays-not-domains is settled law); remove the two-monitor gate; make the SS auto-open or self-initiate; regress the Editor-Surface Gate item 7.

## 7. Dependencies & status

- Independent of the freeze program (Batch W already made its scans non-freezing; F3's "second-screen per-note reads" item overlaps §6's Rule-8 work — coordinate when either starts).
- The bring-up program's CP-26 acceptance checklist remains the per-function gate; this paper supplies its missing concept + history + the new law.
- **Status: PARKED by Boss ruling 2026-07-04.** Entry: Pending Jobs v1.15, PJ-068. Nothing in the live app changes until the Boss reopens it.

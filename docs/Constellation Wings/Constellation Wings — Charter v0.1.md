# Constellation Wings — Sub-Project Charter

**Version 0.1 | 2026-05-19 | Status: CHARTERED, DEFERRED (Eisa schedules the start)**

> "I want you to create a new Constellation sub-project, which we will call 'Constellation Wings', that will design the proper External Plug-in subsystem. I will decide the due time to work on it." — Eisa, 2026-05-19

---

## 1. What Wings is

**Constellation Wings** is the sub-project that will design and build Constellation's **External Plug-in subsystem** — the architecture that lets features live *outside* the core build and load into Constellation as plugins, the way Obsidian community plugins work.

Wings is **chartered but not started.** This document captures the purpose, the architectural groundwork already reasoned through, and the open design questions — so that whenever Eisa schedules the work, it begins from a known position rather than rediscovering it.

## 2. Why Wings exists

Constellation's founding mission (per `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` + the User Manual): **cultivate wisdom through the living link system.** Visualizations and analytical surfaces are *downstream readers* of that system — valuable, but not the core mission.

The insight that produced Wings: if visualizations are downstream readers, they don't need to live *inside* core. Extracting them as plugins:

- Keeps core focused on the living-link infrastructure (the irreplaceable part)
- Lets each visualization evolve on its own cadence without bloating core
- Eventually enables a community to build and share their own plugins
- Mirrors Obsidian's proven model: markdown-files + plugin-host as core; everything visual is community

## 3. The plugin taxonomy (Eisa's distinction, 2026-05-19)

Two tiers of plugin:

| Tier | Definition | Examples | Distribution |
|---|---|---|---|
| **Core Plug-in** | Ships with Constellation, maintained by the core team, on by default. Architecturally isolated behind the plugin boundary, but bundled. | **CECE** (Constellation Epistemic Content Engine) | In the installer |
| **External Constellation Plug-in** | Detached from core, packaged standalone, loadable from outside the build. Eventually community-authored. | **Sight**, **Constellation Map** | In due time: external; for now: disabled in core |

CECE is the first designated Core Plug-in (stays in the right sidebar as-is until confirmed ready). Sight + Map are the first External Plugin candidates (disabled in core as of MIG-038, pending detachment under Wings).

## 4. The central architectural constraint

**Tauri compiles all Rust into a single binary at build time.** There is no runtime-loadable Rust plugin (the way Obsidian loads a JS plugin from a folder). This shapes the entire Wings design, because Constellation's extractable features are *not* pure-frontend:

- **Sight** = Svelte UI + `sight_v6.rs` (IPC commands) + `sight_v6_layout` SQLite cache + trigger-maintained invalidation + i18n + settings.
- **Map** = Svelte UI + `map.rs` (sunburst tree IPC) + i18n.
- **CECE** = Svelte UI (Source Review panel) + a Rust classifier (local LLM: e5-small ONNX + Qwen3-1.7B GGUF via llama.cpp) + `axis_assignment` rows + settings.

So "package as a plugin" splits into two layers with very different effort:

### Layer 1 — Structural isolation (the achievable first step)

Move every file each feature owns (Svelte + Rust + i18n + docs) into a self-contained directory (`plugins/sight/`, `plugins/map/`, `plugins/cece/`) behind a **documented, enforced boundary** — the plugin talks to core only through a defined interface, and core contains zero feature-specific knowledge except "load this plugin." The Rust still compiles into the core binary, but behind a clean module wall. After this, each feature is *structurally* a standalone package; future external extraction becomes "move the folder + flip the loader."

### Layer 2 — Dynamic external loading (the "in due time" end state)

The actual plugin host that loads a plugin from *outside* the build. For the JS half this is Obsidian-style and straightforward. For the Rust half, the options are:

- **(a)** Absorb each plugin's Rust into core as *generic* IPCs (`query_notes`, `get_note_graph`, `subscribe_to_change`) so the plugin becomes pure-JS and loadable. Cleanest end state; most work; risks losing write-time-derived cache performance.
- **(b)** Sidecar process per Rust-bearing plugin. Heavier runtime; preserves Rust.
- **(c)** WASM / QuickJS sandbox for Rust-equivalent plugin logic. Tightest trust envelope; most engineering.

Eisa's earlier steer (2026-05-19): **hybrid API** — JS plugins consuming core Rust IPCs — which points at option (a) for the Rust half.

## 5. The central design decision Wings must resolve

**The hybrid API shape.** Sight currently calls feature-specific IPCs (`sight_v6_get_layout`). For a clean plugin boundary, do those IPCs:

- **(i)** stay in core but get recategorized as "plugin-facing API" (pragmatic, keeps the write-time-cache performance, but core still contains feature-specific code), or
- **(ii)** get generalized into reusable services that *any* plugin can call (cleaner, more work, risks losing the cache perf)?

This is the crux. Layer 1 can proceed with (i); Layer 2 external loading likely needs (ii) for the Rust-bearing plugins.

## 6. Open design questions (for when Wings activates)

1. Plugin manifest format — what does a plugin declare (id, dock icon, mount point, required IPCs, permissions)?
2. Plugin registration API — how does a plugin register a dock button + mount its UI + add settings + add i18n keys?
3. Data access security — how does a plugin read SQLite without becoming a security hole? Scoped commands vs raw queries?
4. The hybrid-API decision (§5: (i) vs (ii)).
5. Distribution model — bundled only / federated GitHub (Obsidian-style) / in-app browser / zip-drop. (Eisa deferred the community/distribution model on 2026-05-19.)
6. Trust + consent — for external plugins, what's the install-consent flow? (Obsidian shows a trust banner.)
7. Versioning + compatibility — how do plugins declare which core API version they target?

## 7. Cross-check sources (for the eventual Architect phase)

Per Working Agreement #5, the Wings Architect must cross-check against:

- **Obsidian's plugin architecture** — the model Eisa cited; core exposes a JS API, plugins register via `manifest.json` + `main.js`, community-plugins.json on GitHub drives the in-app browser.
- **VS Code's extension model** — `package.json` contribution points, activation events, extension host process.
- **Tauri's plugin landscape** — what Tauri natively offers (Tauri plugins are Rust-side, compile-time) vs. what Wings would build on top.
- **Logseq / Reflect / other Electron PKM plugin systems** — JS plugin patterns in knowledge tools.

## 8. Groundwork already done (before Wings starts)

- **MIG-038 (2026-05-19)** — Sight + Map disabled in core (Sight via `SIGHT_V6_ENABLED=false`; Map via a `loadSettings` force-off of `enabledFeatures.constellationMap`). All code intact for later detachment. This is the "first step: disable them" Eisa directed.
- **Version → 0.1.0** — JS-side configs aligned to the Rust-side `Cargo.toml` (commit `26fe4f43`). Constellation is v0.1.
- **The dormant Sight v7 cascade** (`src/lib/sight/v7/`) + the MIG-036 Architect doc remain on disk; some of that thinking (pure-function `density.ts` / `stack.ts`) may inform a future Sight plugin but is not Wings-scoped.

## 9. Status + next action

**Status: CHARTERED, DEFERRED.** No code, no Architect doc beyond this charter. Eisa decides the due time to begin.

When Wings starts, the first action is the four-phase `/migration` Architect doc for the External Plug-in subsystem, seeded by this charter's §4–§7.

---

*This charter is the durable record of the Wings sub-project's purpose and the architectural reasoning developed on 2026-05-19. It supersedes the loose "MIG-038 plugin architecture" framing — the plugin-subsystem design lives here under Wings, not under a MIG number, until Eisa schedules the build.*

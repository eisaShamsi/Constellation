/**
 * MIG-030 (2026-05-18) — vitest configuration for Sight v6 tests.
 *
 * Scope discipline:
 *   - INCLUDE: tests/sight-v6/{perf,tradition-isolation,tradition-perf}.test.ts
 *   - EXCLUDE: tests/sight-v6/layout-fidelity.test.ts (needs @playwright/test —
 *     deferred indefinitely until a playwright runner is wanted)
 *   - EXCLUDE: .claude/worktrees/ (vitest's default file walk picks up
 *     worktree duplicates; we only want primary tests)
 *   - EXCLUDE: node_modules + build (default)
 */
import { defineConfig } from 'vitest/config';
import { fileURLToPath } from 'node:url';

export default defineConfig({
	// Resolve SvelteKit's `$lib` alias so tests can import modules that use it
	// (e.g. editor/completions.ts → $lib/libraries/linkTypeRegistry).
	resolve: {
		alias: {
			$lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
		},
	},
	test: {
		include: [
			// MIG-080 §C.2 (2026-06-21) — natural-language task due-date resolution.
			'tests/mig-080/taskDates.test.ts',
			'tests/sight-v6/perf.test.ts',
			'tests/sight-v6/tradition-isolation.test.ts',
			'tests/sight-v6/tradition-perf.test.ts',
			// MIG-036 P2 (2026-05-19) — Sight v7 rendering primitives.
			// Files live in tests/sight-v6/ for now; renaming the directory
			// is a polish item for the v7 close-out.
			'tests/sight-v6/v7-density.test.ts',
			'tests/sight-v6/v7-stack.test.ts',
			// MIG-060 §E (2026-05-28) — Lens-row threading gestures.
			'tests/mig-060/host-note-gestures.test.ts',
			// MIG-067 §C (2026-05-31) — frontend Link-Type Registry getters.
			'tests/mig-067/linkTypeRegistry.test.ts',
			// MIG-067 §E (2026-05-31) — type-first wikilink autocomplete phases.
			'tests/mig-067/wikilinkCompletion.test.ts',
			// MIG-069 §B (2026-06-01) — Style Presets capture / apply engine.
			'tests/mig-069/stylePresets.test.ts',
			// MIG-076 §C (2026-06-12) — single content ownership: the cure's
			// acceptance harness, the runtime recipe harness (view-vs-disk
			// parity across every failure recipe), + characterization of the
			// current defects.
			'tests/mig-076/noteModel.test.ts',
			'tests/mig-076/runtimeHarness.test.ts',
			'tests/mig-076/currentBugRepro.test.ts',
			// PJ-070 §2 (2026-07-12) — the store-boundary test for adoptExternalChangeIntoTabs:
			// the watcher external-change adopt (clean adopt + reloadVersion bump / dirty-conflict
			// hook / cascade + echo + deleted + Focus-handoff guards) driven end-to-end sans Tauri.
			'tests/mig-076/watcherAdoptStore.test.ts',
			// PJ-092 redo (2026-07-13) — the flush-gate-exclude contract: flushAllTabsInLibrary
			// reports the not-durably-flushed paths → the rename cascade excludes them.
			'tests/mig-076/renameCascadeExclude.test.ts',
			// PJ-089 (2026-07-13) — Recipe Q: the Index-panel preview two-writable-model
			// clobber reproduction (two ids on one path → last-writer-wins, no reconcile).
			'tests/mig-076/indexPreviewClobber.test.ts',
			// PJ-102 (2026-07-14) — Recipe S: the manual-reopen recovery clobber
			// (ensure_cid_cn must never swap recovered content for stale disk).
			'tests/mig-076/reopenRecoveryClobber.test.ts',
			// PJ-108 (2026-07-15) — Recipe RO: a read-only host following a wikilink
			// must not consume the shared recovery net (openNoteTab preserveNet).
			'tests/mig-076/readonlyLinkPreservesNet.test.ts',
			// Sweep-2026-07-18 #2/#10 — Recipe HN: Alt+←/→ history nav must honor the
			// B1 one-path-one-tab dedup + resolveNoteContent recovery (loadTabHistoryEntry).
			'tests/mig-076/historyNavDedup.test.ts',
			// PJ-106 §A0/§A1 (2026-07-14) — the offset-pure RTL direction-resolution recipes
			// (deterministic base; the visual defects are the Boss's live staged tests).
			'tests/pj-106/rtlDirection.test.ts',
			'tests/pj-106/rtlMotion.test.ts',
			// PJ-106 §B1/§B2 (2026-07-15) — paragraph navigation (Ctrl+↑/↓) + select
			// line/paragraph, offset-pure and direction-blind (Arabic == Latin offsets).
			'tests/pj-106/paragraphNav.test.ts',
			// PJ-106 §B3 (2026-07-15) — select-sentence via Intl.Segmenter (UAX #29):
			// breaks on ؟ ! ۔ . but NOT ؛; no decimal false-break (design-inspection H4).
			'tests/pj-106/sentenceSelect.test.ts',
			// PJ-106 §B4 (2026-07-16) — per-paragraph direction override: RLM/LRM at
			// content start, markdown-safe placement, fence/table skips, idempotence.
			'tests/pj-106/paragraphDir.test.ts',
			// MIG-084 §F.2 (2026-06-23) — the Reviewer computed-priority engine.
			'tests/mig-084/priorities.test.ts',
			// MIG-090 §7 (2026-07-05) — the Workbench chips: pure intersection
			// (chips narrow, never expand — the hybrid union-append landmine pin).
			'tests/mig-090/chips.test.ts',
			// MIG-092 §2 (2026-07-05) — Collections membership reducers +
			// Bookmarks→Starred migration (idempotent, type-preserving, pinned).
			'tests/mig-092/collections.test.ts',
			// MIG-093 §B (2026-07-06) — the shared frontend fold (parity-pinned)
			// + the Light10 stem port the Index filter re-points to.
			'tests/mig-093/searchFold.test.ts',
			// MIG-093 §C (2026-07-06) — the switcher banded-ranking model (the
			// pinned Boss case: 'islam' ranks the exact title #1).
			'tests/mig-093/switcherRank.test.ts',
			// G4 (2026-07-08) — the yamlDoc round-trip authority, proven GREEN in
			// isolation (both app-killer recipes + byte-perfect untouched keys +
			// list projection + H1 malformed passthrough). The live noteModel swap
			// is DEFERRED until the projection is unified (an adversarial review
			// found the PropertyEditor's legacy projection ≠ the yamlDoc base would
			// corrupt block-scalar/quoted values on the first property edit).
			// `tests/g4/frontmatterRoundtrip.test.ts` remains as the RED Reproduce-
			// First repro of the legacy bug (run manually; out of the curated suite).
			'tests/g4/yamlDoc.test.ts',
			// G4 Phase 2 (2026-07-08, unified) — the LIVE noteModel save path proves
			// the review's app-killers are fixed: editing one field preserves block
			// scalars / quoted values / lists in every OTHER key (same parseFrontmatter
			// projection for base + current → no diff → byte-perfect).
			'tests/g4/noteModelRoundtrip.test.ts',
			// G4 Phase 3 (2026-07-08) — composeUpdatedContent (byte-perfect round-trip
			// write for closed-note tag/link adds) preserves rich frontmatter the lossy
			// buildFullContent destroys.
			'tests/g4/composeUpdated.test.ts',
			// MIG-100 §2 (2026-07-11) — the auto-session tracker (restore tabs on
			// relaunch): signature guards, debounce, serialization, cancel-and-flush.
			'tests/mig-100/session.test.ts',
			// MIG-100 §7 (2026-07-11) — the boot-restore recipes R1–R7: Gate #8
			// zero-write proof, 0-of-N deferred arm, switch-abort, focus safety,
			// model-as-source, deferred cid drain, arm-in-finally + sentinel.
			'tests/mig-100/restore.test.ts',
			// PJ-114 §0.2 (2026-07-17) — the shared parser-free wikilink finder
			// (findWikilinkAtLineOffset), extracted from CodeMirrorEditor's Ctrl-click
			// so FocusPane's FM+ affordances reuse one copy: hit predicate + alias/#heading strip.
			'tests/pj-114/linkAtPos.test.ts',
			// PJ-114 §3b (2026-07-18) — the shared living-link display helpers
			// (linkDisplay.ts): relative-time buckets + boundaries, the future-timestamp
			// clamp, the unknown-tier key-path guard, and i18n parity across all 15
			// locales for the two vocabularies the chip tooltip reuses.
			'tests/pj-114/linkDisplay.test.ts',
		],
		exclude: [
			'**/node_modules/**',
			'**/build/**',
			'**/.svelte-kit/**',
			'**/.claude/**',
			'tests/sight-v6/layout-fidelity.test.ts',
		],
	},
});

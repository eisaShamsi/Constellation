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
			// G4 Phase 0 (2026-07-08) — frontmatter round-trip Reproduce-First
			// harness (Recipe A nested-map/block-scalar loss; Recipe B quote
			// backslash-doubling). RED against the hand-rolled parser; Phase 1
			// turns them green against the yamlDoc module.
			'tests/g4/frontmatterRoundtrip.test.ts',
			// G4 Phase 1 (2026-07-08) — the yamlDoc round-trip authority proven
			// GREEN in isolation (both recipes + byte-perfect untouched keys + H1
			// malformed passthrough) before the live noteModel swap (Phase 2).
			'tests/g4/yamlDoc.test.ts',
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

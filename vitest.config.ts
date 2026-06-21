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

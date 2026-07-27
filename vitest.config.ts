/**
 * Vitest configuration.
 *
 * PJ-157 (2026-07-26) — CONTRACT CHANGE: the suite is glob-driven, not a curated
 * allow-list. Every `*.test.ts` / `*.spec.ts` under tests/ and src/ RUNS BY
 * DEFAULT. A test that must NOT run requires an explicit `exclude` entry below
 * WITH a reason comment — "not listed" can never again silently skip a test
 * (the old 52-entry allow-list meant a new test file that nobody remembered to
 * register never ran and never failed).
 *
 * Provenance for individual tests lives in each test file's own header, not here.
 *
 * Exclusions:
 *   - the node_modules / build / .svelte-kit / .claude dir globs — vitest
 *     REPLACES its default excludes when `exclude` is set; these four are
 *     load-bearing (worktree duplicates under .claude/worktrees/ must not
 *     run — MIG-030).
 *   - tests/sight-v6/layout-fidelity.test.ts — needs @playwright/test; deferred
 *     until a playwright runner is wanted (MIG-030).
 *   - tests/g4/frontmatterRoundtrip.test.ts — permanently RED by design: the
 *     Reproduce-First repro of the legacy `buildFullContent` frontmatter loss
 *     (store.ts). Removal condition: buildFullContent retired. Run it via
 *     `npm run test:red:frontmatter` (vitest.manual.config.mjs).
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
		include: ['tests/**/*.{test,spec}.ts', 'src/**/*.{test,spec}.ts'],
		exclude: [
			'**/node_modules/**',
			'**/build/**',
			'**/.svelte-kit/**',
			'**/.claude/**',
			// Needs @playwright/test — deferred (MIG-030); `npm run test:sight-v6:layout`.
			'tests/sight-v6/layout-fidelity.test.ts',
			// Permanently RED by design (legacy buildFullContent repro) — see header.
			// Run via `npm run test:red:frontmatter`.
			'tests/g4/frontmatterRoundtrip.test.ts',
		],
	},
});

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

export default defineConfig({
	test: {
		include: [
			'tests/sight-v6/perf.test.ts',
			'tests/sight-v6/tradition-isolation.test.ts',
			'tests/sight-v6/tradition-perf.test.ts',
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

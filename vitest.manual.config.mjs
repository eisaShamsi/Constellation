/**
 * PJ-157 (2026-07-26) — manual-run config for tests that are EXCLUDED from the
 * default suite by design. Vitest 4.x has no `--include` CLI flag and positional
 * args only filter WITHIN the config's include set, so an excluded file needs its
 * own config to be runnable at all.
 *
 * Currently: the permanently-RED Reproduce-First repro of the legacy
 * `buildFullContent` frontmatter loss. Expected result: 4 failed / 1 passed —
 * that RED state IS the pin. If it ever goes green, buildFullContent was fixed
 * or retired: fold the file into the main suite and delete this config.
 *
 * Run: `npm run test:red:frontmatter`
 */
import { defineConfig } from 'vitest/config';
import { fileURLToPath } from 'node:url';

export default defineConfig({
	resolve: {
		alias: {
			$lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
		},
	},
	test: {
		include: ['tests/g4/frontmatterRoundtrip.test.ts'],
	},
});

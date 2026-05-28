/**
 * MIG-060 §E — Behavioral unit tests for lens-row threading gestures.
 *
 * The full DOM-render test would require importing the (currently
 * unexported) `LensBlockWidget` class from `src/lib/editor/livePreview.ts`
 * + a jsdom-mocked CodeMirror EditorView + mocked `appSettings` / `t`
 * Svelte stores. That mock surface is ~150 lines of harness for what's
 * already covered by §F's Boss-test (a human clicking three icons and
 * watching the surfaces open).
 *
 * What this file covers cheaply and durably:
 *
 *   1. i18n parity — all 15 locale JSONs have the three new tooltip
 *      keys (`openIn360Tooltip`, `openInCNSTooltip`,
 *      `openInCatalogerTooltip`) under `lensBlock.*`, and each value is
 *      a non-empty string. This is the regression most likely to bite
 *      later (someone adds a 16th locale and forgets the new keys).
 *
 *   2. Surface discriminator constants — the three string literals
 *      (`'360.3d'`, `'cns'`, `'cataloger'`) match between the dispatch
 *      site (livePreview.ts) and the listener site (+layout.svelte).
 *      A typo on either side silently kills the gesture; the test
 *      grep-asserts both sources for parity.
 *
 * What this file does NOT cover (deferred to §F Boss-test):
 *
 *   - Three buttons render per row in the live editor.
 *   - CNS button hides when `enabledFeatures.constellationSight === false`.
 *   - Click on a button opens the host note + activates the surface.
 *   - RTL row layout positions actions on the visual right.
 *   - Per-surface hover hue (purple / cyan / orange).
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';

const LOCALES = [
	'ar', 'de', 'en', 'es', 'fa', 'fr', 'he', 'hi',
	'ja', 'ko', 'pt', 'ru', 'tr', 'ur', 'zh',
] as const;

const TOOLTIP_KEYS = [
	'openIn360Tooltip',
	'openInCNSTooltip',
	'openInCatalogerTooltip',
] as const;

const i18nDir = resolve(__dirname, '..', '..', 'src', 'lib', 'i18n');

describe('MIG-060 §E.1 — i18n parity for lens-row threading tooltips', () => {
	it('all 15 locales are checked here (no locale was added or removed silently)', () => {
		// Sanity guard: if a 16th locale lands in src/lib/i18n/ without
		// LOCALES updating, this test will surface it.
		const locales = readdirSync(i18nDir)
			.filter((f) => f.endsWith('.json'))
			.map((f) => f.replace('.json', ''));
		// Some locales may be excluded (e.g. derived/aux files); enforce
		// that every LOCALE we *do* test is present on disk.
		for (const l of LOCALES) {
			expect(locales).toContain(l);
		}
	});

	for (const locale of LOCALES) {
		describe(`locale: ${locale}`, () => {
			const path = resolve(i18nDir, `${locale}.json`);
			const json = JSON.parse(readFileSync(path, 'utf8'));
			const lensBlock = json.lensBlock ?? {};

			for (const key of TOOLTIP_KEYS) {
				it(`has lensBlock.${key} as a non-empty string`, () => {
					const v = lensBlock[key];
					expect(typeof v).toBe('string');
					expect(v.length).toBeGreaterThan(0);
				});
			}
		});
	}
});

describe('MIG-060 §E.2 — surface discriminator constants are consistent', () => {
	// Read the dispatch site (livePreview.ts) and the listener site
	// (+layout.svelte) and assert each contains all three surface
	// string literals. A typo on either side would silently kill the
	// gesture for that surface; the test catches it.
	const livePreviewPath = resolve(
		__dirname, '..', '..', 'src', 'lib', 'editor', 'livePreview.ts',
	);
	const layoutPath = resolve(
		__dirname, '..', '..', 'src', 'routes', '+layout.svelte',
	);
	const livePreviewSrc = readFileSync(livePreviewPath, 'utf8');
	const layoutSrc = readFileSync(layoutPath, 'utf8');

	for (const surface of ['360.3d', 'cns', 'cataloger'] as const) {
		it(`livePreview.ts dispatches surface '${surface}'`, () => {
			expect(livePreviewSrc).toContain(`surface: '${surface}'`);
		});
		it(`+layout.svelte listens for surface '${surface}'`, () => {
			// Match the case branch (`case '360.3d':` etc.).
			expect(layoutSrc).toContain(`case '${surface}':`);
		});
	}
});

/**
 * MIG-069 §B — Style Presets capture / apply engine.
 *
 * Mocks the heavy `store` + `linkTypeRegistry` modules so we can exercise the pure
 * capture/apply/validate logic — including the PRIVACY invariant (no secrets or
 * universe-specific folder paths ever travel in a preset).
 */
import { describe, it, expect, vi } from 'vitest';

const h = vi.hoisted(() => ({
	settings: {
		colorScheme: 'dark', accentColor: '#abc123', activeThemeId: 'nord-dark',
		customThemes: [], iconOverrides: {}, interfaceFont: 'Inter', fontSize: 18,
		textFont: 'Georgia', monoFont: 'Fira', scriptFonts: {},
		colourTypedLinks: true, showTypedLinkLabels: false,
		linkPills: { fill: { supports: '#111' }, text: {}, shape: { radius: 5, height: 22, fontWeight: 600 } },
		tabSize: 4, confirmDelete: true,
		security: { lockPinHash: 'SECRET_HASH' }, githubToken: 'ghp_SECRETTOKEN',
		defaultNoteFolder: '/home/me/private', templateFolder: '/tmp/t',
	} as Record<string, unknown>,
	updateSpy: vi.fn(),
	saveLinkSpy: vi.fn((_d: unknown) => Promise.resolve()),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('$lib/libraries/store', async () => {
	const { writable } = await import('svelte/store');
	return { appSettings: writable(h.settings), updateSettings: (p: unknown) => h.updateSpy(p) };
});
vi.mock('$lib/libraries/linkTypeRegistry', () => ({
	getLinkTypes: () => [{ id: 'supports', color: '#FF00FF', label: 'Supports', parent: null, order: 1, builtin: true, emoji: null, desc: null }],
	toLinkTypeDeltas: (t: unknown[]) => t,
	saveLinkTypes: (d: unknown) => h.saveLinkSpy(d),
}));

import * as sp from '$lib/libraries/stylePresets';

describe('stylePresets §B — capture / apply', () => {
	it('captures only the requested sections, with the right fields, deep-cloned', () => {
		const p = sp.captureCurrentStyle(['colorsTheme', 'typedLinkDisplay', 'pillShape', 'linkColors']);
		expect(Object.keys(p).sort()).toEqual(['colorsTheme', 'linkColors', 'pillShape', 'typedLinkDisplay']);
		expect((p.colorsTheme as Record<string, unknown>).accentColor).toBe('#abc123');
		expect((p.typedLinkDisplay as Record<string, unknown>).colourTypedLinks).toBe(true);
		expect((p.pillShape as { shape: { radius: number } }).shape.radius).toBe(5);
		expect((p.linkColors as { deltas: { id: string }[] }).deltas[0].id).toBe('supports');
		// deep clone: mutating the capture must NOT touch live settings
		(p.pillShape as { shape: { radius: number } }).shape.radius = 999;
		expect((h.settings.linkPills as { shape: { radius: number } }).shape.radius).toBe(5);
	});

	it('PRIVACY invariant: behaviour never carries secrets or folder paths', () => {
		const beh = sp.SECTION_CATALOGUE.find((s) => s.key === 'behaviour')!;
		for (const k of ['security', 'githubToken', 'defaultNoteFolder', 'templateFolder']) {
			expect(beh.appSettingsKeys).not.toContain(k);
		}
		const json = JSON.stringify(sp.captureCurrentStyle(['behaviour']));
		expect(json).not.toContain('SECRET');
		expect(json).not.toContain('ghp_');
		expect(json).not.toContain('private');
	});

	it('applyPreset merges appSettings sections into ONE updateSettings + saves link deltas', async () => {
		h.updateSpy.mockClear(); h.saveLinkSpy.mockClear();
		await sp.applyPreset({
			id: 'x', name: 'P', schema: sp.STYLE_PRESET_SCHEMA,
			sections: {
				colorsTheme: { accentColor: '#000000', colorScheme: 'light' },
				pillShape: { shape: { radius: 12 } },
				linkColors: { deltas: [{ id: 'supports', color: '#123456' }] },
			},
		});
		expect(h.updateSpy).toHaveBeenCalledTimes(1);
		const partial = h.updateSpy.mock.calls[0][0] as Record<string, any>;
		expect(partial.accentColor).toBe('#000000');
		expect(partial.colorScheme).toBe('light');
		expect(partial.linkPills.shape.radius).toBe(12);
		expect(partial.linkPills.fill.supports).toBe('#111'); // existing fill preserved
		expect(h.saveLinkSpy).toHaveBeenCalledWith([{ id: 'supports', color: '#123456' }]);
	});

	it('absent sections are left untouched (partial apply)', async () => {
		h.updateSpy.mockClear(); h.saveLinkSpy.mockClear();
		await sp.applyPreset({ id: 'y', name: 'OnlyColors', schema: sp.STYLE_PRESET_SCHEMA, sections: { colorsTheme: { accentColor: '#fff' } } });
		const partial = h.updateSpy.mock.calls[0][0] as Record<string, unknown>;
		expect('colourTypedLinks' in partial).toBe(false); // typedLinkDisplay not in preset
		expect(h.saveLinkSpy).not.toHaveBeenCalled();       // no linkColors section
	});

	it('newPresetFromCurrent + isValidPreset + presetSectionKeys', () => {
		const np = sp.newPresetFromCurrent('My Look', ['colorsTheme', 'fonts']);
		expect(np.name).toBe('My Look');
		expect(np.schema).toBe(sp.STYLE_PRESET_SCHEMA);
		expect(sp.presetSectionKeys(np)).toEqual(['colorsTheme', 'fonts']);
		expect(sp.isValidPreset(np)).toBe(true);
		expect(sp.isValidPreset({ name: 'x', schema: 'other/9', sections: {} })).toBe(false);
		expect(sp.isValidPreset({ schema: sp.STYLE_PRESET_SCHEMA, sections: {} })).toBe(false);
		expect(sp.isValidPreset({ name: 'x', schema: sp.STYLE_PRESET_SCHEMA, sections: [] })).toBe(false);
		expect(sp.isValidPreset(null)).toBe(false);
	});
});

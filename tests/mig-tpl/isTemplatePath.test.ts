/**
 * MIG-TPL §1 — the identity-clean guard.
 *
 * Boss ruling 2026-07-19: **a template never carries `cid_cn` or a creation date.** A template is
 * a MOLD; identity and birth belong to the CAST. Without this predicate the app would break the
 * rule itself — simply OPENING a template to edit it runs `ensure_cid_cn_cmd`, which injects a
 * `cid_cn:` into the file, so every mold would acquire an identity on first edit and every note
 * cast from it would inherit the mold's identity line.
 *
 * `isTemplatePath` gates both `ensure_cid_cn_cmd` call sites in `store.ts`. These tests pin the
 * matching rules — especially the false-positive case, because over-matching would silently stop
 * stamping REAL notes, which is a far worse failure than under-matching a template.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { appSettings, isTemplatePath } from '$lib/libraries/store';

function setFolder(folder: string) {
	appSettings.update((s: any) => ({ ...s, templateFolder: folder }));
}

beforeEach(() => setFolder('Templates'));

describe('isTemplatePath — a relative folder matches as a PATH SEGMENT', () => {
	it('matches a template directly in the folder', () => {
		expect(isTemplatePath('E:/Universe/Templates/Daily.md')).toBe(true);
	});

	it('matches a template in a SUBfolder (subfolders are supported)', () => {
		expect(isTemplatePath('E:/Universe/Templates/Work/Standup.md')).toBe(true);
	});

	it('matches with Windows backslashes', () => {
		expect(isTemplatePath('E:\\Universe\\Templates\\Daily.md')).toBe(true);
	});

	it('is case-insensitive', () => {
		expect(isTemplatePath('E:/Universe/templates/Daily.md')).toBe(true);
		expect(isTemplatePath('E:/Universe/TEMPLATES/Daily.md')).toBe(true);
	});

	it('does NOT match a folder that merely CONTAINS the name — the false-positive that matters', () => {
		// Over-matching would stop stamping real notes with their identity, which is worse than
		// under-matching a template. "Templates" must not match "MyTemplatesArchive".
		expect(isTemplatePath('E:/Universe/MyTemplatesArchive/Note.md')).toBe(false);
		expect(isTemplatePath('E:/Universe/TemplatesOld/Note.md')).toBe(false);
		expect(isTemplatePath('E:/Universe/Notes/Templates-ideas.md')).toBe(false);
	});

	it('does not match an ordinary note elsewhere in the Universe', () => {
		expect(isTemplatePath('E:/Universe/Research/Note.md')).toBe(false);
		expect(isTemplatePath('E:/Universe/Daily/2026-07-19.md')).toBe(false);
	});

	it('does not match a note merely NAMED like the folder', () => {
		expect(isTemplatePath('E:/Universe/Research/Templates.md')).toBe(false);
	});
});

describe('isTemplatePath — an absolute folder (the Settings folder-picker case)', () => {
	beforeEach(() => setFolder('D:/Shared/MyTemplates'));

	it('matches anything under the chosen absolute folder', () => {
		expect(isTemplatePath('D:/Shared/MyTemplates/Daily.md')).toBe(true);
		expect(isTemplatePath('D:/Shared/MyTemplates/Work/Standup.md')).toBe(true);
	});

	it('matches regardless of separator style and case', () => {
		expect(isTemplatePath('D:\\Shared\\MyTemplates\\Daily.md')).toBe(true);
		expect(isTemplatePath('d:/shared/mytemplates/daily.md')).toBe(true);
	});

	it('does not match a sibling folder with the same prefix', () => {
		expect(isTemplatePath('D:/Shared/MyTemplatesBackup/Daily.md')).toBe(false);
	});

	it('does not match a note outside it', () => {
		expect(isTemplatePath('E:/Universe/Research/Note.md')).toBe(false);
	});
});

describe('isTemplatePath — degenerate settings never over-match', () => {
	it('an empty setting falls back to "Templates" — matching the RUST fallback exactly', () => {
		// Both sides must agree. `resolve_templates_dir` (universe.rs) resolves an empty setting to
		// `<universe>/Templates`, so that IS where templates live — and the guard must recognise
		// them there, or the app would list a folder whose files it then stamps with an identity.
		// Divergence between these two fallbacks is the bug this case exists to prevent.
		setFolder('');
		expect(isTemplatePath('E:/Universe/Templates/Daily.md')).toBe(true);
		expect(isTemplatePath('E:/Universe/Research/Note.md')).toBe(false);
	});

	it('a whitespace-only setting behaves like empty (same fallback)', () => {
		setFolder('   ');
		expect(isTemplatePath('E:/Universe/Templates/Daily.md')).toBe(true);
		expect(isTemplatePath('E:/Universe/Research/Note.md')).toBe(false);
	});

	it('a trailing slash in the setting is tolerated', () => {
		setFolder('Templates/');
		expect(isTemplatePath('E:/Universe/Templates/Daily.md')).toBe(true);
	});
});

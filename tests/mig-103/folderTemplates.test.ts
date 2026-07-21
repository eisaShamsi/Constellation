/**
 * MIG-103 D3 — folder default templates, deepest-wins.
 *
 * The headline test is `substring_matches_are_refused`: the shipped matcher used
 * `noteFolder.includes(configuredFolder)`, so a folder configured as `Books` also
 * matched `/Cookbooks/`, `/MyBooks/` and `/Notebooks/` — silently applying a
 * template to notes that should never receive one. That test fails against the
 * old logic and passes against the path-prefix matcher.
 */
import { describe, it, expect } from 'vitest';
import {
	resolveFolderTemplate,
	isAncestorOrSame,
	normalizeFolder,
	templateFileName,
} from '$lib/templates/folderTemplates';

const LIB = 'E:/Universe/Library';

describe('MIG-103 D3 — resolving a folder default template', () => {
	it('off by default — an empty map applies nothing', () => {
		expect(resolveFolderTemplate(`${LIB}/Books`, {})).toBeNull();
		expect(resolveFolderTemplate(`${LIB}/Books`, undefined)).toBeNull();
		expect(resolveFolderTemplate(`${LIB}/Books`, null)).toBeNull();
	});

	it('applies to the configured folder itself', () => {
		const map = { [`${LIB}/Books`]: 'Book Note' };
		expect(resolveFolderTemplate(`${LIB}/Books`, map)).toBe('Book Note');
	});

	it('applies to a descendant folder', () => {
		const map = { [`${LIB}/Books`]: 'Book Note' };
		expect(resolveFolderTemplate(`${LIB}/Books/Fiction/2026`, map)).toBe('Book Note');
	});

	it('does NOT apply to a sibling or an unrelated folder', () => {
		const map = { [`${LIB}/Books`]: 'Book Note' };
		expect(resolveFolderTemplate(`${LIB}/Papers`, map)).toBeNull();
		expect(resolveFolderTemplate(LIB, map)).toBeNull(); // the parent is not covered
	});

	/** THE REGRESSION — the shipped matcher was `includes()`, a substring test. */
	it('substring matches are refused — Books must not match Cookbooks', () => {
		const map = { [`${LIB}/Books`]: 'Book Note' };
		expect(resolveFolderTemplate(`${LIB}/Cookbooks`, map)).toBeNull();
		expect(resolveFolderTemplate(`${LIB}/MyBooks/Draft`, map)).toBeNull();
		expect(resolveFolderTemplate(`${LIB}/Notebooks`, map)).toBeNull();
		// And the reverse direction: a deeper configured folder must not match a
		// shallower note folder.
		expect(resolveFolderTemplate(`${LIB}`, { [`${LIB}/Books/Fiction`]: 'X' })).toBeNull();
	});

	it('DEEPEST WINS — the longest matching path prefix', () => {
		const map = {
			[`${LIB}/Books`]: 'Book Note',
			[`${LIB}/Books/Fiction`]: 'Fiction Note',
		};
		expect(resolveFolderTemplate(`${LIB}/Books/History`, map)).toBe('Book Note');
		expect(resolveFolderTemplate(`${LIB}/Books/Fiction`, map)).toBe('Fiction Note');
		expect(resolveFolderTemplate(`${LIB}/Books/Fiction/2026`, map)).toBe('Fiction Note');
	});

	it('an empty value means CLEARED, not matched', () => {
		const map = { [`${LIB}/Books`]: 'Book Note', [`${LIB}/Books/Fiction`]: '' };
		// Fiction is cleared, so the nearest configured ancestor applies.
		expect(resolveFolderTemplate(`${LIB}/Books/Fiction`, map)).toBe('Book Note');
	});

	/** The carve-out: creating a template must not itself fire a template. */
	it('excluded folders never receive a template', () => {
		const map = { [LIB]: 'Everything' };
		const excluded = [`${LIB}/Templates`];
		expect(resolveFolderTemplate(`${LIB}/Notes`, map, excluded)).toBe('Everything');
		expect(resolveFolderTemplate(`${LIB}/Templates`, map, excluded)).toBeNull();
		expect(resolveFolderTemplate(`${LIB}/Templates/Books`, map, excluded)).toBeNull();
	});

	it('is separator- and case-insensitive (Windows paths)', () => {
		const map = { 'E:\\Universe\\Library\\Books': 'Book Note' };
		expect(resolveFolderTemplate('E:/universe/library/books/fiction', map)).toBe('Book Note');
		expect(resolveFolderTemplate('E:\\Universe\\Library\\Books', map)).toBe('Book Note');
	});

	it('tolerates trailing slashes on either side', () => {
		const map = { [`${LIB}/Books/`]: 'Book Note' };
		expect(resolveFolderTemplate(`${LIB}/Books/`, map)).toBe('Book Note');
		expect(resolveFolderTemplate(`${LIB}/Books`, map)).toBe('Book Note');
	});

	it('helpers behave', () => {
		expect(normalizeFolder('A\\B\\')).toBe('a/b');
		expect(isAncestorOrSame('/a', '/a/b')).toBe(true);
		expect(isAncestorOrSame('/a', '/ab')).toBe(false);
		expect(isAncestorOrSame('', '/a')).toBe(false);
		expect(templateFileName('Book')).toBe('Book.md');
		expect(templateFileName('Book.md')).toBe('Book.md');
	});
});

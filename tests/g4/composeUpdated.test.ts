/**
 * G4 Phase 3 — composeUpdatedContent (the byte-perfect replacement for
 * buildFullContent on a read→edit-props→write of an EXISTING note, e.g. adding a
 * tag/link to a CLOSED note). It must preserve rich frontmatter the lossy
 * buildFullContent would destroy.
 */
import { describe, it, expect } from 'vitest';
import { composeUpdatedContent, parseFrontmatter, buildFullContent } from '$lib/libraries/store';

const RICH = [
	'---',
	'title: Closed Note',
	'source:',
	'  author: Ibn Khaldun',
	'  year: 1377',
	'description: |',
	'  first line',
	'  second line',
	"quote: 'He said: \"hi\"'",
	'tags:',
	'  - a',
	'---',
	'body text',
	'',
].join('\n');

describe('G4 Phase 3 — composeUpdatedContent preserves rich frontmatter on a closed-note edit', () => {
	it('editing a scalar prop keeps nested map + block scalar + quoted value byte-perfect', () => {
		const { properties, body } = parseFrontmatter(RICH);
		const newProps = properties.map((p) => (p.key === 'title' ? { ...p, value: 'Closed Note EDITED' } : p));
		const out = composeUpdatedContent(RICH, newProps, body);
		expect(out).toContain('title: Closed Note EDITED');
		expect(out).toContain('author: Ibn Khaldun'); // nested map preserved
		expect(out).toContain('year: 1377');
		expect(out).toContain('first line'); // block scalar preserved
		expect(out).toContain('second line');
		expect(out).toContain("quote: 'He said: \"hi\"'"); // quoted value byte-perfect
		expect(out).not.toContain('description: "|"'); // NOT the corruption signature
		expect(out).toContain('body text');
	});

	it('a no-op update is byte-perfect', () => {
		const { properties, body } = parseFrontmatter(RICH);
		expect(composeUpdatedContent(RICH, properties, body)).toBe(RICH);
	});

	it('demonstrates the hazard it replaces: the old buildFullContent DESTROYS the same note', () => {
		// This locks in WHY Phase 3 exists — the legacy path is lossy on this note.
		const { properties, body } = parseFrontmatter(RICH);
		const legacy = buildFullContent(properties, body);
		// the block scalar collapses to the literal "|" and the nested children vanish
		expect(legacy).not.toContain('first line'); // proof the old path drops it
	});
});

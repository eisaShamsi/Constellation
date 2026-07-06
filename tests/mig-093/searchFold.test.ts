/**
 * MIG-093 §B — the shared frontend fold (foldForMatch / stemArabicLight10).
 *
 * Guards: (1) fold parity with the documented Rust decisions (NFC + Unicode
 * lowercase + tashkeel/tatweel strip + alef/maqṣūra/tāʾ-marbūṭa unification,
 * hamza-bearers ؤ/ئ DISTINCT); (2) the stem layer stays the exact Light10
 * port the Index filter shipped with; (3) fold is for matching only —
 * deterministic, no length assumptions.
 */
import { describe, it, expect } from 'vitest';
import { foldForMatch, stemArabicLight10 } from '$lib/searchFold';

describe('MIG-093 §B — foldForMatch', () => {
	it('case-folds full Unicode (not just ASCII)', () => {
		expect(foldForMatch('Islam')).toBe('islam');
		expect(foldForMatch('Île-de-France')).toBe('île-de-france'); // Î → î (fold, not strip)
	});

	it('NFC-normalizes so decomposed input matches precomposed', () => {
		// "é" as e + combining acute (NFD) folds equal to precomposed é
		expect(foldForMatch('café')).toBe(foldForMatch('café'));
	});

	it('strips Arabic tashkeel + tatweel', () => {
		expect(foldForMatch('كَتَبَ')).toBe('كتب');
		expect(foldForMatch('كــتــاب')).toBe('كتاب');
		expect(foldForMatch('المَعْرِفَة')).toBe(foldForMatch('المعرفة'));
	});

	it('unifies alef variants, alef maqsura, ta marbuta', () => {
		expect(foldForMatch('أحمد')).toBe('احمد');
		expect(foldForMatch('إسلام')).toBe('اسلام');
		expect(foldForMatch('آفاق')).toBe('افاق');
		expect(foldForMatch('مصطفى')).toBe('مصطفي'); // ى → ي
		expect(foldForMatch('معرفة')).toBe('معرفه'); // ة → ه
	});

	it('keeps hamza-bearers ؤ/ئ distinct (the Light10 false-positive guard)', () => {
		expect(foldForMatch('سؤال')).toBe('سؤال');
		expect(foldForMatch('قائمة')).toBe('قائمه'); // only ة folds; ئ stays
	});

	it('a folded query matches a folded title regardless of diacritics/case', () => {
		const title = 'الزِّراعة المستدامة';
		expect(foldForMatch(title).includes(foldForMatch('الزراعة'))).toBe(true);
		expect(foldForMatch('Sustainable AGRICULTURE').includes(foldForMatch('agriculture'))).toBe(true);
	});
});

describe('MIG-093 §B — stemArabicLight10 (the Index filter port, unchanged)', () => {
	it('sequential prefix strip: والمعرفة → معرف', () => {
		expect(stemArabicLight10('والمعرفة')).toBe('معرف');
	});
	it('definite article strip: المعرفة → معرف', () => {
		expect(stemArabicLight10('المعرفة')).toBe('معرف');
	});
	it('suffix strip: مسلمون → مسلم', () => {
		expect(stemArabicLight10('مسلمون')).toBe('مسلم');
	});
	it('short words untouched by prefix rules', () => {
		expect(stemArabicLight10('علم')).toBe('علم');
	});
});

/**
 * MIG-093 §B — the ONE shared frontend fold for user-facing name/term matching.
 *
 * Extracted from IndexPanel.svelte's inline `normalizeArabicForFilter`
 * (MIG-091-era) so the Quick Switcher, the Index filter, and any future
 * matcher agree on what matches — the no-copy-paste rule.
 *
 * ## Parity with the Rust side (documented, keep in sync)
 * - `foldForMatch` = NFC + full-Unicode lowercase (the `fold_match_key`
 *   recipe in search.rs) + tashkeel/tatweel strip (`normalize_stripped`,
 *   arabic/normalizer.rs) + the alef/maqṣūra/tāʾ-marbūṭa letter unification
 *   (the `fold_letters` collapses MINUS the hamza-bearer pair ؤ/ئ — matching
 *   the Index filter's long-shipped behavior; hamza-bearers stay distinct to
 *   avoid the documented Light10 false-positive class).
 * - `stemArabicLight10` = `foldForMatch` + the sequential 3/2/1-char prefix
 *   and 2/1-char suffix strip — the exact JS port of the backend
 *   `stem_arabic_light10` (libraries.rs). For TERM matching (the Index);
 *   NOT for title matching (stemming over-matches proper names).
 *
 * Fold for MATCHING only — always display the raw string.
 */

const ARABIC_DIACRITICS_RE = /[ً-ٰٟۖ-ۭـ]/g; // tashkeel + Quranic marks + tatweel
const ARABIC_ALEF_VARIANTS_RE = /[آأإٱ]/g; // آ أ إ ٱ → ا
const ARABIC_ALEF_MAKSURA_RE = /[ى]/g; // ى → ي
const ARABIC_TA_MARBUTA_RE = /[ة]/g; // ة → ه

/** The base fold: case-insensitive, diacritic-insensitive (Arabic), variant-unified. */
export function foldForMatch(s: string): string {
	let t = s.normalize('NFC').toLowerCase();
	t = t.replace(ARABIC_DIACRITICS_RE, '');
	t = t.replace(ARABIC_ALEF_VARIANTS_RE, 'ا');
	t = t.replace(ARABIC_ALEF_MAKSURA_RE, 'ي');
	t = t.replace(ARABIC_TA_MARBUTA_RE, 'ه');
	return t;
}

/** Exact JS port of the backend `stem_arabic_light10` (libraries.rs):
 *  fold + sequential 3/2/1-char prefix strip + 2/1-char suffix strip.
 *  Sequential so "والمعرفة" → "معرف" in one pass. */
export function stemArabicLight10(s: string): string {
	const t = foldForMatch(s);

	let chars = Array.from(t);
	let len = chars.length;

	if (len >= 6) {
		const p = chars[0] + chars[1] + chars[2];
		if (p === 'وال' || p === 'بال' || p === 'كال' || p === 'فال') {
			chars = chars.slice(3);
			len = chars.length;
		}
	}
	if (len >= 4) {
		const p = chars[0] + chars[1];
		if (p === 'ال' || p === 'لل') {
			chars = chars.slice(2);
			len = chars.length;
		}
	}
	if (len >= 4 && chars[0] === 'و') {
		chars = chars.slice(1);
		len = chars.length;
	}

	if (len >= 4) {
		const s2 = chars[len - 2] + chars[len - 1];
		if (
			s2 === 'ها' || s2 === 'ان' || s2 === 'ات' || s2 === 'ون' ||
			s2 === 'ين' || s2 === 'يه' || s2 === 'يت' || s2 === 'ته'
		) {
			chars = chars.slice(0, len - 2);
			len = chars.length;
		}
	}
	if (len >= 3) {
		const last = chars[len - 1];
		if (last === 'ه' || last === 'ي') {
			chars = chars.slice(0, len - 1);
		}
	}

	return chars.join('');
}

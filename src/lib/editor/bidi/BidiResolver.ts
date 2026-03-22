/**
 * BidiResolver — bidirectional text support for the Constellation Editor.
 *
 * Leverages the browser's native bidi algorithm via dir="auto" on line elements.
 * Provides helpers for direction detection and cursor movement at bidi boundaries.
 */

// Strong RTL characters: Arabic, Hebrew, Thaana, NKo, Samaritan, etc.
const RTL_REGEX = /[\u0590-\u05FF\u0600-\u06FF\u0700-\u074F\u0780-\u07BF\u07C0-\u07FF\u0800-\u083F\u0840-\u085F\u08A0-\u08FF\uFB1D-\uFB4F\uFB50-\uFDFF\uFE70-\uFEFF]/;
const LTR_REGEX = /[A-Za-z\u00C0-\u024F\u0370-\u03FF\u0400-\u04FF\u1E00-\u1EFF]/;

/**
 * Detect the base direction of a text block based on the first strong character.
 */
export function detectDirection(text: string): 'ltr' | 'rtl' {
	for (let i = 0; i < text.length && i < 200; i++) {
		if (RTL_REGEX.test(text[i])) return 'rtl';
		if (LTR_REGEX.test(text[i])) return 'ltr';
	}
	return 'ltr'; // Default to LTR if no strong character found
}

/**
 * Check if a character is an RTL character.
 */
export function isRTLChar(char: string): boolean {
	return RTL_REGEX.test(char);
}

/**
 * Check if a character is an LTR character.
 */
export function isLTRChar(char: string): boolean {
	return LTR_REGEX.test(char);
}

/**
 * Get the direction attribute value for a line element.
 */
export function getLineDir(lineText: string, baseDir: 'ltr' | 'rtl' | 'auto'): 'ltr' | 'rtl' | 'auto' {
	if (baseDir !== 'auto') return baseDir;
	return 'auto'; // Let the browser handle per-line direction
}

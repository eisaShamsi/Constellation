/**
 * CursorMotion — visual cursor movement for bidirectional text.
 *
 * Arrow keys should follow visual order, not logical order,
 * when crossing RTL/LTR boundaries.
 */

import { isRTLChar, isLTRChar } from './BidiResolver';

/**
 * Determine the visual direction of cursor movement based on
 * the text around the cursor and the base direction.
 *
 * Returns the logical offset to move to for a "visual left" or "visual right" press.
 */
export function visualArrowLeft(
	text: string,
	offset: number,
	baseDir: 'ltr' | 'rtl'
): number {
	// In LTR base: visual left = logical backward
	// In RTL base: visual left = logical forward
	if (baseDir === 'rtl') {
		return Math.min(text.length, offset + 1);
	}
	return Math.max(0, offset - 1);
}

export function visualArrowRight(
	text: string,
	offset: number,
	baseDir: 'ltr' | 'rtl'
): number {
	if (baseDir === 'rtl') {
		return Math.max(0, offset - 1);
	}
	return Math.min(text.length, offset + 1);
}

/**
 * Find the next word boundary in visual order.
 */
export function visualWordLeft(
	text: string,
	offset: number,
	baseDir: 'ltr' | 'rtl'
): number {
	if (baseDir === 'rtl') {
		// Visual left in RTL = logical forward
		let pos = offset;
		while (pos < text.length && /\s/.test(text[pos])) pos++;
		while (pos < text.length && /\S/.test(text[pos])) pos++;
		return pos;
	}
	// Visual left in LTR = logical backward
	let pos = offset;
	if (pos > 0) pos--;
	while (pos > 0 && /\s/.test(text[pos])) pos--;
	while (pos > 0 && /\S/.test(text[pos - 1])) pos--;
	return pos;
}

export function visualWordRight(
	text: string,
	offset: number,
	baseDir: 'ltr' | 'rtl'
): number {
	if (baseDir === 'rtl') {
		let pos = offset;
		if (pos > 0) pos--;
		while (pos > 0 && /\s/.test(text[pos])) pos--;
		while (pos > 0 && /\S/.test(text[pos - 1])) pos--;
		return pos;
	}
	let pos = offset;
	while (pos < text.length && /\S/.test(text[pos])) pos++;
	while (pos < text.length && /\s/.test(text[pos])) pos++;
	return pos;
}

/**
 * InlineParser — parses inline Markdown marks within a text range.
 */

import type { InlineToken, InlineType } from './tokens';

export function parseInlines(text: string, baseOffset: number): InlineToken[] {
	const tokens: InlineToken[] = [];
	let i = 0;

	while (i < text.length) {
		// Bold **text** or __text__
		let match = matchDelimited(text, i, '**', '**');
		if (!match) match = matchDelimited(text, i, '__', '__');
		if (match) {
			tokens.push({
				type: 'bold',
				from: baseOffset + i,
				to: baseOffset + match.end,
				syntaxFrom: baseOffset + i,
				syntaxTo: baseOffset + i + match.openLen,
				syntaxEndFrom: baseOffset + match.end - match.closeLen,
				syntaxEndTo: baseOffset + match.end,
				content: match.content,
				children: parseInlines(match.content, baseOffset + i + match.openLen),
			});
			i = match.end;
			continue;
		}

		// Italic *text* or _text_
		match = matchSingleDelimited(text, i, '*', '*');
		if (!match) match = matchSingleDelimited(text, i, '_', '_');
		if (match) {
			tokens.push({
				type: 'italic',
				from: baseOffset + i,
				to: baseOffset + match.end,
				syntaxFrom: baseOffset + i,
				syntaxTo: baseOffset + i + 1,
				syntaxEndFrom: baseOffset + match.end - 1,
				syntaxEndTo: baseOffset + match.end,
				content: match.content,
				children: parseInlines(match.content, baseOffset + i + 1),
			});
			i = match.end;
			continue;
		}

		// Strikethrough ~~text~~
		match = matchDelimited(text, i, '~~', '~~');
		if (match) {
			tokens.push({
				type: 'strikethrough',
				from: baseOffset + i,
				to: baseOffset + match.end,
				syntaxFrom: baseOffset + i,
				syntaxTo: baseOffset + i + 2,
				syntaxEndFrom: baseOffset + match.end - 2,
				syntaxEndTo: baseOffset + match.end,
				content: match.content,
				children: parseInlines(match.content, baseOffset + i + 2),
			});
			i = match.end;
			continue;
		}

		// Highlight ==text==
		match = matchDelimited(text, i, '==', '==');
		if (match) {
			tokens.push({
				type: 'highlight',
				from: baseOffset + i,
				to: baseOffset + match.end,
				syntaxFrom: baseOffset + i,
				syntaxTo: baseOffset + i + 2,
				syntaxEndFrom: baseOffset + match.end - 2,
				syntaxEndTo: baseOffset + match.end,
				content: match.content,
				children: parseInlines(match.content, baseOffset + i + 2),
			});
			i = match.end;
			continue;
		}

		// Inline code `text`
		match = matchDelimited(text, i, '`', '`');
		if (match) {
			tokens.push({
				type: 'inlineCode',
				from: baseOffset + i,
				to: baseOffset + match.end,
				syntaxFrom: baseOffset + i,
				syntaxTo: baseOffset + i + 1,
				syntaxEndFrom: baseOffset + match.end - 1,
				syntaxEndTo: baseOffset + match.end,
				content: match.content,
			});
			i = match.end;
			continue;
		}

		// Inline math $text$
		if (text[i] === '$' && text[i + 1] !== '$') {
			match = matchDelimited(text, i, '$', '$');
			if (match && !match.content.includes('\n')) {
				tokens.push({
					type: 'inlineMath',
					from: baseOffset + i,
					to: baseOffset + match.end,
					syntaxFrom: baseOffset + i,
					syntaxTo: baseOffset + i + 1,
					syntaxEndFrom: baseOffset + match.end - 1,
					syntaxEndTo: baseOffset + match.end,
					content: match.content,
				});
				i = match.end;
				continue;
			}
		}

		// Wikilink [[target|display]] or [[target]]
		if (text[i] === '[' && text[i + 1] === '[') {
			const closeIdx = text.indexOf(']]', i + 2);
			if (closeIdx >= 0) {
				const inner = text.substring(i + 2, closeIdx);
				const pipeIdx = inner.indexOf('|');
				const target = pipeIdx >= 0 ? inner.substring(0, pipeIdx) : inner;
				const display = pipeIdx >= 0 ? inner.substring(pipeIdx + 1) : inner;

				tokens.push({
					type: 'wikilink',
					from: baseOffset + i,
					to: baseOffset + closeIdx + 2,
					syntaxFrom: baseOffset + i,
					syntaxTo: baseOffset + i + 2,
					syntaxEndFrom: baseOffset + closeIdx,
					syntaxEndTo: baseOffset + closeIdx + 2,
					content: display,
					target,
				});
				i = closeIdx + 2;
				continue;
			}
		}

		// Image ![alt](url)
		if (text[i] === '!' && text[i + 1] === '[') {
			const imgMatch = text.substring(i).match(/^!\[([^\]]*)\]\(([^)]*)\)/);
			if (imgMatch) {
				tokens.push({
					type: 'image',
					from: baseOffset + i,
					to: baseOffset + i + imgMatch[0].length,
					alt: imgMatch[1],
					url: imgMatch[2],
					content: imgMatch[0],
				});
				i += imgMatch[0].length;
				continue;
			}
		}

		// Link [text](url)
		if (text[i] === '[') {
			const linkMatch = text.substring(i).match(/^\[([^\]]*)\]\(([^)]*)\)/);
			if (linkMatch) {
				tokens.push({
					type: 'link',
					from: baseOffset + i,
					to: baseOffset + i + linkMatch[0].length,
					content: linkMatch[1],
					url: linkMatch[2],
					syntaxFrom: baseOffset + i,
					syntaxTo: baseOffset + i + 1,
					syntaxEndFrom: baseOffset + i + linkMatch[0].length - 1,
					syntaxEndTo: baseOffset + i + linkMatch[0].length,
				});
				i += linkMatch[0].length;
				continue;
			}
		}

		// HTML inline tags: <u>, <sub>, <sup>, <span style="...">
		if (text[i] === '<') {
			// Underline <u>text</u>
			const uMatch = text.substring(i).match(/^<u>(.*?)<\/u>/s);
			if (uMatch) {
				tokens.push({
					type: 'underline',
					from: baseOffset + i,
					to: baseOffset + i + uMatch[0].length,
					syntaxFrom: baseOffset + i,
					syntaxTo: baseOffset + i + 3,
					syntaxEndFrom: baseOffset + i + uMatch[0].length - 4,
					syntaxEndTo: baseOffset + i + uMatch[0].length,
					content: uMatch[1],
					children: parseInlines(uMatch[1], baseOffset + i + 3),
				});
				i += uMatch[0].length;
				continue;
			}

			// Subscript <sub>text</sub>
			const subMatch = text.substring(i).match(/^<sub>(.*?)<\/sub>/s);
			if (subMatch) {
				tokens.push({
					type: 'subscript',
					from: baseOffset + i,
					to: baseOffset + i + subMatch[0].length,
					syntaxFrom: baseOffset + i,
					syntaxTo: baseOffset + i + 5,
					syntaxEndFrom: baseOffset + i + subMatch[0].length - 6,
					syntaxEndTo: baseOffset + i + subMatch[0].length,
					content: subMatch[1],
				});
				i += subMatch[0].length;
				continue;
			}

			// Superscript <sup>text</sup>
			const supMatch = text.substring(i).match(/^<sup>(.*?)<\/sup>/s);
			if (supMatch) {
				tokens.push({
					type: 'superscript',
					from: baseOffset + i,
					to: baseOffset + i + supMatch[0].length,
					syntaxFrom: baseOffset + i,
					syntaxTo: baseOffset + i + 5,
					syntaxEndFrom: baseOffset + i + supMatch[0].length - 6,
					syntaxEndTo: baseOffset + i + supMatch[0].length,
					content: supMatch[1],
				});
				i += supMatch[0].length;
				continue;
			}

			// Font span <span style="font-family: ...">text</span>
			const fontMatch = text.substring(i).match(/^<span\s+style="font-family:\s*([^"]+)">(.*?)<\/span>/s);
			if (fontMatch) {
				tokens.push({
					type: 'fontSpan',
					from: baseOffset + i,
					to: baseOffset + i + fontMatch[0].length,
					fontFamily: fontMatch[1],
					content: fontMatch[2],
					children: parseInlines(fontMatch[2], baseOffset + i + fontMatch[0].indexOf('>') + 1),
				});
				i += fontMatch[0].length;
				continue;
			}

			// Color span <span style="color: ...">text</span>
			const colorMatch = text.substring(i).match(/^<span\s+style="color:\s*([^"]+)">(.*?)<\/span>/s);
			if (colorMatch) {
				tokens.push({
					type: 'colorSpan',
					from: baseOffset + i,
					to: baseOffset + i + colorMatch[0].length,
					color: colorMatch[1],
					content: colorMatch[2],
					children: parseInlines(colorMatch[2], baseOffset + i + colorMatch[0].indexOf('>') + 1),
				});
				i += colorMatch[0].length;
				continue;
			}

			// Font-size span <span style="font-size: ...">text</span>
			const sizeMatch = text.substring(i).match(/^<span\s+style="font-size:\s*([^"]+)">(.*?)<\/span>/s);
			if (sizeMatch) {
				tokens.push({
					type: 'fontSpan',
					from: baseOffset + i,
					to: baseOffset + i + sizeMatch[0].length,
					content: sizeMatch[2],
					children: parseInlines(sizeMatch[2], baseOffset + i + sizeMatch[0].indexOf('>') + 1),
				});
				i += sizeMatch[0].length;
				continue;
			}
		}

		// Plain text — advance one character
		const lastToken = tokens[tokens.length - 1];
		if (lastToken && lastToken.type === 'text') {
			lastToken.to = baseOffset + i + 1;
			lastToken.content = (lastToken.content ?? '') + text[i];
		} else {
			tokens.push({
				type: 'text',
				from: baseOffset + i,
				to: baseOffset + i + 1,
				content: text[i],
			});
		}
		i++;
	}

	return tokens;
}

interface DelimitedMatch {
	content: string;
	end: number;
	openLen: number;
	closeLen: number;
}

function matchDelimited(text: string, pos: number, open: string, close: string): DelimitedMatch | null {
	if (text.substring(pos, pos + open.length) !== open) return null;
	const searchStart = pos + open.length;
	// Don't match empty content
	if (text[searchStart] === close[0] && open.length === close.length) return null;

	let closeIdx = text.indexOf(close, searchStart);
	// For single-char delimiters, make sure we don't match the same char
	while (closeIdx >= 0) {
		// Make sure the close delimiter is not preceded by the same char (for ** vs *)
		if (open.length === 1 && closeIdx > 0 && text[closeIdx - 1] === open[0]) {
			closeIdx = text.indexOf(close, closeIdx + 1);
			continue;
		}
		break;
	}
	if (closeIdx < 0) return null;

	return {
		content: text.substring(searchStart, closeIdx),
		end: closeIdx + close.length,
		openLen: open.length,
		closeLen: close.length,
	};
}

function matchSingleDelimited(text: string, pos: number, open: string, close: string): DelimitedMatch | null {
	if (text[pos] !== open) return null;
	// Make sure it's not a double delimiter (e.g., ** for bold)
	if (text[pos + 1] === open) return null;

	const searchStart = pos + 1;
	let closeIdx = -1;
	for (let j = searchStart; j < text.length; j++) {
		if (text[j] === close && text[j - 1] !== '\\') {
			// Make sure it's not a double delimiter
			if (j + 1 < text.length && text[j + 1] === close) continue;
			closeIdx = j;
			break;
		}
	}
	if (closeIdx < 0) return null;

	return {
		content: text.substring(searchStart, closeIdx),
		end: closeIdx + 1,
		openLen: 1,
		closeLen: 1,
	};
}

/**
 * Custom Markdown extension for ==highlight== syntax.
 * Adds Highlight / HighlightMark nodes to the syntax tree,
 * matching the pattern used by livePreview.ts for decoration.
 */
import { type MarkdownExtension } from '@lezer/markdown';

export const Highlight: MarkdownExtension = {
	defineNodes: [
		{ name: 'Highlight', style: 'highlight' as any },
		{ name: 'HighlightMark', style: 'processingInstruction' as any },
	],
	parseInline: [
		{
			name: 'Highlight',
			parse(cx, next, pos) {
				// Look for ==
				if (next !== 61 /* '=' */ || cx.char(pos + 1) !== 61) return -1;
				// Don't match === or more
				if (cx.char(pos + 2) === 61) return -1;

				// Find closing ==
				let end = pos + 2;
				const max = cx.end;
				while (end + 1 < max) {
					if (cx.char(end) === 61 && cx.char(end + 1) === 61) {
						// Don't match ===
						if (end + 2 < max && cx.char(end + 2) === 61) {
							end++;
							continue;
						}
						// Found closing ==
						const content = cx.slice(pos + 2, end);
						if (content.length === 0) return -1; // empty highlight
						return cx.addElement(
							cx.elt('Highlight', pos, end + 2, [
								cx.elt('HighlightMark', pos, pos + 2),
								cx.elt('HighlightMark', end, end + 2),
							])
						);
					}
					end++;
				}
				return -1;
			},
		},
	],
};

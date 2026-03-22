/**
 * MarkdownParser — incremental block-level Markdown parser.
 *
 * On text change: re-parses only affected blocks and their neighbors.
 * Outputs an array of BlockTokens with inline tokens parsed lazily.
 */

import type { BlockToken, BlockType } from './tokens';
import { parseInlines } from './InlineParser';
import type { PieceTable } from '../core/PieceTable';

export class MarkdownParser {
	private blocks: BlockToken[] = [];
	private buffer: PieceTable;

	constructor(buffer: PieceTable) {
		this.buffer = buffer;
	}

	/** Full parse of the document. Used on initial load. */
	parse(): BlockToken[] {
		this.blocks = [];
		const lineCount = this.buffer.lineCount;
		let i = 0;

		while (i < lineCount) {
			const result = this.parseBlock(i);
			if (result) {
				this.blocks.push(result.block);
				i = result.nextLine;
			} else {
				i++;
			}
		}

		return this.blocks;
	}

	/** Incremental re-parse after a change in [fromLine, toLine]. */
	reparse(fromLine: number, toLine: number): BlockToken[] {
		// Find affected block range
		let startBlock = 0;
		let endBlock = this.blocks.length;

		for (let i = 0; i < this.blocks.length; i++) {
			const block = this.blocks[i];
			if (block.line + block.lineCount <= fromLine) {
				startBlock = i + 1;
			}
			if (block.line > toLine) {
				endBlock = i;
				break;
			}
		}

		// Expand to include neighbors for context
		startBlock = Math.max(0, startBlock - 1);
		endBlock = Math.min(this.blocks.length, endBlock + 1);

		// Determine line range to re-parse
		const reparseFromLine = startBlock < this.blocks.length
			? this.blocks[startBlock].line
			: 0;
		const reparseToLine = endBlock > 0 && endBlock <= this.blocks.length
			? this.blocks[endBlock - 1].line + this.blocks[endBlock - 1].lineCount
			: this.buffer.lineCount;

		// Parse the affected region
		const newBlocks: BlockToken[] = [];
		let line = reparseFromLine;
		while (line < this.buffer.lineCount && line < reparseToLine + 2) {
			const result = this.parseBlock(line);
			if (result) {
				newBlocks.push(result.block);
				line = result.nextLine;
			} else {
				line++;
			}
		}

		// Splice new blocks into the block array
		this.blocks.splice(startBlock, endBlock - startBlock, ...newBlocks);

		return this.blocks;
	}

	getBlocks(): BlockToken[] {
		return this.blocks;
	}

	private parseBlock(lineIdx: number): { block: BlockToken; nextLine: number } | null {
		if (lineIdx >= this.buffer.lineCount) return null;

		const lineText = this.buffer.getLine(lineIdx);
		const lineStart = this.buffer.getLineStart(lineIdx);

		// Blank line
		if (lineText.trim() === '') {
			return {
				block: {
					type: 'blankLine',
					from: lineStart,
					to: lineStart + lineText.length,
					line: lineIdx,
					lineCount: 1,
				},
				nextLine: lineIdx + 1,
			};
		}

		// Frontmatter (only at line 0)
		if (lineIdx === 0 && lineText === '---') {
			return this.parseFrontmatter(lineIdx);
		}

		// Heading
		const headingMatch = lineText.match(/^(#{1,6})\s(.*)$/);
		if (headingMatch) {
			const level = headingMatch[1].length;
			const contentStart = lineStart + headingMatch[1].length + 1;
			const contentText = headingMatch[2];
			return {
				block: {
					type: 'heading',
					from: lineStart,
					to: lineStart + lineText.length,
					line: lineIdx,
					lineCount: 1,
					level,
					inlines: parseInlines(contentText, contentStart),
				},
				nextLine: lineIdx + 1,
			};
		}

		// Horizontal rule
		if (/^(-{3,}|\*{3,}|_{3,})$/.test(lineText.trim())) {
			return {
				block: {
					type: 'horizontalRule',
					from: lineStart,
					to: lineStart + lineText.length,
					line: lineIdx,
					lineCount: 1,
				},
				nextLine: lineIdx + 1,
			};
		}

		// Code block
		if (lineText.match(/^```/)) {
			return this.parseCodeBlock(lineIdx);
		}

		// Math block
		if (lineText.trim() === '$$') {
			return this.parseMathBlock(lineIdx);
		}

		// Table
		if (lineText.includes('|') && lineIdx + 1 < this.buffer.lineCount) {
			const nextLine = this.buffer.getLine(lineIdx + 1);
			if (nextLine.match(/^\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/)) {
				return this.parseTable(lineIdx);
			}
		}

		// Blockquote / Callout
		if (lineText.startsWith('> ') || lineText === '>') {
			return this.parseBlockquote(lineIdx);
		}

		// Task list
		if (lineText.match(/^(\s*)- \[([ x])\] /)) {
			return this.parseList(lineIdx, 'task');
		}

		// Unordered list
		if (lineText.match(/^(\s*)[-*+] /)) {
			return this.parseList(lineIdx, 'bullet');
		}

		// Ordered list
		if (lineText.match(/^(\s*)\d+\. /)) {
			return this.parseList(lineIdx, 'ordered');
		}

		// Paragraph (default)
		return this.parseParagraph(lineIdx);
	}

	private parseFrontmatter(lineIdx: number): { block: BlockToken; nextLine: number } | null {
		const lineStart = this.buffer.getLineStart(lineIdx);
		let endLine = lineIdx + 1;
		while (endLine < this.buffer.lineCount) {
			if (this.buffer.getLine(endLine).trim() === '---') {
				const endOffset = this.buffer.getLineStart(endLine) + this.buffer.getLine(endLine).length;
				return {
					block: {
						type: 'frontmatter',
						from: lineStart,
						to: endOffset,
						line: lineIdx,
						lineCount: endLine - lineIdx + 1,
					},
					nextLine: endLine + 1,
				};
			}
			endLine++;
		}
		// No closing --- found, treat as paragraph
		return this.parseParagraph(lineIdx);
	}

	private parseCodeBlock(lineIdx: number): { block: BlockToken; nextLine: number } {
		const lineStart = this.buffer.getLineStart(lineIdx);
		const firstLine = this.buffer.getLine(lineIdx);
		const langMatch = firstLine.match(/^```(\w*)/);
		const language = langMatch?.[1] ?? '';

		let endLine = lineIdx + 1;
		while (endLine < this.buffer.lineCount) {
			if (this.buffer.getLine(endLine).trim() === '```') {
				const endOffset = this.buffer.getLineStart(endLine) + this.buffer.getLine(endLine).length;
				return {
					block: {
						type: 'codeBlock',
						from: lineStart,
						to: endOffset,
						line: lineIdx,
						lineCount: endLine - lineIdx + 1,
						language,
					},
					nextLine: endLine + 1,
				};
			}
			endLine++;
		}

		// Unclosed code block extends to end
		const endOffset = this.buffer.length;
		return {
			block: {
				type: 'codeBlock',
				from: lineStart,
				to: endOffset,
				line: lineIdx,
				lineCount: endLine - lineIdx,
				language,
			},
			nextLine: endLine,
		};
	}

	private parseMathBlock(lineIdx: number): { block: BlockToken; nextLine: number } {
		const lineStart = this.buffer.getLineStart(lineIdx);
		let endLine = lineIdx + 1;
		while (endLine < this.buffer.lineCount) {
			if (this.buffer.getLine(endLine).trim() === '$$') {
				const endOffset = this.buffer.getLineStart(endLine) + this.buffer.getLine(endLine).length;
				return {
					block: {
						type: 'mathBlock',
						from: lineStart,
						to: endOffset,
						line: lineIdx,
						lineCount: endLine - lineIdx + 1,
					},
					nextLine: endLine + 1,
				};
			}
			endLine++;
		}

		const endOffset = this.buffer.length;
		return {
			block: {
				type: 'mathBlock',
				from: lineStart,
				to: endOffset,
				line: lineIdx,
				lineCount: endLine - lineIdx,
			},
			nextLine: endLine,
		};
	}

	private parseTable(lineIdx: number): { block: BlockToken; nextLine: number } {
		const lineStart = this.buffer.getLineStart(lineIdx);
		const children: BlockToken[] = [];
		let endLine = lineIdx;

		while (endLine < this.buffer.lineCount) {
			const text = this.buffer.getLine(endLine);
			if (!text.includes('|')) break;

			const rowStart = this.buffer.getLineStart(endLine);
			children.push({
				type: 'tableRow',
				from: rowStart,
				to: rowStart + text.length,
				line: endLine,
				lineCount: 1,
				inlines: parseInlines(text, rowStart),
			});
			endLine++;
		}

		const endOffset = endLine > lineIdx
			? this.buffer.getLineStart(endLine - 1) + this.buffer.getLine(endLine - 1).length
			: lineStart;

		return {
			block: {
				type: 'table',
				from: lineStart,
				to: endOffset,
				line: lineIdx,
				lineCount: endLine - lineIdx,
				children,
			},
			nextLine: endLine,
		};
	}

	private parseBlockquote(lineIdx: number): { block: BlockToken; nextLine: number } {
		const lineStart = this.buffer.getLineStart(lineIdx);
		const firstLine = this.buffer.getLine(lineIdx);
		let endLine = lineIdx + 1;

		// Check if this is a callout
		const calloutMatch = firstLine.match(/^>\s*\[!(\w+)\]\s*(.*)?$/);

		while (endLine < this.buffer.lineCount) {
			const text = this.buffer.getLine(endLine);
			if (!text.startsWith('> ') && text !== '>') break;
			endLine++;
		}

		const endOffset = this.buffer.getLineStart(endLine - 1) + this.buffer.getLine(endLine - 1).length;

		if (calloutMatch) {
			return {
				block: {
					type: 'callout',
					from: lineStart,
					to: endOffset,
					line: lineIdx,
					lineCount: endLine - lineIdx,
					calloutType: calloutMatch[1],
					calloutTitle: calloutMatch[2]?.trim() || calloutMatch[1],
				},
				nextLine: endLine,
			};
		}

		// Collect blockquote content for inline parsing
		const contentLines: string[] = [];
		for (let l = lineIdx; l < endLine; l++) {
			const text = this.buffer.getLine(l);
			contentLines.push(text.startsWith('> ') ? text.substring(2) : text.substring(1));
		}

		return {
			block: {
				type: 'blockquote',
				from: lineStart,
				to: endOffset,
				line: lineIdx,
				lineCount: endLine - lineIdx,
				inlines: parseInlines(contentLines.join('\n'), lineStart + 2),
			},
			nextLine: endLine,
		};
	}

	private parseList(lineIdx: number, listType: 'bullet' | 'ordered' | 'task'): { block: BlockToken; nextLine: number } {
		const lineStart = this.buffer.getLineStart(lineIdx);
		const children: BlockToken[] = [];
		let endLine = lineIdx;

		while (endLine < this.buffer.lineCount) {
			const text = this.buffer.getLine(endLine);
			const isListItem = listType === 'task'
				? /^(\s*)- \[([ x])\] /.test(text)
				: listType === 'ordered'
					? /^(\s*)\d+\. /.test(text)
					: /^(\s*)[-*+] /.test(text);

			// Continue if it's a list item or a continuation line (indented)
			if (!isListItem && !(text.startsWith('  ') || text.startsWith('\t'))) break;
			if (!isListItem && text.trim() === '') break;

			if (isListItem) {
				const itemStart = this.buffer.getLineStart(endLine);
				let contentText: string;
				let checked: boolean | undefined;

				if (listType === 'task') {
					const m = text.match(/^(\s*)- \[([ x])\] (.*)/);
					contentText = m?.[3] ?? '';
					checked = m?.[2] === 'x';
				} else if (listType === 'ordered') {
					const m = text.match(/^(\s*)\d+\. (.*)/);
					contentText = m?.[2] ?? '';
				} else {
					const m = text.match(/^(\s*)[-*+] (.*)/);
					contentText = m?.[2] ?? '';
				}

				const prefixLen = text.length - contentText.length;
				children.push({
					type: listType === 'task' ? 'taskItem' : 'listItem',
					from: itemStart,
					to: itemStart + text.length,
					line: endLine,
					lineCount: 1,
					checked,
					inlines: parseInlines(contentText, itemStart + prefixLen),
				});
			}
			endLine++;
		}

		const endOffset = endLine > lineIdx
			? this.buffer.getLineStart(endLine - 1) + this.buffer.getLine(endLine - 1).length
			: lineStart;

		return {
			block: {
				type: listType === 'task' ? 'taskList' : 'list',
				from: lineStart,
				to: endOffset,
				line: lineIdx,
				lineCount: endLine - lineIdx,
				ordered: listType === 'ordered',
				children,
			},
			nextLine: endLine,
		};
	}

	private parseParagraph(lineIdx: number): { block: BlockToken; nextLine: number } {
		const lineStart = this.buffer.getLineStart(lineIdx);
		let endLine = lineIdx + 1;

		// A paragraph continues until a blank line or a block-level element
		while (endLine < this.buffer.lineCount) {
			const text = this.buffer.getLine(endLine);
			if (text.trim() === '') break;
			if (/^#{1,6}\s/.test(text)) break;
			if (/^(-{3,}|\*{3,}|_{3,})$/.test(text.trim())) break;
			if (/^```/.test(text)) break;
			if (/^>\s/.test(text) || text === '>') break;
			if (/^(\s*)[-*+] /.test(text)) break;
			if (/^(\s*)\d+\. /.test(text)) break;
			if (/^\$\$$/.test(text.trim())) break;
			if (text.includes('|') && endLine + 1 < this.buffer.lineCount) {
				const next = this.buffer.getLine(endLine + 1);
				if (next.match(/^\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/)) break;
			}
			endLine++;
		}

		const endOffset = this.buffer.getLineStart(endLine - 1) + this.buffer.getLine(endLine - 1).length;
		const fullText = this.buffer.getText(lineStart, endOffset - lineStart);

		return {
			block: {
				type: 'paragraph',
				from: lineStart,
				to: endOffset,
				line: lineIdx,
				lineCount: endLine - lineIdx,
				inlines: parseInlines(fullText, lineStart),
			},
			nextLine: endLine,
		};
	}
}

/**
 * ViewportRenderer — virtual scrolling and line rendering for the Constellation Editor.
 *
 * Only renders lines visible in the viewport + a small buffer.
 * Manages DOM element creation/recycling for minimal layout thrashing.
 */

import type { PieceTable } from '../core/PieceTable';
import type { EditorSelection } from '../core/Selection';
import type { BlockToken, InlineToken } from '../parser/tokens';
import type { Decoration } from './DecorationEngine';
import { LineLayout } from './LineLayout';

export interface RenderLine {
	line: number;
	text: string;
	element: HTMLElement | null;
	block: BlockToken | null;
}

export class ViewportRenderer {
	private container: HTMLElement;
	private scroller: HTMLElement;
	private content: HTMLElement;
	private buffer: PieceTable;
	private layout: LineLayout;
	private renderedLines: Map<number, HTMLElement> = new Map();
	private visibleRange = { start: 0, end: 0 };
	private blocks: BlockToken[] = [];
	private decorations: Decoration[] = [];
	private cursorEl: HTMLElement | null = null;
	private selectionEls: HTMLElement[] = [];
	private onLineClick: ((offset: number) => void) | null = null;
	private onCheckboxToggle: ((line: number) => void) | null = null;
	private lineToBlock: Map<number, BlockToken> = new Map();

	constructor(container: HTMLElement, buffer: PieceTable) {
		this.container = container;
		this.buffer = buffer;
		this.layout = new LineLayout(buffer.lineCount);

		// Create scroll container
		this.scroller = document.createElement('div');
		this.scroller.className = 'ce-scroller';
		this.scroller.style.cssText = 'position: relative; overflow-y: auto; height: 100%;';

		// Create content spacer (for total scrollable height)
		this.content = document.createElement('div');
		this.content.className = 'ce-content';
		this.content.style.cssText = 'position: relative; width: 100%;';

		this.scroller.appendChild(this.content);
		this.container.appendChild(this.scroller);

		this.scroller.addEventListener('scroll', this.onScroll.bind(this), { passive: true });
	}

	setBlocks(blocks: BlockToken[]): void {
		this.blocks = blocks;
		this.lineToBlock.clear();
		for (const block of blocks) {
			for (let l = block.line; l < block.line + block.lineCount; l++) {
				this.lineToBlock.set(l, block);
			}
		}
	}

	setDecorations(decorations: Decoration[]): void {
		this.decorations = decorations;
	}

	setLineClickHandler(handler: (offset: number) => void): void {
		this.onLineClick = handler;
	}

	setCheckboxToggleHandler(handler: (line: number) => void): void {
		this.onCheckboxToggle = handler;
	}

	getLayout(): LineLayout {
		return this.layout;
	}

	getScroller(): HTMLElement {
		return this.scroller;
	}

	/** Full render of visible lines. */
	render(): void {
		this.layout.updateLineCount(this.buffer.lineCount);
		this.content.style.height = this.layout.getTotalHeight() + 'px';

		const scrollTop = this.scroller.scrollTop;
		const viewportHeight = this.scroller.clientHeight;
		const range = this.layout.getVisibleRange(scrollTop, viewportHeight);

		this.updateVisibleRange(range.start, range.end);
	}

	/** Incremental render after edit — only update changed lines. */
	renderChanged(fromLine: number, toLine: number): void {
		this.layout.updateLineCount(this.buffer.lineCount);
		this.content.style.height = this.layout.getTotalHeight() + 'px';

		// Remove stale rendered lines
		for (const [line, el] of this.renderedLines) {
			if (line >= fromLine && line <= toLine) {
				el.remove();
				this.renderedLines.delete(line);
			}
		}

		// Re-render visible range
		this.render();
	}

	/** Update cursor position in the DOM. */
	renderCursor(selection: EditorSelection): void {
		// Remove old cursor
		if (this.cursorEl) {
			this.cursorEl.remove();
			this.cursorEl = null;
		}
		// Remove old selection highlights
		for (const el of this.selectionEls) el.remove();
		this.selectionEls = [];

		// Render cursor caret
		const pos = this.buffer.getPosition(selection.head);
		const lineEl = this.renderedLines.get(pos.line);
		if (!lineEl) return;

		this.cursorEl = document.createElement('div');
		this.cursorEl.className = 'ce-cursor';
		// Position cursor using character offset estimation
		const charWidth = this.estimateCharWidth(lineEl);
		const left = pos.column * charWidth;
		const top = this.layout.getLineTop(pos.line);
		this.cursorEl.style.cssText = `position: absolute; left: ${left}px; top: ${top}px; width: 2px; height: ${this.layout.getLineHeight(pos.line)}px;`;
		this.content.appendChild(this.cursorEl);

		// Render selection highlight
		if (!selection.isCollapsed) {
			const fromPos = this.buffer.getPosition(selection.from);
			const toPos = this.buffer.getPosition(selection.to);

			for (let line = fromPos.line; line <= toPos.line; line++) {
				const lineHeight = this.layout.getLineHeight(line);
				const lineTop = this.layout.getLineTop(line);
				const lineEl = this.renderedLines.get(line);
				if (!lineEl) continue;
				const cw = this.estimateCharWidth(lineEl);

				const startCol = line === fromPos.line ? fromPos.column : 0;
				const lineText = this.buffer.getLine(line);
				const endCol = line === toPos.line ? toPos.column : lineText.length;

				const selEl = document.createElement('div');
				selEl.className = 'ce-selection';
				selEl.style.cssText = `position: absolute; left: ${startCol * cw}px; top: ${lineTop}px; width: ${(endCol - startCol) * cw}px; height: ${lineHeight}px;`;
				this.content.appendChild(selEl);
				this.selectionEls.push(selEl);
			}
		}
	}

	/** Scroll to make a line visible. */
	scrollToLine(line: number): void {
		const top = this.layout.getLineTop(line);
		const height = this.layout.getLineHeight(line);
		const scrollTop = this.scroller.scrollTop;
		const viewportHeight = this.scroller.clientHeight;

		if (top < scrollTop) {
			this.scroller.scrollTop = top;
		} else if (top + height > scrollTop + viewportHeight) {
			this.scroller.scrollTop = top + height - viewportHeight;
		}
	}

	/** Scroll to make a cursor offset visible. */
	scrollToOffset(offset: number): void {
		const line = this.buffer.getLineFromOffset(offset);
		this.scrollToLine(line);
	}

	get scrollTop(): number {
		return this.scroller.scrollTop;
	}

	set scrollTop(value: number) {
		this.scroller.scrollTop = value;
	}

	destroy(): void {
		this.scroller.remove();
		this.renderedLines.clear();
	}

	// ── Private ──

	private onScroll(): void {
		const scrollTop = this.scroller.scrollTop;
		const viewportHeight = this.scroller.clientHeight;
		const range = this.layout.getVisibleRange(scrollTop, viewportHeight);
		this.updateVisibleRange(range.start, range.end);
	}

	private updateVisibleRange(start: number, end: number): void {
		// Remove lines outside the new range
		for (const [line, el] of this.renderedLines) {
			if (line < start || line >= end) {
				el.remove();
				this.renderedLines.delete(line);
			}
		}

		// Add lines in the new range that aren't rendered
		for (let i = start; i < end && i < this.buffer.lineCount; i++) {
			if (!this.renderedLines.has(i)) {
				const el = this.renderLine(i);
				if (el) {
					this.content.appendChild(el);
					this.renderedLines.set(i, el);

					// Measure actual height
					const measuredHeight = el.getBoundingClientRect().height;
					if (measuredHeight > 0) {
						this.layout.setLineHeight(i, measuredHeight);
					}
				}
			}
		}

		this.visibleRange = { start, end };
	}

	private renderLine(lineIdx: number): HTMLElement | null {
		if (lineIdx >= this.buffer.lineCount) return null;

		const lineText = this.buffer.getLine(lineIdx);
		const lineStart = this.buffer.getLineStart(lineIdx);
		const block = this.lineToBlock.get(lineIdx);

		const el = document.createElement('div');
		el.className = 'ce-line';
		el.dataset.line = String(lineIdx);

		// Position the line
		const top = this.layout.getLineTop(lineIdx);
		el.style.cssText = `position: absolute; left: 0; right: 0; top: ${top}px; min-height: 24px;`;

		// Apply line decorations
		const lineDecos = this.decorations.filter(d =>
			d.type === 'line' && d.from <= lineStart + lineText.length && d.to >= lineStart
		);
		for (const deco of lineDecos) {
			if (deco.lineClassName) el.classList.add(...deco.lineClassName.split(' '));
			if (deco.lineStyle) el.style.cssText += '; ' + deco.lineStyle;
		}

		// Check for widget decorations that replace the entire line
		const widgetDecos = this.decorations.filter(d =>
			d.type === 'widget' && d.from <= lineStart && d.to >= lineStart + lineText.length
		);
		for (const wd of widgetDecos) {
			if (wd.widgetType === 'image' && wd.widgetData) {
				const img = document.createElement('img');
				img.src = wd.widgetData.src;
				img.alt = wd.widgetData.alt ?? '';
				img.className = 'ce-image';
				img.style.maxWidth = '100%';
				el.appendChild(img);
				return el;
			}
			if (wd.widgetType === 'table') {
				el.appendChild(this.renderTableWidget(wd));
				return el;
			}
		}

		// Check for checkbox widgets on this line
		const checkboxDecos = this.decorations.filter(d =>
			d.type === 'widget' && d.widgetType === 'checkbox' && d.widgetData?.line === lineIdx
		);
		if (checkboxDecos.length > 0) {
			const checkbox = document.createElement('input');
			checkbox.type = 'checkbox';
			checkbox.className = 'ce-checkbox';
			checkbox.checked = checkboxDecos[0].widgetData?.checked ?? false;
			checkbox.addEventListener('change', () => {
				if (this.onCheckboxToggle) this.onCheckboxToggle(lineIdx);
			});
			el.appendChild(checkbox);

			// Render text after checkbox prefix
			const taskMatch = lineText.match(/^(\s*)- \[([ x])\] (.*)/);
			if (taskMatch) {
				const span = document.createElement('span');
				this.renderInlineContent(span, taskMatch[3], lineStart + taskMatch[0].length - taskMatch[3].length);
				if (checkboxDecos[0].widgetData?.checked) {
					span.classList.add('ce-task-done');
				}
				el.appendChild(span);
				return el;
			}
		}

		// Render text content with inline decorations
		this.renderInlineContent(el, lineText, lineStart);

		// Add click handler for cursor positioning
		el.addEventListener('mousedown', (e) => {
			if (this.onLineClick) {
				const rect = el.getBoundingClientRect();
				const x = e.clientX - rect.left;
				const charWidth = this.estimateCharWidth(el);
				const col = Math.round(x / charWidth);
				const offset = lineStart + Math.min(col, lineText.length);
				this.onLineClick(offset);
			}
		});

		return el;
	}

	private renderInlineContent(container: HTMLElement, text: string, baseOffset: number): void {
		// Collect inline decorations for this range
		const inlineDecos = this.decorations.filter(d =>
			(d.type === 'style' || d.type === 'hide') &&
			d.from < baseOffset + text.length && d.to > baseOffset
		);

		if (inlineDecos.length === 0) {
			if (text === '') {
				// Empty line — add zero-width space for height
				container.appendChild(document.createTextNode('\u200B'));
			} else {
				container.appendChild(document.createTextNode(text));
			}
			return;
		}

		// Build segments based on decorations
		const hideRanges: Array<[number, number]> = [];
		for (const d of inlineDecos) {
			if (d.type === 'hide') {
				if (d.hideFrom !== undefined && d.hideTo !== undefined) {
					hideRanges.push([d.hideFrom, d.hideTo]);
				}
				if (d.hideEndFrom !== undefined && d.hideEndTo !== undefined) {
					hideRanges.push([d.hideEndFrom, d.hideEndTo]);
				}
			}
		}

		// Sort decorations by position
		const styleDecos = inlineDecos
			.filter(d => d.type === 'style')
			.sort((a, b) => a.from - b.from);

		let pos = baseOffset;
		const end = baseOffset + text.length;

		for (let charIdx = 0; charIdx < text.length; charIdx++) {
			const globalPos = baseOffset + charIdx;

			// Check if this character is hidden
			const isHidden = hideRanges.some(([from, to]) => globalPos >= from && globalPos < to);
			if (isHidden) {
				// Create hidden span
				const hiddenSpan = document.createElement('span');
				hiddenSpan.className = 'ce-syntax-hidden';
				hiddenSpan.textContent = text[charIdx];
				container.appendChild(hiddenSpan);
				continue;
			}

			// Find applicable style decorations
			const activeStyles = styleDecos.filter(d => globalPos >= d.from && globalPos < d.to);

			if (activeStyles.length > 0) {
				// Check if next char has same styles — batch them
				let batchEnd = charIdx + 1;
				while (batchEnd < text.length) {
					const nextPos = baseOffset + batchEnd;
					const nextHidden = hideRanges.some(([from, to]) => nextPos >= from && nextPos < to);
					if (nextHidden) break;
					const nextStyles = styleDecos.filter(d => nextPos >= d.from && nextPos < d.to);
					if (nextStyles.length !== activeStyles.length || !nextStyles.every((s, i) => s === activeStyles[i])) break;
					batchEnd++;
				}

				const batchText = text.substring(charIdx, batchEnd);
				let el: HTMLElement = document.createElement('span');

				// Apply styles from innermost to outermost
				for (const style of activeStyles) {
					if (style.tag) {
						const tagEl = document.createElement(style.tag);
						tagEl.appendChild(el);
						el = tagEl;
					}
					if (style.className) el.classList.add(...style.className.split(' '));
					if (style.style) el.style.cssText += style.style;
				}

				// Set text on innermost element
				const textNode = document.createTextNode(batchText);
				const innermost = el.querySelector('span') || el;
				if (innermost.childNodes.length === 0) {
					innermost.appendChild(textNode);
				} else {
					innermost.firstChild!.textContent = batchText;
				}

				container.appendChild(el);
				charIdx = batchEnd - 1;
			} else {
				// Plain text — batch consecutive plain chars
				let batchEnd = charIdx + 1;
				while (batchEnd < text.length) {
					const nextPos = baseOffset + batchEnd;
					const nextHidden = hideRanges.some(([from, to]) => nextPos >= from && nextPos < to);
					if (nextHidden) break;
					const nextStyles = styleDecos.filter(d => nextPos >= d.from && nextPos < d.to);
					if (nextStyles.length > 0) break;
					batchEnd++;
				}
				container.appendChild(document.createTextNode(text.substring(charIdx, batchEnd)));
				charIdx = batchEnd - 1;
			}
		}
	}

	private renderTableWidget(deco: Decoration): HTMLElement {
		const table = document.createElement('table');
		table.className = 'ce-table';
		const children = deco.widgetData?.children as BlockToken[] | undefined;
		if (!children) return table;

		let isFirst = true;
		let isSecond = true;
		for (const row of children) {
			if (isSecond && !isFirst) { isSecond = false; continue; } // Skip separator row
			const tr = document.createElement('tr');
			const lineText = this.buffer.getText(row.from, row.to - row.from);
			const cells = lineText.split('|').filter(c => c.trim() !== '' || c.length > 0).slice(1);
			// Remove last empty cell if the line ends with |
			if (cells.length > 0 && cells[cells.length - 1].trim() === '') cells.pop();

			for (const cell of cells) {
				const td = document.createElement(isFirst ? 'th' : 'td');
				td.textContent = cell.trim();
				tr.appendChild(td);
			}
			table.appendChild(tr);
			isFirst = false;
		}

		return table;
	}

	private estimateCharWidth(lineEl: HTMLElement): number {
		// Use a cached measurement or estimate
		const fontSize = parseFloat(getComputedStyle(lineEl).fontSize) || 16;
		return fontSize * 0.6; // Approximate for monospace/proportional
	}
}

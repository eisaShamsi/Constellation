/**
 * EditEngine — command-based editing for the Constellation Editor.
 *
 * Every edit flows through commands that produce TextChanges,
 * enabling undo/redo and consistent state updates.
 */

import { PieceTable, type TextChange } from './PieceTable';
import { EditorSelection } from './Selection';
import { History } from './History';

export type ChangeListener = (changes: TextChange[], selection: EditorSelection) => void;

export class EditEngine {
	buffer: PieceTable;
	selection: EditorSelection;
	history: History;
	private listeners: ChangeListener[] = [];

	constructor(initialText: string = '') {
		this.buffer = new PieceTable(initialText);
		this.selection = EditorSelection.cursor(0);
		this.history = new History();
	}

	/** Subscribe to changes. */
	onChange(listener: ChangeListener): () => void {
		this.listeners.push(listener);
		return () => {
			const idx = this.listeners.indexOf(listener);
			if (idx >= 0) this.listeners.splice(idx, 1);
		};
	}

	private notify(changes: TextChange[]): void {
		for (const listener of this.listeners) {
			listener(changes, this.selection);
		}
	}

	/** Get the full document text. */
	getText(): string {
		return this.buffer.getFullText();
	}

	/** Get document length. */
	get length(): number {
		return this.buffer.length;
	}

	/** Get line count. */
	get lineCount(): number {
		return this.buffer.lineCount;
	}

	/** Set the full document (resets history). */
	setContent(text: string): void {
		this.buffer = new PieceTable(text);
		this.selection = EditorSelection.cursor(0);
		this.history.clear();
		this.notify([{ offset: 0, oldLength: 0, newLength: text.length, oldText: '', newText: text }]);
	}

	// ── Basic editing commands ──

	/** Insert text at the current cursor position. */
	insertText(text: string): TextChange {
		const selBefore = this.selection.toJSON();
		let change: TextChange;

		if (!this.selection.isCollapsed) {
			// Replace selection
			change = this.buffer.replace(this.selection.from, this.selection.length, text);
		} else {
			change = this.buffer.insert(this.selection.head, text);
		}

		this.selection = EditorSelection.cursor(this.selection.from + text.length);
		this.history.push(change, selBefore, this.selection.toJSON());
		this.notify([change]);
		return change;
	}

	/** Delete the selection, or deleteCount chars forward/backward. */
	deleteText(direction: 'forward' | 'backward', count: number = 1): TextChange | null {
		const selBefore = this.selection.toJSON();
		let change: TextChange;

		if (!this.selection.isCollapsed) {
			change = this.buffer.delete(this.selection.from, this.selection.length);
			this.selection = EditorSelection.cursor(this.selection.from);
		} else if (direction === 'backward') {
			const deleteFrom = Math.max(0, this.selection.head - count);
			const deleteLen = this.selection.head - deleteFrom;
			if (deleteLen === 0) return null;
			change = this.buffer.delete(deleteFrom, deleteLen);
			this.selection = EditorSelection.cursor(deleteFrom);
		} else {
			const deleteLen = Math.min(count, this.buffer.length - this.selection.head);
			if (deleteLen === 0) return null;
			change = this.buffer.delete(this.selection.head, deleteLen);
			// Cursor stays in place
		}

		this.history.push(change, selBefore, this.selection.toJSON());
		this.notify([change]);
		return change;
	}

	/** Delete an entire word backward (Ctrl+Backspace). */
	deleteWordBackward(): TextChange | null {
		if (!this.selection.isCollapsed) return this.deleteText('backward');

		const head = this.selection.head;
		const lineStart = this.buffer.getLineStart(this.buffer.getLineFromOffset(head));
		const textBeforeCursor = this.buffer.getText(lineStart, head - lineStart);

		// Find word boundary: skip whitespace then skip word chars
		let i = textBeforeCursor.length;
		while (i > 0 && /\s/.test(textBeforeCursor[i - 1])) i--;
		while (i > 0 && /\S/.test(textBeforeCursor[i - 1])) i--;

		const deleteFrom = lineStart + i;
		return this.deleteText('backward', head - deleteFrom);
	}

	/** Delete an entire word forward (Ctrl+Delete). */
	deleteWordForward(): TextChange | null {
		if (!this.selection.isCollapsed) return this.deleteText('forward');

		const head = this.selection.head;
		const line = this.buffer.getLineFromOffset(head);
		const lineEnd = this.buffer.getLineEnd(line);
		const textAfterCursor = this.buffer.getText(head, lineEnd - head);

		let i = 0;
		while (i < textAfterCursor.length && /\S/.test(textAfterCursor[i])) i++;
		while (i < textAfterCursor.length && /\s/.test(textAfterCursor[i])) i++;

		if (i === 0) i = 1; // At least delete one char (the newline)
		return this.deleteText('forward', i);
	}

	// ── Markdown formatting commands ──

	/** Toggle inline mark (bold, italic, etc.) around selection. */
	toggleMark(syntax: string): TextChange[] {
		this.history.pushBoundary();
		const { from, to } = this.selection;
		const selectedText = this.buffer.getText(from, to - from);
		const changes: TextChange[] = [];
		const selBefore = this.selection.toJSON();

		// Check if already wrapped
		const beforeSyntax = this.buffer.getText(Math.max(0, from - syntax.length), syntax.length);
		const afterSyntax = this.buffer.getText(to, syntax.length);

		if (beforeSyntax === syntax && afterSyntax === syntax) {
			// Remove marks
			const c1 = this.buffer.delete(to, syntax.length);
			changes.push(c1);
			const c2 = this.buffer.delete(from - syntax.length, syntax.length);
			changes.push(c2);
			this.selection = EditorSelection.range(from - syntax.length, to - syntax.length);
		} else {
			// Add marks
			const c1 = this.buffer.insert(to, syntax);
			changes.push(c1);
			const c2 = this.buffer.insert(from, syntax);
			changes.push(c2);
			this.selection = EditorSelection.range(from + syntax.length, to + syntax.length);
		}

		const combinedChange: TextChange = {
			offset: Math.min(...changes.map(c => c.offset)),
			oldLength: changes.reduce((sum, c) => sum + c.oldLength, 0),
			newLength: changes.reduce((sum, c) => sum + c.newLength, 0),
			oldText: changes.map(c => c.oldText).join(''),
			newText: changes.map(c => c.newText).join(''),
		};
		this.history.push(combinedChange, selBefore, this.selection.toJSON());
		this.notify(changes);
		return changes;
	}

	/** Set the current line to a heading level (0 = paragraph). */
	setHeading(level: number): TextChange[] {
		this.history.pushBoundary();
		const line = this.buffer.getLineFromOffset(this.selection.head);
		const lineStart = this.buffer.getLineStart(line);
		const lineText = this.buffer.getLine(line);
		const selBefore = this.selection.toJSON();

		// Remove existing heading prefix
		const headingMatch = lineText.match(/^(#{1,6})\s/);
		const changes: TextChange[] = [];

		if (headingMatch) {
			const removeLen = headingMatch[0].length;
			changes.push(this.buffer.delete(lineStart, removeLen));
		}

		// Add new heading prefix
		if (level > 0 && level <= 6) {
			const prefix = '#'.repeat(level) + ' ';
			const insertOffset = lineStart - (headingMatch ? headingMatch[0].length : 0) + (changes.length > 0 ? 0 : 0);
			// After deletion, lineStart is adjusted
			const actualLineStart = headingMatch ? lineStart : lineStart;
			changes.push(this.buffer.insert(actualLineStart, prefix));
		}

		// Keep cursor position roughly the same
		this.selection = this.selection.clamp(this.buffer.length);
		const combinedChange: TextChange = {
			offset: lineStart,
			oldLength: changes.reduce((sum, c) => sum + c.oldLength, 0),
			newLength: changes.reduce((sum, c) => sum + c.newLength, 0),
			oldText: changes.map(c => c.oldText).join(''),
			newText: changes.map(c => c.newText).join(''),
		};
		this.history.push(combinedChange, selBefore, this.selection.toJSON());
		this.notify(changes);
		return changes;
	}

	/** Toggle a list type on the current line. */
	toggleList(type: 'bullet' | 'ordered' | 'task'): TextChange[] {
		this.history.pushBoundary();
		const line = this.buffer.getLineFromOffset(this.selection.head);
		const lineStart = this.buffer.getLineStart(line);
		const lineText = this.buffer.getLine(line);
		const selBefore = this.selection.toJSON();
		const changes: TextChange[] = [];

		const bulletMatch = lineText.match(/^(\s*)- /);
		const orderedMatch = lineText.match(/^(\s*)\d+\. /);
		const taskMatch = lineText.match(/^(\s*)- \[([ x])\] /);

		const indent = bulletMatch?.[1] ?? orderedMatch?.[1] ?? taskMatch?.[1] ?? '';
		let existingPrefix = '';
		let existingLen = 0;

		if (taskMatch) {
			existingPrefix = 'task';
			existingLen = taskMatch[0].length;
		} else if (bulletMatch) {
			existingPrefix = 'bullet';
			existingLen = bulletMatch[0].length;
		} else if (orderedMatch) {
			existingPrefix = 'ordered';
			existingLen = orderedMatch[0].length;
		}

		if (existingLen > 0) {
			changes.push(this.buffer.delete(lineStart, existingLen));
		}

		if (existingPrefix !== type) {
			let prefix: string;
			switch (type) {
				case 'bullet': prefix = indent + '- '; break;
				case 'ordered': prefix = indent + '1. '; break;
				case 'task': prefix = indent + '- [ ] '; break;
			}
			changes.push(this.buffer.insert(lineStart, prefix));
		}

		this.selection = this.selection.clamp(this.buffer.length);
		const combinedChange: TextChange = {
			offset: lineStart,
			oldLength: changes.reduce((sum, c) => sum + c.oldLength, 0),
			newLength: changes.reduce((sum, c) => sum + c.newLength, 0),
			oldText: changes.map(c => c.oldText).join(''),
			newText: changes.map(c => c.newText).join(''),
		};
		this.history.push(combinedChange, selBefore, this.selection.toJSON());
		this.notify(changes);
		return changes;
	}

	/** Insert a link at cursor or wrap selection. */
	insertLink(url: string, text?: string): TextChange {
		this.history.pushBoundary();
		const selBefore = this.selection.toJSON();
		const linkText = text ?? (this.selection.isCollapsed ? url : this.buffer.getText(this.selection.from, this.selection.length));
		const markdown = `[${linkText}](${url})`;

		let change: TextChange;
		if (!this.selection.isCollapsed) {
			change = this.buffer.replace(this.selection.from, this.selection.length, markdown);
		} else {
			change = this.buffer.insert(this.selection.head, markdown);
		}

		this.selection = EditorSelection.cursor(this.selection.from + markdown.length);
		this.history.push(change, selBefore, this.selection.toJSON());
		this.notify([change]);
		return change;
	}

	/** Insert an image. */
	insertImage(src: string, alt: string = ''): TextChange {
		return this.insertText(`![${alt}](${src})`);
	}

	/** Insert a code block. */
	insertCodeBlock(language: string = ''): TextChange {
		this.history.pushBoundary();
		const code = this.selection.isCollapsed ? '' : this.buffer.getText(this.selection.from, this.selection.length);
		const block = `\n\`\`\`${language}\n${code}\n\`\`\`\n`;
		const selBefore = this.selection.toJSON();

		let change: TextChange;
		if (!this.selection.isCollapsed) {
			change = this.buffer.replace(this.selection.from, this.selection.length, block);
		} else {
			change = this.buffer.insert(this.selection.head, block);
		}

		// Place cursor inside the code block
		const cursorOffset = this.selection.from + 4 + language.length + 1;
		this.selection = EditorSelection.cursor(cursorOffset);
		this.history.push(change, selBefore, this.selection.toJSON());
		this.notify([change]);
		return change;
	}

	/** Insert a table. */
	insertTable(rows: number, cols: number): TextChange {
		this.history.pushBoundary();
		const header = '| ' + Array.from({ length: cols }, (_, i) => `Column ${i + 1}`).join(' | ') + ' |';
		const separator = '| ' + Array.from({ length: cols }, () => '---').join(' | ') + ' |';
		const row = '| ' + Array.from({ length: cols }, () => '   ').join(' | ') + ' |';
		const table = '\n' + header + '\n' + separator + '\n' + Array.from({ length: rows - 1 }, () => row).join('\n') + '\n';

		return this.insertText(table);
	}

	/** Insert a callout. */
	insertCallout(type: string = 'info', title: string = ''): TextChange {
		this.history.pushBoundary();
		const callout = `\n> [!${type}]${title ? ' ' + title : ''}\n> \n`;
		return this.insertText(callout);
	}

	/** Insert a horizontal rule. */
	insertHorizontalRule(): TextChange {
		this.history.pushBoundary();
		return this.insertText('\n---\n');
	}

	/** Toggle blockquote on current line. */
	toggleBlockquote(): TextChange[] {
		this.history.pushBoundary();
		const line = this.buffer.getLineFromOffset(this.selection.head);
		const lineStart = this.buffer.getLineStart(line);
		const lineText = this.buffer.getLine(line);
		const selBefore = this.selection.toJSON();
		const changes: TextChange[] = [];

		if (lineText.startsWith('> ')) {
			changes.push(this.buffer.delete(lineStart, 2));
		} else {
			changes.push(this.buffer.insert(lineStart, '> '));
		}

		this.selection = this.selection.clamp(this.buffer.length);
		const combinedChange: TextChange = {
			offset: lineStart,
			oldLength: changes.reduce((sum, c) => sum + c.oldLength, 0),
			newLength: changes.reduce((sum, c) => sum + c.newLength, 0),
			oldText: changes.map(c => c.oldText).join(''),
			newText: changes.map(c => c.newText).join(''),
		};
		this.history.push(combinedChange, selBefore, this.selection.toJSON());
		this.notify(changes);
		return changes;
	}

	/** Indent / outdent the current line. */
	indent(outdent: boolean = false): TextChange {
		this.history.pushBoundary();
		const line = this.buffer.getLineFromOffset(this.selection.head);
		const lineStart = this.buffer.getLineStart(line);
		const lineText = this.buffer.getLine(line);
		const selBefore = this.selection.toJSON();
		let change: TextChange;

		if (outdent) {
			// Remove leading tab or up to 4 spaces
			if (lineText.startsWith('\t')) {
				change = this.buffer.delete(lineStart, 1);
			} else {
				const spaces = lineText.match(/^( {1,4})/);
				if (spaces) {
					change = this.buffer.delete(lineStart, spaces[1].length);
				} else {
					return { offset: lineStart, oldLength: 0, newLength: 0, oldText: '', newText: '' };
				}
			}
		} else {
			change = this.buffer.insert(lineStart, '\t');
		}

		this.selection = this.selection.clamp(this.buffer.length);
		this.history.push(change, selBefore, this.selection.toJSON());
		this.notify([change]);
		return change;
	}

	/** Wrap selection with an inline HTML span for font or color. */
	wrapWithSpan(style: string): TextChange {
		this.history.pushBoundary();
		const selBefore = this.selection.toJSON();
		const text = this.selection.isCollapsed ? '' : this.buffer.getText(this.selection.from, this.selection.length);
		const wrapped = `<span style="${style}">${text}</span>`;

		let change: TextChange;
		if (!this.selection.isCollapsed) {
			change = this.buffer.replace(this.selection.from, this.selection.length, wrapped);
		} else {
			change = this.buffer.insert(this.selection.head, wrapped);
		}

		this.selection = EditorSelection.cursor(this.selection.from + wrapped.length);
		this.history.push(change, selBefore, this.selection.toJSON());
		this.notify([change]);
		return change;
	}

	// ── Undo/Redo ──

	undo(): boolean {
		const entry = this.history.undo();
		if (!entry) return false;

		// Apply changes in reverse
		for (let i = entry.changes.length - 1; i >= 0; i--) {
			const change = entry.changes[i];
			if (change.newLength > 0) {
				this.buffer.delete(change.offset, change.newLength);
			}
			if (change.oldLength > 0) {
				this.buffer.insert(change.offset, change.oldText);
			}
		}

		this.selection = EditorSelection.fromJSON(entry.selectionBefore);
		this.notify(entry.changes);
		return true;
	}

	redo(): boolean {
		const entry = this.history.redo();
		if (!entry) return false;

		// Re-apply changes
		for (const change of entry.changes) {
			if (change.oldLength > 0) {
				this.buffer.delete(change.offset, change.oldLength);
			}
			if (change.newLength > 0) {
				this.buffer.insert(change.offset, change.newText);
			}
		}

		this.selection = EditorSelection.fromJSON(entry.selectionAfter);
		this.notify(entry.changes);
		return true;
	}

	// ── Cursor movement ──

	moveCursor(offset: number, extend: boolean = false): void {
		const clamped = Math.max(0, Math.min(offset, this.buffer.length));
		if (extend) {
			this.selection = EditorSelection.range(this.selection.anchor, clamped);
		} else {
			this.selection = EditorSelection.cursor(clamped);
		}
	}

	moveToLineStart(extend: boolean = false): void {
		const line = this.buffer.getLineFromOffset(this.selection.head);
		const lineStart = this.buffer.getLineStart(line);
		this.moveCursor(lineStart, extend);
	}

	moveToLineEnd(extend: boolean = false): void {
		const line = this.buffer.getLineFromOffset(this.selection.head);
		const lineEnd = this.buffer.getLineEnd(line);
		this.moveCursor(lineEnd, extend);
	}

	moveUp(extend: boolean = false): void {
		const pos = this.buffer.getPosition(this.selection.head);
		if (pos.line === 0) {
			this.moveCursor(0, extend);
			return;
		}
		const targetCol = pos.column;
		const prevLineEnd = this.buffer.getLineEnd(pos.line - 1);
		const prevLineStart = this.buffer.getLineStart(pos.line - 1);
		const prevLineLen = prevLineEnd - prevLineStart;
		const newCol = Math.min(targetCol, prevLineLen);
		this.moveCursor(prevLineStart + newCol, extend);
	}

	moveDown(extend: boolean = false): void {
		const pos = this.buffer.getPosition(this.selection.head);
		if (pos.line >= this.buffer.lineCount - 1) {
			this.moveCursor(this.buffer.length, extend);
			return;
		}
		const targetCol = pos.column;
		const nextLineEnd = this.buffer.getLineEnd(pos.line + 1);
		const nextLineStart = this.buffer.getLineStart(pos.line + 1);
		const nextLineLen = nextLineEnd - nextLineStart;
		const newCol = Math.min(targetCol, nextLineLen);
		this.moveCursor(nextLineStart + newCol, extend);
	}

	moveWordLeft(extend: boolean = false): void {
		let pos = this.selection.head;
		const text = this.buffer.getFullText();
		if (pos > 0) pos--;
		while (pos > 0 && /\s/.test(text[pos])) pos--;
		while (pos > 0 && /\S/.test(text[pos - 1])) pos--;
		this.moveCursor(pos, extend);
	}

	moveWordRight(extend: boolean = false): void {
		let pos = this.selection.head;
		const text = this.buffer.getFullText();
		const len = text.length;
		while (pos < len && /\S/.test(text[pos])) pos++;
		while (pos < len && /\s/.test(text[pos])) pos++;
		this.moveCursor(pos, extend);
	}

	selectAll(): void {
		this.selection = EditorSelection.range(0, this.buffer.length);
	}
}

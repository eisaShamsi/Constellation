/**
 * InputHandler — keyboard, IME, and clipboard handling for the Constellation Editor.
 *
 * Uses beforeinput for text operations and keydown for shortcuts.
 * Full IME composition support for CJK + Arabic.
 */

import { EditEngine } from './EditEngine';
import { EditorSelection } from './Selection';

export interface KeyBinding {
	key: string;
	ctrl?: boolean;
	shift?: boolean;
	alt?: boolean;
	action: (engine: EditEngine) => void;
}

const IS_MAC = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform);

export class InputHandler {
	private engine: EditEngine;
	private element: HTMLElement;
	private composing = false;
	private compositionText = '';
	private keyBindings: KeyBinding[] = [];
	private boundHandlers: { event: string; handler: EventListener }[] = [];

	constructor(engine: EditEngine, element: HTMLElement) {
		this.engine = engine;
		this.element = element;
		this.setupDefaultBindings();
	}

	attach(): void {
		const handlers: [string, EventListener][] = [
			['beforeinput', this.onBeforeInput.bind(this) as EventListener],
			['keydown', this.onKeyDown.bind(this) as EventListener],
			['compositionstart', this.onCompositionStart.bind(this) as EventListener],
			['compositionupdate', this.onCompositionUpdate.bind(this) as EventListener],
			['compositionend', this.onCompositionEnd.bind(this) as EventListener],
			['paste', this.onPaste.bind(this) as EventListener],
			['copy', this.onCopy.bind(this) as EventListener],
			['cut', this.onCut.bind(this) as EventListener],
		];

		for (const [event, handler] of handlers) {
			this.element.addEventListener(event, handler);
			this.boundHandlers.push({ event, handler });
		}
	}

	detach(): void {
		for (const { event, handler } of this.boundHandlers) {
			this.element.removeEventListener(event, handler);
		}
		this.boundHandlers = [];
	}

	addKeyBinding(binding: KeyBinding): void {
		this.keyBindings.unshift(binding); // Higher priority for user bindings
	}

	private setupDefaultBindings(): void {
		const mod = IS_MAC ? 'meta' : 'ctrl';

		// Undo/Redo
		this.keyBindings.push(
			{ key: 'z', ctrl: true, action: (e) => e.undo() },
			{ key: 'y', ctrl: true, action: (e) => e.redo() },
			{ key: 'z', ctrl: true, shift: true, action: (e) => e.redo() },
		);

		// Formatting
		this.keyBindings.push(
			{ key: 'b', ctrl: true, action: (e) => e.toggleMark('**') },
			{ key: 'i', ctrl: true, action: (e) => e.toggleMark('*') },
			{ key: 'u', ctrl: true, action: (e) => e.toggleMark('<u>', '</u>') as any },
			{ key: 's', ctrl: true, shift: true, action: (e) => e.toggleMark('~~') },
			{ key: 'h', ctrl: true, shift: true, action: (e) => e.toggleMark('==') },
			{ key: 'e', ctrl: true, action: (e) => e.toggleMark('`') },
		);

		// Headings
		for (let i = 1; i <= 6; i++) {
			this.keyBindings.push({
				key: String(i),
				ctrl: true,
				action: (e) => e.setHeading(i),
			});
		}
		this.keyBindings.push({ key: '0', ctrl: true, action: (e) => e.setHeading(0) });

		// Lists
		this.keyBindings.push(
			{ key: 'b', ctrl: true, shift: true, action: (e) => e.toggleList('bullet') },
			{ key: 'o', ctrl: true, shift: true, action: (e) => e.toggleList('ordered') },
			{ key: 't', ctrl: true, shift: true, action: (e) => e.toggleList('task') },
		);

		// Block formatting
		this.keyBindings.push(
			{ key: 'q', ctrl: true, shift: true, action: (e) => e.toggleBlockquote() },
			{ key: 'c', ctrl: true, shift: true, action: (e) => e.insertCodeBlock() },
		);

		// Selection / Navigation
		this.keyBindings.push(
			{ key: 'a', ctrl: true, action: (e) => e.selectAll() },
			{ key: ']', ctrl: true, action: (e) => e.indent(false) },
			{ key: '[', ctrl: true, action: (e) => e.indent(true) },
		);
	}

	private onBeforeInput(event: InputEvent): void {
		if (this.composing) return;

		switch (event.inputType) {
			case 'insertText':
				if (event.data) {
					event.preventDefault();
					this.engine.insertText(event.data);
				}
				break;
			case 'insertParagraph':
			case 'insertLineBreak':
				event.preventDefault();
				this.handleEnter();
				break;
			case 'deleteContentBackward':
				event.preventDefault();
				this.engine.deleteText('backward');
				break;
			case 'deleteContentForward':
				event.preventDefault();
				this.engine.deleteText('forward');
				break;
			case 'deleteWordBackward':
				event.preventDefault();
				this.engine.deleteWordBackward();
				break;
			case 'deleteWordForward':
				event.preventDefault();
				this.engine.deleteWordForward();
				break;
			case 'deleteSoftLineBackward':
				event.preventDefault();
				this.engine.moveToLineStart(false);
				break;
			case 'deleteSoftLineForward':
				event.preventDefault();
				this.engine.moveToLineEnd(false);
				break;
			case 'insertFromPaste':
				// Handled by paste event
				break;
		}
	}

	private handleEnter(): void {
		const line = this.engine.buffer.getLineFromOffset(this.engine.selection.head);
		const lineText = this.engine.buffer.getLine(line);

		// Auto-continue lists
		const bulletMatch = lineText.match(/^(\s*)- \[([ x])\] /);
		const bulletPlain = lineText.match(/^(\s*)- /);
		const orderedMatch = lineText.match(/^(\s*)(\d+)\. /);

		if (bulletMatch) {
			// If the line is just a checkbox with no content, remove the prefix
			if (lineText.trim() === '- [ ]' || lineText.trim() === '- [x]') {
				const lineStart = this.engine.buffer.getLineStart(line);
				this.engine.buffer.delete(lineStart, lineText.length);
				this.engine.selection = EditorSelection.cursor(lineStart);
				this.engine.insertText('\n');
			} else {
				this.engine.insertText('\n' + bulletMatch[1] + '- [ ] ');
			}
		} else if (bulletPlain) {
			if (lineText.trim() === '-') {
				const lineStart = this.engine.buffer.getLineStart(line);
				this.engine.buffer.delete(lineStart, lineText.length);
				this.engine.selection = EditorSelection.cursor(lineStart);
				this.engine.insertText('\n');
			} else {
				this.engine.insertText('\n' + bulletPlain[1] + '- ');
			}
		} else if (orderedMatch) {
			if (lineText.trim() === orderedMatch[2] + '.') {
				const lineStart = this.engine.buffer.getLineStart(line);
				this.engine.buffer.delete(lineStart, lineText.length);
				this.engine.selection = EditorSelection.cursor(lineStart);
				this.engine.insertText('\n');
			} else {
				const nextNum = parseInt(orderedMatch[2]) + 1;
				this.engine.insertText('\n' + orderedMatch[1] + nextNum + '. ');
			}
		} else if (lineText.startsWith('> ')) {
			if (lineText.trim() === '>') {
				const lineStart = this.engine.buffer.getLineStart(line);
				this.engine.buffer.delete(lineStart, lineText.length);
				this.engine.selection = EditorSelection.cursor(lineStart);
				this.engine.insertText('\n');
			} else {
				this.engine.insertText('\n> ');
			}
		} else {
			this.engine.insertText('\n');
		}
	}

	private onKeyDown(event: KeyboardEvent): void {
		if (this.composing) return;

		// Handle Tab
		if (event.key === 'Tab') {
			event.preventDefault();
			if (event.shiftKey) {
				this.engine.indent(true);
			} else {
				this.engine.indent(false);
			}
			return;
		}

		// Arrow keys
		if (this.handleArrowKeys(event)) return;

		// Handle Home/End
		if (event.key === 'Home') {
			event.preventDefault();
			this.engine.moveToLineStart(event.shiftKey);
			return;
		}
		if (event.key === 'End') {
			event.preventDefault();
			this.engine.moveToLineEnd(event.shiftKey);
			return;
		}

		// Check key bindings
		const ctrl = IS_MAC ? event.metaKey : event.ctrlKey;
		const shift = event.shiftKey;

		for (const binding of this.keyBindings) {
			if (
				binding.key === event.key.toLowerCase() &&
				(binding.ctrl ?? false) === ctrl &&
				(binding.shift ?? false) === shift &&
				(binding.alt ?? false) === event.altKey
			) {
				event.preventDefault();
				binding.action(this.engine);
				return;
			}
		}
	}

	private handleArrowKeys(event: KeyboardEvent): boolean {
		const shift = event.shiftKey;
		const ctrl = IS_MAC ? event.metaKey : event.ctrlKey;

		switch (event.key) {
			case 'ArrowLeft':
				event.preventDefault();
				if (ctrl) {
					this.engine.moveWordLeft(shift);
				} else if (!shift && !this.engine.selection.isCollapsed) {
					this.engine.moveCursor(this.engine.selection.from, false);
				} else {
					this.engine.moveCursor(this.engine.selection.head - 1, shift);
				}
				return true;

			case 'ArrowRight':
				event.preventDefault();
				if (ctrl) {
					this.engine.moveWordRight(shift);
				} else if (!shift && !this.engine.selection.isCollapsed) {
					this.engine.moveCursor(this.engine.selection.to, false);
				} else {
					this.engine.moveCursor(this.engine.selection.head + 1, shift);
				}
				return true;

			case 'ArrowUp':
				event.preventDefault();
				this.engine.moveUp(shift);
				return true;

			case 'ArrowDown':
				event.preventDefault();
				this.engine.moveDown(shift);
				return true;
		}
		return false;
	}

	// ── IME Composition ──

	private onCompositionStart(): void {
		this.composing = true;
		this.compositionText = '';
	}

	private onCompositionUpdate(event: CompositionEvent): void {
		this.compositionText = event.data;
	}

	private onCompositionEnd(event: CompositionEvent): void {
		this.composing = false;
		if (event.data) {
			this.engine.insertText(event.data);
		}
		this.compositionText = '';
	}

	// ── Clipboard ──

	private onCopy(event: ClipboardEvent): void {
		if (this.engine.selection.isCollapsed) return;
		event.preventDefault();
		const text = this.engine.buffer.getText(this.engine.selection.from, this.engine.selection.length);
		event.clipboardData?.setData('text/plain', text);
	}

	private onCut(event: ClipboardEvent): void {
		if (this.engine.selection.isCollapsed) return;
		event.preventDefault();
		const text = this.engine.buffer.getText(this.engine.selection.from, this.engine.selection.length);
		event.clipboardData?.setData('text/plain', text);
		this.engine.deleteText('forward');
	}

	private async onPaste(event: ClipboardEvent): Promise<void> {
		event.preventDefault();
		const clipboardData = event.clipboardData;
		if (!clipboardData) return;

		// Check for images
		for (const item of clipboardData.items) {
			if (item.type.startsWith('image/')) {
				// Emit event for the editor component to handle image saving
				this.element.dispatchEvent(new CustomEvent('paste-image', {
					detail: { file: item.getAsFile() },
					bubbles: true,
				}));
				return;
			}
		}

		let text = clipboardData.getData('text/plain');

		// If pasting HTML and not plain text mode, try to convert to Markdown
		if (!event.shiftKey) {
			const html = clipboardData.getData('text/html');
			if (html && !text) {
				// Basic HTML to Markdown conversion
				text = htmlToMarkdown(html);
			}
		}

		if (text) {
			this.engine.insertText(text);
		}
	}
}

/** Minimal HTML to Markdown converter for paste. */
function htmlToMarkdown(html: string): string {
	const doc = new DOMParser().parseFromString(html, 'text/html');
	return walkNode(doc.body);
}

function walkNode(node: Node): string {
	if (node.nodeType === Node.TEXT_NODE) {
		return node.textContent ?? '';
	}

	if (node.nodeType !== Node.ELEMENT_NODE) return '';
	const el = node as Element;
	const tag = el.tagName.toLowerCase();
	const children = Array.from(el.childNodes).map(walkNode).join('');

	switch (tag) {
		case 'b': case 'strong': return `**${children}**`;
		case 'i': case 'em': return `*${children}*`;
		case 'u': return `<u>${children}</u>`;
		case 's': case 'del': case 'strike': return `~~${children}~~`;
		case 'code': return el.parentElement?.tagName === 'PRE' ? children : `\`${children}\``;
		case 'pre': return `\n\`\`\`\n${children}\n\`\`\`\n`;
		case 'h1': return `\n# ${children}\n`;
		case 'h2': return `\n## ${children}\n`;
		case 'h3': return `\n### ${children}\n`;
		case 'h4': return `\n#### ${children}\n`;
		case 'h5': return `\n##### ${children}\n`;
		case 'h6': return `\n###### ${children}\n`;
		case 'p': return `\n${children}\n`;
		case 'br': return '\n';
		case 'a': {
			const href = el.getAttribute('href');
			return href ? `[${children}](${href})` : children;
		}
		case 'img': {
			const src = el.getAttribute('src') ?? '';
			const alt = el.getAttribute('alt') ?? '';
			return `![${alt}](${src})`;
		}
		case 'li': {
			const parent = el.parentElement;
			if (parent?.tagName === 'OL') {
				const index = Array.from(parent.children).indexOf(el) + 1;
				return `${index}. ${children}\n`;
			}
			return `- ${children}\n`;
		}
		case 'ul': case 'ol': return `\n${children}`;
		case 'blockquote': return children.split('\n').map(line => `> ${line}`).join('\n');
		case 'hr': return '\n---\n';
		case 'mark': return `==${children}==`;
		default: return children;
	}
}

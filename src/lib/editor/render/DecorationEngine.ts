/**
 * DecorationEngine — WYSIWYG decorations for the Constellation Editor.
 *
 * Decorations are visual overlays that hide Markdown syntax and show
 * formatted output. When the cursor enters a decoration, the syntax
 * is revealed for editing.
 */

import type { BlockToken, InlineToken } from '../parser/tokens';

export type DecorationType = 'hide' | 'style' | 'widget' | 'line';

export interface Decoration {
	type: DecorationType;
	from: number;
	to: number;
	// Style decoration
	className?: string;
	style?: string;
	tag?: string;  // 'strong', 'em', etc.
	// Hide decoration (hide syntax characters)
	hideFrom?: number;
	hideTo?: number;
	hideEndFrom?: number;
	hideEndTo?: number;
	// Widget decoration
	widgetType?: string;  // 'image', 'checkbox', 'math', etc.
	widgetData?: Record<string, any>;
	// Line decoration
	lineClassName?: string;
	lineStyle?: string;
}

export class DecorationEngine {
	private decorations: Decoration[] = [];
	private cursorPos: number = -1;

	/** Build decorations from parsed blocks. */
	buildFromBlocks(blocks: BlockToken[], cursorPos: number): Decoration[] {
		this.cursorPos = cursorPos;
		this.decorations = [];

		for (const block of blocks) {
			this.decorateBlock(block);
		}

		return this.decorations;
	}

	getDecorations(): Decoration[] {
		return this.decorations;
	}

	private cursorInRange(from: number, to: number): boolean {
		return this.cursorPos >= from && this.cursorPos <= to;
	}

	private decorateBlock(block: BlockToken): void {
		switch (block.type) {
			case 'heading':
				this.decorateHeading(block);
				break;
			case 'paragraph':
				this.decorateInlines(block.inlines ?? []);
				break;
			case 'codeBlock':
				this.decorateCodeBlock(block);
				break;
			case 'blockquote':
				this.decorateBlockquote(block);
				break;
			case 'callout':
				this.decorateCallout(block);
				break;
			case 'horizontalRule':
				this.decorations.push({
					type: 'line',
					from: block.from,
					to: block.to,
					lineClassName: 'ce-hr',
				});
				break;
			case 'list':
			case 'taskList':
				this.decorateList(block);
				break;
			case 'table':
				this.decorateTable(block);
				break;
			case 'frontmatter':
				this.decorations.push({
					type: 'line',
					from: block.from,
					to: block.to,
					lineClassName: 'ce-frontmatter',
				});
				break;
			case 'mathBlock':
				this.decorateMathBlock(block);
				break;
		}
	}

	private decorateHeading(block: BlockToken): void {
		const level = block.level ?? 1;
		const prefixLen = level + 1; // "## " = 3 chars for h2

		// Line decoration for heading style
		this.decorations.push({
			type: 'line',
			from: block.from,
			to: block.to,
			lineClassName: `ce-heading ce-h${level}`,
		});

		// Hide the "## " prefix unless cursor is on this line
		if (!this.cursorInRange(block.from, block.to)) {
			this.decorations.push({
				type: 'hide',
				from: block.from,
				to: block.from + prefixLen,
				hideFrom: block.from,
				hideTo: block.from + prefixLen,
			});
		}

		// Decorate inline content
		this.decorateInlines(block.inlines ?? []);
	}

	private decorateInlines(inlines: InlineToken[]): void {
		for (const token of inlines) {
			const revealed = this.cursorInRange(token.from, token.to);

			switch (token.type) {
				case 'bold':
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					this.decorations.push({
						type: 'style',
						from: token.syntaxTo ?? token.from,
						to: token.syntaxEndFrom ?? token.to,
						className: 'ce-bold',
						tag: 'strong',
					});
					if (token.children) this.decorateInlines(token.children);
					break;

				case 'italic':
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					this.decorations.push({
						type: 'style',
						from: token.syntaxTo ?? token.from,
						to: token.syntaxEndFrom ?? token.to,
						className: 'ce-italic',
						tag: 'em',
					});
					if (token.children) this.decorateInlines(token.children);
					break;

				case 'strikethrough':
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					this.decorations.push({
						type: 'style',
						from: token.syntaxTo ?? token.from,
						to: token.syntaxEndFrom ?? token.to,
						className: 'ce-strikethrough',
						tag: 's',
					});
					break;

				case 'highlight':
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					this.decorations.push({
						type: 'style',
						from: token.syntaxTo ?? token.from,
						to: token.syntaxEndFrom ?? token.to,
						className: 'ce-highlight',
						tag: 'mark',
					});
					break;

				case 'underline':
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					this.decorations.push({
						type: 'style',
						from: token.syntaxTo ?? token.from,
						to: token.syntaxEndFrom ?? token.to,
						className: 'ce-underline',
						tag: 'u',
					});
					if (token.children) this.decorateInlines(token.children);
					break;

				case 'inlineCode':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-inline-code',
						tag: 'code',
					});
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					break;

				case 'link':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-link',
						style: `--link-url: "${token.url}"`,
					});
					if (!revealed) {
						// In WYSIWYG mode: show only link text
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.from,
							hideTo: token.from! + 1, // hide [
							hideEndFrom: token.to! - (token.url?.length ?? 0) - 2, // hide ](url)
							hideEndTo: token.to,
						});
					}
					break;

				case 'wikilink':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-wikilink',
					});
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					break;

				case 'image':
					this.decorations.push({
						type: 'widget',
						from: token.from,
						to: token.to,
						widgetType: 'image',
						widgetData: { src: token.url, alt: token.alt },
					});
					break;

				case 'inlineMath':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-inline-math',
					});
					if (!revealed) {
						this.decorations.push({
							type: 'widget',
							from: token.from,
							to: token.to,
							widgetType: 'inlineMath',
							widgetData: { expr: token.content },
						});
					}
					break;

				case 'subscript':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-subscript',
						tag: 'sub',
					});
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					break;

				case 'superscript':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-superscript',
						tag: 'sup',
					});
					if (!revealed) {
						this.decorations.push({
							type: 'hide',
							from: token.from,
							to: token.to,
							hideFrom: token.syntaxFrom,
							hideTo: token.syntaxTo,
							hideEndFrom: token.syntaxEndFrom,
							hideEndTo: token.syntaxEndTo,
						});
					}
					break;

				case 'fontSpan':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-font-span',
						style: token.fontFamily ? `font-family: ${token.fontFamily}` : undefined,
					});
					if (token.children) this.decorateInlines(token.children);
					break;

				case 'colorSpan':
					this.decorations.push({
						type: 'style',
						from: token.from,
						to: token.to,
						className: 'ce-color-span',
						style: token.color ? `color: ${token.color}` : undefined,
					});
					if (token.children) this.decorateInlines(token.children);
					break;
			}
		}
	}

	private decorateCodeBlock(block: BlockToken): void {
		this.decorations.push({
			type: 'line',
			from: block.from,
			to: block.to,
			lineClassName: 'ce-code-block',
		});
		if (block.language) {
			this.decorations.push({
				type: 'widget',
				from: block.from,
				to: block.from,
				widgetType: 'codeBlockLabel',
				widgetData: { language: block.language },
			});
		}
	}

	private decorateBlockquote(block: BlockToken): void {
		this.decorations.push({
			type: 'line',
			from: block.from,
			to: block.to,
			lineClassName: 'ce-blockquote',
		});
		this.decorateInlines(block.inlines ?? []);
	}

	private decorateCallout(block: BlockToken): void {
		this.decorations.push({
			type: 'line',
			from: block.from,
			to: block.to,
			lineClassName: `ce-callout ce-callout-${block.calloutType}`,
		});
		this.decorations.push({
			type: 'widget',
			from: block.from,
			to: block.from,
			widgetType: 'callout',
			widgetData: {
				type: block.calloutType,
				title: block.calloutTitle,
			},
		});
	}

	private decorateList(block: BlockToken): void {
		const isTasks = block.type === 'taskList';
		this.decorations.push({
			type: 'line',
			from: block.from,
			to: block.to,
			lineClassName: isTasks ? 'ce-task-list' : (block.ordered ? 'ce-ordered-list' : 'ce-bullet-list'),
		});

		for (const child of block.children ?? []) {
			if (isTasks) {
				this.decorations.push({
					type: 'widget',
					from: child.from,
					to: child.from,
					widgetType: 'checkbox',
					widgetData: { checked: child.checked, line: child.line },
				});
			}
			this.decorateInlines(child.inlines ?? []);
		}
	}

	private decorateTable(block: BlockToken): void {
		this.decorations.push({
			type: 'widget',
			from: block.from,
			to: block.to,
			widgetType: 'table',
			widgetData: { children: block.children },
		});
	}

	private decorateMathBlock(block: BlockToken): void {
		this.decorations.push({
			type: 'widget',
			from: block.from,
			to: block.to,
			widgetType: 'mathBlock',
			widgetData: {
				content: '', // Will be filled by renderer
			},
		});
	}
}

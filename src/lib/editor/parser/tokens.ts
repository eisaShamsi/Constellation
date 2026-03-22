/**
 * Token type definitions for the Constellation Markdown parser.
 */

export type BlockType =
	| 'document'
	| 'frontmatter'
	| 'heading'
	| 'paragraph'
	| 'list'
	| 'listItem'
	| 'taskList'
	| 'taskItem'
	| 'blockquote'
	| 'callout'
	| 'codeBlock'
	| 'table'
	| 'tableRow'
	| 'tableCell'
	| 'horizontalRule'
	| 'mathBlock'
	| 'blankLine';

export type InlineType =
	| 'text'
	| 'bold'
	| 'italic'
	| 'strikethrough'
	| 'highlight'
	| 'underline'
	| 'inlineCode'
	| 'link'
	| 'wikilink'
	| 'image'
	| 'inlineMath'
	| 'subscript'
	| 'superscript'
	| 'fontSpan'
	| 'colorSpan'
	| 'htmlTag';

export interface BlockToken {
	type: BlockType;
	from: number;         // Start offset in document
	to: number;           // End offset in document
	line: number;         // 0-based line number
	lineCount: number;    // Number of lines this block spans
	level?: number;       // Heading level (1-6), list indent level
	language?: string;    // Code block language
	calloutType?: string; // Callout type (tip, warning, etc.)
	calloutTitle?: string;
	ordered?: boolean;    // List ordered/unordered
	checked?: boolean;    // Task list item checked state
	children?: BlockToken[];
	inlines?: InlineToken[];
}

export interface InlineToken {
	type: InlineType;
	from: number;          // Start offset (relative to document)
	to: number;            // End offset
	syntaxFrom?: number;   // Start of opening syntax (e.g., **)
	syntaxTo?: number;     // End of opening syntax
	syntaxEndFrom?: number; // Start of closing syntax
	syntaxEndTo?: number;   // End of closing syntax
	content?: string;      // Text content
	url?: string;          // Link/image URL
	alt?: string;          // Image alt text
	title?: string;        // Link title
	target?: string;       // Wikilink target
	fontFamily?: string;   // Font span
	color?: string;        // Color span
	children?: InlineToken[];
}

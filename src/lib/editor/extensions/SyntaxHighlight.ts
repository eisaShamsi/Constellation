/**
 * SyntaxHighlight — lightweight code block syntax highlighting.
 *
 * Uses regex-based highlighting for common languages.
 * Can be swapped for Shiki in the future for full accuracy.
 */

export interface HighlightToken {
	from: number;
	to: number;
	className: string;
}

const KEYWORD_REGEX: Record<string, RegExp> = {
	javascript: /\b(const|let|var|function|return|if|else|for|while|do|switch|case|break|continue|new|this|class|extends|import|export|default|from|async|await|try|catch|finally|throw|typeof|instanceof|in|of|yield|null|undefined|true|false)\b/g,
	typescript: /\b(const|let|var|function|return|if|else|for|while|do|switch|case|break|continue|new|this|class|extends|import|export|default|from|async|await|try|catch|finally|throw|typeof|instanceof|in|of|yield|null|undefined|true|false|type|interface|enum|implements|public|private|protected|readonly|abstract|as|keyof|never|void|unknown|any)\b/g,
	python: /\b(def|class|return|if|elif|else|for|while|import|from|as|try|except|finally|raise|with|yield|lambda|pass|break|continue|and|or|not|is|in|True|False|None|self|async|await|print)\b/g,
	rust: /\b(fn|let|mut|const|struct|enum|impl|trait|pub|use|mod|crate|self|super|match|if|else|for|while|loop|break|continue|return|async|await|move|ref|where|type|dyn|static|unsafe|extern)\b/g,
	go: /\b(func|var|const|type|struct|interface|map|chan|go|select|case|default|if|else|for|range|switch|return|break|continue|defer|package|import|nil|true|false|make|new|len|cap|append)\b/g,
	html: /(<\/?[\w-]+|>|\/?>)/g,
	css: /(@[\w-]+|:\s*[\w-]+|\.[\w-]+|#[\w-]+)/g,
};

const STRING_REGEX = /(["'`])(?:(?!\1|\\).|\\.)*\1/g;
const COMMENT_REGEX = /\/\/.*$|\/\*[\s\S]*?\*\/|#.*$/gm;
const NUMBER_REGEX = /\b\d+(\.\d+)?\b/g;

export function highlightCode(code: string, language: string): HighlightToken[] {
	const tokens: HighlightToken[] = [];
	const lang = language.toLowerCase();

	// Strings
	let match: RegExpExecArray | null;
	const strRegex = new RegExp(STRING_REGEX.source, 'g');
	while ((match = strRegex.exec(code)) !== null) {
		tokens.push({ from: match.index, to: match.index + match[0].length, className: 'ce-hl-string' });
	}

	// Comments
	const commentRegex = new RegExp(COMMENT_REGEX.source, 'gm');
	while ((match = commentRegex.exec(code)) !== null) {
		tokens.push({ from: match.index, to: match.index + match[0].length, className: 'ce-hl-comment' });
	}

	// Numbers
	const numRegex = new RegExp(NUMBER_REGEX.source, 'g');
	while ((match = numRegex.exec(code)) !== null) {
		tokens.push({ from: match.index, to: match.index + match[0].length, className: 'ce-hl-number' });
	}

	// Keywords
	const kwRegex = KEYWORD_REGEX[lang] ?? KEYWORD_REGEX[resolveAlias(lang)];
	if (kwRegex) {
		const kw = new RegExp(kwRegex.source, 'g');
		while ((match = kw.exec(code)) !== null) {
			tokens.push({ from: match.index, to: match.index + match[0].length, className: 'ce-hl-keyword' });
		}
	}

	return tokens;
}

function resolveAlias(lang: string): string {
	const aliases: Record<string, string> = {
		js: 'javascript',
		ts: 'typescript',
		py: 'python',
		rs: 'rust',
		jsx: 'javascript',
		tsx: 'typescript',
		mjs: 'javascript',
		cjs: 'javascript',
	};
	return aliases[lang] ?? lang;
}

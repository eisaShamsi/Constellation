/**
 * MathRenderer — KaTeX integration for the Constellation Editor.
 *
 * Renders inline $expr$ and block $$ expr $$ as formatted math.
 * Falls back to plain text if KaTeX is not available.
 */

let katexLoaded = false;
let katexModule: any = null;

async function loadKaTeX(): Promise<any> {
	if (katexLoaded) return katexModule;
	try {
		katexModule = await import('katex');
		katexLoaded = true;
		return katexModule;
	} catch {
		return null;
	}
}

/**
 * Render a math expression to HTML.
 */
export async function renderMath(expr: string, displayMode: boolean = false): Promise<string> {
	const katex = await loadKaTeX();
	if (!katex) {
		// Fallback: show expression in monospace
		return `<span class="ce-math-fallback">${escapeHtml(expr)}</span>`;
	}

	try {
		const render = katex.default?.renderToString ?? katex.renderToString;
		return render(expr, {
			displayMode,
			throwOnError: false,
			output: 'html',
		});
	} catch {
		return `<span class="ce-math-error">${escapeHtml(expr)}</span>`;
	}
}

/**
 * Render math synchronously (returns placeholder if not loaded yet).
 */
export function renderMathSync(expr: string, displayMode: boolean = false): string {
	if (!katexLoaded || !katexModule) {
		// Trigger async load for next time
		loadKaTeX();
		return `<span class="ce-math-fallback">${escapeHtml(expr)}</span>`;
	}

	try {
		const render = katexModule.default?.renderToString ?? katexModule.renderToString;
		return render(expr, {
			displayMode,
			throwOnError: false,
			output: 'html',
		});
	} catch {
		return `<span class="ce-math-error">${escapeHtml(expr)}</span>`;
	}
}

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;');
}

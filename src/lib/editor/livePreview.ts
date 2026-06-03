/**
 * Live Preview — CodeMirror ViewPlugin that renders markdown inline.
 * Hides syntax characters when cursor is not on the same line,
 * and styles content (headings, bold, italic, etc.) directly in the editor.
 */
import {
	ViewPlugin,
	type ViewUpdate,
	Decoration,
	type DecorationSet,
	EditorView,
	WidgetType,
} from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';
import { EditorState, RangeSetBuilder, StateField, StateEffect } from '@codemirror/state';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { t, tIn } from '$lib/i18n';
import { detectDir, dominantLocale } from '$lib/utils';
import { appSettings, skyNodePathSet } from '$lib/libraries/store';
import { isLinkTypeValue, getLinkType, linkTypeLabel, subscribe as subscribeLinkTypes } from '$lib/libraries/linkTypeRegistry';
import type { LensResult, LensRow, DimensionValue } from '$lib/lens/store';
import { dataColumns, columnLabel, renderCellValue } from '$lib/lens/tableModel';

// ─── Path state fields (for resolving image embeds) ───
export const setLibraryPath       = StateEffect.define<string>();
export const setNotePath          = StateEffect.define<string>();
export const setAttachmentFolder  = StateEffect.define<string>();

export const libraryPathField = StateField.define<string>({
	create: () => '',
	update(value, tr) {
		for (const effect of tr.effects) {
			if (effect.is(setLibraryPath)) return effect.value;
		}
		return value;
	},
});

export const notePathField = StateField.define<string>({
	create: () => '',
	update(value, tr) {
		for (const effect of tr.effects) {
			if (effect.is(setNotePath)) return effect.value;
		}
		return value;
	},
});

export const attachmentFolderField = StateField.define<string>({
	create: () => '',
	update(value, tr) {
		for (const effect of tr.effects) {
			if (effect.is(setAttachmentFolder)) return effect.value;
		}
		return value;
	},
});

// ─── Living Link traversal count map (P4.2) ───
// `${sourcePath.toLowerCase()}|${targetName.toLowerCase()}` → traversal_count.
// Source is the currently-open note's path (read via notePathField); target
// is the wikilink name. The decoration pipeline consults this map when
// rendering a wikilink so it can emit a `×N` chip after the link for worn
// paths. Empty map = no chips (boot graph not yet loaded, or no links in
// this note have been traversed).
export const setLinkTraversalMap = StateEffect.define<Map<string, number>>();

export const linkTraversalMapField = StateField.define<Map<string, number>>({
	create: () => new Map(),
	update(value, tr) {
		for (const effect of tr.effects) {
			if (effect.is(setLinkTraversalMap)) return effect.value;
		}
		return value;
	},
});

// Accent-tint used by the wikilink `×N` chip. Falls back to the Constellation
// purple so the chip is visible even before the user picks a theme accent.
const WIKILINK_CHIP_ACCENT = 'var(--interactive-accent, #7c3aed)';

class WikilinkTraversalChipWidget extends WidgetType {
	constructor(public count: number) { super(); }
	toDOM() {
		const el = document.createElement('span');
		el.className = 'cm-living-link-chip';
		el.textContent = '×' + this.count;
		el.title = 'Traversed ' + this.count + (this.count === 1 ? ' time' : ' times');
		// Inline styles keep the chip themable without needing a theme rule.
		el.setAttribute('style',
			'display:inline-flex;align-items:center;margin:0 3px;padding:0 6px;' +
			'font-size:0.6rem;font-weight:700;line-height:1;height:15px;' +
			'border-radius:8px;font-variant-numeric:tabular-nums;' +
			'letter-spacing:0.02em;vertical-align:middle;' +
			'color:' + WIKILINK_CHIP_ACCENT + ';' +
			'background:color-mix(in srgb,' + WIKILINK_CHIP_ACCENT + ' 14%, transparent);' +
			'border:1px solid color-mix(in srgb,' + WIKILINK_CHIP_ACCENT + ' 30%, transparent);' +
			'box-sizing:border-box;'
		);
		return el;
	}
	eq(other: WikilinkTraversalChipWidget) { return this.count === other.count; }
	ignoreEvent() { return true; }
}

/** Returns ordered list of candidate absolute paths for an embedded image.
 *  Search order: note's folder → library root.
 *  We return all candidates so ImageWidget can try each via onerror chaining. */
function resolveEmbedCandidates(view: EditorView, filename: string): string[] {
	const libPath  = view.state.field(libraryPathField, false) || '';
	const notePath = view.state.field(notePathField,    false) || '';
	const attachFolder = view.state.field(attachmentFolderField, false) || '';
	if (!libPath && !notePath) return [];

	const sep = (libPath || notePath).includes('\\') ? '\\' : '/';
	const candidates: string[] = [];

	// 1. Folder containing the current note (most common: image next to note)
	if (notePath) {
		const noteDir = notePath.substring(0, notePath.lastIndexOf(sep));
		if (noteDir) candidates.push(noteDir + sep + filename);
	}

	// 2. Custom attachment folder from settings (if configured)
	if (libPath && attachFolder) {
		candidates.push(libPath + sep + attachFolder + sep + filename);
	}

	// 3. Common attachment folders + library root
	if (libPath) {
		if (attachFolder !== 'attachments') candidates.push(libPath + sep + 'attachments' + sep + filename);
		if (attachFolder !== 'images')      candidates.push(libPath + sep + 'images' + sep + filename);
		if (attachFolder !== 'assets')      candidates.push(libPath + sep + 'assets' + sep + filename);
		candidates.push(libPath + sep + filename);
	}

	const results: string[] = [];
	for (const p of candidates) {
		try { results.push(convertFileSrc(p)); } catch { /* skip invalid path */ }
	}
	return results;
}

// Pre-cached module-level decorations — allocated once, reused every rebuild.
// Creating new Decoration objects on every buildDecorations() call generates GC
// pressure and wastes CPU; CM6 already uses eq() to avoid DOM rebuilds.
const headingDecos = [
	Decoration.mark({ class: 'cm-md-heading1' }),
	Decoration.mark({ class: 'cm-md-heading2' }),
	Decoration.mark({ class: 'cm-md-heading3' }),
	Decoration.mark({ class: 'cm-md-heading4' }),
	Decoration.mark({ class: 'cm-md-heading5' }),
	Decoration.mark({ class: 'cm-md-heading6' }),
];

const boldDeco = Decoration.mark({ class: 'cm-md-bold' });
const italicDeco = Decoration.mark({ class: 'cm-md-italic' });
const strikeDeco = Decoration.mark({ class: 'cm-md-strikethrough' });
const codeDeco = Decoration.mark({ class: 'cm-md-code' });
const linkDeco = Decoration.mark({ class: 'cm-md-link' });
const highlightDeco = Decoration.mark({ class: 'cm-md-highlight' });
const hrDeco = Decoration.mark({ class: 'cm-md-hr' });
const blockquoteDeco = Decoration.mark({ class: 'cm-md-blockquote' });
const tagDeco = Decoration.mark({ class: 'cm-md-tag' });
const replaceDeco = Decoration.replace({}); /* cached — avoids allocation per decoration */
const htmlHiddenDeco = Decoration.mark({ class: 'cm-html-hidden' }); /* hide HTML tags without breaking bidi */
const htmlUDeco    = Decoration.mark({ class: 'cm-html-u' });
const htmlSubDeco  = Decoration.mark({ class: 'cm-html-sub' });
const htmlSupDeco  = Decoration.mark({ class: 'cm-html-sup' });
const htmlMarkDeco = Decoration.mark({ class: 'cm-html-mark' });
const htmlDecoMap: Record<string, Decoration> = { u: htmlUDeco, sub: htmlSubDeco, sup: htmlSupDeco, mark: htmlMarkDeco };

// MIG-067 §E — every typed link is coloured by an INLINE style read from the
// Link-Type Registry, built-ins AND custom alike. (The earlier `cm-link-*`
// CSS-class path silently stopped applying — every typed link rendered the default
// link blue.) Inline `!important` always wins; reading the colour from the registry
// means recolouring any of the 8 in §G reflects here too. SEED_COLORS is the
// canonical fallback for the 8 before the registry has seeded (boot edge).
const SEED_COLORS: Record<string, string> = {
	supports: '#4A9EFF', contradicts: '#FF4A4A', causes: '#FF8C42', exemplifies: '#4AFF88',
	generalizes: '#A44AFF', 'derives-from': '#FFD700', 'part-of': '#AAAAAA', supersedes: '#5B7A8A',
};

// Lazily built per id (the type is known by render time), cached, cleared on any
// vocabulary change so recolours take effect.
const typeDecoCache = new Map<string, ReturnType<typeof Decoration.mark>>();
// Rebuild typed-link decorations when the vocabulary changes (recolour / rename /
// add / remove). Labels are keyed by the NOTE's language, not the UI language
// (§E.2), so a UI language switch does NOT invalidate them.
subscribeLinkTypes(() => typeDecoCache.clear());

/** The label above a typed link (§E.2): the type name in the NOTE's language `loc`
 *  (`linkTypes.<id>` — lowercase), so a link reads in its note's own language no
 *  matter the UI language. Falls back to the registry label for a custom type. */
function typedLinkLabel(id: string, loc: string): string {
	const key = 'linkTypes.' + id;
	const tr = tIn(loc, key);
	return tr !== key ? tr : linkTypeLabel(id);
}

/** Decoration for a typed-link id, labelled in the note's language `loc`: an inline
 *  registry/seed colour (`!important`) + the localized type name as data-ltype, or the
 *  plain link decoration for the null `associative` and unknowns. Cached per id|loc. */
function typeDeco(id: string, loc: string): ReturnType<typeof Decoration.mark> {
	const ckey = id + '|' + loc;
	let d = typeDecoCache.get(ckey);
	if (d === undefined) {
		const color = getLinkType(id)?.color ?? SEED_COLORS[id];
		d = color
			? Decoration.mark({ class: 'cm-md-link cm-ltyped', attributes: { 'data-ltype': typedLinkLabel(id, loc), style: `--ltc:${color};color:${color} !important;text-decoration-color:${color}66 !important` } })
			: linkDeco;
		typeDecoCache.set(ckey, d);
	}
	return d;
}

class CheckboxWidget extends WidgetType {
	checked: boolean;
	constructor(checked: boolean) {
		super();
		this.checked = checked;
	}
	toDOM() {
		const cb = document.createElement('input');
		cb.type = 'checkbox';
		cb.checked = this.checked;
		cb.className = 'cm-md-checkbox';
		cb.setAttribute('aria-label', this.checked ? 'Completed' : 'Todo');
		return cb;
	}
	eq(other: CheckboxWidget) { return this.checked === other.checked; }
}
// Pre-cached checkbox replacement decorations — only two possible states exist
const checkboxCheckedDeco   = Decoration.replace({ widget: new CheckboxWidget(true) });
const checkboxUncheckedDeco = Decoration.replace({ widget: new CheckboxWidget(false) });

// Cache resolved image data URLs to avoid repeated IPC calls.
// Key: "libraryPath|notePath|filename", Value: data URL or '' (not found).
const _imageCache = new Map<string, string>();

/** Cache for resolve_embed results so repeat renders don't spam IPC. */
interface EmbedResolution {
	kind: 'image' | 'audio' | 'video' | 'pdf' | 'canvas' | 'excalidraw' | 'note' | 'generic' | 'missing';
	url: string;
	absolute_path?: string | null;
	mime?: string | null;
	size_bytes: number;
	note_body?: string | null;
	heading?: string | null;
	block_id?: string | null;
	tried_paths?: string[];
	attachment_folder?: string;
	similar_files?: string[];
	vault_file_count?: number;
	attachment_folder_listing?: string[];
	attachment_folder_resolved?: string;
}
const _embedCache = new Map<string, EmbedResolution>();
/** Circular-guard for note transclusion: tracks paths currently being rendered. */
const _transcludeStack = new Set<string>();

/** Widget for inline images — resolves via Rust IPC (handles non-ASCII paths correctly). */
class ImageWidget extends WidgetType {
	filename: string;
	alt: string;
	libraryPath: string;
	notePath: string;
	constructor(filename: string, alt: string, libraryPath: string, notePath: string) {
		super();
		this.filename = filename;
		this.alt = alt;
		this.libraryPath = libraryPath;
		this.notePath = notePath;
	}
	toDOM() {
		const wrap = document.createElement('div');
		wrap.className = 'cm-md-image-widget';

		// Absolute URL (http/https/data/asset/file/blob) — render directly, no
		// Rust resolution. Match on the filename itself: the old "paths empty"
		// heuristic confounded "caller intentionally passed no context" with
		// "state fields not yet populated on first render", which let relative
		// paths like `attachments/img/foo.png` take this branch and 404
		// against the dev server on every initial render.
		if (/^(https?:|data:|asset:|file:|blob:)/i.test(this.filename)) {
			const img = document.createElement('img');
			img.src = this.filename;
			img.alt = this.alt || '';
			img.loading = 'lazy';
			img.onerror = () => this._showFallback(wrap);
			wrap.appendChild(img);
			return wrap;
		}

		// Relative path with no resolution context at all — can't fetch safely.
		// Show the fallback placeholder instead of handing a relative URL to
		// the browser (which would resolve it against the dev origin and
		// produce a 404 flood on every first-render before the setLibraryPath
		// / setNotePath effects get dispatched). If even one field is set the
		// Rust side can still try its candidate list.
		if (!this.libraryPath && !this.notePath) {
			this._showFallback(wrap);
			return wrap;
		}

		const cacheKey = `${this.libraryPath}|${this.notePath}|${this.filename}`;
		const cached = _imageCache.get(cacheKey);

		if (cached) {
			// Cache hit — render immediately
			const img = document.createElement('img');
			img.src = cached;
			img.alt = this.alt || '';
			wrap.appendChild(img);
		} else if (cached === '') {
			// Cached as not-found
			this._showFallback(wrap);
		} else {
			// Cache miss — show placeholder, resolve async via Rust
			const placeholder = document.createElement('span');
			placeholder.className = 'cm-md-image-fallback';
			placeholder.textContent = `⏳ ${this.alt || this.filename}`;
			wrap.appendChild(placeholder);

			invoke<string>('resolve_embed_image', {
				libraryPath: this.libraryPath,
				notePath: this.notePath,
				filename: this.filename,
			}).then(dataUrl => {
				if (dataUrl) {
					_imageCache.set(cacheKey, dataUrl);
					wrap.innerHTML = '';
					const img = document.createElement('img');
					img.src = dataUrl;
					img.alt = this.alt || '';
					wrap.appendChild(img);
				} else {
					_imageCache.set(cacheKey, '');
					this._showFallback(wrap);
				}
			}).catch(() => {
				_imageCache.set(cacheKey, '');
				this._showFallback(wrap);
			});
		}
		return wrap;
	}
	private _showFallback(wrap: HTMLDivElement) {
		wrap.innerHTML = '';
		const fallback = document.createElement('span');
		fallback.className = 'cm-md-image-fallback';
		fallback.textContent = `📷 ${this.alt || this.filename}`;
		wrap.appendChild(fallback);
	}
	eq(other: ImageWidget) {
		return this.filename === other.filename
			&& this.libraryPath === other.libraryPath
			&& this.notePath === other.notePath;
	}
}

/**
 * Universal embed widget — resolves `![[target]]` via Rust and renders the
 * appropriate media: image, audio player, video player, PDF iframe, canvas
 * / excalidraw preview, note transclusion, generic file pill, or a visible
 * "not found" placeholder.
 */
class UniversalEmbedWidget extends WidgetType {
	constructor(
		public target: string,
		public displayAlias: string,
		public libraryPath: string,
		public notePath: string,
	) { super(); }

	toDOM() {
		const wrap = document.createElement('div');
		wrap.className = 'cm-md-embed';
		const cacheKey = `${this.libraryPath}|${this.notePath}|${this.target}`;
		const cached = _embedCache.get(cacheKey);
		if (cached) {
			this._render(wrap, cached);
			return wrap;
		}
		// Async resolve — show skeleton, swap when ready
		this._renderLoading(wrap);
		invoke<EmbedResolution>('resolve_embed', {
			libraryPath: this.libraryPath,
			notePath: this.notePath,
			target: this.target,
		}).then(res => {
			_embedCache.set(cacheKey, res);
			wrap.innerHTML = '';
			this._render(wrap, res);
		}).catch(() => {
			wrap.innerHTML = '';
			this._renderMissing(wrap);
		});
		return wrap;
	}

	private _renderLoading(wrap: HTMLDivElement) {
		const el = document.createElement('span');
		el.className = 'cm-embed-loading';
		el.textContent = `⏳ ${this.displayAlias || this.target}`;
		wrap.appendChild(el);
	}

	private _render(wrap: HTMLDivElement, res: EmbedResolution) {
		switch (res.kind) {
			case 'image':      return this._renderImage(wrap, res);
			case 'audio':      return this._renderAudio(wrap, res);
			case 'video':      return this._renderVideo(wrap, res);
			case 'pdf':        return this._renderPdf(wrap, res);
			case 'canvas':     return this._renderCanvas(wrap, res);
			case 'excalidraw': return this._renderExcalidraw(wrap, res);
			case 'note':       return this._renderNote(wrap, res);
			case 'generic':    return this._renderGeneric(wrap, res);
			default:           return this._renderMissing(wrap, res);
		}
	}

	private _renderImage(wrap: HTMLDivElement, res: EmbedResolution) {
		const img = document.createElement('img');
		img.src = res.url;
		img.alt = this.displayAlias || this.target;
		img.loading = 'lazy';
		img.onerror = () => this._renderMissing(wrap);
		wrap.appendChild(img);
	}

	private _renderAudio(wrap: HTMLDivElement, res: EmbedResolution) {
		const a = document.createElement('audio');
		a.src = res.url;
		a.controls = true;
		a.preload = 'metadata';
		wrap.appendChild(a);
		const cap = document.createElement('div');
		cap.className = 'cm-embed-caption';
		cap.textContent = `🎵 ${this.displayAlias || this.target}`;
		wrap.appendChild(cap);
	}

	private _renderVideo(wrap: HTMLDivElement, res: EmbedResolution) {
		const v = document.createElement('video');
		v.src = res.url;
		v.controls = true;
		v.preload = 'metadata';
		v.playsInline = true;
		v.style.maxWidth = '100%';
		wrap.appendChild(v);
		const cap = document.createElement('div');
		cap.className = 'cm-embed-caption';
		cap.textContent = `🎬 ${this.displayAlias || this.target}`;
		wrap.appendChild(cap);
	}

	private _renderPdf(wrap: HTMLDivElement, res: EmbedResolution) {
		// Obsidian supports `#page=N` PDF fragments; carry them through to the viewer.
		const page = this.target.match(/#page=(\d+)/)?.[1];
		const src = page ? `${res.url}#page=${page}` : res.url;
		const iframe = document.createElement('iframe');
		iframe.src = src;
		iframe.className = 'cm-embed-pdf';
		iframe.setAttribute('sandbox', 'allow-same-origin allow-scripts');
		wrap.appendChild(iframe);
		const cap = document.createElement('div');
		cap.className = 'cm-embed-caption';
		cap.textContent = `📄 ${this.displayAlias || this.target}${page ? ` · page ${page}` : ''}`;
		wrap.appendChild(cap);
	}

	private _renderCanvas(wrap: HTMLDivElement, res: EmbedResolution) {
		// Obsidian Canvas is JSON with {nodes, edges}. Render a compact preview.
		let summary = 'Canvas';
		try {
			const doc = JSON.parse(res.note_body ?? '{}');
			const n = doc.nodes?.length ?? 0;
			const e = doc.edges?.length ?? 0;
			summary = `Canvas · ${n} node${n !== 1 ? 's' : ''} · ${e} edge${e !== 1 ? 's' : ''}`;
		} catch {}
		wrap.appendChild(this._card('🗺️', this.displayAlias || this.target, summary, res.absolute_path));
	}

	private _renderExcalidraw(wrap: HTMLDivElement, res: EmbedResolution) {
		// Excalidraw embeds in Obsidian have a `.excalidraw.svg` sibling for preview.
		// If we find it, render the SVG; otherwise show a file-pill placeholder.
		let summary = 'Excalidraw drawing';
		try {
			const doc = JSON.parse(res.note_body ?? '{}');
			const el = doc.elements?.length ?? 0;
			summary = `Excalidraw · ${el} element${el !== 1 ? 's' : ''}`;
		} catch {}
		wrap.appendChild(this._card('✏️', this.displayAlias || this.target, summary, res.absolute_path));
	}

	private _renderNote(wrap: HTMLDivElement, res: EmbedResolution) {
		const absPath = res.absolute_path ?? '';
		if (_transcludeStack.has(absPath)) {
			// Circular transclusion — show a compact badge instead
			wrap.appendChild(this._card('🔄', this.displayAlias || this.target, 'Circular transclusion'));
			return;
		}
		_transcludeStack.add(absPath);
		try {
			let body = res.note_body ?? '';
			// Strip YAML frontmatter
			body = body.replace(/^---\n[\s\S]*?\n---\n?/, '');
			// Scope to heading / block if specified
			if (res.heading) body = extractHeading(body, res.heading);
			if (res.block_id) body = extractBlock(body, res.block_id);
			const container = document.createElement('div');
			container.className = 'cm-embed-transclusion';
			const hdr = document.createElement('div');
			hdr.className = 'cm-embed-transclusion-header';
			hdr.textContent = `📝 ${this.displayAlias || this.target}`;
			container.appendChild(hdr);
			const bodyEl = document.createElement('div');
			bodyEl.className = 'cm-embed-transclusion-body';
			// Minimal inline rendering — the transcluded note renders as markdown-ish
			// plain text. Users who need full CM6 live-preview inside transclusions
			// can click through to the source note (handler below).
			bodyEl.textContent = body.trim().slice(0, 4000);
			container.appendChild(bodyEl);
			hdr.style.cursor = 'pointer';
			hdr.addEventListener('click', () => {
				window.dispatchEvent(new CustomEvent('constellation:open-note', { detail: { path: absPath } }));
			});
			wrap.appendChild(container);
		} finally {
			_transcludeStack.delete(absPath);
		}
	}

	private _renderGeneric(wrap: HTMLDivElement, res: EmbedResolution) {
		const kb = res.size_bytes > 0 ? ` · ${formatBytes(res.size_bytes)}` : '';
		wrap.appendChild(this._card('📎', this.displayAlias || this.target, `File${kb}`, res.absolute_path));
	}

	private _renderMissing(wrap: HTMLDivElement, res?: EmbedResolution) {
		const card = this._card('⚠️', this.target, 'File not found in vault');
		card.classList.add('cm-embed-missing');
		if (res && (res.tried_paths?.length || res.similar_files?.length)) {
			const details = document.createElement('details');
			details.className = 'cm-embed-missing-details';
			const summary = document.createElement('summary');
			summary.textContent = 'Show lookup details';
			details.appendChild(summary);
			const info = document.createElement('div');
			info.className = 'cm-embed-missing-info';
			const af = res.attachment_folder ? `attachmentFolderPath: "${res.attachment_folder}"` : '(.obsidian/app.json not read or empty)';
			const tried = `Looked for:\n  ${(res.tried_paths ?? []).join('\n  ')}`;
			const fc = res.vault_file_count ?? 0;
			const count = `\n\nVault index: ${fc.toLocaleString()} file${fc === 1 ? '' : 's'} scanned`;
			const folderListingBlock = res.attachment_folder_listing?.length
				? `\n\nAttachment folder on disk (${res.attachment_folder_resolved}):\n  ${res.attachment_folder_listing.join('\n  ')}`
				: res.attachment_folder_resolved
					? `\n\nAttachment folder on disk (${res.attachment_folder_resolved}): (folder does not exist or is empty)`
					: '';
			const similar = res.similar_files?.length
				? `\n\nSimilar files in vault:\n  ${res.similar_files.join('\n  ')}`
				: fc === 0
					? '\n\nThe vault index is empty — the library path may not be readable (permission issue) or points to the wrong folder.'
					: '\n\nNo similar filenames found in the vault — the file may not exist, or it was moved/renamed.';
			info.textContent = `${af}${count}\n\n${tried}${folderListingBlock}${similar}`;
			details.appendChild(info);
			wrap.appendChild(details);
		}
		wrap.appendChild(card);
	}

	private _card(icon: string, title: string, subtitle: string, openPath?: string | null) {
		const card = document.createElement('div');
		card.className = 'cm-embed-card';
		const ic = document.createElement('span'); ic.className = 'cm-embed-card-icon'; ic.textContent = icon; card.appendChild(ic);
		const body = document.createElement('div'); body.className = 'cm-embed-card-body';
		const t = document.createElement('div'); t.className = 'cm-embed-card-title'; t.textContent = title; body.appendChild(t);
		const s = document.createElement('div'); s.className = 'cm-embed-card-sub'; s.textContent = subtitle; body.appendChild(s);
		card.appendChild(body);
		if (openPath) {
			card.style.cursor = 'pointer';
			card.addEventListener('click', () => {
				window.dispatchEvent(new CustomEvent('constellation:open-external', { detail: { path: openPath } }));
			});
		}
		return card;
	}

	eq(other: UniversalEmbedWidget) {
		return this.target === other.target
			&& this.libraryPath === other.libraryPath
			&& this.notePath === other.notePath;
	}
}

function extractHeading(md: string, heading: string): string {
	const target = heading.trim().toLowerCase();
	const lines = md.split('\n');
	let start = -1, endLevel = 7;
	for (let i = 0; i < lines.length; i++) {
		const m = lines[i].match(/^(#{1,6})\s+(.+?)\s*$/);
		if (m && m[2].trim().toLowerCase() === target) { start = i; endLevel = m[1].length; break; }
	}
	if (start < 0) return md;
	let end = lines.length;
	for (let i = start + 1; i < lines.length; i++) {
		const m = lines[i].match(/^(#{1,6})\s/);
		if (m && m[1].length <= endLevel) { end = i; break; }
	}
	return lines.slice(start, end).join('\n');
}

function extractBlock(md: string, blockId: string): string {
	const marker = `^${blockId}`;
	const lines = md.split('\n');
	for (let i = 0; i < lines.length; i++) {
		if (lines[i].includes(marker)) {
			// Walk back to start of paragraph / list item
			let start = i;
			while (start > 0 && lines[start - 1].trim() !== '') start--;
			return lines.slice(start, i + 1).join('\n').replace(new RegExp(`\\s*\\^${blockId}\\s*$`), '');
		}
	}
	return md;
}

function formatBytes(n: number): string {
	if (n < 1024) return `${n} B`;
	if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
	if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
	return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

/**
 * IconShortcodeWidget — renders `:set-name:` shortcodes (lucide-heart,
 * phosphor-heart, hi-heart, feather-heart) as inline SVG in the editor.
 * Looks up the icon in the cached iconSets map, lazy-loaded on first hit.
 */
let _iconsByIdCache: Map<string, string> | null = null;
let _iconsLoading: Promise<Map<string, string>> | null = null;

function loadIconsById(): Promise<Map<string, string>> {
	if (_iconsByIdCache) return Promise.resolve(_iconsByIdCache);
	if (!_iconsLoading) {
		_iconsLoading = import('./iconSets').then(async (mod) => {
			const all = await mod.loadAllIcons();
			const map = new Map<string, string>();
			for (const icon of all) map.set(icon.id, icon.svg);
			_iconsByIdCache = map;
			return map;
		});
	}
	return _iconsLoading;
}

class IconShortcodeWidget extends WidgetType {
	constructor(public iconId: string) { super(); }
	toDOM() {
		const span = document.createElement('span');
		span.className = `cm-icon-inline cn-icon-${this.iconId.replace(':', '-')}`;
		span.setAttribute('data-icon', this.iconId);
		// Render synchronously if cache already populated, else render a
		// placeholder that gets swapped when the cache loads.
		if (_iconsByIdCache?.has(this.iconId)) {
			span.innerHTML = _iconsByIdCache.get(this.iconId)!;
		} else {
			span.textContent = '⎔';
			loadIconsById().then((map) => {
				const svg = map.get(this.iconId);
				if (svg) span.innerHTML = svg;
			});
		}
		return span;
	}
	eq(other: IconShortcodeWidget) { return other.iconId === this.iconId; }
}

/** Widget for inline HTML tags (<u>, <sub>, <sup>) — preserves bidi with dir=auto */
class InlineHtmlWidget extends WidgetType {
	content: string;
	tag: string;
	constructor(content: string, tag: string) {
		super();
		this.content = content;
		this.tag = tag;
	}
	toDOM() {
		const el = document.createElement(this.tag === 'mark' ? 'mark' : 'span');
		el.className = this.tag === 'u' ? 'cm-html-u' : this.tag === 'sub' ? 'cm-html-sub' : this.tag === 'sup' ? 'cm-html-sup' : 'cm-html-mark';
		el.dir = 'auto';
		el.textContent = this.content;
		return el;
	}
	eq(other: InlineHtmlWidget) { return this.content === other.content && this.tag === other.tag; }
}

/** Widget for text alignment — replaces <div style="text-align:...">content</div> */
class AlignmentWidget extends WidgetType {
	content: string;
	align: string;
	constructor(content: string, align: string) {
		super();
		this.content = content;
		this.align = align;
	}
	toDOM() {
		const div = document.createElement('div');
		div.className = 'cm-md-align';
		div.style.textAlign = this.align;
		div.dir = 'auto';
		div.textContent = this.content;
		return div;
	}
	eq(other: AlignmentWidget) { return this.content === other.content && this.align === other.align; }
}

/** Widget for code block language label */
class CodeBlockLabelWidget extends WidgetType {
	lang: string;
	constructor(lang: string) {
		super();
		this.lang = lang;
	}
	toDOM() {
		const badge = document.createElement('span');
		badge.className = 'cm-md-codeblock-lang';
		badge.textContent = this.lang;
		return badge;
	}
	eq(other: CodeBlockLabelWidget) { return this.lang === other.lang; }
}

/**
 * MIG-055 §D — Widget for ` ```base ` fenced code blocks (Constellation
 * lenses). Pure-DOM implementation — matches the `UniversalEmbedWidget`
 * pattern in this file (the proven CM6-author-recommended approach for
 * async-data widgets that replace fenced content).
 *
 * ## Why pure DOM (not a Svelte mount)
 *
 * The earlier attempt mounted a Svelte 5 component into the widget's
 * wrapper via `mount(Component, { target: wrap, props })`. It failed
 * silently in production: the lens block stayed raw because Svelte 5's
 * `mount()` does NOT run effects (including `onMount`) synchronously —
 * the component renders its initial empty `loading=true` template into
 * a detached div, CM6 inserts the empty div, and `onMount` (which
 * fires `executeLens`) never reliably runs before CM6 measures the
 * widget at zero content.
 *
 * Source authority for the pure-DOM choice:
 *   - Marijn Haverbeke (CodeMirror 6 author) — explicitly advises
 *     against framework mounts in `WidgetType.toDOM()`:
 *     https://discuss.codemirror.net/t/rendering-react-components-or-similar-in-decoration-todom/3492
 *   - Svelte 5 official docs (`mount` does not flush effects):
 *     https://svelte.dev/docs/svelte/imperative-component-api
 *   - Our own `UniversalEmbedWidget` in this file — production-proven
 *     async-invoke + imperative-DOM pattern for `![[wikilink]]`
 *     transclusions; daily-used in Constellation.
 *
 * ## Lifecycle
 *
 * - `toDOM()` creates the wrapper + immediately paints a loading
 *   placeholder + kicks off `execute_lens` async. On resolution, the
 *   loading placeholder is replaced with the rendered rows. On
 *   rejection, the loading placeholder is replaced with a red error.
 * - `eq(other)` compares the YAML source — when the user edits anything
 *   else in the document, the same widget instance is reused (CM6
 *   short-circuits), no re-fetch.
 *
 * ## i18n
 *
 * UI strings route through `$lib/i18n`'s `t` store. We use
 * `get(t)('key')` (imperative store read via `svelte/store`'s `get`)
 * because we're outside a Svelte component context. If the locale
 * changes WHILE the widget is rendered, the cached strings won't
 * re-translate until the next widget re-render (acceptable for v1 —
 * editor reload picks up the new locale). The fallback English
 * literal in the `||` clause prevents undefined-key crashes.
 */
class LensBlockWidget extends WidgetType {
	constructor(public lensYaml: string) {
		super();
	}

	toDOM() {
		const wrap = document.createElement('div');
		wrap.className = 'cm-lens-block';
		this._renderLoading(wrap);
		invoke<LensResult>('execute_lens', { lensYaml: this.lensYaml })
			.then((res) => {
				wrap.innerHTML = '';
				this._renderResult(wrap, res);
			})
			.catch((err: unknown) => {
				wrap.innerHTML = '';
				const msg =
					typeof err === 'string'
						? err
						: (err as Error)?.message ?? String(err);
				this._renderError(wrap, msg);
			});
		return wrap;
	}

	eq(other: LensBlockWidget) {
		return this.lensYaml === other.lensYaml;
	}

	private _renderLoading(wrap: HTMLDivElement) {
		const el = document.createElement('div');
		el.className = 'cm-lens-loading';
		el.textContent = get(t)('lensBlock.loading') || 'Loading lens…';
		wrap.appendChild(el);
	}

	private _renderResult(wrap: HTMLDivElement, res: LensResult) {
		// ─── Header (lens name + total count) ───
		const header = document.createElement('div');
		header.className = 'cm-lens-header';

		const name = document.createElement('h3');
		name.className = 'cm-lens-name';
		name.textContent = res.lens_name;
		name.setAttribute('dir', detectDir(res.lens_name));
		header.appendChild(name);

		const count = document.createElement('span');
		count.className = 'cm-lens-count';
		count.textContent = String(res.total_count);
		header.appendChild(count);

		wrap.appendChild(header);

		// ─── Body (rows or empty state) ───
		if (res.rows.length === 0) {
			const empty = document.createElement('div');
			empty.className = 'cm-lens-empty';
			empty.textContent =
				get(t)('lensBlock.empty') || 'No notes match this lens.';
			wrap.appendChild(empty);
		} else if (res.view === 'table') {
			// MIG-065 §F — the familiar editable table (the unified Base's
			// Simple default surface).
			wrap.appendChild(this._renderTable(res));
		} else {
			const list = document.createElement('ul');
			list.className = 'cm-lens-rows';
			for (const row of res.rows) {
				list.appendChild(this._renderRow(row));
			}
			wrap.appendChild(list);
		}

		// ─── Footer (query time) ───
		const footer = document.createElement('div');
		footer.className = 'cm-lens-footer';
		const time = document.createElement('span');
		time.className = 'cm-lens-time';
		time.textContent = `${res.query_time_ms}ms`;
		footer.appendChild(time);
		wrap.appendChild(footer);
	}

	/** Dispatch the open-note event the app shell listens for (same as the
	 *  list row's name button). */
	private _openNote(row: LensRow) {
		window.dispatchEvent(
			new CustomEvent('constellation:open-note', {
				detail: {
					path: row.note_path,
					libraryName: row.library_name,
					libraryPath: row.library_path,
				},
			}),
		);
	}

	/** MIG-065 §F — render the lens as a familiar table. First column is always
	 *  the clickable note name; the rest are the lens's declared columns in
	 *  order (note.name excluded — it IS the name column). */
	private _renderTable(res: LensResult): HTMLDivElement {
		const scroll = document.createElement('div');
		scroll.className = 'cm-lens-table-scroll';
		const table = document.createElement('table');
		table.className = 'cm-lens-table';

		// MIG-065 §F.2 — column semantics live in the shared `tableModel` so this
		// inline table and the standalone `BaseTab.svelte` can never drift.
		const tl = (k: string) => get(t)(k);
		const dataCols = dataColumns(res.columns);

		const thead = document.createElement('thead');
		const htr = document.createElement('tr');
		const th0 = document.createElement('th');
		th0.textContent = get(t)('lensBlock.colName') || 'Name';
		htr.appendChild(th0);
		for (const c of dataCols) {
			const th = document.createElement('th');
			th.textContent = columnLabel(c, tl);
			th.setAttribute('dir', 'auto');
			htr.appendChild(th);
		}
		thead.appendChild(htr);
		table.appendChild(thead);

		const tbody = document.createElement('tbody');
		for (const row of res.rows) {
			const tr = document.createElement('tr');
			tr.className = 'cm-lens-trow';

			const td0 = document.createElement('td');
			td0.className = 'cm-lens-cell-name';
			// MIG-065 §F — RTL note names right-align within their cell (parity
			// with the data cells; the dir on the button alone wasn't enough —
			// the cell's text-align follows the cell's own dir).
			td0.setAttribute('dir', detectDir(row.name));
			const btn = document.createElement('button');
			btn.type = 'button';
			btn.className = 'cm-lens-row-name';
			btn.textContent = row.name;
			btn.title = row.note_path;
			btn.setAttribute('dir', detectDir(row.name));
			btn.addEventListener('click', () => this._openNote(row));
			td0.appendChild(btn);
			tr.appendChild(td0);

			for (const c of dataCols) {
				const td = document.createElement('td');
				const text = renderCellValue(row.dimensions[c], c);
				td.textContent = text;
				if (text) td.setAttribute('dir', detectDir(text));
				tr.appendChild(td);
			}
			tbody.appendChild(tr);
		}
		table.appendChild(tbody);
		scroll.appendChild(table);
		return scroll;
	}

	private _renderRow(row: LensRow): HTMLLIElement {
		const li = document.createElement('li');
		li.className = 'cm-lens-row';
		// MIG-055 §H.4 — Row direction follows the note's primary identifier
		// (its name). Per CLAUDE.md Language-First by Design + RTL Support:
		// "Use dir attributes, `detectDir()` from `$lib/utils`." Explicit
		// dir on the row removes dependence on parent/cascade inference —
		// a CSS or parent change elsewhere can't silently flip Arabic rows
		// to LTR. The previous version set `dir` only on the button and
		// the headline span; the row layout (button → dash → headline)
		// happened to render RTL via implicit cascade for Arabic content,
		// but the behavior wasn't documented in code.
		li.setAttribute('dir', detectDir(row.name));

		const btn = document.createElement('button');
		btn.type = 'button';
		btn.className = 'cm-lens-row-name';
		btn.textContent = row.name;
		btn.setAttribute('dir', detectDir(row.name));
		btn.title = row.note_path;
		btn.addEventListener('click', () => {
			// Same custom event UniversalEmbedWidget uses for ![[wikilink]]
			// transclusions — the app-shell layout listens for this and
			// opens the note in the active pane.
			window.dispatchEvent(
				new CustomEvent('constellation:open-note', {
					detail: {
						path: row.note_path,
						libraryName: row.library_name,
						libraryPath: row.library_path,
					},
				}),
			);
		});
		li.appendChild(btn);

		// The lens may or may not have requested `note.headline` as a column.
		// If it did and the value is a non-empty Text, render it after a dash.
		const headlineVal = row.dimensions['note.headline'] as
			| DimensionValue
			| undefined;
		if (typeof headlineVal === 'string' && headlineVal.length > 0) {
			const sep = document.createElement('span');
			sep.className = 'cm-lens-row-sep';
			sep.textContent = '—';
			li.appendChild(sep);

			const headline = document.createElement('span');
			headline.className = 'cm-lens-row-headline';
			headline.textContent = headlineVal;
			headline.setAttribute('dir', detectDir(headlineVal));
			li.appendChild(headline);
		}

		// MIG-060 §B — Threading-gesture buttons: 360.3D / CNS / Cataloger.
		// Architect locks: each gesture dispatches `constellation:open-note-in-surface`
		// with `detail.surface` discriminator. +layout.svelte (§C) opens the host
		// note in the active pane, then toggles the target surface flag.
		// `e.stopPropagation()` is critical: the row name button also has a click
		// handler — without stopPropagation the gesture click would double-fire.
		const actions = document.createElement('div');
		actions.className = 'cm-lens-row-actions';
		// Actions container is logically directionless — `margin-inline-start:auto`
		// in CSS pushes it to the row's trailing edge regardless of `dir`.

		// ─── 360.3D gesture — always shown ───
		const btn360 = document.createElement('button');
		btn360.type = 'button';
		btn360.className = 'cm-lens-row-action cm-lens-row-action-360';
		btn360.innerHTML =
			'<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="3"/><line x1="12" y1="3" x2="12" y2="9"/><line x1="12" y1="15" x2="12" y2="21"/><line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/></svg>';
		btn360.title =
			get(t)('lensBlock.openIn360Tooltip') || 'Open in 360.3D';
		btn360.setAttribute('aria-label', btn360.title);
		btn360.addEventListener('click', (e) => {
			e.stopPropagation();
			window.dispatchEvent(
				new CustomEvent('constellation:open-note-in-surface', {
					detail: {
						surface: '360.3d',
						path: row.note_path,
						libraryName: row.library_name,
						libraryPath: row.library_path,
					},
				}),
			);
		});
		actions.appendChild(btn360);

		// ─── CNS gesture — gated by (1) the user-settings flag AND
		// (2) orphan check (MIG-060 §C-fix-2).
		// (1) Architect Q4 / lock §D: hide entirely when CNS is disabled.
		// (2) Orphan check: CNS shows the linked subgraph only; a note
		//     not in `skyNodePathSet` has no SimNode to focus on, so the
		//     gesture would silently no-op. Hide the icon for orphans.
		//     Edge case — when `skyNodePathSet` is still empty (boot not
		//     finished), permissively show the icon; the §C-fix listener
		//     handles a no-match case gracefully (default fit-to-screen).
		const cnsEnabled =
			get(appSettings).enabledFeatures?.constellationSight !== false;
		const skySet = get(skyNodePathSet);
		const inGraphOrBooting =
			skySet.size === 0 || skySet.has(row.note_path);
		if (cnsEnabled && inGraphOrBooting) {
			const btnCns = document.createElement('button');
			btnCns.type = 'button';
			btnCns.className =
				'cm-lens-row-action cm-lens-row-action-cns';
			btnCns.innerHTML =
				'<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>';
			btnCns.title =
				get(t)('lensBlock.openInCNSTooltip') || 'Open in CNS';
			btnCns.setAttribute('aria-label', btnCns.title);
			btnCns.addEventListener('click', (e) => {
				e.stopPropagation();
				window.dispatchEvent(
					new CustomEvent('constellation:open-note-in-surface', {
						detail: {
							surface: 'cns',
							path: row.note_path,
							libraryName: row.library_name,
							libraryPath: row.library_path,
						},
					}),
				);
			});
			actions.appendChild(btnCns);
		}

		// ─── Cataloger gesture — always shown ───
		const btnCat = document.createElement('button');
		btnCat.type = 'button';
		btnCat.className =
			'cm-lens-row-action cm-lens-row-action-cataloger';
		btnCat.innerHTML =
			'<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>';
		btnCat.title =
			get(t)('lensBlock.openInCatalogerTooltip') ||
			'Open in The Cataloger';
		btnCat.setAttribute('aria-label', btnCat.title);
		btnCat.addEventListener('click', (e) => {
			e.stopPropagation();
			window.dispatchEvent(
				new CustomEvent('constellation:open-note-in-surface', {
					detail: {
						surface: 'cataloger',
						path: row.note_path,
						libraryName: row.library_name,
						libraryPath: row.library_path,
					},
				}),
			);
		});
		actions.appendChild(btnCat);

		li.appendChild(actions);

		return li;
	}

	private _renderError(wrap: HTMLDivElement, msg: string) {
		const el = document.createElement('div');
		el.className = 'cm-lens-error';

		const label = document.createElement('span');
		label.className = 'cm-lens-error-label';
		label.textContent =
			(get(t)('lensBlock.errorLabel') || 'Lens error') + ':';
		el.appendChild(label);

		const msgEl = document.createElement('span');
		msgEl.className = 'cm-lens-error-msg';
		msgEl.textContent = msg;
		el.appendChild(msgEl);

		wrap.appendChild(el);
	}
}

/**
 * MIG-055 §H.3 — Provide ` ```base ` block-replace decorations via a
 * StateField, NOT through `livePreviewPlugin` (which is a ViewPlugin).
 *
 * ## Why a StateField is mandatory
 *
 * CM6's source code rejects multi-line block-replace decorations
 * sourced from a ViewPlugin. From `view/dist/index.js:2719-2723`:
 *
 *   if (this.disallowBlockEffectsFor[index]) {       // true for ViewPlugin
 *     if (deco.block)
 *       throw new RangeError("Block decorations may not be specified via plugins");
 *     if (to > this.view.state.doc.lineAt(from).to)  // crosses line break
 *       throw new RangeError("Decorations that replace line breaks may not be specified via plugins");
 *   }
 *
 * Our `Decoration.replace({ from: node.from, to: node.to })` for a
 * fenced YAML block spans multiple lines — crosses line breaks — so
 * the second branch fires. The throw is raised inside a `requestAnimationFrame`
 * callback and silently swallowed in release builds (no devtools).
 * Symptom: the widget never appears, raw YAML stays visible.
 *
 * StateField-provided decorations bypass `disallowBlockEffectsFor`.
 * Same widget, same visual contract, different provider — legal.
 *
 * ## Lifecycle
 *
 * - `create(state)` — iterate the syntax tree on first mount; build
 *   decorations for every ` ```base ` block in the document.
 * - `update(value, tr)` — rebuild ONLY on `tr.docChanged` (selection
 *   changes don't affect block decorations in v1). Returning the same
 *   `value` for selection-only transactions makes the StateField
 *   effectively free for cursor moves.
 * - `provide: f => EditorView.decorations.from(f)` — register the
 *   StateField's value as a decoration source. CM6 then includes our
 *   decorations in its render pipeline WITHOUT the ViewPlugin gate.
 *
 * ## Why we always render (no cursor-aware reveal in v1)
 *
 * The existing dataview-block pattern in this file hides its widget
 * when the cursor is inside the block (so the user can edit the
 * query). That pattern is also broken per the CM6-source authority
 * (a ViewPlugin can't provide block-replace decorations either way).
 * v1 of the lens block always renders the widget; to edit the YAML,
 * toggle livePreview off via the editor's "Source mode" menu
 * (NotePane already has this toggle). Future enhancement: a
 * selection-aware variant that does NOT throw — see the dataview
 * issue ticket for that broader migration.
 */
export const baseLensField = StateField.define<DecorationSet>({
	create(state) {
		return buildBaseLensDecorations(state);
	},
	update(value, tr) {
		if (tr.docChanged) return buildBaseLensDecorations(tr.state);
		return value;
	},
	provide: (f) => EditorView.decorations.from(f),
});

function buildBaseLensDecorations(state: EditorState): DecorationSet {
	const builder = new RangeSetBuilder<Decoration>();
	const doc = state.doc;
	syntaxTree(state).iterate({
		enter(node) {
			if (node.name !== 'FencedCode') return;
			const firstLine = doc.lineAt(node.from);
			const info = firstLine.text.trim();
			if (!/^```+\s*base\s*$/i.test(info)) return;
			// Slice the YAML payload between the opening + closing fences.
			const innerFrom = firstLine.to + 1;
			const lastLine = doc.lineAt(node.to);
			const innerTo = lastLine.text.trim().startsWith('```')
				? lastLine.from
				: node.to;
			const yamlText =
				innerTo > innerFrom ? doc.sliceString(innerFrom, innerTo) : '';
			builder.add(
				node.from,
				node.to,
				Decoration.replace({ widget: new LensBlockWidget(yamlText) }),
			);
		},
	});
	return builder.finish();
}

/** Widget shown for dataview code blocks when cursor is outside */
class DataviewLabelWidget extends WidgetType {
	query: string;
	constructor(query: string) {
		super();
		this.query = query;
	}
	toDOM() {
		const wrap = document.createElement('div');
		wrap.className = 'cm-dv-label-widget';
		const badge = document.createElement('span');
		badge.className = 'cm-dv-badge';
		badge.textContent = 'Dataview';
		wrap.appendChild(badge);
		const preview = document.createElement('code');
		preview.className = 'cm-dv-query-preview';
		preview.textContent = this.query.length > 80 ? this.query.slice(0, 80) + '…' : this.query;
		wrap.appendChild(preview);
		return wrap;
	}
	eq(other: DataviewLabelWidget) { return this.query === other.query; }
}

function buildDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	// §E.2 — labels above typed links read in the NOTE's own language (detected from
	// its content), independent of the UI language. Sampled once per rebuild (cheap).
	const noteLoc = dominantLocale(doc.sliceString(0, Math.min(doc.length, 2000)));
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const libPath  = view.state.field(libraryPathField, false) || '';
	const notePath = view.state.field(notePathField,    false) || '';
	const traversalMap = view.state.field(linkTraversalMapField, false) ?? new Map<string, number>();
	const notePathLower = notePath.toLowerCase();
	const ranges: { from: number; to: number; deco: Decoration }[] = [];

	// Process only visible ranges for performance
	for (const { from, to } of view.visibleRanges) {
		syntaxTree(view.state).iterate({
			from, to,
			enter(node) {
				const nodeLine = doc.lineAt(node.from).number;
				const nodeEndLine = doc.lineAt(node.to).number;
				const onCursorLine = nodeLine === cursorLine;
				const cursorInBlock = cursorLine >= nodeLine && cursorLine <= nodeEndLine;

				// ATX Headings (# through ######)
				if (node.name.startsWith('ATXHeading') && node.name.length === 11) {
					const level = parseInt(node.name[10]) - 1;
					if (level >= 0 && level < 6) {
						ranges.push({ from: node.from, to: node.to, deco: headingDecos[level] });
					}
				}

				// Hide heading markers (# characters) when cursor is not on that line
				if (node.name === 'HeaderMark' && !onCursorLine) {
					const end = Math.min(node.to + 1, doc.lineAt(node.from).to);
					ranges.push({ from: node.from, to: end, deco: replaceDeco });
				}

				// Strong emphasis (bold)
				if (node.name === 'StrongEmphasis') {
					ranges.push({ from: node.from, to: node.to, deco: boldDeco });
				}

				// Emphasis (italic)
				if (node.name === 'Emphasis') {
					ranges.push({ from: node.from, to: node.to, deco: italicDeco });
				}

				// Hide emphasis markers when not on cursor line
				if (node.name === 'EmphasisMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
				}

				// Strikethrough
				if (node.name === 'Strikethrough') {
					ranges.push({ from: node.from, to: node.to, deco: strikeDeco });
				}
				if (node.name === 'StrikethroughMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
				}

				// Inline code
				if (node.name === 'InlineCode') {
					ranges.push({ from: node.from, to: node.to, deco: codeDeco });
				}
				if (node.name === 'CodeMark' && !onCursorLine) {
					const text = doc.sliceString(node.from, node.to);
					if (text === '`') {
						ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
					}
				}

				// Links
				if (node.name === 'Link' || node.name === 'URL') {
					ranges.push({ from: node.from, to: node.to, deco: linkDeco });
				}

				// Blockquote — muted color on the entire block (text + marker)
				if (node.name === 'Blockquote') {
					ranges.push({ from: node.from, to: node.to, deco: blockquoteDeco });
				}

				// Horizontal rule
				if (node.name === 'HorizontalRule') {
					ranges.push({ from: node.from, to: node.to, deco: hrDeco });
				}

				// Task list checkboxes
				if (node.name === 'TaskMarker') {
					const text = doc.sliceString(node.from, node.to);
					const checked = text.includes('x') || text.includes('X');
					if (!onCursorLine) {
						ranges.push({ from: node.from, to: node.to,
							deco: checked ? checkboxCheckedDeco : checkboxUncheckedDeco });
					}
				}

				// Highlight (==text==)
				if (node.name === 'Highlight') {
					ranges.push({ from: node.from, to: node.to, deco: highlightDeco });
				}
				if (node.name === 'HighlightMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
				}

				// Fenced code blocks
				if (node.name === 'FencedCode') {
					const firstLine = doc.lineAt(node.from);
					const info = firstLine.text.trim();

					if (!cursorInBlock) {
						// Dataview — show label widget
						if (/^```+\s*dataview\s*$/i.test(info)) {
							const innerFrom = firstLine.to + 1;
							const lastLine = doc.lineAt(node.to);
							const innerTo = lastLine.text.trim().startsWith('```') ? lastLine.from : node.to;
							const queryText = innerTo > innerFrom ? doc.sliceString(innerFrom, innerTo).trim() : '';
							ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
								widget: new DataviewLabelWidget(queryText),
							}) });
						}
						// MIG-055 §H.3 — `base` lens blocks are NO LONGER handled here.
						// CM6's `view/dist/index.js:2719-2723` rejects multi-line
						// block-replace decorations sourced from a ViewPlugin (which
						// `livePreviewPlugin` is). The throw fires in rAF and is
						// silently swallowed in release builds. The lens-block
						// decoration is now provided by the `baseLensField`
						// StateField defined below — see its docstring for the
						// CM6-source authority chain.
					}

					// Language label for non-dataview / non-base code blocks
					if (!cursorInBlock) {
						const langMatch = info.match(/^```+\s*(\S+)/);
						if (langMatch && !/^dataview$/i.test(langMatch[1]) && !/^base$/i.test(langMatch[1])) {
							ranges.push({ from: firstLine.to, to: firstLine.to, deco: Decoration.widget({
								widget: new CodeBlockLabelWidget(langMatch[1]),
								side: 1,
							}) });
						}
					}
				}

				// Inline images: ![[file.png]] or ![alt](url)
				if (node.name === 'Image' && !onCursorLine) {
					const text = doc.sliceString(node.from, node.to);
					// Standard markdown: ![alt](url)
					const mdMatch = text.match(/^!\[([^\]]*)\]\(([^)]+)\)/);
					if (mdMatch) {
						const url = mdMatch[2];
						if (/^https?:\/\/|^data:/.test(url)) {
							// Absolute URL — use directly (no Rust resolution needed)
							ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
								widget: new ImageWidget(url, mdMatch[1], '', ''),
							}) });
						} else {
							ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
								widget: new ImageWidget(url, mdMatch[1], libPath, notePath),
							}) });
						}
					}
				}
			}
		});
	}

	// Single-pass line scan for wikilink embeds, wikilinks, and tags
	// (Merged from 3 separate loops to reduce iteration overhead)
	for (const { from: vFrom, to: vTo } of view.visibleRanges) {
		for (let pos = vFrom; pos < vTo;) {
			const line = doc.lineAt(pos);
			if (line.number !== cursorLine) {
				const lineText = line.text;

				// Icon shortcodes: `:lucide-heart:`, `:phosphor-book:`, etc.
				// Renders inline as the corresponding SVG via the lazy iconSets
				// cache. Scoped to known set prefixes so ordinary `:note:` or
				// timestamps like `10:30:` don't accidentally match.
				const iconShortRe = /:(lucide|phosphor|hi|feather)-([a-z0-9][a-z0-9-]*):/g;
				let iconMatch;
				while ((iconMatch = iconShortRe.exec(lineText)) !== null) {
					const setName = iconMatch[1];
					const name = iconMatch[2];
					const iconId = `${setName}:${name}`;
					const absFrom = line.from + iconMatch.index;
					ranges.push({ from: absFrom, to: absFrom + iconMatch[0].length, deco: Decoration.replace({
						widget: new IconShortcodeWidget(iconId),
					}) });
				}

				// Universal embeds: ![[target]] — Rust resolves the type and returns
				// an EmbedResolution the UniversalEmbedWidget routes to the right
				// renderer (image / audio / video / PDF / canvas / excalidraw /
				// note-transclusion / generic / missing).
				const embedRe = /!\[\[([^\]]+)\]\]/g;
				let m;
				while ((m = embedRe.exec(lineText)) !== null) {
					const inner = m[1];
					const pipeIdx = inner.indexOf('|');
					const rawTarget = pipeIdx >= 0 ? inner.slice(0, pipeIdx) : inner;
					const alias = pipeIdx >= 0 ? inner.slice(pipeIdx + 1) : '';
					const absFrom = line.from + m.index;
					ranges.push({ from: absFrom, to: absFrom + m[0].length, deco: Decoration.replace({
						widget: new UniversalEmbedWidget(rawTarget, alias, libPath, notePath),
					}) });
				}

				// Wikilinks: [[note]], [[note|display]], [[note|link-type]] (typed),
				// or [[note|display|link-type]] (typed with explicit alias).
				const wikiRe = /(?<!!)\[\[([^\]]+)\]\]/g;
				while ((m = wikiRe.exec(lineText)) !== null) {
					const absFrom = line.from + m.index;
					const absTo = absFrom + m[0].length;
					const innerFrom = absFrom + 2;
					const innerTo = absTo - 2;
					const raw = m[1];
					const pipeIndex = raw.indexOf('|');
					// Predicate-FIRST (canonical): [[type::target]] / [[type::target|display]].
					const colonIdx = raw.indexOf('::');
					const firstType = (colonIdx > 0 && isLinkTypeValue(raw.slice(0, colonIdx).trim().toLowerCase()))
						? raw.slice(0, colonIdx).trim().toLowerCase() : '';
					// Target = first segment, after any `type::` prefix (for the traversal chip).
					const linkTarget = (firstType
						? raw.slice(colonIdx + 2).split('|')[0]
						: (pipeIndex >= 0 ? raw.slice(0, pipeIndex) : raw)
					).trim().toLowerCase();
					if (firstType) {
						const fdeco = typeDeco(firstType, noteLoc);
						const rest = raw.slice(colonIdx + 2);
						const restPipe = rest.indexOf('|');
						if (restPipe >= 0) {
							// [[type::target|display]] — show display in the type color; hide [[type::target| and ]].
							const dispFrom = innerFrom + colonIdx + 2 + restPipe + 1;
							ranges.push({ from: absFrom, to: dispFrom, deco: replaceDeco });
							ranges.push({ from: dispFrom, to: innerTo, deco: fdeco });
							ranges.push({ from: innerTo, to: absTo, deco: replaceDeco });
						} else {
							// [[type::target]] — show target in the type color; hide [[type:: and ]].
							const tgtFrom = innerFrom + colonIdx + 2;
							ranges.push({ from: absFrom, to: tgtFrom, deco: replaceDeco });
							ranges.push({ from: tgtFrom, to: innerTo, deco: fdeco });
							ranges.push({ from: innerTo, to: absTo, deco: replaceDeco });
						}
					} else if (pipeIndex >= 0) {
						// Recognize a typed annotation only when it sits after the LAST
						// pipe. This covers:
						//   [[note|type]]            — 2-part typed (pipeIndex === lastPipe)
						//   [[note|alias|type]]      — 3-part typed with explicit alias
						// Without the lastIndexOf-based parse, the old code took the
						// naive `slice(firstPipe + 1)` which for 3-part links produced
						// an `afterPipe` like `alias|type` that never matched
						// TYPED_LINK_TYPES — so the "|type]]" trailer leaked into the
						// rendered alias text (the reported bug).
						const lastPipeIndex = raw.lastIndexOf('|');
						const afterLastPipe = raw.slice(lastPipeIndex + 1).trim().toLowerCase();
						const isTyped = lastPipeIndex > 0 && isLinkTypeValue(afterLastPipe);
						if (isTyped && lastPipeIndex === pipeIndex) {
							// 2-part typed: [[note|type]]. Show the note name in the
							// type color; hide [[ and |type]].
							const noteEnd = innerFrom + pipeIndex;
							ranges.push({ from: absFrom, to: innerFrom, deco: replaceDeco }); // hide [[
							ranges.push({ from: innerFrom, to: noteEnd, deco: typeDeco(afterLastPipe, noteLoc) });
							ranges.push({ from: noteEnd, to: absTo, deco: replaceDeco }); // hide |type]]
						} else if (isTyped) {
							// 3-part typed: [[note|alias|type]]. Show the alias in the
							// type color; hide [[note| and |type]].
							const aliasStart = innerFrom + pipeIndex + 1;
							const aliasEnd = innerFrom + lastPipeIndex;
							ranges.push({ from: absFrom, to: aliasStart, deco: replaceDeco }); // hide [[note|
							ranges.push({ from: aliasStart, to: aliasEnd, deco: typeDeco(afterLastPipe, noteLoc) });
							ranges.push({ from: aliasEnd, to: absTo, deco: replaceDeco }); // hide |type]]
						} else {
							// Display alias: [[note|alias]] (alias may contain pipes).
							const displayFrom = innerFrom + pipeIndex + 1;
							ranges.push({ from: absFrom, to: displayFrom, deco: replaceDeco });
							ranges.push({ from: displayFrom, to: innerTo, deco: linkDeco });
							ranges.push({ from: innerTo, to: absTo, deco: replaceDeco });
						}
					} else {
						ranges.push({ from: absFrom, to: innerFrom, deco: replaceDeco });
						ranges.push({ from: innerFrom, to: innerTo, deco: linkDeco });
						ranges.push({ from: innerTo, to: absTo, deco: replaceDeco });
					}

					// P4.2: Living Link traversal chip. Look up the count for
					// this (sourceNote, target) pair and emit a `×N` widget
					// immediately after the wikilink when it has been walked.
					// Target is everything before the first pipe (untyped case)
					// or the segment before the typed annotation — which are
					// the same bytes, since the target is always the first
					// `|`-delimited segment.
					if (notePathLower && traversalMap.size > 0) {
						const targetName = linkTarget;
						if (targetName) {
							const count = traversalMap.get(notePathLower + '|' + targetName);
							if (count && count > 0) {
								ranges.push({
									from: absTo, to: absTo,
									deco: Decoration.widget({
										widget: new WikilinkTraversalChipWidget(count),
										side: 1,
									}),
								});
							}
						}
					}
				}

				// Inline HTML: <u>...</u>, <sub>...</sub>, <sup>...</sup>
				// Use Decoration.mark (not replace) to preserve bidi text flow within the line.
				const htmlInlineRe = /<(u|sub|sup|mark)>(.*?)<\/\1>/gi;
				while ((m = htmlInlineRe.exec(lineText)) !== null) {
					const tag = m[1].toLowerCase();
					const absFrom = line.from + m.index;
					const openEnd = absFrom + tag.length + 2; // after <tag>
					const closeStart = absFrom + m[0].length - tag.length - 3; // before </tag>
					const absTo = absFrom + m[0].length;
					ranges.push({ from: absFrom, to: openEnd, deco: htmlHiddenDeco }); // hide <tag>
					ranges.push({ from: openEnd, to: closeStart, deco: htmlDecoMap[tag] ?? htmlUDeco }); // style content (pre-cached)
					ranges.push({ from: closeStart, to: absTo, deco: htmlHiddenDeco }); // hide </tag>
				}

				// Alignment divs: <div style="text-align: center">...</div>
				const alignRe = /^<div style="text-align:\s*(left|center|right)">(.*)<\/div>$/;
				const alignMatch = lineText.match(alignRe);
				if (alignMatch) {
					ranges.push({ from: line.from, to: line.to, deco: Decoration.replace({
						widget: new AlignmentWidget(alignMatch[2], alignMatch[1]),
					}) });
				}

				// Tags: #tag-name
				const tagRe = /(?:^|\s)(#[a-zA-Z\u0600-\u06FF][\w\u0600-\u06FF/-]*)/g;
				while ((m = tagRe.exec(lineText)) !== null) {
					const tagStart = line.from + m.index + (m[0].length - m[1].length);
					ranges.push({ from: tagStart, to: tagStart + m[1].length, deco: tagDeco });
				}
			}
			pos = line.to + 1;
		}
	}

	// Sort by from position, then by length (shorter ranges first for proper nesting)
	ranges.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const r of ranges) {
		builder.add(r.from, r.to, r.deco);
	}
	return builder.finish();
}

// The ViewPlugin class
/** Dispatched when the link-type vocabulary changes (recolour / rename / add / remove in
 *  the §G editor) so live-preview decorations rebuild LIVE in the editor — matching the
 *  panels — instead of only on note reopen. */
const linkVocabChanged = StateEffect.define<null>();

class LivePreviewPlugin {
	decorations: DecorationSet;
	private rebuildTimer: ReturnType<typeof setTimeout> | null = null;
	private lastCursorLine = -1;
	private view: EditorView;
	private unsubVocab: () => void;

	constructor(view: EditorView) {
		this.view = view;
		this.decorations = buildDecorations(view);
		this.lastCursorLine = view.state.doc.lineAt(view.state.selection.main.head).number;
		// Rebuild on a vocabulary change (recolour / rename / add / remove) so typed-link
		// colours + labels update LIVE here, matching the panels — not on reopen.
		this.unsubVocab = subscribeLinkTypes(() => {
			// Defer to a microtask so we never dispatch DURING an in-progress editor update
			// (re-entrancy) — e.g. when applying a Style changes appSettings + the registry
			// together; a synchronous dispatch there would be swallowed and the colours
			// wouldn't refresh until reopen.
			queueMicrotask(() => { try { this.view.dispatch({ effects: linkVocabChanged.of(null) }); } catch { /* view gone */ } });
		});
	}

	update(update: ViewUpdate) {
		// Detect transactions carrying our context-state effects (library path,
		// note path, attachment folder, or the P4.2 traversal map) — none of
		// these trigger viewportChanged/selectionSet/docChanged on their own,
		// so without this branch the decorations would not rebuild and the
		// chips / image widgets would stay stale.
		const contextChanged = update.transactions.some(tr =>
			tr.effects.some(e =>
				e.is(setLinkTraversalMap) || e.is(setNotePath) ||
				e.is(setLibraryPath) || e.is(setAttachmentFolder) || e.is(linkVocabChanged)
			)
		);
		if (contextChanged) {
			if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
			this.decorations = buildDecorations(update.view);
			return;
		}

		if (update.viewportChanged) {
			// Scroll — always rebuild, clear any pending debounce
			if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
			this.decorations = buildDecorations(update.view);
			this.lastCursorLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			return;
		}

		if (update.selectionSet && !update.docChanged) {
			// Cursor moved — live preview markers are LINE-based (hide when cursor leaves the line),
			// so rebuilding on same-line cursor moves (word nav, char nav) is redundant and causes
			// perceptible lag. Only rebuild when the cursor crosses a line boundary.
			const newLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			if (newLine !== this.lastCursorLine) {
				this.lastCursorLine = newLine;
				if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
				this.decorations = buildDecorations(update.view);
			}
			return;
		}

		if (update.docChanged) {
			// ⚡ Fast path: map existing decorations — O(changes), ~0ms, keeps typing instant
			this.decorations = this.decorations.map(update.changes);
			// Debounced full rebuild — no view.dispatch (fresh decos apply on next natural update)
			if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
			const view = update.view;
			this.rebuildTimer = setTimeout(() => {
				this.rebuildTimer = null;
				if (!(view as any).destroyed) {
					this.decorations = buildDecorations(view);
				}
			}, 300);
		}
	}

	destroy() {
		if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
		this.unsubVocab();
	}
}

/** The live preview extension — add to CM extensions to enable */
export const livePreviewPlugin = ViewPlugin.fromClass(LivePreviewPlugin, {
	decorations: (v) => v.decorations,
});

/** Theme for live preview decorations */
export const livePreviewTheme = EditorView.theme({
	// MIG-070 §3 — heading SIZE lives here (the highlight sets no font-size). COLOUR + WEIGHT
	// are owned by NotePane's `markdownHighlightStyle` (reading --hN-color / --heading-weight),
	// because that HighlightStyle is applied to the heading token and wins over a theme rule here.
	'.cm-md-heading1': { fontSize: 'var(--h1-size, 1.8em)', lineHeight: 'var(--line-height-tight, 1.3)' },
	'.cm-md-heading2': { fontSize: 'var(--h2-size, 1.5em)', lineHeight: 'var(--line-height-tight, 1.3)' },
	'.cm-md-heading3': { fontSize: 'var(--h3-size, 1.25em)', lineHeight: 'var(--line-height-tight, 1.3)' },
	'.cm-md-heading4': { fontSize: 'var(--h4-size, 1.1em)', lineHeight: 'var(--line-height-tight, 1.3)' },
	'.cm-md-heading5': { fontSize: 'var(--h5-size, 1.0em)', lineHeight: 'var(--line-height-tight, 1.3)' },
	'.cm-md-heading6': { fontSize: 'var(--h6-size, 0.95em)', lineHeight: 'var(--line-height-tight, 1.3)' },
	// MIG-070 §3 — bold WEIGHT is owned here (the highlight `strong` sets no weight); bold/italic/
	// strikethrough COLOUR is owned by the highlight (it wins) — see markdownHighlightStyle.
	'.cm-md-bold': {
		fontWeight: 'var(--bold-weight, 700)',
	},
	'.cm-md-italic': {
		fontStyle: 'italic',
	},
	'.cm-md-strikethrough': {
		textDecoration: 'line-through',
		opacity: '0.6',
	},
	'.cm-md-code': {
		fontFamily: 'var(--font-monospace-theme)',
		fontSize: 'var(--font-monospace-size, 1em)',
		backgroundColor: 'var(--code-background, var(--background-primary-alt))',
		color: 'var(--code-normal, inherit)',
		borderRadius: 'var(--radius-s, 3px)',
		padding: '1px 4px',
	},
	'.cm-md-link': {
		color: 'var(--link-color, var(--library-accent, var(--interactive-accent)))',
		textDecoration: 'var(--link-decoration, underline)',
		textDecorationColor: 'color-mix(in srgb, var(--link-color, var(--library-accent, var(--interactive-accent))) 40%, transparent)',
		cursor: 'pointer',
	},
	'.cm-md-link:hover': {
		color: 'var(--link-color-hover, var(--link-color, var(--interactive-accent-hover)))',
	},
	// MIG-067 §E — typed-link colour, painted from the inline `--ltc` custom property
	// (set by typeDeco from the registry) onto the link AND any nested text inside it.
	// The `.cm-ltyped *` descendant rule is the crux: the wikilink target renders in a
	// CHILD element whose own (standard link) colour otherwise wins the visible text —
	// an element-level override only reached the underline (the reported "underline
	// coloured, text still blue" bug). `!important` beats the standard `.cm-md-link`
	// colour; reading `--ltc` means recolouring any of the 8 in §G reflects here too.
	'.cm-ltyped':   { color: 'var(--ltc) !important', textDecorationColor: 'color-mix(in srgb, var(--ltc) 40%, transparent) !important' },
	'.cm-ltyped *': { color: 'var(--ltc) !important' },
	// MIG-067 §E.2 — "Show type name above link" (the `cm-lt-labels` content class,
	// driven by the showTypedLinkLabels setting). Each RENDERED typed link gets its
	// localized type name as a small label riding just above it, in the type colour,
	// so the relationship is unmistakable — no need to recall what a colour means.
	// In label mode the editor LEADING is widened (line-height) so every label has room
	// above its OWN visual row — ruby/furigana-style. (padding-top was wrong: a CM
	// `.cm-line` is one LOGICAL line that wraps into many visual rows, so padding only
	// spaced each paragraph's first row and mid-paragraph labels still grazed the row
	// above.) Scoped to `.cm-lt-labels`, so non-label notes keep the tight 1.75 leading.
	// (The cursor line shows raw `[[type::target]]` with no `.cm-ltyped`, so no label.)
	'.cm-lt-labels .cm-line': { lineHeight: '2.3' },
	'.cm-lt-labels .cm-ltyped': { position: 'relative' },
	'.cm-lt-labels .cm-ltyped::before': {
		content: 'attr(data-ltype)',
		position: 'absolute',
		insetInlineStart: '0',
		bottom: 'calc(100% - 2px)',
		fontSize: '0.62em',
		fontWeight: '600',
		lineHeight: '1',
		letterSpacing: '0.01em',
		color: 'var(--ltc)',
		whiteSpace: 'nowrap',
		pointerEvents: 'none',
		opacity: '0.85',
	},
	// MIG-067 §E.2 — "Colour links by type" OFF (the `cm-lt-plain` content class):
	// typed links revert to the standard wikilink colour (text, underline, and the
	// label), so the type is carried by the LABEL alone. Two classes + !important
	// out-rank the single-class `.cm-ltyped` colour rules above.
	'.cm-lt-plain .cm-ltyped, .cm-lt-plain .cm-ltyped *': {
		color: 'var(--link-color, var(--library-accent, var(--interactive-accent))) !important',
	},
	'.cm-lt-plain .cm-ltyped': {
		textDecorationColor: 'color-mix(in srgb, var(--link-color, var(--library-accent, var(--interactive-accent))) 40%, transparent) !important',
	},
	'.cm-lt-plain .cm-ltyped::before': {
		color: 'var(--link-color, var(--library-accent, var(--interactive-accent))) !important',
	},
	'.cm-md-align':  { display: 'block', width: '100%' },
	'.cm-html-hidden': { fontSize: '0', lineHeight: '0', overflow: 'hidden', display: 'inline', width: '0' },
	'.cm-html-u':    { textDecoration: 'underline' },
	'.cm-html-sub':  { fontSize: '0.75em', verticalAlign: 'sub' },
	'.cm-html-sup':  { fontSize: '0.75em', verticalAlign: 'super' },
	'.cm-html-mark': { backgroundColor: '#fef08a', borderRadius: '2px' },
	'.cm-md-highlight': {
		backgroundColor: 'color-mix(in srgb, var(--color-yellow) 35%, transparent)',
		borderRadius: '2px',
		padding: '1px 0',
	},
	'.cm-md-hr': {
		display: 'block',
		textAlign: 'center',
		color: 'var(--background-modifier-border)',
	},
	'.cm-md-blockquote': {
		// MIG-070 §3 — blockquote text colour, falling back to muted (today's look).
		// The coloured LEFT BAR for plain `>` quotes is a line-decoration change and
		// lands with the §3C rendering work (callouts already own their bar).
		color: 'var(--blockquote-text-color, var(--text-muted))',
	},
	'.cm-md-tag': {
		color: 'var(--library-accent, var(--interactive-accent))',
		backgroundColor: 'color-mix(in srgb, var(--library-accent, var(--interactive-accent)) 10%, transparent)',
		borderRadius: '3px',
		padding: '1px 4px',
		fontSize: '0.9em',
	},
	'.cm-md-checkbox': {
		verticalAlign: 'middle',
		marginInlineEnd: '4px',
		cursor: 'pointer',
		accentColor: 'var(--library-accent, var(--interactive-accent))',
	},
	'.cm-dv-label-widget': {
		display: 'flex',
		alignItems: 'center',
		gap: '8px',
		padding: '6px 10px',
		margin: '4px 0',
		border: '1px solid var(--background-modifier-border)',
		borderRadius: '6px',
		background: 'var(--background-secondary)',
		cursor: 'pointer',
		userSelect: 'none',
	},
	'.cm-dv-badge': {
		fontSize: '11px',
		fontWeight: '600',
		color: 'var(--interactive-accent)',
		textTransform: 'uppercase',
		letterSpacing: '0.5px',
		flexShrink: '0',
	},
	'.cm-dv-query-preview': {
		fontSize: '11px',
		color: 'var(--text-muted)',
		overflow: 'hidden',
		textOverflow: 'ellipsis',
		whiteSpace: 'nowrap',
		background: 'none',
		padding: '0',
		fontFamily: 'var(--font-monospace-theme)',
	},
	// ─── MIG-055 §D — Lens block widget (Constellation Base renderer) ───
	'.cm-lens-block': {
		display: 'flex',
		flexDirection: 'column',
		border: '1px solid var(--background-modifier-border)',
		borderRadius: '8px',
		background: 'var(--background-secondary)',
		padding: '10px 14px',
		margin: '8px 0',
		fontSize: '0.9em',
	},
	// ─── MIG-065 §F — familiar table view ───
	'.cm-lens-table-scroll': {
		overflowX: 'auto',
		maxWidth: '100%',
	},
	'.cm-lens-table': {
		borderCollapse: 'collapse',
		width: '100%',
		fontSize: '0.92em',
	},
	'.cm-lens-table th': {
		textAlign: 'start',
		padding: '4px 10px 6px',
		borderBottom: '1px solid var(--background-modifier-border)',
		color: 'var(--text-muted)',
		fontWeight: '600',
		whiteSpace: 'nowrap',
	},
	'.cm-lens-table td': {
		padding: '4px 10px',
		borderBottom: '1px solid var(--background-modifier-border-hover, var(--background-modifier-border))',
		verticalAlign: 'top',
	},
	'.cm-lens-trow:hover': {
		background: 'var(--background-modifier-hover)',
	},
	'.cm-lens-cell-name .cm-lens-row-name': {
		fontWeight: '500',
	},
	'.cm-lens-loading': {
		color: 'var(--text-muted)',
		fontSize: '0.85em',
		padding: '4px 0',
	},
	'.cm-lens-error': {
		color: 'var(--text-error, #e53e3e)',
		fontSize: '0.85em',
		padding: '4px 0',
		display: 'flex',
		gap: '6px',
		flexWrap: 'wrap',
	},
	'.cm-lens-error-label': { fontWeight: '600' },
	'.cm-lens-error-msg': {
		fontFamily: 'var(--font-monospace)',
		whiteSpace: 'pre-wrap',
	},
	'.cm-lens-header': {
		display: 'flex',
		alignItems: 'baseline',
		justifyContent: 'space-between',
		gap: '8px',
		marginBottom: '8px',
		paddingBottom: '6px',
		borderBottom: '1px solid var(--background-modifier-border)',
	},
	'.cm-lens-name': {
		margin: '0',
		fontSize: '0.95em',
		fontWeight: '600',
		color: 'var(--text-normal)',
	},
	'.cm-lens-count': {
		fontSize: '0.75em',
		fontWeight: '600',
		color: '#fff',
		background: 'var(--interactive-accent, var(--library-accent, #6c5ce7))',
		padding: '1px 8px',
		borderRadius: '10px',
	},
	'.cm-lens-empty': {
		color: 'var(--text-muted)',
		fontStyle: 'italic',
		fontSize: '0.85em',
		padding: '6px 0',
		textAlign: 'center',
	},
	'.cm-lens-rows': {
		listStyle: 'none',
		padding: '0',
		margin: '0',
		display: 'flex',
		flexDirection: 'column',
		gap: '2px',
	},
	'.cm-lens-row': {
		display: 'flex',
		alignItems: 'baseline',
		gap: '6px',
		padding: '3px 0',
		flexWrap: 'wrap',
		borderBottom: '1px dotted transparent',
	},
	'.cm-lens-row:hover': {
		borderBottomColor: 'var(--background-modifier-border)',
	},
	'.cm-lens-row-name': {
		background: 'none',
		border: 'none',
		padding: '0',
		font: 'inherit',
		color: 'var(--interactive-accent, var(--text-accent))',
		cursor: 'pointer',
		textDecoration: 'none',
		fontWeight: '500',
	},
	'.cm-lens-row-name:hover': { textDecoration: 'underline' },
	'.cm-lens-row-sep': { color: 'var(--text-faint)' },
	'.cm-lens-row-headline': {
		color: 'var(--text-muted)',
		fontStyle: 'italic',
	},
	// MIG-060 §D — Threading-gesture action container + buttons.
	// `marginInlineStart: 'auto'` is the magic that makes layout
	// auto-flip: in LTR rows the actions land on the right edge;
	// in RTL rows (Arabic note names) they land on the left edge.
	// Logical properties handle both without manual dir checks.
	'.cm-lens-row-actions': {
		marginInlineStart: 'auto',
		display: 'flex',
		alignItems: 'center',
		gap: '2px',
		opacity: '0.55',
		transition: 'opacity 0.15s',
	},
	'.cm-lens-row:hover .cm-lens-row-actions': {
		opacity: '1',
	},
	'.cm-lens-row-action': {
		width: '20px',
		height: '20px',
		padding: '0',
		display: 'inline-flex',
		alignItems: 'center',
		justifyContent: 'center',
		background: 'transparent',
		border: 'none',
		color: 'var(--text-muted)',
		cursor: 'pointer',
		borderRadius: '3px',
		transition: 'background-color 0.12s, color 0.12s',
	},
	'.cm-lens-row-action:hover': {
		background: 'var(--background-modifier-hover)',
		color: 'var(--text-normal)',
	},
	'.cm-lens-row-action svg': {
		width: '12px',
		height: '12px',
		display: 'block',
	},
	// Per-surface hue hint on hover — keeps the visual mapping
	// (360.3D = purple, CNS = teal/cyan, Cataloger = orange/amber)
	// consistent with the dock buttons' visual identity.
	'.cm-lens-row-action-360:hover': {
		color: 'var(--color-purple)',
	},
	'.cm-lens-row-action-cns:hover': {
		color: 'var(--color-cyan)',
	},
	'.cm-lens-row-action-cataloger:hover': {
		color: 'var(--color-orange)',
	},
	'.cm-lens-footer': {
		display: 'flex',
		justifyContent: 'flex-end',
		marginTop: '8px',
		paddingTop: '6px',
		borderTop: '1px solid var(--background-modifier-border)',
	},
	'.cm-lens-time': {
		fontSize: '0.7em',
		color: 'var(--text-faint)',
	},
	'.cm-md-image-widget': {
		display: 'block',
		margin: '8px 0',
	},
	'.cm-md-image-widget img': {
		maxWidth: '100%',
		borderRadius: '6px',
		border: '1px solid var(--background-modifier-border)',
	},
	'.cm-md-image-fallback': {
		display: 'inline-block',
		padding: '4px 8px',
		fontSize: '12px',
		color: 'var(--text-muted)',
		background: 'var(--background-secondary)',
		borderRadius: '4px',
	},
	'.cm-md-codeblock-lang': {
		display: 'inline-block',
		fontSize: '10px',
		fontWeight: '600',
		color: 'var(--text-muted)',
		textTransform: 'uppercase',
		letterSpacing: '0.5px',
		padding: '1px 6px',
		marginLeft: '8px',
		background: 'var(--background-secondary)',
		borderRadius: '3px',
		verticalAlign: 'middle',
	},
});

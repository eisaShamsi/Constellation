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
import { RangeSetBuilder, StateField, StateEffect } from '@codemirror/state';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

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

// Typed link decorations — one per semantic link type (CE Phase 1)
const TYPED_LINK_TYPES = new Set([
	'supports', 'contradicts', 'causes', 'exemplifies',
	'generalizes', 'derives-from', 'part-of', 'associative',
]);
const typedLinkDecos: Record<string, ReturnType<typeof Decoration.mark>> = {
	supports:       Decoration.mark({ class: 'cm-md-link cm-link-supports' }),
	contradicts:    Decoration.mark({ class: 'cm-md-link cm-link-contradicts' }),
	causes:         Decoration.mark({ class: 'cm-md-link cm-link-causes' }),
	exemplifies:    Decoration.mark({ class: 'cm-md-link cm-link-exemplifies' }),
	generalizes:    Decoration.mark({ class: 'cm-md-link cm-link-generalizes' }),
	'derives-from': Decoration.mark({ class: 'cm-md-link cm-link-derives-from' }),
	'part-of':      Decoration.mark({ class: 'cm-md-link cm-link-part-of' }),
	associative:    linkDeco,
};

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
					}

					// Language label for non-dataview code blocks
					if (!cursorInBlock) {
						const langMatch = info.match(/^```+\s*(\S+)/);
						if (langMatch && !/^dataview$/i.test(langMatch[1])) {
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
					if (pipeIndex >= 0) {
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
						const isTyped = lastPipeIndex > 0 && TYPED_LINK_TYPES.has(afterLastPipe);
						if (isTyped && lastPipeIndex === pipeIndex) {
							// 2-part typed: [[note|type]]. Show the note name in the
							// type color; hide [[ and |type]].
							const noteEnd = innerFrom + pipeIndex;
							ranges.push({ from: absFrom, to: innerFrom, deco: replaceDeco }); // hide [[
							ranges.push({ from: innerFrom, to: noteEnd, deco: typedLinkDecos[afterLastPipe] ?? linkDeco });
							ranges.push({ from: noteEnd, to: absTo, deco: replaceDeco }); // hide |type]]
						} else if (isTyped) {
							// 3-part typed: [[note|alias|type]]. Show the alias in the
							// type color; hide [[note| and |type]].
							const aliasStart = innerFrom + pipeIndex + 1;
							const aliasEnd = innerFrom + lastPipeIndex;
							ranges.push({ from: absFrom, to: aliasStart, deco: replaceDeco }); // hide [[note|
							ranges.push({ from: aliasStart, to: aliasEnd, deco: typedLinkDecos[afterLastPipe] ?? linkDeco });
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
						const targetName = (pipeIndex >= 0 ? raw.slice(0, pipeIndex) : raw).trim().toLowerCase();
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
class LivePreviewPlugin {
	decorations: DecorationSet;
	private rebuildTimer: ReturnType<typeof setTimeout> | null = null;
	private lastCursorLine = -1;

	constructor(view: EditorView) {
		this.decorations = buildDecorations(view);
		this.lastCursorLine = view.state.doc.lineAt(view.state.selection.main.head).number;
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
				e.is(setLibraryPath) || e.is(setAttachmentFolder)
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
				if (!view.destroyed) {
					this.decorations = buildDecorations(view);
				}
			}, 300);
		}
	}

	destroy() {
		if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
	}
}

/** The live preview extension — add to CM extensions to enable */
export const livePreviewPlugin = ViewPlugin.fromClass(LivePreviewPlugin, {
	decorations: (v) => v.decorations,
});

/** Theme for live preview decorations */
export const livePreviewTheme = EditorView.theme({
	'.cm-md-heading1': {
		fontSize: 'var(--h1-size, 1.8em)',
		fontWeight: 'var(--heading-weight, 700)',
		lineHeight: 'var(--line-height-tight, 1.3)',
	},
	'.cm-md-heading2': {
		fontSize: 'var(--h2-size, 1.5em)',
		fontWeight: 'var(--heading-weight, 700)',
		lineHeight: 'var(--line-height-tight, 1.3)',
	},
	'.cm-md-heading3': {
		fontSize: 'var(--h3-size, 1.25em)',
		fontWeight: 'var(--heading-weight, 700)',
		lineHeight: 'var(--line-height-tight, 1.3)',
	},
	'.cm-md-heading4': {
		fontSize: 'var(--h4-size, 1.1em)',
		fontWeight: 'var(--heading-weight, 700)',
		lineHeight: 'var(--line-height-tight, 1.3)',
	},
	'.cm-md-heading5': {
		fontSize: 'var(--h5-size, 1.0em)',
		fontWeight: 'var(--heading-weight, 700)',
		lineHeight: 'var(--line-height-tight, 1.3)',
	},
	'.cm-md-heading6': {
		fontSize: 'var(--h6-size, 0.95em)',
		fontWeight: 'var(--heading-weight, 700)',
		color: 'var(--text-muted)',
		lineHeight: 'var(--line-height-tight, 1.3)',
	},
	'.cm-md-bold': {
		fontWeight: '700',
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
	// CE Phase 1 — Typed Link colors (underline tint matches GraphMind + BacklinksPanel badges)
	'.cm-link-supports':     { color: '#4A9EFF', textDecorationColor: '#4A9EFF66' },
	'.cm-link-contradicts':  { color: '#FF4A4A', textDecorationColor: '#FF4A4A66' },
	'.cm-link-causes':       { color: '#FF8C42', textDecorationColor: '#FF8C4266' },
	'.cm-link-exemplifies':  { color: '#4AFF88', textDecorationColor: '#4AFF8866' },
	'.cm-link-generalizes':  { color: '#A44AFF', textDecorationColor: '#A44AFF66' },
	'.cm-link-derives-from': { color: '#FFD700', textDecorationColor: '#FFD70066' },
	'.cm-link-part-of':      { color: '#AAAAAA', textDecorationColor: '#AAAAAA66' },
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
		color: 'var(--text-muted)',
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

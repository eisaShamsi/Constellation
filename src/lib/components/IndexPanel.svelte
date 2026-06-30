<script lang="ts">
	import { t, tn } from '$lib/i18n';
	import { untrack } from 'svelte';
	import ContextMenu from './ContextMenu.svelte';
	import { openStyleSetterToCategory } from '$lib/stores/styleSetter'; // MIG-077 §F — RC "Style…"
	import {
		SNIPPET_MARK_START,
		SNIPPET_MARK_END,
		type IndexEntry,
		type IndexMention,
		type CooccurringTerm,
		type FilterExpansion,
		type CtseTermSimilarity,
		type IndexHistoryEntry,
		lexiconExpandForFilter,
		ctseSearchTermsByConcept,
		readIndexHistory,
		writeIndexHistoryEntry,
	} from '$lib/libraries/store';
	import VirtualList from '$lib/components/VirtualList.svelte';
	// MIG-044 Phase 2 — NSC summary headlines under each mention row.
	import { getSummariesFor } from '$lib/nsc/summaryStore';

	const SNIPPET_MARK_START_CODE = SNIPPET_MARK_START.charCodeAt(0);
	const SNIPPET_MARK_END_CODE = SNIPPET_MARK_END.charCodeAt(0);

	let {
		entries = [] as IndexEntry[],
		isLoading = false,
		onNoteClick,
		onTermClick,
		onNoteHover = (_path: string, _e: MouseEvent) => {},
		onNoteLeave = () => {},
		activeNotePath = '',
		selectedTerms = new Set<string>(),
		onTermSelect,
		loadMentions,
		loadCooccurrence,
		cacheKey,
		bridgeFilterEnabled = false,
		searchHistoryEnabled = false,
	}: {
		entries: IndexEntry[];
		isLoading?: boolean;
		onNoteClick: (filePath: string, noteName: string, term?: string, e?: MouseEvent) => void;
		onTermClick?: (term: string, mentions: { note_path: string; note_name: string }[]) => void;
		onNoteHover?: (filePath: string, e: MouseEvent) => void;
		onNoteLeave?: () => void;
		activeNotePath?: string;
		selectedTerms?: Set<string>;
		onTermSelect?: (term: string, mentions: { note_path: string; note_name: string }[], selected: boolean) => void;
		/** Lazy-loader for per-term mentions. Called on first expand of a term. */
		loadMentions?: (term: string) => Promise<IndexMention[]>;
		/** Lazy-loader for per-term co-occurring vocabulary terms. Called
		 *  alongside `loadMentions` on expand; results render as a chip
		 *  strip beneath the mentions list. */
		loadCooccurrence?: (term: string) => Promise<CooccurringTerm[]>;
		/** Cache-invalidation token. When this value changes, the panel
		 *  drops `mentionsCache` so the next click re-fetches. Parents
		 *  flip this whenever `loadMentions` would now produce different
		 *  results — e.g. a Settings toggle that affects what mentions
		 *  the IPC returns. Doesn't need a stable type; reference change
		 *  is the signal. The semantic meaning lives in the parent. */
		cacheKey?: unknown;
		/** MIG-011: when true, the filter box ALSO queries the M11
		 *  Lexical Bridge per-keystroke (debounced) and merges cross-
		 *  language matches into the result list with `via {lemma}`
		 *  badges. When false, filter is pure substring (default).
		 *  Toggle source: `$appSettings.index.expandCrossLanguage`. */
		bridgeFilterEnabled?: boolean;
		/** MIG-012: when true, the filter input shows a dropdown of
		 *  recently-used queries on focus / down-arrow, AND saves each
		 *  committed query to history. When false, no read or write —
		 *  total opt-out. Toggle source:
		 *  `$appSettings.index.searchHistoryEnabled`. */
		searchHistoryEnabled?: boolean;
	} = $props();

	// Hoisted top-level state — read by effects and downstream derivations.
	// Originally lower in the file; moved up so the MIG-012 semantic / history
	// effects (which read `filterQuery`) don't trip the "used before declaration"
	// TS error. Other state stays where it was for now.
	let filterQuery = $state('');

	// Per-term mentions cache — populated on demand when the user expands a term.
	// Keeps the initial IPC payload tiny (terms only; no mentions pre-loaded).
	let mentionsCache = $state<Map<string, IndexMention[]>>(new Map());
	let loadingMentions = $state<Set<string>>(new Set());

	// MIG-044 Phase 2 — NSC summary headlines for the mentions of every
	// currently-expanded term. Cache-first, batched via the shared store.
	// The $effect's only tracked dep is `mentionsCache.size`; reads of the
	// cache itself + writes to `summaryHeadlines` happen inside untrack,
	// matching the Rule-2 pattern already used by the mentionsCache
	// invalidation effect above.
	let summaryHeadlines = $state<Map<string, string>>(new Map());
	$effect(() => {
		void mentionsCache.size; // re-fire when a term expands / mentions arrive
		untrack(() => {
			const paths = new Set<string>();
			for (const list of mentionsCache.values()) {
				for (const m of list) if (m.note_path) paths.add(m.note_path);
			}
			if (paths.size === 0) return;
			const list = Array.from(paths);
			(async () => {
				try {
					const entries = await getSummariesFor(list);
					let changed = false;
					const next = new Map(summaryHeadlines);
					for (const [path, entry] of entries) {
						const h = entry.headline ?? '';
						if (h && next.get(path) !== h) { next.set(path, h); changed = true; }
					}
					if (changed) summaryHeadlines = next;
				} catch { /* ignore — surface just renders without headlines */ }
			})();
		});
	});

	// Re-fetch when the parent invalidates. Body wrapped in `untrack` so
	// reading `mentionsCache.size` doesn't accidentally make the cache
	// itself a dependency (classic Rule 2 violation in CLAUDE.md — the
	// effect would clear the cache it just populated, infinite-looping).
	$effect(() => {
		void cacheKey;
		untrack(() => {
			if (mentionsCache.size > 0) mentionsCache = new Map();
			if (loadingMentions.size > 0) loadingMentions = new Set();
			// MIG-011: also blow away the bridge expansion cache when
			// the toggle flips. Stale FilterExpansion would render the
			// old bridge results until the user typed again.
			if (bridgeExpansionCache.size > 0) bridgeExpansionCache = new Map();
			bridgeExpansion = null;
		});
	});

	// ─── MIG-011 — Index filter cross-language bridge ───
	//
	// State: the latest FilterExpansion (or null when none for the
	// current query). Populated via a debounced effect that watches
	// `filterQuery` + `bridgeFilterEnabled`. Cached per-query for the
	// session so re-typing the same query is free.
	let bridgeExpansion = $state<FilterExpansion | null>(null);
	let bridgeExpansionCache = $state<Map<string, FilterExpansion | null>>(new Map());
	let bridgeFetchToken = 0; // monotonic; cancels stale in-flight fetches

	// ─── MIG-013 §1D — CTSE Bridge Adapter (cross-language `≈ similar`) ───
	//
	// Replaces MIG-012's `searchTermsSemantic` + per-library
	// `term_embeddings` table (retired in §1C). The new path is
	// query-time concept expansion: embed the user's filter query,
	// find top-K nearest M11 concepts, expand each to its multilingual
	// lemmas, tokenize through `fts5_tokenizer::tokenize_to_vec`,
	// and return the subset that exists in this Universe's
	// `term_vocab`. The dropdown shows those matched terms with the
	// `≈ similar` badge — same UX MIG-012 had, no per-library setup
	// cost, no boot wait. Aligns with the Lucene SynonymGraphFilter /
	// SQLite FTS5 Method 2 / CLIR query-translation pattern (Law 1.5,
	// Working Agreement #5).
	//
	// `semanticMatches` is a `Map<term, score>` keyed by the FTS5-
	// stored stem (which is what `entry.term` is), so the lookup
	// against `IndexEntry` rows is exact.
	let semanticMatches = $state<Map<string, number>>(new Map());
	let semanticCache = $state<Map<string, Map<string, number>>>(new Map());
	let semanticFetchToken = 0;

	$effect(() => {
		const q = filterQuery.trim().toLowerCase();
		return untrack(() => {
			if (!q) {
				semanticMatches = new Map();
				return undefined;
			}
			if (semanticCache.has(q)) {
				semanticMatches = semanticCache.get(q) ?? new Map();
				return undefined;
			}
			const myToken = ++semanticFetchToken;
			const handle = setTimeout(async () => {
				try {
					const results: CtseTermSimilarity[] = await ctseSearchTermsByConcept(q);
					if (myToken !== semanticFetchToken) return;
					const m = new Map<string, number>();
					for (const r of results) m.set(r.term, r.score);
					semanticCache.set(q, m);
					semanticCache = new Map(semanticCache);
					semanticMatches = m;
				} catch (err) {
					if (myToken !== semanticFetchToken) return;
					console.error('[IndexPanel] ctseSearchTermsByConcept failed for q=', q, err);
					semanticMatches = new Map();
				}
			}, 300);
			return () => clearTimeout(handle);
		});
	});

	// ─── MIG-012 — search history state ───
	//
	// History is loaded on filter-input focus when the toggle is on,
	// and saved when the user commits a non-empty query (Enter or
	// blur with content). The dropdown shows when the input is focused
	// AND filterQuery is empty AND history exists. Typing dismisses it
	// (substring/lexical/semantic results take over).
	let searchHistory = $state<IndexHistoryEntry[]>([]);
	let filterFocused = $state(false);
	let historyDropdownOpen = $derived(
		searchHistoryEnabled
		&& filterFocused
		&& filterQuery.trim() === ''
		&& searchHistory.length > 0
	);
	let lastSavedQuery = '';

	async function loadSearchHistory() {
		if (!searchHistoryEnabled) {
			searchHistory = [];
			return;
		}
		try {
			searchHistory = await readIndexHistory(20);
		} catch (err) {
			console.error('[IndexPanel] readIndexHistory failed:', err);
			searchHistory = [];
		}
	}

	function commitSearchToHistory(query: string) {
		if (!searchHistoryEnabled) return;
		const trimmed = query.trim();
		if (!trimmed || trimmed === lastSavedQuery) return;
		lastSavedQuery = trimmed;
		// Fire-and-forget — don't block the filter UX waiting on the write.
		writeIndexHistoryEntry(trimmed)
			.then(() => loadSearchHistory())
			.catch(err => console.error('[IndexPanel] writeIndexHistoryEntry failed:', err));
	}

	$effect(() => {
		// Refresh history when toggle flips on (or initial mount with toggle on).
		void searchHistoryEnabled;
		untrack(() => { void loadSearchHistory(); });
	});

	$effect(() => {
		// Track the deps explicitly. Cache + state writes go inside untrack
		// per the same Rule-2 reasoning as the mentions-cache effect above.
		// The cleanup return MUST bubble out of `untrack` so Svelte sees it
		// and can cancel the pending timeout on the next run / unmount —
		// otherwise a fast typist would fire one IPC per ~300ms keystroke
		// burst instead of just the final settled query.
		const enabled = bridgeFilterEnabled;
		const q = filterQuery.trim().toLowerCase();
		return untrack(() => {
			// Toggle off OR empty query → clear, no IPC.
			if (!enabled || !q) {
				bridgeExpansion = null;
				return undefined;
			}
			// Cache hit — use cached result instantly, no IPC.
			if (bridgeExpansionCache.has(q)) {
				bridgeExpansion = bridgeExpansionCache.get(q) ?? null;
				return undefined;
			}
			// Debounce + fetch. Token cancels stale results when user
			// keeps typing — only the most recent fetch's result lands.
			const myToken = ++bridgeFetchToken;
			const handle = setTimeout(async () => {
				try {
					const result = await lexiconExpandForFilter(q);
					if (myToken !== bridgeFetchToken) return; // superseded
					bridgeExpansionCache.set(q, result);
					bridgeExpansionCache = new Map(bridgeExpansionCache);
					bridgeExpansion = result;
				} catch (err) {
					if (myToken !== bridgeFetchToken) return;
					console.error('[IndexPanel] lexiconExpandForFilter failed for q=', q, err);
					bridgeExpansion = null;
				}
			}, 300);
			return () => clearTimeout(handle);
		});
	});

	function getMentions(term: string): IndexMention[] {
		return mentionsCache.get(term) ?? [];
	}

	async function ensureMentionsLoaded(term: string) {
		if (!loadMentions) return;
		if (mentionsCache.has(term)) return;
		if (loadingMentions.has(term)) return;
		loadingMentions.add(term);
		loadingMentions = new Set(loadingMentions);
		try {
			const list = await loadMentions(term);
			mentionsCache.set(term, list);
			mentionsCache = new Map(mentionsCache);
		} catch (err) {
			// IPC error (FTS5 MATCH parser hiccup, tokenizer panic, etc.)
			// Without a catch the error bubbles past `finally` as an
			// unhandled rejection — silent in production. Cache an empty
			// array so the UI can render a fallback row instead of just
			// looking blank, and log to console for diagnosis.
			console.error('[IndexPanel] readTermMentions failed for term=', term, err);
			mentionsCache.set(term, []);
			mentionsCache = new Map(mentionsCache);
		} finally {
			loadingMentions.delete(term);
			loadingMentions = new Set(loadingMentions);
		}
	}

	// Per-term co-occurring-terms cache — mirrors the mentions cache shape
	// above; populated on expand alongside `ensureMentionsLoaded`.
	let cooccurrenceCache = $state<Map<string, CooccurringTerm[]>>(new Map());
	let loadingCooccurrence = $state<Set<string>>(new Set());

	function getCooccurrence(term: string): CooccurringTerm[] {
		return cooccurrenceCache.get(term) ?? [];
	}

	async function ensureCooccurrenceLoaded(term: string) {
		if (!loadCooccurrence) return;
		if (cooccurrenceCache.has(term)) return;
		if (loadingCooccurrence.has(term)) return;
		loadingCooccurrence.add(term);
		loadingCooccurrence = new Set(loadingCooccurrence);
		try {
			const list = await loadCooccurrence(term);
			cooccurrenceCache.set(term, list);
			cooccurrenceCache = new Map(cooccurrenceCache);
		} catch {
			// Cache an empty result on error so we don't re-hit the backend
			// on every render — collapse/re-expand to retry.
			cooccurrenceCache.set(term, []);
			cooccurrenceCache = new Map(cooccurrenceCache);
		} finally {
			loadingCooccurrence.delete(term);
			loadingCooccurrence = new Set(loadingCooccurrence);
		}
	}

	// (filterQuery hoisted to top — see comment above mentionsCache.)
	let expandedTerms = $state<Set<string>>(new Set());
	let activeScript = $state<string>('all');
	let activeLetter = $state<string | null>(null);
	let sortMode = $state<'alpha' | 'freq'>('alpha');
	let excludedTerms = $state<Set<string>>(loadExcluded());
	let showHidden = $state(false);
	let contextMenu = $state<{ x: number; y: number; term: string } | null>(null);

	// ═══════════════════════════════════════════════
	// ─── Exclusion list persistence ───
	// ═══════════════════════════════════════════════
	function loadExcluded(): Set<string> {
		try {
			const data = localStorage.getItem('index-excluded-terms');
			return data ? new Set(JSON.parse(data)) : new Set();
		} catch { return new Set(); }
	}
	function saveExcluded() {
		localStorage.setItem('index-excluded-terms', JSON.stringify([...excludedTerms]));
	}
	function hideTerm(term: string) {
		excludedTerms.add(term);
		excludedTerms = new Set(excludedTerms);
		saveExcluded();
		contextMenu = null;
	}
	function unhideTerm(term: string) {
		excludedTerms.delete(term);
		excludedTerms = new Set(excludedTerms);
		saveExcluded();
	}

	// ═══════════════════════════════════════════════
	// ─── Script Detection ───
	// ═══════════════════════════════════════════════
	const ARABIC_RE = /[\u0600-\u06FF\u0750-\u077F\uFB50-\uFDFF\uFE70-\uFEFF]/;
	const HEBREW_RE = /[\u0590-\u05FF]/;
	const LATIN_RE = /[a-zA-Z\u00C0-\u00FF\u0100-\u024F\u1E00-\u1EFF]/;
	const CYRILLIC_RE = /[\u0400-\u04FF]/;
	const DEVANAGARI_RE = /[\u0900-\u097F]/;
	const HANGUL_RE = /[\uAC00-\uD7AF\u3130-\u318F]/;
	const KANA_RE = /[\u3040-\u309F\u30A0-\u30FF]/;
	const CJK_RE = /[\u4E00-\u9FFF\u3400-\u4DBF]/;

	type ScriptKey = 'ar' | 'he' | 'en' | 'ru' | 'hi' | 'ko' | 'ja' | 'zh' | 'other';

	const SCRIPT_LABELS: Record<ScriptKey, string> = {
		ar: 'عربي', he: 'עברית', en: 'English', ru: 'Русский',
		hi: 'हिन्दी', ko: '한국어', ja: '日本語', zh: '中文', other: '#'
	};
	const SCRIPT_ORDER: Record<ScriptKey, number> = {
		ar: 0, he: 1, en: 2, ru: 3, hi: 4, ko: 5, ja: 6, zh: 7, other: 8
	};

	function getScript(ch: string): ScriptKey {
		if (ARABIC_RE.test(ch)) return 'ar';
		if (HEBREW_RE.test(ch)) return 'he';
		if (CYRILLIC_RE.test(ch)) return 'ru';
		if (DEVANAGARI_RE.test(ch)) return 'hi';
		if (HANGUL_RE.test(ch)) return 'ko';
		if (KANA_RE.test(ch)) return 'ja';
		if (CJK_RE.test(ch)) return 'zh';
		if (LATIN_RE.test(ch)) return 'en';
		return 'other';
	}

	// ─── Query normalization for filter matching ───
	// Terms are stored in stemmed form (the backend's Arabic Light10
	// pipeline strips "ال", suffixes, diacritics). A raw substring match
	// would miss "الكتاب" against the indexed "كتاب", so we mirror the
	// Rust `normalize_arabic` + `stem_arabic_light10` (libraries.rs) in JS
	// and match on either the raw query or its stem. Other scripts rely
	// on write-time stemming; query-side mirrors can follow as needed.

	const ARABIC_DIACRITICS_RE = /[\u064B-\u065F\u0670\u06D6-\u06ED\u0640]/g;
	const ARABIC_ALEF_VARIANTS_RE = /[\u0622\u0623\u0625\u0671]/g;   // آ أ إ ٱ → ا
	const ARABIC_ALEF_MAKSURA_RE = /[\u0649]/g;                       // ى → ي
	const ARABIC_TA_MARBUTA_RE = /[\u0629]/g;                         // ة → ه

	/** Exact JS port of the backend `stem_arabic_light10` (libraries.rs):
	 *  normalize + sequential 3/2/1-char prefix strip + 2/1-char suffix
	 *  strip. Sequential so "والمعرفة" → "معرف" in one pass. */
	function normalizeArabicForFilter(s: string): string {
		let t = s.replace(ARABIC_DIACRITICS_RE, '');
		t = t.replace(ARABIC_ALEF_VARIANTS_RE, '\u0627');
		t = t.replace(ARABIC_ALEF_MAKSURA_RE, '\u064A');
		t = t.replace(ARABIC_TA_MARBUTA_RE, '\u0647');

		let chars = Array.from(t);
		let len = chars.length;

		if (len >= 6) {
			const p = chars[0] + chars[1] + chars[2];
			if (p === 'وال' || p === 'بال' || p === 'كال' || p === 'فال') {
				chars = chars.slice(3);
				len = chars.length;
			}
		}
		if (len >= 4) {
			const p = chars[0] + chars[1];
			if (p === 'ال' || p === 'لل') {
				chars = chars.slice(2);
				len = chars.length;
			}
		}
		if (len >= 4 && chars[0] === 'و') {
			chars = chars.slice(1);
			len = chars.length;
		}

		if (len >= 4) {
			const s2 = chars[len - 2] + chars[len - 1];
			if (
				s2 === 'ها' || s2 === 'ان' || s2 === 'ات' || s2 === 'ون' ||
				s2 === 'ين' || s2 === 'يه' || s2 === 'يت' || s2 === 'ته'
			) {
				chars = chars.slice(0, len - 2);
				len = chars.length;
			}
		}
		if (len >= 3) {
			const last = chars[len - 1];
			if (last === 'ه' || last === 'ي') {
				chars = chars.slice(0, len - 1);
			}
		}

		return chars.join('');
	}

	/** A filter sub-query prepared once per filter pass: lower-cased form
	 *  plus its Arabic stem (if the query is Arabic and stemming would
	 *  change it). Keeps the per-entry loop to pure substring checks. */
	type PreparedQuery = { q: string; stem: string | null };
	function prepareQuery(q: string): PreparedQuery {
		if (!q || !ARABIC_RE.test(q)) return { q, stem: null };
		const stemmed = normalizeArabicForFilter(q);
		return { q, stem: stemmed !== q ? stemmed : null };
	}

	// ═══════════════════════════════════════════════
	// ─── Alphabet Orders ───
	// ═══════════════════════════════════════════════
	const ARABIC_ALPHABET = 'ابتثجحخدذرزسشصضطظعغفقكلمنهوي';
	const HEBREW_ALPHABET = 'אבגדהוזחטיכלמנסעפצקרשת';
	const CYRILLIC_ALPHABET = 'АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ';
	const KOREAN_INITIALS = 'ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ';

	function buildOrderMap(alphabet: string): Map<string, number> {
		const m = new Map<string, number>();
		for (let i = 0; i < alphabet.length; i++) m.set(alphabet[i], i);
		return m;
	}
	const ARABIC_ORDER = buildOrderMap(ARABIC_ALPHABET);
	const HEBREW_ORDER = buildOrderMap(HEBREW_ALPHABET);
	const CYRILLIC_ORDER = buildOrderMap(CYRILLIC_ALPHABET);
	const KOREAN_ORDER = buildOrderMap(KOREAN_INITIALS);

	// ═══════════════════════════════════════════════
	// ─── Arabic Indexing ───
	// ═══════════════════════════════════════════════
	function getArabicBaseLetter(ch: string): string {
		const code = ch.charCodeAt(0);
		if (code === 0x0622 || code === 0x0623 || code === 0x0625 || code === 0x0627) return 'ا';
		if (code === 0x0629) return 'ه';
		return ch;
	}

	function stripArabicPrefix(word: string): string {
		const prefixes = ['بال', 'فال', 'كال', 'وال', 'لل', 'ال'];
		for (const p of prefixes) {
			if (word.startsWith(p) && word.length > p.length) return word.slice(p.length);
		}
		return word;
	}

	// ═══════════════════════════════════════════════
	// ─── Hebrew Indexing ───
	// ═══════════════════════════════════════════════
	function getHebrewBaseLetter(ch: string): string {
		const code = ch.charCodeAt(0);
		if (code === 0x05DA) return 'כ';
		if (code === 0x05DD) return 'מ';
		if (code === 0x05DF) return 'נ';
		if (code === 0x05E3) return 'פ';
		if (code === 0x05E5) return 'צ';
		return ch;
	}

	// ═══════════════════════════════════════════════
	// ─── Latin Indexing ───
	// ═══════════════════════════════════════════════
	function normalizeLatin(ch: string): string {
		return ch.toUpperCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '');
	}

	function getLatinIndexLetter(word: string): string {
		if (word.length > 2 && /^[ld]'/i.test(word)) {
			return normalizeLatin(word[2]);
		}
		return normalizeLatin(word[0]);
	}

	// ═══════════════════════════════════════════════
	// ─── Korean Indexing ───
	// ═══════════════════════════════════════════════
	function getKoreanInitial(ch: string): string {
		const code = ch.charCodeAt(0);
		if (code >= 0xAC00 && code <= 0xD7A3) {
			return KOREAN_INITIALS[Math.floor((code - 0xAC00) / 588)] ?? ch;
		}
		return ch;
	}

	// ═══════════════════════════════════════════════
	// ─── Unified Index Info ───
	// ═══════════════════════════════════════════════
	interface IndexInfo { script: ScriptKey; letter: string; sortKey: string; }

	function getIndexInfo(word: string): IndexInfo {
		const first = word[0];
		if (!first) return { script: 'other', letter: '#', sortKey: word };
		const script = getScript(first);
		switch (script) {
			case 'ar': {
				const stripped = stripArabicPrefix(word);
				return { script, letter: getArabicBaseLetter(stripped[0] ?? first), sortKey: stripped };
			}
			case 'he':
				return { script, letter: getHebrewBaseLetter(first), sortKey: word };
			case 'en': {
				const letter = getLatinIndexLetter(word);
				const sortKey = word.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
				return { script, letter, sortKey };
			}
			case 'ru':
				return { script, letter: first.toUpperCase(), sortKey: word.toLowerCase() };
			case 'ko':
				return { script, letter: getKoreanInitial(first), sortKey: word };
			case 'ja':
				return { script, letter: first, sortKey: word };
			case 'zh':
				return { script, letter: first, sortKey: word };
			case 'hi':
				return { script, letter: first, sortKey: word };
			default:
				return { script: 'other', letter: '#', sortKey: word };
		}
	}

	// ═══════════════════════════════════════════════
	// ─── Unified Letter Comparator ───
	// ═══════════════════════════════════════════════
	function compareLetters(a: string, b: string): number {
		const sa = getScript(a === '#' ? '!' : a);
		const sb = getScript(b === '#' ? '!' : b);
		const oa = a === '#' ? SCRIPT_ORDER.other : SCRIPT_ORDER[sa];
		const ob = b === '#' ? SCRIPT_ORDER.other : SCRIPT_ORDER[sb];
		if (oa !== ob) return oa - ob;
		switch (sa) {
			case 'ar': return (ARABIC_ORDER.get(a) ?? 999) - (ARABIC_ORDER.get(b) ?? 999);
			case 'he': return (HEBREW_ORDER.get(a) ?? 999) - (HEBREW_ORDER.get(b) ?? 999);
			case 'ru': return (CYRILLIC_ORDER.get(a) ?? 999) - (CYRILLIC_ORDER.get(b) ?? 999);
			case 'ko': return (KOREAN_ORDER.get(a) ?? 999) - (KOREAN_ORDER.get(b) ?? 999);
			default: return a.localeCompare(b);
		}
	}

	// ═══════════════════════════════════════════════
	// ─── Derived State ───
	// ═══════════════════════════════════════════════
	// MIG-011: filter-pass result — both the matched entries AND a
	// parallel Map<term, viaLemma> for cross-language bridge annotations.
	// The Map is populated only for entries that matched ONLY because of
	// a bridge lemma (not a direct substring); direct matches don't earn
	// badges (M13 same-language exclusion rule, applied to the filter).
	const filteredResult = $derived.by(() => {
		const raw = filterQuery.trim();
		let result = entries;
		if (!showHidden) {
			result = result.filter(e => !excludedTerms.has(e.term.toLowerCase()));
		}
		const annotations = new Map<string, string>();
		const semanticAnnotations = new Map<string, number>();
		if (!raw) return { entries: result, annotations, semanticAnnotations };
		const hasComma = /[،,؛;]/.test(raw);
		const rawQueries = hasComma
			? raw.split(/[،,؛;]/).map(t => t.trim().toLowerCase()).filter(Boolean)
			: [raw.toLowerCase()];
		if (rawQueries.length === 0) return { entries: result, annotations, semanticAnnotations };
		// Prepare each sub-query ONCE — ARABIC_RE.test + stemming run per
		// filter pass, not per entry. The inner loop is then pure .includes.
		const prepared = rawQueries.map(prepareQuery);
		const bridge = bridgeExpansion; // snapshot; effect updates this
		const semantic = semanticMatches; // MIG-013 §1D snapshot
		const matched: IndexEntry[] = [];
		for (const e of result) {
			const lower = e.term.toLowerCase();
			let direct = false;
			for (const { q, stem } of prepared) {
				if (lower.includes(q)) { direct = true; break; }
				if (stem && lower.includes(stem)) { direct = true; break; }
				// Bidirectional substring — see MIG-010-fix justification:
				// FTS5 stores STEMS (shorter than surface). `term.includes(
				// query)` fails when term is shorter; reverse check catches.
				if (q.includes(lower)) { direct = true; break; }
			}
			if (direct) {
				matched.push(e);
				continue;
			}
			let bridged = false;
			// MIG-011: no direct hit — try the cross-language bridge.
			if (bridge && bridge.lemmas.length > 0) {
				for (const { lemma_lower } of bridge.lemmas) {
					// Same bidirectional shape as the substring path.
					if (lower.includes(lemma_lower) || lemma_lower.includes(lower)) {
						matched.push(e);
						annotations.set(e.term, bridge.source_lemma);
						bridged = true;
						break;
					}
				}
			}
			if (bridged) continue;
			// MIG-013 §1D: no direct + no bridge — try the CTSE concept
			// match. `semanticMatches` is keyed by the FTS5-stored stem,
			// which is what `e.term` is, so the lookup is exact.
			if (semantic.size > 0) {
				const score = semantic.get(e.term);
				if (score !== undefined) {
					matched.push(e);
					semanticAnnotations.set(e.term, score);
				}
			}
		}
		return { entries: matched, annotations, semanticAnnotations };
	});

	const filteredEntries = $derived(filteredResult.entries);
	const bridgeFilterAnnotations = $derived(filteredResult.annotations);
	const semanticFilterAnnotations = $derived(filteredResult.semanticAnnotations);

	const availableScripts = $derived.by(() => {
		const scripts = new Set<ScriptKey>();
		for (const entry of filteredEntries) {
			if (entry.term[0]) scripts.add(getScript(entry.term[0]));
		}
		return Array.from(scripts).sort((a, b) => SCRIPT_ORDER[a] - SCRIPT_ORDER[b]);
	});

	const multiScript = $derived(availableScripts.length > 1);

	$effect(() => {
		if (activeScript !== 'all' && !availableScripts.includes(activeScript as ScriptKey)) {
			activeScript = 'all';
		}
	});

	// Same shape for the letter filter: when the active filter query produces
	// no entries matching the active letter (e.g. user clicked "K" then typed
	// an Arabic search), drop the letter filter automatically. Without this,
	// the letter filter silently persists and hides every result that doesn't
	// start with that letter — the "type Arabic in 'All' returns 0 until you
	// bounce through 'عربي'" bug. The بounce was clearing activeLetter via
	// the activeScript-tracking effect; this makes the clearing automatic.
	$effect(() => {
		if (activeLetter && !filteredEntries.some(e => getIndexInfo(e.term).letter === activeLetter)) {
			activeLetter = null;
		}
	});

	const scriptFilteredEntries = $derived.by(() => {
		let result = filteredEntries;
		if (activeScript !== 'all') {
			result = result.filter(e => {
				const first = e.term[0];
				return first && getScript(first) === activeScript;
			});
		}
		if (activeLetter) {
			result = result.filter(e => getIndexInfo(e.term).letter === activeLetter);
		}
		return result;
	});

	// Reset active letter when switching scripts
	$effect(() => {
		activeScript;
		activeLetter = null;
	});

	// maxCount — bounded scan over a sample (first 5000) so we don't hot-loop
	// `Math.max(...arr)` on 500k+ items per derivation. The bar widths are a
	// visual heuristic; sampling the top-of-list is fine since SQL orders by
	// term and high-count terms are sprinkled throughout anyway.
	const maxCount = $derived.by(() => {
		const sample = scriptFilteredEntries.length > 5000
			? scriptFilteredEntries.slice(0, 5000)
			: scriptFilteredEntries;
		let m = 1;
		for (const e of sample) if (e.count > m) m = e.count;
		return m;
	});

	// Alphabetical mode: group by letter. Inner groups keep SQL's byte-order
	// (ORDER BY term) — the custom `constellation` tokenizer produces
	// language-normalized stems so byte order is already an acceptable
	// dictionary order within a single script. No JS re-sort, which would
	// cost O(n log n) localeCompare calls on 100k+ term groups.
	const groupedEntries = $derived.by(() => {
		if (sortMode === 'freq') return [];
		const groups = new Map<string, IndexEntry[]>();
		for (const entry of scriptFilteredEntries) {
			const { letter } = getIndexInfo(entry.term);
			let group = groups.get(letter);
			if (!group) { group = []; groups.set(letter, group); }
			group.push(entry);
		}
		return Array.from(groups.entries()).sort((a, b) => compareLetters(a[0], b[0]));
	});

	// Frequency mode: flat sorted list. Shallow copy so we don't mutate
	// the upstream `scriptFilteredEntries` reference.
	const freqEntries = $derived.by(() => {
		if (sortMode !== 'freq') return [];
		return [...scriptFilteredEntries].sort((a, b) => b.count - a.count);
	});

	// ═══════════════════════════════════════════════
	// ─── Virtualized row model ───
	// Flatten groupedEntries / freqEntries into a linear array of typed
	// rows so VirtualList can render only the visible window. Expanded
	// mentions become their own row, inserted directly after the term row.
	// ═══════════════════════════════════════════════
	type VRow =
		| { kind: 'header'; letter: string }
		| { kind: 'entry'; entry: IndexEntry }
		| { kind: 'expanded'; term: string };

	const ROW_HEIGHT_HEADER = 30;
	const ROW_HEIGHT_ENTRY = 30;
	/** Plain mention row (title-only match — no FTS5 snippet to show). */
	const ROW_HEIGHT_MENTION_PLAIN = 22;
	/** Mention row with a one-line FTS5 snippet beneath the note name. */
	const ROW_HEIGHT_MENTION_SNIPPET = 40;
	/** MIG-044 Phase 2 — extra room reserved BELOW a mention row when its
	 *  NSC headline is loaded. Matches the .gp-ref-headline line height. */
	const ROW_HEIGHT_MENTION_HEADLINE = 16;
	const ROW_HEIGHT_EXPAND_PAD = 12;
	const ROW_HEIGHT_EXPAND_MIN = 32;
	/** Co-occurrence strip height: header line + one row of wrapping chips.
	 *  Two rows of chips (the common case for result_limit=20 at a typical
	 *  panel width) fits in 56px. Loading spinner placeholder uses the
	 *  header-only height. */
	const ROW_HEIGHT_COOCCUR_HEADER = 22;
	const ROW_HEIGHT_COOCCUR_CHIPS = 56;

	const rows = $derived.by((): VRow[] => {
		// MIG-044 Phase 2 — re-derive when summaryHeadlines arrive so the
		// VirtualList sees a new prop ref and re-runs getRowHeight against
		// the now-larger rows. Contents are unchanged; the ref change is the
		// signal. Without this, headlines render but clip to the old height.
		void summaryHeadlines.size;
		const out: VRow[] = [];
		if (sortMode === 'freq') {
			for (const entry of freqEntries) {
				out.push({ kind: 'entry', entry });
				if (expandedTerms.has(entry.term)) {
					out.push({ kind: 'expanded', term: entry.term });
				}
			}
		} else {
			for (const [letter, group] of groupedEntries) {
				out.push({ kind: 'header', letter });
				for (const entry of group) {
					out.push({ kind: 'entry', entry });
					if (expandedTerms.has(entry.term)) {
						out.push({ kind: 'expanded', term: entry.term });
					}
				}
			}
		}
		return out;
	});

	function getRowHeight(r: VRow): number {
		if (r.kind === 'header') return ROW_HEIGHT_HEADER;
		if (r.kind === 'entry') return ROW_HEIGHT_ENTRY;
		const list = mentionsCache.get(r.term);
		let total = ROW_HEIGHT_EXPAND_PAD;
		// Sum per-mention heights — a mention with a FTS5 snippet needs
		// room for both the note name and the one-line context beneath it.
		// MIG-044 Phase 2: also reserve room for the NSC headline line,
		// but ONLY when it's loaded (rows without headlines stay compact).
		if (list && list.length > 0) {
			for (const m of list) {
				let h = m.snippet ? ROW_HEIGHT_MENTION_SNIPPET : ROW_HEIGHT_MENTION_PLAIN;
				if (summaryHeadlines.get(m.note_path)) h += ROW_HEIGHT_MENTION_HEADLINE;
				total += h;
			}
		}
		// Co-occurrence strip: reserve space whenever we're loading or
		// have results. No strip if the loader returned zero terms —
		// saves 22px on rare/isolated terms where the strip would be empty.
		const cooccur = cooccurrenceCache.get(r.term);
		if (loadingCooccurrence.has(r.term)) {
			total += ROW_HEIGHT_COOCCUR_HEADER;
		} else if (cooccur && cooccur.length > 0) {
			total += ROW_HEIGHT_COOCCUR_HEADER + ROW_HEIGHT_COOCCUR_CHIPS;
		}
		return Math.max(ROW_HEIGHT_EXPAND_MIN, total);
	}

	/** Split a snippet string into plain/highlight parts at the sentinel
	 *  boundaries (see {@link SNIPPET_MARK_START}/{@link SNIPPET_MARK_END}).
	 *  Text parts render through default Svelte interpolation (auto-escaped),
	 *  so user note content can never inject HTML. */
	function splitSnippet(s: string | null | undefined): { text: string; mark: boolean }[] {
		if (!s) return [];
		const out: { text: string; mark: boolean }[] = [];
		let buf = '';
		let inMark = false;
		for (let i = 0; i < s.length; i++) {
			const ch = s.charCodeAt(i);
			if (ch === SNIPPET_MARK_START_CODE) {
				if (buf) { out.push({ text: buf, mark: inMark }); buf = ''; }
				inMark = true;
			} else if (ch === SNIPPET_MARK_END_CODE) {
				if (buf) { out.push({ text: buf, mark: inMark }); buf = ''; }
				inMark = false;
			} else {
				buf += s[i];
			}
		}
		if (buf) out.push({ text: buf, mark: inMark });
		return out;
	}

	// String key that changes on any filter/sort/letter switch — VirtualList
	// resets its scroll-to-top when this changes.
	const scrollResetKey = $derived(`${activeScript}|${activeLetter ?? ''}|${sortMode}|${filterQuery}`);

	const allLetters = $derived.by(() => {
		if (sortMode === 'freq') return [];
		const letters = new Set<string>();
		for (const entry of scriptFilteredEntries) {
			const { letter } = getIndexInfo(entry.term);
			letters.add(letter);
		}
		return Array.from(letters).sort(compareLetters);
	});

	// Group alphabet letters by script for multi-line display
	const groupedLetters = $derived.by(() => {
		if (allLetters.length === 0) return [];
		const groups: { script: ScriptKey; dir: 'rtl' | 'ltr'; letters: string[] }[] = [];
		let currentScript: ScriptKey | null = null;
		let currentLetters: string[] = [];

		for (const letter of allLetters) {
			const script = getScript(letter);
			if (script !== currentScript) {
				if (currentLetters.length > 0 && currentScript) {
					const dir = (currentScript === 'ar' || currentScript === 'he') ? 'rtl' : 'ltr';
					groups.push({ script: currentScript, dir, letters: currentLetters });
				}
				currentScript = script;
				currentLetters = [letter];
			} else {
				currentLetters.push(letter);
			}
		}
		if (currentLetters.length > 0 && currentScript) {
			const dir = (currentScript === 'ar' || currentScript === 'he') ? 'rtl' : 'ltr';
			groups.push({ script: currentScript, dir, letters: currentLetters });
		}
		return groups;
	});

	const totalTerms = $derived(scriptFilteredEntries.length);
	const hiddenCount = $derived(entries.filter(e => excludedTerms.has(e.term.toLowerCase())).length);

	// ─── Multi-term comparison (commonality across selected terms) ───
	// Ctrl/Cmd-click adds a term to `selectedTerms`. With 2+ terms we
	// intersect their mention sets and render the notes that contain
	// every one beneath the chip bar. The in-component click paths all
	// preload mentions before adding to `selectedTerms`; this $effect
	// covers the case where a parent pre-populates the set (persisted
	// session state, deep-link) without going through those paths.

	$effect(() => {
		// Only the comparison path needs pre-loaded mentions — a
		// single selection doesn't render anything until the user
		// expands it, at which point `toggleExpand` handles loading.
		if (selectedTerms.size < 2) return;
		const terms = [...selectedTerms];
		// Writes to mentionsCache / loadingMentions must not re-track
		// this effect (CLAUDE.md Rule 2).
		untrack(() => {
			for (const term of terms) void ensureMentionsLoaded(term);
		});
	});

	type ComparisonState =
		| { kind: 'idle' }                                           // <2 terms
		| { kind: 'loading' }                                        // mentions missing
		| { kind: 'ready'; notes: IndexMention[]; termCount: number };

	const comparisonState = $derived.by((): ComparisonState => {
		const selectedArr = [...selectedTerms];
		if (selectedArr.length < 2) return { kind: 'idle' };
		const perTerm: IndexMention[][] = [];
		for (const t of selectedArr) {
			const m = mentionsCache.get(t);
			if (!m) return { kind: 'loading' };
			perTerm.push(m);
		}
		// Anchor intersection on the smallest list — at most |smallest|
		// membership checks even when some terms have thousands of mentions.
		perTerm.sort((a, b) => a.length - b.length);
		const [first, ...rest] = perTerm;
		const restSets = rest.map(list => new Set(list.map(m => m.note_path)));
		const notes = first.filter(m => restSets.every(s => s.has(m.note_path)));
		return { kind: 'ready', notes, termCount: selectedArr.length };
	});

	function toggleExpand(term: string) {
		if (expandedTerms.has(term)) {
			expandedTerms = new Set();
		} else {
			expandedTerms = new Set([term]);
			// Mentions and co-occurrences are independent DB reads —
			// fire both so expand-time UX stays snappy.
			void ensureMentionsLoaded(term);
			void ensureCooccurrenceLoaded(term);
		}
	}

	/** Chip-click in the co-occurrence strip: same mechanic as Ctrl-click
	 *  on a main row — toggle the term's membership in the comparison set.
	 *  Never expands; the user is composing a multi-term view. */
	async function handleCooccurChipClick(term: string, e: MouseEvent) {
		e.stopPropagation();
		if (!onTermSelect) return;
		await ensureMentionsLoaded(term);
		onTermSelect(term, getMentions(term), !selectedTerms.has(term));
	}

	function selectLetter(letter: string) {
		if (activeLetter === letter) {
			activeLetter = null; // toggle off — show all
		} else {
			activeLetter = letter;
			// VirtualList scrolls to top via scrollResetKey change (activeLetter is part of the key).
		}
	}

	function handleContextMenu(e: MouseEvent, term: string) {
		e.preventDefault();
		contextMenu = { x: e.clientX, y: e.clientY, term };
	}

	function closeContextMenu() {
		contextMenu = null;
	}

	// MIG-077 A2 — the Index term menu via the shared ContextMenu (was inline +
	// hardcoded English). One dynamic Hide/Show item.
	function getIndexTermMenuItems(term: string) {
		const lower = term.toLowerCase();
		return [
			excludedTerms.has(lower)
				? { label: $t('indexPanel.showTerm'), action: () => unhideTerm(lower) }
				: { label: $t('indexPanel.hideTerm'), action: () => hideTerm(lower) },
			{ separator: true },
			// MIG-077 §F — style the Index surface.
			{ label: $t('contextMenu.style'), icon: '🎨', action: () => openStyleSetterToCategory('index') },
		];
	}

	// ─── Export ───
	// Mentions are lazy-loaded per term; export walks the filtered list and
	// loads any not-yet-cached ones before assembling the markdown. On very
	// large result sets (> ~5000 terms) exporting everything would produce
	// multi-MB clipboards and hammer the per-term-mentions IPC, so we cap.
	const EXPORT_CAP = 5000;
	async function exportToClipboard() {
		const allTerms: string[] = [];
		if (sortMode === 'freq') {
			for (const e of freqEntries) allTerms.push(e.term);
		} else {
			for (const [, group] of groupedEntries) for (const e of group) allTerms.push(e.term);
		}
		const truncated = allTerms.length > EXPORT_CAP;
		const termsToExport = truncated ? allTerms.slice(0, EXPORT_CAP) : allTerms;
		await Promise.all(termsToExport.map((t) => ensureMentionsLoaded(t)));
		let md = '# Index\n\n';
		if (truncated) md += `_Showing first ${EXPORT_CAP} of ${allTerms.length} terms. Filter to narrow the export._\n\n`;
		if (sortMode === 'freq') {
			for (const term of termsToExport) {
				const entry = freqEntries.find(e => e.term === term);
				if (!entry) continue;
				const notes = getMentions(entry.term).map(m => m.note_name).join(', ');
				md += `- ${entry.term} (${entry.count}) — ${notes}\n`;
			}
		} else {
			const termSet = new Set(termsToExport);
			for (const [letter, group] of groupedEntries) {
				const inGroup = group.filter(e => termSet.has(e.term));
				if (inGroup.length === 0) continue;
				md += `## ${letter}\n`;
				for (const entry of inGroup) {
					const notes = getMentions(entry.term).map(m => m.note_name).join(', ');
					md += `- ${entry.term} (${entry.count}) — ${notes}\n`;
				}
				md += '\n';
			}
		}
		navigator.clipboard.writeText(md);
	}

	// Close context menu on click outside
	function handleWindowClick() { contextMenu = null; }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="index-panel" dir={activeScript === 'ar' || activeScript === 'he' ? 'rtl' : activeScript === 'all' ? 'auto' : 'ltr'}>
	<!-- Toolbar: filter + actions -->
	<div class="gp-toolbar">
		<div class="gp-search">
			<svg class="gp-search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
			</svg>
			<input
				type="text"
				placeholder={$t('indexPanel.filterPlaceholder')}
				bind:value={filterQuery}
				onfocus={() => filterFocused = true}
				onblur={() => {
					// Delay clearing so click-on-dropdown-item can fire first.
					setTimeout(() => { filterFocused = false; }, 150);
					commitSearchToHistory(filterQuery);
				}}
				onkeydown={(e) => {
					if (e.key === 'Enter') commitSearchToHistory(filterQuery);
				}}
			/>
			{#if filterQuery}
				<button class="gp-clear" onclick={() => filterQuery = ''}>×</button>
			{/if}
			<!-- MIG-012 — search history dropdown -->
			{#if historyDropdownOpen}
				<div class="gp-history-dropdown" dir="auto">
					<div class="gp-history-header">{$t('indexPanel.recentSearches') || 'Recent searches'}</div>
					{#each searchHistory as h}
						<button class="gp-history-item" dir="auto"
							onmousedown={(e) => {
								// onmousedown not onclick — fires before onblur clears focus
								e.preventDefault();
								filterQuery = h.query;
							}}
							title={h.query}>
							<span class="gp-history-q">{h.query}</span>
							<span class="gp-history-count">{h.use_count}×</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>
		<div class="gp-actions">
			<span class="gp-total">{$tn('plurals.terms', totalTerms)}</span>
			<div class="gp-btns">
				<button class="gp-icon-btn" class:active={sortMode === 'freq'} onclick={() => sortMode = sortMode === 'alpha' ? 'freq' : 'alpha'} title={sortMode === 'alpha' ? 'Sort by frequency' : 'Sort alphabetically'}>
					{#if sortMode === 'alpha'}
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h12M3 12h9M3 18h6"/></svg>
					{:else}
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M3 12h12M3 18h6"/></svg>
					{/if}
				</button>
				<button class="gp-icon-btn" onclick={exportToClipboard} title="Copy to clipboard">
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
				</button>
				{#if hiddenCount > 0}
					<button class="gp-icon-btn" class:active={showHidden} onclick={() => showHidden = !showHidden} title={$t('indexPanel.hiddenTerms', { count: String(hiddenCount) })}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							{#if showHidden}
								<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>
							{:else}
								<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>
							{/if}
						</svg>
					</button>
				{/if}
			</div>
		</div>
	</div>

	<!-- Language tabs -->
	{#if multiScript}
		<div class="gp-scripts">
			<button class="gp-script-btn" class:active={activeScript === 'all'} onclick={() => activeScript = 'all'}>
				{$t('indexPanel.allLangs')}
			</button>
			{#each availableScripts as script}
				<button class="gp-script-btn" class:active={activeScript === script} onclick={() => activeScript = script}>
					{SCRIPT_LABELS[script]}
				</button>
			{/each}
		</div>
	{/if}

	<!-- Alphabet bar (only in alpha sort) -->
	{#if sortMode === 'alpha' && groupedLetters.length > 0}
		<div class="gp-alphabet">
			{#each groupedLetters as group}
				<div class="gp-alpha-row" dir={group.dir}>
					{#each group.letters as letter}
						<button class="gp-alpha-btn" class:active={activeLetter === letter} onclick={() => selectLetter(letter)}>{letter}</button>
					{/each}
				</div>
			{/each}
		</div>
	{/if}

	<!-- Selected terms anchor bar -->
	{#if selectedTerms.size > 0}
		<div class="gp-anchor-bar">
			<span class="gp-anchor-label">{$t('indexPanel.comparing') || 'Comparing'}:</span>
			{#each [...selectedTerms] as term}
				<button class="gp-anchor-chip" onclick={async () => {
					if (onTermSelect) {
						await ensureMentionsLoaded(term);
						onTermSelect(term, getMentions(term), false);
					}
				}}>
					<span dir="auto">{term}</span>
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
				</button>
			{/each}
			<button class="gp-anchor-clear" onclick={async () => {
				if (onTermSelect) {
					for (const term of selectedTerms) {
						await ensureMentionsLoaded(term);
						onTermSelect(term, getMentions(term), false);
					}
				}
			}}>
				{$t('indexPanel.clearAll') || 'Clear all'}
			</button>
		</div>
	{/if}

	<!-- Commonality — notes that contain every selected term (2+ terms only). -->
	{#if comparisonState.kind === 'loading'}
		<div class="gp-commonality">
			<div class="gp-commonality-empty">{$t('indexPanel.loadingCommonality') || 'Loading commonality…'}</div>
		</div>
	{:else if comparisonState.kind === 'ready'}
		<div class="gp-commonality">
			<div class="gp-commonality-header">
				{#if comparisonState.notes.length === 0}
					<span>{$t('indexPanel.noCommonality') || 'No notes contain all selected terms.'}</span>
				{:else}
					<span>
						{comparisonState.notes.length}
						{comparisonState.notes.length === 1
							? ($t('indexPanel.noteWithAll') || 'note contains all')
							: ($t('indexPanel.notesWithAll') || 'notes contain all')}
						{comparisonState.termCount}
						{$t('indexPanel.selectedTerms') || 'selected terms'}
					</span>
				{/if}
			</div>
			{#if comparisonState.notes.length > 0}
				<div class="gp-commonality-list" dir="auto">
					{#each comparisonState.notes as note}
						<button class="gp-ref" class:active={note.note_path === activeNotePath}
							data-filepath={note.note_path}
							onclick={(e) => onNoteClick(note.note_path, note.note_name, undefined, e)}
							onmouseenter={(e) => onNoteHover(note.note_path, e)}
							onmouseleave={() => onNoteLeave()}>
							<span class="gp-ref-name">{note.note_name}</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Term list (virtualized — see VirtualList.svelte) -->
	{#if scriptFilteredEntries.length === 0}
		<div class="gp-list-empty">
			{#if isLoading}
				<div class="gp-loading">{$t('indexPanel.building') || 'Building index…'}</div>
			{:else}
				<div class="gp-empty">{$t('indexPanel.noTerms')}</div>
			{/if}
		</div>
	{:else}
		<VirtualList
			items={rows}
			getItemHeight={getRowHeight}
			{scrollResetKey}
			overscan={12}
		>
			{#snippet row(r, _i)}
				{#if r.kind === 'header'}
					<div class="gp-letter" data-letter={r.letter}>{r.letter}</div>
				{:else if r.kind === 'entry'}
					{@const entry = r.entry}
					{@const isHidden = excludedTerms.has(entry.term.toLowerCase())}
					<div class="gp-entry" class:hidden-term={isHidden}>
						<button class="gp-term-row" oncontextmenu={(e) => handleContextMenu(e, entry.term)}>
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<span class="gp-chev-btn" onclick={() => toggleExpand(entry.term)}>
								<svg class="gp-chev" class:expanded={expandedTerms.has(entry.term)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
							</span>
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<span class="gp-term-name" dir="auto" class:term-selected={selectedTerms.has(entry.term)}
								onclick={async (e) => {
									if ((e.ctrlKey || e.metaKey) && onTermSelect) {
										await ensureMentionsLoaded(entry.term);
										onTermSelect(entry.term, getMentions(entry.term), !selectedTerms.has(entry.term));
									} else if (onTermClick) {
										await ensureMentionsLoaded(entry.term);
										onTermClick(entry.term, getMentions(entry.term));
										toggleExpand(entry.term);
									} else {
										toggleExpand(entry.term);
									}
								}}>
								{entry.term}
								{#if entry.is_compound}<span class="gp-compound-badge">2w</span>{/if}
								{#if bridgeFilterAnnotations.has(entry.term)}
									{@const viaSource = bridgeFilterAnnotations.get(entry.term) ?? ''}
									<span class="gp-ref-via" dir="auto" title={$t('indexPanel.viaLemmaTooltip') || 'Cross-language match via the Lexical Bridge'}>{($t('indexPanel.viaLemma') || 'via {lemma}').replace('{lemma}', viaSource)}</span>
								{:else if semanticFilterAnnotations.has(entry.term)}
									<span class="gp-ref-semantic" dir="auto" title={$t('indexPanel.semanticMatchTooltip') || 'Semantic match — conceptually related to your query'}>{$t('indexPanel.semanticMatch') || '≈ similar'}</span>
								{/if}
							</span>
							<div class="gp-freq-wrap">
								<div class="gp-freq-bar" style="width: {(entry.count / maxCount) * 100}%"></div>
								<span class="gp-count">{entry.count}</span>
							</div>
						</button>
					</div>
				{:else}
					<div class="gp-references" dir="auto">
						{#each getMentions(r.term) as mention}
							<button class="gp-ref" class:active={mention.note_path === activeNotePath}
								class:has-snippet={!!mention.snippet}
								data-filepath={mention.note_path}
								onclick={(e) => onNoteClick(mention.note_path, mention.note_name, r.term, e)}
								onmouseenter={(e) => onNoteHover(mention.note_path, e)}
								onmouseleave={() => onNoteLeave()}>
								<span class="gp-ref-name" dir="auto">{mention.note_name}</span>
								{#if mention.via_lemma}
									<span class="gp-ref-via" dir="auto" title={$t('indexPanel.viaLemmaTooltip') || 'Cross-language match via the Lexical Bridge'}>{($t('indexPanel.viaLemma') || 'via {lemma}').replace('{lemma}', mention.via_lemma)}</span>
								{/if}
								{#if mention.snippet}
									<span class="gp-ref-snippet" dir="auto">
										{#each splitSnippet(mention.snippet) as part}
											{#if part.mark}<mark class="gp-ref-hit">{part.text}</mark>{:else}{part.text}{/if}
										{/each}
									</span>
								{/if}
								{#if summaryHeadlines.get(mention.note_path)}
									<span class="gp-ref-headline" dir="auto" title={summaryHeadlines.get(mention.note_path)}>{summaryHeadlines.get(mention.note_path)}</span>
								{/if}
							</button>
						{/each}
						<!-- Co-occurring terms chip strip: click to add to the comparison set. -->
						{#if loadingCooccurrence.has(r.term)}
							<div class="gp-cooccur gp-cooccur-loading" dir="auto">
								<span class="gp-cooccur-label">{$t('indexPanel.alsoAppearsWith') || 'Also appears with'}</span>
								<span class="gp-cooccur-loading-text">{$t('indexPanel.building') || 'Loading…'}</span>
							</div>
						{:else if getCooccurrence(r.term).length > 0}
							<div class="gp-cooccur" dir="auto">
								<span class="gp-cooccur-label">{$t('indexPanel.alsoAppearsWith') || 'Also appears with'}</span>
								<div class="gp-cooccur-chips">
									{#each getCooccurrence(r.term) as co}
										<button type="button" class="gp-cooccur-chip" dir="auto"
											title="{co.term} — {co.note_count} {co.note_count === 1 ? $t('indexPanel.noteWithAll') || 'note' : $t('indexPanel.notesWithAll') || 'notes'}"
											onclick={(e) => handleCooccurChipClick(co.term, e)}>
											<span class="gp-cooccur-term">{co.term}</span>
											<span class="gp-cooccur-count">{co.note_count}</span>
										</button>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{/if}
			{/snippet}
		</VirtualList>
	{/if}
</div>

<!-- Context menu (MIG-077 A2 — shared ContextMenu) -->
{#if contextMenu}
	<ContextMenu
		x={contextMenu.x}
		y={contextMenu.y}
		items={getIndexTermMenuItems(contextMenu.term)}
		onClose={closeContextMenu}
	/>
{/if}

<style>
	.index-panel {
		font-size: 0.8rem;
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	/* ── Toolbar ── */
	.gp-toolbar {
		padding: 6px 8px;
		border-bottom: 1px solid var(--border);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.gp-search {
		position: relative; /* anchor for .gp-history-dropdown (MIG-012) */
		display: flex;
		align-items: center;
		gap: 4px;
		background: var(--background-modifier-form-field);
		border-radius: 4px;
		padding: 2px 6px;
	}
	.gp-search input {
		border: none;
		background: none;
		outline: none;
		font-family: inherit;
		font-size: 0.78rem;
		color: var(--text-normal);
		flex: 1;
		min-width: 0;
	}
	.gp-search-icon {
		color: var(--text-faint);
		flex-shrink: 0;
	}
	.gp-clear {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-faint);
		font-size: 0.9rem;
		padding: 0 2px;
	}
	/* MIG-012 — search history dropdown. Floats below the filter input
	   when the toggle is on, the input is focused, and the query is
	   empty. Mousedown (not click) handler on items so onblur doesn't
	   dismiss before the selection lands. */
	.gp-history-dropdown {
		position: absolute;
		top: 100%;
		inset-inline-start: 0;
		inset-inline-end: 0;
		margin-top: 2px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.08);
		z-index: 100;
		max-height: 280px;
		overflow-y: auto;
		padding: 4px 0;
	}
	.gp-history-header {
		padding: 4px 12px;
		font-size: 0.65rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-faint);
	}
	.gp-history-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		padding: 6px 12px;
		background: none;
		border: none;
		text-align: start;
		cursor: pointer;
		font-size: 0.78rem;
		color: var(--text-normal);
	}
	.gp-history-item:hover {
		background: var(--background-modifier-hover);
	}
	.gp-history-q {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.gp-history-count {
		flex-shrink: 0;
		font-size: 0.65rem;
		color: var(--text-faint);
	}
	.gp-actions {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.gp-total {
		font-size: 0.7rem;
		color: var(--text-faint);
	}
	.gp-btns {
		display: flex;
		gap: 2px;
	}
	.gp-icon-btn {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-faint);
		padding: 2px 4px;
		border-radius: 3px;
		display: flex;
		align-items: center;
	}
	.gp-icon-btn:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.gp-icon-btn.active {
		color: var(--interactive-accent);
	}

	/* ── Language script tabs ── */
	.gp-scripts {
		display: flex;
		gap: 4px;
		padding: 4px 8px;
		border-bottom: 1px solid var(--border);
		flex-wrap: wrap;
	}
	.gp-script-btn {
		background: var(--background-modifier-form-field);
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 2px 10px;
		font-size: 0.72rem;
		font-family: inherit;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}
	.gp-script-btn:hover {
		background: var(--background-modifier-hover);
	}
	.gp-script-btn.active {
		background: var(--interactive-accent);
		color: var(--text-on-accent);
		border-color: var(--interactive-accent);
	}

	/* ── Alphabet navigation bar ── */
	.gp-alphabet {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 4px 6px;
		border-bottom: 1px solid var(--border);
	}
	.gp-alpha-row {
		display: flex;
		flex-wrap: wrap;
		gap: 1px;
		justify-content: center;
	}
	.gp-alpha-btn {
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		font-size: 0.68rem;
		font-weight: 600;
		color: var(--interactive-accent);
		padding: 1px 3px;
		border-radius: 3px;
		min-width: 16px;
		text-align: center;
		line-height: 1.4;
	}
	.gp-alpha-btn:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.gp-alpha-btn.active {
		background: var(--interactive-accent);
		color: white;
		border-radius: 4px;
	}

	/* ── List ──
	   The list itself is now a <VirtualList> component (.vlist) — it owns
	   flex:1 and overflow:auto. We keep .gp-list-empty as a wrapper for the
	   empty / loading states only, so the layout still fills the pane. */
	.gp-list-empty {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		padding: 4px 0;
	}
	.gp-empty {
		color: var(--color-base-40);
		font-size: 0.78rem;
		padding: 12px 8px;
		text-align: center;
	}
	.gp-loading {
		color: var(--text-faint);
		font-size: 0.74rem;
		padding: 8px;
		text-align: center;
	}

	/* ── Letter header (rendered as its own virtualized row) ── */
	.gp-letter {
		font-weight: 700;
		font-size: 0.75rem;
		color: var(--interactive-accent);
		text-transform: uppercase;
		padding: 6px 10px 2px;
		letter-spacing: 0.08em;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border);
	}

	/* ── Term entry ──
	   Virtualized: each rendered row lives inside VirtualList's absolutely
	   positioned slot. No multi-column layout, no content-visibility — the
	   virtual list already skips off-screen rows entirely. */
	.gp-entry {
		padding: 0 4px;
	}
	.gp-entry.hidden-term {
		opacity: 0.4;
	}
	.gp-term-row {
		display: flex;
		align-items: center;
		gap: 4px;
		width: 100%;
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		padding: 4px 6px;
		border-radius: 4px;
		color: var(--text-normal);
		font-size: 0.8rem;
	}
	.gp-term-row:hover {
		background: var(--background-modifier-hover);
	}
	.gp-chev-btn {
		display: flex; align-items: center; justify-content: center;
		width: 20px; height: 20px; flex-shrink: 0; cursor: pointer;
		border-radius: 3px;
	}
	.gp-chev-btn:hover { background: var(--background-modifier-hover); }
	.gp-chev {
		transition: transform 0.15s ease;
		flex-shrink: 0;
		color: var(--text-faint);
	}
	.gp-chev.expanded {
		transform: rotate(90deg);
	}
	.gp-term-name {
		flex: 1;
		text-align: start;
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.gp-term-name.term-selected {
		color: var(--interactive-accent); font-weight: 700;
	}

	/* Anchor bar for selected terms */
	.gp-anchor-bar {
		display: flex; align-items: center; gap: 6px;
		padding: 6px 12px; flex-shrink: 0;
		background: color-mix(in srgb, var(--interactive-accent) 6%, var(--background-primary));
		border-bottom: 1px solid color-mix(in srgb, var(--interactive-accent) 20%, transparent);
		flex-wrap: wrap;
	}
	.gp-anchor-label {
		font-size: 11px; font-weight: 600; color: var(--text-muted);
		white-space: nowrap;
	}
	.gp-anchor-chip {
		display: inline-flex; align-items: center; gap: 4px;
		padding: 2px 8px; border-radius: 12px;
		background: var(--interactive-accent); color: white;
		border: none; font-size: 12px; font-weight: 600;
		cursor: pointer; font-family: inherit;
		transition: opacity 0.15s;
	}
	.gp-anchor-chip:hover { opacity: 0.8; }
	.gp-anchor-chip svg { opacity: 0.7; }
	.gp-anchor-chip:hover svg { opacity: 1; }
	.gp-anchor-clear {
		font-size: 11px; color: var(--text-muted);
		background: none; border: none; cursor: pointer;
		text-decoration: underline; padding: 2px 4px;
	}
	.gp-anchor-clear:hover { color: var(--text-normal); }

	/* Commonality panel (notes containing all selected terms) */
	.gp-commonality {
		flex-shrink: 0;
		max-height: 40vh;
		display: flex;
		flex-direction: column;
		background: color-mix(in srgb, var(--interactive-accent) 3%, var(--background-primary));
		border-bottom: 1px solid var(--border);
	}
	.gp-commonality-header {
		font-size: 11px;
		color: var(--text-muted);
		padding: 6px 12px;
		font-weight: 600;
		flex-shrink: 0;
	}
	.gp-commonality-empty {
		font-size: 11px;
		color: var(--text-faint);
		padding: 6px 12px;
		font-style: italic;
	}
	.gp-commonality-list {
		overflow-y: auto;
		padding: 0 8px 6px;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.gp-commonality-list .gp-ref {
		padding: 3px 6px;
	}
	.gp-compound-badge {
		font-size: 0.55rem;
		font-weight: 700;
		color: var(--interactive-accent);
		background: color-mix(in srgb, var(--interactive-accent) 15%, transparent);
		padding: 0 3px;
		border-radius: 3px;
		line-height: 1.4;
		flex-shrink: 0;
	}

	/* ── Frequency bar + count ── */
	.gp-freq-wrap {
		position: relative;
		min-width: 40px;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		flex-shrink: 0;
	}
	.gp-freq-bar {
		position: absolute;
		inset: 0;
		background: color-mix(in srgb, var(--interactive-accent) 10%, transparent);
		border-radius: 2px;
	}
	.gp-count {
		position: relative;
		color: var(--text-faint);
		font-size: 0.7rem;
		z-index: 1;
		padding: 0 3px;
	}

	/* ── Note references ── */
	.gp-references {
		padding: 3px 10px 6px 22px;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.gp-ref {
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		font-size: 0.74rem;
		color: var(--interactive-accent);
		padding: 2px 4px;
		border-radius: 3px;
		text-decoration: none;
		text-align: start;
		display: flex;
		flex-direction: column;
		align-items: stretch;
		gap: 1px;
		min-width: 0;
		max-width: 100%;
	}
	.gp-ref-name {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.gp-ref.active .gp-ref-name {
		font-weight: 700;
	}
	.gp-ref:hover {
		background: var(--background-modifier-hover);
	}
	.gp-ref:hover .gp-ref-name {
		text-decoration: underline;
	}
	/* MIG-012 — semantic match badge. Renders as a small inline chip
	   after the term name when a row surfaced because of an embedding-
	   space cosine match (not substring, not lexical bridge). Visually
	   distinct from .gp-ref-via via a different accent (using
	   --color-cyan / --color-blue mix vs. accent-purple) so users can
	   tell at a glance which signal surfaced the row. */
	.gp-ref-semantic {
		display: inline-block;
		font-size: 0.65rem;
		font-weight: 500;
		color: var(--text-muted);
		background: color-mix(in srgb, var(--color-cyan, #06b6d4) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-cyan, #06b6d4) 24%, transparent);
		border-radius: 3px;
		padding: 1px 5px;
		margin-inline-start: 6px;
		vertical-align: baseline;
		white-space: nowrap;
	}
	/* MIG-010 — cross-language bridge badge. Renders as a small inline
	   chip after the note name when a row surfaced because of M11
	   Lexical Bridge expansion. Logical (start/end) properties so RTL
	   users see it on the correct side; `dir="auto"` on the element
	   itself so the lemma's own script direction reads naturally. */
	.gp-ref-via {
		display: inline-block;
		font-size: 0.65rem;
		font-weight: 500;
		color: var(--text-muted);
		background: color-mix(in srgb, var(--interactive-accent) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent) 24%, transparent);
		border-radius: 3px;
		padding: 1px 5px;
		margin-inline-start: 6px;
		vertical-align: baseline;
		white-space: nowrap;
	}
	/* One-line context snippet beneath the note name. Each snippet carries
	   its own dir="auto" in the template so a Hebrew/Arabic snippet under
	   a Latin note name (or vice versa) still reads in the correct
	   direction. */
	.gp-ref-snippet {
		display: block;
		font-size: 0.68rem;
		line-height: 1.35;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		text-align: start;
		font-weight: 400;
	}
	.gp-ref-hit {
		background: color-mix(in srgb, var(--interactive-accent) 24%, transparent);
		color: var(--text-normal);
		border-radius: 2px;
		padding: 0 2px;
		font-weight: 600;
	}
	/* MIG-044 Phase 2 — NSC summary headline under each mention row.
	   Shares the cross-surface visual grammar: italic, muted, single-line
	   ellipsis. Renders below the snippet (or below the name if no snippet). */
	.gp-ref-headline {
		display: block;
		font-size: 0.68rem;
		line-height: 1.35;
		color: var(--text-faint);
		font-style: italic;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		text-align: start;
	}

	/* ── Co-occurring terms chip strip ── */
	/* Sits below the references list inside an expanded-row. The label +
	   chips are plain flex; chips wrap to multiple rows when the panel is
	   narrow, which getRowHeight accounts for by reserving two lines. */
	.gp-cooccur {
		margin-top: 6px;
		padding-top: 5px;
		border-top: 1px dashed var(--border);
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.gp-cooccur-label {
		font-size: 0.66rem;
		font-weight: 600;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}
	.gp-cooccur-loading-text {
		font-size: 0.7rem;
		color: var(--text-muted);
		font-style: italic;
	}
	.gp-cooccur-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
	}
	.gp-cooccur-chip {
		background: var(--background-modifier-form-field);
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 1px 7px 1px 8px;
		font-family: inherit;
		font-size: 0.7rem;
		color: var(--text-muted);
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		line-height: 1.5;
		transition: background-color 0.08s, color 0.08s, border-color 0.08s;
	}
	.gp-cooccur-chip:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
		border-color: color-mix(in srgb, var(--interactive-accent) 35%, var(--border));
	}
	.gp-cooccur-term {
		font-weight: 500;
	}
	.gp-cooccur-count {
		font-size: 0.62rem;
		color: var(--text-faint);
		font-variant-numeric: tabular-nums;
	}

	/* ── RTL support ── */
	:global([dir="rtl"]) .gp-chev {
		transform: rotate(180deg);
	}
	:global([dir="rtl"]) .gp-chev.expanded {
		transform: rotate(90deg);
	}
	:global([dir="rtl"]) .gp-references {
		padding: 3px 22px 6px 10px;
	}
	:global([dir="rtl"]) .gp-letter {
		text-align: right;
	}
</style>

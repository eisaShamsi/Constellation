<script lang="ts">
	import { t } from '$lib/i18n';
	import type { IndexEntry, IndexMention } from '$lib/vaults/store';

	let {
		entries = [] as IndexEntry[],
		onNoteClick,
	}: {
		entries: IndexEntry[];
		onNoteClick: (filePath: string, noteName: string, term?: string, e?: MouseEvent) => void;
	} = $props();

	let filterQuery = $state('');
	let expandedTerms = $state<Set<string>>(new Set());
	let visibleCount = $state(200);
	let activeScript = $state<string>('all');
	let sortMode = $state<'alpha' | 'freq'>('alpha');
	let excludedTerms = $state<Set<string>>(loadExcluded());
	let showHidden = $state(false);
	let listEl: HTMLDivElement | undefined;
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
	const filteredEntries = $derived.by(() => {
		const raw = filterQuery.trim();
		let result = entries;
		// Apply exclusion filter
		if (!showHidden) {
			result = result.filter(e => !excludedTerms.has(e.term.toLowerCase()));
		}
		if (!raw) return result;
		const hasComma = /[،,؛;]/.test(raw);
		if (hasComma) {
			const terms = raw.split(/[،,؛;]/).map(t => t.trim().toLowerCase()).filter(Boolean);
			if (terms.length === 0) return result;
			return result.filter(e => {
				const lower = e.term.toLowerCase();
				return terms.some(t => lower === t);
			});
		} else {
			const q = raw.toLowerCase();
			return result.filter(e => e.term.toLowerCase().includes(q));
		}
	});

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

	const scriptFilteredEntries = $derived.by(() => {
		if (activeScript === 'all') return filteredEntries;
		return filteredEntries.filter(e => {
			const first = e.term[0];
			return first && getScript(first) === activeScript;
		});
	});

	const visibleEntries = $derived(scriptFilteredEntries.slice(0, visibleCount));
	const hasMore = $derived(scriptFilteredEntries.length > visibleCount);
	const maxCount = $derived(Math.max(1, ...scriptFilteredEntries.map(e => e.count)));

	// Alphabetical mode: group by letter
	const groupedEntries = $derived.by(() => {
		if (sortMode === 'freq') return [];
		const groups = new Map<string, IndexEntry[]>();
		for (const entry of visibleEntries) {
			const { letter } = getIndexInfo(entry.term);
			const group = groups.get(letter) ?? [];
			group.push(entry);
			groups.set(letter, group);
		}
		for (const [, group] of groups) {
			group.sort((a, b) => {
				const ia = getIndexInfo(a.term);
				const ib = getIndexInfo(b.term);
				return ia.sortKey.localeCompare(ib.sortKey);
			});
		}
		return Array.from(groups.entries()).sort((a, b) => compareLetters(a[0], b[0]));
	});

	// Frequency mode: flat sorted list
	const freqEntries = $derived.by(() => {
		if (sortMode !== 'freq') return [];
		return [...visibleEntries].sort((a, b) => b.count - a.count);
	});

	const allLetters = $derived.by(() => {
		if (sortMode === 'freq') return [];
		const letters = new Set<string>();
		for (const entry of scriptFilteredEntries) {
			const { letter } = getIndexInfo(entry.term);
			letters.add(letter);
		}
		return Array.from(letters).sort(compareLetters);
	});

	const totalTerms = $derived(entries.length);
	const hiddenCount = $derived(entries.filter(e => excludedTerms.has(e.term.toLowerCase())).length);

	function toggleExpand(term: string) {
		if (expandedTerms.has(term)) expandedTerms.delete(term);
		else expandedTerms.add(term);
		expandedTerms = new Set(expandedTerms);
	}

	function handleScroll(e: Event) {
		const el = e.target as HTMLElement;
		if (hasMore && el.scrollTop + el.clientHeight >= el.scrollHeight - 100) {
			visibleCount += 200;
		}
	}

	function scrollToLetter(letter: string) {
		const idx = scriptFilteredEntries.findIndex(e => getIndexInfo(e.term).letter === letter);
		if (idx >= 0 && idx >= visibleCount) {
			visibleCount = idx + 200;
		}
		requestAnimationFrame(() => {
			const el = listEl?.querySelector(`[data-letter="${CSS.escape(letter)}"]`);
			if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
		});
	}

	function handleContextMenu(e: MouseEvent, term: string) {
		e.preventDefault();
		contextMenu = { x: e.clientX, y: e.clientY, term };
	}

	function closeContextMenu() {
		contextMenu = null;
	}

	// ─── Export ───
	function exportToClipboard() {
		const source = sortMode === 'freq' ? freqEntries : visibleEntries;
		let md = '# Index\n\n';
		if (sortMode === 'freq') {
			for (const entry of source) {
				const notes = entry.mentions.map(m => m.note_name).join(', ');
				md += `- ${entry.term} (${entry.count}) — ${notes}\n`;
			}
		} else {
			for (const [letter, group] of groupedEntries) {
				md += `## ${letter}\n`;
				for (const entry of group) {
					const notes = entry.mentions.map(m => m.note_name).join(', ');
					md += `- ${entry.term} (${entry.count}) — ${notes}\n`;
				}
				md += '\n';
			}
		}
		navigator.clipboard.writeText(md);
	}

	$effect(() => {
		filterQuery;
		activeScript;
		sortMode;
		visibleCount = 200;
	});

	// Close context menu on click outside
	function handleWindowClick() { contextMenu = null; }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="index-panel">
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
			/>
			{#if filterQuery}
				<button class="gp-clear" onclick={() => filterQuery = ''}>×</button>
			{/if}
		</div>
		<div class="gp-actions">
			<span class="gp-total">{totalTerms} {$t('indexPanel.terms')}</span>
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
					<button class="gp-icon-btn" class:active={showHidden} onclick={() => showHidden = !showHidden} title="{hiddenCount} hidden terms">
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
	{#if sortMode === 'alpha' && allLetters.length > 0}
		<div class="gp-alphabet">
			{#each allLetters as letter}
				<button class="gp-alpha-btn" onclick={() => scrollToLetter(letter)}>{letter}</button>
			{/each}
		</div>
	{/if}

	<!-- Term list -->
	<div class="gp-list" bind:this={listEl} onscroll={handleScroll}>
		{#if scriptFilteredEntries.length === 0}
			<div class="gp-empty">{$t('indexPanel.noTerms')}</div>
		{:else if sortMode === 'alpha'}
			{#each groupedEntries as [letter, group]}
				<div class="gp-letter-group">
					<div class="gp-letter" data-letter={letter}>{letter}</div>
					{#each group as entry}
						{@const isHidden = excludedTerms.has(entry.term.toLowerCase())}
						<div class="gp-entry" class:hidden-term={isHidden}>
							<button class="gp-term-row" onclick={() => toggleExpand(entry.term)} oncontextmenu={(e) => handleContextMenu(e, entry.term)}>
								<svg class="gp-chev" class:expanded={expandedTerms.has(entry.term)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<span class="gp-term-name">
									{entry.term}
									{#if entry.is_compound}<span class="gp-compound-badge">2w</span>{/if}
								</span>
								<div class="gp-freq-wrap">
									<div class="gp-freq-bar" style="width: {(entry.count / maxCount) * 100}%"></div>
									<span class="gp-count">{entry.count}</span>
								</div>
							</button>

							{#if expandedTerms.has(entry.term)}
								<div class="gp-references">
									{#each entry.mentions as mention, i}
										<button class="gp-ref" onclick={(e) => onNoteClick(mention.note_path, mention.note_name, entry.term, e)}>
											{mention.note_name}</button>{#if i < entry.mentions.length - 1}<span class="gp-sep">,</span>{/if}
									{/each}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/each}
		{:else}
			{#each freqEntries as entry}
				{@const isHidden = excludedTerms.has(entry.term.toLowerCase())}
				<div class="gp-entry" class:hidden-term={isHidden}>
					<button class="gp-term-row" onclick={() => toggleExpand(entry.term)} oncontextmenu={(e) => handleContextMenu(e, entry.term)}>
						<svg class="gp-chev" class:expanded={expandedTerms.has(entry.term)} width="8" height="8" viewBox="0 0 10 10">
							<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
						</svg>
						<span class="gp-term-name">
							{entry.term}
							{#if entry.is_compound}<span class="gp-compound-badge">2w</span>{/if}
						</span>
						<div class="gp-freq-wrap">
							<div class="gp-freq-bar" style="width: {(entry.count / maxCount) * 100}%"></div>
							<span class="gp-count">{entry.count}</span>
						</div>
					</button>

					{#if expandedTerms.has(entry.term)}
						<div class="gp-references">
							{#each entry.mentions as mention, i}
								<button class="gp-ref" onclick={(e) => onNoteClick(mention.note_path, mention.note_name, entry.term, e)}>
									{mention.note_name}</button>{#if i < entry.mentions.length - 1}<span class="gp-sep">,</span>{/if}
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		{/if}
		{#if hasMore}
			<div class="gp-loading">{$t('indexPanel.terms')}...</div>
		{/if}
	</div>
</div>

<!-- Context menu -->
{#if contextMenu}
	<div class="gp-context-menu" style="left: {contextMenu.x}px; top: {contextMenu.y}px">
		{#if excludedTerms.has(contextMenu.term.toLowerCase())}
			<button onclick={() => { unhideTerm(contextMenu!.term.toLowerCase()); contextMenu = null; }}>Show term</button>
		{:else}
			<button onclick={() => hideTerm(contextMenu!.term.toLowerCase())}>Hide term</button>
		{/if}
	</div>
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
		flex-wrap: wrap;
		gap: 1px;
		padding: 4px 6px;
		border-bottom: 1px solid var(--border);
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

	/* ── List ── */
	.gp-list {
		overflow-y: auto;
		flex: 1;
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

	/* ── Letter groups ── */
	.gp-letter-group {
		margin-bottom: 2px;
	}
	.gp-letter {
		font-weight: 700;
		font-size: 0.75rem;
		color: var(--interactive-accent);
		text-transform: uppercase;
		padding: 6px 10px 2px;
		letter-spacing: 0.08em;
		position: sticky;
		top: 0;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border);
	}

	/* ── Term entry ── */
	.gp-entry {
		padding: 0 4px;
		margin-bottom: 2px;
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
		flex-wrap: wrap;
		align-items: baseline;
		gap: 1px;
	}
	.gp-ref {
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		font-size: 0.74rem;
		color: var(--interactive-accent);
		padding: 1px 2px;
		border-radius: 2px;
		text-decoration: none;
	}
	.gp-ref:hover {
		text-decoration: underline;
		background: var(--background-modifier-hover);
	}
	.gp-sep {
		color: var(--text-faint);
		font-size: 0.74rem;
		margin-inline-end: 2px;
	}

	/* ── Context menu ── */
	.gp-context-menu {
		position: fixed;
		z-index: 9999;
		background: var(--background-primary);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 4px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.15);
		min-width: 120px;
	}
	.gp-context-menu button {
		display: block;
		width: 100%;
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		font-size: 0.78rem;
		color: var(--text-normal);
		padding: 6px 10px;
		border-radius: 4px;
		text-align: start;
	}
	.gp-context-menu button:hover {
		background: var(--background-modifier-hover);
	}
</style>

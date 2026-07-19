<script lang="ts">
	// MIG-086 — the shared "Connect to:" surface. Turns a diagnosis ("this orphan needs a
	// link") into an ACTION: a ranked list of related-but-unlinked notes, each with the
	// distinctive terms that explain WHY (the concept's mandatory legible reason), and a
	// one-click typed-link button. Hosted in the Reviewer, the NotePane sidebar, the 360
	// Inspector, the note's Health tab, and the Sky View node menu (the same component
	// everywhere — never relearn it). §B: display only; the Link button is wired in §C.
	//
	// Concept invariants (do not regress): one Link button PER candidate (never "Link all");
	// the shared-term *why* is mandatory; invitational, not assertive; only UNCONNECTED
	// relatives; an honest empty state (never a fabricated match).
	import { t, dir as uiDir } from '$lib/i18n';
	// PJ-114 §3b — Suggested Connections joins the app-drawn tooltip. The name and snippet use
	// the `detectDir` already imported below (script DOMINANCE, not dir="auto"'s first strong
	// character — PJ-106 §A1): a candidate titled in Latin but written in Arabic still lays out RTL.
	import '$lib/links/linkTip';
	import { suggestRelatedNotes, addLinkToNote, type RelatedCandidate } from '$lib/libraries/store';
	import { detectDir } from '$lib/utils';
	import LinkTypePicker from './LinkTypePicker.svelte';

	let {
		notePath,
		noteName,                    // §C — the in-hand note's display name = the typed link's TARGET
		libraryPath,
		defaultType = 'associative', // §C — pre-selects the 8-type picker (orphan→associative, fragile→derives-from)
		heading = null,              // override the default "Connect to:" label (fragile = "Shore it up — connect to:")
		onConnected,                 // §C — called with the linked candidate's path after a successful connect
		direction = 'inbound',       // §D — host-set link direction (see choose()). NOT a user toggle (PJ-067).
	}: {
		notePath: string | null;
		noteName?: string | null;
		libraryPath: string | null;
		defaultType?: string;
		heading?: string | null;
		onConnected?: (linkedPath: string) => void;
		direction?: 'inbound' | 'outbound';
	} = $props();

	let candidates = $state<RelatedCandidate[]>([]);
	let loading = $state(false);
	let loaded = $state(false); // distinguishes "not fetched yet" from "fetched, empty"
	// Dedup key is a PLAIN (non-reactive) var — NOT $state. Reading+writing a tracked $state
	// inside this effect would self-retrigger it (Rule 2 $effect loop), and the re-run's
	// cleanup would cancel the in-flight fetch before it resolves → permanent "loading".
	let lastKey: string | null = null;

	// §C — one-click typed-link action state. Declared above the fetch $effect so that
	// effect can reset them on a note change (a stale picker after a note switch would
	// otherwise let choose() write to the WRONG target — closed below, host-independent).
	let picker = $state<{ candidate: RelatedCandidate; x: number; y: number } | null>(null);
	let connecting = $state<string | null>(null); // candidate path mid-connect (disables its button)

	// Fire ONCE per note (keyed on notePath) — never on every render, never per-keystroke
	// (Rule 1/3). A note change re-queries; the cleanup cancels any in-flight stale response.
	$effect(() => {
		const path = notePath;
		const lib = libraryPath;
		if (!path || !lib) {
			candidates = [];
			loaded = false;
			lastKey = null;
			picker = null;       // §C — no stale picker/connecting across a note change
			connecting = null;
			return;
		}
		const key = `${lib}\u0000${path}`;
		if (key === lastKey) return; // already fetched for this note
		lastKey = key;
		picker = null;           // §C — the note changed: drop any open picker + in-flight
		connecting = null;       //      connect so choose() can never target the old note
		loading = true;
		loaded = false;
		let cancelled = false;
		// No count cap (Boss): request ALL related notes the engine ranks (the §A BM25 pool
		// is the only ceiling), sequenced closest-first. The list below scrolls if long.
		suggestRelatedNotes(lib, path)
			.then((res) => {
				if (!cancelled) {
					candidates = res;
					loaded = true;
				}
			})
			.catch(() => {
				if (!cancelled) {
					candidates = [];
					loaded = true;
				}
			})
			.finally(() => {
				if (!cancelled) loading = false;
			});
		return () => {
			cancelled = true;
		};
	});

	const labelText = $derived(heading ?? ($t('reviewer.suggestLabel') || 'Connect to:'));

	// The block mirrors when EITHER the UI is RTL (labels are localized → "Connect to:" /
	// "shared:" must right-align in an Arabic UI, incl. the empty state) OR the note content
	// is RTL (an Arabic note's cards mirror even in an LTR UI). The note names/snippets keep
	// dir="auto" for per-item content direction within that frame. (Both Boss RTL findings.)
	const contentRTL = $derived(
		candidates.length > 0 && detectDir(candidates.map((c) => c.note_name).join(' ')) === 'rtl'
	);
	const blockDir = $derived($uiDir === 'rtl' || contentRTL ? 'rtl' : 'ltr');

	// §C/§D — the one-click typed-link action. Clicking Link opens the 8-type picker
	// (C-1: ALWAYS typed, one candidate at a time, never a bulk accept). On choose,
	// addLinkToNote declares the link as a frontmatter type-as-property on the SOURCE
	// note (§F2); index_note derives the note_links row at confidence 'hypothesis' (C-4).
	// The host sets `direction` (NOT a user In/Out/Both toggle — that's PJ-067):
	//   • inbound  (diagnostic hosts: Reviewer orphan/fragile, 360 Inspector, Health) —
	//     suggestion → in-hand note. The link lives in the CANDIDATE's frontmatter
	//     pointing at the in-hand note, so an orphan in hand GAINS an incoming link and
	//     leaves the orphan lens (de-orphan / shore-up). source=cand, target=noteName.
	//   • outbound (general hosts: NotePane Backlinks tab, Sky node) — in-hand note →
	//     suggestion. The link lives in the IN-HAND note's frontmatter pointing at the
	//     candidate. source=notePath, target=cand.note_name.
	// (picker/connecting are declared above so the fetch $effect can reset them per note.)
	function openPicker(e: MouseEvent, c: RelatedCandidate) {
		e.preventDefault();
		e.stopPropagation();
		picker = { candidate: c, x: e.clientX, y: e.clientY };
	}

	async function choose(type: string) {
		const cand = picker?.candidate;
		picker = null;
		if (!cand || !libraryPath) return;
		// Resolve (source, target) by the host-set direction. Bail BEFORE the mid-connect
		// state if the required endpoint is missing — no half-applied connect, no stuck
		// spinner (the early return precedes `connecting = …`).
		const source = direction === 'outbound' ? notePath : cand.note_path;
		const target = direction === 'outbound' ? cand.note_name : noteName;
		if (!source || !target) return;
		// Snapshot the in-hand note identity. The write below targets the captured (correct)
		// `source`, but if the host swaps to a DIFFERENT note while addLinkToNote is in flight,
		// the note-change $effect has already reset + re-queried `candidates` for the NEW note —
		// so the optimistic removal / onConnected must NOT fire against that fresh list (it would
		// wrongly drop a candidate of the new note). Guard on the snapshot after the await.
		const startNote = notePath;
		const startLib = libraryPath;
		connecting = cand.note_path; // keyed on the clicked candidate's row in both directions
		try {
			await addLinkToNote(source, type, target);
			if (notePath !== startNote || libraryPath !== startLib) return; // note switched mid-connect
			// Optimistically drop the just-connected candidate (it is now linked, so the
			// next §A query would anti-join it out anyway). Remaining candidates stay —
			// each is its own deliberate, typed act (C-1 allows many singles, never a bulk).
			candidates = candidates.filter((c) => c.note_path !== cand.note_path);
			onConnected?.(cand.note_path);
		} catch (err) {
			console.error('[RelatedCandidates] connect failed', err);
		} finally {
			connecting = null;
		}
	}
</script>

<div class="rc" dir={blockDir}>
	<span class="rc-label">{labelText}</span>

	{#if loading}
		<div class="rc-state">{$t('reviewer.suggestLoading') || 'Finding related notes…'}</div>
	{:else if loaded && candidates.length === 0}
		<div class="rc-state rc-empty">
			{$t('reviewer.suggestEmpty') ||
				'No strong matches in your Library yet — connect it manually, or mark it standalone.'}
		</div>
	{:else if candidates.length > 0}
		<ul class="rc-list">
			{#each candidates as c (c.note_path)}
				<li class="rc-item">
					<div class="rc-row">
						<span class="rc-name" dir={detectDir(c.note_name)} data-linktip={c.note_name}>{c.note_name}</span>
						<button
							class="rc-link"
							onclick={(e) => openPicker(e, c)}
							disabled={connecting === c.note_path}
						>
							{connecting === c.note_path ? '⏳' : '🔗'} {$t('reviewer.suggestLinkBtn') || 'Link'}
						</button>
					</div>
					{#if c.shared_terms.length}
						<div class="rc-why">
							<span class="rc-why-label">{$t('reviewer.suggestSharedTerms') || 'shared:'}</span>
							{c.shared_terms.join(' · ')}
						</div>
					{/if}
					{#if c.snippet}
						<div class="rc-snippet" dir={detectDir(c.snippet)} data-linktip={c.snippet}>{c.snippet}</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>

{#if picker}
	<LinkTypePicker
		x={picker.x}
		y={picker.y}
		{defaultType}
		onChoose={choose}
		onCancel={() => (picker = null)}
	/>
{/if}

<style>
	.rc {
		margin-top: 18px;
	}
	.rc-label {
		display: block; /* full-width so text-align:start puts it at the right edge in RTL */
		text-align: start;
		font-size: calc(0.66rem * var(--rs-scale, 1));
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--interactive-accent, #7c3aed);
		font-weight: 600;
	}
	.rc-state {
		font-size: calc(0.82rem * var(--rs-scale, 1));
		color: var(--text-muted, #888);
		margin-top: 6px;
		line-height: 1.4;
	}
	.rc-empty {
		font-style: italic;
	}
	.rc-list {
		list-style: none;
		margin: 8px 0 0;
		padding: 0;
		/* The list is now uncapped (all related notes, closest-first) — bound its height so a
		   long list scrolls in place instead of pushing the rest of the host pane off-screen. */
		max-height: 60vh;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.rc-item {
		padding: 8px 10px;
		border-radius: 8px;
		background: var(--background-secondary, rgba(0, 0, 0, 0.03));
	}
	.rc-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.rc-name {
		flex: 1;
		min-width: 0;
		font-size: calc(0.9rem * var(--rs-scale, 1));
		color: var(--text-normal);
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.rc-link {
		flex-shrink: 0;
		font-size: calc(0.78rem * var(--rs-scale, 1));
		padding: 3px 8px;
		border-radius: 6px;
		border: 1px solid var(--background-modifier-border, #ccc);
		background: var(--background-primary, #fff);
		color: var(--text-normal);
		cursor: pointer;
		white-space: nowrap;
	}
	.rc-link:disabled {
		opacity: 0.5;
		cursor: default;
	}
	.rc-why {
		margin-top: 3px;
		font-size: calc(0.72rem * var(--rs-scale, 1));
		color: var(--text-muted, #888);
		line-height: 1.35;
	}
	.rc-why-label {
		color: var(--interactive-accent, #7c3aed);
	}
	.rc-snippet {
		margin-top: 3px;
		font-size: calc(0.74rem * var(--rs-scale, 1));
		color: var(--text-faint, #999);
		line-height: 1.35;
		max-height: 2.7em;
		overflow: hidden;
	}
	/* RTL: the container's dir (set from the candidates' language) drives flex direction
	   (name → start/right, Link → end/left) and text alignment — no manual row-reverse. */
</style>

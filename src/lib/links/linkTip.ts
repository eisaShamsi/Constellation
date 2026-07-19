/**
 * PJ-114 §3b — the living-link chip's tooltip, drawn by Constellation instead of by the OS.
 *
 * Concept (the horse): the `×N` chip answers "how much has this connection carried thought?"
 * — and the answer has to be READABLE. A native `title` gives us no say over where the box
 * lands or how big it is, so on the Boss test the English box was measured wider than its own
 * text and the Arabic string was crowded against the frame (its tanwin — the mark above the
 * final ـً in "يومًا" — collided with the top edge).
 *
 * This is the SECOND time this exact native-tooltip failure has been reported in this app:
 * `StructuralOutlinePanel.svelte:125` records the "bleeding-tip fix" of 2026-06-28, where a
 * native `title` bled off the sidebar edge in WebView2 and was replaced with a clamped
 * `position: fixed` box. Same cause, same answer.
 *
 * WHY A PLAIN MODULE AND NOT A SVELTE COMPONENT: one of the three chips lives inside a
 * CodeMirror `WidgetType` (`livePreview.ts`), which builds raw DOM and cannot mount a Svelte
 * component. A component would serve two call sites and fail the third — and shipping a third
 * copy of the PRESENTATION, immediately after `linkDisplay.ts` consolidated three copies of
 * the WORDS, would be the same mistake one layer up.
 *
 * THE CONTRACT: an element declares its own tooltip text in `data-linktip`. Nothing else.
 * No listener per chip, no component per row, nothing to tear down — which is what makes it
 * safe inside a virtualized list whose rows re-run on every scroll tick, and inside a CM6
 * widget that is created and discarded on every decoration rebuild.
 */

/** The attribute a chip sets to declare its tooltip. Resolved with `closest()`, so it may sit
 *  on the chip or an ancestor — but keep it on the chip itself, since the box anchors to the
 *  element that carries it. */
export const LINK_TIP_ATTR = 'data-linktip';

/** How long the pointer must rest before the box appears. Matches the house hover-intent
 *  values (`+layout.svelte` uses 300ms for index rows, 400ms for wikilink previews). */
const HOVER_INTENT_MS = 250;

/** Distance from the anchor, and the minimum distance from any viewport edge. */
const GAP = 8;
const PAD = 8;

let el: HTMLDivElement | null = null;
let anchor: HTMLElement | null = null;
let armTimer: ReturnType<typeof setTimeout> | null = null;

/** Dismissers live only while a box is on screen, so the idle cost of this module is the two
 *  delegated listeners below and nothing else. */
function addDismissers() {
	document.addEventListener('scroll', hideLinkTip, true);
	document.addEventListener('wheel', hideLinkTip, { capture: true, passive: true });
	document.addEventListener('keydown', hideLinkTip, true);
	window.addEventListener('blur', hideLinkTip);
}
function removeDismissers() {
	document.removeEventListener('scroll', hideLinkTip, true);
	document.removeEventListener('wheel', hideLinkTip, true);
	document.removeEventListener('keydown', hideLinkTip, true);
	window.removeEventListener('blur', hideLinkTip);
}

/** Hide the box and forget the anchor. Idempotent — safe to call from any dismisser. */
export function hideLinkTip(): void {
	if (armTimer) { clearTimeout(armTimer); armTimer = null; }
	if (!anchor && !el) return;
	removeDismissers();
	if (el) el.style.visibility = 'hidden';
	anchor = null;
}

function show(anchorEl: HTMLElement, text: string): void {
	armTimer = null;
	// The anchor can vanish between arming and firing — a virtualized row recycling, or a CM6
	// decoration rebuild discarding the widget. Anchoring to a detached node would place the
	// box at 0,0.
	if (!anchorEl.isConnected) return;

	if (!el) {
		el = document.createElement('div');
		el.className = 'link-tip';
		el.setAttribute('role', 'tooltip');
		document.body.appendChild(el);
	}
	anchor = anchorEl;
	el.textContent = text;

	// TWO DIRECTION SIGNALS, DELIBERATELY SEPARATE — they disagree in exactly one real case:
	// an Arabic interface with an English note open. The tooltip's WORDS follow the interface
	// (Boss ruling, `livePreview.ts`), so the text must lay out RTL; but the chip sits inside
	// an LTR note, so the box must still open leftward into that note rather than outward over
	// the sidebar. Reading one signal for both would break one of the two.
	el.dir = document.documentElement.dir === 'rtl' ? 'rtl' : 'ltr';
	const rtlAnchor = getComputedStyle(anchorEl).direction === 'rtl';

	// MEASURE, THEN PLACE, THEN REVEAL. The rendered width swings widely across 15 locales, so
	// nothing here assumes a width — an improvement on `HelpTip` (halfWidth = 200) and
	// `StructuralOutlinePanel` (halfW = 150), both of which guess and can misplace a box whose
	// real width differs. Parked hidden at 0,0 first so there is no flash at an unclamped spot.
	el.style.left = '0px';
	el.style.top = '0px';
	const r = anchorEl.getBoundingClientRect();
	const b = el.getBoundingClientRect();
	// JS pixels, never CSS `vh` — `LinkTypePicker.svelte:64` records this webview not honouring
	// viewport units for exactly this kind of clamp.
	const vw = window.innerWidth;
	const vh = window.innerHeight;

	// HORIZONTAL — the box's TRAILING edge is pinned to the anchor's trailing edge, so the body
	// of the box lies back along the direction the text came from. In an LTR interface that puts
	// it to the LEFT of the pointer (the Boss's request); in RTL it mirrors, which is the same
	// rule and not a special case — a box that always went left would sit over the file tree in
	// an Arabic interface.
	let left = rtlAnchor ? r.left : r.right - b.width;
	// A safety net only. The side is chosen by DIRECTION, never by proximity to a screen edge:
	// that was tried, Boss-reported and reverted at `cc35524d` (with the sidebar open the anchor
	// is far from any edge, so proximity logic stops flipping and the box spills over the tree).
	left = Math.max(PAD, Math.min(left, vw - b.width - PAD));

	// VERTICAL — above the chip. In both panels the chip sits on the FIRST line of a row whose
	// context, annotation and headline are stacked beneath it, so a box below would cover the
	// very text being read; above covers the previous row, which is not in focus. Flip below
	// only when there is genuinely no room above.
	let top = r.top - b.height - GAP;
	if (top < PAD) top = r.bottom + GAP;
	top = Math.max(PAD, Math.min(top, vh - b.height - PAD));

	el.style.left = `${left}px`;
	el.style.top = `${top}px`;
	el.style.visibility = 'visible';
	addDismissers();
}

function onOver(e: MouseEvent): void {
	const t = e.target as Element | null;
	const hit = t && t.nodeType === 1
		? (t.closest(`[${LINK_TIP_ATTR}]`) as HTMLElement | null)
		: null;
	if (!hit) {
		// Free self-heal: if the element we were describing has been detached (a CM6 rebuild
		// under the pointer does not reliably fire mouseout in Chromium), drop the box on the
		// next pointer boundary. Costs one boolean read on ordinary hovers.
		if (anchor && !anchor.isConnected) hideLinkTip();
		return;
	}
	if (hit === anchor) return;
	hideLinkTip();
	const text = hit.getAttribute(LINK_TIP_ATTR);
	if (!text) return;
	armTimer = setTimeout(() => show(hit, text), HOVER_INTENT_MS);
}

function onOut(e: MouseEvent): void {
	if (!anchor) return;
	const t = e.target as Node | null;
	if (t && (t === anchor || anchor.contains(t))) hideLinkTip();
}

/**
 * Installed once per window, for the window's lifetime — TWO listeners, bounded and O(1),
 * collected with the document.
 *
 * This is a deliberate, named exception to the project's "remove every listener on destroy"
 * rule. The alternative — installing and uninstalling per panel mount — would have the editor
 * and both link panels racing to own a shared resource, and the first one to unmount would
 * silently kill the tooltip for the other two. That is the worse failure mode.
 *
 * `mouseover`/`mouseout` (which bubble) rather than `mouseenter`/`mouseleave` (which do not),
 * so one pair covers every chip including ones that do not exist yet.
 *
 * The second screen is a separate window realm with its own document, so it evaluates this
 * module independently and gets its own singleton. Nothing is shared or synced across windows
 * — a tooltip is purely local chrome.
 */
if (typeof document !== 'undefined') {
	document.addEventListener('mouseover', onOver);
	document.addEventListener('mouseout', onOut);
}

/** Full teardown. Not called in app code — it exists for tests and hot-module reload. */
export function destroyLinkTip(): void {
	hideLinkTip();
	if (typeof document !== 'undefined') {
		document.removeEventListener('mouseover', onOver);
		document.removeEventListener('mouseout', onOut);
	}
	if (el) { el.remove(); el = null; }
}

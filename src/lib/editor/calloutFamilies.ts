/**
 * Callout family data — the single source of truth for the 10 built-in callout
 * families, shared by calloutPlugin (rendering), customCallouts (collision check),
 * CalloutTypesEditor (the Setter UI), and NotePane (the live-refresh signature).
 *
 * Dependency-free on purpose: importing this from both calloutPlugin and
 * customCallouts avoids a circular import (MIG-089 Phase B).
 */

// Built-in callout type → default emoji icon. Keys include every alias.
export const CALLOUT_ICONS: Record<string, string> = {
	note: 'ℹ️',      info: 'ℹ️',
	tip: '💡',       hint: '💡',       important: '💡',
	success: '✅',   check: '✅',      done: '✅',
	question: '❓',  help: '❓',       faq: '❓',
	warning: '⚠️',   caution: '⚠️',    attention: '⚠️',
	failure: '❌',   fail: '❌',       missing: '❌',
	danger: '⛔',    error: '⛔',
	bug: '🐛',
	example: '📝',
	quote: '💬',     cite: '💬',
	abstract: '📋',  summary: '📋',    tldr: '📋',
};

// The 10 canonical families (the Style Setter colour controls + icon slots key on these).
export const CALLOUT_FAMILIES = ['note', 'abstract', 'tip', 'success', 'question', 'warning', 'failure', 'danger', 'example', 'quote'] as const;

// Every built-in type (incl. aliases) → its canonical family. An alias inherits
// its family's colour AND icon.
export const CALLOUT_FAMILY: Record<string, string> = {
	note: 'note', info: 'note',
	abstract: 'abstract', summary: 'abstract', tldr: 'abstract',
	tip: 'tip', hint: 'tip', important: 'tip',
	success: 'success', check: 'success', done: 'success',
	question: 'question', help: 'question', faq: 'question',
	warning: 'warning', caution: 'warning', attention: 'warning',
	failure: 'failure', fail: 'failure', missing: 'failure',
	danger: 'danger', error: 'danger', bug: 'danger',
	example: 'example',
	quote: 'quote', cite: 'quote',
};

// The set of all reserved built-in trigger words (the keys above) — used to block
// a custom callout from shadowing a built-in (MIG-089 D4).
export const CALLOUT_BUILTIN_TYPES = new Set(Object.keys(CALLOUT_FAMILY));

/** The built-in default emoji for a callout family (the Setter's "no override" preview). */
export function calloutDefaultIcon(family: string): string { return CALLOUT_ICONS[family] ?? 'ℹ️'; }

/** True if `type` (case-insensitive) is one of the reserved built-in callout types. */
export function isBuiltinCalloutType(type: string): boolean { return CALLOUT_BUILTIN_TYPES.has((type || '').toLowerCase()); }

/**
 * Second Screen — cross-window communication helpers.
 * Uses Tauri IPC commands + events for window management and note passing.
 */

import { invoke } from '@tauri-apps/api/core';
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';

/* ------------------------------------------------------------------ */
/*  Types                                                              */
/* ------------------------------------------------------------------ */

export interface ScreenNote {
	path: string;
	name: string;
	libraryName: string;
	libraryPath: string;
	libraryColor: string;
}

export type ScreenMode = 'grid' | 'star' | 'detail' | 'skyview';
export type ContextMode = 'editor' | 'skyview' | 'browser';

export interface SkyViewNodeInfo {
	path: string;
	name: string;
	libraryName: string;
	libraryPath: string;
	libraryColor: string;
}

/* ------------------------------------------------------------------ */
/*  Monitor / display detection                                        */
/* ------------------------------------------------------------------ */

export interface MonitorInfo {
	name: string | null;
	x: number;
	y: number;
	width: number;
	height: number;
	scale_factor: number;
	is_primary: boolean;
}

/** List all connected monitors. */
export async function listMonitors(): Promise<MonitorInfo[]> {
	return invoke<MonitorInfo[]>('list_monitors');
}

/** Returns true if 2+ monitors are connected. */
export async function hasMultipleMonitors(): Promise<boolean> {
	const monitors = await listMonitors();
	return monitors.length > 1;
}

/* ------------------------------------------------------------------ */
/*  Window management                                                  */
/* ------------------------------------------------------------------ */

export async function openSecondScreen(): Promise<void> {
	await invoke('open_second_screen');
}

/**
 * Open SS and auto-position on secondary monitor.
 * Falls back to normal open if only one monitor.
 */
export async function openSecondScreenSmart(): Promise<void> {
	const monitors = await listMonitors().catch(() => []);
	console.log('[SS] Monitors detected:', monitors.length, monitors);
	if (monitors.length > 1) {
		console.log('[SS] Using smart positioning on secondary monitor');
		await invoke('open_second_screen_on_monitor');
	} else {
		console.log('[SS] Single monitor — using default open');
		await invoke('open_second_screen');
	}
}

export async function closeSecondScreen(): Promise<void> {
	await invoke('close_second_screen');
}

export async function isSecondScreenOpen(): Promise<boolean> {
	return invoke<boolean>('is_second_screen_open');
}

/* ------------------------------------------------------------------ */
/*  Events: Main → Second Screen                                       */
/* ------------------------------------------------------------------ */

/** Send a note to the second screen's Detail view */
export async function sendNoteToScreen(note: ScreenNote): Promise<void> {
	await emit('screen:open-note', note);
}

/** Notify second screen that the universe changed */
export async function notifyUniverseSwitch(): Promise<void> {
	await emit('screen:universe-switched');
}

/** Notify second screen that theme/settings changed, passing the current settings */
export async function notifySettingsChanged(settings?: Record<string, any>): Promise<void> {
	await emit('screen:settings-changed', settings || {});
}

/* ------------------------------------------------------------------ */
/*  Events: Second Screen → Main                                       */
/* ------------------------------------------------------------------ */

/** Tell main window to open a note */
export async function sendNoteToMain(note: ScreenNote): Promise<void> {
	await emit('screen:open-in-main', note);
}

/** Notify main that the second screen was closed */
export async function notifyScreenClosed(): Promise<void> {
	await emit('screen:closed');
}

/* ------------------------------------------------------------------ */
/*  Events: Bidirectional                                              */
/* ------------------------------------------------------------------ */

/** Broadcast that a note was saved (both windows should listen) */
export async function broadcastNoteSaved(path: string): Promise<void> {
	await emit('screen:note-saved', { path });
}

/* ------------------------------------------------------------------ */
/*  Workspace state exchange                                           */
/* ------------------------------------------------------------------ */

export interface ScreenState {
	mode: ScreenMode;
	linkedBrowsing: boolean;
	tabs: { path: string; libraryName: string; libraryColor: string }[];
	activeTabPath: string | null;
}

/** Main → Second Screen: request current state for workspace save */
export async function requestScreenState(): Promise<void> {
	await emit('screen:state-request');
}

/** Second Screen → Main: reply with current state */
export async function sendScreenState(state: ScreenState): Promise<void> {
	await emit('screen:state-response', state);
}

/** Main → Second Screen: restore a saved workspace state */
export async function sendWorkspaceRestore(state: ScreenState): Promise<void> {
	await emit('screen:workspace-restore', state);
}

/* ------------------------------------------------------------------ */
/*  Listeners                                                          */
/* ------------------------------------------------------------------ */

export function onNoteToScreen(callback: (note: ScreenNote) => void): Promise<UnlistenFn> {
	return listen<ScreenNote>('screen:open-note', (event) => callback(event.payload));
}

export function onNoteToMain(callback: (note: ScreenNote) => void): Promise<UnlistenFn> {
	return listen<ScreenNote>('screen:open-in-main', (event) => callback(event.payload));
}

export function onNoteSaved(callback: (path: string) => void): Promise<UnlistenFn> {
	return listen<{ path: string }>('screen:note-saved', (event) => callback(event.payload.path));
}

export function onUniverseSwitch(callback: () => void): Promise<UnlistenFn> {
	return listen('screen:universe-switched', () => callback());
}

export function onSettingsChanged(callback: (settings?: Record<string, any>) => void): Promise<UnlistenFn> {
	return listen<Record<string, any>>('screen:settings-changed', (event) => callback(event.payload));
}

export function onScreenClosed(callback: () => void): Promise<UnlistenFn> {
	return listen('screen:closed', () => callback());
}

export function onStateRequest(callback: () => void): Promise<UnlistenFn> {
	return listen('screen:state-request', () => callback());
}

export function onStateResponse(callback: (state: ScreenState) => void): Promise<UnlistenFn> {
	return listen<ScreenState>('screen:state-response', (event) => callback(event.payload));
}

export function onWorkspaceRestore(callback: (state: ScreenState) => void): Promise<UnlistenFn> {
	return listen<ScreenState>('screen:workspace-restore', (event) => callback(event.payload));
}

/* ------------------------------------------------------------------ */
/*  Sky View context events                                            */
/* ------------------------------------------------------------------ */

/** Main → Second Screen: context mode changed (skyview/editor) */
export async function emitContextChanged(mode: ContextMode): Promise<void> {
	await emit('screen:context-changed', { mode });
}

/** Main → Second Screen: node hovered in Sky View */
export async function emitSkyViewHover(node: SkyViewNodeInfo | null): Promise<void> {
	await emit('screen:skyview-hover', { node });
}

/** Main → Second Screen: node clicked in Sky View */
export async function emitSkyViewClick(node: SkyViewNodeInfo): Promise<void> {
	await emit('screen:skyview-click', { node });
}

export function onContextChanged(callback: (mode: ContextMode) => void): Promise<UnlistenFn> {
	return listen<{ mode: ContextMode }>('screen:context-changed', (event) => callback(event.payload.mode));
}

export function onSkyViewHover(callback: (node: SkyViewNodeInfo | null) => void): Promise<UnlistenFn> {
	return listen<{ node: SkyViewNodeInfo | null }>('screen:skyview-hover', (event) => callback(event.payload.node));
}

export function onSkyViewClick(callback: (node: SkyViewNodeInfo) => void): Promise<UnlistenFn> {
	return listen<{ node: SkyViewNodeInfo }>('screen:skyview-click', (event) => callback(event.payload.node));
}

/* ------------------------------------------------------------------ */
/*  Sidebar mode sync events                                            */
/* ------------------------------------------------------------------ */

export type SidebarMode = 'tree' | 'list' | 'skyview';

/** Main → Second Screen: sidebar mode changed */
export async function emitSidebarModeChanged(mode: SidebarMode): Promise<void> {
	await emit('screen:sidebar-mode-changed', { mode });
}

export function onSidebarModeChanged(callback: (mode: SidebarMode) => void): Promise<UnlistenFn> {
	return listen<{ mode: SidebarMode }>('screen:sidebar-mode-changed', (event) => callback(event.payload.mode));
}

/* ------------------------------------------------------------------ */
/*  Split view companion events                                        */
/* ------------------------------------------------------------------ */

export interface SplitCompanionData {
	active: boolean;
	notePath?: string;
	noteName?: string;
	libraryName?: string;
	libraryPath?: string;
	content?: string;
}

/** Main → Second Screen: split view state changed */
export async function emitSplitModeChanged(data: SplitCompanionData): Promise<void> {
	await emit('screen:split-mode-changed', data);
}

export function onSplitModeChanged(callback: (data: SplitCompanionData) => void): Promise<UnlistenFn> {
	return listen<SplitCompanionData>('screen:split-mode-changed', (event) => callback(event.payload));
}

/* ------------------------------------------------------------------ */
/*  Dashboard → Second Screen events                                   */
/* ------------------------------------------------------------------ */

export interface DashboardTagData {
	tag: string;
	notes: { name: string; path: string; libraryName: string }[];
}

/** Main → Second Screen: open a note from dashboard (recently edited/opened) */
export async function emitDashboardOpenNote(note: ScreenNote): Promise<void> {
	await emit('screen:dashboard-open-note', note);
}

/** Main → Second Screen: tag clicked on dashboard — show tag notes list */
export async function emitDashboardTagSelected(data: DashboardTagData): Promise<void> {
	await emit('screen:dashboard-tag-selected', data);
}

export function onDashboardOpenNote(callback: (note: ScreenNote) => void): Promise<UnlistenFn> {
	return listen<ScreenNote>('screen:dashboard-open-note', (event) => callback(event.payload));
}

export function onDashboardTagSelected(callback: (data: DashboardTagData) => void): Promise<UnlistenFn> {
	return listen<DashboardTagData>('screen:dashboard-tag-selected', (event) => callback(event.payload));
}

/* ------------------------------------------------------------------ */
/*  Index → Second Screen events                                       */
/* ------------------------------------------------------------------ */

export interface IndexTermData {
	term: string;
	notes: { note_path: string; note_name: string }[];
}

export interface IndexCompareData {
	terms: IndexTermData[];
}

/** Main → Second Screen: term clicked in Index — show notes for that term */
export async function emitIndexTermSelected(data: IndexTermData): Promise<void> {
	await emit('screen:index-term-selected', data);
}

/** Main → Second Screen: multi-term compare mode */
export async function emitIndexCompare(data: IndexCompareData): Promise<void> {
	await emit('screen:index-compare', data);
}

export function onIndexTermSelected(callback: (data: IndexTermData) => void): Promise<UnlistenFn> {
	return listen<IndexTermData>('screen:index-term-selected', (event) => callback(event.payload));
}

export function onIndexCompare(callback: (data: IndexCompareData) => void): Promise<UnlistenFn> {
	return listen<IndexCompareData>('screen:index-compare', (event) => callback(event.payload));
}

/* ------------------------------------------------------------------ */
/*  Constellation Map → Second Screen events                           */
/* ------------------------------------------------------------------ */

export interface MapCompanionData {
	active: boolean;
	colorMode: 'maturity' | 'stratum' | 'library';
	focusNode: any | null;        // MapNode — the current drill-down node
	parentNode: any | null;       // MapNode — 2 levels up (for note context)
	clickedNote: { path: string; name: string; libraryName: string; libraryPath: string } | null;
}

/** Main → Second Screen: Map state changed */
export async function emitMapCompanion(data: MapCompanionData): Promise<void> {
	await emit('screen:map-companion', data);
}

export function onMapCompanion(callback: (data: MapCompanionData) => void): Promise<UnlistenFn> {
	return listen<MapCompanionData>('screen:map-companion', (event) => callback(event.payload));
}

/* ------------------------------------------------------------------ */
/*  Editor panels companion (right sidebar migration to SS)            */
/* ------------------------------------------------------------------ */

export interface EditorPanelsData {
	active: boolean;
	notePath?: string;
	noteName?: string;
	libraryName?: string;
	libraryPath?: string;
	content?: string;
}

/** Main → SS: editor panels companion state (mirrors right sidebar data) */
export async function emitEditorPanels(data: EditorPanelsData): Promise<void> {
	await emit('screen:editor-panels', data);
}

export function onEditorPanels(callback: (data: EditorPanelsData) => void): Promise<UnlistenFn> {
	return listen<EditorPanelsData>('screen:editor-panels', (event) => callback(event.payload));
}

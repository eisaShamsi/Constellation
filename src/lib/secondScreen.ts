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
	vaultName: string;
	vaultPath: string;
	vaultColor: string;
}

export type ScreenMode = 'grid' | 'graph' | 'detail';

/* ------------------------------------------------------------------ */
/*  Window management                                                  */
/* ------------------------------------------------------------------ */

export async function openSecondScreen(): Promise<void> {
	await invoke('open_second_screen');
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

/** Notify second screen that theme/settings changed */
export async function notifySettingsChanged(): Promise<void> {
	await emit('screen:settings-changed');
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

export function onSettingsChanged(callback: () => void): Promise<UnlistenFn> {
	return listen('screen:settings-changed', () => callback());
}

export function onScreenClosed(callback: () => void): Promise<UnlistenFn> {
	return listen('screen:closed', () => callback());
}

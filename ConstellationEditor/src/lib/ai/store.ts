/**
 * AI state management — Svelte stores for AI configuration and status.
 */

import { writable, derived } from 'svelte/store';
import type { ProviderId } from './provider';
import { DEFAULT_MODELS } from './provider';

const STORAGE_KEY = 'constellation-ai-config';

export interface AISettings {
	provider: ProviderId | null;
	apiKey: string;
	model: string;
	baseUrl: string;
	isConnected: boolean;
}

const defaultSettings: AISettings = {
	provider: null,
	apiKey: '',
	model: '',
	baseUrl: 'http://localhost:11434',
	isConnected: false
};

function loadSettings(): AISettings {
	if (typeof window === 'undefined') return defaultSettings;
	try {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved) {
			const parsed = JSON.parse(saved);
			return { ...defaultSettings, ...parsed };
		}
	} catch {
		// ignore parse errors
	}
	return defaultSettings;
}

export const aiSettings = writable<AISettings>(loadSettings());

// Persist changes (excluding sensitive apiKey from localStorage — key goes through Rust)
aiSettings.subscribe(($settings) => {
	if (typeof window !== 'undefined') {
		const toSave = { ...$settings, apiKey: '' }; // Never store key in browser
		localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave));
	}
});

export const hasProvider = derived(aiSettings, ($s) => $s.provider !== null && $s.isConnected);

export const activeProvider = derived(aiSettings, ($s) => $s.provider);

export function updateAISettings(updates: Partial<AISettings>) {
	aiSettings.update((current) => ({ ...current, ...updates }));
}

export function setProvider(provider: ProviderId) {
	aiSettings.update((current) => ({
		...current,
		provider,
		model: current.model || DEFAULT_MODELS[provider],
		isConnected: false
	}));
}

export function clearProvider() {
	aiSettings.update((current) => ({
		...current,
		provider: null,
		apiKey: '',
		model: '',
		isConnected: false
	}));
}

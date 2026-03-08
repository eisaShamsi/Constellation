/**
 * AI Engine — Routes requests to the active provider via Tauri backend.
 * API calls go through Rust for security (keys never touch the browser).
 */

import { invoke } from '@tauri-apps/api/core';
import type { ProviderId, AIMessage, AIResponse, ModelInfo } from './provider';

export interface AIEngineConfig {
	provider: ProviderId;
	apiKey: string;
	model: string;
	baseUrl?: string;
}

/**
 * Send a message to the AI provider via the Rust backend.
 */
export async function sendMessage(
	config: AIEngineConfig,
	messages: AIMessage[],
	options?: { maxTokens?: number; temperature?: number }
): Promise<AIResponse> {
	return await invoke('ai_send_message', {
		provider: config.provider,
		apiKey: config.apiKey,
		model: config.model,
		baseUrl: config.baseUrl || '',
		messages: messages,
		maxTokens: options?.maxTokens ?? 2048,
		temperature: options?.temperature ?? 0.7
	});
}

/**
 * Validate the connection to the AI provider.
 */
export async function validateConnection(config: AIEngineConfig): Promise<boolean> {
	return await invoke('ai_validate_connection', {
		provider: config.provider,
		apiKey: config.apiKey,
		model: config.model,
		baseUrl: config.baseUrl || ''
	});
}

/**
 * List available models for a provider.
 */
export async function listModels(config: AIEngineConfig): Promise<ModelInfo[]> {
	return await invoke('ai_list_models', {
		provider: config.provider,
		apiKey: config.apiKey,
		baseUrl: config.baseUrl || ''
	});
}

/**
 * Quick helper: send a single prompt and get a response.
 */
export async function prompt(
	config: AIEngineConfig,
	userPrompt: string,
	systemPrompt?: string
): Promise<string> {
	const messages: AIMessage[] = [];
	if (systemPrompt) {
		messages.push({ role: 'system', content: systemPrompt });
	}
	messages.push({ role: 'user', content: userPrompt });

	const response = await sendMessage(config, messages);
	return response.content;
}

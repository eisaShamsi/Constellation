/**
 * AI Provider abstraction layer.
 * All providers implement this interface so the app works identically
 * regardless of which AI service the user chooses.
 */

export type ProviderId = 'openai' | 'anthropic' | 'gemini' | 'ollama';

export interface ProviderConfig {
	id: ProviderId;
	name: string;
	apiKey?: string;
	model: string;
	baseUrl?: string; // For Ollama or custom endpoints
}

export interface AIMessage {
	role: 'system' | 'user' | 'assistant';
	content: string;
}

export interface AIRequestOptions {
	messages: AIMessage[];
	maxTokens?: number;
	temperature?: number;
	stream?: boolean;
}

export interface AIResponse {
	content: string;
	model: string;
	tokensUsed?: number;
}

export interface ModelInfo {
	id: string;
	name: string;
}

export interface AIProvider {
	readonly id: ProviderId;
	readonly name: string;

	/** Send a message and get a complete response */
	sendMessage(options: AIRequestOptions): Promise<AIResponse>;

	/** List available models for this provider */
	listModels(): Promise<ModelInfo[]>;

	/** Validate the API key / connection */
	validateConnection(): Promise<boolean>;
}

/** Default models for each provider */
export const DEFAULT_MODELS: Record<ProviderId, string> = {
	openai: 'gpt-4o',
	anthropic: 'claude-sonnet-4-20250514',
	gemini: 'gemini-pro',
	ollama: 'llama3'
};

/** Provider display info */
export const PROVIDER_INFO: Record<ProviderId, { name: string; requiresKey: boolean; hasBaseUrl: boolean }> = {
	openai: { name: 'OpenAI', requiresKey: true, hasBaseUrl: false },
	anthropic: { name: 'Claude (Anthropic)', requiresKey: true, hasBaseUrl: false },
	gemini: { name: 'Google Gemini', requiresKey: true, hasBaseUrl: false },
	ollama: { name: 'Ollama (Local)', requiresKey: false, hasBaseUrl: true }
};

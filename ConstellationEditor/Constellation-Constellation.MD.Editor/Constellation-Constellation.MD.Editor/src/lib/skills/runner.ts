/**
 * Skill Runner — Executes a skill by filling its prompt template
 * with user inputs and sending it to the AI engine.
 */

import type { SkillDefinition, SkillExecutionResult } from './types';
import { prompt } from '$lib/ai/engine';
import type { AIEngineConfig } from '$lib/ai/engine';

/**
 * Fill a prompt template with input values.
 * Replaces {{key}} with the corresponding value.
 * Supports simple {{#if key}}...{{/if}} conditionals.
 */
function fillTemplate(template: string, inputs: Record<string, string>): string {
	let result = template;

	// Handle {{#if key}}...{{/if}} blocks
	result = result.replace(
		/\{\{#if (\w+)\}\}([\s\S]*?)\{\{\/if\}\}/g,
		(_: string, key: string, content: string) => {
			return inputs[key] ? content.replace(/\{\{(\w+)\}\}/g, (__: string, k: string) => inputs[k] || '') : '';
		}
	);

	// Replace remaining {{key}} placeholders
	result = result.replace(/\{\{(\w+)\}\}/g, (_, key) => inputs[key] || '');

	return result;
}

/**
 * Execute a skill with the given inputs.
 */
export async function runSkill(
	skill: SkillDefinition,
	inputs: Record<string, string>,
	aiConfig: AIEngineConfig
): Promise<SkillExecutionResult> {
	const filledPrompt = fillTemplate(skill.promptTemplate, inputs);

	const response = await prompt(aiConfig, filledPrompt, skill.systemPrompt);

	return {
		skillId: skill.id,
		content: response,
		timestamp: Date.now()
	};
}

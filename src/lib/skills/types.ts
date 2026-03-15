/**
 * Skill definition types.
 * A Skill is a packaged AI workflow — prompt template + inputs + output format.
 */

export interface SkillInput {
	type: 'note-select' | 'library-select' | 'text' | 'textarea' | 'select';
	key: string;
	label: string;
	label_ar?: string;
	placeholder?: string;
	placeholder_ar?: string;
	options?: { value: string; label: string; label_ar?: string }[];
	required?: boolean;
}

export interface SkillDefinition {
	id: string;
	name: string;
	name_ar: string;
	description: string;
	description_ar: string;
	icon: string;
	category: 'analysis' | 'writing' | 'organization' | 'generation' | 'research';
	inputs: SkillInput[];
	systemPrompt?: string;
	promptTemplate: string;
	output: 'markdown' | 'text' | 'json';
	builtin: boolean;
}

export interface SkillExecutionResult {
	skillId: string;
	content: string;
	timestamp: number;
	tokensUsed?: number;
}

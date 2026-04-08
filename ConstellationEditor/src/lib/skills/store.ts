/**
 * Skills state management.
 */

import { writable, derived } from 'svelte/store';
import type { SkillDefinition } from './types';
import { builtinSkills } from './builtin';

// All available skills (built-in + user-installed)
export const skills = writable<SkillDefinition[]>([...builtinSkills]);

// Filter by category
export function getSkillsByCategory(category: string) {
	return derived(skills, ($skills) =>
		$skills.filter((s) => s.category === category)
	);
}

// Get a skill by ID
export function getSkillById(id: string) {
	return derived(skills, ($skills) =>
		$skills.find((s) => s.id === id) ?? null
	);
}

// Install a custom skill
export function installSkill(skill: SkillDefinition) {
	skills.update((current) => {
		// Don't add duplicates
		if (current.some((s) => s.id === skill.id)) return current;
		return [...current, { ...skill, builtin: false }];
	});
}

// Remove a custom skill (can't remove built-in)
export function removeSkill(id: string) {
	skills.update((current) => current.filter((s) => s.id !== id || s.builtin));
}

<script lang="ts">
	import { t, locale } from '$lib/i18n';
	import { skills } from '$lib/skills/store';
	import { hasProvider, aiSettings } from '$lib/ai/store';
	import { runSkill } from '$lib/skills/runner';
	import type { SkillDefinition } from '$lib/skills/types';

	let selectedSkill = $state<SkillDefinition | null>(null);
	let inputValues = $state<Record<string, string>>({});
	let result = $state('');
	let running = $state(false);

	function selectSkill(skill: SkillDefinition) {
		selectedSkill = skill;
		inputValues = {};
		result = '';
	}

	function getName(skill: SkillDefinition) {
		return $locale === 'ar' ? skill.name_ar : skill.name;
	}

	function getDesc(skill: SkillDefinition) {
		return $locale === 'ar' ? skill.description_ar : skill.description;
	}

	async function execute() {
		if (!selectedSkill || !$aiSettings.provider) return;
		running = true;
		result = '';
		try {
			const res = await runSkill(selectedSkill, inputValues, {
				provider: $aiSettings.provider,
				apiKey: $aiSettings.apiKey,
				model: $aiSettings.model,
				baseUrl: $aiSettings.baseUrl
			});
			result = res.content;
		} catch (e) {
			result = `Error: ${e}`;
		}
		running = false;
	}
</script>

<div class="skills-page">
	<h1>{$t('skills.title')}</h1>
	<p class="desc">{$t('skills.description')}</p>

	{#if !$hasProvider}
		<div class="notice">{$t('skills.noProvider')}</div>
	{/if}

	<div class="skills-layout">
		<!-- Skills List -->
		<div class="skills-list">
			{#each $skills as skill}
				<button
					class="skill-card"
					class:active={selectedSkill?.id === skill.id}
					onclick={() => selectSkill(skill)}
				>
					<span class="skill-icon">{skill.icon}</span>
					<div>
						<div class="skill-name">{getName(skill)}</div>
						<div class="skill-desc">{getDesc(skill)}</div>
					</div>
				</button>
			{/each}
		</div>

		<!-- Skill Panel -->
		{#if selectedSkill}
			<div class="skill-panel">
				<h2>{getName(selectedSkill)}</h2>
				<p class="panel-desc">{getDesc(selectedSkill)}</p>

				{#each selectedSkill.inputs as input}
					<label class="field">
						<span>{$locale === 'ar' ? (input.label_ar ?? input.label) : input.label}</span>
						{#if input.type === 'textarea'}
							<textarea
								placeholder={$locale === 'ar' ? (input.placeholder_ar ?? input.placeholder ?? '') : (input.placeholder ?? '')}
								oninput={(e) => { inputValues[input.key] = (e.target as HTMLTextAreaElement).value; }}
							></textarea>
						{:else if input.type === 'select'}
							<select onchange={(e) => { inputValues[input.key] = (e.target as HTMLSelectElement).value; }}>
								<option value="">—</option>
								{#each input.options ?? [] as opt}
									<option value={opt.value}>{$locale === 'ar' ? (opt.label_ar ?? opt.label) : opt.label}</option>
								{/each}
							</select>
						{:else}
							<input
								type="text"
								placeholder={$locale === 'ar' ? (input.placeholder_ar ?? input.placeholder ?? '') : (input.placeholder ?? '')}
								oninput={(e) => { inputValues[input.key] = (e.target as HTMLInputElement).value; }}
							/>
						{/if}
					</label>
				{/each}

				<button class="run-btn" onclick={execute} disabled={running || !$hasProvider}>
					{running ? $t('skills.running') : $t('skills.run')}
				</button>

				{#if result}
					<div class="result">
						<h3>{$t('skills.result')}</h3>
						<pre>{result}</pre>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.skills-page { max-width: 100%; }
	h1 { font-size: 1.8rem; margin-bottom: 0.25rem; }
	.desc { color: #57606a; margin-bottom: 1.5rem; }

	.notice {
		background: #fffbeb;
		border: 1px solid #d97706;
		color: #92400e;
		padding: 0.75rem 1rem;
		border-radius: 8px;
		margin-bottom: 1.5rem;
		font-size: 0.9rem;
	}

	.skills-layout { display: flex; gap: 1.5rem; }

	.skills-list {
		flex: 0 0 280px;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.skill-card {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.75rem;
		background: #f6f8fa;
		border: 1px solid #d0d7de;
		border-radius: 8px;
		cursor: pointer;
		text-align: start;
		color: #24292f;
		transition: all 0.2s;
		width: 100%;
	}
	.skill-card:hover { border-color: #afb8c1; background: #eaeef2; }
	.skill-card.active { border-color: #7c3aed; }

	.skill-icon { font-size: 1.3rem; flex-shrink: 0; margin-top: 2px; }
	.skill-name { font-weight: 600; font-size: 0.9rem; }
	.skill-desc { color: #57606a; font-size: 0.8rem; margin-top: 2px; }

	.skill-panel {
		flex: 1;
		background: #f6f8fa;
		border: 1px solid #d0d7de;
		border-radius: 8px;
		padding: 1.5rem;
	}
	.skill-panel h2 { font-size: 1.3rem; margin-bottom: 0.25rem; }
	.panel-desc { color: #57606a; font-size: 0.9rem; margin-bottom: 1.5rem; }

	.field { display: block; margin-bottom: 1rem; }
	.field span { display: block; font-size: 0.85rem; color: #57606a; margin-bottom: 0.3rem; }

	textarea, select, input {
		width: 100%;
		padding: 0.6em 0.8em;
		background: #ffffff;
		border: 1px solid #d0d7de;
		border-radius: 6px;
		color: #1f2328;
		font-size: 0.95rem;
		font-family: inherit;
		box-sizing: border-box;
	}
	textarea { min-height: 100px; resize: vertical; }
	textarea:focus, select:focus, input:focus { border-color: #7c3aed; outline: none; }

	.run-btn {
		background: #7c3aed;
		border: none;
		color: white;
		padding: 0.7em 1.5em;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.95rem;
		font-weight: 600;
		transition: background 0.2s;
	}
	.run-btn:hover { background: #6d28d9; }
	.run-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.result {
		margin-top: 1.5rem;
		padding: 1rem;
		background: #ffffff;
		border: 1px solid #d0d7de;
		border-radius: 6px;
	}
	.result h3 { font-size: 0.95rem; margin-bottom: 0.5rem; }
	.result pre {
		white-space: pre-wrap;
		word-break: break-word;
		font-size: 0.9rem;
		color: #24292f;
		margin: 0;
	}
</style>

<script lang="ts">
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';

	let { onUnlock }: { onUnlock: () => void } = $props();

	let pin = $state('');
	let error = $state(false);
	let shaking = $state(false);

	async function hashPin(input: string): Promise<string> {
		const encoder = new TextEncoder();
		const data = encoder.encode(input);
		const hash = await crypto.subtle.digest('SHA-256', data);
		return Array.from(new Uint8Array(hash))
			.map(b => b.toString(16).padStart(2, '0')).join('');
	}

	async function handleSubmit() {
		const hash = await hashPin(pin);
		if (hash === $appSettings.security.lockPinHash) {
			error = false;
			pin = '';
			onUnlock();
		} else {
			error = true;
			shaking = true;
			pin = '';
			setTimeout(() => shaking = false, 500);
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && pin.length >= 4) {
			handleSubmit();
		}
		if (e.key === 'Escape' || e.ctrlKey || e.metaKey || e.altKey) {
			e.preventDefault();
			e.stopPropagation();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="lock-overlay" onkeydown={handleKeydown}>
	<div class="lock-content">
		<svg class="lock-icon" width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
			<path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/>
		</svg>
		<div class="lock-title">{$t('lockScreen.title')}</div>
		<div class="lock-subtitle">{$t('lockScreen.subtitle')}</div>
		<div class="lock-input-row" class:shake={shaking}>
			<input
				type="password"
				class="lock-input"
				placeholder={$t('lockScreen.enterPin')}
				maxlength="8"
				bind:value={pin}
				autofocus
			/>
			<button class="lock-btn" onclick={handleSubmit} disabled={pin.length < 4}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
					stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>
				</svg>
			</button>
		</div>
		{#if error}
			<div class="lock-error">{$t('lockScreen.wrongPin')}</div>
		{/if}
	</div>
</div>

<style>
	.lock-overlay {
		position: fixed; inset: 0; z-index: 9999;
		background: var(--background-primary);
		display: flex; align-items: center; justify-content: center;
		user-select: none;
	}
	.lock-content {
		text-align: center; max-width: 320px; width: 100%;
		padding: 32px;
	}
	.lock-icon {
		color: var(--interactive-accent); margin-bottom: 16px;
		opacity: 0.8;
	}
	.lock-title {
		font-size: 1.3rem; font-weight: 700; color: var(--text-normal);
		margin-bottom: 4px;
	}
	.lock-subtitle {
		font-size: 0.85rem; color: var(--text-muted); margin-bottom: 24px;
	}
	.lock-input-row {
		display: flex; gap: 8px; justify-content: center;
	}
	.lock-input {
		width: 180px; padding: 10px 14px; text-align: center;
		font-size: 1.1rem; letter-spacing: 0.3em;
		background: var(--background-secondary);
		border: 2px solid var(--background-modifier-border);
		border-radius: 8px; color: var(--text-normal);
		font-family: var(--font-monospace-theme);
		outline: none;
	}
	.lock-input:focus {
		border-color: var(--interactive-accent);
	}
	.lock-btn {
		width: 42px; height: 42px;
		display: flex; align-items: center; justify-content: center;
		border: none; border-radius: 8px; cursor: pointer;
		background: var(--interactive-accent);
		color: var(--text-on-accent);
		transition: opacity 0.15s;
	}
	.lock-btn:hover { opacity: 0.9; }
	.lock-btn:disabled { opacity: 0.4; cursor: not-allowed; }
	.lock-error {
		margin-top: 12px; font-size: 0.82rem; color: var(--text-error);
		font-weight: 500;
	}
	.shake {
		animation: shake 0.4s ease-in-out;
	}
	@keyframes shake {
		0%, 100% { transform: translateX(0); }
		20% { transform: translateX(-8px); }
		40% { transform: translateX(8px); }
		60% { transform: translateX(-6px); }
		80% { transform: translateX(6px); }
	}
</style>

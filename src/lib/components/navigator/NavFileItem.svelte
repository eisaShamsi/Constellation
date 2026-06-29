<script lang="ts">
	import type { NoteWithMeta } from '$lib/libraries/store';

	let {
		note,
		color = '#7c3aed',
		selected = false,
		focused = false,
		onSelect,
		onClick,
		onDoubleClick,
	}: {
		note: NoteWithMeta;
		color?: string;
		selected?: boolean;
		focused?: boolean;
		onSelect?: (path: string, checked: boolean) => void;
		onClick?: (note: NoteWithMeta) => void;
		onDoubleClick?: (note: NoteWithMeta) => void;
	} = $props();

	function relativeDate(ms: number): string {
		if (!ms) return '';
		const diff = Date.now() - ms;
		const mins = Math.floor(diff / 60000);
		if (mins < 1) return 'now';
		if (mins < 60) return `${mins}m`;
		const hrs = Math.floor(mins / 60);
		if (hrs < 24) return `${hrs}h`;
		const days = Math.floor(hrs / 24);
		if (days < 30) return `${days}d`;
		const months = Math.floor(days / 30);
		if (months < 12) return `${months}mo`;
		return `${Math.floor(months / 12)}y`;
	}

	const displayName = $derived(note.name.replace(/\.md$/, ''));
	const dateStr = $derived(relativeDate(note.modified));
</script>

<button
	class="nav-file-item"
	class:selected
	class:focused
	dir="auto"
	data-path={note.path}
	onclick={() => onClick?.(note)}
	ondblclick={() => onDoubleClick?.(note)}
>
	<input
		type="checkbox"
		class="nav-file-check"
		checked={selected}
		onclick={(e) => e.stopPropagation()}
		onchange={(e) => onSelect?.(note.path, (e.target as HTMLInputElement).checked)}
	/>
	<span class="nav-file-dot" style="background:{color}"></span>
	<div class="nav-file-info">
		<div class="nav-file-title" dir="auto">{displayName}</div>
		{#if note.preview}
			<div class="nav-file-preview" dir="auto">{note.preview.slice(0, 100)}</div>
		{/if}
		{#if note.tags.length > 0}
			<div class="nav-file-tags">
				{#each note.tags.slice(0, 4) as tag}
					<span class="nav-tag-badge">#{tag}</span>
				{/each}
				{#if note.tags.length > 4}
					<span class="nav-tag-more">+{note.tags.length - 4}</span>
				{/if}
			</div>
		{/if}
	</div>
	<span class="nav-file-date">{dateStr}</span>
</button>

<style>
	.nav-file-item {
		display: flex; align-items: flex-start; gap: 8px;
		width: 100%; padding: 8px 10px; border: none; border-radius: 4px;
		background: transparent; cursor: pointer; text-align: start;
		font-family: var(--font-interface-theme), sans-serif;
		border-bottom: 1px solid var(--background-modifier-border, #e2e8f0);
		transition: background 0.1s;
	}
	.nav-file-item:hover { background: var(--background-modifier-hover); }
	.nav-file-item.selected { background: color-mix(in srgb, var(--interactive-accent) 10%, transparent); }
	.nav-file-item.focused { outline: 2px solid var(--interactive-accent); outline-offset: -2px; }

	.nav-file-check { flex-shrink: 0; margin-top: 2px; accent-color: var(--interactive-accent); }
	.nav-file-dot { flex-shrink: 0; width: 8px; height: 8px; border-radius: 50%; margin-top: 5px; }

	.nav-file-info { flex: 1; min-width: 0; overflow: hidden; }
	.nav-file-title {
		font-size: 13px; font-weight: 500; color: var(--text-normal);
		white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}
	.nav-file-preview {
		font-size: 11px; color: var(--text-muted); margin-top: 2px;
		white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}
	.nav-file-tags { display: flex; flex-wrap: wrap; gap: 3px; margin-top: 4px; }
	.nav-tag-badge {
		font-size: 10px; padding: 1px 5px; border-radius: 8px;
		background: var(--background-secondary); color: var(--text-muted);
	}
	.nav-tag-more { font-size: 10px; color: var(--text-faint); }

	.nav-file-date { flex-shrink: 0; font-size: 11px; color: var(--text-faint); margin-top: 3px; white-space: nowrap; }
</style>

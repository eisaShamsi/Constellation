/**
 * Entry point for the second screen window.
 * This file is loaded by static/screen.html and mounts the SecondScreenPage Svelte component.
 * It bypasses SvelteKit's routing entirely — the second window is a standalone Svelte app.
 */
import { mount } from 'svelte';
import '$lib/theme.css';
import SecondScreenPage from '$lib/components/SecondScreenPage.svelte';

// Apply theme class to body
const body = document.body;
if (!body.classList.contains('theme-light') && !body.classList.contains('theme-dark')) {
	body.classList.add('theme-light');
}

// Mount the second screen component
const target = document.getElementById('app');
if (target) {
	mount(SecondScreenPage, { target });
} else {
	console.error('[Screen 2] No #app element found');
}

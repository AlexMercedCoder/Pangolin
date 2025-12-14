import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark' | 'system';

function createThemeStore() {
	const { subscribe, set } = writable<Theme>('system');

	return {
		subscribe,
		setTheme: (theme: Theme) => {
			if (browser) {
				localStorage.setItem('theme', theme);
				applyTheme(theme);
			}
			set(theme);
		},
		loadTheme: () => {
			if (browser) {
				const stored = localStorage.getItem('theme') as Theme | null;
				const theme = stored || 'system';
				applyTheme(theme);
				set(theme);
			}
		},
	};
}

function applyTheme(theme: Theme) {
	if (!browser) return;

	const root = document.documentElement;
	
	if (theme === 'system') {
		const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
		root.classList.toggle('dark', prefersDark);
	} else {
		root.classList.toggle('dark', theme === 'dark');
	}
}

export const themeStore = createThemeStore();

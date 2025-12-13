import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { lightTheme, darkTheme, type Theme } from './theme';

const createThemeStore = () => {
    // Initial value
    let initialTheme: Theme = lightTheme;
    let initialIsDark = false;

    if (browser) {
        const stored = localStorage.getItem('theme');
        if (stored === 'dark') {
            initialTheme = darkTheme;
            initialIsDark = true;
        } else if (window.matchMedia('(prefers-color-scheme: dark)').matches && stored !== 'light') {
            initialTheme = darkTheme;
            initialIsDark = true;
        }
    }

    const { subscribe, set, update } = writable<{ theme: Theme; isDark: boolean }>({
        theme: initialTheme,
        isDark: initialIsDark,
    });

    return {
        subscribe,
        toggle: () => {
            update((state) => {
                const isDark = !state.isDark;
                const theme = isDark ? darkTheme : lightTheme;
                if (browser) {
                    localStorage.setItem('theme', isDark ? 'dark' : 'light');
                }
                return { theme, isDark };
            });
        },
        setDark: (isDark: boolean) => {
            const theme = isDark ? darkTheme : lightTheme;
            if (browser) {
                localStorage.setItem('theme', isDark ? 'dark' : 'light');
            }
            set({ theme, isDark });
        }
    };
};

export const themeStore = createThemeStore();

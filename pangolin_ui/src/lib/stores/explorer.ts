import { writable } from 'svelte/store';

export const refreshExplorer = writable<{ type: 'catalog' | 'content' | 'all', timestamp: number }>({ type: 'all', timestamp: 0 });

export function triggerExplorerRefresh(type: 'catalog' | 'content' | 'all' = 'all') {
    refreshExplorer.set({ type, timestamp: Date.now() });
}

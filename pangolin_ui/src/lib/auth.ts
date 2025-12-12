import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface User {
    username: string;
    roles: string[];
    tenantId?: string;
}

// Initialize from localStorage if available
const storedUser = browser ? localStorage.getItem('pangolin_user') : null;
const storedToken = browser ? localStorage.getItem('pangolin_token') : null;

export const user = writable<User | null>(storedUser ? JSON.parse(storedUser) : null);
export const token = writable<string | null>(storedToken);

if (browser) {
    user.subscribe((u) => {
        if (u) localStorage.setItem('pangolin_user', JSON.stringify(u));
        else localStorage.removeItem('pangolin_user');
    });

    token.subscribe((t) => {
        if (t) localStorage.setItem('pangolin_token', t);
        else localStorage.removeItem('pangolin_token');
    });
}

export function logout() {
    user.set(null);
    token.set(null);
}

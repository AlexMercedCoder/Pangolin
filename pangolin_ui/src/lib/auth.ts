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
export const selectedTenant = writable<string | null>(null); // For Root user context switching

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
export async function checkAuth() {
    // If we have a token, we rely on it.
    // If we don't, we try /api/v1/users/me to see if NO_AUTH is active on server
    
    // Logic:
    // 1. If token exists, we are good (or valid until 401).
    // 2. If no token, call /me. If 200, it means NO_AUTH is on.
    
    const currentUser = get(user);
    if (currentUser) return; // Already logged in

    try {
        const res = await fetch('/api/v1/users/me');
        if (res.ok) {
            const data = await res.json();
            // NO_AUTH active, we got a user back!
            user.set(data);
            // No token needed, or maybe server set simple session.
            // If server returned one, we could set it, but NO_AUTH usually ignores headers.
        }
    } catch (e) {
        console.error("Auth check failed", e);
    }
}

import { get } from 'svelte/store';

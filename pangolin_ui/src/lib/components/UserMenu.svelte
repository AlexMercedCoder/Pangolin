<script lang="ts">
    import { user, logout } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';

    let isOpen = false;
    let menuRef: HTMLDivElement;

    function toggleMenu() {
        isOpen = !isOpen;
    }

    function handleLogout() {
        logout();
        goto('/login');
        isOpen = false;
    }

    function handleClickOutside(event: MouseEvent) {
        if (menuRef && !menuRef.contains(event.target as Node)) {
            isOpen = false;
        }
    }

    onMount(() => {
        document.addEventListener('click', handleClickOutside);
        return () => {
            document.removeEventListener('click', handleClickOutside);
        };
    });
</script>

<div class="user-menu-container" bind:this={menuRef}>
    <button class="user-btn" on:click|stopPropagation={toggleMenu}>
        <div class="avatar">
            {$user?.username.charAt(0).toUpperCase()}
        </div>
        <span class="username">{$user?.username}</span>
        <span class="arrow">▼</span>
    </button>

    {#if isOpen}
        <div class="menu-dropdown" transition:fade={{ duration: 100 }}>
            <div class="menu-header">
                <span class="role-badge">{$user?.role || 'User'}</span>
                {#if $user?.tenant_id}
                    <div class="tenant-info">
                        Tenant: <span class="mono">{$user.tenant_id}</span>
                    </div>
                {/if}
            </div>
            
            <a href="/profile/tokens" class="menu-item">
                <span class="icon">👤</span>
                <span class="label">Profile</span>
            </a>
            
            <a href="/profile/tokens" class="menu-item">
                <span class="icon">🔑</span>
                <span class="label">My Tokens</span>
            </a>

            <a href="https://www.pangolincatalog.org" target="_blank" rel="noopener noreferrer" class="menu-item">
                <span class="icon">📚</span>
                <span class="label">Docs</span>
            </a>

            <div class="divider"></div>

            <button class="menu-item logout" on:click={handleLogout}>
                <span class="icon">🚪</span>
                <span class="label">Logout</span>
            </button>
        </div>
    {/if}
</div>

<style>
    .user-menu-container {
        position: relative;
    }

    .user-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background: transparent;
        border: none;
        color: inherit;
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 100px;
        transition: background-color 0.2s;
    }

    .user-btn:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }

    .avatar {
        width: 32px;
        height: 32px;
        background-color: var(--md-sys-color-primary, #6750a4);
        color: var(--md-sys-color-on-primary, #ffffff);
        border-radius: 50%;
        display: flex;
        justify-content: center;
        align-items: center;
        font-weight: bold;
        font-size: 1rem;
    }

    .username {
        font-weight: 500;
        font-size: 0.875rem;
    }

    .arrow {
        font-size: 0.8rem;
        margin-left: 0.25rem;
    }
    
    .icon {
        font-size: 1.25rem;
        width: 1.5rem;
        text-align: center;
    }
    
    .label {
        font-size: 0.9rem;
        font-weight: 500;
    }

    .menu-dropdown {
        position: absolute;
        top: 100%;
        right: 0;
        margin-top: 0.5rem;
        background-color: white; /* Fallback */
        background-color: var(--md-sys-color-surface, #ffffff);
        color: var(--md-sys-color-on-surface, #1d1b20);
        border-radius: 12px;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
        min-width: 200px;
        overflow: hidden;
        z-index: 100;
        border: 1px solid rgba(0,0,0,0.1);
    }
    
    :global(.dark) .menu-dropdown {
        background-color: #2d2d2d;
        background-color: var(--md-sys-color-surface, #2d2d2d);
        color: white;
        color: var(--md-sys-color-on-surface, #e6e1e5);
        border: 1px solid rgba(255,255,255,0.1);
    }
    .menu-header {
        padding: 1rem;
        background-color: rgba(0,0,0,0.02);
        border-bottom: 1px solid rgba(0,0,0,0.05);
    }

    .role-badge {
        display: inline-block;
        font-size: 0.75rem;
        padding: 2px 8px;
        background-color: var(--md-sys-color-secondary-container);
        color: var(--md-sys-color-on-secondary-container);
        border-radius: 4px;
        font-weight: 500;
    }

    .tenant-info {
        font-size: 0.75rem;
        margin-top: 0.5rem;
        opacity: 0.7;
    }

    .mono {
        font-family: monospace;
    }

    .menu-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem 1rem;
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        color: inherit;
        text-decoration: none;
        cursor: pointer;
        transition: background-color 0.2s;
        font-size: 0.875rem;
        box-sizing: border-box;
    }

    .menu-item:hover {
        background-color: rgba(0, 0, 0, 0.05);
    }

    .menu-item.logout {
        color: var(--md-sys-color-error);
    }

    .menu-item.logout:hover {
        background-color: var(--md-sys-color-error-container);
        color: var(--md-sys-color-on-error-container);
    }

    .divider {
        height: 1px;
        background-color: rgba(0,0,0,0.05);
        margin: 0.25rem 0;
    }

    .material-icons {
        font-size: 1.25rem;
    }
</style>

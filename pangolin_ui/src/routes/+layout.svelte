<script>
    import '../app.css';
    import { onMount } from 'svelte';
    import { user, checkAuth } from '$lib/auth';
    import { goto } from '$app/navigation';
    import { page } from '$app/stores';
    import { themeStore } from '$lib/stores';
    import { applyTheme } from '$lib/theme';
    import UserMenu from '$lib/components/UserMenu.svelte';
    import TenantSelector from '$lib/components/TenantSelector.svelte';
    import 'material-icons/iconfont/material-icons.css';

    let isAuthChecked = false;

    // Reactively run auth check when user or path changes
    $: if (isAuthChecked && $user && $page.url.pathname === '/login') {
         goto('/');
    }

    $: if (isAuthChecked && !$user && $page.url.pathname !== '/login') {
         goto('/login');
    }

    onMount(() => {
        // Initial Theme Application
        themeStore.subscribe(state => {
            applyTheme(state.theme);
        });

        // Try to auto-login (NO_AUTH support)
        // checkAuth imported at top level
        checkAuth().then(() => {
            user.subscribe(u => {
                 isAuthChecked = true;
                 if (!u && $page.url.pathname !== '/login') {
                     goto('/login');
                 }
            });
        });

    });
</script>

<!-- Only render main app if auth checked to prevent flashes -->
{#if isAuthChecked}
    <div class="app-container" style="background-color: var(--md-sys-color-background); color: var(--md-sys-color-on-background); min-height: 100vh;">
        {#if $user}
            <nav style="background-color: var(--md-sys-color-surface); color: var(--md-sys-color-on-surface); padding: 0.75rem 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); border-bottom: 1px solid rgba(0,0,0,0.05);">
                <div style="display: flex; justify-content: space-between; align-items: center; max-width: 1400px; margin: 0 auto;">
                    <div style="display: flex; align-items: center; gap: 2rem;">
                        <a href="/" style="display: flex; align-items: center; gap: 0.5rem; text-decoration: none; color: inherit;">
                            <img src="/logo.png" alt="Logo" style="height: 32px; width: 32px; object-fit: contain;" />
                            <span style="font-size: 1.25rem; font-weight: 500; font-family: 'Roboto', sans-serif;">Pangolin</span>
                        </a>
                        
                        <div style="display: flex; gap: 0.5rem;">
                            <a href="/tenants" class="nav-link" class:active={$page.url.pathname.startsWith('/tenants')}>Tenants</a>
                            <a href="/warehouses" class="nav-link" class:active={$page.url.pathname.startsWith('/warehouses')}>Warehouses</a>
                            <a href="/assets" class="nav-link" class:active={$page.url.pathname.startsWith('/assets')}>Assets</a>
                        </div>
                    </div>
                    
                    <div style="display: flex; align-items: center; gap: 1rem;">
                        <button on:click={themeStore.toggle} class="icon-btn" title="Toggle Theme">
                            <span class="material-icons">
                                {$themeStore.isDark ? 'light_mode' : 'dark_mode'}
                            </span>
                        </button>
                        
                        <div class="divider"></div>
                        
                        <TenantSelector />
                        <UserMenu />
                    </div>
                </div>
            </nav>
        {/if}

        <main style="max-width: 1400px; margin: 0 auto; padding: 2rem; box-sizing: border-box;">
            <slot />
        </main>
    </div>
{:else}
    <!-- Loading state while checking auth -->
    <div style="display: flex; justify-content: center; align-items: center; height: 100vh; background: var(--md-sys-color-background);">
        Loading...
    </div>
{/if}

<style>
    :global(body) {
        margin: 0;
        font-family: 'Roboto', sans-serif;
    }

    .nav-link {
        text-decoration: none;
        color: inherit;
        padding: 0.5rem 1rem;
        border-radius: 100px;
        transition: background-color 0.2s;
        font-size: 0.875rem;
        font-weight: 500;
        opacity: 0.8;
    }

    .nav-link:hover {
        background-color: rgba(0,0,0,0.05);
        opacity: 1;
    }

    .nav-link.active {
        background-color: var(--md-sys-color-secondary-container);
        color: var(--md-sys-color-on-secondary-container);
        opacity: 1;
    }

    .icon-btn {
        background: none;
        border: none;
        cursor: pointer;
        color: inherit;
        padding: 8px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color 0.2s;
    }

    .icon-btn:hover {
        background-color: rgba(0,0,0,0.05);
    }

    .divider {
        width: 1px;
        height: 24px;
        background-color: rgba(0,0,0,0.1);
    }
</style>

<script lang="ts">
	import '../app.scss'; // Global styles
	import { onMount } from 'svelte';
	import { user, token, themeStore, tenantStore } from '$lib/stores';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { fade } from 'svelte/transition';
	
	// Components
	import UserMenu from '$lib/components/UserMenu.svelte';
	import TenantSelector from '$lib/components/TenantSelector.svelte';

	let loading = true;
	let showSidebar = false;

	// Auth Guard
	onMount(async () => {
		// Initialize theme
		document.body.setAttribute('data-theme', $themeStore);
		themeStore.subscribe(val => {
			document.body.setAttribute('data-theme', val);
		});

        // Check App Config (Auth Mode)
        try {
            const res = await fetch('/api/v1/app-config');
            if (res.ok) {
                const config = await res.json();
                if (!config.auth_enabled) {
                    // NO_AUTH mode: Auto-login as root
                    if (!$user) {
                        user.set({ 
                            id: '00000000-0000-0000-0000-000000000000', 
                            username: 'root', 
                            email: 'root@local', 
                            role: 'Root',
                            tenant_id: null 
                        });
                        token.set('no-auth-token');
                    }
                }
            }
        } catch (e) {
            console.error('Failed to fetch app config', e);
        }

		// Check Auth
		const unsubscribe = user.subscribe(u => {
			if (!u && $page.url.pathname !== '/login') {
				goto('/login');
			}
			if (u && $page.url.pathname === '/login') {
				goto('/');
			}
		});

		loading = false;
		return () => unsubscribe();
	});

	function toggleSidebar() {
		showSidebar = !showSidebar;
	}
</script>

{#if loading}
	<div class="loading-screen">
		<div class="spinner"></div>
		<p>Loading Pangolin...</p>
	</div>
{:else}
	<div class="app-layout">
		{#if $user}
			<!-- Sidebar (Desktop) -->
			<aside class="sidebar" class:open={showSidebar}>
				<div class="sidebar-header">
					<img src="/logo.png" alt="Pangolin" class="logo" />
					<span class="brand">Pangolin</span>
					<button class="icon-btn mobile-only" on:click={toggleSidebar}>
						<span class="material-icons">close</span>
					</button>
				</div>

				<nav class="nav-links">
					<a href="/" class:active={$page.url.pathname === '/'} on:click={() => showSidebar = false}>
						<span class="material-icons">dashboard</span>
						Dashboard
					</a>
					<div class="divider"></div>
					<div class="section-title">Catalog</div>
					<a href="/search" class:active={$page.url.pathname.startsWith('/search')} on:click={() => showSidebar = false}>
						<span class="material-icons">search</span>
						Search
					</a>
					<a href="/access-requests" class:active={$page.url.pathname.startsWith('/access-requests')} on:click={() => showSidebar = false}>
						<span class="material-icons">lock_person</span>
						Access Requests
					</a>
					
					<div class="divider"></div>
					<div class="section-title">Management</div>
					{#if $user.role !== 'TenantUser'}
						<a href="/users" class:active={$page.url.pathname.startsWith('/users')} on:click={() => showSidebar = false}>
							<span class="material-icons">group</span>
							Users
						</a>
						<a href="/roles" class:active={$page.url.pathname.startsWith('/roles')} on:click={() => showSidebar = false}>
							<span class="material-icons">shield</span>
							Roles
						</a>
						<a href="/permissions" class:active={$page.url.pathname.startsWith('/permissions')} on:click={() => showSidebar = false}>
							<span class="material-icons">vpn_key</span>
							Permissions
						</a>
					{/if}
				</nav>

                <div class="sidebar-footer">
                    <p class="version">v0.1.0 Alpha</p>
                </div>
			</aside>

			<!-- Main Content -->
			<main class="main-content">
				<header class="top-bar">
					<div class="left">
						<button class="icon-btn menu-btn" on:click={toggleSidebar}>
							<span class="material-icons">menu</span>
						</button>
                        {#if $user.role === 'Root'}
						    <TenantSelector />
                        {/if}
					</div>
					<div class="right">
						<UserMenu />
					</div>
				</header>
				<div class="page-container">
					<slot />
				</div>
			</main>

			<!-- Backdrop for mobile sidebar -->
			{#if showSidebar}
				<div class="backdrop" on:click={toggleSidebar} transition:fade></div>
			{/if}
		{:else}
			<slot />
		{/if}
	</div>
{/if}

<style>
	/* Layout Variables */
	:global(:root) {
		--sidebar-width: 260px;
		--top-bar-height: 64px;
	}

	.app-layout {
		display: flex;
		height: 100vh;
		background-color: var(--md-sys-color-background);
		color: var(--md-sys-color-on-background);
        transition: background-color 0.3s, color 0.3s;
	}

	.loading-screen {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		height: 100vh;
		background-color: var(--md-sys-color-surface);
		color: var(--md-sys-color-primary);
	}

	/* Sidebar */
	.sidebar {
		width: var(--sidebar-width);
		background-color: var(--md-sys-color-surface-container);
		border-right: 1px solid var(--md-sys-color-outline-variant);
		display: flex;
		flex-direction: column;
		transition: transform 0.3s ease;
		z-index: 100;
        height: 100vh;
	}

	.sidebar-header {
		height: var(--top-bar-height);
		display: flex;
		align-items: center;
		padding: 0 1.5rem;
		gap: 0.75rem;
	}

	.logo { width: 32px; height: 32px; }
	.brand { font-size: 1.25rem; font-weight: 500; color: var(--md-sys-color-primary); }

	.nav-links {
		flex: 1;
		padding: 1rem;
		overflow-y: auto;
	}

	.nav-links a {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1rem;
		margin-bottom: 0.25rem;
		border-radius: 100px; /* Pillow shape */
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		font-weight: 500;
		transition: background-color 0.2s, color 0.2s;
	}

	.nav-links a:hover {
		background-color: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}

	.nav-links a.active {
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}

    .section-title {
        font-size: 0.75rem;
        font-weight: bold;
        text-transform: uppercase;
        color: var(--md-sys-color-outline);
        margin: 1rem 0 0.5rem 1rem;
    }

	.divider {
		height: 1px;
		background-color: var(--md-sys-color-outline-variant);
		margin: 0.5rem 0;
	}

    .sidebar-footer {
        padding: 1rem;
        text-align: center;
        border-top: 1px solid var(--md-sys-color-outline-variant);
    }
    .version { font-size: 0.75rem; color: var(--md-sys-color-outline); margin: 0; }

	/* Main Content */
	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.top-bar {
		height: var(--top-bar-height);
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0 1.5rem;
		/* background-color: var(--md-sys-color-surface); Can be transparent or surface */
        border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
    
    .top-bar .left { display: flex; align-items: center; gap: 1rem; }

	.page-container {
		flex: 1;
		overflow-y: auto;
		padding: 1rem; /* Can be 0 for full width pages */
	}

	/* Responsive */
	.mobile-only { display: none; }
	.menu-btn { display: none; }

	@media (max-width: 768px) {
		.sidebar {
			position: fixed;
			top: 0;
			left: 0;
			bottom: 0;
			transform: translateX(-100%);
		}
		.sidebar.open { transform: translateX(0); }
		.menu-btn { display: block; }
		.mobile-only { display: block; }
		
		.backdrop {
			position: fixed;
			top: 0; left: 0; right: 0; bottom: 0;
			background-color: rgba(0,0,0,0.5);
			z-index: 90;
		}
	}
    
    .icon-btn {
        background: none; border: none; cursor: pointer; color: var(--md-sys-color-on-surface); padding: 8px; border-radius: 50%; display: flex; align-items: center; justify-content: center;
    }
    .icon-btn:hover { background-color: rgba(0,0,0,0.05); }

    .spinner {
         width: 40px; height: 40px; border: 4px solid var(--md-sys-color-primary-container); border-top-color: var(--md-sys-color-primary); border-radius: 50%; animation: spin 1s linear infinite; margin-bottom: 1rem;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
</style>

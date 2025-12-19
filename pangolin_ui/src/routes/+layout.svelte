<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { authStore, isRoot, isTenantAdmin } from '$lib/stores/auth';
	import { themeStore } from '$lib/stores/theme';
	import { tenantStore } from '$lib/stores/tenant';
	import { helpStore } from '$lib/stores/help';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import Notification from '$lib/components/ui/Notification.svelte';
	import HelpPanel from '$lib/components/ui/HelpPanel.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import '../app.css';

	let sidebarOpen = true;
	let tenants: Tenant[] = [];
	let loadingTenants = false;

	onMount(() => {
		// Initialize auth - checks server config and auto-authenticates if NO_AUTH mode
		const init = async () => {
			await authStore.initialize();
		};
		init();
		
		// Load theme
		themeStore.loadTheme();

		// Redirect to login if not authenticated (and auth is enabled)
		const unsubscribeAuth = authStore.subscribe(async state => {
			if (!state.isLoading && !state.isAuthenticated && state.authEnabled && $page.url.pathname !== '/login') {
				goto('/login');
			} else if (!state.isLoading && state.isAuthenticated && $page.url.pathname === '/login') {
				// If authenticated and on login page, redirect to dashboard
				goto('/');
			}

			// Load tenants if root and not loaded
			if (state.isAuthenticated && state.user?.role?.toLowerCase() === 'root' && tenants.length === 0 && !loadingTenants) {
				loadingTenants = true;
				try {
					tenants = await tenantsApi.list();
				} catch (e) {
					console.error('Failed to load tenants for switcher', e);
				} finally {
					loadingTenants = false;
				}
			}
		});

		return unsubscribeAuth;
	});

	// Route guards
	$: if ($authStore.isAuthenticated && !$authStore.isLoading) {
		const path = $page.url.pathname;
		
		// Root-only routes
		const rootRoutes = ['/tenants', '/root-dashboard'];
		if (rootRoutes.some(r => path === r || path.startsWith(r + '/')) && !$isRoot) {
			goto('/');
		}

		// Admin-only routes (Root or TenantAdmin)
		const adminRoutes = ['/users', '/warehouses', '/roles', '/admin/requests'];
		if (adminRoutes.some(r => path === r || path.startsWith(r + '/')) && !$isRoot && !$isTenantAdmin) {
			goto('/');
		}
	}

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
	}

	function handleTenantChange(event: Event) {
		const select = event.target as HTMLSelectElement;
		const tenantId = select.value;
		
		if (tenantId) {
			const tenant = tenants.find(t => t.id === tenantId);
			if (tenant) {
				tenantStore.selectTenant(tenant.id, tenant.name);
			}
		} else {
			tenantStore.clearTenant();
		}
		
		// Reload to apply context globally (simplest way to ensure all API calls and stores use new context)
		window.location.reload(); 
	}
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900">
	{#if $authStore.isLoading}
		<!-- Loading state -->
		<div class="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-900">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
		</div>
	{:else if $authStore.isAuthenticated}
		<!-- Authenticated layout with sidebar -->
		<div class="flex h-screen overflow-hidden">
			<!-- Sidebar -->
			<aside
				class="bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 transition-all duration-300 {sidebarOpen ? 'w-64' : 'w-16'}"
			>
				<div class="h-full flex flex-col">
					<!-- Logo -->
					<div class="p-4 border-b border-gray-200 dark:border-gray-700">
						<div class="flex items-center gap-3">
							<div class="w-8 h-8 bg-primary-600 rounded-lg flex items-center justify-center text-white font-bold">
								P
							</div>
							{#if sidebarOpen}
								<span class="font-semibold text-gray-900 dark:text-white">Pangolin</span>
							{/if}
						</div>
					</div>

					<!-- Navigation -->
					<nav class="flex-1 p-4 space-y-2 overflow-y-auto custom-scrollbar">
						{#if !$isRoot}
						<a
							href="/"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">📊</span>
							{#if sidebarOpen}
								<span>Dashboard</span>
							{/if}
						</a>
						<a
							href="/explorer"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">🔍</span>
							{#if sidebarOpen}
								<span>Data Explorer</span>
							{/if}
						</a>
						<a
							href="/discovery"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">🔎</span>
							{#if sidebarOpen}
								<span>Discovery</span>
							{/if}
						</a>
						{/if}
						{#if $isRoot}
						<a
							href="/root-dashboard"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">🌐</span>
							{#if sidebarOpen}
								<span>Root Dashboard</span>
							{/if}
						</a>
						<a
							href="/tenants"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">🏛️</span>
							{#if sidebarOpen}
								<span>Tenants</span>
							{/if}
						</a>
						{/if}
						
						{#if !$isRoot}
						<a
							href="/catalogs"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">📚</span>
							{#if sidebarOpen}
								<span>Catalogs</span>
							{/if}
						</a>
						{#if $isTenantAdmin}
						<a
							href="/warehouses"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">🏢</span>
							{#if sidebarOpen}
								<span>Warehouses</span>
							{/if}
						</a>
						<a
							href="/users"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">👥</span>
							{#if sidebarOpen}

								<span>Users</span>
							{/if}
						</a>
						<a
							href="/roles"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">🛡️</span>
							{#if sidebarOpen}
								<span>Roles</span>
							{/if}
						</a>
						<a
							href="/admin/requests"
							class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">📫</span>
							{#if sidebarOpen}
								<span>Access Requests</span>
							{/if}
						</a>
						{/if}
						{/if}
					</nav>

					<!-- Toggle button -->
					<div class="p-4 border-t border-gray-200 dark:border-gray-700">
						<button
							on:click={toggleSidebar}
							class="w-full flex items-center justify-center px-3 py-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						>
							<span class="text-xl">{sidebarOpen ? '◀' : '▶'}</span>
						</button>
					</div>
				</div>
			</aside>

			<!-- Main content -->
			<div class="flex-1 flex flex-col overflow-hidden">
				<!-- Top bar -->
				<header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
					<div class="flex items-center justify-between">
						<div>
							<h2 class="text-xl font-semibold text-gray-900 dark:text-white">
								{$page.url.pathname === '/' ? 'Dashboard' : $page.url.pathname.split('/')[1] || 'Pangolin'}
							</h2>
						</div>

						<div class="flex items-center gap-4">
							<!-- Context Switcher Removed for Root -->

							<!-- Theme toggle -->
							<button
								on:click={() => {
									const current = $themeStore;
									const next = current === 'light' ? 'dark' : current === 'dark' ? 'system' : 'light';
									themeStore.setTheme(next);
								}}
								class="p-2 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
								title="Toggle theme"
							>
								{#if $themeStore === 'light'}
									☀️
								{:else if $themeStore === 'dark'}
									🌙
								{:else}
									💻
								{/if}
							</button>

							<!-- User menu -->
							<UserMenu />
						</div>
					</div>
				</header>

				<!-- Page content -->
				<main class="flex-1 overflow-y-auto p-6 custom-scrollbar">
					<slot />
				</main>
			</div>
		</div>
	{:else if $page.url.pathname === '/login'}
		<!-- Unauthenticated layout (login page) -->
		<slot />
	{/if}
</div>

<!-- Global Components -->
<Notification />
<HelpPanel />

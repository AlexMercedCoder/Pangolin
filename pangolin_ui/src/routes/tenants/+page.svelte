<script lang="ts">
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth';
	import Card from '$lib/components/ui/Card.svelte';

	let tenants: any[] = [];
	let error = '';
	let loading = true;
	let noAuthMode = false;

	onMount(async () => {
		try {
			// Check if we're in NO_AUTH mode
			const configRes = await fetch('http://localhost:8080/api/v1/app-config');
			if (configRes.ok) {
				const config = await configRes.json();
				noAuthMode = !config.auth_enabled;
			}

			// Try to fetch tenants
			const res = await fetch('http://localhost:8080/api/v1/tenants');
			if (res.ok) {
				tenants = await res.json();
			} else if (res.status === 403) {
				// Forbidden - likely NO_AUTH mode
				const errorData = await res.json();
				error = errorData.error || 'Cannot create tenants in NO_AUTH mode';
			} else {
				error = 'Failed to load tenants';
			}
		} catch (e: any) {
			error = e.message || 'Failed to connect to server';
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Tenants - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Tenants</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Manage multi-tenant isolation and organization
		</p>
	</div>

	{#if loading}
		<Card>
			<p class="text-gray-600 dark:text-gray-400">Loading tenants...</p>
		</Card>
	{:else if noAuthMode}
		<Card>
			<div class="space-y-4">
				<div class="flex items-start gap-3">
					<span class="text-3xl">⚠️</span>
					<div>
						<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
							Tenant Management Disabled in NO_AUTH Mode
						</h3>
						<p class="mt-2 text-gray-600 dark:text-gray-400">
							NO_AUTH mode is designed for local development and testing with a single default tenant.
							Multi-tenant features are not available in this mode.
						</p>
						<div class="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
							<p class="text-sm text-blue-800 dark:text-blue-200 font-medium">
								To enable tenant management:
							</p>
							<ol class="mt-2 ml-4 text-sm text-blue-700 dark:text-blue-300 space-y-1 list-decimal">
								<li>Stop the API server</li>
								<li>Unset or remove the <code class="px-1 py-0.5 bg-blue-100 dark:bg-blue-800 rounded">PANGOLIN_NO_AUTH</code> environment variable</li>
								<li>Set up authentication with JWT tokens or OAuth</li>
								<li>Restart the server</li>
							</ol>
						</div>
					</div>
				</div>
				<div class="mt-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
					<p class="text-sm text-gray-600 dark:text-gray-400">
						<strong>Default Tenant:</strong> All operations in NO_AUTH mode use the default tenant
						(<code class="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded font-mono text-xs">00000000-0000-0000-0000-000000000000</code>)
					</p>
				</div>
			</div>
		</Card>
	{:else if error}
		<Card>
			<div class="flex items-start gap-3">
				<span class="text-2xl">❌</span>
				<div>
					<h3 class="font-semibold text-gray-900 dark:text-white">Error Loading Tenants</h3>
					<p class="mt-1 text-gray-600 dark:text-gray-400">{error}</p>
				</div>
			</div>
		</Card>
	{:else if tenants.length === 0}
		<Card>
			<div class="text-center py-8">
				<span class="text-6xl">🏛️</span>
				<h3 class="mt-4 text-lg font-semibold text-gray-900 dark:text-white">
					No Tenants Yet
				</h3>
				<p class="mt-2 text-gray-600 dark:text-gray-400">
					Create your first tenant to get started with multi-tenant organization
				</p>
				<button class="mt-4 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors">
					Create Tenant
				</button>
			</div>
		</Card>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each tenants as tenant}
				<Card>
					<div class="space-y-3">
						<div class="flex items-start justify-between">
							<div>
								<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
									{tenant.name}
								</h3>
								<p class="mt-1 text-sm text-gray-500 dark:text-gray-400 font-mono">
									{tenant.id}
								</p>
							</div>
							<span class="text-2xl">🏛️</span>
						</div>
						{#if tenant.properties && Object.keys(tenant.properties).length > 0}
							<div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
								<p class="text-xs text-gray-500 dark:text-gray-400 uppercase tracking-wide">Properties</p>
								<div class="mt-2 space-y-1">
									{#each Object.entries(tenant.properties) as [key, value]}
										<div class="text-sm">
											<span class="text-gray-600 dark:text-gray-400">{key}:</span>
											<span class="ml-2 text-gray-900 dark:text-white">{value}</span>
										</div>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				</Card>
			{/each}
		</div>
	{/if}
</div>

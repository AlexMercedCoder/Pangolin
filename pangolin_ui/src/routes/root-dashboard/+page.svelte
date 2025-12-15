<script lang="ts">
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { usersApi, type User } from '$lib/api/users';
	import { onMount } from 'svelte';
	
	let tenantCount = 0;
	let userCount = 0;
	let loading = true;

	onMount(async () => {
		try {
			const [tenants, users] = await Promise.all([
				tenantsApi.list(),
				usersApi.list()
			]);
			tenantCount = tenants.length;
			userCount = users.length;
		} catch (e) {
			console.error('Failed to load root dashboard data', e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-2xl font-bold text-gray-900 dark:text-white">System Overview</h1>
		<p class="text-gray-500 dark:text-gray-400">Global metrics for the Pangolin instance.</p>
	</div>

	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
		<!-- Tenant Count -->
		<div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-4">
				<div class="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg text-blue-600 dark:text-blue-400">
					<span class="text-2xl">🏛️</span>
				</div>
				<div>
					<p class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Tenants</p>
					{#if loading}
						<div class="h-8 w-16 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mt-1"></div>
					{:else}
						<p class="text-2xl font-semibold text-gray-900 dark:text-white">{tenantCount}</p>
					{/if}
				</div>
			</div>
		</div>

		<!-- User Count -->
		<div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-4">
				<div class="p-3 bg-purple-100 dark:bg-purple-900/30 rounded-lg text-purple-600 dark:text-purple-400">
					<span class="text-2xl">👥</span>
				</div>
				<div>
					<p class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Users</p>
					{#if loading}
						<div class="h-8 w-16 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mt-1"></div>
					{:else}
						<p class="text-2xl font-semibold text-gray-900 dark:text-white">{userCount}</p>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>

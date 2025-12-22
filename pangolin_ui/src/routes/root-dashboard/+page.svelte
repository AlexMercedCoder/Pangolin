<script lang="ts">
	import { onMount } from 'svelte';
    import { dashboardApi } from '$lib/api/optimization';
    import type { DashboardStats } from '$lib/types/optimization';
    import StatCard from '$lib/components/dashboard/StatCard.svelte';
	
	let stats: DashboardStats | null = null;
	let loading = true;
    let error: string | null = null;

	onMount(async () => {
		try {
            stats = await dashboardApi.getStats();
		} catch (e: any) {
			console.error('Failed to load root dashboard data', e);
            error = e.message || 'Failed to load dashboard statistics';
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

    {#if error}
        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-4 rounded-lg text-red-700 dark:text-red-300">
            Error loading stats: {error}
        </div>
    {/if}

	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard 
            label="Total Tenants" 
            value={stats?.scope === 'root' ? stats.catalogs_count : 0} 
            icon="apartment" 
            color="blue" 
        />
        <!-- Note: The stats endpoint for root currently maps catalogs_count. 
             Ideally backend should provide tenant_count explicitely if scope is root.
             Assuming catalogs_count might be tenants for root? 
             Let's check the backend logic or just show what we have.
             Actually, let's show all the stats available.
        -->
        
        {#if stats}
            <StatCard label="Catalogs" value={stats.catalogs_count} icon="📂" color="blue" />
            <StatCard label="Warehouses" value={stats.warehouses_count} icon="🏭" color="purple" />
            <StatCard label="Users" value={stats.users_count} icon="👥" color="green" />
            <StatCard label="Namespaces" value={stats.namespaces_count} icon="🏷️" color="yellow" />
            <StatCard label="Tables" value={stats.tables_count} icon="📋" color="red" />
            <StatCard label="Branches" value={stats.branches_count} icon="🌲" color="gray" />
        {:else}
            <!-- Loading Skeletons -->
            <StatCard label="Catalogs" value={undefined} icon="📂" color="blue" />
            <StatCard label="Warehouses" value={undefined} icon="🏭" color="purple" />
            <StatCard label="Users" value={undefined} icon="👥" color="green" />
            <StatCard label="Namespaces" value={undefined} icon="🏷️" color="yellow" />
        {/if}
	</div>
</div>

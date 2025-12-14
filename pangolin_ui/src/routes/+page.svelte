<script lang="ts">
	import Card from '$lib/components/ui/Card.svelte';
</script>

<svelte:head>
	<title>Dashboard - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">Welcome to Pangolin Lakehouse Catalog</p>
	</div>

	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<Card title="Catalogs">
			<div class="text-center">
				<p class="text-4xl font-bold text-primary-600">0</p>
				<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">Total Catalogs</p>
			</div>
		</Card>

		<Card title="Namespaces">
			<div class="text-center">
				<p class="text-4xl font-bold text-secondary-600">0</p>
				<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">Total Namespaces</p>
			</div>
		</Card>

		<Card title="Tables">
			<div class="text-center">
				<p class="text-4xl font-bold text-success-600">0</p>
				<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">Total Tables</p>
			</div>
		</Card>
	</div>

	<Card title="Recent Activity">
		<p class="text-gray-600 dark:text-gray-400">No recent activity</p>
	</Card>
</div>

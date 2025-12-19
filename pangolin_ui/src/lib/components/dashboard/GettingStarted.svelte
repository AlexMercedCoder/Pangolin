<script lang="ts">
	import { authStore } from '$lib/stores/auth';
	import { tenantStore } from '$lib/stores/tenant';
	import { catalogsApi } from '$lib/api/catalogs';
	import { onMount } from 'svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { notifications } from '$lib/stores/notifications';

	let firstCatalogName = '';
	let loading = true;

	onMount(async () => {
		try {
			const catalogs = await catalogsApi.list();
			if (catalogs.length > 0) {
				firstCatalogName = catalogs[0].name;
			}
		} catch (e) {
			console.error('Failed to load catalogs:', e);
		} finally {
			loading = false;
		}
	});

	$: apiUrl = typeof window !== 'undefined' ? window.location.origin : 'http://localhost:8080';
	$: tenantId = $tenantStore.selectedTenantId || '<TENANT_ID>';
	$: catalogName = firstCatalogName || '<CATALOG_NAME>';

	$: codeSnippet = `from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", **{
    "uri": "${apiUrl}/api/v1/catalogs/${catalogName}",
    "s3.endpoint": "http://<S3_HOST>:9000",
    "py-iceberg.catalog-impl": "pyiceberg.catalog.rest.RestCatalog",
    "header.X-Pangolin-Tenant": "${tenantId}",
    "token": "<YOUR_ACCESS_TOKEN>"
})

# List namespaces
namespaces = catalog.list_namespaces()
print(namespaces)

# Create a namespace
catalog.create_namespace("my_namespace")

# List tables
tables = catalog.list_tables("my_namespace")
print(tables)`;

	async function copyToClipboard() {
		try {
			await navigator.clipboard.writeText(codeSnippet);
			notifications.success('Code copied to clipboard!');
		} catch (err) {
			notifications.error('Failed to copy to clipboard');
		}
	}
</script>

<Card>
	<div class="p-6">
		<div class="flex justify-between items-start mb-4">
			<div>
				<h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
					Getting Started with PyIceberg
				</h2>
				<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
					Connect to Pangolin using the Python Iceberg client
				</p>
			</div>
			<Button size="sm" variant="ghost" on:click={copyToClipboard}>
				<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
				</svg>
				Copy
			</Button>
		</div>

		{#if loading}
			<div class="animate-pulse">
				<div class="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
			</div>
		{:else}
			<div class="relative">
				<pre class="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm font-mono"><code>{codeSnippet}</code></pre>
			</div>

			<div class="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
				<p class="text-sm text-blue-800 dark:text-blue-200">
					<strong>Note:</strong> Replace <code class="bg-blue-100 dark:bg-blue-800 px-1 rounded">&lt;S3_HOST&gt;</code> with your S3 endpoint 
					and <code class="bg-blue-100 dark:bg-blue-800 px-1 rounded">&lt;YOUR_ACCESS_TOKEN&gt;</code> with a valid access token.
					{#if !firstCatalogName}
						Create a catalog first to get started.
					{/if}
				</p>
			</div>
		{/if}
	</div>
</Card>

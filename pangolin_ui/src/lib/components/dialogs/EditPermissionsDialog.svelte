<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { onMount } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { permissionsApi, type Permission, type PermissionScope, type Action, getScopeDisplay } from '$lib/api/permissions';
	import { catalogsApi } from '$lib/api/catalogs';
	import { notifications } from '$lib/stores/notifications';

	export let open = false;
	export let entityType: 'user' | 'role' = 'user';
	export let entityId: string;
	export let entityName: string;

	const dispatch = createEventDispatcher();

	let permissions: Permission[] = [];
	let catalogs: any[] = [];
	let loading = true;
	let submitting = false;

	// Form state for adding permission
	let scopeType: 'Global' | 'Catalog' | 'Namespace' | 'Table' | 'Tag' = 'Catalog';
	let selectedCatalog = '';
	let namespace = '';
	let tableName = '';
	let tagName = '';
	let selectedActions: Set<Action> = new Set(['Read']);

	const allActions: Action[] = ['Read', 'Write', 'Delete', 'Admin'];

	onMount(async () => {
		if (open) {
			await loadData();
		}
	});

	$: if (open) {
		loadData();
	}

	async function loadData() {
		loading = true;
		try {
			// Load catalogs for scope selection
			catalogs = await catalogsApi.list();
			if (catalogs.length > 0) {
				selectedCatalog = catalogs[0].name;
			}

			// Load existing permissions for this entity
			if (entityType === 'user') {
				permissions = await permissionsApi.getUserPermissions(entityId);
			}
			// Note: Role permissions would need a separate endpoint
		} catch (error: any) {
			notifications.error(`Failed to load data: ${error.message}`);
		}
		loading = false;
	}

	function buildScope(): PermissionScope {
		switch (scopeType) {
			case 'Global':
				return { Global: null };
			case 'Catalog':
				return { Catalog: selectedCatalog };
			case 'Namespace':
				const nsParts = namespace.split('.').filter(p => p.length > 0);
				return { Namespace: { catalog: selectedCatalog, namespace: nsParts } };
			case 'Table':
				const tblNsParts = namespace.split('.').filter(p => p.length > 0);
				return { Table: { catalog: selectedCatalog, namespace: tblNsParts, table: tableName } };
			case 'Tag':
				return { Tag: tagName };
		}
	}

	async function handleAddPermission() {
		if (selectedActions.size === 0) {
			notifications.error('Please select at least one action');
			return;
		}

		if (scopeType !== 'Global' && !selectedCatalog) {
			notifications.error('Please select a catalog');
			return;
		}

		if (scopeType === 'Namespace' && !namespace) {
			notifications.error('Please enter a namespace');
			return;
		}

		if (scopeType === 'Table' && (!namespace || !tableName)) {
			notifications.error('Please enter namespace and table name');
			return;
		}

		if (scopeType === 'Tag' && !tagName) {
			notifications.error('Please enter a tag name');
			return;
		}

		submitting = true;
		try {
			const scope = buildScope();
			const permission = await permissionsApi.grant({
				user_id: entityId,
				scope,
				actions: Array.from(selectedActions)
			});

			permissions = [...permissions, permission];
			
			// Reset form
			selectedActions = new Set(['Read']);
			namespace = '';
			tableName = '';

			notifications.success('Permission granted successfully');
		} catch (error: any) {
			notifications.error(`Failed to grant permission: ${error.message}`);
		}
		submitting = false;
	}

	async function handleRevokePermission(permissionId: string) {
		try {
			await permissionsApi.revoke(permissionId);
			permissions = permissions.filter(p => p.id !== permissionId);
			notifications.success('Permission revoked successfully');
		} catch (error: any) {
			notifications.error(`Failed to revoke permission: ${error.message}`);
		}
	}

	function toggleAction(action: Action) {
		if (selectedActions.has(action)) {
			selectedActions.delete(action);
		} else {
			selectedActions.add(action);
		}
		selectedActions = selectedActions; // Trigger reactivity
	}

	function handleClose() {
		open = false;
		dispatch('updated');
	}
</script>

{#if open}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col">
			<!-- Header -->
			<div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white">
					Edit Permissions: {entityName}
				</h2>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-y-auto p-6 space-y-6">
				<!-- Add Permission Form -->
				<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 space-y-4">
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">Add Permission</h3>

					<!-- Scope Type -->
					<!-- Scope Type -->
					<div>
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
							Scope Type
						</label>
						<select
							bind:value={scopeType}
							class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 dark:text-white"
						>
							<option value="Global">Global</option>
							<option value="Catalog">Catalog</option>
							<option value="Namespace">Namespace</option>
							<option value="Table">Table</option>
							<option value="Tag">Tag</option>
						</select>
					</div>

					<!-- Catalog (for Catalog, Namespace, Table) -->
					{#if scopeType === 'Catalog' || scopeType === 'Namespace' || scopeType === 'Table'}
						<div>
							<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
								Catalog
							</label>
							<select
								bind:value={selectedCatalog}
								class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 dark:text-white"
							>
								{#each catalogs as catalog}
									<option value={catalog.name}>{catalog.name}</option>
								{/each}
							</select>
						</div>
					{/if}

					<!-- Namespace (for Namespace and Table) -->
					{#if scopeType === 'Namespace' || scopeType === 'Table'}
						<Input
							label="Namespace"
							bind:value={namespace}
							placeholder="level1.level2"
							helpText="Dot-separated namespace path"
						/>
					{/if}

					<!-- Table Name (for Table only) -->
					{#if scopeType === 'Table'}
						<Input
							label="Table Name"
							bind:value={tableName}
							placeholder="my_table"
						/>
					{/if}

					<!-- Tag Name (for Tag-based ABAC) -->
					{#if scopeType === 'Tag'}
						<Input
							label="Tag Name"
							bind:value={tagName}
							placeholder="pii"
							helpText="Grant access to all assets with this tag (Attribute-Based Access Control)"
						/>
					{/if}

					<!-- Actions -->
					<div class="space-y-2">
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
							Actions
						</label>
						<div class="grid grid-cols-2 md:grid-cols-4 gap-3">
							{#each allActions as action}
								<label class="flex items-center gap-2 cursor-pointer">
									<input
										type="checkbox"
										checked={selectedActions.has(action)}
										on:change={() => toggleAction(action)}
										class="rounded border-gray-300 text-primary-600 focus:ring-primary-500"
									/>
									<span class="text-sm text-gray-700 dark:text-gray-300">{action}</span>
								</label>
							{/each}
						</div>
					</div>

					<Button on:click={handleAddPermission} loading={submitting} disabled={loading}>
						{submitting ? 'Adding...' : 'Add Permission'}
					</Button>
				</div>

				<!-- Existing Permissions -->
				<div class="space-y-3">
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">
						Current Permissions ({permissions.length})
					</h3>

					{#if loading}
						<div class="flex items-center justify-center py-8">
							<div class="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin" />
						</div>
					{:else if permissions.length === 0}
						<p class="text-gray-600 dark:text-gray-400 text-center py-8">
							No permissions assigned yet
						</p>
					{:else}
						<div class="space-y-2">
							{#each permissions as permission}
								<div class="flex items-center justify-between p-3 bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
									<div class="flex-1">
										<div class="font-medium text-gray-900 dark:text-white">
											{getScopeDisplay(permission.scope)}
										</div>
										<div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
											Actions: {permission.actions.join(', ')}
										</div>
									</div>
									<button
										on:click={() => handleRevokePermission(permission.id)}
										class="text-error-600 hover:text-error-700 dark:text-error-400 dark:hover:text-error-300 text-sm font-medium"
									>
										Revoke
									</button>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<!-- Footer -->
			<div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end">
				<Button variant="ghost" on:click={handleClose}>
					Close
				</Button>
			</div>
		</div>
	</div>
{/if}

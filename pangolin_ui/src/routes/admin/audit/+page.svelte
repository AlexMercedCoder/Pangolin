<script lang="ts">
	import { onMount } from 'svelte';
	import { auditApi, type AuditLogEntry, type AuditListQuery } from '$lib/api/audit';
	import { usersApi, type User } from '$lib/api/users';
	import { notifications } from '$lib/stores/notifications';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
    import { isRoot } from '$lib/stores/auth';

	let events: AuditLogEntry[] = [];
	let users: User[] = [];
	let loading = true;
	let totalCount = 0;
	
	// Filter state
	let filters: AuditListQuery = {
		limit: 20,
		offset: 0
	};
    
    let selectedEvent: AuditLogEntry | null = null;
    let showDetailModal = false;

	onMount(async () => {
		await Promise.all([
			loadEvents(),
			loadUsers()
		]);
	});

	async function loadEvents() {
		loading = true;
		try {
			const [eventsData, countData] = await Promise.all([
				auditApi.list(filters),
				auditApi.count(filters)
			]);
			events = eventsData;
			totalCount = countData;
		} catch (error: any) {
			notifications.error(`Failed to load audit logs: ${error.message}`);
		} finally {
			loading = false;
		}
	}

	async function loadUsers() {
		try {
			users = await usersApi.list();
		} catch (error) {
			console.error('Failed to load users for filter', error);
		}
	}

	function handleFilterChange() {
		filters.offset = 0;
		loadEvents();
	}

	function handlePageChange(newOffset: number) {
		filters.offset = newOffset;
		loadEvents();
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleString();
	}

	function getActionColor(action: string): 'success' | 'warning' | 'error' | 'info' {
        action = action.toLowerCase();
		if (action.includes('create') || action.includes('grant')) return 'success';
		if (action.includes('delete') || action.includes('revoke')) return 'error';
		if (action.includes('update')) return 'warning';
		return 'info';
	}
    
    function viewDetails(event: AuditLogEntry) {
        selectedEvent = event;
        showDetailModal = true;
    }
</script>

<svelte:head>
	<title>Audit Logs - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Audit Logs</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Track all administrative and security actions across the system.
			</p>
		</div>
        <Button variant="secondary" on:click={loadEvents} disabled={loading}>
            <span class="mr-2 text-sm">🔄</span> Refresh
        </Button>
	</div>

	<Card>
		<div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
			<div>
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">User</label>
				<select 
					bind:value={filters.user_id} 
					on:change={handleFilterChange}
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white"
				>
					<option value={undefined}>All Users</option>
					{#each users as user}
						<option value={user.id}>{user.username}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Result</label>
				<select 
					bind:value={filters.result} 
					on:change={handleFilterChange}
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white"
				>
					<option value={undefined}>All Results</option>
					<option value="success">Success</option>
					<option value="failure">Failure</option>
				</select>
			</div>
			<div>
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Resource Type</label>
				<input 
					type="text" 
					placeholder="e.g. table, user"
					bind:value={filters.resource_type}
					on:change={handleFilterChange}
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white text-sm"
				/>
			</div>
            <div>
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Action</label>
				<input 
					type="text" 
					placeholder="e.g. create_table"
					bind:value={filters.action}
					on:change={handleFilterChange}
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white text-sm"
				/>
			</div>
		</div>

		{#if loading && events.length === 0}
			<div class="flex justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
			</div>
		{:else if events.length === 0}
			<div class="text-center py-12 text-gray-500">
				No audit logs found matching the filters.
			</div>
		{:else}
			<div class="overflow-x-auto">
				<table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
					<thead class="bg-gray-50 dark:bg-gray-800">
						<tr>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Timestamp</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">User</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Action</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Resource</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Result</th>
							<th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
						</tr>
					</thead>
					<tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
						{#each events as event}
							<tr class="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
								<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
									{formatDate(event.timestamp)}
								</td>
								<td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
									{event.username}
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<Badge variant={getActionColor(event.action)}>{event.action.replace('_', ' ')}</Badge>
								</td>
								<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
									<span class="font-medium text-gray-700 dark:text-gray-200">{event.resource_type}:</span> {event.resource_name || event.resource_id || '-'}
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									{#if event.result === 'success'}
										<span class="inline-flex items-center text-green-600 dark:text-green-400 text-sm">
											<span class="text-xs mr-1">✅</span> Success
										</span>
									{:else}
										<span class="inline-flex items-center text-red-600 dark:text-red-400 text-sm">
											<span class="text-xs mr-1">❌</span> Failure
										</span>
									{/if}
								</td>
								<td class="px-6 py-4 whitespace-nowrap text-right text-sm">
									<button 
                                        class="text-primary-600 hover:text-primary-900 dark:text-primary-400 dark:hover:text-primary-300 font-medium"
                                        on:click={() => viewDetails(event)}
                                    >
                                        Details
                                    </button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<div class="mt-6 flex items-center justify-between border-t dark:border-gray-700 pt-6">
                <div class="text-sm text-gray-500">
                    Showing {(filters.offset || 0) + 1} to {Math.min((filters.offset || 0) + (filters.limit || 20), totalCount)} of {totalCount} entries
                </div>
				<div class="flex gap-2">
					<Button 
						variant="secondary" 
						size="sm" 
						disabled={(filters.offset || 0) === 0 || loading}
						on:click={() => handlePageChange(Math.max(0, (filters.offset || 0) - (filters.limit || 20)))}
					>
						Previous
					</Button>
					<Button 
						variant="secondary" 
						size="sm" 
						disabled={(filters.offset || 0) + (filters.limit || 20) >= totalCount || loading}
						on:click={() => handlePageChange((filters.offset || 0) + (filters.limit || 20))}
					>
						Next
					</Button>
				</div>
			</div>
		{/if}
	</Card>
</div>

<Modal bind:open={showDetailModal} title="Audit Event Details">
    {#if selectedEvent}
        <div class="space-y-4">
            <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                    <p class="text-gray-500">Event ID</p>
                    <p class="font-mono">{selectedEvent.id}</p>
                </div>
                <div>
                    <p class="text-gray-500">Timestamp</p>
                    <p>{formatDate(selectedEvent.timestamp)}</p>
                </div>
                <div>
                    <p class="text-gray-500">User</p>
                    <p>{selectedEvent.username} ({selectedEvent.user_id})</p>
                </div>
                <div>
                    <p class="text-gray-500">Action</p>
                    <p class="capitalize">{selectedEvent.action.replace('_', ' ')}</p>
                </div>
                <div>
                    <p class="text-gray-500">Resource</p>
                    <p>{selectedEvent.resource_type}: {selectedEvent.resource_name || 'N/A'}</p>
                </div>
                <div>
                    <p class="text-gray-500">Result</p>
                    <p>{selectedEvent.result}</p>
                </div>
            </div>
            
            {#if selectedEvent.error_message}
                <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300 text-sm">
                    <p class="font-bold mb-1">Error Message</p>
                    <p>{selectedEvent.error_message}</p>
                </div>
            {/if}
            
            <div>
                <p class="text-sm text-gray-500 mb-1">Client Info</p>
                <div class="p-3 bg-gray-50 dark:bg-gray-900 rounded border border-gray-200 dark:border-gray-800 text-xs font-mono space-y-1">
                    <p>IP: {selectedEvent.ip_address || 'Unknown'}</p>
                    <p class="break-all text-gray-400">UA: {selectedEvent.user_agent || 'Unknown'}</p>
                </div>
            </div>
            
            {#if selectedEvent.details}
                <div>
                    <p class="text-sm text-gray-500 mb-1">Additional Details</p>
                    <pre class="p-3 bg-gray-50 dark:bg-gray-900 rounded border border-gray-200 dark:border-gray-800 text-xs overflow-auto max-h-60">{JSON.stringify(selectedEvent.details, null, 2)}</pre>
                </div>
            {/if}
        </div>
    {/if}
    <div slot="footer">
        <Button on:click={() => showDetailModal = false}>Close</Button>
    </div>
</Modal>

<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { authStore } from '$lib/stores/auth';
    
    // Derived values for template compatibility
    $: token = $authStore.token;
    $: user = $authStore.user;
    import { fade } from 'svelte/transition';

    let requests: any[] = [];
    let loading = true;
    let error = '';

    // Filter
    let filter = 'pending'; // pending, all

    async function fetchRequests() {
        loading = true;
        try {
            const res = await fetch('/api/v1/access-requests', {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (res.ok) {
                requests = await res.json();
            } else {
                error = 'Failed to load requests';
            }
        } catch (e: any) { error = e.message; }
        finally { loading = false; }
    }

    async function updateStatus(req: any, status: string) {
        if (!confirm(`Are you sure you want to ${status} this request?`)) return;
        
        try {
            const res = await fetch(`/api/v1/access-requests/${req.id}`, {
                method: 'PUT',
                headers: { 
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status: status === 'Approve' ? 'Approved' : 'Rejected', comment: `${status} by admin` })
            });

            if (res.ok) {
                fetchRequests();
            }
        } catch (e: any) { alert('Action failed'); }
    }

    onMount(() => {
        fetchRequests();
    });
    
    $: filteredRequests = requests.filter(r => {
        if (filter === 'pending') return r.status === 'Pending';
        return true;
    });
</script>

<div class="page-header">
    <h1>Access Requests</h1>
    <div class="controls">
        <select bind:value={filter} class="filter-select">
            <option value="pending">Pending</option>
            <option value="all">All History</option>
        </select>
        <button class="icon-btn" on:click={fetchRequests} title="Refresh">
            <span class="material-icons">refresh</span>
        </button>
    </div>
</div>

{#if loading}
    <div class="loading">Loading requests...</div>
{:else if error}
    <div class="error">{error}</div>
{:else if filteredRequests.length === 0}
    <div class="empty-state">
        <span class="material-icons">inbox</span>
        <p>No {filter} requests found.</p>
    </div>
{:else}
    <div class="requests-list">
        {#each filteredRequests as req}
            <div class="request-card" transition:fade>
                <div class="req-content">
                    <div class="req-header">
                        <strong>Request for Asset ID: {req.asset_id.slice(0,8)}...</strong>
                        <span class="status-badge {req.status.toLowerCase()}">{req.status}</span>
                    </div>
                    <div class="req-meta">
                        <span>User ID: {req.user_id}</span>
                        <span>•</span>
                        <span>{new Date(req.requested_at).toLocaleDateString()}</span>
                    </div>
                    {#if req.reason}
                        <p class="reason">"{req.reason}"</p>
                    {/if}
                </div>

                {#if req.status === 'Pending' && (user?.role === 'TenantAdmin' || user?.role === 'Root')}
                    <div class="req-actions">
                        <button class="action-btn approve" on:click={() => updateStatus(req, 'Approve')}>
                            <span class="material-icons">check</span>
                        </button>
                        <button class="action-btn reject" on:click={() => updateStatus(req, 'Reject')}>
                            <span class="material-icons">close</span>
                        </button>
                    </div>
                {/if}
            </div>
        {/each}
    </div>
{/if}

<style>
    .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
    h1 { margin: 0; }
    
    .filter-select { padding: 0.5rem; border-radius: 8px; border: 1px solid var(--md-sys-color-outline); background: var(--md-sys-color-surface); color: inherit; }
    .icon-btn { background: none; border: none; cursor: pointer; color: inherit; }

    .requests-list { display: flex; flex-direction: column; gap: 1rem; max-width: 800px; margin: 0 auto; }
    .request-card {
        background: var(--md-sys-color-surface);
        padding: 1.5rem; border-radius: 12px;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        display: flex; justify-content: space-between; align-items: center;
    }

    .req-header { display: flex; align-items: center; gap: 1rem; margin-bottom: 0.5rem; }
    .status-badge { font-size: 0.75rem; padding: 2px 8px; border-radius: 4px; font-weight: bold; text-transform: uppercase; }
    .status-badge.pending { background: #fff3e0; color: #e65100; }
    .status-badge.approved { background: #e8f5e9; color: #2e7d32; }
    .status-badge.rejected { background: #ffebee; color: #c62828; }

    .req-meta { font-size: 0.875rem; color: var(--md-sys-color-on-surface-variant); margin-bottom: 0.5rem; }
    .reason { font-style: italic; color: var(--md-sys-color-on-surface); background: var(--md-sys-color-surface-container); padding: 0.5rem; border-radius: 4px; border-left: 3px solid var(--md-sys-color-outline); }

    .req-actions { display: flex; gap: 0.5rem; }
    .action-btn { width: 36px; height: 36px; border-radius: 50%; border: none; display: flex; justify-content: center; align-items: center; cursor: pointer; color: #fff; transition: transform 0.1s; }
    .action-btn:hover { transform: scale(1.1); }
    .action-btn.approve { background: var(--md-sys-color-primary); }
    .action-btn.reject { background: var(--md-sys-color-error); }

    .empty-state { text-align: center; padding: 3rem; opacity: 0.5; }
    .empty-state .material-icons { font-size: 3rem; margin-bottom: 1rem; }
</style>

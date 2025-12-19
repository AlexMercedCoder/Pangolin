<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { user, token } from '$lib/stores/auth';

    let commits: any[] = [];
    let branchName = $page.url.searchParams.get('branch');
    let catalogName = $page.url.searchParams.get('catalog') || 'default';
    let error = "";

    onMount(async () => {
        if (!$user || !branchName) return;
        
        const res = await fetch(`/api/v1/branches/${branchName}/commits`, {
            headers: { 'Authorization': `Bearer ${$token}` }
        });
        
        if (res.ok) {
            commits = await res.json();
        } else {
            error = "Failed to load commits";
        }
    });

    function formatDate(ts: number) {
        return new Date(ts).toLocaleString();
    }
</script>

<div class="p-6">
    <div class="flex items-center mb-6">
        <a href="/branches" class="text-blue-400 hover:text-blue-300 mr-4">&larr; Back to Branches</a>
        <h1 class="text-2xl font-bold">Commits for {branchName}</h1>
    </div>

    {#if error}
        <p class="text-red-500 mb-4">{error}</p>
    {/if}

    <div class="bg-gray-800 rounded overflow-hidden">
        <table class="w-full text-left">
            <thead class="bg-gray-700">
                <tr>
                    <th class="p-4">Commit ID</th>
                    <th class="p-4">Author</th>
                    <th class="p-4">Message</th>
                    <th class="p-4">Date</th>
                    <th class="p-4">Operations</th>
                </tr>
            </thead>
            <tbody class="divide-y divide-gray-700">
                {#each commits as commit}
                    <tr class="hover:bg-gray-750">
                        <td class="p-4 font-mono text-sm text-gray-300">{commit.id}</td>
                        <td class="p-4">{commit.author}</td>
                        <td class="p-4">{commit.message}</td>
                        <td class="p-4 text-gray-400">{formatDate(commit.timestamp)}</td>
                        <td class="p-4 text-sm">
                            {#each commit.operations as op}
                                <div class:text-green-400={op.op_type === 'Put'} class:text-red-400={op.op_type === 'Delete'}>
                                    {op.op_type}: {op.path}
                                </div>
                            {/each}
                        </td>
                    </tr>
                {/each}
                {#if commits.length === 0 && !error}
                    <tr>
                        <td colspan="5" class="p-8 text-center text-gray-500">No commits found</td>
                    </tr>
                {/if}
            </tbody>
        </table>
    </div>
</div>

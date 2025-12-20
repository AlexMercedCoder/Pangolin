<script lang="ts">
    import { onMount } from 'svelte';
    import Card from '$lib/components/ui/Card.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import { authStore } from '$lib/stores/auth';
    import { get } from 'svelte/store';

    export let catalogName: string = 'my_catalog';
    
    // Dynamic Values
    let apiUrl = '';
    let tenantId = '';
    let token = '';

    // Tabs
    const tabs = [
        { id: 'vending', label: 'Credential Vending' },
        { id: 's3', label: 'AWS S3' },
        { id: 'minio', label: 'S3-Compatible (MinIO)' },
        { id: 'azure', label: 'Azure' },
        { id: 'gcp', label: 'GCP' },
    ];
    let activeTab = 'vending';
    let copySuccess = false;

    onMount(() => {
        // Hydrate values
        const auth = get(authStore);
        apiUrl = window.location.origin + '/api/v1/catalogs/' + catalogName;
        tenantId = auth.user?.tenant_id || '<TENANT_ID>';
        token = '<YOUR_ACCESS_TOKEN>'; 
    });

    $: codeSnippet = getSnippet(activeTab, apiUrl, tenantId, token);

    function getSnippet(scenario: string, uri: string, tenant: string, tok: string) {
        const commonHeaders = `
    "uri": "${uri}",
    "header.X-Pangolin-Tenant": "${tenant}",
    "token": "${tok}",`;

        switch(scenario) {
            case 'vending':
                return `from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", **{${commonHeaders}
    "header.X-Iceberg-Access-Delegation": "vended-credentials"
})`;
            case 's3': 
                return `catalog = load_catalog("pangolin", **{${commonHeaders}
    "s3.region": "us-east-1",
    "s3.access-key-id": "<AWS_ACCESS_KEY>",
    "s3.secret-access-key": "<AWS_SECRET_KEY>"
})`;
            case 'minio':
                return `catalog = load_catalog("pangolin", **{${commonHeaders}
    "s3.endpoint": "http://minio:9000",
    "s3.access-key-id": "<MINIO_USER>",
    "s3.secret-access-key": "<MINIO_PASSWORD>"
})`;
            case 'azure':
                return `catalog = load_catalog("pangolin", **{${commonHeaders}
    "adls.account.name": "<ACCOUNT_NAME>",
    "adls.account.key": "<ACCOUNT_KEY>"
})`;
            case 'gcp':
                return `catalog = load_catalog("pangolin", **{${commonHeaders}
    "gcs.project-id": "<PROJECT_ID>",
    "gcs.oauth2.token": "<GCS_TOKEN>"
})`;
            default: return '';
        }
    }

    async function copyCode() {
        try {
            await navigator.clipboard.writeText(codeSnippet);
            copySuccess = true;
            setTimeout(() => copySuccess = false, 2000);
        } catch (err) {
            console.error('Failed to copy', err);
        }
    }
</script>

<Card>
    <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <span class="material-icons text-blue-500">code</span>
            Getting Started with PyIceberg
        </h3>
        <a 
            href="https://py.iceberg.apache.org/" 
            target="_blank" 
            class="text-sm text-primary-600 hover:text-primary-700 hover:underline"
        >
            View Docs
        </a>
    </div>

    <!-- Tabs -->
    <div class="border-b border-gray-200 dark:border-gray-700 mb-4">
        <nav class="-mb-px flex space-x-4 overflow-x-auto" aria-label="Tabs">
            {#each tabs as tab}
                <button
                    on:click={() => activeTab = tab.id}
                    class="whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm transition-colors
                        {activeTab === tab.id
                            ? 'border-primary-500 text-primary-600 dark:text-primary-400'
                            : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'}"
                >
                    {tab.label}
                </button>
            {/each}
        </nav>
    </div>

    <div class="relative group">
        <pre class="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto font-mono text-sm leading-relaxed">{codeSnippet}</pre>
        
        <button
            on:click={copyCode}
            class="absolute top-2 right-2 p-2 bg-gray-800 text-gray-300 rounded hover:bg-gray-700 opacity-0 group-hover:opacity-100 transition-opacity"
            title="Copy to clipboard"
        >
            {#if copySuccess}
                <span class="material-icons text-green-400 text-sm">check</span>
            {:else}
                <span class="material-icons text-sm">content_copy</span>
            {/if}
        </button>
    </div>
    
    <div class="mt-4 text-sm text-gray-600 dark:text-gray-400">
        <p>
            Replace <code>&lt;YOUR_ACCESS_TOKEN&gt;</code> with a token generated from your <a href="/profile/tokens" class="text-primary-600 hover:underline">Profile</a>.
        </p>
    </div>
</Card>

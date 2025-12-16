<script lang="ts">
    export let field: any;
    export let depth: number = 0;
    
    $: isNested = field.type === 'struct' && field.fields;
    $: isArray = field.type === 'list' && field.element;
    $: isMap = field.type === 'map' && field.key && field.value;
</script>

<div class="schema-field" style="margin-left: {depth * 20}px">
    <div class="flex items-start gap-2 py-1">
        <span class="font-mono text-sm text-blue-400 dark:text-blue-300">
            {field.name || 'element'}
        </span>
        <span class="text-gray-500">:</span>
        <span class="font-mono text-sm text-green-400 dark:text-green-300">
            {field.type}
        </span>
        {#if field.required}
            <span class="text-xs text-red-400">required</span>
        {/if}
        {#if field.doc}
            <span class="text-xs text-gray-400 italic">- {field.doc}</span>
        {/if}
    </div>
    
    {#if isNested}
        <div class="ml-4 border-l border-gray-700 pl-2">
            {#each field.fields as subfield}
                <svelte:self field={subfield} depth={depth + 1} />
            {/each}
        </div>
    {:else if isArray}
        <div class="ml-4 border-l border-gray-700 pl-2">
            <svelte:self field={{...field.element, name: 'element'}} depth={depth + 1} />
        </div>
    {:else if isMap}
        <div class="ml-4 border-l border-gray-700 pl-2">
            <div class="text-xs text-gray-500">Key:</div>
            <svelte:self field={{...field.key, name: 'key'}} depth={depth + 1} />
            <div class="text-xs text-gray-500 mt-1">Value:</div>
            <svelte:self field={{...field.value, name: 'value'}} depth={depth + 1} />
        </div>
    {/if}
</div>

<style>
    .schema-field {
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    }
</style>

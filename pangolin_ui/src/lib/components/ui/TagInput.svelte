<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { slide } from 'svelte/transition';

    export let tags: string[] = [];
    export let placeholder = "Add a tag...";
    export let maxTags: number | undefined = undefined;

    const dispatch = createEventDispatcher();
    let inputValue = "";

    function addTag() {
        const tag = inputValue.trim();
        if (!tag) return;
        
        if (tags.includes(tag)) {
            inputValue = "";
            return;
        }

        if (maxTags && tags.length >= maxTags) {
            return;
        }

        tags = [...tags, tag];
        inputValue = "";
        dispatch('change', tags);
    }

    function removeTag(index: number) {
        tags = tags.filter((_, i) => i !== index);
        dispatch('change', tags);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
            e.preventDefault();
            addTag();
        } else if (e.key === 'Backspace' && inputValue === "" && tags.length > 0) {
            removeTag(tags.length - 1);
        }
    }
</script>

<div class="tag-input-container">
    <div class="chips">
        {#each tags as tag, i (tag)}
            <div class="chip" transition:slide|local={{ axis: 'x', duration: 200 }}>
                <span>{tag}</span>
                <button type="button" class="remove-btn" on:click={() => removeTag(i)} aria-label="Remove tag">
                    <span class="material-icons">close</span>
                </button>
            </div>
        {/each}
        
        <input 
            type="text" 
            bind:value={inputValue} 
            on:keydown={handleKeydown}
            on:blur={addTag}
            {placeholder}
            class="input-field"
            disabled={maxTags !== undefined && tags.length >= maxTags}
        />
    </div>
</div>

<style>
    .tag-input-container {
        background: var(--md-sys-color-surface-container);
        border: 1px solid var(--md-sys-color-outline);
        border-radius: 8px;
        padding: 0.5rem;
        transition: border-color 0.2s;
    }

    .tag-input-container:focus-within {
        border-color: var(--md-sys-color-primary);
        outline: 2px solid var(--md-sys-color-primary-container);
    }

    .chips {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        align-items: center;
    }

    .chip {
        display: flex;
        align-items: center;
        background: var(--md-sys-color-secondary-container);
        color: var(--md-sys-color-on-secondary-container);
        padding: 4px 8px;
        border-radius: 6px;
        font-size: 0.875rem;
        font-weight: 500;
    }

    .remove-btn {
        background: none;
        border: none;
        padding: 0;
        margin-left: 4px;
        cursor: pointer;
        display: flex;
        align-items: center;
        color: inherit;
        opacity: 0.7;
    }

    .remove-btn:hover {
        opacity: 1;
    }

    .remove-btn .material-icons {
        font-size: 14px;
    }

    .input-field {
        flex: 1;
        min-width: 120px;
        border: none;
        outline: none;
        background: transparent;
        color: var(--md-sys-color-on-surface);
        padding: 4px 0;
        font-size: 0.875rem;
        font-family: inherit;
    }
    
    .input-field::placeholder {
        color: var(--md-sys-color-on-surface-variant);
        opacity: 0.7;
    }
</style>

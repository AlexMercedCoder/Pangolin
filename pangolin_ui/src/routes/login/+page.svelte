<script lang="ts">
    import { user, token } from '$lib/auth';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';

    let username = '';
    let password = '';
    let error = '';
    let loading = false;

    async function handleLogin() {
        loading = true;
        error = '';
        
        try {
            const response = await fetch('/api/v1/users/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ username, password })
            });

            if (response.ok) {
                const data = await response.json();
                user.set(data.user);
                token.set(data.token);
                goto('/');
            } else {
                const err = await response.json();
                error = err.error || 'Login failed';
            }
        } catch (e) {
            error = 'Network error occurred';
            console.error(e);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        // If already logged in, redirect home
        /* user.subscribe(u => {
            if (u) goto('/');
        }); */
    });
</script>

<div class="login-container">
    <div class="login-card">
        <div class="logo-section">
            <img src="/logo.png" alt="Pangolin Logo" class="logo" />
            <h1>Pangolin</h1>
            <p>Lakehouse Catalog</p>
        </div>

        {#if error}
            <div class="error-message" transition:fade>
                <span class="material-icons">error_outline</span>
                {error}
            </div>
        {/if}

        <form on:submit|preventDefault={handleLogin} class="login-form">
            <div class="input-group">
                <label for="username">Username</label>
                <input 
                    type="text" 
                    id="username" 
                    bind:value={username} 
                    placeholder="Enter your username"
                    required
                    disabled={loading}
                />
            </div>

            <div class="input-group">
                <label for="password">Password</label>
                <input 
                    type="password" 
                    id="password" 
                    bind:value={password} 
                    placeholder="Enter your password"
                    required
                    disabled={loading}
                />
            </div>

            <button type="submit" class="login-btn" disabled={loading}>
                {#if loading}
                    <span class="spinner"></span>
                {:else}
                    Sign In
                {/if}
            </button>
        </form>
        
        <div class="footer">
             <p>v0.1.0 Alpha</p>
        </div>
    </div>
</div>

<style>
    .login-container {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: 100vh;
        background-color: var(--md-sys-color-background);
        padding: 1rem;
    }

    .login-card {
        background-color: var(--md-sys-color-surface);
        padding: 2.5rem;
        border-radius: 28px; /* Material 3 styled */
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
        width: 100%;
        max-width: 400px;
        color: var(--md-sys-color-on-surface);
        transition: transform 0.2s;
    }

    .logo-section {
        text-align: center;
        margin-bottom: 2rem;
    }

    .logo {
        width: 100px;
        height: 100px;
        margin-bottom: 1rem;
        object-fit: contain;
    }

    h1 {
        margin: 0;
        font-size: 2rem;
        font-weight: 500;
        color: var(--md-sys-color-primary);
        font-family: 'Roboto', sans-serif;
    }

    p {
        margin: 0.5rem 0 0;
        opacity: 0.8;
    }

    .input-group {
        margin-bottom: 1.5rem;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        font-size: 0.875rem;
        font-weight: 500;
        color: var(--md-sys-color-primary);
    }

    input {
        width: 100%;
        padding: 12px 16px;
        border: 1px solid var(--md-sys-color-secondary); /* Using secondary as border for flair */
        border-radius: 8px; /* M3 small rounding */
        background-color: var(--md-sys-color-surface);
        color: var(--md-sys-color-on-surface);
        font-size: 1rem;
        box-sizing: border-box; /* Fix width issues */
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    input:focus {
        outline: none;
        border-color: var(--md-sys-color-primary);
        box-shadow: 0 0 0 2px var(--md-sys-color-primary-container, rgba(25, 118, 210, 0.2));
    }

    .login-btn {
        width: 100%;
        padding: 12px;
        background-color: var(--md-sys-color-primary);
        color: var(--md-sys-color-on-primary);
        border: none;
        border-radius: 100px; /* Pillow shape for buttons */
        font-size: 1rem;
        font-weight: 500;
        cursor: pointer;
        transition: opacity 0.2s, transform 0.1s;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .login-btn:hover:not(:disabled) {
        opacity: 0.9;
        transform: translateY(-1px);
    }

    .login-btn:active:not(:disabled) {
        transform: translateY(0);
    }

    .login-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .error-message {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background-color: var(--md-sys-color-error-container, #ffebee);
        color: var(--md-sys-color-on-error-container, #b71c1c); /* Fallback */
        padding: 0.75rem;
        border-radius: 8px;
        margin-bottom: 1.5rem;
        font-size: 0.875rem;
    }

    .footer {
        text-align: center;
        margin-top: 2rem;
        font-size: 0.75rem;
        opacity: 0.6;
    }

    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid #ffffff;
        border-top-color: transparent;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }
</style>

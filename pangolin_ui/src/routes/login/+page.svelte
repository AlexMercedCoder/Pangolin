<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth';
	import { authApi } from '$lib/api/auth';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';

	let username = '';
	let password = '';
	let error = '';
	let loading = false;

	import { page } from '$app/stores';

	onMount(async () => {
        // Check for OAuth callback token
        const token = $page.url.searchParams.get('token');
        if (token) {
            loading = true;
            const result = await authStore.handleOAuthLogin(token);
            if (result.success) {
                // Clear URL params
                window.history.replaceState({}, document.title, '/');
                goto('/');
                return;
            } else {
                error = result.error || 'OAuth login failed';
            }
            loading = false;
        }

		// If already authenticated, redirect to dashboard
		if ($authStore.isAuthenticated) {
			goto('/');
		}
	});

	async function handleLogin() {
		error = '';
		loading = true;

		try {
			const result = await authStore.login(username, password);
			if (result.success) {
				goto('/');
			} else {
				error = result.error || 'Login failed. Please check your credentials.';
			}
		} catch (e: any) {
			error = e.message || 'Login failed. Please check your credentials.';
		} finally {
			loading = false;
		}
	}

	function handleKeyPress(e: any) {
		if (e.key === 'Enter') {
			handleLogin();
		}
	}
</script>

<svelte:head>
	<title>Login - Pangolin</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-primary-50 to-secondary-50 dark:from-gray-900 dark:to-gray-800 px-4">
	<div class="w-full max-w-md">
		<!-- Logo and Title -->
		<div class="text-center mb-8">
			<h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-2">Pangolin</h1>
			<p class="text-gray-600 dark:text-gray-400">Lakehouse Catalog Management</p>
		</div>

		<Card>
			<h2 class="text-2xl font-semibold text-gray-900 dark:text-white mb-6">Sign In</h2>

			{#if error}
				<div class="mb-4 p-3 bg-error-50 dark:bg-error-900 border border-error-200 dark:border-error-700 rounded-lg">
					<p class="text-sm text-error-700 dark:text-error-200">{error}</p>
				</div>
			{/if}

			<form on:submit|preventDefault={handleLogin} class="space-y-4">
				<Input
					label="Username"
					type="text"
					bind:value={username}
					placeholder="Enter your username"
					required
					disabled={loading}
					on:keypress={handleKeyPress}
				/>

				<Input
					label="Password"
					type="password"
					bind:value={password}
					placeholder="Enter your password"
					required
					disabled={loading}
					on:keypress={handleKeyPress}
				/>

				<Button
					type="submit"
					variant="primary"
					fullWidth
					{loading}
					disabled={loading || !username || !password}
				>
					{loading ? 'Signing in...' : 'Sign In'}
				</Button>
			</form>

			<!-- OAuth Options -->
			<div class="mt-6">
				<div class="relative">
					<div class="absolute inset-0 flex items-center">
						<div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
					</div>
					<div class="relative flex justify-center text-sm">
						<span class="px-2 bg-white dark:bg-gray-800 text-gray-500">Or continue with</span>
					</div>
				</div>

				<div class="mt-4 grid grid-cols-2 gap-3">
                    {#each ['google', 'github', 'microsoft', 'okta'] as provider}
                        <Button
                            variant="secondary"
                            fullWidth
                            on:click={() => {
                                // Direct navigation to backend OAuth endpoint
                                // We use current origin + /login as redirect_uri to come back here and process token
                                const redirectUri = encodeURIComponent(`${window.location.origin}/login`);
                                // Assuming API is on relative path /api or simple /oauth if proxied, 
                                // but safe to use the env var from client.ts logic if possible.
                                // For now, let's assume relative path /oauth/authorize/ if served from same domain,
                                // or we need the API_URL.
                                // Let's use a cleaner approach: construct URL relative to current location if proxied, 
                                // or strictly use configured API URL.
                                // Since we don't have API_URL exposed easily here without importing from env,
                                // let's try relative path assuming proxy setup or modify client to expose it.
                                // Fallback to hardcoded for now or use window.location.origin if in dev.
                                const apiUrl = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080';
                                window.location.href = `${apiUrl}/oauth/authorize/${provider}?redirect_uri=${redirectUri}`;
                            }}
                        >
                            <span class="capitalize">{provider}</span>
                        </Button>
                    {/each}
				</div>
			</div>
		</Card>

		<p class="mt-4 text-center text-sm text-gray-600 dark:text-gray-400">
			Don't have an account? Contact your administrator.
		</p>
	</div>
</div>

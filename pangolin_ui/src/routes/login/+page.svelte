<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth';
	import { authApi } from '$lib/api/auth';
	import { API_URL, apiClient } from '$lib/api/client';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';

	let username = '';
	let password = '';
	let error = '';
	let loading = false;
	let showTenantSelector = false;
	let tenantId = '';

	// B33: this page used to render all four provider buttons unconditionally,
	// because `GET /api/v1/oauth/providers` did not exist and returned 404. The
	// buttons for providers the operator had never configured went nowhere. The
	// endpoint now exists and is public, so only configured providers render.
	let oauthProviders: string[] = [];

	import { page } from '$app/stores';

	onMount(async () => {
        // Which OAuth providers this deployment actually has configured.
        apiClient
            .get<{ providers: string[] }>('/api/v1/oauth/providers')
            .then((res) => {
                oauthProviders = res.error ? [] : (res.data?.providers ?? []);
            });

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
            loading = false;
        }

        // Check for tenant_id to auto-populate
        const paramTenant = $page.url.searchParams.get('tenant_id') || $page.url.searchParams.get('tenant');
        if (paramTenant) {
            tenantId = paramTenant;
            showTenantSelector = true;
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
            // Check for No-Auth Root Login trigger
            if (username === 'root' && password === 'root' && !$authStore.authEnabled) {
                const result = authStore.loginNoAuth();
                if (result.success) {
                    goto('/');
                    return;
                }
            }
            
            
            // Default to Zero GUID if not specified when not using tenant selector
            // This is primarily for the default tenant admin case (0000...)
			let tenantIdToSend = showTenantSelector && tenantId ? tenantId : null;
            
            // Conditional Default Logic:
            // - No Auth Mode (!authEnabled): Default to Zero UUID for legacy behavior
            // - Auth Mode (authEnabled): Default to null to allow Root Env Var fallback
            if (!showTenantSelector && !tenantIdToSend) {
                if (!$authStore.authEnabled) {
                     tenantIdToSend = '00000000-0000-0000-0000-000000000000';
                }
            }
            
			const result = await authStore.login(username, password, tenantIdToSend);
			if (result.success) {
				goto('/');
			} else {
                console.error('Login failed:', result.error);
				error = result.error || 'Login failed. Please check your credentials.';
			}
		} catch (e: any) {
            console.error('Login exception:', e);
			error = e.message || 'An unexpected error occurred during login.';
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

			<!-- Tenant-Scoped Login Option -->
			<div class="flex items-center space-x-2">
				<input
					type="checkbox"
					id="tenant-login"
					bind:checked={showTenantSelector}
					class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
					disabled={loading}
				/>
				<label
					for="tenant-login"
					class="text-sm font-medium text-gray-700 dark:text-gray-300 cursor-pointer"
				>
					Tenant-specific login
				</label>
			</div>

			{#if showTenantSelector}
				<Input
					label="Tenant ID"
					type="text"
					bind:value={tenantId}
					placeholder="Enter tenant ID (UUID)"
					disabled={loading}
					on:keypress={handleKeyPress}
				/>
				<p class="text-xs text-gray-500 dark:text-gray-400 -mt-2">
					Enter the tenant ID if you have duplicate usernames across tenants
				</p>
			{/if}

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
			{#if oauthProviders.length}
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
                    {#each oauthProviders as provider (provider)}
                        <Button
                            variant="secondary"
                            fullWidth
                            on:click={() => {
                                // B31: this read `import.meta.env.VITE_API_URL` -
                                // a *third* spelling of the API base URL, after
                                // client.ts's PUBLIC_API_URL and the compose
                                // files' VITE_API_URL, none of which agreed. It
                                // now uses the one resolved base URL.
                                const redirectUri = encodeURIComponent(`${window.location.origin}/login`);
                                window.location.href = `${API_URL}/oauth/authorize/${provider}?redirect_uri=${redirectUri}`;
                            }}
                        >
                            <span class="capitalize">{provider}</span>
                        </Button>
                    {/each}
				</div>
			</div>
			{/if}
		</Card>

		<p class="mt-4 text-center text-sm text-gray-600 dark:text-gray-400">
			Don't have an account? Contact your administrator.
		</p>
	</div>
</div>

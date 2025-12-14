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

	onMount(() => {
		// If already authenticated, redirect to dashboard
		if ($authStore.isAuthenticated) {
			goto('/');
		}
	});

	async function handleLogin() {
		error = '';
		loading = true;

		try {
			const response = await authApi.login({ username, password });
			authStore.setUser(response.user, response.token);
			goto('/');
		} catch (e: any) {
			error = e.message || 'Login failed. Please check your credentials.';
		} finally {
			loading = false;
		}
	}

	function handleKeyPress(e: KeyboardEvent) {
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

			<!-- OAuth Options (placeholder for future) -->
			<div class="mt-6">
				<div class="relative">
					<div class="absolute inset-0 flex items-center">
						<div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
					</div>
					<div class="relative flex justify-center text-sm">
						<span class="px-2 bg-white dark:bg-gray-800 text-gray-500">Or continue with</span>
					</div>
				</div>

				<div class="mt-4 text-center text-sm text-gray-500">
					OAuth providers coming soon
				</div>
			</div>
		</Card>

		<p class="mt-4 text-center text-sm text-gray-600 dark:text-gray-400">
			Don't have an account? Contact your administrator.
		</p>
	</div>
</div>

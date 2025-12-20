<script lang="ts">
	import { onMount } from 'svelte';
	import { systemConfigApi, type SystemSettings } from '$lib/api/system_config';
	import { notifications } from '$lib/stores/notifications';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	
	let settings: SystemSettings = {
		allow_public_signup: false,
		default_warehouse_bucket: '',
		default_retention_days: 30,
		smtp_host: '',
		smtp_port: 587,
		smtp_user: '',
		smtp_password: ''
	};
	
	let loading = false;
	let saving = false;

	onMount(async () => {
		loadSettings();
	});

	async function loadSettings() {
		loading = true;
		try {
			const data = await systemConfigApi.getSettings();
			settings = { ...settings, ...data };
		} catch (error: any) {
			notifications.error(`Failed to load system settings: ${error.message}`);
		} finally {
			loading = false;
		}
	}

	async function handleSave() {
		saving = true;
		try {
			// Convert types if necessary (inputs might be strings)
            const payload = {
                ...settings,
                default_retention_days: Number(settings.default_retention_days),
                smtp_port: Number(settings.smtp_port)
            };
            
			const updated = await systemConfigApi.updateSettings(payload);
			settings = { ...settings, ...updated };
			notifications.success('System settings updated successfully');
		} catch (error: any) {
			notifications.error(`Failed to update settings: ${error.message}`);
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>System Settings - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">System Configuration</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Manage global settings for the Pangolin instance.
		</p>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-6">
			<!-- General Settings -->
			<Card>
				<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">General Settings</h2>
				<div class="space-y-4">
					<div class="flex items-center">
						<input 
							id="allow_public_signup" 
							type="checkbox" 
							bind:checked={settings.allow_public_signup}
							class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded"
						>
						<label for="allow_public_signup" class="ml-2 block text-sm text-gray-900 dark:text-gray-300">
							Allow Public Signup
						</label>
					</div>

					<Input 
						label="Default Warehouse Bucket" 
						bind:value={settings.default_warehouse_bucket} 
						placeholder="e.g. s3://pangolin-warehouse"
						helpText="Base S3 path for new warehouses if not specified."
					/>

					<Input 
						label="Default Retention Days" 
						type="number"
						bind:value={settings.default_retention_days} 
						placeholder="30"
					/>
				</div>
			</Card>

			<!-- SMTP Configuration -->
			<Card>
				<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">SMTP Configuration</h2>
				<p class="text-sm text-gray-500 mb-4">Configure email settings for system notifications and invites.</p>
				
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<Input 
						label="SMTP Host" 
						bind:value={settings.smtp_host} 
						placeholder="smtp.example.com"
					/>
					
					<Input 
						label="SMTP Port" 
						type="number"
						bind:value={settings.smtp_port} 
						placeholder="587"
					/>
					
					<Input 
						label="SMTP User" 
						bind:value={settings.smtp_user} 
						placeholder="user@example.com"
					/>
					
					<Input 
						label="SMTP Password" 
						type="password"
						bind:value={settings.smtp_password} 
						placeholder="••••••••"
					/>
				</div>
			</Card>

			<div class="flex justify-end">
				<Button 
					variant="primary" 
					size="lg"
					disabled={saving}
					on:click={handleSave}
				>
					{saving ? 'Saving...' : 'Save Configuration'}
				</Button>
			</div>
		</div>
	{/if}
</div>

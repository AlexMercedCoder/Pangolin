<script lang="ts">
    import { goto } from '$app/navigation';
    import { user, token } from '$lib/auth';

    let username = '';
    let password = '';
    let error = '';

    async function handleLogin() {
        const credentials = btoa(`${username}:${password}`);
        const authHeader = `Basic ${credentials}`;

        try {
            // Verify credentials by calling a protected endpoint
            const res = await fetch('http://localhost:3000/api/v1/tenants', {
                headers: {
                    'Authorization': authHeader
                }
            });

            if (res.ok) {
                token.set(authHeader);
                user.set({ username, roles: ['Root'] }); // Assuming Root for now
                goto('/');
            } else {
                error = 'Invalid credentials';
            }
        } catch (e) {
            error = 'Login failed';
        }
    }
</script>

<div class="flex items-center justify-center min-h-screen bg-gray-100">
    <div class="p-8 bg-white rounded shadow-md w-96">
        <h1 class="mb-6 text-2xl font-bold text-center">Pangolin Login</h1>
        
        {#if error}
            <div class="p-2 mb-4 text-red-700 bg-red-100 rounded">{error}</div>
        {/if}

        <form on:submit|preventDefault={handleLogin}>
            <div class="mb-4">
                <label class="block mb-2 text-sm font-bold text-gray-700" for="username">Username</label>
                <input class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="username" type="text" bind:value={username} placeholder="Username">
            </div>
            <div class="mb-6">
                <label class="block mb-2 text-sm font-bold text-gray-700" for="password">Password</label>
                <input class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="password" type="password" bind:value={password} placeholder="******************">
            </div>
            <div class="flex items-center justify-between">
                <button class="px-4 py-2 font-bold text-white bg-blue-500 rounded hover:bg-blue-700 focus:outline-none focus:shadow-outline" type="submit">
                    Sign In
                </button>
            </div>
        </form>
    </div>
</div>
